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
use petgraph::Direction;
use petgraph::visit::EdgeRef;
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

/// One part of a projection that re-projects alone, so a held index can
/// refresh what an edit touched (quad index plan P4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProjectionSource {
    /// A surface node: its own quads, its outgoing statements with their
    /// reifiers, and their withdrawn reifiers.
    Node(Uuid),
    /// A resource: its own metadata, its outgoing edges' held assertions with
    /// their reifiers, and their withdrawn reifiers.
    Resource(Uuid),
    /// Every resource at once: named in a touched list when an edit cannot be
    /// attributed to particular resources; a refresh expands it.
    Resources,
}

/// One resource's quads, or `None` when it is not in the graph. The same rows
/// the whole-graph walk ([`resource_quads`], [`resource_withdrawn_reifier_quads`])
/// emits for it, from the same per-resource and per-edge functions.
pub fn resource_source_quads(resources: &ResourceGraph, id: Uuid) -> Option<SourceQuads> {
    let key = resources.key_of(&id)?;
    let resource = resources.node(key)?;
    let mut held = crate::resource_metadata::quads(resources, resource);
    let mut withdrawn = Vec::new();
    let subject = NamedNode::new(resource.canonical_iri()).ok();
    for edge in resources.inner().edges_directed(key, Direction::Outgoing) {
        let Some(to) = resources.node(edge.target()) else {
            continue;
        };
        held.extend(resource_edge_quads(resource, to, edge.weight()));
        if let (Some(subject), Ok(target)) = (&subject, NamedNode::new(to.canonical_iri())) {
            push_withdrawn_reifiers(&mut withdrawn, subject, &target, edge.weight());
        }
    }
    Some(SourceQuads {
        subject: Some(resource.canonical_iri().to_string()),
        held,
        withdrawn,
    })
}

/// Every resource as a source.
pub fn resource_sources(resources: &ResourceGraph) -> impl Iterator<Item = ProjectionSource> + '_ {
    resources
        .resource_nodes()
        .map(|resource| ProjectionSource::Resource(resource.id()))
}

/// The resources whose edges name `id`'s IRI.
pub fn resource_dependents(resources: &ResourceGraph, id: Uuid) -> Vec<ProjectionSource> {
    let Some(key) = resources.key_of(&id) else {
        return Vec::new();
    };
    let mut sources: Vec<ProjectionSource> = resources
        .inner()
        .edges_directed(key, Direction::Incoming)
        .filter(|edge| edge.source() != key)
        .filter_map(|edge| resources.node(edge.source()))
        .map(|resource| ProjectionSource::Resource(resource.id()))
        .collect();
    sources.sort_unstable();
    sources.dedup();
    sources
}

/// One source's quads.
#[derive(Clone, Debug, Default)]
pub struct SourceQuads {
    /// The IRI the source's quads name it by, which other sources' quads
    /// also carry; a change means those must be re-projected too.
    pub subject: Option<String>,
    /// Held quads; may repeat a quad.
    pub held: Vec<Quad>,
    /// Withdrawn reifiers in [`ASSERTION_HISTORY_GRAPH_IRI`]; may repeat.
    pub withdrawn: Vec<Quad>,
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

    /// The sources whose quads together are the projection, when each
    /// re-projects alone; `None` when the projection does not split.
    fn sources(&self) -> Option<Vec<ProjectionSource>> {
        None
    }

    /// One source's quads, or `None` when the source no longer exists or the
    /// projection does not split.
    fn source_quads(&self, _source: ProjectionSource) -> Option<SourceQuads> {
        None
    }

    /// The other sources whose quads carry `source`'s subject IRI, which a
    /// change of that IRI also changes.
    fn dependents(&self, _source: ProjectionSource) -> Vec<ProjectionSource> {
        Vec::new()
    }

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

    fn sources(&self) -> Option<Vec<ProjectionSource>> {
        Some(resource_sources(self).collect())
    }

    fn source_quads(&self, source: ProjectionSource) -> Option<SourceQuads> {
        match source {
            ProjectionSource::Resource(id) => resource_source_quads(self, id),
            ProjectionSource::Node(_) | ProjectionSource::Resources => None,
        }
    }

    fn dependents(&self, source: ProjectionSource) -> Vec<ProjectionSource> {
        match source {
            ProjectionSource::Resource(id) => resource_dependents(self, id),
            ProjectionSource::Node(_) | ProjectionSource::Resources => Vec::new(),
        }
    }

    fn resource_graph(&self) -> &ResourceGraph {
        self
    }
}
