/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Opt-in managed settings rows with explicit draft and request lifecycle.

use std::fmt;

#[path = "setting_row_managed_editor.rs"]
mod setting_row_managed_editor;

use mere_surface_api::settings::{SettingControl, SettingSpec, SettingValue};
use meristem::View;

use crate::component::{ComponentView, component};
use crate::setting_row_session::{SettingRowPhase, SettingRowSession, SettingRowSnapshot};
use crate::{
    Action, GenetCtx, GenetElement, OptionalAction, PointerClick, RadioGroup, Slider, TextInput,
    button, el,
};

/// Row events translated by the product into its own command/owner vocabulary.
#[derive(Clone, PartialEq)]
pub enum SettingRowEvent {
    Edited {
        value: SettingValue,
        edit_generation: u64,
    },
    Apply {
        value: SettingValue,
        edit_generation: u64,
    },
    Discard,
    ReviewMine,
    RetryMine {
        value: SettingValue,
        edit_generation: u64,
    },
}

impl Action for SettingRowEvent {}

impl fmt::Debug for SettingRowEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edited {
                edit_generation, ..
            } => f
                .debug_struct("Edited")
                .field("value", &"<redacted>")
                .field("edit_generation", edit_generation)
                .finish(),
            Self::Apply {
                edit_generation, ..
            } => f
                .debug_struct("Apply")
                .field("value", &"<redacted>")
                .field("edit_generation", edit_generation)
                .finish(),
            Self::Discard => f.write_str("Discard"),
            Self::ReviewMine => f.write_str("ReviewMine"),
            Self::RetryMine {
                edit_generation, ..
            } => f
                .debug_struct("RetryMine")
                .field("value", &"<redacted>")
                .field("edit_generation", edit_generation)
                .finish(),
        }
    }
}

#[derive(Clone, PartialEq)]
struct ManagedProps {
    session: SettingRowSession,
    snapshot: SettingRowSnapshot,
    label: String,
}

enum ManagedControl {
    Text(TextInput),
    Number(Slider),
    Toggle(bool),
    Choice(RadioGroup),
    Unsupported,
}

struct ManagedLocal {
    baseline: SettingSpec,
    observed_spec: SettingSpec,
    control: ManagedControl,
    session: SettingRowSession,
}

fn number_range(min: Option<f64>, max: Option<f64>) -> (f64, f64) {
    let min = min.unwrap_or(0.0);
    let max = max
        .filter(|value| value.is_finite() && *value > min)
        .unwrap_or(min + 1.0);
    (min, max)
}

fn number_fraction(value: f64, min: Option<f64>, max: Option<f64>) -> f32 {
    let (low, high) = number_range(min, max);
    ((value - low) / (high - low)).clamp(0.0, 1.0) as f32
}

fn number_value(fraction: f32, min: Option<f64>, max: Option<f64>) -> f64 {
    let (low, high) = number_range(min, max);
    low + f64::from(fraction.clamp(0.0, 1.0)) * (high - low)
}

fn control_for(spec: &SettingSpec, value: &SettingValue) -> ManagedControl {
    match (&spec.control, value) {
        (SettingControl::Text, SettingValue::Text(value)) => {
            ManagedControl::Text(TextInput::new(value.clone()))
        },
        (SettingControl::Number { min, max, step }, SettingValue::Number(value)) => {
            let (low, high) = number_range(*min, *max);
            let step = step
                .map(|step| (step / (high - low)) as f32)
                .unwrap_or(0.01);
            ManagedControl::Number(
                Slider::new(number_fraction(*value, *min, *max))
                    .with_label(spec.label.clone())
                    .with_steps(step, (step * 5.0).max(0.1)),
            )
        },
        (SettingControl::Number { min, max, step }, SettingValue::Integer(value)) => {
            let (low, high) = number_range(*min, *max);
            let step = step
                .map(|step| (step / (high - low)) as f32)
                .unwrap_or(0.01);
            ManagedControl::Number(
                Slider::new(number_fraction(*value as f64, *min, *max))
                    .with_label(spec.label.clone())
                    .with_steps(step, (step * 5.0).max(0.1)),
            )
        },
        (SettingControl::Toggle, SettingValue::Boolean(value)) => ManagedControl::Toggle(*value),
        (SettingControl::Choice { options }, SettingValue::Text(value)) => {
            let selected = options
                .iter()
                .position(|option| option.value == *value)
                .unwrap_or(0);
            ManagedControl::Choice(RadioGroup::new(selected).with_label(spec.label.clone()))
        },
        _ => ManagedControl::Unsupported,
    }
}

fn control_value(spec: &SettingSpec, control: &ManagedControl) -> SettingValue {
    match control {
        ManagedControl::Text(input) => SettingValue::Text(input.text().to_owned()),
        ManagedControl::Number(input) => match &spec.control {
            SettingControl::Number { min, max, .. } => match spec.value {
                SettingValue::Integer(_) => {
                    SettingValue::Integer(number_value(input.value, *min, *max).round() as i64)
                },
                _ => SettingValue::Number(number_value(input.value, *min, *max)),
            },
            _ => spec.value.clone(),
        },
        ManagedControl::Toggle(value) => SettingValue::Boolean(*value),
        ManagedControl::Choice(choice) => match &spec.control {
            SettingControl::Choice { options } => SettingValue::Text(
                options
                    .get(choice.selected)
                    .or_else(|| options.first())
                    .map(|option| option.value.clone())
                    .unwrap_or_default(),
            ),
            _ => spec.value.clone(),
        },
        ManagedControl::Unsupported => spec.value.clone(),
    }
}

fn action_button(
    label: &'static str,
    class: &'static str,
    action: impl Fn(&mut ManagedLocal, PointerClick) -> Option<SettingRowEvent> + 'static,
) -> impl View<ManagedLocal, SettingRowEvent, GenetCtx, Element = GenetElement> {
    button(label, action).attr("class", class)
}

fn apply_button(
    setting_id: String,
    enabled: bool,
    action: impl Fn(&mut ManagedLocal, PointerClick) -> Option<SettingRowEvent> + 'static,
) -> impl View<ManagedLocal, SettingRowEvent, GenetCtx, Element = GenetElement> {
    button("Apply", action)
        .attr("class", "setting-apply")
        .attr("data-setting", setting_id)
        .attr("aria-disabled", if enabled { "false" } else { "true" })
}

fn discard_button(
    setting_id: String,
    action: impl Fn(&mut ManagedLocal, PointerClick) -> Option<SettingRowEvent> + 'static,
) -> impl View<ManagedLocal, SettingRowEvent, GenetCtx, Element = GenetElement> {
    button("Discard edit", action)
        .attr("class", "setting-discard")
        .attr("data-setting", setting_id)
}

fn visible_value(value: &SettingValue) -> String {
    match value {
        SettingValue::Text(value) => value.clone(),
        SettingValue::Integer(value) => value.to_string(),
        SettingValue::Number(value) => value.to_string(),
        SettingValue::Boolean(value) => value.to_string(),
    }
}

fn row_body(
    props: &ManagedProps,
    local: &ManagedLocal,
) -> ComponentView<ManagedLocal, SettingRowEvent> {
    let snapshot = props.session.snapshot();
    let enabled = snapshot.pending_generation.is_none()
        && snapshot.apply_compatible
        && matches!(
            snapshot.phase,
            SettingRowPhase::Clean | SettingRowPhase::Dirty
        );
    let pending = snapshot.pending_generation;
    let main = setting_row_managed_editor::editor(local);
    let name = el::<_, ManagedLocal, SettingRowEvent>("div", props.label.clone())
        .attr("class", "setting-label");
    let apply_session = props.session.clone();
    let apply = apply_button(local.baseline.id.clone(), enabled, move |local, _| {
        let current = apply_session.snapshot();
        let requestable = enabled
            && current.pending_generation.is_none()
            && current.apply_compatible
            && matches!(
                current.phase,
                SettingRowPhase::Clean | SettingRowPhase::Dirty
            );
        requestable.then(|| {
            let value = control_value(&local.baseline, &local.control);
            let edit_generation = current.edit_generation;
            SettingRowEvent::Apply {
                value,
                edit_generation,
            }
        })
    });
    let mut children: Vec<ComponentView<ManagedLocal, SettingRowEvent>> =
        vec![Box::new(name), main, Box::new(apply)];

    if snapshot.phase == SettingRowPhase::Conflict {
        let discard_session = props.session.clone();
        children.push(Box::new(action_button(
            "Use current value",
            "setting-use-current",
            move |_, _| {
                let current = discard_session.snapshot();
                (current.pending_generation.is_none() && current.phase == SettingRowPhase::Conflict)
                    .then_some(SettingRowEvent::Discard)
            },
        )));
        if snapshot.apply_compatible {
            let review_session = props.session.clone();
            children.push(Box::new(action_button(
                "Review my edit",
                "setting-review-mine",
                move |_, _| {
                    let current = review_session.snapshot();
                    (current.pending_generation.is_none()
                        && current.apply_compatible
                        && current.phase == SettingRowPhase::Conflict)
                        .then_some(SettingRowEvent::ReviewMine)
                },
            )));
        }
        children.push(Box::new(
            el::<_, ManagedLocal, SettingRowEvent>(
                "div",
                format!(
                    "Current value: {}; Your edit: {}",
                    visible_value(&snapshot.observed_spec.value),
                    visible_value(&snapshot.draft_value),
                ),
            )
            .attr("class", "setting-row-conflict-values"),
        ));
    }
    if pending.is_none()
        && matches!(
            snapshot.phase,
            SettingRowPhase::Dirty
                | SettingRowPhase::Failed
                | SettingRowPhase::Unavailable
                | SettingRowPhase::ReviewingMine
        )
    {
        let discard_session = props.session.clone();
        children.push(Box::new(discard_button(
            local.baseline.id.clone(),
            move |_, _| {
                let current = discard_session.snapshot();
                (current.pending_generation.is_none()
                    && matches!(
                        current.phase,
                        SettingRowPhase::Dirty
                            | SettingRowPhase::Failed
                            | SettingRowPhase::Unavailable
                            | SettingRowPhase::ReviewingMine
                    ))
                .then_some(SettingRowEvent::Discard)
            },
        )));
    }
    if snapshot.phase == SettingRowPhase::ReviewingMine
        || (snapshot.phase == SettingRowPhase::Failed && snapshot.apply_compatible)
    {
        let session = props.session.clone();
        let retry = action_button("Apply my edit", "setting-retry-mine", move |local, _| {
            let current = session.snapshot();
            let requestable = current.pending_generation.is_none()
                && current.apply_compatible
                && matches!(
                    current.phase,
                    SettingRowPhase::ReviewingMine | SettingRowPhase::Failed
                );
            requestable.then(|| {
                let value = control_value(&local.baseline, &local.control);
                SettingRowEvent::RetryMine {
                    edit_generation: current.edit_generation,
                    value,
                }
            })
        });
        children.push(Box::new(retry));
    }
    if let Some(failure) = snapshot.failure {
        children.push(Box::new(
            el::<_, ManagedLocal, SettingRowEvent>("div", failure)
                .attr("class", "setting-row-status"),
        ));
    }
    if pending.is_some() {
        children.push(Box::new(
            el::<_, ManagedLocal, SettingRowEvent>("div", "Applying…")
                .attr("class", "setting-row-applying"),
        ));
    }
    Box::new(
        el::<_, ManagedLocal, SettingRowEvent>("div", children)
            .attr("class", "setting-row setting-row-managed")
            .attr("data-setting", local.baseline.id.clone()),
    )
}

/// Build an opt-in managed row. Keep `session` stable for this component
/// instance; products call `observe`, `acknowledge`, `fail`, and `reset` at the
/// provider boundary. The legacy [`crate::setting_row`] API is unchanged.
pub fn setting_row_managed<State, A, Output, L, F>(
    session: &SettingRowSession,
    label: L,
    on_event: F,
) -> impl View<State, A, GenetCtx, Element = GenetElement> + use<State, A, Output, L, F>
where
    State: 'static,
    A: 'static,
    Output: OptionalAction<A> + 'static,
    L: Into<String>,
    F: Fn(&mut State, SettingRowEvent) -> Output + 'static,
{
    let snapshot = session.snapshot();
    let props = ManagedProps {
        session: session.clone(),
        snapshot: snapshot.clone(),
        label: label.into(),
    };
    component(
        props,
        |props: &ManagedProps| ManagedLocal {
            baseline: props.snapshot.baseline_spec.clone(),
            observed_spec: props.snapshot.observed_spec.clone(),
            control: control_for(&props.snapshot.baseline_spec, &props.snapshot.draft_value),
            session: props.session.clone(),
        },
        |prev: &ManagedProps, next: &ManagedProps, local: &mut ManagedLocal| {
            local.observed_spec = next.snapshot.observed_spec.clone();
            match &mut local.control {
                ManagedControl::Number(slider) => {
                    slider.label = next.snapshot.observed_spec.label.clone();
                },
                ManagedControl::Choice(group) => {
                    group.label = next.snapshot.observed_spec.label.clone();
                },
                _ => {},
            }
            if prev.snapshot.reset_generation != next.snapshot.reset_generation {
                local.baseline = next.snapshot.baseline_spec.clone();
                local.control =
                    control_for(&next.snapshot.baseline_spec, &next.snapshot.draft_value);
            }
        },
        row_body,
        move |state: &mut State, event: SettingRowEvent| on_event(state, event),
    )
    .memo()
}

#[cfg(test)]
#[path = "setting_row_managed_tests.rs"]
mod tests;
