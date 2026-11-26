// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The resource half of the statement-handle index (engine split D243): where
//! each handle is held on a resource edge or as a resource literal. Built on
//! first read from the store and kept by its writes; a host keeps its own
//! surface half and asks both.

use std::sync::{Arc, OnceLock};

use petgraph::graph::EdgeIndex;
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use uuid::Uuid;

use super::resource_graph::properties_in;
use super::{NodeProperty, ResourceStore, SemanticStatement, Shared};

type Holding<T> = rpds::HashTrieMapSync<Arc<str>, Shared<Vec<T>>>;

/// The index: built lazily, plus the edges handed out for writing since the
/// last fold, which reads check live.
#[derive(Clone, Debug, Default)]
pub(super) struct ResourceHandles {
    built: OnceLock<Held>,
    dirty: Vec<EdgeIndex>,
}

#[derive(Clone, Debug, Default)]
struct Held {
    relations: Holding<EdgeIndex>,
    literals: Holding<Uuid>,
}

fn add<T: PartialEq + Clone>(map: &mut Holding<T>, id: &str, holder: T) {
    match map.get(id) {
        // Already held: leave the map shared.
        Some(holders) if holders.contains(&holder) => {},
        Some(_) => {
            if let Some(holders) = map.get_mut(id) {
                holders.push(holder);
            }
        },
        None => map.insert_mut(Arc::from(id), Shared::new(vec![holder])),
    }
}

impl ResourceStore {
    fn held(&self) -> &Held {
        self.handles.built.get_or_init(|| {
            let mut held = Held::default();
            for edge in self.inner().edge_references() {
                for statement in edge.weight().assertions() {
                    add(&mut held.relations, &statement.statement_id, edge.id());
                }
            }
            for (resource, facets) in self.resources_with_facets() {
                for property in properties_in(facets) {
                    add(&mut held.literals, &property.statement_id, resource.id());
                }
            }
            held
        })
    }

    /// Whether the index has been built, so writes keep it.
    pub fn handle_index_built(&self) -> bool {
        self.handles.built.get().is_some()
    }

    /// Forget the index; the next read rebuilds it from the store.
    pub fn clear_handle_index(&mut self) {
        self.handles = ResourceHandles::default();
    }

    /// Fold the edges written since the last note, then mark `key` written.
    pub fn note_edge(&mut self, key: EdgeIndex) {
        let mut handles = std::mem::take(&mut self.handles);
        if let Some(held) = handles.built.get_mut() {
            for dirty in handles.dirty.drain(..) {
                if let Some(payload) = self.inner().edge_weight(dirty) {
                    for statement in payload.assertions() {
                        add(&mut held.relations, &statement.statement_id, dirty);
                    }
                }
            }
            handles.dirty.push(key);
        }
        self.handles = handles;
    }

    /// Note the literals resource `id` now holds.
    pub(super) fn note_literals(&mut self, id: Uuid) {
        if self.handles.built.get().is_none() {
            return;
        }
        let held = self.resource_properties(id);
        if let Some(index) = self.handles.built.get_mut() {
            for property in held {
                add(&mut index.literals, &property.statement_id, id);
            }
        }
    }

    fn holds(&self, key: EdgeIndex, statement_id: &str) -> Option<&SemanticStatement> {
        self.inner()
            .edge_weight(key)?
            .assertions()
            .iter()
            .find(|statement| statement.statement_id == statement_id)
    }

    fn candidates(&self, statement_id: &str) -> impl Iterator<Item = EdgeIndex> + '_ {
        self.held()
            .relations
            .get(statement_id)
            .into_iter()
            .flatten()
            .chain(&self.handles.dirty)
            .copied()
    }

    /// Every resource edge retaining `statement_id`, in edge order.
    pub fn statement_owner_edges(&self, statement_id: &str) -> Vec<EdgeIndex> {
        let mut keys: Vec<_> = self
            .candidates(statement_id)
            .filter(|key| self.holds(*key, statement_id).is_some())
            .collect();
        keys.sort_by_key(|key| key.index());
        keys.dedup();
        keys
    }

    /// The first resource edge, in edge order, retaining `statement_id`.
    pub fn find_statement(&self, statement_id: &str) -> Option<(EdgeIndex, &SemanticStatement)> {
        self.candidates(statement_id)
            .filter_map(|key| Some((key, self.holds(key, statement_id)?)))
            .min_by_key(|(key, _)| key.index())
    }

    /// Whether a resource edge `keep` accepts retains `statement_id`.
    pub fn edge_holds(&self, statement_id: &str, keep: impl Fn(EdgeIndex) -> bool) -> bool {
        self.candidates(statement_id)
            .any(|key| keep(key) && self.holds(key, statement_id).is_some())
    }

    /// Whether a held resource other than `except` holds `statement_id` as a literal.
    pub fn literal_holds(&self, statement_id: &str, except: Option<Uuid>) -> bool {
        self.held()
            .literals
            .get(statement_id)
            .into_iter()
            .flatten()
            .filter(|holder| Some(**holder) != except && self.resource(**holder).is_some())
            .any(|holder| {
                self.resource_properties(*holder)
                    .iter()
                    .any(|held| held.statement_id == statement_id)
            })
    }

    /// Whether `property`'s handle is held as a literal anywhere but as this
    /// same property on `resource`.
    pub fn literal_taken(&self, resource: Uuid, property: &NodeProperty) -> bool {
        let id = property.statement_id.as_str();
        self.held()
            .literals
            .get(id)
            .into_iter()
            .flatten()
            .filter(|holder| self.resource(**holder).is_some())
            .any(|holder| {
                self.resource_properties(*holder).iter().any(|held| {
                    held.statement_id == id && (*holder != resource || !held.content_eq(property))
                })
            })
    }
}
