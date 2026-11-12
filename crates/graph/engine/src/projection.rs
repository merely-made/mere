// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The RDF projection's source and its resource half (engine split P5 L3).
//!
//! Every format writer, the held index and SPARQL read a graph through
//! [`RdfSource`]. [`ResourceGraph`] implements it with the resource projection
//! below; `mere-kernel`'s surface graph implements it by putting its surface
//! walk before the same resource projection, which keeps the order and content
//! of the projection the kernel had before the split.

use oxrdf::{GraphName, NamedNode, Quad};
use uuid::Uuid;

pub use crate::resource_metadata::{SKOS_CONCEPT, SKOS_PREF_LABEL};

use crate::model::{CoverageNote, EdgePayload, ResourceGraph, ResourceNode};
use crate::{
    ASSERTION_HISTORY_GRAPH_IRI, graph_name_for_scope, push_quad,
    push_semantic_statement_metadata_quads,
};

/// Identifies one state of a projection: a held index built at one key is
/// current for exactly the sources that report the same key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProjectionKey([u64; 8]);

impl ProjectionKey {
    pub const fn new(parts: [u64; 8]) -> Self {
        Self(parts)
    }
}

/// A graph the engine projects to RDF.
pub trait RdfSource {
    /// Every held quad of the projection, in a stable order; may repeat a quad.
    fn held_quads(&self) -> Box<dyn Iterator<Item = Quad> + '_>;

    /// The reifiers of withdrawn assertions in [`ASSERTION_HISTORY_GRAPH_IRI`];
    /// may repeat a quad.
    fn withdrawn_reifier_quads(&self) -> Vec<Quad>;

    /// Subjects a JSON-LD export lists even when no quad describes them.
    fn listed_subjects(&self) -> Vec<String> {
        Vec::new()
    }

    /// Changes whenever the projection may change.
    fn projection_key(&self) -> ProjectionKey;

    /// The resources beneath the projection.
    fn resource_graph(&self) -> &ResourceGraph;

    /// Known limits of this graph. Empty means none known, not completeness.
    fn coverage_note(&self) -> CoverageNote {
        CoverageNote::default()
    }

    /// Known limits that bear on these resource references.
    fn coverage_for_resource_refs(&self, _references: &[Uuid]) -> CoverageNote {
        CoverageNote::default()
    }
}

/// The held quads of every resource and resource edge: per-resource metadata
/// first, then each edge's held assertions with their reifiers.
pub fn resource_quads(resources: &ResourceGraph) -> impl Iterator<Item = Quad> + '_ {
    resources
        .resource_nodes()
        .flat_map(|resource| crate::resource_metadata::quads(resources, resource))
        .chain(
            resources
                .resource_edges()
                .flat_map(|(from, to, payload)| resource_edge_quads(from, to, payload)),
        )
}

fn resource_edge_quads(from: &ResourceNode, to: &ResourceNode, payload: &EdgePayload) -> Vec<Quad> {
    let mut quads = Vec::new();
    let (Ok(subject), Ok(target)) = (
        NamedNode::new(from.canonical_iri()),
        NamedNode::new(to.canonical_iri()),
    ) else {
        return quads;
    };
    for statement in payload.held_assertions() {
        push_quad(
            &mut quads,
            &subject,
            &statement.predicate,
            target.clone().into(),
            &statement.graph_scope,
        );
        if let Some(graph_name) = graph_name_for_scope(&statement.graph_scope) {
            push_semantic_statement_metadata_quads(
                &mut quads,
                &subject,
                &statement.predicate,
                target.clone().into(),
                statement,
                graph_name,
                false,
            );
        }
    }
    let mut seen = std::collections::HashSet::new();
    quads.retain(|quad| seen.insert(quad.clone()));
    quads
}

/// The history graph's name.
pub fn history_graph() -> GraphName {
    GraphName::NamedNode(NamedNode::new_unchecked(ASSERTION_HISTORY_GRAPH_IRI))
}

/// Push the reifier of every withdrawn assertion on one subject-target bucket
/// into the history graph.
pub fn push_withdrawn_reifiers(
    quads: &mut Vec<Quad>,
    subject: &NamedNode,
    target: &NamedNode,
    payload: &EdgePayload,
) {
    for statement in payload
        .assertions()
        .iter()
        .filter(|statement| statement.withdrawn.is_some())
    {
        push_semantic_statement_metadata_quads(
            quads,
            subject,
            &statement.predicate,
            target.clone().into(),
            statement,
            history_graph(),
            true,
        );
    }
}

/// The withdrawn reifiers of every resource edge.
pub fn resource_withdrawn_reifier_quads(resources: &ResourceGraph) -> Vec<Quad> {
    let mut quads = Vec::new();
    for (from, to, payload) in resources.resource_edges() {
        let (Ok(subject), Ok(target)) = (
            NamedNode::new(from.canonical_iri()),
            NamedNode::new(to.canonical_iri()),
        ) else {
            continue;
        };
        push_withdrawn_reifiers(&mut quads, &subject, &target, payload);
    }
    quads
}

impl RdfSource for ResourceGraph {
    fn held_quads(&self) -> Box<dyn Iterator<Item = Quad> + '_> {
        Box::new(resource_quads(self))
    }

    fn withdrawn_reifier_quads(&self) -> Vec<Quad> {
        resource_withdrawn_reifier_quads(self)
    }

    fn projection_key(&self) -> ProjectionKey {
        ProjectionKey::new([self.version(), 0, 0, 0, 0, 0, 0, 0])
    }

    fn resource_graph(&self) -> &ResourceGraph {
        self
    }
}
