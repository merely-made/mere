// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Graph {
    pub fn find_held_semantic_statement(
        &self,
        statement_id: &str,
    ) -> Option<(RelationKey, &SemanticStatement)> {
        self.find_semantic_statement(statement_id)
            .filter(|(_, statement)| statement.is_held())
    }

    pub(super) fn held_statement_on_pair(
        &self,
        from: NodeKey,
        to: NodeKey,
        spec: &SemanticStatementSpec,
    ) -> Result<Option<RelationKey>, StatementWriteError> {
        let resource_pair = self.prospective_resource_pair(from, to);
        let mut matches = std::collections::BTreeMap::new();
        for (key, payload) in self
            .inner
            .inner()
            .edges_connecting(from, to)
            .map(|edge| (RelationKey::Surface(edge.id()), edge.weight()))
            .chain(
                resource_pair
                    .into_iter()
                    .flat_map(|(source, target)| self.resource_relations_on_pair(source, target))
                    .map(|(key, payload)| (RelationKey::Resource(key), payload)),
            )
        {
            for statement in payload
                .assertions()
                .iter()
                .filter(|statement| matches_spec(statement, spec))
            {
                matches.insert(statement.statement_id.clone(), key);
            }
        }
        if matches.len() > 1 {
            return Err(StatementWriteError::AmbiguousAssertion {
                predicate: spec.predicate.clone(),
                statement_ids: matches.into_keys().collect(),
            });
        }
        Ok(matches.into_values().next())
    }

    /// Resource relations from `from` to `to`, in `resource_relations` order
    /// (by edge index), without walking the others.
    pub(super) fn resource_relations_on_pair(
        &self,
        from: Uuid,
        to: Uuid,
    ) -> Vec<(ResourceEdgeKey, &EdgePayload)> {
        let (Some(source), Some(target)) = (self.resources.key_of(&from), self.resources.key_of(&to))
        else {
            return Vec::new();
        };
        let mut edges: Vec<_> = self
            .resources
            .inner()
            .edges_connecting(source, target)
            .map(|edge| (ResourceEdgeKey::from_raw(edge.id()), edge.weight()))
            .collect();
        edges.sort_by_key(|(key, _)| key.raw());
        edges
    }

    /// Surface edges whose endpoints show `from` and `to`, by edge index, read
    /// through the surfaces showing each Resource instead of every edge.
    pub(super) fn surface_edges_showing_pair(
        &self,
        from: Uuid,
        to: Uuid,
    ) -> Vec<(crate::graph::EdgeKey, &EdgePayload)> {
        let showing = |resource: Uuid| -> Vec<NodeKey> {
            self.resource_surfaces
                .get(&resource)
                .into_iter()
                .flatten()
                .filter_map(|surface| self.inner.key_of(surface))
                .collect()
        };
        let (sources, targets) = (showing(from), showing(to));
        let graph = self.inner.inner();
        let mut edges: Vec<_> = sources
            .iter()
            .flat_map(|source| {
                targets
                    .iter()
                    .flat_map(move |target| graph.edges_connecting(*source, *target))
            })
            .map(|edge| (edge.id(), edge.weight()))
            .collect();
        edges.sort_by_key(|(key, _)| *key);
        edges.dedup_by_key(|(key, _)| *key);
        edges
    }
}
