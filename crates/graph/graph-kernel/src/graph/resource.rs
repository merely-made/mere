// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Resource identity beneath browsing surfaces.

use super::{Graph, NodeKey};
use crate::persistence::{PersistedEdge, PersistedResourceRecord};
use mere::model::ResourceDelta;
use uuid::Uuid;

pub use super::resource_pair_bounds::ResourceEdgesBoundError;
pub use mere::model::resource::{ResourceNode, TAGGED_WITH_IRI};

impl Graph {
    /// Read a resource by its stable identity.
    pub fn resource(&self, id: Uuid) -> Option<&ResourceNode> {
        self.resources.resource(id)
    }

    /// The resources currently held in this graph.
    pub fn resource_nodes(&self) -> impl Iterator<Item = &ResourceNode> {
        self.resources.resource_nodes()
    }

    /// Resource edges with their resource endpoints. Surface indices never cross this seam.
    pub fn resource_edges(
        &self,
    ) -> impl Iterator<Item = (&ResourceNode, &ResourceNode, &super::EdgePayload)> {
        self.resources.resource_edges()
    }

    /// Resource metadata, separate from surface metadata.
    pub fn resource_facets(&self) -> &portolan::FacetStore<Uuid> {
        self.resources.facets()
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
        self.resources.resource_record(id)
    }

    pub(crate) fn set_resource_record(
        &mut self,
        id: Uuid,
        record: Option<PersistedResourceRecord>,
    ) -> bool {
        self.write_resource_record(id, record).is_some()
    }

    /// Write or remove a resource record through the engine store, returning
    /// the delta it made for the caller to record (engine split D250).
    pub(crate) fn write_resource_record(
        &mut self,
        id: Uuid,
        record: Option<PersistedResourceRecord>,
    ) -> Option<ResourceDelta> {
        let Some(record) = record else {
            // A binding still showing the resource is the join's to refuse.
            if self
                .shown_resources
                .values()
                .any(|resource| *resource == id)
            {
                return None;
            }
            let delta = self.resources.remove_record(id)?;
            self.bump_revision();
            return Some(delta);
        };
        let delta = self.resources.put_record(id, record)?;
        self.bump_revision();
        Some(delta)
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
            if !self.resource_surfaces.contains_key(&resource) {
                let surfaces = std::collections::BTreeSet::from([surface]);
                self.resource_surfaces.insert_mut(resource, surfaces.into());
            } else if let Some(surfaces) = self.resource_surfaces.get_mut(&resource) {
                surfaces.insert(surface);
            }
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
        // Reads first: a persistent map copies its path on any write.
        let Some(surfaces) = self.resource_surfaces.get(&resource) else {
            return;
        };
        if !surfaces.contains(&surface) {
            return;
        }
        if surfaces.len() == 1 {
            self.resource_surfaces.remove_mut(&resource);
        } else if let Some(surfaces) = self.resource_surfaces.get_mut(&resource) {
            surfaces.remove(&surface);
        }
    }

    pub(crate) fn persisted_resource_edges_between(
        &self,
        from: Uuid,
        to: Uuid,
    ) -> Vec<PersistedEdge> {
        self.resources.persisted_edges_between(from, to)
    }

    pub(crate) fn set_resource_edges_between(
        &mut self,
        from: Uuid,
        to: Uuid,
        edges: &[PersistedEdge],
    ) -> bool {
        self.write_resource_edges(from, to, edges).is_some()
    }

    /// Replace a resource pair's edges through the engine store, returning the
    /// delta it made for the caller to record (engine split D250).
    pub(crate) fn write_resource_edges(
        &mut self,
        from: Uuid,
        to: Uuid,
        edges: &[PersistedEdge],
    ) -> Option<ResourceDelta> {
        let (Some(source), Some(target)) =
            (self.resources.key_of(&from), self.resources.key_of(&to))
        else {
            return None;
        };
        if !super::edge_admission::resource_rows_wellformed(from, to, edges) {
            return None;
        }
        let mut statements = std::collections::BTreeMap::new();
        for statement in edges.iter().flat_map(|edge| edge.retained_assertions()) {
            if let Some(previous) = statements.insert(&statement.statement_id, statement)
                && previous != statement
            {
                return None;
            }
        }
        if statements
            .keys()
            .any(|id| self.handle_held_off_resource_pair(id, source, target))
        {
            return None;
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
        let delta = self.resources.replace_edges(from, to, payloads)?;
        self.bump_revision();
        Some(delta)
    }
}

#[cfg(test)]
#[path = "resource/tests.rs"]
mod tests;
