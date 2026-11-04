/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A state-owning boundary for reusable Cambium controls.
//!
//! A component receives ordinary parent-owned props, retains its own local
//! state, and emits a typed event which the parent translates into its action
//! vocabulary. The erasure is deliberately narrow: [`AnyView`] hides the
//! concrete child tree while `Local` and `Event` stay statically typed.
//!
//! This belongs in Cambium rather than Meristem. It fixes the backend to
//! [`GenetCtx`] and [`GenetElement`], and its optional probe identity is a DOM
//! attribute. Meristem remains the renderer-neutral diff and message core.

use std::{marker::PhantomData, sync::OnceLock};

use layout_dom_api::{LayoutDom, LayoutDomMut};
use meristem::{AnyView, MessageCtx, MessageResult, Mut, View, ViewMarker};

use crate::reactive_diagnostics::{DiagnosticDetails, DiagnosticKind, DiagnosticReason, Producer};
use crate::{GenetCtx, GenetElement, OptionalAction, attr_qual};

static REACTIVE_PRODUCER: OnceLock<Option<Producer>> = OnceLock::new();

fn trace_component(kind: DiagnosticKind, reason: DiagnosticReason, path_depth: usize) {
    let producer = REACTIVE_PRODUCER.get_or_init(|| Producer::new().ok());
    if let Some(producer) = producer {
        let _ = producer.emit(
            kind,
            None,
            DiagnosticDetails {
                path_depth: Some(u16::try_from(path_depth).unwrap_or(u16::MAX)),
                path_truncated: path_depth > usize::from(u16::MAX),
                reason,
                ..DiagnosticDetails::default()
            },
        );
    }
}

/// The DOM attribute stamped by [`Component::probe_id`].
///
/// It is automation metadata only. Keyed sequence identity and Meristem message
/// routing continue to use their existing typed identities.
pub const COMPONENT_PROBE_ATTR: &str = "data-cambium-component";

/// The erased child tree retained inside a [`Component`].
pub type ComponentView<Local, Event> = Box<dyn AnyView<Local, Event, GenetCtx, GenetElement>>;

/// A reusable control boundary with props in, retained local state, and typed
/// events out.
///
/// `reconcile` runs on every parent rebuild and receives both the previous and
/// current props. It should update only parent-controlled axes of `Local`.
/// Component-owned interaction state should remain untouched when its props did
/// not change.
#[must_use = "View values do nothing unless provided to a Cambium runner"]
pub struct Component<Props, Local, Event, State, Action, Output, Init, Reconcile, Body, OnEvent> {
    props: Props,
    init: Init,
    reconcile: Reconcile,
    body: Body,
    on_event: OnEvent,
    probe_id: Option<String>,
    // A stored fn pointer rather than a `Props: PartialEq` bound on the View
    // impl: only `memo()` requires equality, so un-memoized components keep
    // working with non-comparable props (closures, trait objects).
    props_unchanged: Option<fn(&Props, &Props) -> bool>,
    phantom: PhantomData<fn(Local, Event, State) -> (Action, Output)>,
}

/// Construct a state-owning Cambium component.
///
/// `body` receives the current props and local state and returns one erased
/// child view. `on_event` is the sole point where the child's event vocabulary
/// meets the parent's state and action vocabulary. Like Cambium's native event
/// handlers, it may return `()`, an `Action`, or an `Option<Action>`.
pub fn component<Props, Local, Event, State, Action, Output, Init, Reconcile, Body, OnEvent>(
    props: Props,
    init: Init,
    reconcile: Reconcile,
    body: Body,
    on_event: OnEvent,
) -> Component<Props, Local, Event, State, Action, Output, Init, Reconcile, Body, OnEvent>
where
    Local: 'static,
    Event: 'static,
    State: 'static,
    Action: 'static,
    Init: Fn(&Props) -> Local + 'static,
    Reconcile: Fn(&Props, &Props, &mut Local) + 'static,
    Body: Fn(&Props, &Local) -> ComponentView<Local, Event> + 'static,
    Output: OptionalAction<Action> + 'static,
    OnEvent: Fn(&mut State, Event) -> Output + 'static,
{
    Component {
        props,
        init,
        reconcile,
        body,
        on_event,
        probe_id: None,
        props_unchanged: None,
        phantom: PhantomData,
    }
}

impl<Props, Local, Event, State, Action, Output, Init, Reconcile, Body, OnEvent>
    Component<Props, Local, Event, State, Action, Output, Init, Reconcile, Body, OnEvent>
{
    /// Stamp a caller-owned, DOM-visible identity on the component root for
    /// `taproot` attribute selectors.
    #[must_use]
    pub fn probe_id(mut self, id: impl Into<String>) -> Self {
        self.probe_id = Some(id.into());
        self
    }

    /// Skip `reconcile`, `body`, and the child rebuild when the props are
    /// unchanged and no message has touched the local state since the last
    /// rebuild.
    ///
    /// This is the memo boundary: correct only because a component's child
    /// tree depends on nothing but its props and its local state. A message
    /// delivered to the subtree marks the local state dirty, so an
    /// interaction always renders even under equal props.
    #[must_use]
    pub fn memo(mut self) -> Self
    where
        Props: PartialEq,
    {
        self.props_unchanged = Some(|prev, next| prev == next);
        self
    }
}

/// Retained state for [`Component`]. Public only because it is a `View`
/// associated type.
#[doc(hidden)]
pub struct ComponentState<Local: 'static, Event: 'static> {
    local: Local,
    child: ComponentView<Local, Event>,
    child_state: <ComponentView<Local, Event> as View<Local, Event, GenetCtx>>::ViewState,
    /// Set when a message reaches this subtree (it may have mutated `local`),
    /// cleared when a rebuild has run `body`. Gates the `memo()` skip.
    local_dirty: bool,
    /// The placement path at which the retained child last registered routes.
    route_path: Vec<meristem::ViewId>,
}

impl<Props, Local, Event, State, Action, Output, Init, Reconcile, Body, OnEvent> ViewMarker
    for Component<Props, Local, Event, State, Action, Output, Init, Reconcile, Body, OnEvent>
{
}

impl<Props, Local, Event, State, Action, Output, Init, Reconcile, Body, OnEvent>
    View<State, Action, GenetCtx>
    for Component<Props, Local, Event, State, Action, Output, Init, Reconcile, Body, OnEvent>
where
    Props: 'static,
    Local: 'static,
    Event: 'static,
    State: 'static,
    Action: 'static,
    Init: Fn(&Props) -> Local + 'static,
    Reconcile: Fn(&Props, &Props, &mut Local) + 'static,
    Body: Fn(&Props, &Local) -> ComponentView<Local, Event> + 'static,
    Output: OptionalAction<Action> + 'static,
    OnEvent: Fn(&mut State, Event) -> Output + 'static,
{
    type Element = GenetElement;
    type ViewState = ComponentState<Local, Event>;

    fn build(
        &self,
        ctx: &mut GenetCtx,
        _app_state: &mut State,
    ) -> (Self::Element, Self::ViewState) {
        trace_component(
            DiagnosticKind::ComponentBuilt,
            DiagnosticReason::Accepted,
            ctx.current_view_path().len(),
        );
        let mut local = (self.init)(&self.props);
        let child = (self.body)(&self.props, &local);
        let (element, child_state) = child.build(ctx, &mut local);
        apply_probe_id(&element.dom, element.node, self.probe_id.as_deref());
        (
            element,
            ComponentState {
                local,
                child,
                child_state,
                local_dirty: false,
                route_path: ctx.current_view_path().to_vec(),
            },
        )
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut GenetCtx,
        mut element: Mut<'_, Self::Element>,
        _app_state: &mut State,
    ) {
        trace_component(
            DiagnosticKind::ComponentRebuildStarted,
            if ctx.route_refresh_requested() {
                DiagnosticReason::RouteChanged
            } else {
                DiagnosticReason::None
            },
            ctx.current_view_path().len(),
        );
        if ctx.route_refresh_requested() {
            // A host placement walk is routing-only even for components that
            // are not memoized. Preserve the retained factory result and local
            // state, and let each nested component make the same decision.
            state.child.rebuild(
                &state.child,
                &mut state.child_state,
                ctx,
                element.reborrow_mut(),
                &mut state.local,
            );
            state.route_path = ctx.current_view_path().to_vec();
            apply_probe_id(&element.dom, *element.node, self.probe_id.as_deref());
            trace_component(
                DiagnosticKind::ComponentRouteRefreshed,
                DiagnosticReason::RouteChanged,
                ctx.current_view_path().len(),
            );
            return;
        }
        if let Some(unchanged) = self.props_unchanged {
            if unchanged(&prev.props, &self.props)
                && !state.local_dirty
                && state.route_path == ctx.current_view_path()
            {
                // The probe stamp is cheap and idempotent, and `probe_id` is
                // not covered by the props comparison, so keep it honest.
                apply_probe_id(&element.dom, *element.node, self.probe_id.as_deref());
                trace_component(
                    DiagnosticKind::ComponentMemoSkipped,
                    DiagnosticReason::EqualAndClean,
                    ctx.current_view_path().len(),
                );
                return;
            }
            if unchanged(&prev.props, &self.props)
                && !state.local_dirty
                && state.route_path != ctx.current_view_path()
            {
                // Route-only traversal: re-enter the retained child at its
                // current path. In particular, do not call reconcile/body or
                // mutate local state because placement changed.
                state.child.rebuild(
                    &state.child,
                    &mut state.child_state,
                    ctx,
                    element.reborrow_mut(),
                    &mut state.local,
                );
                state.route_path = ctx.current_view_path().to_vec();
                apply_probe_id(&element.dom, *element.node, self.probe_id.as_deref());
                trace_component(
                    DiagnosticKind::ComponentRouteRefreshed,
                    DiagnosticReason::RouteChanged,
                    ctx.current_view_path().len(),
                );
                return;
            }
        }
        let was_local_dirty = state.local_dirty;
        (self.reconcile)(&prev.props, &self.props, &mut state.local);
        let child = (self.body)(&self.props, &state.local);
        child.rebuild(
            &state.child,
            &mut state.child_state,
            ctx,
            element.reborrow_mut(),
            &mut state.local,
        );
        state.child = child;
        state.local_dirty = false;
        state.route_path = ctx.current_view_path().to_vec();
        apply_probe_id(&element.dom, *element.node, self.probe_id.as_deref());
        trace_component(
            DiagnosticKind::ComponentRebuildCompleted,
            if was_local_dirty {
                DiagnosticReason::LocalStateDirty
            } else {
                DiagnosticReason::DependencyChanged
            },
            ctx.current_view_path().len(),
        );
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut GenetCtx,
        element: Mut<'_, Self::Element>,
    ) {
        trace_component(
            DiagnosticKind::ComponentTornDown,
            DiagnosticReason::Teardown,
            ctx.current_view_path().len(),
        );
        state.child.teardown(&mut state.child_state, ctx, element);
    }

    fn message(
        &self,
        state: &mut Self::ViewState,
        message: &mut MessageCtx,
        element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        trace_component(
            DiagnosticKind::ComponentMessageRouted,
            DiagnosticReason::None,
            message.current_path().len(),
        );
        // Any delivered message may have mutated `local` through a handler, so
        // the next rebuild must run `body` even under `memo()` with equal props.
        state.local_dirty = true;
        match state
            .child
            .message(&mut state.child_state, message, element, &mut state.local)
        {
            MessageResult::Action(event) => match (self.on_event)(app_state, event).action() {
                Some(action) => MessageResult::Action(action),
                None => MessageResult::Nop,
            },
            MessageResult::RequestRebuild => MessageResult::RequestRebuild,
            MessageResult::Nop => MessageResult::Nop,
            MessageResult::Stale => MessageResult::Stale,
        }
    }
}

fn apply_probe_id(dom: &crate::DomHandle, node: genet_scripted_dom::NodeId, id: Option<&str>) {
    let mut dom = dom.borrow_mut();
    let qual = attr_qual(COMPONENT_PROBE_ATTR);
    // This runs on every rebuild, and `set_attribute` records a mutation even
    // for an unchanged value (faithful MutationObserver semantics), which
    // would feed a spurious restyle per rebuild. Skip when already stamped.
    if dom.attribute(node, &qual.ns, &qual.local) == id {
        return;
    }
    match id {
        Some(id) => dom.set_attribute(node, qual, id),
        None => dom.remove_attribute(node, qual),
    }
}

#[cfg(test)]
#[path = "component/tests.rs"]
mod tests;
