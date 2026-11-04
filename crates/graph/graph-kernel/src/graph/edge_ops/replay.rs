// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Graph {
    /// Replay helper: add node only if UUID is not already present.
    pub(crate) fn replay_add_node_with_id_if_missing(
        &mut self,
        id: Uuid,
        url: String,
        position: Point2D<f32>,
    ) -> Option<NodeKey> {
        let (key, inserted) = self.insert_node_if_missing(id, url, position);
        inserted.then_some(key)
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
        if selector != RelationSelector::Family(super::super::EdgeFamily::Traversal) {
            return self.replay_withdraw_selected(from_key, to_key, selector);
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
        trigger: super::super::edge_taxonomy::NavigationTrigger,
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

    /// Historical replay path. Missing source time stays absent; replay never
    /// stamps a wall clock into a prior journal prefix.
    pub(crate) fn replay_surface_relation_as(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        assertion: EdgeAssertion,
        asserter_iri: String,
    ) -> Option<RelationKey> {
        if let EdgeAssertion::Semantic { sub_kind, label } = assertion {
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
                        predicate: super::super::predicate_iri(sub_kind).into(),
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

    /// Historical selector replay retains an undated tombstone instead of
    /// stamping replay time. Exact journal pair snapshots remain the primary
    /// replay form; this covers legacy selector deltas only.
    pub(crate) fn replay_retract_relations(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        selector: RelationSelector,
    ) -> usize {
        if selector != RelationSelector::Family(super::super::EdgeFamily::Traversal) {
            return self.replay_withdraw_selected(from, to, selector);
        }
        self.retract_relations(from, to, selector)
    }

    /// Replay withdrawal of every held statement `selector` names between the
    /// pair: undated, and never refused.
    pub(super) fn replay_withdraw_selected(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        selector: RelationSelector,
    ) -> usize {
        self.withdraw_selected_semantic_assertions(from, to, selector, false, &mut |_| true)
            .withdrawn
    }
}
