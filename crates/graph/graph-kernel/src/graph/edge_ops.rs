// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Edge operations — assert, replay, dissolve, retract, traversal
//! recording.
//!
//! Extracted from `graph/mod.rs` per the 2026-05-11 kernel
//! decomposition pass. Stage 4 of the 2026-05-11 relation-taxonomy
//! plan removed the legacy `add_edge` / `remove_edges` /
//! `replay_add_edge_by_ids` / `replay_remove_edges_by_ids` paths and
//! their `EdgeType`-shaped helpers; everything goes through
//! [`EdgeAssertion`] / [`RelationSelector`] now.

use euclid::default::Point2D;
use petgraph::Direction;
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use uuid::Uuid;

use crate::types::GraphScope;

use super::edge_data::Traversal;
use super::edge_data::{SemanticStatement, SemanticStatementSpec, StatementAssert};
use super::edge_payload::EdgePayload;
use super::edge_taxonomy::{EdgeAssertion, RelationKind, RelationSelector, SemanticSubKind};
use super::identity::{EdgeKey, NodeKey, RelationKey};
use super::{DissolvedTraversalRecord, Graph};
use crate::persistence::PersistedEdge;

impl Graph {
    pub(crate) fn assert_relation(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        assertion: EdgeAssertion,
    ) -> Option<RelationKey> {
        self.assert_relation_as(
            from,
            to,
            assertion,
            self.write_author().asserter_iri(),
            None,
        )
    }

    pub(crate) fn assert_relation_as(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        assertion: EdgeAssertion,
        asserter_iri: String,
        asserted_at_ms: Option<u64>,
    ) -> Option<RelationKey> {
        if let EdgeAssertion::Semantic {
            sub_kind, label, ..
        } = assertion
        {
            return self
                .assert_semantic_statement(
                    from,
                    to,
                    SemanticStatementSpec {
                        predicate: super::predicate_iri(sub_kind).into(),
                        recognized_sub_kind: Some(sub_kind),
                        label,
                        graph_scope: GraphScope::Default,
                        provenance_iri: Some(asserter_iri),
                        asserted_at_ms,
                    },
                )
                .and_then(|(edge, outcome)| outcome.changed.then_some(edge));
        }
        let kind = match &assertion {
            EdgeAssertion::Containment { sub_kind } => RelationKind::Containment(*sub_kind),
            EdgeAssertion::Arrangement { sub_kind } => RelationKind::Arrangement(*sub_kind),
            EdgeAssertion::Imported { sub_kind } => RelationKind::Imported(*sub_kind),
            EdgeAssertion::Provenance { sub_kind } => RelationKind::Provenance(*sub_kind),
            EdgeAssertion::Semantic { .. } => unreachable!(),
        };
        let key = self
            .relation_bucket(from, to, super::built_in_relation_stratum(kind))
            .ok()?;
        if !self.get_relation_mut(key)?.assert_relation(assertion) {
            return None;
        }
        self.bump_revision();
        self.capture_statement_bucket(key);
        Some(key)
    }

    pub(crate) fn assert_surface_relation_as(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        assertion: EdgeAssertion,
        asserter_iri: String,
        asserted_at_ms: Option<u64>,
    ) -> Option<RelationKey> {
        if let EdgeAssertion::Semantic {
            sub_kind,
            label,
            decay_progress: _,
        } = assertion
        {
            return self
                .assert_surface_semantic_statement(
                    from,
                    to,
                    SemanticStatementSpec {
                        predicate: super::predicate_iri(sub_kind).into(),
                        recognized_sub_kind: Some(sub_kind),
                        label,
                        graph_scope: GraphScope::Default,
                        provenance_iri: Some(asserter_iri),
                        asserted_at_ms,
                    },
                )
                .and_then(|(edge, outcome)| outcome.changed.then_some(edge));
        }
        if !self.inner.contains_node(from) || !self.inner.contains_node(to) {
            return None;
        }
        if let Some(edge_key) = self.find_edge_key(from, to) {
            // Existing edge: assert onto its payload. A real change (a new relation on the pair —
            // a multiplicity bump a weighted signal reads) advances the revision.
            let changed = {
                let payload = self.inner.edge_mut(edge_key)?;
                payload.assert_relation(assertion)
            };
            if changed {
                self.bump_revision();
                return Some(RelationKey::Surface(edge_key));
            }
            return None;
        }
        let mut payload = EdgePayload::new();
        if !payload.assert_relation(assertion) {
            return None;
        }
        let edge_key = self.inner.connect(from, to, payload);
        self.bump_revision();
        Some(RelationKey::Surface(edge_key))
    }

    pub(crate) fn assert_semantic_relation_in_scope(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        sub_kind: SemanticSubKind,
        label: Option<String>,
        graph_scope: GraphScope,
    ) -> Option<RelationKey> {
        self.assert_semantic_statement(
            from,
            to,
            SemanticStatementSpec {
                predicate: super::predicate_iri(sub_kind).into(),
                recognized_sub_kind: Some(sub_kind),
                label,
                graph_scope,
                provenance_iri: Some(self.write_author().asserter_iri()),
                asserted_at_ms: None,
            },
        )
        .and_then(|(edge, outcome)| outcome.changed.then_some(edge))
    }

    /// Assert an **open predicate** semantic relation from `from` to `to`,
    /// identified by an IRI rather than a closed [`SemanticSubKind`]. Creates the
    /// pair-local edge bucket if absent and appends one open semantic statement to
    /// it; an edge carrying only such a statement still reports the `Semantic`
    /// family. Returns the edge, or `None` if either endpoint is missing or the
    /// same open predicate statement already exists. Write path for raw web
    /// predicates (linked-data ingest / knot `rel`s outside Mere's vocabulary).
    pub(crate) fn assert_semantic_predicate(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        predicate: String,
    ) -> Option<RelationKey> {
        self.assert_semantic_predicate_in_scope(from, to, predicate, GraphScope::Default)
    }

    pub(crate) fn assert_semantic_predicate_in_scope(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        predicate: String,
        graph_scope: GraphScope,
    ) -> Option<RelationKey> {
        self.assert_semantic_statement(
            from,
            to,
            SemanticStatementSpec {
                recognized_sub_kind: None,
                predicate,
                label: None,
                graph_scope,
                provenance_iri: Some(self.write_author().asserter_iri()),
                asserted_at_ms: None,
            },
        )
        .and_then(|(edge, outcome)| outcome.changed.then_some(edge))
    }

    /// Replay helper: add node only if UUID is not already present.
    pub(crate) fn replay_add_node_with_id_if_missing(
        &mut self,
        id: Uuid,
        url: String,
        position: Point2D<f32>,
    ) -> Option<NodeKey> {
        if self.get_node_key_by_id(id).is_some() {
            return None;
        }
        Some(self.add_node_with_id(id, url, position))
    }

    /// Replay helper: remove node by stable UUID.
    pub(crate) fn replay_remove_node_by_id(&mut self, node_id: Uuid) -> bool {
        let Some(key) = self.get_node_key_by_id(node_id) else {
            return false;
        };
        self.remove_node(key)
    }

    pub(crate) fn replay_retract_relations_by_ids(
        &mut self,
        from_id: Uuid,
        to_id: Uuid,
        selector: RelationSelector,
    ) -> usize {
        let Some(from_key) = self.get_node_key_by_id(from_id) else {
            return 0;
        };
        let Some(to_key) = self.get_node_key_by_id(to_id) else {
            return 0;
        };
        self.retract_surface_relations(from_key, to_key, selector)
    }

    pub(crate) fn replay_assert_relation_by_ids(
        &mut self,
        from_id: Uuid,
        to_id: Uuid,
        assertion: EdgeAssertion,
    ) -> Option<RelationKey> {
        let from_key = self.get_node_key_by_id(from_id)?;
        let to_key = self.get_node_key_by_id(to_id)?;
        self.assert_surface_relation_as(
            from_key,
            to_key,
            assertion,
            self.write_author().asserter_iri(),
            None,
        )
    }

    pub(crate) fn replay_set_edge_semantic_predicate_by_ids(
        &mut self,
        from_id: Uuid,
        to_id: Uuid,
        predicate: Option<String>,
    ) -> bool {
        let Some(from_key) = self.get_node_key_by_id(from_id) else {
            return false;
        };
        let Some(to_key) = self.get_node_key_by_id(to_id) else {
            return false;
        };
        let Some(edge_key) = self.find_edge_key(from_key, to_key) else {
            return false;
        };
        self.set_edge_semantic_predicate(edge_key, predicate)
    }

    pub(crate) fn replay_assert_semantic_predicate_by_ids(
        &mut self,
        from_id: Uuid,
        to_id: Uuid,
        predicate: String,
    ) -> Option<RelationKey> {
        let from_key = self.get_node_key_by_id(from_id)?;
        let to_key = self.get_node_key_by_id(to_id)?;
        self.assert_semantic_predicate(from_key, to_key, predicate)
    }

    pub(crate) fn replay_append_traversal_by_ids(
        &mut self,
        from_id: Uuid,
        to_id: Uuid,
        trigger: super::edge_taxonomy::NavigationTrigger,
        timestamp_ms: u64,
    ) -> bool {
        let Some(from_key) = self.get_node_key_by_id(from_id) else {
            return false;
        };
        let Some(to_key) = self.get_node_key_by_id(to_id) else {
            return false;
        };
        self.append_traversal(from_key, to_key, trigger, Some(timestamp_ms))
    }

    /// Dissolve helper: collect traversals from all incident edges and remove the node.
    pub(crate) fn dissolve_remove_node_collect_traversals(
        &mut self,
        key: NodeKey,
    ) -> Option<Vec<DissolvedTraversalRecord>> {
        let _ = self.get_node(key)?;

        let mut records = Vec::new();
        let inner = self.inner.inner();
        for edge in inner
            .edges_directed(key, Direction::Outgoing)
            .chain(inner.edges_directed(key, Direction::Incoming))
        {
            if edge.weight().traversals().is_empty() {
                continue;
            }

            let from_node = self.get_node(edge.source())?;
            let to_node = self.get_node(edge.target())?;
            records.push(DissolvedTraversalRecord {
                from_node_id: from_node.id,
                to_node_id: to_node.id,
                traversals: edge.weight().traversals().to_vec(),
            });
        }

        let _ = self.remove_node(key);
        Some(records)
    }

    /// Collect traversals from all incident edges without mutating graph state.
    pub fn collect_node_traversals(&self, key: NodeKey) -> Option<Vec<DissolvedTraversalRecord>> {
        let _ = self.get_node(key)?;

        let mut records = Vec::new();
        let inner = self.inner.inner();
        for edge in inner
            .edges_directed(key, Direction::Outgoing)
            .chain(inner.edges_directed(key, Direction::Incoming))
        {
            if edge.weight().traversals().is_empty() {
                continue;
            }

            let from_node = self.get_node(edge.source())?;
            let to_node = self.get_node(edge.target())?;
            records.push(DissolvedTraversalRecord {
                from_node_id: from_node.id,
                to_node_id: to_node.id,
                traversals: edge.weight().traversals().to_vec(),
            });
        }

        Some(records)
    }

    /// Statement-aware assert (petgraph-RDF Phase 1 write API): assert one
    /// semantic statement with full per-statement metadata on the `(from, to)`
    /// pair bucket, creating the bucket edge if absent. Content-dedup returns
    /// the EXISTING fact handle (updating its metadata in place); a genuine
    /// change bumps the revision. This compatibility surface writer remains
    /// during P2 integration; the checked counterpart uses durable placement.
    pub fn assert_semantic_statement(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        spec: SemanticStatementSpec,
    ) -> Option<(RelationKey, StatementAssert)> {
        self.try_assert_semantic_statement(from, to, spec).ok()
    }

    pub(crate) fn assert_surface_semantic_statement(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        mut spec: SemanticStatementSpec,
    ) -> Option<(RelationKey, StatementAssert)> {
        if spec.provenance_iri.is_none() {
            spec.provenance_iri = Some(self.write_author().asserter_iri());
        }
        if !self.inner.contains_node(from) || !self.inner.contains_node(to) {
            return None;
        }
        let edge_key = self
            .find_edge_key(from, to)
            .unwrap_or_else(|| self.inner.connect(from, to, EdgePayload::new()));
        let outcome = {
            let payload = self.inner.edge_mut(edge_key)?;
            payload.assert_semantic_statement(spec)
        };
        if outcome.changed {
            self.bump_revision();
            self.capture_semantic_pair(from, to);
        }
        Some((RelationKey::Surface(edge_key), outcome))
    }

    /// Assert a semantic statement whose id is ALREADY minted — the re-ingest /
    /// round-trip path (a reifier carried the fact handle, and preserving it is
    /// what makes `RDF -> kernel -> RDF` id-stable). Reasserting the same
    /// asserter updates its existing handle; endpoints must exist. The checked
    /// counterpart supplies declaration-aware placement and collision errors.
    pub fn assert_persisted_semantic_statement(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        statement: SemanticStatement,
    ) -> Option<RelationKey> {
        self.try_assert_persisted_semantic_statement(from, to, statement)
            .ok()
    }

    pub(crate) fn assert_surface_persisted_semantic_statement(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        mut statement: SemanticStatement,
    ) -> Option<RelationKey> {
        if statement.provenance_iri.is_none() {
            statement.provenance_iri = Some(self.write_author().asserter_iri());
        }
        if !self.inner.contains_node(from) || !self.inner.contains_node(to) {
            return None;
        }
        let edge_key = self
            .find_edge_key(from, to)
            .unwrap_or_else(|| self.inner.connect(from, to, EdgePayload::new()));
        let changed = {
            let payload = self.inner.edge_mut(edge_key)?;
            payload.upsert_persisted_semantic_statement(statement)
        };
        if changed {
            self.bump_revision();
            self.capture_semantic_pair(from, to);
        }
        Some(RelationKey::Surface(edge_key))
    }

    /// Precise retract by fact handle (the id `assert_semantic_statement`
    /// returned, also carried on every projected reifier). Removes the
    /// statement from the pair bucket; an emptied payload removes the petgraph
    /// edge itself, so "there is an edge A -> B" keeps meaning "at least one
    /// fact or other family payload exists on the pair".
    pub fn retract_semantic_statement(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        statement_id: &str,
    ) -> bool {
        let Some((owner, _)) = self.find_semantic_statement(statement_id) else {
            return false;
        };
        if matches!(owner, RelationKey::Resource(_)) {
            return self.retract_assertion(statement_id);
        }
        if let RelationKey::Surface(key) = owner
            && self.inner.inner().edge_endpoints(key) == Some((from, to))
        {
            return self.retract_assertion(statement_id);
        }
        false
    }

    /// Keep assertion ids and metadata exact through journal replay and undo.
    pub(crate) fn capture_semantic_pair(&self, from: NodeKey, to: NodeKey) {
        let (Some(from_node), Some(to_node)) = (self.get_node(from), self.get_node(to)) else {
            return;
        };
        self.record_delta(&super::capture::CapturedDelta::ReplaySetEdgesByIds {
            from_id: from_node.id.to_string(),
            to_id: to_node.id.to_string(),
            edges: self.persisted_edges_between(from, to),
        });
    }

    pub(crate) fn retract_relations(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        selector: RelationSelector,
    ) -> usize {
        let mut removed = self.retract_surface_relations(from, to, selector);
        if removed > 0 {
            self.capture_semantic_pair(from, to);
        }
        let Some((source, target)) = self.shown_resource_id(from).zip(self.shown_resource_id(to))
        else {
            return removed;
        };
        let keys: Vec<_> = self
            .resource_relations()
            .filter(|(_, a, b, payload)| {
                (*a, *b) == (source, target) && payload.has_relation(selector)
            })
            .map(|(key, _, _, _)| key)
            .collect();
        let mut resource_removed = 0;
        for key in keys {
            let payload = self
                .resources
                .edge_mut(key.raw())
                .expect("resource relation");
            if payload.retract_relation(selector) {
                resource_removed += 1;
                if payload.is_empty() {
                    self.resources.disconnect(key.raw());
                }
            }
        }
        if resource_removed > 0 {
            self.bump_revision();
            self.capture_resource_pair(source, target);
            removed += resource_removed;
        }
        removed
    }

    fn retract_surface_relations(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        selector: RelationSelector,
    ) -> usize {
        let edge_ids: Vec<EdgeKey> = self
            .inner
            .inner()
            .edge_references()
            .filter(|edge| {
                edge.source() == from && edge.target() == to && edge.weight().has_relation(selector)
            })
            .map(|edge| edge.id())
            .collect();

        let mut removed = 0usize;
        let mut edges_to_delete = Vec::new();
        for edge_id in edge_ids {
            if let Some(payload) = self.inner.edge_mut(edge_id)
                && payload.retract_relation(selector)
            {
                removed += 1;
                if payload.is_empty() {
                    edges_to_delete.push(edge_id);
                }
            }
        }
        for edge_id in edges_to_delete {
            let _ = self.inner.disconnect(edge_id);
        }
        if removed > 0 {
            self.bump_revision();
        }
        removed
    }

    /// Replace every relation from `from` to `to` with `edges`, given in
    /// persisted form, or with none. Undo's exact edge write: the relations land
    /// as a snapshot load would make them.
    pub(crate) fn set_edges_between(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        edges: &[PersistedEdge],
    ) {
        let existing: Vec<EdgeKey> = self
            .inner
            .inner()
            .edges_connecting(from, to)
            .map(|edge| edge.id())
            .collect();
        for key in existing {
            let _ = self.inner.disconnect(key);
        }
        for edge in edges {
            self.restore_persisted_edge(from, to, edge);
        }
        let restored: Vec<_> = self
            .inner
            .inner()
            .edges_connecting(from, to)
            .map(|edge| edge.id())
            .collect();
        for key in restored {
            if let Some(payload) = self.inner.edge_mut(key)
                && let Some(semantic) = &mut payload.semantic
            {
                for statement in &mut semantic.statements {
                    statement.normalize_legacy_asserter();
                }
            }
        }
        self.bump_revision();
    }

    /// Get a mutable edge payload by key.
    pub(crate) fn get_edge_mut(&mut self, key: EdgeKey) -> Option<&mut EdgePayload> {
        self.inner.edge_mut(key)
    }

    /// Legacy replay only: set the aggregate predicate on an existing edge.
    /// Live corrections retract one statement and assert its replacement.
    pub(crate) fn set_edge_semantic_predicate(
        &mut self,
        key: EdgeKey,
        predicate: Option<String>,
    ) -> bool {
        self.set_edge_semantic_predicate_as(key, predicate, self.write_author().asserter_iri())
    }

    /// Legacy replay only, with the old record's attribution envelope.
    pub(crate) fn set_edge_semantic_predicate_as(
        &mut self,
        key: EdgeKey,
        predicate: Option<String>,
        asserter: String,
    ) -> bool {
        let Some(payload) = self.inner.edge_mut(key) else {
            return false;
        };
        let before = payload.clone();
        payload.set_semantic_predicate(predicate);
        if let Some(semantic) = payload.semantic.as_mut() {
            for statement in &mut semantic.statements {
                if statement.provenance_iri.is_none() {
                    statement.provenance_iri = Some(asserter.clone());
                }
            }
        }
        if *payload != before {
            self.bump_revision();
        }
        true
    }

    /// Get an edge payload by key.
    pub fn get_edge(&self, key: EdgeKey) -> Option<&EdgePayload> {
        self.inner.edge(key)
    }

    /// Find the first directed edge key between two nodes.
    pub fn find_edge_key(&self, from: NodeKey, to: NodeKey) -> Option<EdgeKey> {
        self.inner.inner().find_edge(from, to)
    }

    /// Every edge between `a` and `b` in either direction: all `a->b`
    /// edges first, then all `b->a` edges, each run in petgraph's
    /// `edges_connecting` order (deterministic for a given graph). The
    /// substrate is a multigraph, so a pair may carry an antiparallel
    /// or parallel set; read predicates that mean "the pair" rather
    /// than "the first arc" go through here instead of
    /// [`Self::find_edge_key`]. Self-loops (`a == b`) are yielded once.
    pub fn edges_between_undirected(
        &self,
        a: NodeKey,
        b: NodeKey,
    ) -> impl Iterator<Item = (EdgeKey, &EdgePayload)> + '_ {
        use petgraph::visit::EdgeRef;
        let inner = self.inner.inner();
        let forward = inner.edges_connecting(a, b);
        let backward = inner.edges_connecting(b, a).filter(move |_| a != b);
        forward
            .chain(backward)
            .map(|edge| (edge.id(), edge.weight()))
    }

    /// Append a traversal event to an existing edge, or create an edge carrying the traversal.
    pub(crate) fn push_traversal(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        traversal: Traversal,
    ) -> bool {
        if from == to || !self.inner.contains_node(from) || !self.inner.contains_node(to) {
            return false;
        }
        if let Some(edge_key) = self.find_edge_key(from, to)
            && let Some(payload) = self.inner.edge_mut(edge_key)
        {
            // Existing edge: a re-visit appends a traversal but does not change the structure, so
            // the revision holds (navigation re-visits must not churn the structural caches).
            payload.push_traversal(traversal);
            return true;
        }
        let mut payload = EdgePayload::new();
        payload.push_traversal(traversal);
        let _ = self.inner.connect(from, to, payload);
        self.bump_revision();
        true
    }

    /// Append a traversal event by `(trigger, timestamp_ms?)`.
    /// Thin wrapper over [`Self::push_traversal`] matching the
    /// 2026-05-11 relation-taxonomy plan §7 signature — callers
    /// that just have a `NavigationTrigger` don't need to build a
    /// `Traversal` struct themselves. `timestamp_ms = None` stamps
    /// now-via-`Traversal::now`; `Some(t)` uses the supplied
    /// timestamp (importers, replay).
    pub(crate) fn append_traversal(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        trigger: super::edge_taxonomy::NavigationTrigger,
        timestamp_ms: Option<u64>,
    ) -> bool {
        let traversal = match timestamp_ms {
            Some(timestamp_ms) => Traversal {
                timestamp_ms,
                trigger,
            },
            None => Traversal::now(trigger),
        };
        self.push_traversal(from, to, traversal)
    }
}

#[cfg(test)]
mod legacy_predicate_revision_tests {
    use super::*;

    fn fixture(asserter: Option<&str>) -> (Graph, NodeKey, NodeKey, EdgeKey) {
        let mut graph = Graph::new();
        let from = graph.add_node("https://source.test/".into(), Point2D::zero());
        let to = graph.add_node("https://target.test/".into(), Point2D::zero());
        let mut payload = EdgePayload::new();
        payload.push_persisted_semantic_statement(SemanticStatement {
            statement_id: "carried legacy handle".into(),
            predicate: "https://predicate.test/before".into(),
            recognized_sub_kind: None,
            label: Some("held label".into()),
            graph_scope: GraphScope::User,
            provenance_iri: asserter.map(str::to_owned),
            asserted_at_ms: Some(17),
        });
        let edge = graph.inner.connect(from, to, payload);
        (graph, from, to, edge)
    }

    #[test]
    fn legacy_predicate_replay_advances_revision_only_for_changed_payload() {
        use super::super::apply::{GraphDelta, GraphDeltaResult, apply_graph_delta};

        let (mut graph, from, to, edge) = fixture(Some("https://author.test/"));
        let original = graph.get_edge(edge).unwrap().semantic_statements()[0].clone();
        let from_id = graph.get_node(from).unwrap().id;
        let to_id = graph.get_node(to).unwrap().id;
        let delta = GraphDelta::ReplaySetEdgeSemanticPredicateByIds {
            from_id,
            to_id,
            predicate: Some("https://predicate.test/after".into()),
            asserter_iri: "https://other-author.test/".into(),
        };
        let before_revision = graph.revision();
        assert!(matches!(
            apply_graph_delta(&mut graph, delta.clone()),
            GraphDeltaResult::NodeMetadataUpdated(true)
        ));
        assert_eq!(graph.revision(), before_revision + 1);
        let mut expected = original;
        expected.predicate = "https://predicate.test/after".into();
        assert_eq!(
            graph.get_edge(edge).unwrap().semantic_statements(),
            &[expected]
        );

        let edited = graph.get_edge(edge).unwrap().clone();
        let edited_revision = graph.revision();
        assert!(matches!(
            apply_graph_delta(&mut graph, delta),
            GraphDeltaResult::NodeMetadataUpdated(true)
        ));
        assert_eq!(graph.get_edge(edge), Some(&edited));
        assert_eq!(
            graph.revision(),
            edited_revision,
            "same input remains a no-op"
        );

        assert!(graph.set_edge_semantic_predicate_as(edge, None, "unused".into()));
        assert!(
            graph
                .get_edge(edge)
                .unwrap()
                .semantic_statements()
                .is_empty()
        );
        assert_eq!(graph.revision(), edited_revision + 1);
        let cleared_revision = graph.revision();
        assert!(graph.set_edge_semantic_predicate_as(edge, None, "unused".into()));
        assert_eq!(graph.revision(), cleared_revision);
        assert!(!graph.set_edge_semantic_predicate_as(EdgeKey::new(1_000), None, "unused".into()));
        assert_eq!(graph.revision(), cleared_revision);
    }

    #[test]
    fn legacy_predicate_attribution_repair_alone_advances_revision() {
        let (mut graph, _, _, edge) = fixture(None);
        let original = graph.get_edge(edge).unwrap().semantic_statements()[0].clone();
        let before_revision = graph.revision();
        assert!(graph.set_edge_semantic_predicate_as(
            edge,
            Some(original.predicate.clone()),
            "https://author.test/".into()
        ));
        let mut expected = original;
        expected.provenance_iri = Some("https://author.test/".into());
        assert_eq!(
            graph.get_edge(edge).unwrap().semantic_statements(),
            &[expected]
        );
        assert_eq!(graph.revision(), before_revision + 1);
        let repaired = graph.get_edge(edge).unwrap().clone();
        let repaired_revision = graph.revision();
        assert!(graph.set_edge_semantic_predicate_as(
            edge,
            Some("https://predicate.test/before".into()),
            "https://other-author.test/".into()
        ));
        assert_eq!(graph.get_edge(edge), Some(&repaired));
        assert_eq!(
            graph.revision(),
            repaired_revision,
            "existing attribution stays exact"
        );
    }
}
