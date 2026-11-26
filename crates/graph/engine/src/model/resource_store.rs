// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The engine's recorded resource store (engine split D243, D250).
//!
//! It holds the resource graph and makes every write to it. A write checks
//! what the store can check alone, applies, and returns the [`ResourceDelta`]
//! it made: the resource's or the pair's state afterwards, in the form the
//! journal already stores. An unchanged or refused write returns `None`. The
//! store holds no recorder: its host records the deltas in its one journal,
//! beside its surface deltas, and refuses what only it can judge (a binding
//! that still shows a resource, a handle held elsewhere) before writing.
//!
//! [`connect`](ResourceStore::connect), [`edge_mut`](ResourceStore::edge_mut)
//! and [`disconnect`](ResourceStore::disconnect) are the parts of a host's
//! composite assertion edit; the host records [`pair_delta`](ResourceStore::pair_delta)
//! for each pair it touched once the edit is done.

use std::collections::BTreeMap;
use std::ops::Deref;

use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use uuid::Uuid;

use super::edge_encode::persisted_edge_for_ids;
use super::{
    EdgePayload, PersistedEdge, PersistedResourceRecord, ResourceDelta, ResourceGraph, ResourceNode,
};

/// The resource graph behind its write path. Reads go through [`Deref`].
#[derive(Clone, Debug, Default)]
pub struct ResourceStore {
    graph: ResourceGraph,
    /// The resource half of the statement-handle index.
    pub(super) handles: super::resource_handles::ResourceHandles,
}

impl Deref for ResourceStore {
    type Target = ResourceGraph;

    fn deref(&self) -> &ResourceGraph {
        &self.graph
    }
}

impl ResourceStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Write `record` as the resource `id`. `None` when the record breaks a
    /// rule (its identity is not `id`, a frozen selection or link differs
    /// from the one held, a declaration or content facet is malformed, a
    /// facet repeats or is not JSON) or the store already holds it.
    pub fn put_record(
        &mut self,
        id: Uuid,
        record: PersistedResourceRecord,
    ) -> Option<ResourceDelta> {
        let mut resource = ResourceNode::from_canonical_iri(&record.canonical_iri);
        let Ok(selection) = super::frozen_rules::selection_from_record(&record) else {
            return None;
        };
        let Ok(link) = super::frozen_rules::validate_frozen_link_record(&record) else {
            return None;
        };
        if let Some(existing) = self
            .graph
            .resource(id)
            .and_then(|resource| resource.nested_selection())
            && selection.as_ref() != Some(existing)
        {
            return None;
        }
        if self
            .graph
            .resource(id)
            .is_some_and(|resource| resource.nested_selection().is_some())
        {
            let stored = self
                .graph
                .facets()
                .get(&id, &portolan::FacetId::new(super::FROZEN_SELECTION));
            let incoming = record
                .facets
                .iter()
                .find(|facet| facet.facet == super::FROZEN_SELECTION)
                .and_then(|facet| {
                    serde_json::from_str::<serde_json::Value>(&facet.value_json).ok()
                });
            if stored != incoming.as_ref() {
                return None;
            }
        }
        if let Some(existing) = self
            .graph
            .resource(id)
            .and_then(|resource| resource.nested_link())
            && link.as_ref() != Some(existing)
        {
            return None;
        }
        if self
            .graph
            .resource(id)
            .is_some_and(|resource| resource.nested_link().is_some())
        {
            let stored = self
                .graph
                .facets()
                .get(&id, &portolan::FacetId::new(super::FROZEN_LINK));
            let incoming = record
                .facets
                .iter()
                .find(|facet| facet.facet == super::FROZEN_LINK)
                .and_then(|facet| {
                    serde_json::from_str::<serde_json::Value>(&facet.value_json).ok()
                });
            if stored != incoming.as_ref() {
                return None;
            }
        }
        resource = resource.with_frozen(selection, link);
        if resource.id() != id {
            return None;
        }
        if super::predicate_declarations::predicate_declarations_from_record(&record).is_err() {
            return None;
        }
        if super::resource_rules::validate_content_record(&record).is_err() {
            return None;
        }
        let mut facets = std::collections::BTreeMap::new();
        for facet in record.facets {
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&facet.value_json) else {
                return None;
            };
            if facets
                .insert(portolan::FacetId::new(facet.facet), value)
                .is_some()
            {
                return None;
            }
        }
        self.write(resource, facets)
    }

    fn write(
        &mut self,
        resource: ResourceNode,
        facets: BTreeMap<portolan::FacetId, serde_json::Value>,
    ) -> Option<ResourceDelta> {
        let id = resource.id();
        let held: BTreeMap<_, _> = self
            .graph
            .facets()
            .facets_of(&id)
            .into_iter()
            .flat_map(|values| values.iter())
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        if self.graph.resource(id) == Some(&resource) && held == facets {
            return None;
        }
        self.graph.insert(resource.into());
        self.write_facets(id, facets);
        self.note_literals(id);
        Some(self.record_delta(id))
    }

    /// Remove a resource that no edge touches. `None` when it is absent or an
    /// edge still touches it.
    pub fn remove_record(&mut self, id: Uuid) -> Option<ResourceDelta> {
        let key = self.graph.key_of(&id)?;
        if self
            .graph
            .inner()
            .edge_references()
            .any(|edge| edge.source() == key || edge.target() == key)
        {
            return None;
        }
        self.graph.remove(key);
        self.graph.facets_mut().remove_node(&id);
        Some(self.record_delta(id))
    }

    /// Replace every edge from `from` to `to` with `payloads`, in order.
    /// `None` when an endpoint is absent or the pair already holds them.
    pub fn replace_edges(
        &mut self,
        from: Uuid,
        to: Uuid,
        payloads: Vec<EdgePayload>,
    ) -> Option<ResourceDelta> {
        let (source, target) = (self.graph.key_of(&from)?, self.graph.key_of(&to)?);
        let current: Vec<_> = self.pair_keys(source, target);
        if current.len() == payloads.len()
            && current.iter().zip(&payloads).all(|(key, new)| {
                self.graph
                    .inner()
                    .edge_weight(*key)
                    .is_some_and(|old| **old == *new)
            })
        {
            return None;
        }
        for key in current {
            self.graph.disconnect(key);
        }
        // Connected last to first, so the pair reads back in `payloads` order.
        for payload in payloads.into_iter().rev() {
            let key = self.graph.connect(source, target, payload.into());
            self.note_edge(key);
        }
        Some(self.pair_delta(from, to))
    }

    /// One part of a composite edit: a new edge between held resources.
    pub fn connect(&mut self, from: NodeIndex, to: NodeIndex, payload: EdgePayload) -> EdgeIndex {
        let key = self.graph.connect(from, to, payload.into());
        self.note_edge(key);
        key
    }

    /// One part of a composite edit: an edge's payload, in place.
    pub fn edge_mut(&mut self, key: EdgeIndex) -> Option<&mut EdgePayload> {
        self.note_edge(key);
        self.graph.edge_mut(key).map(|edge| &mut **edge)
    }

    /// One part of a composite edit: an edge removed.
    pub fn disconnect(&mut self, key: EdgeIndex) {
        self.graph.disconnect(key);
    }

    /// The delta that sets `id` to what the store holds now.
    pub fn record_delta(&self, id: Uuid) -> ResourceDelta {
        ResourceDelta::ReplaySetResourceRecordById {
            resource_id: id.to_string(),
            record: self.graph.resource_record(id),
        }
    }

    /// The delta that sets the pair `from`, `to` to what the store holds now.
    pub fn pair_delta(&self, from: Uuid, to: Uuid) -> ResourceDelta {
        ResourceDelta::ReplaySetResourceEdgesByIds {
            from_resource_id: from.to_string(),
            to_resource_id: to.to_string(),
            edges: self.persisted_edges_between(from, to),
        }
    }

    /// The pair's edges in persisted form, in the order they read back.
    pub fn persisted_edges_between(&self, from: Uuid, to: Uuid) -> Vec<PersistedEdge> {
        let (Some(source), Some(target)) = (self.graph.key_of(&from), self.graph.key_of(&to))
        else {
            return Vec::new();
        };
        self.graph
            .inner()
            .edges_connecting(source, target)
            .map(|edge| persisted_edge_for_ids(from, to, edge.weight()))
            .collect()
    }

    fn pair_keys(&self, source: NodeIndex, target: NodeIndex) -> Vec<EdgeIndex> {
        self.graph
            .inner()
            .edges_connecting(source, target)
            .map(|edge| edge.id())
            .collect()
    }

    /// Replace a resource's facets with `facets`: drop the ones it no longer
    /// holds and write the rest in place. Taking the node out and putting it
    /// back would rebalance the persistent store twice per record (engine
    /// split D168); the store ends the same either way.
    fn write_facets(&mut self, id: Uuid, facets: BTreeMap<portolan::FacetId, serde_json::Value>) {
        let dropped: Vec<portolan::FacetId> = self
            .graph
            .facets()
            .facets_of(&id)
            .into_iter()
            .flat_map(|held| held.iter().map(|(facet, _)| facet))
            .filter(|facet| !facets.contains_key(*facet))
            .cloned()
            .collect();
        let store = self.graph.facets_mut();
        if facets.is_empty() {
            // An emptied resource holds no facet node at all, as before.
            store.remove_node(&id);
        }
        for facet in &dropped {
            store.remove(&id, facet);
        }
        for (facet, value) in facets {
            store
                .set(id, facet, value, &portolan::AcceptAll)
                .expect("permissive facet validator");
        }
    }
}

#[cfg(test)]
#[path = "resource_store_tests.rs"]
mod tests;
