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

/// What a selector retraction did to the statements it found: those it
/// withdrew, and those the caller's predicate refused, which stay held.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Retraction {
    pub withdrawn: usize,
    pub refused: usize,
}

impl Graph {
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
        self.retract_relations_where(from, to, selector, |_| true)
            .withdrawn
    }

    /// [`retract_relations`](Self::retract_relations) that asks `allow` about
    /// each statement before withdrawing it, across both strata of the pair.
    /// A refused statement stays held and is counted in [`Retraction::refused`].
    /// Traversal history holds no statements, so `allow` is not consulted for
    /// that family.
    pub fn retract_relations_where(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        selector: RelationSelector,
        mut allow: impl FnMut(&SemanticStatement) -> bool,
    ) -> Retraction {
        if selector == RelationSelector::Family(super::EdgeFamily::Traversal) {
            return Retraction {
                withdrawn: self.retract_traversals(from, to, selector),
                refused: 0,
            };
        }
        self.withdraw_selected_semantic_assertions(from, to, selector, true, &mut allow)
    }

    fn retract_traversals(
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

    fn withdraw_selected_semantic_assertions(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        selector: RelationSelector,
        authored: bool,
        allow: &mut dyn FnMut(&SemanticStatement) -> bool,
    ) -> Retraction {
        let matches_selector = |statement: &SemanticStatement| {
            super::edge_payload::selector_matches_kind(selector, statement.relation_kind())
        };
        let mut found = self
            .inner
            .inner()
            .edges_connecting(from, to)
            .flat_map(|edge| edge.weight().held_assertions())
            .filter(|statement| matches_selector(statement))
            .cloned()
            .collect::<Vec<_>>();
        if authored
            && let Some((source, target)) =
                self.shown_resource_id(from).zip(self.shown_resource_id(to))
        {
            found.extend(
                self.resource_relations()
                    .filter(|(_, a, b, _)| (*a, *b) == (source, target))
                    .flat_map(|(_, _, _, payload)| payload.held_assertions())
                    .filter(|statement| matches_selector(statement))
                    .cloned(),
            );
        }
        found.sort_by(|a, b| a.statement_id.cmp(&b.statement_id));
        found.dedup_by(|a, b| a.statement_id == b.statement_id);
        let (allowed, refused): (Vec<_>, Vec<_>) =
            found.into_iter().partition(|statement| allow(statement));
        let withdrawn = allowed
            .iter()
            .filter(|statement| {
                let id = &statement.statement_id;
                if authored {
                    self.withdraw_semantic_assertion(id, None, None)
                } else {
                    let author = self.write_author().asserter_iri();
                    self.apply_imported_semantic_withdrawal(id, None, Some(author))
                }
            })
            .count();
        Retraction {
            withdrawn,
            refused: refused.len(),
        }
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
#[path = "edge_ops/legacy_predicate_revision_tests.rs"]
mod legacy_predicate_revision_tests;

fn is_derived_containment(kind: RelationKind) -> bool {
    matches!(
        kind,
        RelationKind::Containment(
            super::ContainmentSubKind::UrlPath | super::ContainmentSubKind::Domain
        )
    )
}

#[path = "edge_ops/replay.rs"]
mod replay;

#[path = "edge_ops/admission.rs"]
mod admission;

#[path = "edge_ops/lookup.rs"]
mod lookup;
