// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The wire retains routing and freshness; Hest carries command data only.

use crate::{
    AdvertisedAction, InstanceId, IntentInvocation, IntentResult, ProjectionSession, Revision,
    SceneEpoch,
};
use hest::{Invocation, Outcome};

impl AdvertisedAction {
    /// Compose opaque command data from this endpoint's advertised contract.
    pub fn command_invocation(&self, arguments: Vec<u8>) -> Invocation {
        Invocation {
            id: self.intent.0.clone(),
            schema_id: self.payload_schema.clone(),
            arguments,
            effect: self.effect.into(),
        }
    }
}

impl IntentInvocation {
    /// Bind command data to this protocol's observation, checking the current
    /// advertisement. Endpoint authority, payload decoding and target lifetime
    /// are still checked by the endpoint when it receives the wire request.
    pub fn from_command(
        command: &Invocation,
        advertised: &AdvertisedAction,
        session: ProjectionSession,
        target: InstanceId,
        observed_epoch: SceneEpoch,
        observed_revision: Revision,
    ) -> Result<Self, Outcome> {
        if command.id != advertised.intent.0
            || command.schema_id != advertised.payload_schema
            || command.effect != advertised.effect.into()
        {
            return Err(Outcome::Refused {
                reason: "command does not match the advertised contract".into(),
            });
        }
        Ok(Self {
            session,
            target,
            observed_epoch,
            observed_revision,
            intent: command.id.clone(),
            payload: command.arguments.clone(),
        })
    }

    /// Recover the command using the endpoint advertisement that supplies the
    /// schema and effect (the existing wire carries those in its offer).
    pub fn command_invocation(&self, advertised: &AdvertisedAction) -> Result<Invocation, Outcome> {
        if self.intent != advertised.intent.0 {
            return Err(Outcome::Refused {
                reason: "intent is not the advertised command".into(),
            });
        }
        Ok(advertised.command_invocation(self.payload.clone()))
    }
}

impl IntentResult {
    /// Validation/dispatch acceptance is pending, never terminal completion.
    pub fn command_outcome(&self) -> Option<Outcome> {
        match self {
            Self::Accepted => None,
            Self::Rejected { reason } => Some(Outcome::Refused {
                reason: reason.clone(),
            }),
            Self::Stale { .. } => Some(Outcome::Stale),
        }
    }

    /// Encode a refusal or stale result in the existing dispatch protocol.
    ///
    /// This protocol cannot encode terminal `Done`. A stale result also needs
    /// the endpoint's current observation; Hest cannot invent that context.
    pub fn from_command_outcome(
        outcome: Outcome,
        current: Option<(SceneEpoch, Revision)>,
    ) -> Option<Self> {
        match outcome {
            Outcome::Done => None,
            Outcome::Refused { reason } => Some(Self::Rejected { reason }),
            Outcome::Stale => current.map(|(current_epoch, current_revision)| Self::Stale {
                current_epoch,
                current_revision,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CarrierRequestBody, IntentEffect, IntentReference};

    fn offer() -> AdvertisedAction {
        AdvertisedAction {
            intent: IntentReference("host:edit".into()),
            label: "Edit".into(),
            explanation: "Host owned".into(),
            payload_schema: "host.edit/v1".into(),
            input_form: None,
            effect: IntentEffect::DomainTruth,
        }
    }

    #[test]
    fn opaque_command_round_trips_through_the_real_wire_with_context_intact() {
        let offer = offer();
        let command = offer.command_invocation(vec![0, 255, 128]);
        let wire = IntentInvocation::from_command(
            &command,
            &offer,
            ProjectionSession("live".into()),
            InstanceId(7),
            SceneEpoch(4),
            Revision(9),
        )
        .unwrap();
        let encoded = serde_json::to_vec(&CarrierRequestBody::Intent(wire.clone())).unwrap();
        let CarrierRequestBody::Intent(received) = serde_json::from_slice(&encoded).unwrap() else {
            panic!("intent request")
        };
        assert_eq!(received, wire);
        assert_eq!(received.command_invocation(&offer), Ok(command));
    }

    #[test]
    fn id_schema_and_effect_must_match_the_advertisement() {
        let offer = offer();
        for field in 0..3 {
            let mut command = offer.command_invocation(vec![]);
            match field {
                0 => command.id.push_str("-other"),
                1 => command.schema_id.push_str("-other"),
                _ => command.effect = hest::EffectClass::Curation,
            }
            assert!(matches!(
                IntentInvocation::from_command(
                    &command,
                    &offer,
                    ProjectionSession("live".into()),
                    InstanceId(7),
                    SceneEpoch(4),
                    Revision(9)
                ),
                Err(Outcome::Refused { .. })
            ));
        }
    }

    #[test]
    fn acceptance_never_becomes_done_and_done_never_becomes_acceptance() {
        assert_eq!(IntentResult::Accepted.command_outcome(), None);
        assert_eq!(
            IntentResult::from_command_outcome(Outcome::Done, None),
            None
        );
        let refused = IntentResult::Rejected {
            reason: "confirmation required".into(),
        };
        assert_eq!(
            IntentResult::from_command_outcome(refused.command_outcome().unwrap(), None),
            Some(refused)
        );
        let stale = IntentResult::Stale {
            current_epoch: SceneEpoch(5),
            current_revision: Revision(12),
        };
        assert_eq!(stale.command_outcome(), Some(Outcome::Stale));
        assert_eq!(
            IntentResult::from_command_outcome(Outcome::Stale, Some((SceneEpoch(5), Revision(12)))),
            Some(stale)
        );
        assert_eq!(
            IntentResult::from_command_outcome(Outcome::Stale, None),
            None
        );
    }
}
