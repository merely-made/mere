// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Invocation data and terminal outcomes. Hosts own decoding and enactment.

use crate::EffectClass;

/// A request to invoke a declared command, not permission to execute it.
///
/// The host defines the versioned argument schema and validates these bytes
/// against its current declaration. Routing, target lifetime, confirmation and
/// authority remain outside this record. The effect is a declaration to check,
/// not an authorization supplied by the caller.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Invocation {
    pub id: String,
    pub schema_id: String,
    pub arguments: Vec<u8>,
    pub effect: EffectClass,
}

/// A terminal result established by the host that owns the operation.
///
/// Dispatch, queuing, a handled event and admission are not `Done`. A host
/// with deferred work returns no terminal outcome until it knows the result.
/// `Stale` means the host's target or observation is no longer usable; any
/// replacement target, epoch or revision belongs to that host's protocol.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Outcome {
    Done,
    Refused { reason: String },
    Stale,
}

impl crate::menu::Command {
    /// Form an invocation using this declaration's id and effect class.
    ///
    /// This does not admit the request or bypass `disabled_reason`; the host
    /// must recheck its current declaration and policy when receiving it.
    pub fn invocation(&self, schema_id: impl Into<String>, arguments: Vec<u8>) -> Invocation {
        Invocation {
            id: self.id.clone(),
            schema_id: schema_id.into(),
            arguments,
            effect: self.effect,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invocation_preserves_opaque_bytes_and_declared_effect() {
        let command =
            crate::menu::Command::new("host:write", "Write", "host", EffectClass::DomainTruth);
        let invocation = command.invocation("host.write/v1", vec![0, 255, 128]);
        let encoded = serde_json::to_vec(&invocation).unwrap();
        assert_eq!(
            serde_json::from_slice::<Invocation>(&encoded).unwrap(),
            invocation
        );
        assert_eq!(invocation.id, command.id);
        assert_eq!(invocation.effect, command.effect);
    }

    #[test]
    fn terminal_outcomes_round_trip_without_an_admission_success_variant() {
        for outcome in [
            Outcome::Done,
            Outcome::Refused {
                reason: "confirmation required".into(),
            },
            Outcome::Stale,
        ] {
            let encoded = serde_json::to_vec(&outcome).unwrap();
            assert_eq!(
                serde_json::from_slice::<Outcome>(&encoded).unwrap(),
                outcome
            );
        }
    }
}
