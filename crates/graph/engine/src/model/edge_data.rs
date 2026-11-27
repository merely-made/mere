// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The assertion records an edge carries: one list per directed pair, every
//! family alike, each record naming its predicate by IRI (declared-lens D20).

use rkyv::{Archive, Deserialize, Serialize};

use crate::model::types::{
    AssertionBasis, AssertionWithdrawal, GraphScope, mint_local_statement_id,
};

use super::relation::Vocabulary;

/// Attribution for stored assertions whose original asserter is unknown.
pub const UNKNOWN_LEGACY_ASSERTER_IRI: &str = "https://mere.computer/ns/agent#unknown-legacy";

#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize, serde::Serialize)]
pub struct SemanticStatement {
    pub statement_id: String,
    pub predicate: String,
    pub label: Option<String>,
    /// Why this assertion exists. `label` remains a compatibility projection.
    pub basis: AssertionBasis,
    pub graph_scope: GraphScope,
    pub provenance_iri: Option<String>,
    /// First record time. `None` is retained for legacy/imported records whose
    /// source did not provide a time.
    pub first_asserted_ms: Option<u64>,
    /// Latest confirmation time; kept under its legacy field name for stored
    /// compatibility.
    pub asserted_at_ms: Option<u64>,
    /// Withdrawal is a retained tombstone, not a hard delete.
    pub withdrawn: Option<AssertionWithdrawal>,
}

/// All-family assertion record. The legacy semantic name remains source compatible.
pub type RelationAssertion = SemanticStatement;
pub type RelationAssertionSpec = SemanticStatementSpec;
pub type AssertionData = SemanticData;

impl SemanticStatement {
    /// Whether the record's predicate counts as semantic in `vocabulary`.
    pub fn is_semantic_in(&self, vocabulary: Vocabulary) -> bool {
        vocabulary.is_semantic(&self.predicate)
    }

    /// Whether it does in the core profile.
    pub fn is_semantic(&self) -> bool {
        self.is_semantic_in(Vocabulary::core())
    }

    pub fn is_held(&self) -> bool {
        self.withdrawn.is_none()
    }

    /// Whether both records are one claim under a carried handle: predicate,
    /// scope, asserter and basis reason. Labels, times and method version may differ.
    pub fn same_claim_as(&self, other: &Self) -> bool {
        self.predicate == other.predicate
            && self.graph_scope == other.graph_scope
            && self.provenance_iri == other.provenance_iri
            && self.basis.same_reason_as(&other.basis)
    }

    pub fn normalize_basis_compat(&mut self) {
        if self.basis.note.is_none() {
            self.basis.note = self.label.clone();
        }
        self.label = self.basis.note.clone();
    }

    /// Normalize missing attribution on a legacy record without inventing authorship.
    pub fn normalize_legacy_asserter(&mut self) -> bool {
        if self.provenance_iri.is_some() {
            return false;
        }
        self.provenance_iri = Some(UNKNOWN_LEGACY_ASSERTER_IRI.to_string());
        true
    }

    pub fn new(
        predicate: String,
        label: Option<String>,
        graph_scope: GraphScope,
        provenance_iri: Option<String>,
        asserted_at_ms: Option<u64>,
    ) -> Self {
        Self {
            statement_id: mint_local_statement_id(),
            predicate,
            label: label.clone(),
            basis: AssertionBasis {
                note: label.clone(),
                ..Default::default()
            },
            graph_scope,
            provenance_iri,
            first_asserted_ms: asserted_at_ms,
            asserted_at_ms,
            withdrawn: None,
        }
    }
}

// Keep the missing-vs-explicit-null distinction for first_asserted_ms. Old
// serialized statements omitted the field, so only that case falls back to
// the legacy asserted_at_ms value. An explicitly unknown first time stays
// unknown after every reopen.
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
        Ok(Self(Some(Option::<AssertionBasis>::deserialize(
            deserializer,
        )?)))
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

impl<'de> serde::Deserialize<'de> for SemanticStatement {
    /// A record written before L7 also names `recognized_sub_kind`; the
    /// predicate IRI already says it, so the name is not read (D17).
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Fields {
            statement_id: String,
            predicate: String,
            #[serde(default)]
            label: Option<String>,
            #[serde(default)]
            basis: PresentBasis,
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
        }

        let fields = <Fields as serde::Deserialize>::deserialize(deserializer)?;
        let basis = fields.basis.0.flatten().unwrap_or_else(|| AssertionBasis {
            note: fields.label.clone(),
            ..Default::default()
        });
        let label = basis.note.clone();
        Ok(Self {
            statement_id: fields.statement_id,
            predicate: fields.predicate,
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

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
    Default,
)]
pub struct SemanticData {
    /// Compatibility aggregate over the statement labels.
    pub label: Option<String>,
    /// The one predicate the held semantic statements share, if they share
    /// one: a compatibility view over the statements, not the durable truth.
    pub predicate: Option<String>,
    /// The one durable assertion list carried between a directed pair.
    pub statements: Vec<SemanticStatement>,
    /// The table the records are classified by; never stored.
    #[serde(skip)]
    #[rkyv(with = rkyv::with::Skip)]
    pub vocabulary: Vocabulary,
}

/// A statement to assert: everything a [`SemanticStatement`] carries except
/// its id, which the kernel mints (device-safe) at assert time. The
/// statement-aware write API's input — the petgraph-RDF plan's Phase 1
/// requirement that interactive writes can carry per-statement metadata, not
/// just the legacy edge-wide predicate stamp.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct SemanticStatementSpec {
    pub predicate: String,
    pub label: Option<String>,
    /// Optional reason metadata; on admission `basis.note` is authoritative.
    pub basis: AssertionBasis,
    pub graph_scope: GraphScope,
    pub provenance_iri: Option<String>,
    pub asserted_at_ms: Option<u64>,
}

impl SemanticStatementSpec {
    pub fn normalized(mut self) -> Self {
        if self.basis.note.is_none() {
            self.basis.note = self.label.clone();
        }
        self.label = self.basis.note.clone();
        self
    }
}

/// What [`SemanticData::assert_statement`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementAssert {
    /// The asserted fact's handle — freshly minted, or the existing
    /// statement's id when content-dedup matched.
    pub statement_id: String,
    /// Whether anything changed (a new statement, or metadata updated on the
    /// deduped one). `false` = the exact statement already existed.
    pub changed: bool,
}

#[cfg(test)]
#[path = "edge_data/canonical_family_controls.rs"]
mod canonical_family_controls;

#[path = "edge_data/semantic.rs"]
mod semantic;
