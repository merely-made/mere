// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use mere::model::Vocabulary;
use serde::{Deserialize, Serialize};

/// The relation an [`PersonalGraphEvent::AssertRelation`] names: its
/// predicate IRI and, for a semantic one, its label. An event signed before
/// L7 names it by family and sub-kind in `assertion`, which reads as the IRI
/// it meant; new events write the IRI only (declared-lens D17).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationClaim {
    pub predicate: String,
    pub label: Option<String>,
}

impl RelationClaim {
    pub fn new(predicate: impl Into<String>, label: Option<String>) -> Self {
        Self {
            predicate: predicate.into(),
            label,
        }
    }

    /// Whether the claim's predicate counts as semantic in the core profile.
    pub fn is_semantic(&self) -> bool {
        Vocabulary::core().is_semantic(&self.predicate)
    }
}

impl Serialize for RelationClaim {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(1 + usize::from(self.label.is_some())))?;
        map.serialize_entry("predicate", &self.predicate)?;
        if let Some(label) = &self.label {
            map.serialize_entry("label", label)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for RelationClaim {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Fields {
            #[serde(default)]
            predicate: Option<String>,
            #[serde(default)]
            label: Option<String>,
            #[serde(default)]
            assertion: Option<mere::model::relation::legacy::LegacyAssertion>,
        }
        let fields = Fields::deserialize(deserializer)?;
        match (fields.predicate, fields.assertion) {
            (Some(predicate), None) => Ok(Self {
                predicate,
                label: fields.label,
            }),
            (None, Some(old)) => Ok(Self {
                predicate: old.predicate,
                label: old.label,
            }),
            _ => Err(serde::de::Error::custom(
                "a relation names one predicate or one legacy assertion",
            )),
        }
    }
}

/// One signed authoring turn.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonalGraphRecord {
    pub events: Vec<PersonalGraphEvent>,
    #[serde(default)]
    pub parents: Vec<[u8; 32]>,
    #[serde(default)]
    pub writer_attestation: Option<DerivedKeyAttestation>,
}
