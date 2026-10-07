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
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use genet_scripted_dom::{NodeId, ScriptedDom};
    use layout_dom_api::{LayoutDom, NodeKind};
    use mere_surface_api::{
        ProviderId, SourceKindId, SurfaceId, SurfaceSourceShape, SurfaceUnavailableReason,
    };

    use crate::{
        DomHandle, El, FocusExit, GenetAppRunner, Key, KeyEvent, Modifiers, NamedKey, OnClick,
        PointerClick, el, on_click, on_key,
    };

    use super::*;

    struct First {
        count: u32,
        width: f32,
    }

    struct Second {
        count: u32,
        width: f32,
    }

    type FirstView = OnClick<El<String, First, ()>, First, (), fn(&mut First, PointerClick)>;

    type SecondView = OnClick<El<String, Second, ()>, Second, (), fn(&mut Second, PointerClick)>;

    fn first_click(state: &mut First, _event: PointerClick) {
        state.count += 1;
    }

    fn second_click(state: &mut Second, _event: PointerClick) {
        state.count += 10;
    }

    fn first_view(state: &First) -> FirstView {
        on_click(
            el::<_, First, ()>(
                "button",
                format!("first:{}:{:.0}", state.count, state.width),
            ),
            first_click as fn(&mut First, PointerClick),
        )
    }

    fn second_view(state: &Second) -> SecondView {
        on_click(
            el::<_, Second, ()>("div", format!("second:{}:{:.0}", state.count, state.width)),
            second_click as fn(&mut Second, PointerClick),
        )
    }

    fn descriptor(id: &str) -> SurfaceDescriptor {
        SurfaceDescriptor {
            provider_id: ProviderId::from("example"),
            surface_id: SurfaceId::from(id),
            label: id.to_owned(),
            accepted_source: SurfaceSourceShape::One(SourceKindId::from("example.source")),
        }
    }

    fn fresh_dom() -> DomHandle {
        Rc::new(RefCell::new(ScriptedDom::new()))
    }

    fn root_text(dom: &DomHandle, root: NodeId) -> String {
        let dom = dom.borrow();
        let text = dom
            .dom_children(root)
            .find(|node| dom.kind(*node) == NodeKind::Text)
            .expect("surface root has a text child");
        dom.text(text).expect("text child has text").to_owned()
    }

    #[test]
    fn erased_sessions_retain_distinct_runner_state_and_dispatch_independently() {
        let first_dom = fresh_dom();
        let first_runner = GenetAppRunner::new(
            first_dom.clone(),
            first_view,
            First {
                count: 0,
                width: 0.0,
            },
        );
        let first = RunnerSurfaceSession::new(
            descriptor("example.first"),
            first_runner,
            |state: &First| {
                if state.count == 0 {
                    SurfaceAvailability::Available
                } else {
                    SurfaceAvailability::Unavailable(SurfaceUnavailableReason::Locked)
                }
            },
            |state: &mut First, viewport| state.width = viewport.width,
            |_action: ()| Vec::new(),
        );
        let first_root = first.root();

        let second_dom = fresh_dom();
        let second_runner = GenetAppRunner::new(
            second_dom.clone(),
            second_view,
            Second {
                count: 10,
                width: 0.0,
            },
        );
        let second = RunnerSurfaceSession::new(
            descriptor("example.second"),
            second_runner,
            |_state: &Second| SurfaceAvailability::Available,
            |state: &mut Second, viewport| state.width = viewport.width,
            |_action: ()| Vec::new(),
        );
        let second_root = second.root();

        let mut sessions: Vec<Box<dyn RetainedSurfaceSession>> =
            vec![Box::new(first), Box::new(second)];

        assert_eq!(sessions[0].root(), first_root);
        assert_eq!(sessions[1].root(), second_root);
        assert_eq!(
            sessions[0].dispatch(ResolvedSurfaceEvent::Click {
                target: first_root,
                event: PointerClick::at((0.0, 0.0)),
            }),
            vec![SurfaceEffect::Redraw]
        );
        assert_eq!(root_text(&first_dom, first_root), "first:1:0");
        assert_eq!(root_text(&second_dom, second_root), "second:10:0");
        assert_eq!(
            sessions[0].availability(),
            SurfaceAvailability::Unavailable(SurfaceUnavailableReason::Locked)
        );

        assert_eq!(
            sessions[1].sync_viewport(SurfaceViewport {
                width: 240.0,
                height: 160.0,
                scale_factor: 1.0,
            }),
            vec![SurfaceEffect::Redraw]
        );
        assert_eq!(root_text(&first_dom, first_root), "first:1:0");
        assert_eq!(root_text(&second_dom, second_root), "second:10:240");
        assert_eq!(
            sessions[1].sync_viewport(SurfaceViewport {
                width: 240.0,
                height: 160.0,
                scale_factor: 1.0,
            }),
            Vec::<SurfaceEffect>::new()
        );

        sessions[1].dispatch(ResolvedSurfaceEvent::Click {
            target: second_root,
            event: PointerClick::at((0.0, 0.0)),
        });
        assert_eq!(root_text(&first_dom, first_root), "first:1:0");
        assert_eq!(root_text(&second_dom, second_root), "second:20:240");
    }

    /// A session whose host turns on focus exits reports Tab past its last
    /// focusable as an effect, so the host can move on; with exits off it
    /// wraps, as standalone apps expect.
    #[test]
    fn a_session_reports_focus_leaving_past_its_edge() {
        let dom = fresh_dom();
        let noop: fn(&mut (), KeyEvent) = |_, _| {};
        let runner = GenetAppRunner::<_, _, _, ()>::new(
            dom,
            move |_: &()| {
                el::<_, (), ()>(
                    "div",
                    (
                        on_key(el::<_, (), ()>("a", ()), noop),
                        on_key(el::<_, (), ()>("b", ()), noop),
                    ),
                )
            },
            (),
        );
        let mut session = RunnerSurfaceSession::new(
            descriptor("example.focus"),
            runner,
            |_: &()| SurfaceAvailability::Available,
            |_: &mut (), _| {},
            |_action: ()| Vec::new(),
        );
        let tab = |shift: bool| {
            ResolvedSurfaceEvent::Key(KeyEvent::with_mods(
                Key::Named(NamedKey::Tab),
                Modifiers {
                    shift,
                    ..Default::default()
                },
            ))
        };

        session.focus_traverse(true);
        session.focus_traverse(true);
        assert_eq!(
            session.dispatch(tab(false)),
            vec![SurfaceEffect::Redraw],
            "exits off: Tab wraps inside the session"
        );
        assert!(session.focus().is_some());

        assert!(session.set_focus_exits(true));
        session.focus_traverse(true);
        assert_eq!(
            session.dispatch(tab(false)),
            vec![
                SurfaceEffect::Redraw,
                SurfaceEffect::FocusExit(FocusExit::Forward)
            ]
        );
        assert_eq!(session.focus(), None);
        assert_eq!(
            session.focus_traverse(false),
            vec![SurfaceEffect::Redraw],
            "the host enters again from the far end"
        );
        assert_eq!(
            session.dispatch(tab(true)),
            vec![SurfaceEffect::Redraw],
            "from the last, Shift+Tab moves back inside"
        );
        assert_eq!(
            session.dispatch(tab(true)),
            vec![
                SurfaceEffect::Redraw,
                SurfaceEffect::FocusExit(FocusExit::Backward)
            ]
        );
    }

    /// A panic in one guest's dispatch is caught at the boundary: that guest
    /// retires as unhealthy and is never called again, and a guest beside it
    /// carries on.
    #[test]
    fn a_contained_session_retires_on_a_panic_and_its_neighbour_carries_on() {
        fn boom(_: &mut First, _: PointerClick) {
            panic!("guest bug");
        }
        fn failing_view(state: &First) -> FirstView {
            on_click(
                el::<_, First, ()>("button", format!("first:{}", state.count)),
                boom as fn(&mut First, PointerClick),
            )
        }

        let failing_runner = GenetAppRunner::new(
            fresh_dom(),
            failing_view,
            First {
                count: 0,
                width: 0.0,
            },
        );
        let mut failing = ContainedSession::new(RunnerSurfaceSession::new(
            descriptor("example.failing"),
            failing_runner,
            |_: &First| SurfaceAvailability::Available,
            |_: &mut First, _| {},
            |_action: ()| Vec::new(),
        ));
        let failing_root = failing.root();

        let second_dom = fresh_dom();
        let second_runner = GenetAppRunner::new(
            second_dom.clone(),
            second_view,
            Second {
                count: 0,
                width: 0.0,
            },
        );
        let mut second = ContainedSession::new(RunnerSurfaceSession::new(
            descriptor("example.second"),
            second_runner,
            |_: &Second| SurfaceAvailability::Available,
            |_: &mut Second, _| {},
            |_action: ()| Vec::new(),
        ));
        let second_root = second.root();

        assert_eq!(failing.availability(), SurfaceAvailability::Available);
        let click = |target| ResolvedSurfaceEvent::Click {
            target,
            event: PointerClick::at((0.0, 0.0)),
        };
        assert_eq!(
            failing.dispatch(click(failing_root)),
            vec![SurfaceEffect::Redraw],
            "the failing call asks for one redraw"
        );
        assert_eq!(
            failing.failure(),
            Some(SessionFailure {
                call: "dispatch",
                message: Some("guest bug".to_owned()),
            })
        );
        assert_eq!(
            failing.availability(),
            SurfaceAvailability::Unavailable(SurfaceUnavailableReason::Unhealthy)
        );
        assert_eq!(failing.focusables(), Vec::<NodeId>::new());
        assert_eq!(failing.focus(), None);
        assert!(!failing.set_focus_exits(true));
        assert_eq!(failing.dispatch(click(failing_root)), Vec::new());
        assert_eq!(failing.root(), failing_root, "the root stays readable");

        second.dispatch(click(second_root));
        assert_eq!(root_text(&second_dom, second_root), "second:10:0");
        assert_eq!(second.failure(), None);
        assert_eq!(second.availability(), SurfaceAvailability::Available);
    }

    /// A leaf the test can tell apart from another under the same key.
    struct Tagged(&'static str);

    impl sprigging::Leaf for Tagged {
        fn measure(
            &mut self,
            _known: sprigging::SizeHint,
            _available: sprigging::SizeHint,
        ) -> sprigging::Size {
            sprigging::Size {
                width: 1.0,
                height: 1.0,
            }
        }

        fn paint(&mut self, _cx: &mut sprigging::PaintCx<'_>) {}

        fn paint_dirty(&self) -> bool {
            false
        }
    }

    /// Two sessions that place a leaf under the same key each keep their own
    /// leaf, because each owns its registry. The control: one registry shared
    /// between them holds one leaf per key, so one session would paint the
    /// other's.
    #[test]
    fn sessions_keep_their_own_leaves_under_a_shared_key() {
        const KEY: u64 = 0x5753_4642;
        let session = |name: &'static str| {
            let mut leaves = LeafRegistry::new();
            leaves.insert(KEY, Box::new(Tagged(name)));
            let runner = GenetAppRunner::new(
                fresh_dom(),
                first_view,
                First {
                    count: 0,
                    width: 0.0,
                },
            );
            ContainedSession::new(
                RunnerSurfaceSession::new(
                    descriptor(name),
                    runner,
                    |_: &First| SurfaceAvailability::Available,
                    |_: &mut First, _| {},
                    |_action: ()| Vec::new(),
                )
                .with_leaves(leaves),
            )
        };
        let mut sessions = [session("example.a"), session("example.b")];
        let painted: Vec<&'static str> = sessions
            .iter_mut()
            .map(|session| {
                session
                    .leaves()
                    .and_then(|leaves| leaves.get_mut_as::<Tagged>(&KEY))
                    .map(|leaf| leaf.0)
                    .expect("each session holds its leaf")
            })
            .collect();
        assert_eq!(painted, ["example.a", "example.b"]);

        let mut shared = LeafRegistry::new();
        shared.insert(KEY, Box::new(Tagged("example.a")));
        shared.insert(KEY, Box::new(Tagged("example.b")));
        assert_eq!(
            shared.get_mut_as::<Tagged>(&KEY).map(|leaf| leaf.0),
            Some("example.b"),
            "control: a shared registry lets the second session's leaf replace the first's"
        );
    }
}
