// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Per-family edge runtime data structs and semantic IRI mapping.
//! Extracted from `edge_taxonomy.rs` per the 600-LOC ceiling.

use std::collections::BTreeSet;

use rkyv::{Archive, Deserialize, Serialize};

use crate::model::types::{
    AssertionBasis, AssertionWithdrawal, GraphScope, mint_local_statement_id,
};

use super::edge_taxonomy::{
    ArrangementSubKind, ContainmentSubKind, ImportedSubKind, NavigationTrigger, ProvenanceSubKind,
    RelationDurability, SemanticSubKind,
};

/// A temporal traversal event recorded on an edge.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Traversal {
    pub timestamp_ms: u64,
    pub trigger: NavigationTrigger,
}

impl Traversal {
    pub fn now(trigger: NavigationTrigger) -> Self {
        let timestamp_ms = crate::model::clock::unix_epoch_millis();
        Self {
            timestamp_ms,
            trigger,
        }
    }
}

/// Durable traversal aggregates retained even when rolling-window records are evicted.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct EdgeMetrics {
    pub total_navigations: u64,
    pub forward_navigations: u64,
    pub backward_navigations: u64,
    pub last_navigated_at: Option<u64>,
}

impl EdgeMetrics {
    fn new() -> Self {
        Self {
            total_navigations: 0,
            forward_navigations: 0,
            backward_navigations: 0,
            last_navigated_at: None,
        }
    }

    fn record(&mut self, traversal: Traversal) {
        self.total_navigations = self.total_navigations.saturating_add(1);
        if traversal.trigger.contributes_to_forward_count() {
            self.forward_navigations = self.forward_navigations.saturating_add(1);
        } else {
            self.backward_navigations = self.backward_navigations.saturating_add(1);
        }
        self.last_navigated_at = Some(traversal.timestamp_ms);
    }
}

impl Default for EdgeMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Attribution for stored assertions whose original asserter is unknown.
pub const UNKNOWN_LEGACY_ASSERTER_IRI: &str = "https://mere.computer/ns/agent#unknown-legacy";

#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize, serde::Serialize)]
pub struct SemanticStatement {
    pub statement_id: String,
    pub predicate: String,
    pub recognized_sub_kind: Option<SemanticSubKind>,
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
    /// Classify the assertion by its canonical predicate. The old semantic
    /// discriminant remains a compatibility input for existing records.
    pub fn relation_kind(&self) -> super::RelationKind {
        super::RelationKind::from_predicate_iri(&self.predicate)
            .or_else(|| self.recognized_sub_kind.map(super::RelationKind::Semantic))
            .unwrap_or(super::RelationKind::OpenPredicate)
    }

    pub fn is_semantic(&self) -> bool {
        self.relation_kind().family() == super::EdgeFamily::Semantic
    }

    pub fn is_held(&self) -> bool {
        self.withdrawn.is_none()
    }

    /// Whether both records are one claim under a carried handle: predicate, sub-kind,
    /// scope, asserter and basis reason. Labels, times and method version may differ.
    pub fn same_claim_as(&self, other: &Self) -> bool {
        self.predicate == other.predicate
            && self.recognized_sub_kind == other.recognized_sub_kind
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
        recognized_sub_kind: Option<SemanticSubKind>,
        label: Option<String>,
        graph_scope: GraphScope,
        provenance_iri: Option<String>,
        asserted_at_ms: Option<u64>,
    ) -> Self {
        Self {
            statement_id: mint_local_statement_id(),
            predicate,
            recognized_sub_kind,
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
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Fields {
            statement_id: String,
            predicate: String,
            #[serde(default)]
            recognized_sub_kind: Option<SemanticSubKind>,
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
            recognized_sub_kind: fields.recognized_sub_kind,
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
    /// Compatibility aggregate: kept during the bucket migration so existing
    /// readers can keep asking the pair-local edge what semantic kinds it
    /// carries. The statement list below is the durable truth.
    pub sub_kinds: BTreeSet<SemanticSubKind>,
    /// Compatibility aggregate over the statement labels.
    pub label: Option<String>,
    /// Open predicate IRI (the statements-over-schema substrate, linked-data
    /// plan Phase 0). `None` when the edge's meaning is fully carried by
    /// `sub_kinds`; a canonical IRI ([`predicate_iri`]) for a recognized
    /// predicate, or a raw IRI for an unrecognized one. `sub_kinds` stays
    /// authoritative for behaviour; `predicate` carries identity + round-trip.
    /// During the statement-bucket migration this is a compatibility view over
    /// the statement predicates, not the durable truth.
    pub predicate: Option<String>,
    /// The one durable assertion list carried between a directed pair.
    pub statements: Vec<SemanticStatement>,
}

/// A statement to assert: everything a [`SemanticStatement`] carries except
/// its id, which the kernel mints (device-safe) at assert time. The
/// statement-aware write API's input — the petgraph-RDF plan's Phase 1
/// requirement that interactive writes can carry per-statement metadata, not
/// just the legacy edge-wide predicate stamp.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct SemanticStatementSpec {
    pub predicate: String,
    pub recognized_sub_kind: Option<SemanticSubKind>,
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

/// Base IRI for Mere's canonical relation vocabulary. Recognized
/// [`SemanticSubKind`]s map to `{REL_VOCAB}{slug}` (the "small Mere vocabulary
/// of canonical IRIs"); the JSON-LD export layer maps these onto standard
/// vocabularies (CiTO, schema.org, …) through its `@context`. Provisional base.
pub const REL_VOCAB: &str = "https://mere.computer/ns/rel#";

/// The canonical relation IRI for a recognized semantic sub-kind. Inverse of
/// [`sub_kind_from_iri`].
pub fn predicate_iri(sub_kind: SemanticSubKind) -> &'static str {
    match sub_kind {
        SemanticSubKind::Hyperlink => "https://mere.computer/ns/rel#hyperlink",
        SemanticSubKind::UserGrouped => "https://mere.computer/ns/rel#user-grouped",
        SemanticSubKind::AgentDerived => "https://mere.computer/ns/rel#agent-derived",
        SemanticSubKind::Cites => "https://mere.computer/ns/rel#cites",
        SemanticSubKind::Quotes => "https://mere.computer/ns/rel#quotes",
        SemanticSubKind::Summarizes => "https://mere.computer/ns/rel#summarizes",
        SemanticSubKind::Elaborates => "https://mere.computer/ns/rel#elaborates",
        SemanticSubKind::ExampleOf => "https://mere.computer/ns/rel#example-of",
        SemanticSubKind::Supports => "https://mere.computer/ns/rel#supports",
        SemanticSubKind::Contradicts => "https://mere.computer/ns/rel#contradicts",
        SemanticSubKind::Questions => "https://mere.computer/ns/rel#questions",
        SemanticSubKind::SameEntityAs => "https://mere.computer/ns/rel#same-entity-as",
        SemanticSubKind::DuplicateOf => "https://mere.computer/ns/rel#duplicate-of",
        SemanticSubKind::CanonicalMirrorOf => "https://mere.computer/ns/rel#canonical-mirror-of",
        SemanticSubKind::DependsOn => "https://mere.computer/ns/rel#depends-on",
        SemanticSubKind::Blocks => "https://mere.computer/ns/rel#blocks",
        SemanticSubKind::NextStep => "https://mere.computer/ns/rel#next-step",
    }
}

/// The recognized semantic sub-kind for a canonical relation IRI, if any.
/// Inverse of [`predicate_iri`]; `None` for an unrecognized (raw) IRI, which is
/// stored as an open predicate without a sub-kind.
pub fn sub_kind_from_iri(iri: &str) -> Option<SemanticSubKind> {
    use strum::IntoEnumIterator;
    SemanticSubKind::iter().find(|&sub_kind| predicate_iri(sub_kind) == iri)
}

/// Every recognized semantic sub-kind, in declaration order. The kernel owns the
/// enumeration of its recognized relation vocabulary; downstream projection code
/// (e.g. linked-data's standard-vocabulary alignment) maps over it without
/// re-deriving the list or pulling in `strum`.
pub fn all_semantic_sub_kinds() -> impl Iterator<Item = SemanticSubKind> {
    use strum::IntoEnumIterator;
    SemanticSubKind::iter()
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
pub struct TraversalData {
    pub traversals: Vec<Traversal>,
    pub metrics: EdgeMetrics,
}

impl TraversalData {
    pub fn push(&mut self, traversal: Traversal) {
        self.metrics.record(traversal);
        self.traversals.push(traversal);
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
pub struct ArrangementData {
    pub sub_kinds: BTreeSet<ArrangementSubKind>,
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
pub struct ContainmentData {
    pub sub_kinds: BTreeSet<ContainmentSubKind>,
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
pub struct ImportedData {
    pub sub_kinds: BTreeSet<ImportedSubKind>,
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
pub struct ProvenanceData {
    pub sub_kinds: BTreeSet<ProvenanceSubKind>,
}

impl ContainmentData {
    pub fn insert(&mut self, sub_kind: ContainmentSubKind) -> bool {
        self.sub_kinds.insert(sub_kind)
    }

    pub fn remove(&mut self, sub_kind: ContainmentSubKind) -> bool {
        self.sub_kinds.remove(&sub_kind)
    }

    pub fn contains(&self, sub_kind: ContainmentSubKind) -> bool {
        self.sub_kinds.contains(&sub_kind)
    }

    pub fn is_empty(&self) -> bool {
        self.sub_kinds.is_empty()
    }
}

impl ArrangementData {
    pub fn insert(&mut self, sub_kind: ArrangementSubKind) -> bool {
        self.sub_kinds.insert(sub_kind)
    }

    pub fn remove(&mut self, sub_kind: ArrangementSubKind) -> bool {
        self.sub_kinds.remove(&sub_kind)
    }

    pub fn contains(&self, sub_kind: ArrangementSubKind) -> bool {
        self.sub_kinds.contains(&sub_kind)
    }

    pub fn is_empty(&self) -> bool {
        self.sub_kinds.is_empty()
    }

    pub fn has_durable_relation(&self) -> bool {
        self.sub_kinds
            .iter()
            .copied()
            .any(|sub_kind| sub_kind.durability() == RelationDurability::Durable)
    }

    pub fn has_session_relation(&self) -> bool {
        self.sub_kinds
            .iter()
            .copied()
            .any(|sub_kind| sub_kind.durability() == RelationDurability::Session)
    }
}

#[cfg(test)]
#[path = "edge_data/canonical_family_controls.rs"]
mod canonical_family_controls;

#[path = "edge_data/semantic.rs"]
mod semantic;
