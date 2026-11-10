// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Where each statement handle is held: a runtime index behind the handle rule.
//!
//! Derived, never persisted. It may name holders that no longer hold an id,
//! since removals are not tracked; every read checks the live store, so only a
//! missing holder could mislead. Writes that can add a handle note it: through
//! [`Graph::get_edge_mut`], [`Graph::get_relation_mut`], the connect helpers
//! here, and the facet writers. Writes that replace a store wholesale clear the
//! index, and the next read rebuilds it.

use super::node_facets::SEMANTIC_PROPERTIES;
use super::{EdgeKey, EdgePayload, Graph, NodeKey, RelationKey, ResourceEdgeKey, SemanticStatement};
use crate::types::NodeProperty;
use petgraph::stable_graph::EdgeIndex;
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use std::collections::HashMap;
use std::sync::OnceLock;
use uuid::Uuid;

#[derive(Clone, Default)]
pub(crate) struct HandleIndex {
    built: OnceLock<Holders>,
    /// Relations handed out for writing since the last fold. Reads check them
    /// live; the next note folds them in, so this stays a few keys long.
    dirty: Vec<RelationKey>,
}

#[derive(Clone, Default)]
struct Holders {
    relations: HashMap<String, Vec<RelationKey>>,
    resource_literals: HashMap<String, Vec<Uuid>>,
    /// Retained Surface literals, by Surface node id.
    surface_literals: HashMap<String, Vec<Uuid>>,
}

fn add<T: PartialEq>(map: &mut HashMap<String, Vec<T>>, id: &str, holder: T) {
    let holders = map.entry(id.to_owned()).or_default();
    if !holders.contains(&holder) {
        holders.push(holder);
    }
}

/// Iteration order of [`Graph::find_semantic_statement`]'s scan: every Surface
/// edge, then every Resource relation, each by edge index.
fn scan_order(key: RelationKey) -> (u8, usize) {
    match key {
        RelationKey::Surface(key) => (0, key.index()),
        RelationKey::Resource(key) => (1, key.raw().index()),
    }
}

impl Holders {
    fn note_relation(&mut self, key: RelationKey, payload: &EdgePayload) {
        for statement in payload.assertions() {
            add(&mut self.relations, &statement.statement_id, key);
        }
    }
}

impl Graph {
    fn handle_holders(&self) -> &Holders {
        self.handle_index.built.get_or_init(|| {
            let mut holders = Holders::default();
            for edge in self.inner.inner().edge_references() {
                holders.note_relation(RelationKey::Surface(edge.id()), edge.weight());
            }
            for (key, ..) in self.resource_relations() {
                if let Some(payload) = self.get_resource_edge(key) {
                    holders.note_relation(RelationKey::Resource(key), payload);
                }
            }
            for resource in self.resource_nodes() {
                for held in self.resource_properties(resource.id()) {
                    add(&mut holders.resource_literals, &held.statement_id, resource.id());
                }
            }
            for (key, node) in self.nodes() {
                for held in self.legacy_node_properties(key).unwrap_or_default() {
                    add(&mut holders.surface_literals, &held.statement_id, node.id);
                }
            }
            holders
        })
    }

    /// Forget the index; the next read rebuilds it from the stores.
    pub(crate) fn clear_handle_index(&mut self) {
        self.handle_index = HandleIndex::default();
    }

    /// Fold relations written since the last note, then mark `key` as written.
    pub(crate) fn note_relation_handles(&mut self, key: RelationKey) {
        let mut index = std::mem::take(&mut self.handle_index);
        if let Some(holders) = index.built.get_mut() {
            for dirty in index.dirty.drain(..) {
                if let Some(payload) = self.get_relation(dirty) {
                    holders.note_relation(dirty, payload);
                }
            }
            index.dirty.push(key);
        }
        self.handle_index = index;
    }

    pub(crate) fn note_resource_literals(&mut self, id: Uuid) {
        if self.handle_index.built.get().is_none() {
            return;
        }
        let held = self.resource_properties(id);
        if let Some(holders) = self.handle_index.built.get_mut() {
            for property in held {
                add(&mut holders.resource_literals, &property.statement_id, id);
            }
        }
    }

    pub(crate) fn note_surface_literals(&mut self, node_id: Uuid) {
        if self.handle_index.built.get().is_none() {
            return;
        }
        let held: Vec<NodeProperty> = self
            .facets
            .get(&node_id, &chartulary::FacetId::new(SEMANTIC_PROPERTIES))
            .and_then(|value| serde::Deserialize::deserialize(value).ok())
            .unwrap_or_default();
        if let Some(holders) = self.handle_index.built.get_mut() {
            for property in held {
                add(&mut holders.surface_literals, &property.statement_id, node_id);
            }
        }
    }

    pub(crate) fn connect_surface(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        payload: EdgePayload,
    ) -> EdgeKey {
        let key = self.inner.connect(from, to, payload);
        self.note_relation_handles(RelationKey::Surface(key));
        key
    }

    pub(crate) fn connect_resource(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        payload: EdgePayload,
    ) -> EdgeIndex {
        let key = self.resources.connect(from, to, payload);
        self.note_relation_handles(RelationKey::Resource(ResourceEdgeKey::from_raw(key)));
        key
    }

    /// Mutable Resource relation payload, noted as written.
    pub(crate) fn resource_edge_mut(&mut self, key: EdgeIndex) -> Option<&mut EdgePayload> {
        self.note_relation_handles(RelationKey::Resource(ResourceEdgeKey::from_raw(key)));
        self.resources.edge_mut(key)
    }

    /// Locate any retained handle, including withdrawn history, independent of
    /// current nature. Use `find_held_semantic_statement` for default reads.
    pub fn find_semantic_statement(
        &self,
        statement_id: &str,
    ) -> Option<(RelationKey, &SemanticStatement)> {
        let indexed = self
            .handle_holders()
            .relations
            .get(statement_id)
            .into_iter()
            .flatten();
        indexed
            .chain(&self.handle_index.dirty)
            .filter_map(|key| {
                let statement = self
                    .get_relation(*key)?
                    .assertions()
                    .iter()
                    .find(|statement| statement.statement_id == statement_id)?;
                Some((*key, statement))
            })
            .min_by_key(|(key, _)| scan_order(*key))
    }

    /// Whether `property` may be held as a literal on `resource`: its id is no
    /// semantic statement, no other Resource or Surface literal, and on
    /// `resource` itself only a literal with the same content.
    pub(crate) fn literal_handle_available(&self, resource: Uuid, property: &NodeProperty) -> bool {
        let id = &property.statement_id;
        if self.find_semantic_statement(id).is_some() {
            return false;
        }
        let holders = self.handle_holders();
        for holder in holders.resource_literals.get(id).into_iter().flatten() {
            if self.resource(*holder).is_none() {
                continue;
            }
            if self.resource_properties(*holder).iter().any(|held| {
                held.statement_id == *id && (*holder != resource || !held.content_eq(property))
            }) {
                return false;
            }
        }
        !holders
            .surface_literals
            .get(id)
            .into_iter()
            .flatten()
            .filter_map(|node_id| self.get_node_key_by_id(*node_id))
            .any(|key| {
                self.legacy_node_properties(key)
                    .into_iter()
                    .flatten()
                    .any(|held| held.statement_id == *id)
            })
    }
}
