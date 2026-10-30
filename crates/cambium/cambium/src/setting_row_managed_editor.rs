/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Existing Cambium controls with session edit observation at their message boundary.

use mere_surface_api::settings::SettingControl;
use meristem::{AnyView, MessageCtx, MessageResult, Mut, View, ViewMarker};

use super::{ManagedControl, ManagedLocal, SettingRowEvent, control_value};
use crate::component::ComponentView;
use crate::{GenetCtx, GenetElement, lens, radio_group, slider, text_field_typed, toggle};

type Child = Box<dyn AnyView<ManagedLocal, (), GenetCtx, GenetElement>>;

pub(super) fn editor(local: &ManagedLocal) -> ComponentView<ManagedLocal, SettingRowEvent> {
    let child: Child = match &local.control {
        ManagedControl::Text(_) => Box::new(lens(
            |input: &mut crate::TextInput| text_field_typed(input),
            |local: &mut ManagedLocal| match &mut local.control {
                ManagedControl::Text(input) => input,
                _ => unreachable!("text control retained its typed editor"),
            },
        )),
        ManagedControl::Number(_) => Box::new(lens(
            |control: &mut crate::Slider| slider(control),
            |local: &mut ManagedLocal| match &mut local.control {
                ManagedControl::Number(control) => control,
                _ => unreachable!("number control retained its typed editor"),
            },
        )),
        ManagedControl::Toggle(_) => Box::new(lens(
            |checked: &mut bool| toggle(*checked),
            |local: &mut ManagedLocal| match &mut local.control {
                ManagedControl::Toggle(checked) => checked,
                _ => unreachable!("toggle control retained its typed editor"),
            },
        )),
        ManagedControl::Choice(_) => {
            let labels: Vec<String> = match &local.observed_spec.control {
                SettingControl::Choice { options } => {
                    options.iter().map(|option| option.label.clone()).collect()
                },
                _ => Vec::new(),
            };
            Box::new(lens(
                move |choice: &mut crate::RadioGroup| radio_group(choice, &labels),
                |local: &mut ManagedLocal| match &mut local.control {
                    ManagedControl::Choice(choice) => choice,
                    _ => unreachable!("choice control retained its typed editor"),
                },
            ))
        },
        ManagedControl::Unsupported => Box::new(crate::el::<_, ManagedLocal, ()>(
            "div",
            "Unsupported control/value pair",
        )),
    };
    Box::new(ObserveEdits { child })
}

/// A small adapter around existing actionless controls. It compares typed
/// control values before and after a routed message, leaving widget navigation,
/// pointer phases, text selection, and accessibility behavior to the widgets.
struct ObserveEdits {
    child: Child,
}

impl ViewMarker for ObserveEdits {}

impl View<ManagedLocal, SettingRowEvent, GenetCtx> for ObserveEdits {
    type Element = GenetElement;
    type ViewState = <Child as View<ManagedLocal, (), GenetCtx>>::ViewState;

    fn build(
        &self,
        ctx: &mut GenetCtx,
        state: &mut ManagedLocal,
    ) -> (GenetElement, Self::ViewState) {
        self.child.as_ref().dyn_build(ctx, state)
    }

    fn rebuild(
        &self,
        previous: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut GenetCtx,
        element: Mut<'_, GenetElement>,
        state: &mut ManagedLocal,
    ) {
        self.child
            .as_ref()
            .dyn_rebuild(view_state, ctx, previous.child.as_ref(), element, state);
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut GenetCtx,
        element: Mut<'_, GenetElement>,
    ) {
        let _ = self.child.as_ref().dyn_teardown(view_state, ctx, element);
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        element: Mut<'_, GenetElement>,
        state: &mut ManagedLocal,
    ) -> MessageResult<SettingRowEvent> {
        let before = control_value(&state.baseline, &state.control);
        let result = self
            .child
            .as_ref()
            .dyn_message(view_state, message, element, state);
        let after = control_value(&state.baseline, &state.control);
        if before != after {
            if let Some(edit_generation) = state.session.record_edit(after.clone()) {
                return MessageResult::Action(SettingRowEvent::Edited {
                    value: after,
                    edit_generation,
                });
            }
        }
        match result {
            MessageResult::Action(()) => MessageResult::Nop,
            MessageResult::RequestRebuild => MessageResult::RequestRebuild,
            MessageResult::Nop => MessageResult::Nop,
            MessageResult::Stale => MessageResult::Stale,
        }
    }
}

#[cfg(test)]
#[path = "setting_row_managed_editor_tests.rs"]
mod tests;
