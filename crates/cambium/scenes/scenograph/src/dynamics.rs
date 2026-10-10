// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The opaque dynamics carrier (F192). Only the binding host interprets the
//! specification. Scenograph checks the JSON envelope, without depending on a
//! physics engine or deciding which terms a host can run.

use serde::{Deserialize, Serialize};

pub const DYNAMICS_SLOT_VERSION: u16 = 1;

/// Canonical JSON text, with a version belonging to the carrier rather than
/// the physics specification inside it. Ordered JSON maps and compact encoding
/// give equivalent input objects the same authoring identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "SlotWire")]
pub struct DynamicsSlot {
    pub version: u16,
    pub spec: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SlotWire {
    version: u16,
    spec: String,
}

impl DynamicsSlot {
    pub fn from_json(version: u16, json: &str) -> Result<Self, String> {
        if version != DYNAMICS_SLOT_VERSION {
            return Err(format!(
                "dynamics.version: found {version}, reader {DYNAMICS_SLOT_VERSION}"
            ));
        }
        let mut value: serde_json::Value =
            serde_json::from_str(json).map_err(|error| format!("dynamics.spec: {error}"))?;
        if !value.is_object() {
            return Err("dynamics.spec: expected a JSON object".into());
        }
        value.sort_all_objects();
        Ok(Self {
            version,
            spec: serde_json::to_string(&value).map_err(|error| error.to_string())?,
        })
    }

    /// Checks only the carrier. Unknown terms, channels and spec versions are
    /// preserved here and refused by the physics binding host.
    pub fn validate(&self) -> Result<(), String> {
        let canonical = Self::from_json(self.version, &self.spec)?;
        if canonical.spec != self.spec {
            return Err(
                "dynamics.spec: construct the canonical carrier with DynamicsSlot::from_json"
                    .into(),
            );
        }
        Ok(())
    }
}

impl TryFrom<SlotWire> for DynamicsSlot {
    type Error = String;

    fn try_from(value: SlotWire) -> Result<Self, Self::Error> {
        Self::from_json(value.version, &value.spec)
    }
}

/// A variant replaces dynamics by one preset or one complete specification
/// (F194). A tagged choice prevents ambiguous precedence between the two.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DynamicsVariant {
    Preset { id: String },
    Spec { slot: DynamicsSlot },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equivalent_json_has_one_identity_without_interpreting_terms() {
        let a = DynamicsSlot::from_json(1, r#"{ "z": {"future": true}, "a": 1 }"#).unwrap();
        let b = DynamicsSlot::from_json(1, r#"{"a":1,"z":{"future":true}}"#).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.spec, r#"{"a":1,"z":{"future":true}}"#);
        assert_eq!(
            serde_json::from_slice::<DynamicsSlot>(&serde_json::to_vec(&a).unwrap()).unwrap(),
            a
        );
        assert!(
            DynamicsSlot::from_json(2, "{}")
                .unwrap_err()
                .contains("reader 1")
        );
        assert!(DynamicsSlot::from_json(1, "[]").is_err());
    }

    #[test]
    fn a_variant_cannot_name_both_replacements() {
        assert!(
            serde_json::from_str::<DynamicsVariant>(
                r#"{"kind":"preset","id":"spring.rapier","slot":{"version":1,"spec":"{}"}}"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<DynamicsVariant>(
                r#"{"kind":"spec","id":"spring.rapier","slot":{"version":1,"spec":"{}"}}"#
            )
            .is_err()
        );
    }
}
