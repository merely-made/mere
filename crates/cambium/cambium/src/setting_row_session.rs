/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Ephemeral admission metadata for one managed settings-row instance.
//!
//! This session stores value snapshots and generations, not a control's local
//! editor state. TextInput, Slider, Toggle, and RadioGroup remain owned by the
//! retained component. A product supplies snapshots and adjudicates requests.

use std::{cell::RefCell, fmt, rc::Rc};

use mere_surface_api::settings::{SettingControl, SettingSpec, SettingValue};

/// Presentation/admission phase for one row session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingRowPhase {
    Clean,
    Dirty,
    Applying,
    Conflict,
    ReviewingMine,
    Failed,
    Unavailable,
}

/// Why an attempted Apply did not commit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingRowFailureKind {
    Conflict,
    Persistence,
    Validation,
    Unavailable,
}

/// Result when an acknowledged edit is admitted into the current session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingRowAckDisposition {
    Clean,
    KeepNewerDraft,
}

/// Result of observing a new provider snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingRowObservation {
    AdoptedClean,
    PreservedDraft,
    IncompatibleDraft,
}

/// Read-only copy of session metadata. Debug output redacts both setting
/// values; clients that need them must access the explicit fields directly.
#[derive(Clone, PartialEq)]
pub struct SettingRowSnapshot {
    pub baseline_spec: SettingSpec,
    pub observed_spec: SettingSpec,
    pub draft_value: SettingValue,
    pub dirty: bool,
    pub edit_generation: u64,
    pub reset_generation: u64,
    pub phase: SettingRowPhase,
    pub apply_compatible: bool,
    pub pending_generation: Option<u64>,
    pub failure: Option<String>,
}

impl fmt::Debug for SettingRowSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SettingRowSnapshot")
            .field("setting_id", &self.baseline_spec.id)
            .field("draft_value", &"<redacted>")
            .field(
                "dirty",
                &session_is_dirty(&self.draft_value, &self.baseline_spec.value, self.phase),
            )
            .field("edit_generation", &self.edit_generation)
            .field("reset_generation", &self.reset_generation)
            .field("phase", &self.phase)
            .field("apply_compatible", &self.apply_compatible)
            .field("pending_generation", &self.pending_generation)
            .field("failure", &self.failure.as_ref().map(|_| "<redacted>"))
            .finish()
    }
}

#[derive(Clone)]
pub struct SettingRowSession(Rc<RefCell<SessionState>>);

struct SessionState {
    baseline_spec: SettingSpec,
    observed_spec: SettingSpec,
    draft_value: SettingValue,
    edit_generation: u64,
    reset_generation: u64,
    phase: SettingRowPhase,
    apply_compatible: bool,
    pending_generation: Option<u64>,
    failure: Option<String>,
}

impl fmt::Debug for SettingRowSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.0.borrow();
        f.debug_struct("SettingRowSession")
            .field("setting_id", &state.baseline_spec.id)
            .field("draft_value", &"<redacted>")
            .field("edit_generation", &state.edit_generation)
            .field("reset_generation", &state.reset_generation)
            .field("phase", &state.phase)
            .field("apply_compatible", &state.apply_compatible)
            .field("pending_generation", &state.pending_generation)
            .finish()
    }
}

impl PartialEq for SettingRowSession {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl SettingRowSession {
    /// Create a distinct ephemeral session for one live component instance.
    pub fn new(spec: SettingSpec) -> Self {
        Self(Rc::new(RefCell::new(SessionState {
            baseline_spec: spec.clone(),
            observed_spec: spec.clone(),
            draft_value: spec.value.clone(),
            edit_generation: 0,
            reset_generation: 0,
            phase: SettingRowPhase::Clean,
            apply_compatible: true,
            pending_generation: None,
            failure: None,
        })))
    }

    /// Current session metadata. Control-specific editor state is not stored here.
    pub fn snapshot(&self) -> SettingRowSnapshot {
        let state = self.0.borrow();
        SettingRowSnapshot {
            baseline_spec: state.baseline_spec.clone(),
            observed_spec: state.observed_spec.clone(),
            draft_value: state.draft_value.clone(),
            dirty: session_is_dirty(&state.draft_value, &state.baseline_spec.value, state.phase),
            edit_generation: state.edit_generation,
            reset_generation: state.reset_generation,
            phase: state.phase,
            apply_compatible: state.apply_compatible,
            pending_generation: state.pending_generation,
            failure: state.failure.clone(),
        }
    }

    /// Observe a coherent provider snapshot. Clean sessions adopt it; dirty
    /// sessions retain their original editor and write baseline. A changed
    /// id/control disables Apply until an explicit reset.
    pub fn observe(&self, spec: SettingSpec) -> SettingRowObservation {
        let mut state = self.0.borrow_mut();
        state.observed_spec = spec.clone();
        if state.pending_generation.is_some() {
            state.apply_compatible = state.baseline_spec.id == spec.id
                && controls_compatible(&state.baseline_spec.control, &spec.control);
            return if state.apply_compatible {
                SettingRowObservation::PreservedDraft
            } else {
                SettingRowObservation::IncompatibleDraft
            };
        }
        if state.phase == SettingRowPhase::Unavailable {
            let same_target = state.baseline_spec.id == spec.id
                && controls_compatible(&state.baseline_spec.control, &spec.control);
            if same_target && state.draft_value == state.baseline_spec.value {
                let reset_control = editor_spec_changed(&state.baseline_spec, &spec);
                state.baseline_spec = spec.clone();
                state.draft_value = spec.value.clone();
                state.phase = SettingRowPhase::Clean;
                state.apply_compatible = true;
                state.failure = None;
                if reset_control {
                    state.reset_generation = state.reset_generation.saturating_add(1);
                }
                return SettingRowObservation::AdoptedClean;
            }
            state.apply_compatible = same_target;
            return if same_target {
                SettingRowObservation::PreservedDraft
            } else {
                SettingRowObservation::IncompatibleDraft
            };
        }
        if state.phase == SettingRowPhase::Clean {
            if state.baseline_spec != spec {
                let reset_control = editor_spec_changed(&state.baseline_spec, &spec);
                state.baseline_spec = spec.clone();
                state.draft_value = spec.value.clone();
                if reset_control {
                    state.reset_generation = state.reset_generation.saturating_add(1);
                }
            }
            state.apply_compatible = true;
            state.failure = None;
            return SettingRowObservation::AdoptedClean;
        }
        if state.draft_value == state.baseline_spec.value
            && !matches!(
                state.phase,
                SettingRowPhase::Conflict | SettingRowPhase::ReviewingMine
            )
        {
            let reset_control = editor_spec_changed(&state.baseline_spec, &spec);
            state.phase = SettingRowPhase::Clean;
            state.baseline_spec = spec.clone();
            state.draft_value = spec.value.clone();
            if reset_control {
                state.reset_generation = state.reset_generation.saturating_add(1);
            }
            state.apply_compatible = true;
            state.failure = None;
            return SettingRowObservation::AdoptedClean;
        }
        state.apply_compatible = state.baseline_spec.id == spec.id
            && controls_compatible(&state.baseline_spec.control, &spec.control);
        if state.apply_compatible {
            SettingRowObservation::PreservedDraft
        } else {
            SettingRowObservation::IncompatibleDraft
        }
    }

    /// Explicitly discard local work and adopt `spec`, invalidating pending acks.
    pub fn reset(&self, spec: SettingSpec) {
        let mut state = self.0.borrow_mut();
        state.baseline_spec = spec.clone();
        state.observed_spec = spec.clone();
        state.draft_value = spec.value.clone();
        state.edit_generation = state.edit_generation.saturating_add(1);
        state.reset_generation = state.reset_generation.saturating_add(1);
        state.phase = SettingRowPhase::Clean;
        state.apply_compatible = true;
        state.pending_generation = None;
        state.failure = None;
    }

    /// Explicitly choose the latest observed provider value.
    pub fn reset_to_observed(&self) {
        let spec = self.0.borrow().observed_spec.clone();
        self.reset(spec);
    }

    /// Record a real value change. Equal-value navigation is ignored; editing
    /// back to baseline makes the session clean without a provider write.
    pub fn record_edit(&self, value: SettingValue) -> Option<u64> {
        let mut state = self.0.borrow_mut();
        if !value_matches_control(&state.baseline_spec.control, &value)
            || value == state.draft_value
        {
            return None;
        }
        let previous_phase = state.phase;
        state.edit_generation = state.edit_generation.saturating_add(1);
        state.draft_value = value;
        match previous_phase {
            SettingRowPhase::Conflict
            | SettingRowPhase::ReviewingMine
            | SettingRowPhase::Unavailable => {},
            _ => {
                state.failure = None;
                state.phase = if state.draft_value == state.baseline_spec.value {
                    SettingRowPhase::Clean
                } else {
                    SettingRowPhase::Dirty
                };
            },
        }
        Some(state.edit_generation)
    }

    /// Start a single Apply generation. The product still owns request id and
    /// revision guard validation; this is only local presentation admission.
    pub fn begin_apply(&self, edit_generation: u64) -> bool {
        let mut state = self.0.borrow_mut();
        if !state.apply_compatible
            || state.pending_generation.is_some()
            || state.edit_generation != edit_generation
            || !matches!(
                state.phase,
                SettingRowPhase::Clean
                    | SettingRowPhase::Dirty
                    | SettingRowPhase::ReviewingMine
                    | SettingRowPhase::Failed
            )
        {
            return false;
        }
        state.pending_generation = Some(edit_generation);
        state.phase = SettingRowPhase::Applying;
        state.failure = None;
        true
    }

    /// Accept a provider-confirmed canonical value only for the pending edit.
    /// A newer local edit stays dirty while the acknowledged baseline advances.
    pub fn acknowledge(
        &self,
        edit_generation: u64,
        canonical_spec: SettingSpec,
    ) -> Option<SettingRowAckDisposition> {
        let mut state = self.0.borrow_mut();
        if state.pending_generation != Some(edit_generation)
            || canonical_spec.id != state.baseline_spec.id
            || !controls_compatible(&canonical_spec.control, &state.baseline_spec.control)
            || !value_matches_control(&canonical_spec.control, &canonical_spec.value)
        {
            return None;
        }
        let observed_unchanged = state.observed_spec == state.baseline_spec;
        state.baseline_spec = canonical_spec.clone();
        if observed_unchanged {
            state.observed_spec = canonical_spec.clone();
        }
        state.pending_generation = None;
        state.apply_compatible = state.observed_spec.id == state.baseline_spec.id
            && controls_compatible(&state.observed_spec.control, &state.baseline_spec.control);
        state.failure = None;
        if state.edit_generation == edit_generation {
            if state.draft_value != canonical_spec.value {
                state.reset_generation = state.reset_generation.saturating_add(1);
            }
            state.draft_value = canonical_spec.value;
            state.phase = SettingRowPhase::Clean;
            Some(SettingRowAckDisposition::Clean)
        } else {
            state.phase = if state.draft_value == state.baseline_spec.value {
                SettingRowPhase::Clean
            } else {
                SettingRowPhase::Dirty
            };
            Some(SettingRowAckDisposition::KeepNewerDraft)
        }
    }

    /// Record a failed request without changing the draft or baseline.
    pub fn fail(&self, kind: SettingRowFailureKind, message: impl Into<String>) {
        let mut state = self.0.borrow_mut();
        state.pending_generation = None;
        state.failure = Some(message.into());
        state.phase = match kind {
            SettingRowFailureKind::Conflict => SettingRowPhase::Conflict,
            SettingRowFailureKind::Unavailable => SettingRowPhase::Unavailable,
            SettingRowFailureKind::Persistence | SettingRowFailureKind::Validation => {
                SettingRowPhase::Failed
            },
        };
    }

    /// Mark the row unavailable while preserving any current draft.
    pub fn set_unavailable(&self, reason: impl Into<String>) {
        let mut state = self.0.borrow_mut();
        state.phase = SettingRowPhase::Unavailable;
        state.failure = Some(reason.into());
    }

    /// Resume after the product has confirmed the same owner incarnation is
    /// available again. This does not adopt a snapshot or change the baseline.
    pub fn resume_available(&self) -> bool {
        let mut state = self.0.borrow_mut();
        if state.phase != SettingRowPhase::Unavailable
            || !state.apply_compatible
            || state.pending_generation.is_some()
        {
            return false;
        }
        state.failure = None;
        state.phase = if state.draft_value == state.baseline_spec.value {
            SettingRowPhase::Clean
        } else {
            SettingRowPhase::Dirty
        };
        true
    }

    /// Move from a conflict prompt to explicit review of the retained draft.
    pub fn review_mine(&self) -> bool {
        let mut state = self.0.borrow_mut();
        if state.phase != SettingRowPhase::Conflict || !state.apply_compatible {
            return false;
        }
        state.phase = SettingRowPhase::ReviewingMine;
        true
    }
}

fn value_matches_control(control: &SettingControl, value: &SettingValue) -> bool {
    match (control, value) {
        (SettingControl::Text, SettingValue::Text(_))
        | (SettingControl::Choice { .. }, SettingValue::Text(_))
        | (SettingControl::Toggle, SettingValue::Boolean(_)) => true,
        (SettingControl::Number { .. }, SettingValue::Number(value)) => value.is_finite(),
        (SettingControl::Number { .. }, SettingValue::Integer(_)) => true,
        _ => false,
    }
}

fn editor_spec_changed(previous: &SettingSpec, next: &SettingSpec) -> bool {
    previous.id != next.id
        || previous.value != next.value
        || !controls_compatible(&previous.control, &next.control)
}

fn session_is_dirty(draft: &SettingValue, baseline: &SettingValue, phase: SettingRowPhase) -> bool {
    draft != baseline
        || matches!(
            phase,
            SettingRowPhase::Conflict | SettingRowPhase::ReviewingMine
        )
}

/// Choice labels are presentation metadata; ordered values define selection
/// identity. All other controls retain their full current compatibility rule.
fn controls_compatible(left: &SettingControl, right: &SettingControl) -> bool {
    match (left, right) {
        (SettingControl::Choice { options: a }, SettingControl::Choice { options: b }) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| a.value == b.value)
        },
        _ => left == right,
    }
}
