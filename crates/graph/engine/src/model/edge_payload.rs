// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One assertion list per directed pair. Every family's records carry the
//! same exact identity and lifecycle; a record's predicate IRI says what it
//! is, read through the payload's [`Vocabulary`] (declared-lens D20). The
//! surface's traversal events live with the surface's own edge record (D22).

use super::edge_data::{
    AssertionData, SemanticData, SemanticStatement, SemanticStatementSpec, StatementAssert,
    UNKNOWN_LEGACY_ASSERTER_IRI,
};
use super::relation::{RelationSelector, Vocabulary};
use crate::model::types::{AssertionWithdrawal, GraphScope};
use crate::profile::intern::TermId;
use portolan::{Classified, Predicated, RelationClass};
use rkyv::{Archive, Deserialize, Serialize};
use std::collections::BTreeSet;

/// Borrowed semantic compatibility projection, never another assertion store.
pub struct SemanticDataView<'a> {
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
        }
    }

    /// An empty payload that classifies by `vocabulary`.
    pub fn with_vocabulary(vocabulary: Vocabulary) -> Self {
        let mut payload = Self::new();
        payload.assertions.vocabulary = vocabulary;
        payload
    }

    pub fn vocabulary(&self) -> Vocabulary {
        self.assertions.vocabulary
    }

    /// Classify by `vocabulary` from now on.
    pub fn set_vocabulary(&mut self, vocabulary: Vocabulary) {
        self.assertions.set_vocabulary(vocabulary);
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
        let vocabulary = self.vocabulary();
        let count = self
            .assertions()
            .partition_point(|s| s.is_semantic_in(vocabulary));
        &self.assertions()[..count]
    }

    pub fn held_semantic_statements(&self) -> impl Iterator<Item = &SemanticStatement> {
        self.semantic_statements()
            .iter()
            .filter(|statement| statement.is_held())
    }

    pub fn semantic_data(&self) -> Option<SemanticDataView<'_>> {
        (!self.semantic_statements().is_empty()).then(|| SemanticDataView {
            label: &self.assertions.label,
            predicate: &self.assertions.predicate,
            statements: self.semantic_statements(),
        })
    }

    /// The family a predicate counts in here.
    pub fn family_of(&self, predicate: &str) -> TermId {
        self.vocabulary().relation_family(predicate)
    }

    pub fn has_relation(&self, selector: &RelationSelector) -> bool {
        let vocabulary = self.vocabulary();
        self.held_assertions()
            .any(|statement| selector.matches(&statement.predicate, vocabulary))
    }

    /// Whether a held record names `predicate`.
    pub fn has_predicate(&self, predicate: &str) -> bool {
        self.held_assertions().any(|s| s.predicate == predicate)
    }

    /// Legacy membership ensure. A caller who has an independent reason uses
    /// the statement-aware Graph writer and its exact returned assertion ID.
    pub fn assert_relation(&mut self, predicate: &str) -> bool {
        self.assert_relation_with_context(predicate, None, None, None)
    }

    /// Hold `predicate` once: a non-semantic family's record is a membership,
    /// so a second is no change; a semantic one dedups by content.
    pub fn assert_relation_with_context(
        &mut self,
        predicate: &str,
        label: Option<String>,
        asserter: Option<String>,
        at_ms: Option<u64>,
    ) -> bool {
        if !self.vocabulary().is_semantic(predicate) && self.has_predicate(predicate) {
            return false;
        }
        self.assertions
            .assert_statement(SemanticStatementSpec {
                predicate: predicate.into(),
                label,
                provenance_iri: asserter.or_else(|| Some(UNKNOWN_LEGACY_ASSERTER_IRI.into())),
                asserted_at_ms: at_ms,
                ..Default::default()
            })
            .changed
    }

    /// Physical removal remains reserved for regeneration, migration and
    /// compaction. Authored withdrawals use Graph's exact-ID lifecycle writer.
    pub fn retract_relation(&mut self, selector: &RelationSelector) -> bool {
        let vocabulary = self.vocabulary();
        let before = self.assertions().len();
        self.assertions
            .statements
            .retain(|s| !s.is_held() || !selector.matches(&s.predicate, vocabulary));
        if self.assertions().len() == before {
            return false;
        }
        self.assertions.rebuild_compat();
        true
    }

    pub fn is_empty(&self) -> bool {
        self.assertions().is_empty()
    }
    pub fn label(&self) -> Option<&str> {
        self.assertions.label.as_deref()
    }

    pub fn assert_semantic_predicate(&mut self, predicate: String) -> bool {
        self.assert_semantic_predicate_in_scope(predicate, GraphScope::Default)
    }
    /// Hold `predicate` with `label` in `scope`, deduped by content.
    pub fn assert_semantic_relation_in_scope(
        &mut self,
        predicate: &str,
        label: Option<String>,
        scope: GraphScope,
    ) -> bool {
        self.assertions
            .insert_statement(predicate.into(), label, scope, None, None)
    }
    pub fn assert_semantic_predicate_in_scope(
        &mut self,
        predicate: String,
        scope: GraphScope,
    ) -> bool {
        self.assertions
            .insert_statement(predicate, None, scope, None, None)
    }
    pub fn assert_semantic_statement(&mut self, spec: SemanticStatementSpec) -> StatementAssert {
        self.assertions.assert_statement(spec)
    }
    pub fn withdraw_semantic_statement(
        &mut self,
        id: &str,
        withdrawal: AssertionWithdrawal,
    ) -> bool {
        self.assertions.withdraw_statement(id, withdrawal)
    }
    pub fn compact_semantic_statement(&mut self, id: &str) -> bool {
        self.assertions.compact_statement(id)
    }
    pub fn upsert_persisted_semantic_statement(&mut self, statement: SemanticStatement) -> bool {
        self.assertions.upsert_persisted_statement(statement)
    }
    pub fn push_persisted_semantic_statement(&mut self, statement: SemanticStatement) -> bool {
        self.assertions.push_persisted_statement(statement)
    }
    pub fn set_semantic_predicate(&mut self, predicate: Option<String>) {
        self.assertions.set_statement_predicate(predicate);
    }

    /// The held predicates of `family`, as this payload's table reads them.
    pub fn held_predicates(&self, family: TermId) -> BTreeSet<&str> {
        self.held_assertions()
            .map(|s| s.predicate.as_str())
            .filter(|p| self.family_of(p) == family)
            .collect()
    }

    /// The families of the held records.
    pub fn families(&self) -> BTreeSet<TermId> {
        self.held_assertions()
            .map(|s| self.family_of(&s.predicate))
            .collect()
    }

    pub fn has_held_topology(&self) -> bool {
        self.held_assertions().next().is_some()
    }
}

impl Default for EdgePayload {
    fn default() -> Self {
        Self::new()
    }
}

/// The single predicate IRI a semantic edge projects, or `None` if the edge
/// carries no semantic relation (a containment, imported, provenance or
/// workspace edge). Withdrawn assertions remain in history but do not
/// project. This single-predicate capability selects the first held semantic
/// statement; readers needing every statement use the assertion APIs.
fn edge_predicate(payload: &EdgePayload) -> Option<&str> {
    payload
        .held_semantic_statements()
        .next()
        .map(|statement| statement.predicate.as_str())
}

impl Predicated for EdgePayload {
    fn predicate(&self) -> Option<&str> {
        edge_predicate(self)
    }
}

impl Classified for EdgePayload {
    fn class(&self) -> RelationClass {
        match edge_predicate(self) {
            // A semantic edge joins the shared ring as an open predicate (its IRI),
            // so it projects to RDF.
            Some(iri) => RelationClass::open(iri.to_string()),
            // An experience-layer edge is mere's private family: it does not project.
            None => RelationClass::app("mere", 0),
        }
    }
}
