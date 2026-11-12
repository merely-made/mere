// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The resource graph: resources by canonical IRI, the assertion-bearing edges
//! between them, and their facet store (engine split P5 L3).
//!
//! It is a [`chartulary::Graph`] of [`ResourceNode`]s and [`EdgePayload`]s (it
//! dereferences to one) plus the facets that hold resource content. A host that
//! journals its writes, such as `mere-kernel`, owns one privately and never
//! hands out `&mut`, so its recorded write path stays the only way in.

use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};

use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use serde::de::DeserializeOwned;
use uuid::Uuid;

use super::{
    ClassificationVariant, EdgePayload, NodeProperty, PersistedResourceFacet,
    PersistedResourceRecord, RESOURCE_CLASSIFICATIONS, RESOURCE_PROPERTIES, ResourceNode,
    TAG_CONCEPT, TAGGED_WITH_IRI, TagConcept,
};

/// A version drawn from one process-wide counter, so two graphs (or a graph
/// and its edited clone) never share one after a write.
fn fresh_version() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

/// Resources, the edges between them, and their facets.
#[derive(Clone)]
pub struct ResourceGraph {
    graph: chartulary::Graph<ResourceNode, EdgePayload>,
    facets: chartulary::FacetStore<Uuid>,
    /// Advanced by every mutable access; not persisted.
    version: u64,
}

impl std::fmt::Debug for ResourceGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceGraph")
            .field("resources", &self.graph.node_count())
            .field("edges", &self.graph.edge_count())
            .field("version", &self.version)
            .finish()
    }
}

impl Default for ResourceGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for ResourceGraph {
    type Target = chartulary::Graph<ResourceNode, EdgePayload>;

    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}

impl DerefMut for ResourceGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.version = fresh_version();
        &mut self.graph
    }
}

impl ResourceGraph {
    pub fn new() -> Self {
        Self {
            graph: chartulary::Graph::new(),
            facets: chartulary::FacetStore::new(),
            version: fresh_version(),
        }
    }

    /// Changes whenever the graph or its facets may have changed.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Resource metadata, separate from surface metadata.
    pub fn facets(&self) -> &chartulary::FacetStore<Uuid> {
        &self.facets
    }

    pub fn facets_mut(&mut self) -> &mut chartulary::FacetStore<Uuid> {
        self.version = fresh_version();
        &mut self.facets
    }

    /// Read a resource by its stable identity.
    pub fn resource(&self, id: Uuid) -> Option<&ResourceNode> {
        self.graph.get(&id)
    }

    /// The resources currently held.
    pub fn resource_nodes(&self) -> impl Iterator<Item = &ResourceNode> {
        self.graph.nodes().map(|(_, resource)| resource)
    }

    /// Resource edges with their resource endpoints.
    pub fn resource_edges(
        &self,
    ) -> impl Iterator<Item = (&ResourceNode, &ResourceNode, &EdgePayload)> {
        self.graph.inner().edge_references().map(|edge| {
            (
                self.graph
                    .node(edge.source())
                    .expect("resource endpoint exists"),
                self.graph
                    .node(edge.target())
                    .expect("resource endpoint exists"),
                edge.weight(),
            )
        })
    }

    /// A resource's identity and facets in their persisted form.
    pub fn resource_record(&self, id: Uuid) -> Option<PersistedResourceRecord> {
        let resource = self.resource(id)?;
        let facets = self
            .facets
            .facets_of(&id)
            .into_iter()
            .flat_map(|facets| facets.iter())
            .map(|(facet, value)| PersistedResourceFacet {
                facet: facet.as_str().to_string(),
                value_json: serde_json::to_string(value).expect("JSON value serializes"),
            })
            .collect();
        Some(PersistedResourceRecord {
            canonical_iri: resource.canonical_iri().to_string(),
            facets,
        })
    }

    /// One typed facet of a resource, when present and well formed.
    pub fn resource_content<T: DeserializeOwned>(&self, id: Uuid, facet: &str) -> Option<T> {
        let value = self.facets.get(&id, &chartulary::FacetId::new(facet))?;
        T::deserialize(value).ok()
    }

    pub fn resource_properties(&self, id: Uuid) -> Vec<NodeProperty> {
        self.resource_content(id, RESOURCE_PROPERTIES)
            .unwrap_or_default()
    }

    pub fn resource_classification_variants(&self, id: Uuid) -> Vec<ClassificationVariant> {
        self.resource_content(id, RESOURCE_CLASSIFICATIONS)
            .unwrap_or_default()
    }

    pub fn resource_tag_concept(&self, id: Uuid) -> Option<TagConcept> {
        self.resource_content(id, TAG_CONCEPT)
    }

    /// Labels of the concepts this resource is held to be tagged with.
    pub fn resource_tag_labels(&self, id: Uuid) -> HashSet<String> {
        self.resource_edges()
            .filter(|(from, _, payload)| {
                from.id() == id
                    && payload
                        .held_semantic_statements()
                        .any(|statement| statement.predicate == TAGGED_WITH_IRI)
            })
            .filter_map(|(_, to, _)| {
                self.resource_tag_concept(to.id())
                    .map(|concept| concept.label)
            })
            .collect()
    }
}
