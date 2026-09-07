/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A native numeric-value event view for accessible range controls.

use core::marker::PhantomData;

use genet_scripted_dom::NodeId;
use meristem::{MessageCtx, MessageResult, Mut, View, ViewId, ViewMarker, ViewPathTracker};

use crate::pod::GenetElement;
use crate::{ElementView, GenetCtx, OptionalAction};

const ON_VALUE_ID: ViewId = ViewId::new(0x5641_4C55);

/// A finite numeric value requested by an accessibility host.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ValueEvent {
    /// The requested value. Hosts must not route non-finite values.
    pub value: f64,
}

/// Wraps an element view with an accessible numeric-value handler.
pub struct OnValue<V, State, Action, F> {
    child: V,
    handler: F,
    phantom: PhantomData<fn() -> (State, Action)>,
}

/// Attach a handler for a finite numeric value requested through accessibility.
pub fn on_value<V, State, Action, OA, F>(child: V, handler: F) -> OnValue<V, State, Action, F>
where
    State: 'static,
    Action: 'static,
    V: ElementView<State, Action>,
    OA: OptionalAction<Action>,
    F: Fn(&mut State, ValueEvent) -> OA + 'static,
{
    OnValue {
        child,
        handler,
        phantom: PhantomData,
    }
}

/// Retained state for [`OnValue`].
pub struct OnValueState<S> {
    child_state: S,
    node: NodeId,
    path: Vec<ViewId>,
}

impl<V, State, Action, F> ViewMarker for OnValue<V, State, Action, F> {}

impl<V, State, Action, OA, F> View<State, Action, GenetCtx> for OnValue<V, State, Action, F>
where
    State: 'static,
    Action: 'static,
    V: ElementView<State, Action>,
    OA: OptionalAction<Action>,
    F: Fn(&mut State, ValueEvent) -> OA + 'static,
{
    type ViewState = OnValueState<V::ViewState>;
    type Element = GenetElement;

    fn build(&self, ctx: &mut GenetCtx, app_state: &mut State) -> (Self::Element, Self::ViewState) {
        ctx.with_id(ON_VALUE_ID, |ctx| {
            let (element, child_state) = self.child.build(ctx, app_state);
            let node = element.node;
            let path = ctx.view_path().to_vec();
            ctx.register_value(node, path.clone());
            (
                element,
                OnValueState {
                    child_state,
                    node,
                    path,
                },
            )
        })
    }

    fn rebuild(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut GenetCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) {
        ctx.with_id(ON_VALUE_ID, |ctx| {
            let prev_node = view_state.node;
            self.child.rebuild(
                &prev.child,
                &mut view_state.child_state,
                ctx,
                element.reborrow_mut(),
                app_state,
            );
            let node = *element.node;
            if node != prev_node || ctx.view_path() != view_state.path.as_slice() {
                ctx.unregister_value(prev_node);
                let path = ctx.view_path().to_vec();
                ctx.register_value(node, path.clone());
                view_state.node = node;
                view_state.path = path;
            }
        });
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut GenetCtx,
        element: Mut<'_, Self::Element>,
    ) {
        ctx.with_id(ON_VALUE_ID, |ctx| {
            ctx.unregister_value(view_state.node);
            self.child
                .teardown(&mut view_state.child_state, ctx, element);
        });
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        let Some(first) = message.take_first() else {
            return MessageResult::Stale;
        };
        if first != ON_VALUE_ID {
            return MessageResult::Stale;
        }
        if message.remaining_path().is_empty() {
            match message.take_message::<ValueEvent>() {
                Some(event) => match (self.handler)(app_state, *event).action() {
                    Some(action) => MessageResult::Action(action),
                    None => MessageResult::Nop,
                },
                None => MessageResult::Stale,
            }
        } else {
            self.child
                .message(&mut view_state.child_state, message, element, app_state)
        }
    }
}

impl<V, State, Action, OA, F> ElementView<State, Action> for OnValue<V, State, Action, F>
where
    State: 'static,
    Action: 'static,
    V: ElementView<State, Action>,
    OA: OptionalAction<Action>,
    F: Fn(&mut State, ValueEvent) -> OA + 'static,
{
}
