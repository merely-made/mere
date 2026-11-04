/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Type erasure for retained product-owned Cambium runners.
//!
//! Frozen as v1 with the descriptor vocabulary (2026-08-26): the fourteen
//! trait methods stand, including the three no erased host calls yet —
//! `root`, `focusables`, and `pointer_capture` erase the same capabilities
//! the concrete hosts already rely on (subtree identity, spatial focus, and
//! mid-drag capture routing). Changes are additive until a v2.
//!
//! This boundary starts after a host has translated platform input and resolved
//! any DOM hit target. It intentionally does not perform layout, hit testing,
//! scene conversion, scrolling policy, accessibility hosting, or lifetime
//! management for the host.

use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};

use genet_scripted_dom::NodeId;
use mere_surface_api::{SurfaceAvailability, SurfaceDescriptor, SurfaceUnavailableReason};
use meristem::View;
use sprigging::LeafRegistry;

use crate::{
    DomHandle, FocusExit, GenetAppRunner, GenetCtx, GenetElement, HoverEvent, KeyEvent,
    PointerClick, PointerEvent, WheelEvent,
};

/// The smallest generic effect a retained surface can request from its host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurfaceEffect {
    Redraw,
    /// Traversal left the session past its last focusable (forward) or before
    /// its first (backward), and the session cleared its focus. The host moves
    /// focus to its next stop in that direction. Only a session whose host
    /// turned on [focus exits](RetainedSurfaceSession::set_focus_exits)
    /// reports it.
    FocusExit(FocusExit),
}

/// Viewport facts supplied by a host after it has laid out a surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceViewport {
    pub width: f32,
    pub height: f32,
    pub scale_factor: f32,
}

/// A Cambium event whose DOM target, when one is needed, was already resolved
/// by the host. Raw host input has no DOM target and therefore cannot drive the
/// runner by itself.
#[derive(Clone, Debug)]
pub enum ResolvedSurfaceEvent {
    Click { target: NodeId, event: PointerClick },
    Key(KeyEvent),
    PointerDown { target: NodeId, event: PointerEvent },
    PointerMove(PointerEvent),
    PointerUp(PointerEvent),
    Hover { target: NodeId, event: HoverEvent },
    Wheel { target: NodeId, event: WheelEvent },
}

/// An object-safe retained product surface.
///
/// The concrete `GenetAppRunner` state, view, and action types remain inside
/// the session. Hosts retain this object and render its DOM through their own
/// layout path.
pub trait RetainedSurfaceSession {
    fn descriptor(&self) -> &SurfaceDescriptor;
    fn availability(&self) -> SurfaceAvailability;
    fn dom(&self) -> DomHandle;
    fn root(&self) -> NodeId;
    fn focus(&self) -> Option<NodeId>;
    fn set_focus(&mut self, node: Option<NodeId>) -> Vec<SurfaceEffect>;
    fn focus_traverse(&mut self, forward: bool) -> Vec<SurfaceEffect>;
    fn focusables(&self) -> Vec<NodeId>;
    fn pointer_capture(&self) -> Option<NodeId>;
    fn pointer_target(&self, hit: NodeId) -> Option<NodeId>;
    fn hover_target(&self, hit: NodeId) -> Option<NodeId>;
    fn wheel_target(&self, hit: NodeId) -> Option<NodeId>;
    fn sync_viewport(&mut self, viewport: SurfaceViewport) -> Vec<SurfaceEffect>;
    fn dispatch(&mut self, event: ResolvedSurfaceEvent) -> Vec<SurfaceEffect>;

    /// Stop focus traversal at the session's edges and report
    /// [`SurfaceEffect::FocusExit`] there, instead of wrapping inside the
    /// session. A host showing the session beside other content turns this on
    /// so Tab can leave; without it every composed session is a keyboard trap.
    ///
    /// Returns whether the session honours it. The provided method returns
    /// `false` and changes nothing, so a host can tell a session that still
    /// wraps. Added 2026-10-07 (app composition brief, recommendation 1);
    /// additive to v1.
    fn set_focus_exits(&mut self, exits: bool) -> bool {
        let _ = exits;
        false
    }

    /// The session's own leaf registry, for a session whose document places
    /// custom leaves. The host paints and projects this session's document
    /// against it, and keeps the session's rendered-leaf cache apart too, so
    /// two sessions that use the same leaf key each get their own leaf. Keys
    /// stay as the product wrote them. The provided method returns `None`: the
    /// session places no leaves of its own. Added 2026-10-07 (per-session
    /// registries, ruled by Mark); additive to v1.
    fn leaves(&mut self) -> Option<&mut LeafRegistry<u64>> {
        None
    }
}

/// A generic retained-session wrapper around one concrete [`GenetAppRunner`].
///
/// Availability, viewport synchronization, and action effects remain
/// product-owned closures. The host observes their results but does not author
/// product availability or actions.
pub struct RunnerSurfaceSession<State, Logic, V, Action, ViewportSync, ActionEffects>
where
    State: 'static,
    Action: 'static,
    Logic: FnMut(&State) -> V,
    V: View<State, Action, GenetCtx, Element = GenetElement>,
    ViewportSync: FnMut(&mut State, SurfaceViewport),
    ActionEffects: FnMut(Action) -> Vec<SurfaceEffect>,
{
    descriptor: SurfaceDescriptor,
    runner: GenetAppRunner<State, Logic, V, Action>,
    availability: Box<dyn Fn(&State) -> SurfaceAvailability>,
    viewport: Option<SurfaceViewport>,
    viewport_sync: ViewportSync,
    action_effects: ActionEffects,
    leaves: Option<LeafRegistry<u64>>,
}

impl<State, Logic, V, Action, ViewportSync, ActionEffects>
    RunnerSurfaceSession<State, Logic, V, Action, ViewportSync, ActionEffects>
where
    State: 'static,
    Action: 'static,
    Logic: FnMut(&State) -> V,
    V: View<State, Action, GenetCtx, Element = GenetElement>,
    ViewportSync: FnMut(&mut State, SurfaceViewport),
    ActionEffects: FnMut(Action) -> Vec<SurfaceEffect>,
{
    pub fn new(
        descriptor: SurfaceDescriptor,
        runner: GenetAppRunner<State, Logic, V, Action>,
        availability: impl Fn(&State) -> SurfaceAvailability + 'static,
        viewport_sync: ViewportSync,
        action_effects: ActionEffects,
    ) -> Self {
        Self {
            descriptor,
            runner,
            availability: Box::new(availability),
            viewport: None,
            viewport_sync,
            action_effects,
            leaves: None,
        }
    }

    /// Give the session its own leaf registry, holding the leaves its view
    /// places with `custom_leaf`. See [`RetainedSurfaceSession::leaves`].
    pub fn with_leaves(mut self, leaves: LeafRegistry<u64>) -> Self {
        self.leaves = Some(leaves);
        self
    }

    fn effects(&mut self, actions: Vec<Action>) -> Vec<SurfaceEffect> {
        let mut effects = Vec::with_capacity(actions.len() + 2);
        effects.push(SurfaceEffect::Redraw);
        for action in actions {
            effects.extend((self.action_effects)(action));
        }
        if let Some(exit) = self.runner.take_focus_exit() {
            effects.push(SurfaceEffect::FocusExit(exit));
        }
        effects
    }
}

impl<State, Logic, V, Action, ViewportSync, ActionEffects> RetainedSurfaceSession
    for RunnerSurfaceSession<State, Logic, V, Action, ViewportSync, ActionEffects>
where
    State: 'static,
    Action: 'static,
    Logic: FnMut(&State) -> V,
    V: View<State, Action, GenetCtx, Element = GenetElement>,
    ViewportSync: FnMut(&mut State, SurfaceViewport),
    ActionEffects: FnMut(Action) -> Vec<SurfaceEffect>,
{
    fn descriptor(&self) -> &SurfaceDescriptor {
        &self.descriptor
    }

    fn availability(&self) -> SurfaceAvailability {
        (self.availability)(self.runner.state())
    }

    fn dom(&self) -> DomHandle {
        self.runner.dom()
    }

    fn root(&self) -> NodeId {
        self.runner.root()
    }

    fn focus(&self) -> Option<NodeId> {
        self.runner.focus()
    }

    fn set_focus(&mut self, node: Option<NodeId>) -> Vec<SurfaceEffect> {
        self.runner.set_focus(node);
        vec![SurfaceEffect::Redraw]
    }

    fn focus_traverse(&mut self, forward: bool) -> Vec<SurfaceEffect> {
        self.runner.focus_traverse(forward);
        self.effects(Vec::new())
    }

    fn focusables(&self) -> Vec<NodeId> {
        self.runner.focusables()
    }

    fn pointer_capture(&self) -> Option<NodeId> {
        self.runner.pointer_capture()
    }

    fn pointer_target(&self, hit: NodeId) -> Option<NodeId> {
        self.runner.pointer_target(hit)
    }

    fn hover_target(&self, hit: NodeId) -> Option<NodeId> {
        self.runner.hover_target(hit)
    }

    fn wheel_target(&self, hit: NodeId) -> Option<NodeId> {
        self.runner.wheel_target(hit)
    }

    fn sync_viewport(&mut self, viewport: SurfaceViewport) -> Vec<SurfaceEffect> {
        if self.viewport == Some(viewport) {
            return Vec::new();
        }
        self.viewport = Some(viewport);
        let viewport_sync = &mut self.viewport_sync;
        self.runner.update(|state| viewport_sync(state, viewport));
        vec![SurfaceEffect::Redraw]
    }

    fn dispatch(&mut self, event: ResolvedSurfaceEvent) -> Vec<SurfaceEffect> {
        let actions = match event {
            ResolvedSurfaceEvent::Click { target, event } => {
                self.runner.dispatch_click(target, event)
            },
            ResolvedSurfaceEvent::Key(event) => self.runner.dispatch_key(event),
            ResolvedSurfaceEvent::PointerDown { target, event } => {
                self.runner.dispatch_pointer_down(target, event)
            },
            ResolvedSurfaceEvent::PointerMove(event) => self.runner.dispatch_pointer_move(event),
            ResolvedSurfaceEvent::PointerUp(event) => self.runner.dispatch_pointer_up(event),
            ResolvedSurfaceEvent::Hover { target, event } => {
                self.runner.dispatch_hover(target, event)
            },
            ResolvedSurfaceEvent::Wheel { target, event } => {
                self.runner.dispatch_wheel(target, event)
            },
        };
        self.effects(actions)
    }

    fn set_focus_exits(&mut self, exits: bool) -> bool {
        self.runner.set_focus_exits(exits);
        true
    }

    fn leaves(&mut self) -> Option<&mut LeafRegistry<u64>> {
        self.leaves.as_mut()
    }
}

/// A guest session's panic, caught at the session boundary by
/// [`ContainedSession`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionFailure {
    /// The trait method the panic unwound out of.
    pub call: &'static str,
    /// The panic's message, when it carried a string.
    pub message: Option<String>,
}

/// A retained session whose panics stop at the session boundary.
///
/// Every call into the wrapped session runs under `catch_unwind`. After the
/// first caught panic the session is retired: it reports
/// `Unavailable(Unhealthy)`, has no focus, focusables or targets, ignores input,
/// and never calls the wrapped session again. The host shows an unavailable
/// surface in its place and leaves the session's accessibility subtree out of
/// its next frame. The DOM handle and root stay readable, as last seen, for a
/// host that is mid-frame, but the host stops laying the session out.
///
/// Only unwinding panics are caught. A build with `panic = "abort"`, an abort,
/// or a hang still takes the host down; containing those needs a process
/// boundary (app composition brief, AC4, ruled 2026-10-07).
pub struct ContainedSession<S> {
    session: S,
    descriptor: SurfaceDescriptor,
    dom: RefCell<DomHandle>,
    root: RefCell<NodeId>,
    failure: RefCell<Option<SessionFailure>>,
}

impl<S: RetainedSurfaceSession> ContainedSession<S> {
    pub fn new(session: S) -> Self {
        Self {
            descriptor: session.descriptor().clone(),
            dom: RefCell::new(session.dom()),
            root: RefCell::new(session.root()),
            failure: RefCell::new(None),
            session,
        }
    }

    /// The caught panic, once the session has failed.
    pub fn failure(&self) -> Option<SessionFailure> {
        self.failure.borrow().clone()
    }

    fn failed(&self) -> bool {
        self.failure.borrow().is_some()
    }

    fn record(&self, call: &'static str, payload: Box<dyn std::any::Any + Send>) {
        let message = payload
            .downcast_ref::<&str>()
            .map(|text| (*text).to_owned())
            .or_else(|| payload.downcast_ref::<String>().cloned());
        *self.failure.borrow_mut() = Some(SessionFailure { call, message });
    }

    /// Ask the wrapped session, or answer `inert` once it has failed.
    fn read<T>(&self, call: &'static str, inert: T, ask: impl FnOnce(&S) -> T) -> T {
        if self.failed() {
            return inert;
        }
        match catch_unwind(AssertUnwindSafe(|| ask(&self.session))) {
            Ok(value) => value,
            Err(payload) => {
                self.record(call, payload);
                inert
            },
        }
    }

    /// Drive the wrapped session. The call that fails asks for one redraw,
    /// so the host paints the unavailable surface; later calls do nothing.
    fn drive(
        &mut self,
        call: &'static str,
        act: impl FnOnce(&mut S) -> Vec<SurfaceEffect>,
    ) -> Vec<SurfaceEffect> {
        if self.failed() {
            return Vec::new();
        }
        match catch_unwind(AssertUnwindSafe(|| act(&mut self.session))) {
            Ok(effects) => effects,
            Err(payload) => {
                self.record(call, payload);
                vec![SurfaceEffect::Redraw]
            },
        }
    }
}

impl<S: RetainedSurfaceSession> RetainedSurfaceSession for ContainedSession<S> {
    fn descriptor(&self) -> &SurfaceDescriptor {
        &self.descriptor
    }

    fn availability(&self) -> SurfaceAvailability {
        let unhealthy = SurfaceAvailability::Unavailable(SurfaceUnavailableReason::Unhealthy);
        self.read("availability", unhealthy, |session| session.availability())
    }

    fn dom(&self) -> DomHandle {
        let last = self.dom.borrow().clone();
        let dom = self.read("dom", last, |session| session.dom());
        *self.dom.borrow_mut() = dom.clone();
        dom
    }

    fn root(&self) -> NodeId {
        let last = *self.root.borrow();
        let root = self.read("root", last, |session| session.root());
        *self.root.borrow_mut() = root;
        root
    }

    fn focus(&self) -> Option<NodeId> {
        self.read("focus", None, |session| session.focus())
    }

    fn set_focus(&mut self, node: Option<NodeId>) -> Vec<SurfaceEffect> {
        self.drive("set_focus", |session| session.set_focus(node))
    }

    fn focus_traverse(&mut self, forward: bool) -> Vec<SurfaceEffect> {
        self.drive("focus_traverse", |session| session.focus_traverse(forward))
    }

    fn focusables(&self) -> Vec<NodeId> {
        self.read("focusables", Vec::new(), |session| session.focusables())
    }

    fn pointer_capture(&self) -> Option<NodeId> {
        self.read("pointer_capture", None, |session| session.pointer_capture())
    }

    fn pointer_target(&self, hit: NodeId) -> Option<NodeId> {
        self.read("pointer_target", None, |session| {
            session.pointer_target(hit)
        })
    }

    fn hover_target(&self, hit: NodeId) -> Option<NodeId> {
        self.read("hover_target", None, |session| session.hover_target(hit))
    }

    fn wheel_target(&self, hit: NodeId) -> Option<NodeId> {
        self.read("wheel_target", None, |session| session.wheel_target(hit))
    }

    fn sync_viewport(&mut self, viewport: SurfaceViewport) -> Vec<SurfaceEffect> {
        self.drive("sync_viewport", |session| session.sync_viewport(viewport))
    }

    fn dispatch(&mut self, event: ResolvedSurfaceEvent) -> Vec<SurfaceEffect> {
        self.drive("dispatch", |session| session.dispatch(event))
    }

    fn set_focus_exits(&mut self, exits: bool) -> bool {
        if self.failed() {
            return false;
        }
        match catch_unwind(AssertUnwindSafe(|| self.session.set_focus_exits(exits))) {
            Ok(honoured) => honoured,
            Err(payload) => {
                self.record("set_focus_exits", payload);
                false
            },
        }
    }

    /// A failed session's leaves are no longer the host's to paint.
    fn leaves(&mut self) -> Option<&mut LeafRegistry<u64>> {
        if self.failed() {
            return None;
        }
        // Asked twice: a borrow returned out of one arm of the match below
        // would keep `self` borrowed in the arm that records the failure.
        let session = &mut self.session;
        match catch_unwind(AssertUnwindSafe(move || session.leaves().is_some())) {
            Ok(true) => self.session.leaves(),
            Ok(false) => None,
            Err(payload) => {
                self.record("leaves", payload);
                None
            },
        }
    }
}

#[cfg(test)]
#[path = "surface/tests.rs"]
mod tests;
