// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Destructive-action gate probe — every destructive `ActionDispatched`
//! must be preceded by a `ConfirmDialog` Confirmed dismissal.

use std::sync::Mutex;

use hest::menu::CommandSet;

use crate::ux_observability::{DismissReason, SurfaceId, UxEvent};

use super::{ProbeFailure, UxProbe};

/// Asserts that every [`UxEvent::ActionDispatched`] for a destructive
/// `ActionId` is preceded (as the most recent ConfirmDialog event) by
/// a `ConfirmDialog` Confirmed dismissal. Covers the §4.10 guarantee
/// that destructive actions (Delete node, Retract relation, ...) always carry
/// a confirmation step.
///
/// The offered command set supplies the destructive declaration. The host
/// still owns the confirmation gesture and action dispatch.
pub struct DestructiveActionGateProbe {
    commands: CommandSet,
    /// True if the most recent ConfirmDialog event was a Confirmed
    /// dismissal. Cleared by Cancelled / Superseded dismissals or by
    /// any subsequent ActionDispatched (the grant is consumed).
    confirm_grant: Mutex<bool>,
    failures: Mutex<Vec<ProbeFailure>>,
}

impl DestructiveActionGateProbe {
    pub fn new(commands: CommandSet) -> Self {
        Self {
            commands,
            confirm_grant: Mutex::new(false),
            failures: Mutex::new(Vec::new()),
        }
    }
}

impl UxProbe for DestructiveActionGateProbe {
    fn name(&self) -> &'static str {
        "destructive_action_gate"
    }

    fn observe(&self, event: &UxEvent) {
        match event {
            UxEvent::SurfaceDismissed {
                surface: SurfaceId::ConfirmDialog,
                reason,
            } => {
                let mut grant = self.confirm_grant.lock().unwrap();
                *grant = matches!(reason, DismissReason::Confirmed);
            },
            UxEvent::ActionDispatched { action_id, .. } => {
                let mut grant = self.confirm_grant.lock().unwrap();
                if self
                    .commands
                    .get(action_id.key())
                    .is_some_and(|command| command.destructive)
                    && !*grant
                {
                    self.failures.lock().unwrap().push(ProbeFailure {
                        probe_name: self.name(),
                        description: format!(
                            "destructive action {:?} dispatched without a \
                             preceding ConfirmDialog Confirmed dismissal",
                            action_id
                        ),
                        triggering_event: event.clone(),
                    });
                }
                // Either way, consume the grant. A lingering grant must
                // not authorise a later, unrelated destructive dispatch.
                *grant = false;
            },
            _ => {},
        }
    }

    fn drain_failures(&self) -> Vec<ProbeFailure> {
        std::mem::take(&mut *self.failures.lock().unwrap())
    }
}
