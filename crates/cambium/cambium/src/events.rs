/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Cambium's listeners on the shared event layer.
//!
//! Click, key and focus listeners live in a [`genet_events::Registry`] under
//! [`CAMBIUM_OWNER`] (script listeners are [`Owner::SCRIPT`]), so one
//! per-node list, ordered by registration, holds every owner's listeners and one
//! [`genet_events::dispatch`] walk runs them. The registry knows a listener only
//! as `(owner, token)`; [`Listeners`] is Cambium's half, the map from a token to
//! the view path that routes the message to its handler.
//!
//! Plan: genet `docs/2026-10-17_native_event_dispatch_plan.md`, Phase 2.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use genet_events::{Invocation, InvokeResult, Invoker, ListenerSpec, Owner, Registry, TargetKey};
use genet_scripted_dom::NodeId;
use meristem::{DynMessage, MessageCtx, MessageResult, View, ViewId};

use crate::{
    DomHandle, FocusEvent, FocusPhase, GenetCtx, GenetElement, GenetElementMut, PointerClick,
    Propagation,
};

/// The owner Cambium's runner registers its listeners under. The script runtime
/// is [`Owner::SCRIPT`] (0); a registry shared with it keeps both apart by this.
pub const CAMBIUM_OWNER: Owner = Owner(1);

/// What a Cambium listener listens for. Each kind is one event type, or two for
/// focus, whose one observer sees both transitions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Kind {
    Click,
    Key,
    Focus,
}

impl Kind {
    fn types(self) -> &'static [&'static str] {
        match self {
            Kind::Click => &["click"],
            Kind::Key => &["keydown"],
            Kind::Focus => &["focus", "blur"],
        }
    }
}

struct Entry {
    node: NodeId,
    kind: Kind,
    /// The `view_path()` captured inside the handler's `with_id`: it routes
    /// straight to the handler's `message`, and with `(node, kind)` it names the
    /// listener for removal.
    path: Vec<ViewId>,
    capture: bool,
    /// Whether this listener also puts its node in the focus traversal set (key
    /// listeners only).
    focusable: bool,
}

/// A native call that reached Cambium's owner while the runner was already inside
/// a dispatch, kept for the runner to run when that dispatch unwinds.
pub(crate) struct Queued {
    pub(crate) token: u64,
    pub(crate) event_type: Rc<str>,
}

/// The `genet_events` registry plus Cambium's token table.
pub(crate) struct Listeners {
    events: Rc<RefCell<Registry>>,
    next_token: u64,
    entries: HashMap<u64, Entry>,
    /// Tokens per node, in registration order.
    by_node: HashMap<NodeId, Vec<u64>>,
    queue: Rc<RefCell<VecDeque<Queued>>>,
}

impl Listeners {
    pub(crate) fn new(events: Rc<RefCell<Registry>>) -> Self {
        Self {
            events,
            next_token: 0,
            entries: HashMap::new(),
            by_node: HashMap::new(),
            queue: Rc::default(),
        }
    }

    pub(crate) fn events(&self) -> Rc<RefCell<Registry>> {
        self.events.clone()
    }

    /// Register `path` as *a* listener of `kind` on `node`, after any already
    /// there. Idempotent per `(node, kind, path)`: registering again replaces the
    /// earlier registration, so its phase updates and it takes the latest place.
    pub(crate) fn add(
        &mut self,
        node: NodeId,
        kind: Kind,
        path: Vec<ViewId>,
        capture: bool,
        focusable: bool,
    ) {
        self.remove(node, kind, &path);
        self.next_token += 1;
        let token = self.next_token;
        let mut events = self.events.borrow_mut();
        for event_type in kind.types() {
            events.add(
                TargetKey::Node(node),
                ListenerSpec {
                    owner: CAMBIUM_OWNER,
                    token,
                    event_type: (*event_type).to_owned(),
                    capture,
                    once: false,
                    passive: false,
                },
            );
        }
        drop(events);
        self.by_node.entry(node).or_default().push(token);
        self.entries.insert(
            token,
            Entry {
                node,
                kind,
                path,
                capture,
                focusable,
            },
        );
    }

    /// Drop the listener of `kind` with `path` from `node`, leaving its siblings.
    pub(crate) fn remove(&mut self, node: NodeId, kind: Kind, path: &[ViewId]) {
        let found = self.by_node.get(&node).and_then(|tokens| {
            tokens.iter().copied().find(|token| {
                let entry = &self.entries[token];
                entry.kind == kind && entry.path == path
            })
        });
        if let Some(token) = found {
            self.drop_token(token);
        }
    }

    /// Drop every listener of `kind` on `node`.
    pub(crate) fn remove_all(&mut self, node: NodeId, kind: Kind) {
        let found: Vec<u64> = self
            .by_node
            .get(&node)
            .into_iter()
            .flatten()
            .copied()
            .filter(|token| self.entries[token].kind == kind)
            .collect();
        for token in found {
            self.drop_token(token);
        }
    }

    fn drop_token(&mut self, token: u64) {
        let Some(entry) = self.entries.remove(&token) else {
            return;
        };
        let mut events = self.events.borrow_mut();
        for event_type in entry.kind.types() {
            events.remove(
                TargetKey::Node(entry.node),
                CAMBIUM_OWNER,
                token,
                event_type,
                entry.capture,
            );
        }
        drop(events);
        if let Some(tokens) = self.by_node.get_mut(&entry.node) {
            tokens.retain(|t| *t != token);
            if tokens.is_empty() {
                self.by_node.remove(&entry.node);
            }
        }
    }

    /// The routing path of the listener `token` names, if it is still registered.
    pub(crate) fn path(&self, token: u64) -> Option<&[ViewId]> {
        self.entries.get(&token).map(|entry| entry.path.as_slice())
    }

    pub(crate) fn has(&self, node: NodeId, kind: Kind) -> bool {
        self.of_node(node).any(|entry| entry.kind == kind)
    }

    /// Whether a key listener on `node` also makes it focusable.
    pub(crate) fn has_focusable_key(&self, node: NodeId) -> bool {
        self.of_node(node)
            .any(|entry| entry.kind == Kind::Key && entry.focusable)
    }

    fn of_node(&self, node: NodeId) -> impl Iterator<Item = &Entry> {
        self.by_node
            .get(&node)
            .into_iter()
            .flatten()
            .map(|token| &self.entries[token])
    }

    pub(crate) fn owner(&self) -> NativeOwner {
        NativeOwner {
            queue: self.queue.clone(),
        }
    }

    pub(crate) fn take_queued(&self) -> Option<Queued> {
        self.queue.borrow_mut().pop_front()
    }
}

/// The runner's endpoint for calls that reach Cambium's owner from a dispatch it
/// is not running: a script listener that calls `dispatchEvent` during one of the
/// runner's own walks, say, whose nested walk meets a Cambium listener.
///
/// The runner's state is borrowed by the outer walk, so the nested call cannot
/// run where it arrives. It is queued, and the runner runs it, as a synthesised
/// event, when its current click, key or focus dispatch unwinds. Hand this to
/// the other owner's dispatcher as the [`Invoker`] for [`CAMBIUM_OWNER`].
///
/// This departs from the DOM order, deliberately (plan §6, ruling 2): a queued
/// listener runs after the outer walk finishes, not in the middle of it, and
/// whatever it stops or cancels cannot reach the nested dispatch that triggered
/// it. A call that arrives while no dispatch is running waits for the next one.
/// Only `click`, `focus` and `blur` calls are kept: the layer carries no payload
/// yet, so a click is synthesised at the origin (as keyboard activation does)
/// and a `keydown` has nothing to synthesise from.
#[derive(Clone)]
pub struct NativeOwner {
    queue: Rc<RefCell<VecDeque<Queued>>>,
}

impl Invoker for NativeOwner {
    fn invoke(&mut self, call: Invocation<'_>) -> InvokeResult {
        if call.listener.owner == CAMBIUM_OWNER {
            self.queue.borrow_mut().push_back(Queued {
                token: call.listener.token,
                event_type: call.listener.event_type.clone(),
            });
        }
        InvokeResult::default()
    }
}

/// The runner's side of one walk: runs Cambium's listeners as `View::message`
/// and hands every other owner's to the host's invoker.
pub(crate) struct Routing<'a, State, V, Action>
where
    State: 'static,
    Action: 'static,
    V: View<State, Action, GenetCtx, Element = GenetElement>,
{
    pub(crate) view: &'a V,
    pub(crate) view_state: &'a mut V::ViewState,
    pub(crate) root: &'a mut GenetElement,
    pub(crate) dom: &'a DomHandle,
    pub(crate) mount: NodeId,
    pub(crate) state: &'a mut State,
    pub(crate) listeners: &'a Listeners,
    pub(crate) foreign: Option<&'a mut (dyn Invoker + 'static)>,
    /// Builds the message the walked event delivers to a Cambium listener.
    pub(crate) message: &'a dyn Fn() -> DynMessage,
    /// The event's shared flags, read back after each listener (focus events
    /// carry none).
    pub(crate) prop: Option<&'a Propagation>,
    /// The actions that reached the root.
    pub(crate) actions: Vec<Action>,
    /// How many listeners ran.
    pub(crate) invoked: usize,
}

impl<State, V, Action> Routing<'_, State, V, Action>
where
    State: 'static,
    Action: 'static,
    V: View<State, Action, GenetCtx, Element = GenetElement>,
{
    /// Send `message` down the view path `token` names; a listener removed
    /// since is skipped. The recorded path is the whole routing target, so the
    /// context is not needed. A root-level `MessageResult::Action` is the
    /// runner's `Action` home: it is collected for the caller.
    fn route(&mut self, token: u64, message: DynMessage) -> bool {
        let Some(path) = self.listeners.path(token) else {
            return false;
        };
        let mut message = MessageCtx::new(path.to_vec(), message);
        let element = GenetElementMut {
            node: &mut self.root.node,
            dom: self.dom.clone(),
            parent: Some(self.mount),
        };
        if let MessageResult::Action(action) =
            self.view
                .message(self.view_state, &mut message, element, self.state)
        {
            self.actions.push(action);
        }
        self.invoked += 1;
        true
    }

    /// Run the calls that reached [`NativeOwner`] while the walk was in flight,
    /// in the order they arrived. A listener one of them removes is skipped, and
    /// one of them that queues another runs it here, after the rest.
    pub(crate) fn run_queued(&mut self) {
        while let Some(queued) = self.listeners.take_queued() {
            let message = match &*queued.event_type {
                "click" => DynMessage::new(PointerClick::at((0.0, 0.0))),
                "focus" => DynMessage::new(FocusEvent::new(FocusPhase::Gained)),
                "blur" => DynMessage::new(FocusEvent::new(FocusPhase::Lost)),
                _ => continue,
            };
            self.route(queued.token, message);
        }
    }
}

impl<State, V, Action> Invoker for Routing<'_, State, V, Action>
where
    State: 'static,
    Action: 'static,
    V: View<State, Action, GenetCtx, Element = GenetElement>,
{
    fn invoke(&mut self, call: Invocation<'_>) -> InvokeResult {
        if call.listener.owner != CAMBIUM_OWNER {
            // Another owner's listener shares the node: its own invoker runs it,
            // in this walk, at this listener's place in the order.
            return match self.foreign.as_mut() {
                Some(foreign) => {
                    self.invoked += 1;
                    foreign.invoke(call)
                },
                None => InvokeResult::default(),
            };
        }
        if !self.route(call.listener.token, (self.message)()) {
            return InvokeResult::default();
        }
        self.prop
            .map_or_else(InvokeResult::default, |prop| InvokeResult {
                stop_propagation: prop.stopped(),
                stop_immediate: prop.stopped_immediately(),
                canceled: prop.default_prevented(),
            })
    }
}
