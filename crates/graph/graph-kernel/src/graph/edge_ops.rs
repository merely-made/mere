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

use crate::time::unix_epoch_millis;
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
                        basis: Default::default(),
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
        let derived = is_derived_containment(kind);
        let stratum = super::built_in_relation_stratum(kind);
        if derived
            && self.derived_output_id_is_retained_for_pair(
                match stratum {
                    super::GraphStratum::Surface => "surface",
                    super::GraphStratum::Resource => "resource",
                },
                from,
                to,
                kind,
            )
        {
            return None;
        }
        let key = self.relation_bucket(from, to, stratum).ok()?;
        if derived && self.derived_output_id_is_retained(key, kind) {
            return None;
        }
        if !self.get_relation_mut(key)?.assert_relation_with_context(
            assertion,
            Some(asserter_iri),
            if derived {
                None
            } else {
                Some(asserted_at_ms.unwrap_or_else(unix_epoch_millis))
            },
        ) {
            return None;
        }
        if derived {
            self.normalize_derived_membership(key, kind);
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
                        basis: Default::default(),
                    },
                )
                .and_then(|(edge, outcome)| outcome.changed.then_some(edge));
        }
        let derived_kind = match &assertion {
            EdgeAssertion::Containment { sub_kind }
                if matches!(
                    sub_kind,
                    super::ContainmentSubKind::UrlPath | super::ContainmentSubKind::Domain
                ) =>
            {
                Some(RelationKind::Containment(*sub_kind))
            },
            _ => None,
        };
        if !self.inner.contains_node(from) || !self.inner.contains_node(to) {
            return None;
        }
        if let Some(edge_key) = self.find_edge_key(from, to) {
            if let Some(kind) = derived_kind {
                if self.derived_output_id_is_retained(RelationKey::Surface(edge_key), kind) {
                    return None;
                }
            }
            // Existing edge: assert onto its payload. A real change (a new relation on the pair —
            // a multiplicity bump a weighted signal reads) advances the revision.
            let changed = {
                let payload = self.inner.edge_mut(edge_key)?;
                payload.assert_relation_with_context(
                    assertion,
                    Some(asserter_iri.clone()),
                    asserted_at_ms,
                )
            };
            if changed {
                if let Some(kind) = derived_kind {
                    self.normalize_derived_membership(RelationKey::Surface(edge_key), kind);
                }
                self.bump_revision();
                return Some(RelationKey::Surface(edge_key));
            }
            return None;
        }
        if let Some(kind) = derived_kind {
            if self.derived_output_id_is_retained_for_pair("surface", from, to, kind) {
                return None;
            }
        }
        let mut payload = EdgePayload::new();
        if !payload.assert_relation_with_context(
            assertion,
            Some(asserter_iri.clone()),
            asserted_at_ms,
        ) {
            return None;
        }
        let edge_key = self.inner.connect(from, to, payload);
        if let Some(kind) = derived_kind {
            self.normalize_derived_membership(RelationKey::Surface(edge_key), kind);
        }
        self.bump_revision();
        Some(RelationKey::Surface(edge_key))
    }

    /// Derived containment is a rule output, not authored history. Its ID is
    /// stable for this directed source bucket and its time remains undated.
    fn normalize_derived_membership(&mut self, key: RelationKey, kind: RelationKind) {
        let endpoints = match key {
            RelationKey::Surface(key) => self.inner.inner().edge_endpoints(key).map(|(a, b)| {
                (
                    "surface",
                    self.inner.node(a).unwrap().id,
                    self.inner.node(b).unwrap().id,
                )
            }),
            RelationKey::Resource(key) => {
                self.resources
                    .inner()
                    .edge_endpoints(key.raw())
                    .map(|(a, b)| {
                        (
                            "resource",
                            self.resources.node(a).unwrap().id(),
                            self.resources.node(b).unwrap().id(),
                        )
                    })
            },
        };
        let Some((stratum, from, to)) = endpoints else {
            return;
        };
        let id = derived_containment_statement_id(stratum, from, to, kind);
        if self.find_semantic_statement(&id).is_some() {
            return;
        }
        for statement in &mut self.get_relation_mut(key).unwrap().assertions.statements {
            if !statement.is_held() || statement.relation_kind() != kind {
                continue;
            }
            statement.statement_id = id.clone();
            statement.provenance_iri = Some("urn:mere:rule:derived-containment:v1".into());
            statement.first_asserted_ms = None;
            statement.asserted_at_ms = None;
            statement.basis = crate::types::AssertionBasis {
                method: Some(crate::types::AssertionMethod {
                    iri: "urn:mere:method:derived-containment".into(),
                    version: Some("1".into()),
                }),
                ..Default::default()
            };
        }
    }

    fn derived_output_id_is_retained(&self, key: RelationKey, kind: RelationKind) -> bool {
        let endpoints = match key {
            RelationKey::Surface(key) => {
                self.inner.inner().edge_endpoints(key).and_then(|(a, b)| {
                    Some(("surface", self.inner.node(a)?.id, self.inner.node(b)?.id))
                })
            },
            RelationKey::Resource(key) => self
                .resources
                .inner()
                .edge_endpoints(key.raw())
                .and_then(|(a, b)| {
                    Some((
                        "resource",
                        self.resources.node(a)?.id(),
                        self.resources.node(b)?.id(),
                    ))
                }),
        };
        endpoints.is_some_and(|(stratum, from, to)| {
            self.derived_output_id_is_retained_for_identity(stratum, from, to, kind)
        })
    }

    fn derived_output_id_is_retained_for_pair(
        &self,
        stratum: &str,
        from_key: NodeKey,
        to_key: NodeKey,
        kind: RelationKind,
    ) -> bool {
        let (Some(from), Some(to)) = (self.get_node(from_key), self.get_node(to_key)) else {
            return false;
        };
        let (from, to) = if stratum == "resource" {
            let from_id = self
                .shown_resource_id(from_key)
                .unwrap_or_else(|| super::resource::ResourceNode::new(from.url()).id());
            let to_id = self
                .shown_resource_id(to_key)
                .unwrap_or_else(|| super::resource::ResourceNode::new(to.url()).id());
            (from_id, to_id)
        } else {
            (from.id, to.id)
        };
        self.derived_output_id_is_retained_for_identity(stratum, from, to, kind)
    }

    fn derived_output_id_is_retained_for_identity(
        &self,
        stratum: &str,
        from: Uuid,
        to: Uuid,
        kind: RelationKind,
    ) -> bool {
        let id = derived_containment_statement_id(stratum, from, to, kind);
        self.find_semantic_statement(&id).is_some()
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
                basis: Default::default(),
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
                basis: Default::default(),
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
        if selector != RelationSelector::Family(super::EdgeFamily::Traversal) {
            return self.withdraw_selected_semantic_assertions(from_key, to_key, selector, false);
        }
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
        self.replay_surface_relation_as(
            from_key,
            to_key,
            assertion,
            self.write_author().asserter_iri(),
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
        asserter_iri: String,
    ) -> Option<RelationKey> {
        let from_key = self.get_node_key_by_id(from_id)?;
        let to_key = self.get_node_key_by_id(to_id)?;
        if !self.inner.contains_node(from_key) || !self.inner.contains_node(to_key) {
            return None;
        }
        let edge_key = self
            .find_edge_key(from_key, to_key)
            .unwrap_or_else(|| self.inner.connect(from_key, to_key, EdgePayload::new()));
        let outcome =
            self.inner
                .edge_mut(edge_key)?
                .assert_semantic_statement(SemanticStatementSpec {
                    predicate,
                    recognized_sub_kind: None,
                    label: None,
                    graph_scope: GraphScope::Default,
                    provenance_iri: Some(asserter_iri),
                    asserted_at_ms: None,
                    basis: Default::default(),
                });
        if outcome.changed {
            self.bump_revision();
            self.capture_semantic_pair(from_key, to_key);
        }
        Some(RelationKey::Surface(edge_key))
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
        spec = spec.normalized();
        if spec.provenance_iri.is_none() {
            spec.provenance_iri = Some(self.write_author().asserter_iri());
        }
        if !self.inner.contains_node(from) || !self.inner.contains_node(to) {
            return None;
        }
        if let Some((key, existing)) = self
            .find_edge_key(from, to)
            .and_then(|key| self.inner.edge(key).map(|payload| (key, payload)))
            .and_then(|(key, payload)| {
                payload
                    .assertions()
                    .iter()
                    .find(|statement| {
                        statement.withdrawn.is_none()
                            && statement.predicate == spec.predicate
                            && statement.recognized_sub_kind == spec.recognized_sub_kind
                            && statement.graph_scope == spec.graph_scope
                            && statement.provenance_iri == spec.provenance_iri
                            && statement.basis.same_reason_as(&spec.basis)
                    })
                    .map(|statement| (key, statement.statement_id.clone()))
            })
        {
            let changed = self.update_assertion_metadata_if_requested(
                &existing,
                spec.label,
                spec.asserted_at_ms,
                Some(spec.basis),
            );
            return Some((
                RelationKey::Surface(key),
                StatementAssert {
                    statement_id: existing,
                    changed,
                },
            ));
        }
        spec.asserted_at_ms.get_or_insert_with(unix_epoch_millis);
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

    /// Historical replay path. Missing source time stays absent; replay never
    /// stamps a wall clock into a prior journal prefix.
    pub(crate) fn replay_surface_relation_as(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        assertion: EdgeAssertion,
        asserter_iri: String,
    ) -> Option<RelationKey> {
        if let EdgeAssertion::Semantic {
            sub_kind,
            label,
            decay_progress: _,
        } = assertion
        {
            if !self.inner.contains_node(from) || !self.inner.contains_node(to) {
                return None;
            }
            let edge_key = self
                .find_edge_key(from, to)
                .unwrap_or_else(|| self.inner.connect(from, to, EdgePayload::new()));
            let outcome =
                self.inner
                    .edge_mut(edge_key)?
                    .assert_semantic_statement(SemanticStatementSpec {
                        predicate: super::predicate_iri(sub_kind).into(),
                        recognized_sub_kind: Some(sub_kind),
                        label,
                        graph_scope: GraphScope::Default,
                        provenance_iri: Some(asserter_iri),
                        asserted_at_ms: None,
                        basis: Default::default(),
                    });
            if outcome.changed {
                self.bump_revision();
                self.capture_semantic_pair(from, to);
            }
            return Some(RelationKey::Surface(edge_key));
        }
        let before: std::collections::BTreeSet<_> = self
            .edges_between_undirected(from, to)
            .flat_map(|(_, payload)| payload.assertions())
            .map(|s| s.statement_id.clone())
            .collect();
        let result = self.assert_surface_relation_as(from, to, assertion, asserter_iri, None);
        if let Some(RelationKey::Surface(key)) = result {
            let from_id = self.get_node(from)?.id;
            let to_id = self.get_node(to)?.id;
            let payload = self.inner.edge_mut(key)?;
            let mut generations = std::collections::BTreeMap::<String, usize>::new();
            for statement in &payload.assertions.statements {
                if before.contains(&statement.statement_id) {
                    *generations.entry(statement.predicate.clone()).or_default() += 1;
                }
            }
            for statement in &mut payload.assertions.statements {
                if !before.contains(&statement.statement_id)
                    && !is_derived_containment(statement.relation_kind())
                {
                    let generation = *generations.get(&statement.predicate).unwrap_or(&0);
                    let bytes = serde_json::to_vec(&(
                        "mere.legacy-replay-assertion",
                        1u8,
                        "surface",
                        from_id,
                        to_id,
                        &statement.predicate,
                        generation,
                    ))
                    .expect("string tuple JSON");
                    statement.statement_id =
                        Uuid::new_v5(&Uuid::NAMESPACE_URL, &bytes).urn().to_string();
                }
            }
        }
        result
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

    /// Compatibility spelling for an authored exact-ID withdrawal. The
    /// assertion remains as a tombstone in its original pair bucket.
    pub fn retract_semantic_statement(
        &mut self,
        _from: NodeKey,
        _to: NodeKey,
        statement_id: &str,
    ) -> bool {
        self.withdraw_semantic_assertion(statement_id, None, None)
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
        if selector != RelationSelector::Family(super::EdgeFamily::Traversal) {
            return self.withdraw_selected_semantic_assertions(from, to, selector, true);
        }
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

    /// Historical selector replay retains an undated tombstone instead of
    /// stamping replay time. Exact journal pair snapshots remain the primary
    /// replay form; this covers legacy selector deltas only.
    pub(crate) fn replay_retract_relations(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        selector: RelationSelector,
    ) -> usize {
        if selector != RelationSelector::Family(super::EdgeFamily::Traversal) {
            return self.withdraw_selected_semantic_assertions(from, to, selector, false);
        }
        self.retract_relations(from, to, selector)
    }

    fn withdraw_selected_semantic_assertions(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        selector: RelationSelector,
        authored: bool,
    ) -> usize {
        let matches_selector = |statement: &SemanticStatement| {
            super::edge_payload::selector_matches_kind(selector, statement.relation_kind())
        };
        let mut ids = self
            .inner
            .inner()
            .edges_connecting(from, to)
            .flat_map(|edge| edge.weight().held_assertions())
            .filter(|statement| matches_selector(statement))
            .map(|statement| statement.statement_id.clone())
            .collect::<Vec<_>>();
        if authored
            && let Some((source, target)) =
                self.shown_resource_id(from).zip(self.shown_resource_id(to))
        {
            ids.extend(
                self.resource_relations()
                    .filter(|(_, a, b, _)| (*a, *b) == (source, target))
                    .flat_map(|(_, _, _, payload)| payload.held_assertions())
                    .filter(|statement| matches_selector(statement))
                    .map(|statement| statement.statement_id.clone()),
            );
        }
        ids.sort();
        ids.dedup();
        ids.into_iter()
            .filter(|id| {
                if authored {
                    self.withdraw_semantic_assertion(id, None, None)
                } else {
                    let author = self.write_author().asserter_iri();
                    self.apply_imported_semantic_withdrawal(id, None, Some(author))
                }
            })
            .count()
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
    /// persisted form, or with none. Exact replay and undo retain each carried
    /// row, rather than merging parallel payloads through the legacy loader.
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
        // Pair iteration visits the newest edge first, so insert in reverse
        // order to preserve the exact capture's row order.
        for (ordinal, edge) in edges.iter().enumerate().rev() {
            let mut payload = super::snapshot::payload_from_persisted_at(
                edge,
                super::GraphStratum::Surface,
                ordinal,
            );
            {
                let semantic = &mut payload.assertions;
                for statement in &mut semantic.statements {
                    statement.normalize_legacy_asserter();
                }
            }
            if !payload.is_empty() || edge.semantic.is_some() || edge.traversal.is_some() {
                self.inner.connect(from, to, payload);
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
        {
            let semantic = &mut payload.assertions;
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

fn derived_containment_statement_id(
    stratum: &str,
    from: Uuid,
    to: Uuid,
    kind: RelationKind,
) -> String {
    let predicate = kind
        .predicate_iri()
        .expect("derived containment has a canonical predicate");
    let bytes = serde_json::to_vec(&(
        "mere.derived-containment",
        1u8,
        stratum,
        from,
        to,
        predicate,
    ))
    .expect("derived containment identity is serializable");
    Uuid::new_v5(&Uuid::NAMESPACE_URL, &bytes).urn().to_string()
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
            first_asserted_ms: Some(17),
            withdrawn: None,
            basis: Default::default(),
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
        let payload = graph.get_edge(edge).unwrap();
        assert!(payload.held_assertions().next().is_none());
        assert_eq!(payload.semantic_statements().len(), 1);
        assert!(payload.semantic_statements()[0].withdrawn.is_some());
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

fn is_derived_containment(kind: RelationKind) -> bool {
    matches!(
        kind,
        RelationKind::Containment(
            super::ContainmentSubKind::UrlPath | super::ContainmentSubKind::Domain
        )
    )
}
