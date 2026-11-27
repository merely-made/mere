// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The surface's projection: the resource stratum and one quad per surface.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use mere::ResourceGraph;
use mere::projection::{ProjectionKey, ProjectionSource, RdfSource, SourceQuads};
use oxrdf::{GraphName, NamedNode, Quad};
use uuid::Uuid;

use super::Surface;

/// The predicate a surface's one projected quad names its resource by.
pub const SHOWS: &str = "https://mere.test/surface#shows";

/// A surface's subject IRI in the projection.
pub fn surface_iri(surface: Uuid) -> String {
    format!("urn:mere-test-surface:{surface}")
}

impl Surface {
    /// The quad that says surface `surface` shows its resource: the surface
    /// stratum, which the projection carries and the resource stratum does not.
    fn surface_quad(&self, surface: Uuid) -> Option<Quad> {
        let resource = self.resources.resource(self.shown(surface)?)?;
        Some(Quad::new(
            NamedNode::new_unchecked(surface_iri(surface)),
            NamedNode::new_unchecked(SHOWS),
            NamedNode::new_unchecked(resource.canonical_iri()),
            GraphName::DefaultGraph,
        ))
    }
}

impl RdfSource for Surface {
    fn held_quads(&self) -> Box<dyn Iterator<Item = Quad> + '_> {
        let surfaces = self.shown.keys().filter_map(|id| self.surface_quad(*id));
        Box::new(RdfSource::held_quads(&*self.resources).chain(surfaces))
    }

    fn withdrawn_reifier_quads(&self) -> Vec<Quad> {
        RdfSource::withdrawn_reifier_quads(&*self.resources)
    }

    fn projection_key(&self) -> ProjectionKey {
        let mut hasher = DefaultHasher::new();
        RdfSource::projection_key(&*self.resources).hash(&mut hasher);
        self.shown.hash(&mut hasher);
        ProjectionKey::new([hasher.finish(), 0, 0, 0, 0, 0, 0, 0])
    }

    fn sources(&self) -> Option<Vec<ProjectionSource>> {
        let mut sources = RdfSource::sources(&*self.resources)?;
        sources.extend(self.shown.keys().map(|id| ProjectionSource::Node(*id)));
        Some(sources)
    }

    fn source_quads(&self, source: ProjectionSource) -> Option<SourceQuads> {
        match source {
            ProjectionSource::Node(id) => Some(SourceQuads {
                subject: Some(surface_iri(id)),
                held: vec![self.surface_quad(id)?],
                withdrawn: Vec::new(),
            }),
            other => RdfSource::source_quads(&*self.resources, other),
        }
    }

    /// A resource's dependents carry the surfaces showing it, whose quads name
    /// it by its IRI.
    fn dependents(&self, source: ProjectionSource) -> Vec<ProjectionSource> {
        let mut dependents = RdfSource::dependents(&*self.resources, source);
        if let ProjectionSource::Resource(resource) = source {
            dependents.extend(
                self.shown
                    .iter()
                    .filter(|(_, shown)| **shown == resource)
                    .map(|(surface, _)| ProjectionSource::Node(*surface)),
            );
        }
        dependents
    }

    fn resource_graph(&self) -> &ResourceGraph {
        &self.resources
    }
}
