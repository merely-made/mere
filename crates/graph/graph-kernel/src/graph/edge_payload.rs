// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One assertion list per directed bucket, plus the traversal event rollup.
//!
//! The four former structural sidecars are compatibility projections. Their
//! predicates now carry the same exact identity and lifecycle as semantic facts.

use super::edge_data::{
    ArrangementData, AssertionData, ContainmentData, EdgeMetrics, ImportedData, ProvenanceData,
    SemanticData, SemanticStatement, SemanticStatementSpec, StatementAssert, Traversal,
    TraversalData, UNKNOWN_LEGACY_ASSERTER_IRI, predicate_iri,
};
use super::edge_taxonomy::{
    ArrangementSubKind, EdgeAssertion, EdgeFamily, RelationKind, RelationSelector, SemanticSubKind,
};
use crate::types::{AssertionWithdrawal, GraphScope};
use rkyv::{Archive, Deserialize, Serialize};
use std::collections::BTreeSet;

/// Borrowed semantic compatibility projection, never another assertion store.
pub struct SemanticDataView<'a> {
    pub sub_kinds: &'a BTreeSet<SemanticSubKind>,
    pub label: &'a Option<String>,
    pub predicate: &'a Option<String>,
    pub statements: &'a [SemanticStatement],
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
)]
pub struct EdgePayload {
    /// All retained assertions, including withdrawal tombstones. The legacy
    /// `SemanticData` name is a source compatibility alias for the record store.
    pub assertions: AssertionData,
    pub traversal: Option<TraversalData>,
}

pub fn selector_matches_kind(selector: RelationSelector, kind: RelationKind) -> bool {
    match selector {
        RelationSelector::Family(family) => family == kind.family(),
        RelationSelector::Semantic(value) => kind == RelationKind::Semantic(value),
        RelationSelector::Containment(value) => kind == RelationKind::Containment(value),
        RelationSelector::Arrangement(value) => kind == RelationKind::Arrangement(value),
        RelationSelector::Imported(value) => kind == RelationKind::Imported(value),
        RelationSelector::Provenance(value) => kind == RelationKind::Provenance(value),
    }
}

impl<'a> SemanticDataView<'a> {
    pub fn statements(&self) -> &'a [SemanticStatement] {
        self.statements
    }
}

impl EdgePayload {
    pub fn new() -> Self {
        Self {
            assertions: SemanticData::default(),
            traversal: None,
        }
    }

    pub fn assertions(&self) -> &[SemanticStatement] {
        &self.assertions.statements
    }

    pub fn held_assertions(&self) -> impl Iterator<Item = &SemanticStatement> {
        self.assertions()
            .iter()
            .filter(|statement| statement.is_held())
    }

    pub fn semantic_statements(&self) -> &[SemanticStatement] {
        let count = self
            .assertions()
            .partition_point(SemanticStatement::is_semantic);
        &self.assertions()[..count]
    }

    pub fn held_semantic_statements(&self) -> impl Iterator<Item = &SemanticStatement> {
        self.semantic_statements()
            .iter()
            .filter(|statement| statement.is_held())
    }

    pub fn semantic_data(&self) -> Option<SemanticDataView<'_>> {
        (!self.semantic_statements().is_empty()).then(|| SemanticDataView {
            sub_kinds: &self.assertions.sub_kinds,
            label: &self.assertions.label,
            predicate: &self.assertions.predicate,
            statements: self.semantic_statements(),
        })
    }

    pub fn has_relation(&self, selector: RelationSelector) -> bool {
        if selector == RelationSelector::Family(EdgeFamily::Traversal) {
            return self.traversal.is_some();
        }
        self.held_assertions()
            .any(|statement| selector_matches_kind(selector, statement.relation_kind()))
    }

    /// Legacy membership ensure. A caller who has an independent reason uses
    /// the statement-aware Graph writer and its exact returned assertion ID.
    pub fn assert_relation(&mut self, assertion: EdgeAssertion) -> bool {
        self.assert_relation_with_context(assertion, None, None)
    }

    pub(crate) fn assert_relation_with_context(
        &mut self,
        assertion: EdgeAssertion,
        asserter: Option<String>,
        at_ms: Option<u64>,
    ) -> bool {
        let (kind, label) = match assertion {
            EdgeAssertion::Semantic {
                sub_kind, label, ..
            } => (RelationKind::Semantic(sub_kind), label),
            EdgeAssertion::Containment { sub_kind } => (RelationKind::Containment(sub_kind), None),
            EdgeAssertion::Arrangement { sub_kind } => (RelationKind::Arrangement(sub_kind), None),
            EdgeAssertion::Imported { sub_kind } => (RelationKind::Imported(sub_kind), None),
            EdgeAssertion::Provenance { sub_kind } => (RelationKind::Provenance(sub_kind), None),
        };
        if kind.family() != EdgeFamily::Semantic
            && self.held_assertions().any(|s| s.relation_kind() == kind)
        {
            return false;
        }
        self.assertions
            .assert_statement(SemanticStatementSpec {
                predicate: kind
                    .predicate_iri()
                    .expect("asserted kind has a predicate")
                    .into(),
                recognized_sub_kind: match kind {
                    RelationKind::Semantic(k) => Some(k),
                    _ => None,
                },
                label,
                provenance_iri: asserter.or_else(|| Some(UNKNOWN_LEGACY_ASSERTER_IRI.into())),
                asserted_at_ms: at_ms,
                ..Default::default()
            })
            .changed
    }

    /// Physical removal remains reserved for regeneration, migration and
    /// compaction. Authored withdrawals use Graph's exact-ID lifecycle writer.
    pub(crate) fn retract_relation(&mut self, selector: RelationSelector) -> bool {
        if selector == RelationSelector::Family(EdgeFamily::Traversal) {
            return self.traversal.take().is_some();
        }
        let before = self.assertions().len();
        self.assertions
            .statements
            .retain(|s| !s.is_held() || !selector_matches_kind(selector, s.relation_kind()));
        if self.assertions().len() == before {
            return false;
        }
        self.assertions.rebuild_compat();
        true
    }

    pub fn is_empty(&self) -> bool {
        self.assertions().is_empty() && self.traversal.is_none()
    }
    pub fn label(&self) -> Option<&str> {
        self.assertions.label.as_deref()
    }

    pub(crate) fn assert_semantic_predicate(&mut self, predicate: String) -> bool {
        self.assert_semantic_predicate_in_scope(predicate, GraphScope::Default)
    }
    pub(crate) fn assert_semantic_relation_in_scope(
        &mut self,
        sub_kind: SemanticSubKind,
        label: Option<String>,
        scope: GraphScope,
    ) -> bool {
        self.assertions.insert_statement(
            Some(sub_kind),
            predicate_iri(sub_kind).into(),
            label,
            scope,
            None,
            None,
        )
    }
    pub(crate) fn assert_semantic_predicate_in_scope(
        &mut self,
        predicate: String,
        scope: GraphScope,
    ) -> bool {
        self.assertions
            .insert_statement(None, predicate, None, scope, None, None)
    }
    pub(crate) fn assert_semantic_statement(
        &mut self,
        spec: SemanticStatementSpec,
    ) -> StatementAssert {
        self.assertions.assert_statement(spec)
    }
    pub(crate) fn withdraw_semantic_statement(
        &mut self,
        id: &str,
        withdrawal: AssertionWithdrawal,
    ) -> bool {
        self.assertions.withdraw_statement(id, withdrawal)
    }
    pub(crate) fn compact_semantic_statement(&mut self, id: &str) -> bool {
        self.assertions.compact_statement(id)
    }
    pub(crate) fn upsert_persisted_semantic_statement(
        &mut self,
        statement: SemanticStatement,
    ) -> bool {
        self.assertions.upsert_persisted_statement(statement)
    }
    pub(crate) fn push_persisted_semantic_statement(
        &mut self,
        statement: SemanticStatement,
    ) -> bool {
        self.assertions.push_persisted_statement(statement)
    }
    pub fn set_semantic_predicate(&mut self, predicate: Option<String>) {
        self.assertions.set_statement_predicate(predicate);
    }

    pub fn traversal_data(&self) -> Option<&TraversalData> {
        self.traversal.as_ref()
    }
    pub fn arrangement_data(&self) -> Option<ArrangementData> {
        let sub_kinds = self
            .held_assertions()
            .filter_map(|s| match s.relation_kind() {
                RelationKind::Arrangement(k) => Some(k),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        (!sub_kinds.is_empty()).then_some(ArrangementData { sub_kinds })
    }
    pub fn containment_data(&self) -> Option<ContainmentData> {
        let sub_kinds = self
            .held_assertions()
            .filter_map(|s| match s.relation_kind() {
                RelationKind::Containment(k) => Some(k),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        (!sub_kinds.is_empty()).then_some(ContainmentData { sub_kinds })
    }
    pub fn imported_data(&self) -> Option<ImportedData> {
        let sub_kinds = self
            .held_assertions()
            .filter_map(|s| match s.relation_kind() {
                RelationKind::Imported(k) => Some(k),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        (!sub_kinds.is_empty()).then_some(ImportedData { sub_kinds })
    }
    pub fn provenance_data(&self) -> Option<ProvenanceData> {
        let sub_kinds = self
            .held_assertions()
            .filter_map(|s| match s.relation_kind() {
                RelationKind::Provenance(k) => Some(k),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        (!sub_kinds.is_empty()).then_some(ProvenanceData { sub_kinds })
    }
    pub fn has_arrangement_sub_kind(&self, kind: ArrangementSubKind) -> bool {
        self.has_relation(RelationSelector::Arrangement(kind))
    }
    pub fn has_durable_arrangement_relation(&self) -> bool {
        self.arrangement_data()
            .is_some_and(|data| data.has_durable_relation())
    }
    pub fn has_session_arrangement_relation(&self) -> bool {
        self.arrangement_data()
            .is_some_and(|data| data.has_session_relation())
    }
    pub fn traversals(&self) -> &[Traversal] {
        self.traversal
            .as_ref()
            .map(|data| data.traversals.as_slice())
            .unwrap_or(&[])
    }
    pub fn metrics(&self) -> EdgeMetrics {
        self.traversal
            .as_ref()
            .map(|data| data.metrics)
            .unwrap_or_default()
    }
    pub fn push_traversal(&mut self, traversal: Traversal) {
        self.traversal
            .get_or_insert_with(TraversalData::default)
            .push(traversal);
    }
    pub fn families(&self) -> BTreeSet<EdgeFamily> {
        let mut families = self
            .held_assertions()
            .map(|s| s.relation_kind().family())
            .collect::<BTreeSet<_>>();
        if self.traversal.is_some() {
            families.insert(EdgeFamily::Traversal);
        }
        families
    }
    pub fn has_held_topology(&self) -> bool {
        self.traversal.is_some() || self.held_assertions().next().is_some()
    }
}

impl Default for EdgePayload {
    fn default() -> Self {
        Self::new()
    }
}
