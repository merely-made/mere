// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Resource identity beneath browsing surfaces.

use super::{Graph, NodeKey};
use crate::persistence::{PersistedEdge, PersistedResourceFacet, PersistedResourceRecord};
use chartulary::{Address, Addressed, Identified};
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use uuid::Uuid;

pub use super::resource_pair_bounds::ResourceEdgesBoundError;

/// A tagging assertion points from the tagged resource to its concept resource.
pub const TAGGED_WITH_IRI: &str = "https://mere.computer/ns/rel#taggedWith";

/// One resource identified by a canonical IRI.
///
/// Identity and address are immutable together. Content records will be
/// attached through the kernel's recorded write path.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceNode {
    container: chartulary::Container<Uuid>,
    pub(super) nested_selection: Option<super::FrozenSelection>,
    pub(super) nested_link: Option<super::FrozenLink>,
}

impl ResourceNode {
    /// Identify a resource using the common canonicalizer and UUID namespace.
    pub fn new(iri: &str) -> Self {
        let canonical = chartulary::canonical_url(iri);
        Self::from_canonical_iri(&canonical)
    }

    /// Identify a vocabulary term by its exact IRI, including fragment and case.
    /// Page resources use [`Self::new`] instead.
    pub fn for_term(iri: &str) -> Self {
        Self::from_canonical_iri(iri)
    }

    pub(crate) fn from_canonical_iri(canonical: &str) -> Self {
        Self {
            nested_selection: None,
            nested_link: None,
            container: chartulary::Container::with_identity(
                chartulary::resource_id_from_canonical_iri(canonical),
            )
            .with_address_record(Address::new(canonical)),
        }
    }

    /// Immutable nested selection borne by this Resource, when frozen.
    pub fn nested_selection(&self) -> Option<&super::FrozenSelection> {
        self.nested_selection.as_ref()
    }

    /// Immutable link reading borne by this Resource, when frozen.
    pub fn nested_link(&self) -> Option<&super::FrozenLink> {
        self.nested_link.as_ref()
    }

    /// The stable identity, shared by every surface showing this resource.
    pub fn id(&self) -> Uuid {
        self.container.id
    }

    /// The canonical IRI from which the stable identity was derived.
    pub fn canonical_iri(&self) -> &str {
        self.container.addresses[0].as_str()
    }
}

impl Graph {
    /// Read a resource by its stable identity.
    pub fn resource(&self, id: Uuid) -> Option<&ResourceNode> {
        self.resources.get(&id)
    }

    /// The resources currently held in this graph.
    pub fn resource_nodes(&self) -> impl Iterator<Item = &ResourceNode> {
        self.resources.nodes().map(|(_, resource)| resource)
    }

    /// Resource edges with their resource endpoints. Surface indices never cross this seam.
    pub fn resource_edges(
        &self,
    ) -> impl Iterator<Item = (&ResourceNode, &ResourceNode, &super::EdgePayload)> {
        self.resources.inner().edge_references().map(|edge| {
            (
                self.resources
                    .node(edge.source())
                    .expect("resource endpoint exists"),
                self.resources
                    .node(edge.target())
                    .expect("resource endpoint exists"),
                edge.weight(),
            )
        })
    }

    /// Resource metadata, separate from surface metadata.
    pub fn resource_facets(&self) -> &chartulary::FacetStore<Uuid> {
        &self.resource_facets
    }

    /// The resource explicitly shown by a surface, when recorded.
    pub fn shown_resource_id(&self, surface: NodeKey) -> Option<Uuid> {
        self.shown_resources
            .get(&self.get_node(surface)?.id)
            .copied()
    }

    /// Surface identities showing a resource, in stable order.
    pub fn surface_ids_showing_resource(&self, resource_id: Uuid) -> Vec<Uuid> {
        self.resource_surfaces
            .get(&resource_id)
            .map(|surfaces| surfaces.iter().copied().collect())
            .unwrap_or_default()
    }

    pub fn resource_record(&self, id: Uuid) -> Option<PersistedResourceRecord> {
        let resource = self.resource(id)?;
        let facets = self
            .resource_facets
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

    pub(crate) fn set_resource_record(
        &mut self,
        id: Uuid,
        record: Option<PersistedResourceRecord>,
    ) -> bool {
        let Some(record) = record else {
            let Some(key) = self.resources.key_of(&id) else {
                return false;
            };
            if self
                .shown_resources
                .values()
                .any(|resource| *resource == id)
                || self
                    .resources
                    .inner()
                    .edge_references()
                    .any(|edge| edge.source() == key || edge.target() == key)
            {
                return false;
            }
            self.resources.remove(key);
            self.resource_facets.remove_node(&id);
            self.bump_revision();
            return true;
        };
        let mut resource = ResourceNode::from_canonical_iri(&record.canonical_iri);
        let Ok(selection) = super::frozen_selection::selection_from_record(&record) else {
            return false;
        };
        let Ok(link) = super::frozen_link::validate_frozen_link_record(&record) else {
            return false;
        };
        if let Some(existing) = self
            .resource(id)
            .and_then(|resource| resource.nested_selection())
            && selection.as_ref() != Some(existing)
        {
            return false;
        }
        if self
            .resource(id)
            .is_some_and(|resource| resource.nested_selection().is_some())
        {
            let stored = self.resource_facets.get(
                &id,
                &chartulary::FacetId::new(super::frozen_selection::FROZEN_SELECTION),
            );
            let incoming = record
                .facets
                .iter()
                .find(|facet| facet.facet == super::frozen_selection::FROZEN_SELECTION)
                .and_then(|facet| {
                    serde_json::from_str::<serde_json::Value>(&facet.value_json).ok()
                });
            if stored != incoming.as_ref() {
                return false;
            }
        }
        if let Some(existing) = self
            .resource(id)
            .and_then(|resource| resource.nested_link())
            && link.as_ref() != Some(existing)
        {
            return false;
        }
        if self
            .resource(id)
            .is_some_and(|resource| resource.nested_link().is_some())
        {
            let stored = self.resource_facets.get(
                &id,
                &chartulary::FacetId::new(super::frozen_link::FROZEN_LINK),
            );
            let incoming = record
                .facets
                .iter()
                .find(|facet| facet.facet == super::frozen_link::FROZEN_LINK)
                .and_then(|facet| {
                    serde_json::from_str::<serde_json::Value>(&facet.value_json).ok()
                });
            if stored != incoming.as_ref() {
                return false;
            }
        }
        resource.nested_selection = selection;
        resource.nested_link = link;
        if resource.id() != id {
            return false;
        }
        if super::predicate_declarations::predicate_declarations_from_record(&record).is_err() {
            return false;
        }
        if super::resource_content::validate_content_record(&record).is_err() {
            return false;
        }
        let mut facets = std::collections::BTreeMap::new();
        for facet in record.facets {
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&facet.value_json) else {
                return false;
            };
            if facets
                .insert(chartulary::FacetId::new(facet.facet), value)
                .is_some()
            {
                return false;
            }
        }
        if self
            .resource(id)
            .is_some_and(|existing| existing == &resource)
            && self
                .resource_facets
                .facets_of(&id)
                .map(|values| {
                    values
                        .iter()
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect::<std::collections::BTreeMap<_, _>>()
                })
                .unwrap_or_default()
                == facets
        {
            return false;
        }
        self.resources.insert(resource);
        self.resource_facets.remove_node(&id);
        for (facet, value) in facets {
            self.resource_facets
                .set(id, facet, value, &chartulary::AcceptAll)
                .expect("permissive facet validator");
        }
        self.note_resource_literals(id);
        self.bump_revision();
        true
    }

    /// Runtime receipt for completed new shown-Resource associations.
    /// This advances on admission, including navigation to an already held Resource.
    pub fn appearance_admission_revision(&self) -> u64 {
        self.appearance_admission_revision
    }

    pub(crate) fn set_shown_resource(&mut self, surface: Uuid, resource: Option<Uuid>) -> bool {
        if self.inner.key_of(&surface).is_none()
            || resource.is_some_and(|id| self.resource(id).is_none())
        {
            return false;
        }
        if self.shown_resources.get(&surface).copied() == resource {
            return false;
        }
        let previous = if let Some(resource) = resource {
            self.resource_surfaces
                .entry(resource)
                .or_default()
                .insert(surface);
            self.appearance_admission_revision = self.appearance_admission_revision.wrapping_add(1);
            self.shown_resources.insert(surface, resource)
        } else {
            self.shown_resources.remove(&surface)
        };
        if let Some(previous) = previous {
            self.unlink_shown_surface(previous, surface);
        }
        self.bump_revision();
        true
    }

    pub(crate) fn unlink_shown_surface(&mut self, resource: Uuid, surface: Uuid) {
        if let Some(surfaces) = self.resource_surfaces.get_mut(&resource) {
            surfaces.remove(&surface);
            if surfaces.is_empty() {
                self.resource_surfaces.remove(&resource);
            }
        }
    }

    pub(crate) fn persisted_resource_edges_between(
        &self,
        from: Uuid,
        to: Uuid,
    ) -> Vec<PersistedEdge> {
        let (Some(source), Some(target)) =
            (self.resources.key_of(&from), self.resources.key_of(&to))
        else {
            return Vec::new();
        };
        self.resources
            .inner()
            .edges_connecting(source, target)
            .map(|edge| super::snapshot::persisted_edge_for_ids(from, to, edge.weight()))
            .collect()
    }

    pub(crate) fn set_resource_edges_between(
        &mut self,
        from: Uuid,
        to: Uuid,
        edges: &[PersistedEdge],
    ) -> bool {
        let (Some(source), Some(target)) =
            (self.resources.key_of(&from), self.resources.key_of(&to))
        else {
            return false;
        };
        if !super::edge_admission::resource_rows_wellformed(from, to, edges) {
            return false;
        }
        let mut statements = std::collections::BTreeMap::new();
        for statement in edges.iter().flat_map(|edge| edge.retained_assertions()) {
            if let Some(previous) = statements.insert(&statement.statement_id, statement)
                && previous != statement
            {
                return false;
            }
        }
        for edge in self.resources.inner().edge_references() {
            if edge.source() == source && edge.target() == target {
                continue;
            }
            if edge
                .weight()
                .assertions()
                .iter()
                .any(|statement| statements.contains_key(&statement.statement_id))
            {
                return false;
            }
        }
        if self.inner.inner().edge_weights().any(|edge| {
            edge.assertions()
                .iter()
                .any(|statement| statements.contains_key(&statement.statement_id))
        }) {
            return false;
        }
        let payloads: Vec<_> = edges
            .iter()
            .enumerate()
            .map(|(ordinal, edge)| {
                super::snapshot::payload_from_persisted_at(
                    edge,
                    super::GraphStratum::Resource,
                    ordinal,
                )
            })
            .collect();
        let current: Vec<_> = self
            .resources
            .inner()
            .edges_connecting(source, target)
            .map(|edge| edge.weight())
            .collect();
        if current.len() == payloads.len()
            && current.iter().zip(&payloads).all(|(old, new)| *old == new)
        {
            return false;
        }
        let old: Vec<_> = self
            .resources
            .inner()
            .edges_connecting(source, target)
            .map(|edge| edge.id())
            .collect();
        for edge in old {
            self.resources.disconnect(edge);
        }
        for payload in payloads.into_iter().rev() {
            self.connect_resource(source, target, payload);
        }
        self.bump_revision();
        true
    }
}

impl Identified for ResourceNode {
    type Id = Uuid;

    fn id(&self) -> &Uuid {
        &self.container.id
    }
}

impl Addressed for ResourceNode {
    fn addresses(&self) -> Vec<Address> {
        self.container.addresses.clone()
    }
}

#[cfg(test)]
#[path = "resource/tests.rs"]
mod tests;
