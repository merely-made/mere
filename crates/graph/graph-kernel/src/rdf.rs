// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The legacy snapshot adapter's RDF half (engine split P5 L3, D141 F5).
//!
//! The surface walk of the graph-to-RDF projection and the contribution
//! importer (`apply_contribution`, `apply_import`) moved here from the `mere`
//! crate, so the engine no longer depends on the kernel. [`Graph`] implements
//! [`mere::RdfSource`] by walking its surfaces first and then the engine's
//! resource projection, which keeps the projection's content and order, so the
//! held index (`mere::query::IndexedDataset::for_graph`) and every format writer
//! read a kernel graph as before. This module leaves with the kernel at P8.

use mere::model::ClassificationScheme;
use mere::projection::{
    push_withdrawn_reifiers, resource_dependents, resource_quads, resource_source_quads,
    resource_sources, resource_withdrawn_reifier_quads,
};
use mere::{
    ProjectionKey, ProjectionSource, RDF_TYPE, RdfSource, SourceQuads, SCHEMA_KEYWORDS, SCHEMA_NAME, graph_name_for_scope,
    property_literal, push_quad, push_semantic_statement_metadata_quads,
    push_statement_metadata_quads,
};
use oxrdf::{GraphName, Literal, NamedNode, Quad};
use uuid::Uuid;

use crate::graph::CoverageNote;
use crate::graph::{Graph, Node, NodeKey};
use crate::types::GraphScope;

#[cfg(not(target_arch = "wasm32"))]
mod apply;
mod refresh;
pub use refresh::touched_sources;
#[cfg(not(target_arch = "wasm32"))]
pub use apply::{
    ApplyOutcome, apply_contribution, apply_contribution_with_identity, apply_import,
    apply_import_with_identity,
};

/// A node's `@id`: its primary address URL, or a skolemized `urn:uuid:` IRI when
/// the node has no dereferenceable address.
pub fn node_id(node: &Node) -> String {
    let url = node.primary_address().as_url_str();
    if url.is_empty() {
        format!("urn:uuid:{}", node.id)
    } else {
        url.to_string()
    }
}

/// The direct RDF quads for one node: its `@id` subject carrying `rdf:type`,
/// the curated literals (`schema:name` / `schema:keywords`), the recognized and
/// raw `Semantic` edges, and the open literal properties, with graph scope taken
/// from semantic statements / node properties.
fn node_direct_quads(graph: &Graph, key: NodeKey, node: &Node) -> Vec<Quad> {
    let mut quads = Vec::new();
    let Ok(subject) = NamedNode::new(node_id(node)) else {
        return quads;
    };

    // `rdf:type` from the node's `rdf:type` classifications, sorted + deduped.
    let classifications = graph.legacy_node_classifications(key).unwrap_or_default();
    let mut types: Vec<&str> = classifications
        .iter()
        .filter(|c| {
            c.status.is_affirmative()
                && matches!(&c.scheme, ClassificationScheme::Custom(s) if s == "rdf:type")
        })
        .map(|c| c.value.as_str())
        .collect();
    types.sort_unstable();
    types.dedup();
    for ty in types {
        if let Ok(ty) = NamedNode::new(ty) {
            push_quad(
                &mut quads,
                &subject,
                RDF_TYPE,
                ty.into(),
                &GraphScope::Default,
            );
        }
    }

    // Curated literals. Skip a title that is only the URL fallback (`add_node`
    // seeds `title = url` for an untitled node), which is not a real name.
    let url = node.primary_address().as_url_str();
    if !node.title.is_empty() && node.title != url {
        push_quad(
            &mut quads,
            &subject,
            SCHEMA_NAME,
            Literal::new_simple_literal(node.title.as_str()).into(),
            &GraphScope::Default,
        );
    }
    let mut tags: Vec<&str> = node.tags.iter().map(String::as_str).collect();
    tags.sort_unstable();
    for tag in tags {
        push_quad(
            &mut quads,
            &subject,
            SCHEMA_KEYWORDS,
            Literal::new_simple_literal(tag).into(),
            &GraphScope::Default,
        );
    }

    // Canonical assertion edges -> predicate IRIs -> target `@id`s, sorted by
    // (predicate, target, graph) for a stable dataset.
    let mut edges: Vec<(String, String, GraphScope)> = Vec::new();
    for (_, _, target, payload) in graph.surface_outgoing_relation_buckets(key) {
        let Some(target_node) = graph.get_node(target) else {
            continue;
        };
        let target_id = node_id(target_node);
        for statement in payload.held_assertions() {
            edges.push((
                statement.predicate.clone(),
                target_id.clone(),
                statement.graph_scope.clone(),
            ));
        }
    }
    edges.sort();
    edges.dedup();
    for (predicate, target_id, graph_scope) in edges {
        if let Ok(target) = NamedNode::new(target_id) {
            push_quad(
                &mut quads,
                &subject,
                &predicate,
                target.into(),
                &graph_scope,
            );
        }
    }

    // Open literal properties, sorted by full literal identity.
    let mut props = graph.legacy_node_properties(key).unwrap_or_default();
    props.sort_by(|a, b| {
        (
            a.predicate.as_str(),
            a.value.as_str(),
            a.datatype.as_deref(),
            a.lang.as_deref(),
            &a.graph_scope,
        )
            .cmp(&(
                b.predicate.as_str(),
                b.value.as_str(),
                b.datatype.as_deref(),
                b.lang.as_deref(),
                &b.graph_scope,
            ))
    });
    for property in props {
        push_quad(
            &mut quads,
            &subject,
            property.predicate.as_str(),
            property_literal(&property).into(),
            &property.graph_scope,
        );
    }

    quads
}

/// Dataset-only metadata quads for one node. These are reifier nodes that carry
/// statement handles plus provenance / time / label metadata without polluting
/// the node-subject JSON-LD shaper.
fn node_metadata_quads(graph: &Graph, key: NodeKey, node: &Node) -> Vec<Quad> {
    let mut quads = Vec::new();
    let Ok(subject) = NamedNode::new(node_id(node)) else {
        return quads;
    };

    let mut semantic_statements = Vec::new();
    for target in graph.out_neighbors(key) {
        let Some(edge_key) = graph.find_edge_key(key, target) else {
            continue;
        };
        let Some(payload) = graph.get_edge(edge_key) else {
            continue;
        };
        let Some(target_node) = graph.get_node(target) else {
            continue;
        };
        let Ok(target_id) = NamedNode::new(node_id(target_node)) else {
            continue;
        };
        for statement in payload.held_assertions() {
            semantic_statements.push((statement.clone(), target_id.clone()));
        }
    }
    semantic_statements.sort_by(|a, b| {
        (
            &a.0.predicate,
            a.1.as_str(),
            &a.0.graph_scope,
            &a.0.statement_id,
        )
            .cmp(&(
                &b.0.predicate,
                b.1.as_str(),
                &b.0.graph_scope,
                &b.0.statement_id,
            ))
    });
    for (statement, target) in semantic_statements {
        if let Some(graph_name) = graph_name_for_scope(&statement.graph_scope) {
            push_semantic_statement_metadata_quads(
                &mut quads,
                &subject,
                &statement.predicate,
                target.into(),
                &statement,
                graph_name,
                false,
            );
        }
    }

    let mut properties = graph.legacy_node_properties(key).unwrap_or_default();
    properties.sort_by(|a, b| {
        (
            a.predicate.as_str(),
            a.value.as_str(),
            a.datatype.as_deref(),
            a.lang.as_deref(),
            &a.graph_scope,
            a.statement_id.as_str(),
        )
            .cmp(&(
                b.predicate.as_str(),
                b.value.as_str(),
                b.datatype.as_deref(),
                b.lang.as_deref(),
                &b.graph_scope,
                b.statement_id.as_str(),
            ))
    });
    for property in properties {
        push_statement_metadata_quads(
            &mut quads,
            &subject,
            property.statement_id.as_str(),
            property.predicate.as_str(),
            property_literal(&property).into(),
            &property.graph_scope,
            None,
            property.provenance_iri.as_deref(),
            property.asserted_at_ms,
        );
    }

    quads
}

fn node_dataset_quads(graph: &Graph, key: NodeKey, node: &Node) -> Vec<Quad> {
    let mut quads = node_direct_quads(graph, key, node);
    quads.extend(node_metadata_quads(graph, key, node));
    quads
}

/// The RDF quads for one node, restricted to the default graph. This remains
/// the compatibility view for callers that need direct default-graph node quads.
pub fn node_quads(graph: &Graph, key: NodeKey, node: &Node) -> Vec<Quad> {
    node_direct_quads(graph, key, node)
        .into_iter()
        .filter(|quad| matches!(quad.graph_name, GraphName::DefaultGraph))
        .collect()
}

/// The withdrawn reifiers of every surface relation bucket.
fn surface_withdrawn_reifier_quads(graph: &Graph) -> Vec<Quad> {
    let mut quads = Vec::new();
    for (_, from, to, payload) in graph.surface_relation_buckets() {
        let (Some(from), Some(to)) = (graph.get_node(from), graph.get_node(to)) else {
            continue;
        };
        let (Ok(subject), Ok(target)) =
            (NamedNode::new(node_id(from)), NamedNode::new(node_id(to)))
        else {
            continue;
        };
        push_withdrawn_reifiers(&mut quads, &subject, &target, payload);
    }
    quads
}

impl RdfSource for Graph {
    fn held_quads(&self) -> Box<dyn Iterator<Item = Quad> + '_> {
        Box::new(
            self.nodes()
                .flat_map(|(key, node)| node_dataset_quads(self, key, node))
                .chain(resource_quads(&self.resources)),
        )
    }

    fn withdrawn_reifier_quads(&self) -> Vec<Quad> {
        let mut quads = surface_withdrawn_reifier_quads(self);
        quads.extend(resource_withdrawn_reifier_quads(&self.resources));
        quads
    }

    fn listed_subjects(&self) -> Vec<String> {
        self.nodes().map(|(_, node)| node_id(node)).collect()
    }

    /// The counters the held index keyed on before the split: structure,
    /// content and semantic observation, plus the structural and content input
    /// keys, so a clone or a fork never reads another's index as current.
    fn projection_key(&self) -> ProjectionKey {
        let structural = self.structural_input_key().parts();
        let content = self.content_input_key().parts();
        ProjectionKey::new([
            self.revision(),
            self.content_revision(),
            self.semantic_observation_revision(),
            structural.0,
            structural.1,
            content.0,
            content.1,
            0,
        ])
    }

    fn sources(&self) -> Option<Vec<ProjectionSource>> {
        let nodes = self.nodes().map(|(_, node)| ProjectionSource::Node(node.id));
        Some(nodes.chain(resource_sources(&self.resources)).collect())
    }

    /// A node's quads are what [`RdfSource::held_quads`] walks for it, plus
    /// the withdrawn reifiers of its outgoing buckets; a resource's come from
    /// the engine's resource projection.
    fn source_quads(&self, source: ProjectionSource) -> Option<SourceQuads> {
        let id = match source {
            ProjectionSource::Node(id) => id,
            ProjectionSource::Resource(id) => return resource_source_quads(&self.resources, id),
            ProjectionSource::Resources => return None,
        };
        let key = self.get_node_key_by_id(id)?;
        let node = self.get_node(key)?;
        let subject = node_id(node);
        let mut withdrawn = Vec::new();
        if let Ok(from) = NamedNode::new(subject.as_str()) {
            for (_, _, to, payload) in self.surface_outgoing_relation_buckets(key) {
                let target = self.get_node(to).map(node_id);
                if let Some(Ok(target)) = target.map(NamedNode::new) {
                    push_withdrawn_reifiers(&mut withdrawn, &from, &target, payload);
                }
            }
        }
        Some(SourceQuads {
            subject: Some(subject),
            held: node_dataset_quads(self, key, node),
            withdrawn,
        })
    }

    /// A node's in-neighbours, whose edges and reifiers name its IRI; for a
    /// resource, the resources with edges into it.
    fn dependents(&self, source: ProjectionSource) -> Vec<ProjectionSource> {
        let id = match source {
            ProjectionSource::Node(id) => id,
            ProjectionSource::Resource(id) => return resource_dependents(&self.resources, id),
            ProjectionSource::Resources => return Vec::new(),
        };
        let Some(key) = self.get_node_key_by_id(id) else {
            return Vec::new();
        };
        let mut sources: Vec<ProjectionSource> = self
            .surface_incoming_relation_buckets(key)
            .filter(|(_, from, _, _)| *from != key)
            .filter_map(|(_, from, _, _)| self.get_node(from))
            .map(|node| ProjectionSource::Node(node.id))
            .collect();
        sources.sort_unstable();
        sources.dedup();
        sources
    }

    fn resource_graph(&self) -> &mere::ResourceGraph {
        &self.resources
    }

    fn coverage_note(&self) -> CoverageNote {
        Graph::coverage_note(self)
    }

    fn coverage_for_resource_refs(&self, references: &[Uuid]) -> CoverageNote {
        Graph::coverage_for_resource_refs(self, references)
    }
}

#[cfg(test)]
#[path = "rdf/tests/ingest.rs"]
mod ingest_tests;
#[cfg(test)]
#[path = "rdf/tests/jsonld.rs"]
mod jsonld_tests;
#[cfg(all(test, feature = "query"))]
#[path = "rdf/tests/query/coverage.rs"]
mod query_coverage_tests;
#[cfg(all(test, feature = "query"))]
#[path = "rdf/tests/query/history_projection.rs"]
mod query_history_projection_tests;
#[cfg(all(test, feature = "query"))]
#[path = "rdf/tests/query/indexed.rs"]
mod query_indexed_tests;
#[cfg(all(test, feature = "query"))]
#[path = "rdf/tests/query/refresh.rs"]
mod query_refresh_tests;
#[cfg(all(test, feature = "query"))]
#[path = "rdf/tests/query/resource_member.rs"]
mod query_resource_member_tests;
#[cfg(all(test, feature = "query"))]
#[path = "rdf/tests/query.rs"]
mod query_tests;
#[cfg(test)]
#[path = "rdf/tests/reifier.rs"]
mod reifier_tests;
#[cfg(test)]
#[path = "rdf/tests/resource_attribution.rs"]
mod resource_attribution_tests;
#[cfg(test)]
#[path = "rdf/tests/resource_metadata.rs"]
mod resource_metadata_tests;
#[cfg(test)]
#[path = "rdf/tests/serialize.rs"]
mod serialize_tests;
#[cfg(test)]
#[path = "rdf/tests/projection.rs"]
mod tests;
