// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Graph {
    /// Locate any retained handle, including withdrawn history, independent of
    /// current nature. Use `find_held_semantic_statement` for default reads.
    pub fn find_semantic_statement(
        &self,
        statement_id: &str,
    ) -> Option<(RelationKey, &SemanticStatement)> {
        self.inner
            .inner()
            .edge_references()
            .map(|edge| (RelationKey::Surface(edge.id()), edge.weight()))
            .chain(
                self.resource_relations()
                    .map(|(key, _, _, payload)| (RelationKey::Resource(key), payload)),
            )
            .find_map(|(key, payload)| {
                payload
                    .assertions()
                    .iter()
                    .find(|statement| statement.statement_id == statement_id)
                    .map(|statement| (key, statement))
            })
    }

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
                self.resource_relations()
                    .filter(move |(_, source, target, _)| resource_pair == Some((*source, *target)))
                    .map(|(key, _, _, payload)| (RelationKey::Resource(key), payload)),
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
}
