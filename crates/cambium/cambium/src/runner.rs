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
    /// Build the initial tree from `state` and attach its root under the
    /// document root of `dom` (the ordinary single-tree / N-doms path).
    pub(crate) fn build(
        dom: DomHandle,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
    ) -> Self {
        let doc_root = dom.borrow().document();
        Self::build_at(dom, doc_root, logic, state)
    }

    /// Build the initial tree and attach its root under `mount` — the
    /// **forest-dom** path (F1: a runner tree mounts at a node, not the
    /// document root), so several projections can share one document as
    /// sibling window-root subtrees. `build` is the `mount == document()`
    /// case. The retained root, rebuild, and teardown all stay scoped to
    /// `mount`, so a mutation or teardown in one window never reaches another.
    pub(crate) fn build_at(
        dom: DomHandle,
        mount: NodeId,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
    ) -> Self {
        let ctx = GenetCtx::new(dom.clone());
        Self::build_in(dom, mount, ctx, logic, state)
    }

    /// [`build_at`](Self::build_at) over a context the caller made, whose event
    /// registry a host may share with another owner.
    pub(crate) fn build_in(
        dom: DomHandle,
        mount: NodeId,
        mut ctx: GenetCtx,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
    ) -> Self {
        let view = logic(state);
        let (root, view_state) = view.build(&mut ctx, state);

        // Attach the produced root under the mount node (append).
        dom.borrow_mut().insert_before(mount, root.node, None);
        let focus = match ctx.take_focus_request() {
            Some(FocusRequest::Focus(node)) if node_is_admitted_in_dom(&dom.borrow(), node) => {
                Some(node)
            },
            _ => None,
        };
        let file_request = ctx
            .take_file_request()
            .filter(|request| dom.borrow().is_live(request.node));
        ctx.set_interaction_snapshot(focus, None, None);

        Self {
            dom,
            ctx,
            view,
            view_state,
            root,
            mount,
            focus,
            focus_request_serial: 0,
            pointer_capture: None,
            pointer_owner: None,
            ime_preedit: None,
            preparing_ime_commit: false,
            prepared_ime_commit: None,
            last_default_prevented: false,
            focus_exits: false,
            focus_exit: None,
            file_request,
            foreign: None,
            phantom: PhantomData,
        }
    }

    /// The node this tree's root attaches under (its window-root, or the
    /// document root for a non-forest tree).
    pub(crate) fn mount(&self) -> NodeId {
        self.mount
    }

    /// Re-run the logic against the current state and diff the produced view
    /// into the retained one.
    ///
    /// The disjoint-field borrows matter: `rebuild` needs `&prev_view`,
    /// `&mut view_state`, `&mut ctx`, the root `Mut`, and `&mut state` all at
    /// once, so the fields are destructured into separate `&mut`s. The root
    /// `GenetElementMut` is constructed exactly as `Harness::rebuild` does —
    /// borrowing `root.node` so a view *could* swap the root node, and cloning
    /// the shared `dom` handle.
    pub(crate) fn rebuild(&mut self, logic: &mut impl FnMut(&State) -> V, state: &mut State) {
        self.sync_interaction_snapshot();
        let next = logic(state);

        // Disjoint borrows: each field separately so `rebuild`'s argument set
        // does not alias `self`.
        let Self {
            ctx,
            view,
            view_state,
            root,
            dom,
            mount,
            ..
        } = self;

        let mut_ref = GenetElementMut {
            node: &mut root.node,
            dom: dom.clone(),
            // The root is attached under `mount` (the document root, or a
            // window-root in the forest dom), so a type-changing `AnyView`
            // root can swap its node there via `replace_inner`.
            parent: Some(*mount),
        };
        next.rebuild(view, view_state, ctx, mut_ref, state);

        // The freshly diffed view becomes the retained `prev` for the next tick.
        *view = next;

        // Tear down any portable child parked during this rebuild and not
        // claimed within it — its key really left every portable list in this
        // tree, so it gets a real teardown and its (still-attached) node is
        // removed. Per-tree draining also confines parked elements to their
        // own dom: a cross-dom adoption is impossible by construction.
        // (moveBefore plan S5.)
        self.ctx.drain_nursery();

        // A node can remain live while an ordinary view hides or inerts an
        // ancestor. Deliver lifecycle cleanup through the retained old route
        // before the generic scrub drops those handles. The callbacks run only
        // after the root rebuild has unwound; if one routes, rebuild once to
        // publish any local-state changes.
        if self.cleanup_inadmitted_interactions(state) {
            self.rebuild(logic, state);
            return;
        }

        // Focus and pointer capture are retained NodeIds, so a rebuild that
        // replaces or removes their element can retire either handle. The DOM
        // dangle contract makes `is_live` the only safe read in that state.
        // Clear them at the publication boundary before the host can ask for
        // the next dispatch target or captured-element rect.
        self.scrub_dead_interaction_handles();
        if let Some(request) = self.ctx.take_focus_request() {
            self.focus_request_serial = self.focus_request_serial.wrapping_add(1);
            match request {
                FocusRequest::Focus(node) if self.node_is_admitted(node) => {
                    if self.focus != Some(node) {
                        self.prepared_ime_commit = None;
                        self.ime_preedit = None;
                    }
                    self.focus = Some(node);
                },
                FocusRequest::Blur(node) if self.focus == Some(node) => {
                    self.focus = None;
                    self.prepared_ime_commit = None;
                    self.ime_preedit = None;
                },
                _ => {},
            }
        }
        if let Some(request) = self.ctx.take_file_request()
            && self.node_is_live(request.node)
        {
            self.file_request = Some(request);
        }
        self.sync_interaction_snapshot();
    }

    fn sync_interaction_snapshot(&mut self) {
        let preedit = self
            .ime_preedit
            .as_ref()
            .map(|(node, _)| *node)
            .filter(|node| self.focus == Some(*node));
        self.ctx
            .set_interaction_snapshot(self.focus, preedit, self.pointer_capture);
    }

    /// Refresh retained event routes after a host moves this subtree to a new
    /// placement path. This deliberately uses the stored view and state; it
    /// does not call the logic closure or recreate the component tree.
    pub(crate) fn refresh_routes(&mut self, state: &mut State) {
        self.sync_interaction_snapshot();
        let Self {
            ctx,
            view,
            view_state,
            root,
            dom,
            mount,
            ..
        } = self;
        let mut_ref = GenetElementMut {
            node: &mut root.node,
            dom: dom.clone(),
            parent: Some(*mount),
        };
        ctx.with_route_refresh(|ctx| {
            view.rebuild(view, view_state, ctx, mut_ref, state);
        });
        self.scrub_dead_interaction_handles();
        self.sync_interaction_snapshot();
    }

    /// Finish Cambium-owned interaction state before a host reparents a live
    /// subtree. Pointer capture receives a real Cancel, and an active focused
    /// IME preedit receives a synthetic Commit through its old retained route.
    /// This does not ask the operating system to finalize its IME candidate;
    /// hosts must still suppress a matching late native commit. A prevented
    /// synthetic Commit leaves the preedit active, so the host must check
    /// [`subtree_move_has_active_interaction`](Self::subtree_move_has_active_interaction)
    /// and refuse the relocation.
    pub(crate) fn prepare_subtree_move(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        subtree: NodeId,
    ) -> Vec<Action> {
        if !self.node_is_live(subtree) {
            return Vec::new();
        }
        self.sync_interaction_snapshot();
        let mut actions = Vec::new();
        if let Some((target, text)) = self.ime_preedit.clone()
            && self.focus == Some(target)
            && self.is_descendant_or_self(subtree, target)
            && self.node_is_admitted(target)
        {
            let commit = KeyEvent::new(Key::Composition(CompositionEvent::Commit(text.clone())));
            let was_preparing = self.preparing_ime_commit;
            self.preparing_ime_commit = true;
            actions.extend(self.dispatch_key(logic, state, commit.clone()));
            self.preparing_ime_commit = was_preparing;
            if commit.prop.default_prevented() {
                self.ime_preedit = Some((target, text));
                self.prepared_ime_commit = None;
            } else {
                self.ime_preedit = None;
                self.prepared_ime_commit = Some((target, text));
            }
        }
        if self
            .pointer_capture
            .is_some_and(|node| self.is_descendant_or_self(subtree, node))
        {
            let (pointer_id, button) = self.pointer_owner.unwrap_or((0, PointerButton::Primary));
            let mut cancel = PointerEvent::new(crate::PointerPhase::Cancel, (0.0, 0.0), (0.0, 0.0))
                .with_button(button);
            cancel.pointer_id = pointer_id;
            actions.extend(self.dispatch_pointer_up(logic, state, cancel));
        }
        self.sync_interaction_snapshot();
        actions
    }

    pub(crate) fn subtree_move_has_active_interaction(&self, subtree: NodeId) -> bool {
        self.pointer_capture
            .is_some_and(|node| self.is_descendant_or_self(subtree, node))
            || self.ime_preedit.as_ref().is_some_and(|(node, _)| {
                self.focus == Some(*node) && self.is_descendant_or_self(subtree, *node)
            })
    }

    fn is_descendant_or_self(&self, ancestor: NodeId, node: NodeId) -> bool {
        let dom = self.dom.borrow();
        let mut current = Some(node);
        while let Some(candidate) = current {
            if candidate == ancestor {
                return true;
            }
            current = dom.parent(candidate);
        }
        false
    }

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

    fn node_is_live(&self, node: NodeId) -> bool {
        self.dom.borrow().is_live(node)
    }

    fn node_is_admitted(&self, node: NodeId) -> bool {
        node_is_admitted_in_dom(&self.dom.borrow(), node)
    }

    fn cleanup_inadmitted_interactions(&mut self, state: &mut State) -> bool {
        let mut routed = false;
        if let Some(node) = self.focus
            && self.node_is_live(node)
            && !self.node_is_admitted(node)
        {
            self.focus = None;
            self.prepared_ime_commit = None;
            self.ime_preedit = None;
            let (_, focus_routed) = self.route_focus_transition(state, Some(node), None);
            routed |= focus_routed;
        }
        if let Some(node) = self.pointer_capture
            && self.node_is_live(node)
            && !self.node_is_admitted(node)
        {
            let (pointer_id, button) = self.pointer_owner.unwrap_or((0, PointerButton::Primary));
            // Clear the old handles before invoking user code so a cleanup
            // callback cannot cause this publication boundary to cancel twice.
            self.pointer_capture = None;
            self.pointer_owner = None;
            routed |= self.route_pointer_without_rebuild(
                state,
                node,
                PointerEvent::new(crate::PointerPhase::Cancel, (0.0, 0.0), (0.0, 0.0))
                    .with_button(button)
                    .with_pointer_id(pointer_id),
            );
        }
        self.sync_interaction_snapshot();
        routed
    }

    fn route_pointer_without_rebuild(
        &mut self,
        state: &mut State,
        node: NodeId,
        event: PointerEvent,
    ) -> bool {
        let Some(path) = self.ctx.pointer_handler(node).map(<[ViewId]>::to_vec) else {
            return false;
        };
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
        let _ = view.message(view_state, &mut message, element, state);
        true
    }

    fn scrub_dead_interaction_handles(&mut self) {
        let dom = self.dom.borrow();
        if self
            .focus
            .is_some_and(|node| !dom.is_live(node) || !node_is_admitted_in_dom(&dom, node))
        {
            self.focus = None;
            self.prepared_ime_commit = None;
            self.ime_preedit = None;
        }
        if self
            .pointer_capture
            .is_some_and(|node| !dom.is_live(node) || !node_is_admitted_in_dom(&dom, node))
        {
            self.pointer_capture = None;
            self.pointer_owner = None;
        }
    }

    /// Tear the whole tree down (a projection being removed): run the retained
    /// view's teardown, then remove the root node from the document.
    pub(crate) fn teardown(mut self) {
        let Self {
            ctx,
            view,
            view_state,
            root,
            dom,
            mount,
            ..
        } = &mut self;
        let mut_ref = GenetElementMut {
            node: &mut root.node,
            dom: dom.clone(),
            parent: Some(*mount),
        };
        view.teardown(view_state, ctx, mut_ref);
        let root_node = root.node;
        dom.borrow_mut().remove(root_node);
        // A teardown can park portable children; nothing can ever claim them
        // (the tree is gone), so drain immediately.
        ctx.drain_nursery();
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

    /// Route a focus change through the retained view before the caller's
    /// publication rebuild. Lost is delivered first, then Gained. Each is a
    /// `blur` or `focus` event walked through the layer to its node; neither
    /// bubbles, so only that node's own observer (and any other owner's listeners
    /// there) hears it.
    fn route_focus_transition(
        &mut self,
        state: &mut State,
        old: Option<NodeId>,
        new: Option<NodeId>,
    ) -> (Vec<Action>, bool) {
        if old == new {
            return (Vec::new(), false);
        }
        let mut actions = Vec::new();
        let mut routed = false;
        for (node, event_type, phase) in [
            (old, "blur", FocusPhase::Lost),
            (new, "focus", FocusPhase::Gained),
        ] {
            let Some(node) = node else {
                continue;
            };
            let walk = self.walk(state, node, event_type, false, None, &|| {
                DynMessage::new(FocusEvent::new(phase))
            });
            routed |= walk.invoked > 0;
            actions.extend(walk.actions);
        }
        (actions, routed)
    }

    /// Dispatch a native pointer click that hit `target`. See
    /// [`GenetAppRunner::dispatch_click`] for the full contract; this is the
    /// per-tree implementation with state and logic threaded in.
    pub(crate) fn dispatch_click(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        target: NodeId,
        event: PointerClick,
    ) -> Vec<Action> {
        self.scrub_dead_interaction_handles();
        if !self.node_is_admitted(target) {
            self.last_default_prevented = false;
            return Vec::new();
        }

        // 1. Find the nearest focusable (key-handler-bearing or marked) ancestor
        //    for click-to-focus. Done under a short-lived shared borrow of
        //    `dom`, fully released before routing.
        let new_focus: Option<NodeId> = {
            let dom = self.dom.borrow();
            let mut current = Some(target);
            let mut focus = None;
            while let Some(node) = current {
                // The first (nearest) focusable ancestor wins; the walk runs
                // target → root, so the first hit is the deepest. Focusability
                // is phase-independent.
                if self.ctx.is_focusable(node) {
                    focus = Some(node);
                    break;
                }
                current = dom.parent(node);
            }
            focus
        };

        // Click-to-focus: focus the nearest focusable ancestor, or clear it when
        // the click landed outside any focusable element. Independent of whether
        // a click handler fired below.
        let old_focus = self.focus;
        self.focus = new_focus;
        if old_focus != new_focus {
            self.prepared_ime_commit = None;
            self.ime_preedit = None;
        }
        let (mut actions, focus_routed) = self.route_focus_transition(state, old_focus, new_focus);

        // 2. Walk the click through the event layer: capture pass first
        //    (root → target, capture listeners only), then bubble pass
        //    (target → root, the rest). A listener sits in exactly one phase,
        //    so it routes once. Any action that reaches the root (a
        //    `MessageResult::Action`) is collected; `Action = ()` collects
        //    nothing meaningful. A handler's stop_propagation ends the walk, and
        //    its prevent_default is read below.
        let walk = self.walk(state, target, "click", true, Some(&event.prop), &|| {
            DynMessage::new(event.clone())
        });
        actions.extend(walk.actions);

        // Record whether a handler prevented the default action — the host reads
        // this (default_prevented()) to gate its own default (the cancellation
        // seam's consumption point).
        self.last_default_prevented = event.prop.default_prevented();

        if walk.invoked == 0 {
            // Focus observers may still have mutated state even when the hit had
            // no click handler, so publish that transition before returning.
            if focus_routed {
                self.rebuild(logic, state);
            }
            return actions;
        }

        // 3. Rebuild so the handler's state mutation reaches the DOM — the same
        //    tail `update` runs.
        self.rebuild(logic, state);

        actions
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

    pub(crate) fn set_focus_exits(&mut self, exits: bool) {
        self.focus_exits = exits;
        if !exits {
            self.focus_exit = None;
        }
    }

    pub(crate) fn take_focus_exit(&mut self) -> Option<FocusExit> {
        self.focus_exit.take()
    }

    pub(crate) fn focus(&self) -> Option<NodeId> {
        self.focus
    }

    pub(crate) fn set_focus(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        node: Option<NodeId>,
    ) -> Vec<Action> {
        let next = node.filter(|&node| self.node_is_admitted(node));
        let old = self.focus;
        self.focus = next;
        if old != next {
            self.prepared_ime_commit = None;
            self.ime_preedit = None;
        }
        let (actions, _) = self.route_focus_transition(state, old, next);
        if old != next {
            self.rebuild(logic, state);
        }
        actions
    }

    /// Every focusable element, in document order.
    pub(crate) fn focusables(&self) -> Vec<NodeId> {
        let dom = self.dom.borrow();
        let mut out = Vec::new();
        collect_focusables(&dom, &self.ctx, self.mount, &mut out);
        out
    }

    /// Move focus to the next (`forward`) or previous focusable element in
    /// document order. The Tab-traversal default: a document engine has no
    /// built-in tab order, so the runner provides one over the focusable set
    /// (elements carrying a key handler, per [`GenetCtx::is_focusable`]) in DOM
    /// pre-order. With nothing focused, `forward` focuses the first focusable and
    /// backward the last. Rebuilds after (focus may drive `:focus` styling
    /// later).
    ///
    /// Past the last focusable (forward) or the first (backward), traversal
    /// wraps, or, with focus exits on, clears focus and records the exit for
    /// the host. With no focusable elements it is a no-op, or an exit.
    pub(crate) fn focus_traverse(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        forward: bool,
    ) {
        let exit = if forward {
            FocusExit::Forward
        } else {
            FocusExit::Backward
        };
        let focusables: Vec<NodeId> = self.focusables();
        if focusables.is_empty() {
            if self.focus_exits {
                self.focus_exit = Some(exit);
            }
            return;
        }
        let next = match self
            .focus
            .and_then(|f| focusables.iter().position(|&n| n == f))
        {
            Some(i) => {
                let len = focusables.len();
                let at_edge = if forward { i + 1 == len } else { i == 0 };
                if self.focus_exits && at_edge {
                    self.focus_exit = Some(exit);
                    self.set_focus(logic, state, None);
                    return;
                }
                if forward {
                    (i + 1) % len
                } else {
                    (i + len - 1) % len
                }
            },
            None => {
                if forward {
                    0
                } else {
                    focusables.len() - 1
                }
            },
        };
        self.set_focus(logic, state, Some(focusables[next]));
    }

    /// Dispatch a native key event to the focused node, then apply the
    /// runner-level keyboard defaults (Tab traversal, Enter/Space activation).
    /// See [`GenetAppRunner::dispatch_key`] for the full contract.
    pub(crate) fn dispatch_key(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        event: KeyEvent,
    ) -> Vec<Action> {
        // A fresh pass: the routing tail records the real cancellation value; a
        // pass that routes nothing leaves it false.
        self.last_default_prevented = false;
        event.prop.take_tab_focus();
        event.prop.set_key_target(None);
        self.scrub_dead_interaction_handles();
        let focus_request_serial = self.focus_request_serial;

        // With nothing focused, the only key carrying a runner default is Tab,
        // which enters the focusable set (no element to route to). Everything else
        // no-ops — no focused node means nowhere to send keys.
        let Some(focus) = self.focus else {
            if matches!(event.key, Key::Named(NamedKey::Tab)) {
                self.focus_traverse(logic, state, !event.mods.shift);
            }
            return Vec::new();
        };

        if let Key::Composition(CompositionEvent::Commit(text)) = &event.key
            && self
                .prepared_ime_commit
                .as_ref()
                .is_some_and(|(target, committed)| *target == focus && committed == text)
        {
            self.prepared_ime_commit = None;
            return Vec::new();
        }
        if !matches!(&event.key, Key::Composition(CompositionEvent::Commit(_))) {
            self.prepared_ime_commit = None;
        }

        // 1. Collect the ancestor chain (focus → … → document) once.
        let chain: Vec<NodeId> = {
            let dom = self.dom.borrow();
            let mut chain = Vec::new();
            let mut current = Some(focus);
            while let Some(node) = current {
                chain.push(node);
                current = dom.parent(node);
            }
            chain
        };

        // 2. Walk the key from the focused element through the event layer in
        //    propagation order (capture then bubble), exactly as `dispatch_click`.
        //    Tab is delivered here too (no longer swallowed pre-routing), so a
        //    view can handle it and `prevent_default` to override the traversal
        //    in step 3. Nothing runs for a focusable node with no `on_key` — a
        //    plain `focusable(button(..))`.
        let walk = self.walk(state, focus, "keydown", true, Some(&event.prop), &|| {
            DynMessage::new(event.clone())
        });
        let routed = walk.invoked > 0;
        let mut actions = walk.actions;
        if routed {
            match &event.key {
                Key::Composition(CompositionEvent::Preedit { text, .. }) => {
                    self.ime_preedit = Some((focus, text.clone()));
                },
                Key::Composition(CompositionEvent::Commit(_))
                    if self.preparing_ime_commit && event.prop.default_prevented() =>
                {
                    // Keep the tracked preedit live through the following
                    // rebuild. RetainedComposition must still refuse a move
                    // while a handler prevented this synthetic commit.
                },
                Key::Composition(
                    CompositionEvent::Enabled
                    | CompositionEvent::Commit(_)
                    | CompositionEvent::Disabled,
                ) => self.ime_preedit = None,
                _ => self.ime_preedit = None,
            }
        } else if !matches!(
            &event.key,
            Key::Composition(CompositionEvent::Preedit { .. })
        ) {
            self.ime_preedit = None;
        }
        if routed {
            // Record whether a handler prevented the default (gates step 3 + host).
            self.last_default_prevented = event.prop.default_prevented();
        }

        // 3. Runner-level key defaults, each suppressed when a handler prevented it
        //    (the escape hatches, G2.3).
        if !event.prop.default_prevented() {
            let activates = {
                let dom = self.dom.borrow();
                semantic_activation_matches(&dom, focus, &event.key)
            };
            match event.key {
                // Tab traverses focus across the focusable set, the runner-level
                // default a document engine otherwise lacks.
                Key::Named(NamedKey::Tab) => {
                    self.focus_traverse(logic, state, !event.mods.shift);
                    return actions; // focus_traverse rebuilt
                },
                // Enter/Space activate a focusable control that has a click handler
                // according to its native tag or ARIA role. Checkbox, switch,
                // roles use Space; buttons use Enter and Space. Radio groups
                // own their authored-widget behavior in Cambium.
                Key::Named(NamedKey::Enter | NamedKey::Space)
                    if self.ctx.has_click(focus) && !self.ctx.has_key(focus) && activates =>
                {
                    actions.extend(self.dispatch_click(
                        logic,
                        state,
                        focus,
                        PointerClick::at((0.0, 0.0)),
                    ));
                    // The key was consumed as activation: tell the host not to run
                    // its own default (e.g. Space scrolling the page).
                    self.last_default_prevented = true;
                    return actions; // dispatch_click rebuilt
                },
                _ => {},
            }
        }

        // 4. A handler ran but no runner default fired: rebuild so the handler's
        //    state change reaches the DOM. When nothing routed and no default
        //    fired, nothing changed, so the rebuild is skipped.
        if routed {
            self.rebuild(logic, state);
            let request = event.prop.take_tab_focus();
            if self.focus == Some(focus) && self.focus_request_serial == focus_request_serial {
                let next = request.and_then(|(origin, group, index)| {
                    let dom = self.dom.borrow();
                    let owned = |node| {
                        let mut current = Some(node);
                        while let Some(node) = current {
                            if !dom.is_live(node) {
                                return false;
                            }
                            if node == self.root.node {
                                return true;
                            }
                            current = dom.parent(node);
                        }
                        false
                    };
                    // The group was captured by the actual handler before diff.
                    // A nested runner or foreign host cannot contribute a seat.
                    if origin != focus
                        || !chain.contains(&origin)
                        || !owned(origin)
                        || !owned(group)
                        || dom.parent(origin) != Some(group)
                        || !self.ctx.has_key(origin)
                        || attr(&dom, origin, "role") != Some("tab")
                        || attr(&dom, group, "role") != Some("tablist")
                    {
                        return None;
                    }
                    let candidate = dom.dom_children(group).into_iter().nth(index)?;
                    (owned(candidate)
                        && self.ctx.is_focusable(candidate)
                        && self.ctx.has_key(candidate)
                        && attr(&dom, candidate, "role") == Some("tab")
                        && attr(&dom, candidate, "aria-selected") == Some("true")
                        && attr(&dom, candidate, "tabindex") == Some("0"))
                    .then_some(candidate)
                });
                if let Some(next) = next {
                    actions.extend(self.set_focus(logic, state, Some(next)));
                }
            }
        }

        actions
    }

    /// A button went down on `target`. Primary begins a drag capture;
    /// secondary routes one `Down` and may explicitly request capture. See
    /// [`GenetAppRunner::dispatch_pointer_down`].
    pub(crate) fn dispatch_pointer_down(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        target: NodeId,
        event: PointerEvent,
    ) -> Vec<Action> {
        // A fresh pass: reset the cancellation flag; route_pointer records the
        // real value when a handler runs, and a no-capture press leaves it false.
        self.last_default_prevented = false;
        self.scrub_dead_interaction_handles();
        if !self.node_is_admitted(target) {
            return Vec::new();
        }
        let captured = {
            let dom = self.dom.borrow();
            let mut current = Some(target);
            let mut found = None;
            while let Some(node) = current {
                if self.ctx.pointer_handler(node).is_some() {
                    found = Some(node);
                    break;
                }
                current = dom.parent(node);
            }
            found
        };
        // Primary and auxiliary gestures capture before routing. Secondary
        // capture is opt-in after routing and cannot steal an existing gesture.
        if matches!(
            event.button,
            PointerButton::Primary | PointerButton::Auxiliary
        ) {
            if self
                .pointer_owner
                .is_some_and(|owner| owner != (event.pointer_id, event.button))
            {
                return Vec::new();
            }
            self.pointer_capture = captured;
            self.pointer_owner = captured.map(|_| (event.pointer_id, event.button));
        }
        match captured {
            Some(node) => {
                let request = event.clone();
                let actions = self.route_pointer(logic, state, node, event);
                if request.button == PointerButton::Secondary
                    && request.capture_requested()
                    && self.pointer_capture.is_none()
                    && self.node_is_admitted(node)
                    && self.pointer_target(node) == Some(node)
                {
                    self.pointer_capture = Some(node);
                    self.pointer_owner = Some((request.pointer_id, request.button));
                }
                actions
            },
            None => Vec::new(),
        }
    }

    /// Route a `Move` to the element capturing the drag (if any).
    pub(crate) fn dispatch_pointer_move(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        event: PointerEvent,
    ) -> Vec<Action> {
        self.last_default_prevented = false;
        self.scrub_dead_interaction_handles();
        if self
            .pointer_owner
            .is_some_and(|(id, _)| id != event.pointer_id)
        {
            return Vec::new();
        }
        match self.pointer_capture {
            Some(node) => self.route_pointer(logic, state, node, event),
            None => Vec::new(),
        }
    }

    /// Route an `Up` to the capturing element and end the drag.
    pub(crate) fn dispatch_pointer_up(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        event: PointerEvent,
    ) -> Vec<Action> {
        self.last_default_prevented = false;
        self.scrub_dead_interaction_handles();
        if self.pointer_owner.is_some_and(|(id, button)| {
            id != event.pointer_id
                || (event.phase != crate::PointerPhase::Cancel && button != event.button)
        }) {
            return Vec::new();
        }
        self.pointer_owner = None;
        match self.pointer_capture.take() {
            Some(node) => self.route_pointer(logic, state, node, event),
            None => Vec::new(),
        }
    }

    pub(crate) fn pointer_capture_owner(&self) -> Option<(i32, PointerButton)> {
        self.pointer_owner
    }

    pub(crate) fn pointer_capture(&self) -> Option<NodeId> {
        self.pointer_capture
    }

    /// The nearest ancestor of `hit` (including itself) carrying an
    /// [`on_pointer`](crate::on_pointer) handler.
    pub(crate) fn pointer_target(&self, hit: NodeId) -> Option<NodeId> {
        if !self.node_is_admitted(hit) {
            return None;
        }
        let dom = self.dom.borrow();
        let mut current = Some(hit);
        while let Some(node) = current {
            if self.ctx.pointer_handler(node).is_some() {
                return Some(node);
            }
            current = dom.parent(node);
        }
        None
    }

    /// Route one host-computed hover transition to the nearest registered
    /// ancestor of `target`.
    pub(crate) fn dispatch_hover(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        target: NodeId,
        event: HoverEvent,
    ) -> Vec<Action> {
        self.last_default_prevented = false;
        self.scrub_dead_interaction_handles();
        if !self.node_is_admitted(target) {
            return Vec::new();
        }
        match self.hover_target(target) {
            Some(node) => self.route_hover(logic, state, node, event),
            None => Vec::new(),
        }
    }

    /// The nearest ancestor of `hit` (including itself) carrying a hover
    /// handler.
    pub(crate) fn hover_target(&self, hit: NodeId) -> Option<NodeId> {
        if !self.node_is_admitted(hit) {
            return None;
        }
        let dom = self.dom.borrow();
        let mut current = Some(hit);
        while let Some(node) = current {
            if self.ctx.hover_handler(node).is_some() {
                return Some(node);
            }
            current = dom.parent(node);
        }
        None
    }

    /// Route a hover event down one registered view path, then rebuild.
    fn route_hover(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        node: NodeId,
        event: HoverEvent,
    ) -> Vec<Action> {
        let Some(path) = self.ctx.hover_handler(node).map(<[ViewId]>::to_vec) else {
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
            let mut message = MessageCtx::new(path, DynMessage::new(event.clone()));
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
        self.last_default_prevented = event.prop.default_prevented();
        if !event.rebuild_is_deferred() {
            self.rebuild(logic, state);
        }
        actions
    }

    /// Route a [`PointerEvent`] down `node`'s registered pointer path through the
    /// faithful message cycle, then rebuild.
    fn route_pointer(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        node: NodeId,
        event: PointerEvent,
    ) -> Vec<Action> {
        let Some(path) = self.ctx.pointer_handler(node).map(<[ViewId]>::to_vec) else {
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
            // Clone into the message: the handler mutates its clone's shared
            // `Propagation` cell, and the original below reads back what it set.
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
        // Record this pointer pass's own cancellation (the host gates its default
        // drag behavior on it), mirroring dispatch_click / dispatch_key — not the
        // stale value left by an earlier click/key.
        self.last_default_prevented = event.prop.default_prevented();
        if !event.rebuild_is_deferred() {
            self.rebuild(logic, state);
        }
        actions
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

impl<State, Logic, V, Action> GenetAppRunner<State, Logic, V, Action>
where
    State: 'static,
    Action: 'static,
    Logic: FnMut(&State) -> V,
    V: View<State, Action, GenetCtx, Element = GenetElement>,
{
    /// Build the initial tree from `state` and attach its root under the
    /// document root.
    ///
    /// Mirrors Cambium's test harness: run the logic to
    /// get a view, `View::build` it into the `ScriptedDom`, then
    /// `insert_before(document, root, None)` to append it under the document.
    pub fn new(dom: DomHandle, mut logic: Logic, mut state: State) -> Self {
        let tree = RunnerTree::build(dom, &mut logic, &mut state);
        Self { state, logic, tree }
    }

    /// Build beneath a caller-owned live mount in the same document.
    ///
    /// The mount may move between window subtrees without rebuilding this
    /// runner. Its state, widget tree, focus and pointer capture stay owned by
    /// this runner; the receiving host must withdraw stale window input first.
    pub fn new_at(dom: DomHandle, mount: NodeId, mut logic: Logic, mut state: State) -> Self {
        assert!(dom.borrow().is_live(mount), "runner mount must be live");
        let tree = RunnerTree::build_at(dom, mount, &mut logic, &mut state);
        Self { state, logic, tree }
    }

    /// [`new_at`](Self::new_at) whose listeners register in `events`, an event
    /// registry the host shares with another owner (a script runtime's). Both
    /// owners' listeners on one node then interleave in registration order in one
    /// walk; the other owner's run through
    /// [`set_foreign_invoker`](Self::set_foreign_invoker), and without one they are
    /// skipped. Cambium's own are [`CAMBIUM_OWNER`](crate::CAMBIUM_OWNER)'s.
    pub fn with_events(
        dom: DomHandle,
        mount: NodeId,
        events: Rc<RefCell<Registry>>,
        mut logic: Logic,
        mut state: State,
    ) -> Self {
        assert!(dom.borrow().is_live(mount), "runner mount must be live");
        let ctx = GenetCtx::with_events(dom.clone(), events);
        let tree = RunnerTree::build_in(dom, mount, ctx, &mut logic, &mut state);
        Self { state, logic, tree }
    }

    /// Run the listeners of other owners in a shared registry through
    /// `invoker`, inside the walks of [`dispatch_click`](Self::dispatch_click) and
    /// [`dispatch_key`](Self::dispatch_key) and the focus transitions, at their
    /// place in registration order. A listener the layer marks `once` is
    /// removed before it runs, so without an invoker those would be lost: install
    /// it before the first dispatch.
    pub fn set_foreign_invoker(&mut self, invoker: Option<Box<dyn Invoker>>) {
        self.tree.set_foreign_invoker(invoker);
    }

    /// The endpoint other owners' dispatchers hand [`CAMBIUM_OWNER`](crate::CAMBIUM_OWNER)
    /// calls to. See [`NativeOwner`].
    pub fn native_owner(&self) -> NativeOwner {
        self.tree.native_owner()
    }

    /// Apply a state update, then rebuild the view tree against the new state.
    ///
    /// `f` is the externally-driven "message": it mutates the owned state. The
    /// runner then reruns the logic and diffs the produced view against the
    /// retained one, emitting `DomMutation`s into the `ScriptedDom`.
    pub fn update(&mut self, f: impl FnOnce(&mut State)) {
        f(&mut self.state);
        self.tree.rebuild(&mut self.logic, &mut self.state);
    }

    /// Refresh retained listener routes after an external host changes this
    /// runner's placement path, without evaluating application logic.
    pub fn refresh_routes(&mut self) {
        self.tree.refresh_routes(&mut self.state);
    }

    /// Prepare a subtree for synchronous relocation. Call this before changing
    /// its parent or placement metadata, and forward the returned actions to
    /// the product owner. This can dispatch pointer Cancel and a synthetic IME
    /// Commit through the subtree's current retained routes.
    pub fn prepare_subtree_move(&mut self, subtree: NodeId) -> Vec<Action> {
        self.tree
            .prepare_subtree_move(&mut self.logic, &mut self.state, subtree)
    }

    /// Whether relocating `subtree` now would discard an active Cambium
    /// pointer capture or focused IME preedit. A host can use this as a
    /// fail-closed check when it cannot call [`Self::prepare_subtree_move`].
    pub fn subtree_move_has_active_interaction(&self, subtree: NodeId) -> bool {
        self.tree.subtree_move_has_active_interaction(subtree)
    }

    /// Update retained paint state without rebuilding the semantic view tree.
    ///
    /// Use for host-driven animation whose changing geometry is painted by a
    /// separately refreshed leaf. The caller must refresh that leaf and request
    /// frames while it is active. Changes affecting DOM content or hit targets
    /// still require [`Self::update`], including reconciliation after animation.
    pub fn update_deferred<R>(&mut self, f: impl FnOnce(&mut State) -> R) -> R {
        f(&mut self.state)
    }

    /// Dispatch a native pointer click that hit `target`.
    ///
    /// `target` is the node Genet's hit-test resolved the pointer to (the
    /// `point → NodeId` half lives in the host's `hit_test_node`). This is the
    /// faithful-routing dispatch: no native handler registry of `Rc<dyn Fn>`,
    /// just the `meristem` message cycle the browser path also uses.
    ///
    /// Dispatch runs the full DOM propagation order, **capture → target →
    /// bubble**; a `.capture(true)` ancestor fires before a default (bubble)
    /// descendant/target, yielding the browser/`xilem_web` ordering.
    ///
    /// Returns the `Action`s that bubbled all the way to the root — i.e. any
    /// handler action no parent [`map_action`](meristem::map_action) absorbed.
    ///
    /// After routing and rebuilding, focus is set to the nearest ancestor of
    /// `target` (including `target` itself) that carries
    /// a key handler (is focusable); if none is found, focus is cleared. So
    /// clicking a focusable element focuses it and clicking elsewhere defocuses,
    /// independent of whether any *click* handler fired.
    pub fn dispatch_click(&mut self, target: NodeId, event: PointerClick) -> Vec<Action> {
        self.tree
            .dispatch_click(&mut self.logic, &mut self.state, target, event)
    }

    /// Dispatch a finite numeric value requested by an accessibility host.
    /// Invalid values and nodes without an [`on_value`](crate::on_value) handler
    /// are ignored.
    pub fn dispatch_value(&mut self, target: NodeId, event: ValueEvent) -> Vec<Action> {
        self.tree
            .dispatch_value(&mut self.logic, &mut self.state, target, event)
    }

    /// The file request a view filed since the host last asked, if any. The
    /// host shows its chooser for it and answers with
    /// [`dispatch_file`](Self::dispatch_file).
    pub fn take_file_request(&mut self) -> Option<crate::FileRequest> {
        self.tree.take_file_request()
    }

    /// Dispatch a host's answer to a file request: the chosen files, or none
    /// when the person cancelled. A test or scenario supplies one the same way.
    pub fn dispatch_file(&mut self, target: NodeId, event: crate::FileEvent) -> Vec<Action> {
        self.tree
            .dispatch_file(&mut self.logic, &mut self.state, target, event)
    }

    /// Whether the most recent dispatch had its default action prevented by a
    /// handler (`prevent_default` on the event). The host calls this right after
    /// dispatch to decide whether to run its own default action — navigation,
    /// caret movement, form activation, drag start. This is the native side of
    /// the converged cancellation contract (the JS side returns it from
    /// `dispatchEvent`); a handler that does not prevent leaves it `false`.
    pub fn default_prevented(&self) -> bool {
        self.tree.default_prevented()
    }

    /// The currently focused node, if any.
    ///
    /// Set by [`dispatch_click`](Self::dispatch_click) (click-to-focus) or
    /// [`set_focus`](Self::set_focus); read by [`dispatch_key`](Self::dispatch_key)
    /// as the root of its bubble walk.
    pub fn focus(&self) -> Option<NodeId> {
        self.tree.focus()
    }

    /// Set (or clear, with `None`) the focused node directly.
    ///
    /// The keyboard counterpart of a programmatic `element.focus()`. No
    /// validation that a live `node` is focusable: a test (or a host) may aim
    /// focus at any live node, and [`dispatch_key`](Self::dispatch_key) simply
    /// finds no key handler to route to if it is not focusable. A retired node
    /// clears focus under the DOM dangle contract.
    pub fn set_focus(&mut self, node: Option<NodeId>) {
        self.tree.set_focus(&mut self.logic, &mut self.state, node);
    }

    /// Move focus to the next (`forward`) or previous focusable element in
    /// document order, wrapping unless [focus exits](Self::set_focus_exits)
    /// are on.
    pub fn focus_traverse(&mut self, forward: bool) {
        self.tree
            .focus_traverse(&mut self.logic, &mut self.state, forward);
    }

    /// Stop traversal at the ends of the focusable set instead of wrapping.
    ///
    /// For a host that shows this runner beside other content: a Tab past the
    /// last focusable (or a Shift-Tab before the first) clears focus and
    /// records a [`FocusExit`], which the host takes with
    /// [`take_focus_exit`](Self::take_focus_exit) and answers by moving focus
    /// to whatever comes next in its own order. Without it, Tab cycles inside
    /// the runner and a keyboard user cannot leave. Off by default, which keeps
    /// a standalone app's wrapping order. Turning it off drops a pending exit.
    pub fn set_focus_exits(&mut self, exits: bool) {
        self.tree.set_focus_exits(exits);
    }

    /// The end the latest traversal left by, once. `None` when traversal
    /// stayed inside, or when focus exits are off.
    pub fn take_focus_exit(&mut self) -> Option<FocusExit> {
        self.tree.take_focus_exit()
    }

    /// Every focusable element, in document order — the set
    /// [`focus_traverse`](Self::focus_traverse) walks.
    ///
    /// Public because document order is not the only order worth having.
    /// A host with the laid-out geometry can move focus *spatially* (arrow keys
    /// through a grid), which document order cannot express and which the view
    /// layer cannot compute, having no layout. This is the half the runner
    /// knows; the host supplies the other half.
    pub fn focusables(&self) -> Vec<NodeId> {
        self.tree.focusables()
    }

    /// Dispatch a native key event to the focused node, then apply the
    /// runner-level keyboard defaults (Tab traversal, Enter/Space activation) the
    /// model leaves to the host — each overridable by a handler that calls
    /// `prevent_default` (the G2.3 escape hatches).
    pub fn dispatch_key(&mut self, event: KeyEvent) -> Vec<Action> {
        self.tree
            .dispatch_key(&mut self.logic, &mut self.state, event)
    }

    /// A button went down on `target`. A `Down` [`PointerEvent`] is routed to
    /// the nearest ancestor of `target` (including itself) carrying an
    /// [`on_pointer`](crate::on_pointer) handler. Returns the actions that
    /// bubbled to the root.
    ///
    /// For [`PointerButton::Primary`] that element also becomes the capture:
    /// subsequent [`dispatch_pointer_move`](Self::dispatch_pointer_move) /
    /// [`dispatch_pointer_up`](Self::dispatch_pointer_up) go to it until
    /// release. For [`PointerButton::Secondary`] the press is one-shot — the
    /// capture, whatever it was, is untouched, so a right press during a drag
    /// does not steal it and a right press outside one leaves nothing for a
    /// later release to find.
    ///
    /// `event.local` / `event.size` are the press point + element box in the
    /// captured element's coordinate space; the host computes them from the
    /// laid-out rect (the headless view layer has no layout).
    pub fn dispatch_pointer_down(&mut self, target: NodeId, event: PointerEvent) -> Vec<Action> {
        self.tree
            .dispatch_pointer_down(&mut self.logic, &mut self.state, target, event)
    }

    /// Route a `Move` to the element capturing the drag (if any). No-op when no
    /// drag is active.
    pub fn dispatch_pointer_move(&mut self, event: PointerEvent) -> Vec<Action> {
        self.tree
            .dispatch_pointer_move(&mut self.logic, &mut self.state, event)
    }

    /// Route an `Up` to the capturing element and end the drag (clearing
    /// capture). No-op when no drag is active.
    pub fn dispatch_pointer_up(&mut self, event: PointerEvent) -> Vec<Action> {
        self.tree
            .dispatch_pointer_up(&mut self.logic, &mut self.state, event)
    }

    /// The element currently capturing a pointer drag, if any. The host reads
    /// this between [`dispatch_pointer_down`](Self::dispatch_pointer_down) and
    /// `up` to know which element's rect to measure for the move's local coords.
    pub fn pointer_capture_owner(&self) -> Option<(i32, PointerButton)> {
        self.tree.pointer_owner
    }
    pub fn dispatch_pointer_cancel(&mut self, mut event: PointerEvent) -> Vec<Action> {
        event.phase = crate::PointerPhase::Cancel;
        self.tree
            .dispatch_pointer_up(&mut self.logic, &mut self.state, event)
    }

    pub fn pointer_capture(&self) -> Option<NodeId> {
        self.tree.pointer_capture()
    }

    /// The nearest ancestor of `hit` (including itself) carrying an
    /// [`on_pointer`](crate::on_pointer) handler — the element a press there
    /// would capture, resolved *without* starting a drag.
    pub fn pointer_target(&self, hit: NodeId) -> Option<NodeId> {
        self.tree.pointer_target(hit)
    }

    /// Route a hover Enter, Leave, or Move event from `target` to the nearest
    /// ancestor carrying [`on_hover`](crate::on_hover).
    ///
    /// The host owns hit testing and transition detection. When its hovered
    /// node changes it calls this once for Leave on the old hit and once for
    /// Enter on the new hit; Move uses the current hit.
    pub fn dispatch_hover(&mut self, target: NodeId, event: HoverEvent) -> Vec<Action> {
        self.tree
            .dispatch_hover(&mut self.logic, &mut self.state, target, event)
    }

    /// The nearest ancestor of `hit` (including itself) carrying an
    /// [`on_hover`](crate::on_hover) handler.
    pub fn hover_target(&self, hit: NodeId) -> Option<NodeId> {
        self.tree.hover_target(hit)
    }

    /// Route a wheel/scroll notch to the nearest ancestor of `target` (including
    /// itself) carrying an [`on_wheel`](crate::on_wheel) handler. Unlike a
    /// pointer drag there is no capture: each notch resolves its own target by
    /// the ancestor walk. Returns the actions that bubbled to the root.
    pub fn dispatch_wheel(&mut self, target: NodeId, event: WheelEvent) -> Vec<Action> {
        self.tree
            .dispatch_wheel(&mut self.logic, &mut self.state, target, event)
    }

    /// The nearest ancestor of `hit` (including itself) carrying an
    /// [`on_wheel`](crate::on_wheel) handler — the element a wheel there would
    /// scroll, resolved *without* dispatching.
    pub fn wheel_target(&self, hit: NodeId) -> Option<NodeId> {
        self.tree.wheel_target(hit)
    }

    /// The shared document handle the runner mutates.
    pub fn dom(&self) -> DomHandle {
        self.tree.dom()
    }

    /// The DOM node of the current root element (attached under the document
    /// root). Stable across `update`s unless a view swaps the root node.
    pub fn root(&self) -> NodeId {
        self.tree.root()
    }

    /// The current app state.
    pub fn state(&self) -> &State {
        &self.state
    }
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
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use genet_scripted_dom::{NodeId, ScriptedDom};
    use layout_dom_api::{DomMutation, LayoutDom, LayoutDomMut, NodeKind};

    use crate::{CompositionEvent, DomHandle, El, FocusPhase, el};

    use super::{GenetAppRunner, is_true_token};

    /// A counter: the canonical Stage 1b app state.
    struct Counter {
        count: u32,
    }

    /// The app logic: a `<div>` holding the count as text. The element type is
    /// the uniform `GenetElement`, so the view type is concrete (no boxing).
    fn counter_view(s: &Counter) -> El<String, Counter, ()> {
        el::<_, Counter, ()>("div", s.count.to_string())
    }

    /// The text data of the (single) text child under `node`.
    fn text_child(dom: &ScriptedDom, node: NodeId) -> Option<String> {
        dom.dom_children(node)
            .find(|&c| dom.kind(c) == NodeKind::Text)
            .and_then(|c| dom.text(c).map(str::to_string))
    }

    fn drain(dom: &DomHandle) -> Vec<DomMutation<NodeId>> {
        let mut out = Vec::new();
        dom.borrow_mut().drain_mutations(&mut out);
        out
    }

    #[test]
    fn deferred_paint_updates_leave_semantics_until_reconciled() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::new(dom.clone(), counter_view, Counter { count: 0 });
        drain(&dom);
        for _ in 0..60 {
            runner.update_deferred(|state| state.count += 1);
        }
        assert_eq!(runner.state().count, 60);
        assert!(drain(&dom).is_empty());
        assert_eq!(
            text_child(&dom.borrow(), runner.root()).as_deref(),
            Some("0")
        );
        runner.update(|_| {});
        assert_eq!(
            text_child(&dom.borrow(), runner.root()).as_deref(),
            Some("60")
        );
        assert!(!drain(&dom).is_empty());
    }

    #[test]
    fn counter_ticks_through_runner() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::new(dom.clone(), counter_view, Counter { count: 0 });

        // The initial tree: a <div> under the document root holding "0".
        let root = runner.root();
        {
            let dom = runner.dom();
            let dom = dom.borrow();
            assert_eq!(dom.kind(root), NodeKind::Element);
            assert_eq!(dom.element_name(root).unwrap().local.to_string(), "div");
            assert_eq!(text_child(&dom, root).as_deref(), Some("0"));
            // The root is attached under the document root.
            assert!(dom.dom_children(dom.document()).any(|c| c == root));
        }
        // The build recorded inserts (text under div, div under document) and
        // no removals.
        let build_muts = drain(&dom);
        assert!(
            build_muts
                .iter()
                .any(|m| matches!(m, DomMutation::Inserted { .. })),
            "build should record inserts: {build_muts:?}"
        );

        // Tick the counter a few times; each update diffs the new count into
        // the existing text node (a CharacterDataChanged, not a re-create).
        for expected in 1..=3u32 {
            runner.update(|s| s.count += 1);
            assert_eq!(runner.state().count, expected);

            {
                let dom = runner.dom();
                let dom = dom.borrow();
                assert_eq!(
                    text_child(&dom, runner.root()).as_deref(),
                    Some(expected.to_string().as_str()),
                    "text under div should read the latest count"
                );
            }

            let muts = drain(&dom);
            // The only structural mutation a count change produces is the text
            // node's character-data update — no node churn.
            assert!(
                muts.iter()
                    .any(|m| matches!(m, DomMutation::CharacterDataChanged { .. })),
                "tick to {expected} should record a CharacterDataChanged: {muts:?}"
            );
            assert!(
                !muts
                    .iter()
                    .any(|m| matches!(m, DomMutation::Inserted { .. })),
                "tick to {expected} should not insert nodes: {muts:?}"
            );
            assert!(
                !muts
                    .iter()
                    .any(|m| matches!(m, DomMutation::Removed { .. })),
                "tick to {expected} should not remove nodes: {muts:?}"
            );
        }
    }

    #[test]
    fn hidden_and_parked_ancestors_are_not_admitted_or_focusable() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::new(
            dom.clone(),
            |_count: &u32| {
                crate::button::<u32, (), (), _>(
                    "increment",
                    |count: &mut u32, _: crate::PointerClick| {
                        *count += 1;
                    },
                )
            },
            0_u32,
        );
        let root = runner.root();
        runner.set_focus(Some(root));
        assert_eq!(runner.focus(), Some(root));

        dom.borrow_mut()
            .set_attribute(root, crate::attr_qual("inert"), "");

        assert!(
            runner
                .dispatch_key(crate::KeyEvent::new(crate::Key::Named(
                    crate::NamedKey::Enter
                )))
                .is_empty()
        );
        assert_eq!(runner.state(), &0, "hidden key activation must be rejected");
        assert!(
            runner
                .dispatch_click(root, crate::PointerClick::at((0.0, 0.0)))
                .is_empty()
        );
        assert!(runner.focusables().is_empty());
        assert_eq!(runner.focus(), None, "a hidden focused node is scrubbed");
        assert_eq!(
            runner.state(),
            &0,
            "hidden activation must not mutate app state"
        );
        runner.set_focus(Some(root));
        assert_eq!(
            runner.focus(),
            None,
            "programmatic focus cannot target hidden content"
        );
    }

    #[derive(Default)]
    struct HideInteractionState {
        hidden: bool,
        focus: Vec<FocusPhase>,
        pointer: Vec<crate::PointerPhase>,
        keys: usize,
        clicks: usize,
        values: usize,
    }

    fn hide_interaction_view(
        state: &HideInteractionState,
    ) -> impl crate::View<HideInteractionState, (), crate::GenetCtx, Element = crate::GenetElement> + use<>
    {
        let element = if state.hidden {
            el::<_, HideInteractionState, ()>("div", "interaction").attr("hidden", "")
        } else {
            el::<_, HideInteractionState, ()>("div", "interaction")
        };
        crate::focusable(crate::on_focus(
            crate::on_value(
                crate::on_key(
                    crate::on_pointer(
                        crate::on_click(element, |state: &mut HideInteractionState, _| {
                            state.clicks += 1;
                        }),
                        |state: &mut HideInteractionState, event: crate::PointerEvent| {
                            state.pointer.push(event.phase);
                        },
                    ),
                    |state: &mut HideInteractionState, _: crate::KeyEvent| state.keys += 1,
                ),
                |state: &mut HideInteractionState, event: crate::ValueEvent| {
                    state.values += event.value as usize;
                },
            ),
            |state: &mut HideInteractionState, event: crate::FocusEvent| {
                state.focus.push(event.phase);
            },
        ))
    }

    #[test]
    fn normal_hidden_rebuild_publishes_focus_lost_and_pointer_cancel_once() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner =
            GenetAppRunner::new(dom, hide_interaction_view, HideInteractionState::default());
        let root = runner.root();
        runner.set_focus(Some(root));
        runner.dispatch_pointer_down(
            root,
            crate::PointerEvent::new(crate::PointerPhase::Down, (1.0, 1.0), (4.0, 4.0)),
        );
        assert_eq!(runner.pointer_capture(), Some(root));
        assert_eq!(runner.state().focus, [FocusPhase::Gained]);
        assert_eq!(runner.state().pointer, [crate::PointerPhase::Down]);

        runner.update(|state| state.hidden = true);

        assert_eq!(runner.focus(), None);
        assert_eq!(runner.pointer_capture(), None);
        assert_eq!(runner.state().focus, [FocusPhase::Gained, FocusPhase::Lost]);
        assert_eq!(
            runner.state().pointer,
            [crate::PointerPhase::Down, crate::PointerPhase::Cancel]
        );
        runner.dispatch_key(crate::KeyEvent::new(crate::Key::Named(
            crate::NamedKey::Enter,
        )));
        runner.dispatch_pointer_down(
            root,
            crate::PointerEvent::new(crate::PointerPhase::Down, (1.0, 1.0), (4.0, 4.0)),
        );
        runner.dispatch_click(root, crate::PointerClick::at((1.0, 1.0)));
        assert_eq!(runner.state().keys, 0);
        assert_eq!(runner.state().clicks, 0);
        assert_eq!(runner.state().focus, [FocusPhase::Gained, FocusPhase::Lost]);
        assert_eq!(
            runner.state().pointer,
            [crate::PointerPhase::Down, crate::PointerPhase::Cancel]
        );
    }

    #[test]
    fn aria_hidden_keeps_physical_clicks_but_rejects_accessibility_actions() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::new(
            dom.clone(),
            hide_interaction_view,
            HideInteractionState::default(),
        );
        let root = runner.root();
        dom.borrow_mut()
            .set_attribute(root, crate::attr_qual("aria-hidden"), "true");

        runner.dispatch_click(root, crate::PointerClick::at((1.0, 1.0)));
        assert_eq!(runner.state().clicks, 1);
        assert!(runner.focusables().is_empty());
        assert!(
            runner
                .dispatch_value(root, crate::ValueEvent { value: 5.0 })
                .is_empty()
        );
        assert_eq!(runner.state().values, 0);
    }

    #[derive(Default)]
    struct FileCompletionState {
        hidden: bool,
        completed: usize,
    }

    fn file_completion_view(
        state: &FileCompletionState,
    ) -> impl crate::View<FileCompletionState, (), crate::GenetCtx, Element = crate::GenetElement> + use<>
    {
        let element = if state.hidden {
            el::<_, FileCompletionState, ()>("button", "choose").attr("hidden", "")
        } else {
            el::<_, FileCompletionState, ()>("button", "choose")
        };
        crate::open_file(
            element,
            true,
            crate::FileFilter::default(),
            |state: &mut FileCompletionState, _: crate::FileEvent| state.completed += 1,
        )
    }

    #[test]
    fn hidden_live_file_owner_receives_already_started_completion() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner =
            GenetAppRunner::new(dom, file_completion_view, FileCompletionState::default());
        let request = runner.take_file_request().expect("initial file request");
        runner.update(|state| state.hidden = true);

        runner.dispatch_file(request.node, crate::FileEvent::default());

        assert_eq!(runner.state().completed, 1);
    }

    #[test]
    fn admission_tokens_are_exact_and_boolean_attributes_are_presence_based() {
        assert!(is_true_token(" true "));
        assert!(is_true_token("1"));
        assert!(!is_true_token("true-ish"));
        assert!(!is_true_token("false"));
    }

    #[test]
    fn hidden_numeric_control_does_not_accept_accessibility_value() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::new(
            dom.clone(),
            |_: &u32| {
                crate::on_value(
                    crate::el::<_, u32, ()>("input", ""),
                    |state: &mut u32, event: crate::ValueEvent| *state = event.value as u32,
                )
            },
            0_u32,
        );
        let root = runner.root();
        dom.borrow_mut()
            .set_attribute(root, crate::attr_qual("inert"), "");
        assert!(
            runner
                .dispatch_value(root, crate::ValueEvent { value: 7.0 })
                .is_empty()
        );
        assert_eq!(runner.state(), &0);
    }

    #[test]
    fn hiding_captured_pointer_target_cancels_capture_before_next_move() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::new(
            dom.clone(),
            |_: &u32| {
                crate::on_pointer(
                    crate::el::<_, u32, ()>("div", "drag"),
                    |count: &mut u32, _: crate::PointerEvent| *count += 1,
                )
            },
            0_u32,
        );
        let root = runner.root();
        runner.dispatch_pointer_down(
            root,
            crate::PointerEvent::new(crate::PointerPhase::Down, (1.0, 1.0), (10.0, 10.0)),
        );
        assert_eq!(runner.pointer_capture(), Some(root));
        dom.borrow_mut()
            .set_attribute(root, crate::attr_qual("hidden"), "");

        runner.dispatch_pointer_move(crate::PointerEvent::new(
            crate::PointerPhase::Move,
            (2.0, 2.0),
            (10.0, 10.0),
        ));
        assert_eq!(runner.pointer_capture(), None);
        assert_eq!(
            runner.state(),
            &1,
            "hidden move must not reach the captured handler"
        );
    }

    #[test]
    fn prepare_move_routes_pointer_cancel_before_host_reparents() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::new(
            dom,
            |_: &Vec<crate::PointerPhase>| {
                crate::on_pointer(
                    crate::el::<_, Vec<crate::PointerPhase>, ()>("div", "drag"),
                    |phases: &mut Vec<crate::PointerPhase>, event: crate::PointerEvent| {
                        phases.push(event.phase);
                    },
                )
            },
            Vec::new(),
        );
        let root = runner.root();
        runner.dispatch_pointer_down(
            root,
            crate::PointerEvent::new(crate::PointerPhase::Down, (1.0, 1.0), (10.0, 10.0)),
        );
        assert!(runner.subtree_move_has_active_interaction(root));

        runner.prepare_subtree_move(root);

        assert_eq!(
            runner.state(),
            &[crate::PointerPhase::Down, crate::PointerPhase::Cancel]
        );
        assert_eq!(runner.pointer_capture(), None);
        assert!(!runner.subtree_move_has_active_interaction(root));
    }

    #[test]
    fn preparing_move_commits_active_preedit_once_before_relocation() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner =
            GenetAppRunner::new(dom, crate::text_field_typed, crate::TextInput::new(""));
        let root = runner.root();
        runner.set_focus(Some(root));
        runner.dispatch_key(crate::KeyEvent::new(crate::Key::Composition(
            CompositionEvent::Preedit {
                text: "漢".into(),
                selection: None,
            },
        )));
        assert_eq!(runner.state().preedit(), "漢");
        assert!(runner.subtree_move_has_active_interaction(root));

        runner.prepare_subtree_move(root);
        assert!(!runner.subtree_move_has_active_interaction(root));
        assert_eq!(runner.state().text(), "漢");
        assert_eq!(runner.state().preedit(), "");

        runner.dispatch_key(crate::KeyEvent::new(crate::Key::Composition(
            CompositionEvent::Commit("漢".into()),
        )));
        assert_eq!(
            runner.state().text(),
            "漢",
            "matching late commit is deduplicated"
        );
    }

    #[test]
    fn prepared_ime_commit_expires_on_lifecycle_or_ordinary_input() {
        let resets = [
            (crate::Key::Composition(CompositionEvent::Enabled), "漢漢"),
            (crate::Key::Composition(CompositionEvent::Disabled), "漢漢"),
            (
                crate::Key::Composition(CompositionEvent::Preedit {
                    text: "漢".into(),
                    selection: None,
                }),
                "漢漢",
            ),
            (crate::Key::Character("q".into()), "漢q漢"),
        ];
        for (reset, expected) in resets {
            let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
            let mut runner =
                GenetAppRunner::new(dom, crate::text_field_typed, crate::TextInput::new(""));
            let root = runner.root();
            runner.set_focus(Some(root));
            runner.dispatch_key(crate::KeyEvent::new(crate::Key::Composition(
                CompositionEvent::Preedit {
                    text: "漢".into(),
                    selection: None,
                },
            )));
            runner.prepare_subtree_move(root);

            runner.dispatch_key(crate::KeyEvent::new(reset));
            runner.dispatch_key(crate::KeyEvent::new(crate::Key::Composition(
                CompositionEvent::Commit("漢".into()),
            )));

            assert_eq!(runner.state().text(), expected);
        }
    }
}
