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
//!
//! This is the surface half. The resource half, resource edges and resource
//! literals, is the engine store's own (engine split D243); every query here
//! asks both, surface holders first.
//!
//! Admission reads it too: a write is checked against the holders of its own
//! handles, so replaying a journal costs each entry its handles, not the graph.

use super::node_facets::SEMANTIC_PROPERTIES;
use super::resource_content::{LiteralWrite, PendingWrite};
use super::{
    EdgeKey, EdgePayload, Graph, NodeKey, RelationKey, ResourceEdgeKey, SemanticStatement,
};
use crate::types::NodeProperty;
use petgraph::stable_graph::EdgeIndex;
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;
use uuid::Uuid;

#[derive(Clone, Default)]
pub(crate) struct HandleIndex {
    built: OnceLock<Holders>,
    /// Surface edges handed out for writing since the last fold. Reads check
    /// them live; the next note folds them in, so this stays a few keys long.
    dirty: Vec<EdgeKey>,
}

#[derive(Clone, Default)]
struct Holders {
    relations: Holding<EdgeKey>,
    /// Retained Surface literals, by Surface node id.
    surface_literals: Holding<Uuid>,
}

type Holding<T> = rpds::HashTrieMapSync<std::sync::Arc<str>, mere::model::Shared<Vec<T>>>;

fn add<T: PartialEq + Clone>(map: &mut Holding<T>, id: &str, holder: T) {
    match map.get(id) {
        // Already held: leave the map shared.
        Some(holders) if holders.contains(&holder) => {},
        Some(_) => {
            if let Some(holders) = map.get_mut(id) {
                holders.push(holder);
            }
        },
        None => map.insert_mut(
            std::sync::Arc::from(id),
            mere::model::Shared::new(vec![holder]),
        ),
    }
}

impl Holders {
    fn note_relation(&mut self, key: EdgeKey, payload: &EdgePayload) {
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
                holders.note_relation(edge.id(), edge.weight());
            }
            use mere::model::resource_graph::facets_of_each;
            let nodes: Vec<Uuid> = self.nodes().map(|(_, node)| node.id).collect();
            let semantic = portolan::FacetId::new(SEMANTIC_PROPERTIES);
            for (id, facets) in nodes.iter().zip(facets_of_each(&self.facets, &nodes)) {
                let held: Vec<NodeProperty> = facets
                    .and_then(|facets| facets.get(&semantic))
                    .and_then(|value| serde::Deserialize::deserialize(value).ok())
                    .unwrap_or_default();
                for held in held {
                    add(&mut holders.surface_literals, &held.statement_id, *id);
                }
            }
            holders
        })
    }

    /// Forget both halves of the index; the next reads rebuild them.
    pub(crate) fn clear_handle_index(&mut self) {
        self.handle_index = HandleIndex::default();
        self.resources.clear_handle_index();
    }

    /// Fold relations written since the last note, then mark `key` as written.
    /// A resource edge is the store's to note.
    pub(crate) fn note_relation_handles(&mut self, key: RelationKey) {
        let key = match key {
            RelationKey::Surface(key) => key,
            RelationKey::Resource(key) => return self.resources.note_edge(key.raw()),
        };
        let mut index = std::mem::take(&mut self.handle_index);
        if let Some(holders) = index.built.get_mut() {
            for dirty in index.dirty.drain(..) {
                if let Some(payload) = self.get_edge(dirty) {
                    holders.note_relation(dirty, payload);
                }
            }
            index.dirty.push(key);
        }
        self.handle_index = index;
    }

    pub(crate) fn note_surface_literals(&mut self, node_id: Uuid) {
        if self.handle_index.built.get().is_none() {
            return;
        }
        let held: Vec<NodeProperty> = self
            .facets
            .get(&node_id, &portolan::FacetId::new(SEMANTIC_PROPERTIES))
            .and_then(|value| serde::Deserialize::deserialize(value).ok())
            .unwrap_or_default();
        if let Some(holders) = self.handle_index.built.get_mut() {
            for property in held {
                add(
                    &mut holders.surface_literals,
                    &property.statement_id,
                    node_id,
                );
            }
        }
    }

    pub(crate) fn connect_surface(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        payload: EdgePayload,
    ) -> EdgeKey {
        let key = self.inner.connect(from, to, payload.into());
        self.note_relation_handles(RelationKey::Surface(key));
        key
    }

    /// A new resource edge; the store notes its handles.
    pub(crate) fn connect_resource(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        payload: EdgePayload,
    ) -> EdgeIndex {
        self.resources.connect(from, to, payload)
    }

    /// Mutable Resource relation payload, noted as written by the store.
    pub(crate) fn resource_edge_mut(&mut self, key: EdgeIndex) -> Option<&mut EdgePayload> {
        self.resources.edge_mut(key)
    }

    fn surface_candidates(&self, statement_id: &str) -> impl Iterator<Item = EdgeKey> + '_ {
        self.handle_holders()
            .relations
            .get(statement_id)
            .into_iter()
            .flatten()
            .chain(&self.handle_index.dirty)
            .copied()
    }

    fn surface_statement(&self, key: EdgeKey, statement_id: &str) -> Option<&SemanticStatement> {
        self.get_edge(key)?
            .assertions()
            .iter()
            .find(|statement| statement.statement_id == statement_id)
    }

    /// Every relation that retains `statement_id`, in scan order (Surface
    /// edges, then Resource edges). Looked up through the handle index, not by
    /// walking every edge; duplicates across buckets are all returned.
    pub(super) fn semantic_statement_owner_keys(&self, statement_id: &str) -> Vec<RelationKey> {
        let mut surface: Vec<EdgeKey> = self
            .surface_candidates(statement_id)
            .filter(|key| self.surface_statement(*key, statement_id).is_some())
            .collect();
        surface.sort_by_key(|key| key.index());
        surface.dedup();
        surface
            .into_iter()
            .map(RelationKey::Surface)
            .chain(
                self.resources
                    .statement_owner_edges(statement_id)
                    .into_iter()
                    .map(|key| RelationKey::Resource(ResourceEdgeKey::from_raw(key))),
            )
            .collect()
    }

    pub fn find_semantic_statement(
        &self,
        statement_id: &str,
    ) -> Option<(RelationKey, &SemanticStatement)> {
        self.surface_candidates(statement_id)
            .filter_map(|key| Some((key, self.surface_statement(key, statement_id)?)))
            .min_by_key(|(key, _)| key.index())
            .map(|(key, statement)| (RelationKey::Surface(key), statement))
            .or_else(|| {
                self.resources
                    .find_statement(statement_id)
                    .map(|(key, statement)| {
                        (
                            RelationKey::Resource(ResourceEdgeKey::from_raw(key)),
                            statement,
                        )
                    })
            })
    }

    pub(crate) fn literal_handle_available(&self, resource: Uuid, property: &NodeProperty) -> bool {
        let id = &property.statement_id;
        if self.find_semantic_statement(id).is_some() {
            return false;
        }
        if self.resources.literal_taken(resource, property) {
            return false;
        }
        !self.surface_literal_holds(id)
    }

    fn relation_holds(&self, statement_id: &str, keep: impl Fn(RelationKey) -> bool) -> bool {
        self.surface_candidates(statement_id).any(|key| {
            keep(RelationKey::Surface(key)) && self.surface_statement(key, statement_id).is_some()
        }) || self.resources.edge_holds(statement_id, |key| {
            keep(RelationKey::Resource(ResourceEdgeKey::from_raw(key)))
        })
    }

    pub(super) fn handle_held_off_resource_pair(
        &self,
        statement_id: &str,
        source: NodeKey,
        target: NodeKey,
    ) -> bool {
        self.relation_holds(statement_id, |key| match key {
            RelationKey::Surface(_) => true,
            RelationKey::Resource(key) => {
                self.resources.inner().edge_endpoints(key.raw()) != Some((source, target))
            },
        })
    }

    fn resource_literal_holds(&self, statement_id: &str, except: Option<Uuid>) -> bool {
        self.resources.literal_holds(statement_id, except)
    }

    fn surface_literal_holds(&self, statement_id: &str) -> bool {
        let holders = self.handle_holders().surface_literals.get(statement_id);
        holders
            .into_iter()
            .flatten()
            .filter_map(|node_id| self.get_node_key_by_id(*node_id))
            .any(|key| {
                self.legacy_node_properties(key)
                    .into_iter()
                    .flatten()
                    .any(|held| held.statement_id == statement_id)
            })
    }

    pub(super) fn write_collides(&self, write: PendingWrite<'_>) -> bool {
        match write {
            PendingWrite::Literals(LiteralWrite::Resource(owner, incoming)) => {
                let mut claimed = HashMap::<&str, &NodeProperty>::new();
                incoming.iter().any(|property| {
                    let id = property.statement_id.as_str();
                    claimed
                        .insert(id, property)
                        .is_some_and(|first| first != property)
                        || self.relation_holds(id, |_| true)
                        || self.resource_literal_holds(id, Some(owner))
                        || self.surface_literal_holds(id)
                })
            },
            PendingWrite::Literals(LiteralWrite::Surface(_, incoming)) => {
                incoming.iter().any(|property| {
                    let id = property.statement_id.as_str();
                    self.relation_holds(id, |key| matches!(key, RelationKey::Resource(_)))
                        || self.resource_literal_holds(id, None)
                })
            },
            PendingWrite::ResourceEdges(from, to, rows) => {
                let Some((source, target)) =
                    self.resources.key_of(&from).zip(self.resources.key_of(&to))
                else {
                    return true;
                };
                let mut claimed = HashSet::new();
                rows.iter().any(|id| {
                    !claimed.insert(id.as_str())
                        || self.handle_held_off_resource_pair(id, source, target)
                        || self.resource_literal_holds(id, None)
                        || self.surface_literal_holds(id)
                })
            },
            PendingWrite::SurfaceEdges(from, to, rows) => {
                let Some(pair) = self
                    .get_node_key_by_id(from)
                    .zip(self.get_node_key_by_id(to))
                else {
                    return true;
                };
                let mut claimed = HashSet::new();
                rows.iter().any(|id| {
                    !claimed.insert(id.as_str())
                        || self.relation_holds(id, |key| match key {
                            RelationKey::Resource(_) => true,
                            RelationKey::Surface(key) => {
                                self.inner.inner().edge_endpoints(key) != Some(pair)
                            },
                        })
                        || self.resource_literal_holds(id, None)
                })
            },
        }
    }
}
