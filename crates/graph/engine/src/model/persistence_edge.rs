// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! An edge as a snapshot and a journal store it: the pair and its assertion
//! records, each naming its predicate by UUIDv5, and by IRI when the core
//! pack lacks it (declared-lens D17, D28). The sidecars
//! older engines wrote, by family and sub-kind name, are read and never
//! written. A surface's own edge record wraps this one with what only the
//! surface holds (D22).

use rkyv::{Archive, Deserialize, Serialize};

use crate::model::types::{AssertionBasis, AssertionWithdrawal, GraphScope};

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[rkyv(derive(Debug, PartialEq))]
pub struct PersistedSemanticStatement {
    pub statement_id: String,
    /// The predicate IRI. Stored as its UUIDv5, and as text too only when the
    /// core pack lacks the term (D28).
    pub predicate: String,
    pub label: Option<String>,
    pub graph_scope: GraphScope,
    pub provenance_iri: Option<String>,
    pub asserted_at_ms: Option<u64>,
    /// Appended after the legacy persisted fields. `asserted_at_ms` remains
    /// the last-confirmation time for stored compatibility.
    pub first_asserted_ms: Option<u64>,
    pub withdrawn: Option<AssertionWithdrawal>,
    /// Appended to preserve the existing positional field sequence.
    pub basis: AssertionBasis,
}

struct PresentOptionU64(Option<Option<u64>>);
struct PresentBasis(Option<Option<AssertionBasis>>);

impl Default for PresentOptionU64 {
    fn default() -> Self {
        Self(None)
    }
}

impl Default for PresentBasis {
    fn default() -> Self {
        Self(None)
    }
}

impl<'de> serde::Deserialize<'de> for PresentBasis {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = if deserializer.is_human_readable() {
            Option::<AssertionBasis>::deserialize(deserializer)?
        } else {
            Some(AssertionBasis::deserialize(deserializer)?)
        };
        Ok(Self(Some(value)))
    }
}

impl<'de> serde::Deserialize<'de> for PresentOptionU64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <Option<u64> as serde::Deserialize>::deserialize(deserializer)?;
        Ok(Self(Some(value)))
    }
}

impl<'de> serde::Deserialize<'de> for PersistedSemanticStatement {
    /// A record names its predicate by UUID, with the IRI when the core pack
    /// lacks the term (D28). One written before L7 names the IRI alone and
    /// also `recognized_sub_kind`, which the IRI already says, so the name is
    /// not read (D17).
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Fields {
            statement_id: String,
            #[serde(default)]
            predicate_uuid: Option<uuid::Uuid>,
            #[serde(default)]
            predicate: Option<String>,
            #[serde(default)]
            label: Option<String>,
            #[serde(default)]
            graph_scope: GraphScope,
            #[serde(default)]
            provenance_iri: Option<String>,
            #[serde(default)]
            asserted_at_ms: Option<u64>,
            #[serde(default)]
            first_asserted_ms: PresentOptionU64,
            #[serde(default)]
            withdrawn: Option<AssertionWithdrawal>,
            #[serde(default)]
            basis: PresentBasis,
        }

        let fields = <Fields as serde::Deserialize>::deserialize(deserializer)?;
        let predicate =
            crate::model::relation::stored::read_predicate(fields.predicate_uuid, fields.predicate)
                .map_err(serde::de::Error::custom)?;
        let basis = fields.basis.0.flatten().unwrap_or_else(|| AssertionBasis {
            note: fields.label.clone(),
            ..Default::default()
        });
        let label = basis.note.clone();
        Ok(Self {
            statement_id: fields.statement_id,
            predicate,
            label,
            basis,
            graph_scope: fields.graph_scope,
            provenance_iri: fields.provenance_iri,
            first_asserted_ms: fields.first_asserted_ms.0.unwrap_or(fields.asserted_at_ms),
            asserted_at_ms: fields.asserted_at_ms,
            withdrawn: fields.withdrawn,
        })
    }
}

/// The semantic sidecar engines wrote before assertion records: sub-kind
/// names, a label, an open predicate and an older statement bucket. Read only.
#[derive(
    Archive,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq))]
pub struct PersistedSemanticEdgeData {
    #[serde(default)]
    pub sub_kinds: Vec<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub agent_decay_progress: Option<f32>,
    #[serde(default)]
    pub predicate: Option<String>,
    #[serde(default)]
    pub statements: Vec<PersistedSemanticStatement>,
}

/// A family's sidecar engines wrote before assertion records: sub-kind names.
/// Read only.
#[derive(
    Archive,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq))]
pub struct PersistedSubKinds {
    #[serde(default)]
    pub sub_kinds: Vec<String>,
}

/// Persisted edge. Its JSON names the pair and the records only; a
/// positional format keeps every field in place (`serialization.rs`).
#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, serde::Deserialize)]
pub struct PersistedEdge {
    pub from_node_id: String,
    pub to_node_id: String,
    /// Legacy sidecars, by family: read when `assertions` is absent, never written.
    #[serde(default)]
    pub semantic: Option<PersistedSemanticEdgeData>,
    #[serde(default)]
    pub containment: Option<PersistedSubKinds>,
    #[serde(default)]
    pub imported: Option<PersistedSubKinds>,
    #[serde(default)]
    pub provenance: Option<PersistedSubKinds>,
    /// Canonical all-family records. None selects the legacy sidecar decoder;
    /// Some(empty) is an explicitly empty list, never a request to infer facts.
    #[serde(default)]
    pub assertions: Option<Vec<PersistedSemanticStatement>>,
}

impl PersistedEdge {
    /// An edge holding `assertions`, in the form every new write takes.
    pub fn of(
        from_node_id: String,
        to_node_id: String,
        assertions: Vec<PersistedSemanticStatement>,
    ) -> Self {
        Self {
            from_node_id,
            to_node_id,
            semantic: None,
            containment: None,
            imported: None,
            provenance: None,
            assertions: Some(assertions),
        }
    }
    pub fn retained_assertions(&self) -> &[PersistedSemanticStatement] {
        self.assertions.as_deref().unwrap_or_else(|| {
            self.semantic
                .as_ref()
                .map(|data| data.statements.as_slice())
                .unwrap_or(&[])
        })
    }
    pub fn retained_assertions_mut(&mut self) -> &mut [PersistedSemanticStatement] {
        match self.assertions.as_mut() {
            Some(records) => records.as_mut_slice(),
            None => self
                .semantic
                .as_mut()
                .map(|data| data.statements.as_mut_slice())
                .unwrap_or(&mut []),
        }
    }
    pub fn has_legacy_assertion_data(&self) -> bool {
        self.semantic.as_ref().is_some_and(|data| {
            !data.statements.is_empty()
                || !data.sub_kinds.is_empty()
                || data.predicate.is_some()
                || data.label.is_some()
        }) || [&self.containment, &self.imported, &self.provenance]
            .into_iter()
            .any(|data| data.as_ref().is_some_and(|d| !d.sub_kinds.is_empty()))
    }
    /// Each legacy sidecar's family name and sub-kind names, in decode order.
    pub fn legacy_sub_kinds(&self) -> [(&'static str, &[String]); 3] {
        fn names(data: &Option<PersistedSubKinds>) -> &[String] {
            data.as_ref().map_or(&[], |d| d.sub_kinds.as_slice())
        }
        [
            ("Containment", names(&self.containment)),
            ("Imported", names(&self.imported)),
            ("Provenance", names(&self.provenance)),
        ]
    }
}

#[cfg(test)]
#[path = "persistence_edge/statement_serialization_tests.rs"]
mod statement_serialization_tests;

#[path = "persistence_edge/serialization.rs"]
mod serialization;
