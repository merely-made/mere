/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The Genet-native runner that owns app state and the retained view tree.
//!
//! Meristem provides view diffing and message routing but does not schedule
//! work. [`GenetAppRunner`] owns state, the root node, and the rebuild cadence
//! that turns a state change into DOM mutations.
//!
//! [`GenetAppRunner`] is that owner, kept deliberately thin: it depends only
//! on `meristem` and this backend, never on Livery/Buckram or presentation. A host
//! crate such as `pelt-desktop` drives the render side over the
//! [`ScriptedDom`](genet_scripted_dom::ScriptedDom) the runner mutates.
//!
//! Native event dispatch takes a hit [`NodeId`] (the `point → NodeId` half is
//! the host's responsibility), walks the node's ancestor chain, and routes
//! events through Meristem's message cycle. The timer tick that drives
//! [`update`](GenetAppRunner::update), platform input, relayout, rendering, and
//! presentation remain host concerns.
//!
//! **One state, N windows** (one-state-N-windows design, step 2): the per-tree
//! half of the runner — the dom, ctx, retained view, focus, and pointer
//! capture — lives in the crate-internal [`RunnerTree`], with app state and
//! the view-producing logic threaded in per call. [`GenetAppRunner`] is one
//! state + one tree (this file, public API unchanged);
//! [`GenetMultiRunner`](crate::GenetMultiRunner) is one state + N trees,
//! each a projection of the same state into its own `ScriptedDom`.

use core::marker::PhantomData;
use std::cell::RefCell;
use std::rc::Rc;

use genet_events::{EventInit, Invoker, Registry, TargetKey, build_path};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LayoutDomMut, LocalName, Namespace};
use meristem::{DynMessage, MessageCtx, MessageResult, View, ViewId};

use crate::context::FocusRequest;
use crate::events::{NativeOwner, Routing};
use crate::{
    CompositionEvent, DomHandle, FocusEvent, FocusPhase, GenetCtx, GenetElement, GenetElementMut,
    HoverEvent, Key, KeyEvent, NamedKey, PointerButton, PointerClick, PointerEvent, Propagation,
    ValueEvent, WheelEvent,
};

/// Which end of a runner's focusable set traversal left by, with
/// [focus exits](GenetAppRunner::set_focus_exits) on.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FocusExit {
    /// Past the last focusable: the host's next stop comes after this runner.
    Forward,
    /// Before the first focusable: the host's next stop comes before it.
    Backward,
}

/// What one [`walk`](RunnerTree::walk) produced.
struct Walk<Action> {
    /// The actions that reached the root.
    actions: Vec<Action>,
    /// How many listeners ran.
    invoked: usize,
}

impl<Action> Default for Walk<Action> {
    fn default() -> Self {
        Self {
            actions: Vec::new(),
            invoked: 0,
        }
    }
}

/// The per-tree half of a runner: one `ScriptedDom` target, its [`GenetCtx`]
/// (its listeners are keyed by this dom's `NodeId`s), the retained view
/// tree, and the per-window interaction state (focus, pointer capture, the
/// cancellation flag). App state and the view-producing logic are *not* here —
/// they are threaded into every method, so one state can drive one tree
/// ([`GenetAppRunner`]) or many ([`GenetMultiRunner`](crate::GenetMultiRunner)).
pub(crate) struct RunnerTree<State, V, Action>
where
    State: 'static,
    Action: 'static,
    V: View<State, Action, GenetCtx, Element = GenetElement>,
{
    dom: DomHandle,
    ctx: GenetCtx,
    view: V,
    view_state: V::ViewState,
    /// The retained root element produced by the current `view`. Its `node`
    /// stays attached under [`mount`](Self::mount) for the tree's lifetime.
    root: GenetElement,
    /// The node the tree's root attaches under. The document root for an
    /// ordinary single-tree/N-doms projection ([`build`](Self::build)); a
    /// **window-root element** for a forest-dom projection
    /// ([`build_at`](Self::build_at)), so N windows share one document as
    /// sibling subtrees and a cross-window `move_before` stays intra-document.
    mount: NodeId,
    /// The currently focused node, if any. A click sets this to the nearest
    /// focusable (key-handler-bearing) ancestor of the click target, or clears
    /// it to `None` when the click lands outside any focusable element.
    /// [`dispatch_key`](Self::dispatch_key) walks from here.
    focus: Option<NodeId>,
    /// Explicit view focus requests outrank a stock tab's deferred arrow request.
    focus_request_serial: u64,
    /// The element capturing the current pointer drag, if any. Set by
    /// [`dispatch_pointer_down`](Self::dispatch_pointer_down) to the nearest
    /// pointer-handler ancestor of the press, consulted by
    /// [`dispatch_pointer_move`](Self::dispatch_pointer_move) to route moves, and
    /// cleared by [`dispatch_pointer_up`](Self::dispatch_pointer_up). So a drag
    /// keeps reaching the element it started on even if the cursor leaves it.
    pointer_capture: Option<NodeId>,
    pointer_owner: Option<(i32, PointerButton)>,
    /// Latest admitted IME preedit, owned by the focused target.
    ime_preedit: Option<(NodeId, String)>,
    /// True only while dispatching the synthetic commit used to prepare a
    /// subtree move. A prevented synthetic commit keeps the interaction live
    /// through its rebuild; native commits retain their normal semantics.
    preparing_ime_commit: bool,
    /// One prepared semantic commit awaiting a possible native echo. Correlation
    /// is bounded to `(target, text)` and expires on the next non-commit key,
    /// focus change, or lifecycle event; a host session token is needed to
    /// distinguish every possible late echo from a later identical commit.
    prepared_ime_commit: Option<(NodeId, String)>,
    /// Whether the most recent dispatch had its default action prevented (a
    /// handler called `prevent_default` on the shared [`Propagation`](crate::Propagation)
    /// cell). Read via `default_prevented` after dispatch to gate the host's
    /// own default action.
    last_default_prevented: bool,
    /// Whether focus traversal stops at the ends of the focusable set and
    /// leaves the tree, instead of wrapping. Off unless a host turns it on.
    focus_exits: bool,
    /// The end traversal left by, waiting for the host to take it.
    focus_exit: Option<FocusExit>,
    /// A file request a view filed in the latest build or rebuild, waiting
    /// for the host to take it.
    file_request: Option<crate::FileRequest>,
    /// Runs the listeners of other owners that share this tree's registry, in
    /// the walks the runner drives. Without one they are skipped.
    foreign: Option<Box<dyn Invoker>>,
    phantom: PhantomData<fn() -> (State, Action)>,
}

impl<State, V, Action> RunnerTree<State, V, Action>
where
    State: 'static,
    Action: 'static,
    V: View<State, Action, GenetCtx, Element = GenetElement>,
{
    /// The file request a view filed, once, for the host to answer.
    pub(crate) fn take_file_request(&mut self) -> Option<crate::FileRequest> {
        self.file_request
            .take()
            .filter(|request| self.node_is_live(request.node))
    }

    /// Route a host's file answer to the [`open_file`](crate::open_file) view
    /// on `node`. A node that is gone or asked for nothing is ignored.
    pub(crate) fn dispatch_file(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        node: NodeId,
        event: crate::FileEvent,
    ) -> Vec<Action> {
        self.scrub_dead_interaction_handles();
        // A file answer completes an already-started host operation. Keep
        // delivering it to the live owner even if it became hidden while the
        // chooser was open; this is not new input admission.
        if !self.node_is_live(node) {
            return Vec::new();
        }
        let Some(path) = self.ctx.file_handler(node).map(<[ViewId]>::to_vec) else {
            return Vec::new();
        };
        let mut actions = Vec::new();
        {
            let Self {
                view,
                view_state,
                root,
                dom,
                mount,
                ..
            } = self;
            let mut message = MessageCtx::new(path, DynMessage::new(event));
            let element = GenetElementMut {
                node: &mut root.node,
                dom: dom.clone(),
                parent: Some(*mount),
            };
            if let MessageResult::Action(action) =
                view.message(view_state, &mut message, element, state)
            {
                actions.push(action);
            }
        }
        self.rebuild(logic, state);
        actions
    }

    /// Walk `target`'s event path through the event layer, in DOM propagation
    /// order (**capture → target → bubble**), and route each Cambium listener it
    /// reaches down its view path as `View::message`. The shared core of
    /// [`dispatch_click`](Self::dispatch_click),
    /// [`dispatch_key`](Self::dispatch_key) and the focus transitions.
    ///
    /// `genet_events::dispatch` owns the walk, with this tree as the
    /// [`Invoker`]: a listener fires in exactly one phase, a node's listeners run
    /// in registration order, and `prop`'s flags are read back after each, so
    /// `stop_propagation` ends the walk at the node and `stop_immediate_propagation`
    /// ends it at the listener. The rebuild is the caller's, after the walk,
    /// as before: no rebuild between listeners.
    ///
    /// Calls that reached [`NativeOwner`] during the walk (a nested native
    /// dispatch, queued rather than run mid-walk) run last, once the walk has
    /// unwound; see [`NativeOwner`] for the ordering this departs from.
    fn walk(
        &mut self,
        state: &mut State,
        target: NodeId,
        event_type: &str,
        bubbles: bool,
        prop: Option<&Propagation>,
        message: &dyn Fn() -> DynMessage,
    ) -> Walk<Action> {
        let events = self.ctx.events();
        // The fast path: nobody listens for this type, so no path to build.
        if !events.borrow().has_type(event_type) {
            return Walk::default();
        }
        let path = build_path(
            &self.dom.borrow(),
            TargetKey::Node(target),
            None,
            event_type,
            true,
        );
        let init = EventInit {
            event_type: event_type.to_owned(),
            bubbles,
            stop_propagation: prop.is_some_and(Propagation::stopped),
            stop_immediate: prop.is_some_and(Propagation::stopped_immediately),
            canceled: prop.is_some_and(Propagation::default_prevented),
            activation_event: false,
        };
        let Self {
            ctx,
            view,
            view_state,
            root,
            dom,
            mount,
            foreign,
            ..
        } = self;
        let mut routing = Routing {
            view: &*view,
            view_state,
            root,
            dom: &*dom,
            mount: *mount,
            state,
            listeners: ctx.listeners(),
            foreign: foreign.as_deref_mut(),
            message,
            prop,
            actions: Vec::new(),
            invoked: 0,
        };
        genet_events::dispatch(&events, path, &init, &mut routing);
        routing.run_queued();
        Walk {
            actions: routing.actions,
            invoked: routing.invoked,
        }
    }

    pub(crate) fn set_foreign_invoker(&mut self, invoker: Option<Box<dyn Invoker>>) {
        self.foreign = invoker;
    }

    pub(crate) fn native_owner(&self) -> NativeOwner {
        self.ctx.native_owner()
    }

    pub(crate) fn default_prevented(&self) -> bool {
        self.last_default_prevented
    }

    /// Route an accessibility numeric value to its exact registered element.
    pub(crate) fn dispatch_value(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        node: NodeId,
        event: ValueEvent,
    ) -> Vec<Action> {
        self.scrub_dead_interaction_handles();
        if !event.value.is_finite()
            || !node_is_accessibility_admitted_in_dom(&self.dom.borrow(), node)
        {
            return Vec::new();
        }
        let Some(path) = self.ctx.value_handler(node).map(<[ViewId]>::to_vec) else {
            return Vec::new();
        };
        let mut actions = Vec::new();
        {
            let Self {
                view,
                view_state,
                root,
                dom,
                mount,
                ..
            } = self;
            let mut message = MessageCtx::new(path, DynMessage::new(event));
            let element = GenetElementMut {
                node: &mut root.node,
                dom: dom.clone(),
                parent: Some(*mount),
            };
            if let MessageResult::Action(action) =
                view.message(view_state, &mut message, element, state)
            {
                actions.push(action);
            }
        }
        self.rebuild(logic, state);
        actions
    }

    /// Route a wheel/scroll notch to the nearest ancestor of `target` (including
    /// itself) carrying an [`on_wheel`](crate::on_wheel) handler.
    pub(crate) fn dispatch_wheel(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        target: NodeId,
        event: WheelEvent,
    ) -> Vec<Action> {
        self.last_default_prevented = false;
        self.scrub_dead_interaction_handles();
        if !self.node_is_admitted(target) {
            return Vec::new();
        }
        let resolved = {
            let dom = self.dom.borrow();
            let mut current = Some(target);
            let mut found = None;
            while let Some(node) = current {
                if self.ctx.wheel_handler(node).is_some() {
                    found = Some(node);
                    break;
                }
                current = dom.parent(node);
            }
            found
        };
        match resolved {
            Some(node) => self.route_wheel(logic, state, node, event),
            None => Vec::new(),
        }
    }

    /// The nearest ancestor of `hit` (including itself) carrying an
    /// [`on_wheel`](crate::on_wheel) handler.
    pub(crate) fn wheel_target(&self, hit: NodeId) -> Option<NodeId> {
        if !self.node_is_admitted(hit) {
            return None;
        }
        let dom = self.dom.borrow();
        let mut current = Some(hit);
        while let Some(node) = current {
            if self.ctx.wheel_handler(node).is_some() {
                return Some(node);
            }
            current = dom.parent(node);
        }
        None
    }

    /// Route a [`WheelEvent`] down `node`'s registered wheel path through the
    /// faithful message cycle, then rebuild. Mirrors
    /// [`route_pointer`](Self::route_pointer).
    fn route_wheel(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        node: NodeId,
        event: WheelEvent,
    ) -> Vec<Action> {
        let Some(path) = self.ctx.wheel_handler(node).map(<[ViewId]>::to_vec) else {
            return Vec::new();
        };
        let mut actions = Vec::new();
        {
            let Self {
                view,
                view_state,
                root,
                dom,
                mount,
                ..
            } = self;
            let mut msg = MessageCtx::new(path, DynMessage::new(event.clone()));
            let mut_ref = GenetElementMut {
                node: &mut root.node,
                dom: dom.clone(),
                parent: Some(*mount),
            };
            if let MessageResult::Action(a) = view.message(view_state, &mut msg, mut_ref, state) {
                actions.push(a);
            }
        }
        self.last_default_prevented = event.prop.default_prevented();
        self.rebuild(logic, state);
        actions
    }

    /// The shared document handle the tree mutates.
    pub(crate) fn dom(&self) -> DomHandle {
        self.dom.clone()
    }

    /// The DOM node of the current root element.
    pub(crate) fn root(&self) -> NodeId {
        self.root.node
    }
}

/// Owns the app state, the view-producing logic, and one retained view tree,
/// rebuilding the [`ScriptedDom`] whenever the state changes.
///
/// `Logic: FnMut(&State) -> V` reads state and produces the root view. Its
/// element is always the uniform [`GenetElement`].
///
/// `Action` is the root view's action type. When a handler returns an action that no
/// parent [`map_action`](meristem::map_action) consumes, it surfaces as a
/// [`MessageResult::Action`] which [`dispatch_click`](Self::dispatch_click)
/// collects and returns. The common `Action = ()` case is the no-op it was — a
/// `()` action carries nothing to observe.
pub struct GenetAppRunner<State, Logic, V, Action = ()>
where
    State: 'static,
    Action: 'static,
    Logic: FnMut(&State) -> V,
    V: View<State, Action, GenetCtx, Element = GenetElement>,
{
    state: State,
    logic: Logic,
    tree: RunnerTree<State, V, Action>,
}

/// Append `node`'s focusable descendants (including itself), in document
/// pre-order, to `out`. Focusable = carries a key handler ([`GenetCtx::is_focusable`]).
fn collect_focusables(dom: &ScriptedDom, ctx: &GenetCtx, node: NodeId, out: &mut Vec<NodeId>) {
    // Hidden, parked, and accessibility-hidden ancestors exclude their entire
    // subtree from DOM-order focus traversal, even when listener records remain.
    if !node_is_accessibility_admitted_in_dom(dom, node) {
        return;
    }
    if ctx.is_focusable(node) {
        out.push(node);
    }
    for child in dom.dom_children(node) {
        collect_focusables(dom, ctx, child, out);
    }
}

fn is_true_token(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "1" | "yes" | "on"
    )
}

fn node_is_admitted_in_dom(dom: &ScriptedDom, node: NodeId) -> bool {
    if !dom.is_live(node) {
        return false;
    }
    let mut current = Some(node);
    while let Some(ancestor) = current {
        if attr(dom, ancestor, "hidden").is_some()
            || attr(dom, ancestor, "inert").is_some()
            || attr(dom, ancestor, "data-cambium-parked").is_some_and(is_true_token)
        {
            return false;
        }
        current = dom.parent(ancestor);
    }
    true
}

fn node_is_accessibility_admitted_in_dom(dom: &ScriptedDom, node: NodeId) -> bool {
    if !node_is_admitted_in_dom(dom, node) {
        return false;
    }
    let mut current = Some(node);
    while let Some(ancestor) = current {
        if attr(dom, ancestor, "aria-hidden").is_some_and(is_true_token) {
            return false;
        }
        current = dom.parent(ancestor);
    }
    true
}

fn attr<'a>(dom: &'a ScriptedDom, node: NodeId, name: &str) -> Option<&'a str> {
    dom.attribute(node, &Namespace::from(""), &LocalName::from(name))
}

fn semantic_activation_matches(dom: &ScriptedDom, node: NodeId, key: &Key) -> bool {
    let role = attr(dom, node, "role");
    match (role, key) {
        (Some("checkbox" | "switch"), Key::Named(NamedKey::Space)) => true,
        (Some("button"), Key::Named(NamedKey::Enter | NamedKey::Space)) => true,
        (None, Key::Named(NamedKey::Enter | NamedKey::Space)) => {
            dom.element_name(node)
                .is_some_and(|name| &*name.local == "button")
                || !dom
                    .element_name(node)
                    .is_some_and(|name| matches!(name.local.as_ref(), "input" | "textarea"))
        },
        _ => false,
    }
}

#[cfg(test)]
#[path = "runner/tests.rs"]
mod tests;

#[path = "runner/app_runner.rs"]
mod app_runner;

#[path = "runner/keyboard.rs"]
mod keyboard;

#[path = "runner/pointer.rs"]
mod pointer;

#[path = "runner/lifecycle.rs"]
mod lifecycle;
