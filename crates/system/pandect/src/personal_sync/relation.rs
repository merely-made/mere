// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use mere::model::Vocabulary;
use mere::model::relation::stored::{predicate_uuid, read_predicate, stored_iri};
use serde::{Deserialize, Serialize};

/// The relation an [`PersonalGraphEvent::AssertRelation`] names: its
/// predicate and, for a semantic one, its label. A new event names the
/// predicate by UUIDv5, and by IRI too when the core pack lacks the term
/// (declared-lens D28). An event signed before L7 names the IRI alone, or a
/// family and sub-kind in `assertion`, which reads as the IRI it meant (D17).
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
        let iri = stored_iri(&self.predicate);
        let entries = 1 + usize::from(iri.is_some()) + usize::from(self.label.is_some());
        let mut map = serializer.serialize_map(Some(entries))?;
        map.serialize_entry("predicate_uuid", &predicate_uuid(&self.predicate))?;
        if let Some(iri) = iri {
            map.serialize_entry("predicate", iri)?;
        }
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
            predicate_uuid: Option<uuid::Uuid>,
            #[serde(default)]
            predicate: Option<String>,
            #[serde(default)]
            label: Option<String>,
            #[serde(default)]
            assertion: Option<mere::model::relation::legacy::LegacyAssertion>,
        }
        let fields = Fields::deserialize(deserializer)?;
        match (fields.predicate_uuid, fields.predicate, fields.assertion) {
            (None, None, Some(old)) => Ok(Self {
                predicate: old.predicate,
                label: old.label,
            }),
            (uuid, iri, None) if uuid.is_some() || iri.is_some() => Ok(Self {
                predicate: read_predicate(uuid, iri).map_err(serde::de::Error::custom)?,
                label: fields.label,
            }),
            _ => Err(serde::de::Error::custom(
                "a relation names one predicate or one legacy assertion",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use mere::profile::core::{self, ids};
    use serde_json::json;

    use super::*;

    #[test]
    fn a_claim_names_its_predicate_by_uuid_and_by_iri_off_core() {
        let cites = RelationClaim::new(core::iri(ids::REL_CITES), Some("why".into()));
        let wire = serde_json::to_value(&cites).unwrap();
        assert_eq!(
            wire,
            json!({ "predicate_uuid": predicate_uuid(&cites.predicate), "label": "why" })
        );
        assert_eq!(
            serde_json::from_value::<RelationClaim>(wire).unwrap(),
            cites
        );
        let open = RelationClaim::new("urn:mere:test:open", None);
        let wire = serde_json::to_value(&open).unwrap();
        assert_eq!(wire["predicate"], "urn:mere:test:open");
        assert_eq!(serde_json::from_value::<RelationClaim>(wire).unwrap(), open);
    }

    #[test]
    fn a_claim_that_does_not_resolve_is_refused_and_an_old_one_reads() {
        let off_core = json!({ "predicate_uuid": predicate_uuid("urn:mere:test:open") });
        assert!(serde_json::from_value::<RelationClaim>(off_core).is_err());
        let wrong = json!({
            "predicate_uuid": predicate_uuid(core::iri(ids::REL_QUOTES)),
            "predicate": core::iri(ids::REL_CITES),
        });
        assert!(serde_json::from_value::<RelationClaim>(wrong).is_err());
        let old = json!({ "predicate": core::iri(ids::REL_CITES) });
        assert_eq!(
            serde_json::from_value::<RelationClaim>(old)
                .unwrap()
                .predicate,
            core::iri(ids::REL_CITES)
        );
    }
}
