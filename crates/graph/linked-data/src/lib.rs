// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! # linked-data
//!
//! JSON-LD bridge between the Mere graph [`kernel`] and linked data (linked-data
//! plan, `design_docs/mere_docs/implementation_strategy/2026-05-22_linked_data_ingest_export_plan.md`).
//!
//! **Export.** [`to_jsonld`] emits *expanded* JSON-LD (full IRIs, no `@context`);
//! [`to_jsonld_compact`] emits the *compacted* form (a `@graph` under an inline
//! `@context`). Each held exact assertion becomes an RDF predicate and reifier,
//! including semantic, containment, arrangement, imported, and provenance
//! relation kinds. Recognized relation kinds use their canonical Mere IRI;
//! open predicates pass through verbatim. The traversal event rollup remains
//! Mere's experience layer and is not exported as a relation assertion.
//! Curated literals stay on their fast paths (`title` → `schema:name`, `tags` →
//! `schema:keywords`), and the open node property bag projects as RDF literals
//! with datatype / language-tag fidelity. `@id` is the node's URL, or a
//! skolemized `urn:uuid:` IRI when it has none.
//!
//! **Ingest.** [`from_jsonld`] parses a document into a [`GraphContribution`]
//! (via `oxjsonld`); [`from_jsonld_with_contexts`] resolves a remote `@context`
//! from a bundled [`ContextCache`] rather than the network. [`apply_contribution`]
//! materializes a contribution into a graph (recognized predicate → typed
//! sub-kind, raw → open-predicate edge). Export ↔ ingest round-trips.
//!
//! Hosts parse HTML and dispatch preserved `application/ld+json` blocks into
//! these JSON-LD entry points. `mere-document-lanes` owns that Fleece adapter;
//! this crate remains an RDF processor rather than a second HTML parser.
//!
//! Later: `@type` ↔ classification mapping.

#![doc(html_root_url = "https://docs.rs/linked-data/0.0.1")]

use kernel::graph::{Graph, Node, NodeKey, SemanticStatement};
use kernel::types::ClassificationScheme;
use kernel::types::{GraphScope, NodeProperty};
use oxrdf::{GraphName, Literal, NamedNode, Quad, Term, Triple};
use serde_json::{Map, Value};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

mod jsonld;
mod reifier;
mod resource_metadata;

use reifier::statement_reifier_id;

/// JSON-LD ingest (Phase 2): `application/ld+json` → a graph contribution.
pub mod ingest;

/// Standard-vocabulary alignment for Mere's recognized relations: the 3-category
/// (exact / approximate / Mere-only) mapping projected as `owl:equivalentProperty`
/// / `rdfs:subPropertyOf` quads.
pub mod vocab;

/// Turtle-family file I/O (N-Quads / TriG) for the RDF projection, via `oxttl`.
pub mod serialize;


/// SPARQL query over the graph via spareval evaluating directly over the
/// borrowed RDF projection (the `query` feature).
#[cfg(feature = "query")]
pub mod query;

#[cfg(not(target_arch = "wasm32"))]
pub use ingest::{
    ApplyOutcome, apply_contribution, apply_contribution_with_identity, apply_import,
    apply_import_with_identity,
};
pub use ingest::{
    ContextCache, EdgeContribution, GraphContribution, ImportEnvelope, IngestError,
    NodeContribution, SubjectIdentity, from_jsonld, from_jsonld_envelope,
    from_jsonld_envelope_with_contexts, from_jsonld_envelope_with_contexts_and_base_iri,
    from_jsonld_with_contexts, from_jsonld_with_contexts_and_base_iri, from_quads,
    from_quads_envelope, is_bundled_context, referenced_context_urls,
};
pub use serialize::{
    from_nquads, from_nquads_envelope, from_trig, from_trig_envelope, to_nquads,
    to_nquads_with_history, to_trig, to_trig_with_history,
};
pub use vocab::{Alignment, alignment, vocabulary_alignment_quads};

/// `schema:name` — the curated mapping target for a node's title.
pub(crate) const SCHEMA_NAME: &str = "https://schema.org/name";
/// `schema:keywords` — the curated mapping target for a node's tags.
pub(crate) const SCHEMA_KEYWORDS: &str = "https://schema.org/keywords";
/// `rdf:type` — the predicate JSON-LD `@type` expands to.
pub(crate) const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const RDF_REIFIES: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies";
const RDFS_LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";
const PROV_WAS_ATTRIBUTED_TO: &str = "http://www.w3.org/ns/prov#wasAttributedTo";
const PROV_GENERATED_AT_TIME: &str = "http://www.w3.org/ns/prov#generatedAtTime";
const PROV_INVALIDATED_AT_TIME: &str = "http://www.w3.org/ns/prov#invalidatedAtTime";
const PROV_WAS_INVALIDATED_BY: &str = "http://www.w3.org/ns/prov#wasInvalidatedBy";
const PROV_WAS_ASSOCIATED_WITH: &str = "http://www.w3.org/ns/prov#wasAssociatedWith";
const PROV_WAS_GENERATED_BY: &str = "http://www.w3.org/ns/prov#wasGeneratedBy";
const PROV_WAS_DERIVED_FROM: &str = "http://www.w3.org/ns/prov#wasDerivedFrom";
const PROV_ACTIVITY: &str = "http://www.w3.org/ns/prov#Activity";
const MERE_LAST_ASSERTED_AT: &str = "https://mere.computer/ns/lastAssertedAt";
const MERE_METHOD_VERSION: &str = "https://mere.computer/ns/methodVersion";
const MERE_LIFECYCLE_VERSION: &str = "https://mere.computer/ns/assertionLifecycleVersion";
const MERE_ASSERTED_GRAPH_KIND: &str = "https://mere.computer/ns/assertedGraphKind";
const MERE_ASSERTED_GRAPH_NAME: &str = "https://mere.computer/ns/assertedGraphName";
/// Reserved named graph containing reifiers emitted by explicit history views.
pub const ASSERTION_HISTORY_GRAPH_IRI: &str = "https://mere.computer/ns/graph#history";
const WITHDRAWAL_ACTIVITY_PREFIX: &str = "urn:mere:withdrawal-activity:v1:";
const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
const XSD_DATETIME: &str = "http://www.w3.org/2001/XMLSchema#dateTime";
const GRAPH_SCOPE_SOURCE: &str = "https://mere.computer/ns/graph#source";
const GRAPH_SCOPE_USER: &str = "https://mere.computer/ns/graph#user";
const GRAPH_SCOPE_AGENT: &str = "https://mere.computer/ns/graph#agent";
const GRAPH_SCOPE_MOOT: &str = "https://mere.computer/ns/graph#moot";

/// Selects the RDF assertion projection. The ordinary projection is held-only;
/// history explicitly adds withdrawn reifiers in [`ASSERTION_HISTORY_GRAPH_IRI`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AssertionProjection {
    #[default]
    Held,
    History,
}

/// Export the complete RDF projection as expanded JSON-LD.
/// Named graphs retain their scopes; assertions use classic RDF reification.
pub fn to_jsonld(graph: &Graph) -> Value {
    jsonld::export(graph, false)
}

/// Export JSON-LD with retained withdrawal history in the explicit history graph.
/// Withdrawn fact triples are reified there and are never asserted as base triples.
pub fn to_jsonld_with_history(graph: &Graph) -> Value {
    jsonld::export_with_projection(graph, false, AssertionProjection::History)
}

/// Pretty-printed [`to_jsonld`], for goldens and human inspection.
pub fn to_jsonld_string(graph: &Graph) -> String {
    serde_json::to_string_pretty(&to_jsonld(graph))
        .expect("a JSON-LD document of strings is always serializable")
}

/// A node's `@id`: its primary address URL, or a skolemized `urn:uuid:` IRI when
/// the node has no dereferenceable address.
fn node_id(node: &Node) -> String {
    let url = node.primary_address().as_url_str();
    if url.is_empty() {
        format!("urn:uuid:{}", node.id)
    } else {
        url.to_string()
    }
}

fn graph_name_for_scope(scope: &GraphScope) -> Option<GraphName> {
    match scope {
        GraphScope::Default => Some(GraphName::DefaultGraph),
        GraphScope::Source => NamedNode::new(GRAPH_SCOPE_SOURCE).ok().map(GraphName::from),
        GraphScope::User => NamedNode::new(GRAPH_SCOPE_USER).ok().map(GraphName::from),
        GraphScope::Agent => NamedNode::new(GRAPH_SCOPE_AGENT).ok().map(GraphName::from),
        GraphScope::Moot => NamedNode::new(GRAPH_SCOPE_MOOT).ok().map(GraphName::from),
        GraphScope::Custom(iri) => NamedNode::new(iri.as_str()).ok().map(GraphName::from),
    }
}

/// Push a quad `subject —predicate→ object` in the given graph scope, skipping
/// it when either the predicate or the graph name is not a valid IRI.
fn push_quad(
    quads: &mut Vec<Quad>,
    subject: &NamedNode,
    predicate: &str,
    object: Term,
    graph_scope: &GraphScope,
) {
    if let (Ok(predicate), Some(graph_name)) =
        (NamedNode::new(predicate), graph_name_for_scope(graph_scope))
    {
        quads.push(Quad::new(subject.clone(), predicate, object, graph_name));
    }
}

fn literal_json_value(literal: &Literal) -> Value {
    let mut value = Map::new();
    value.insert(
        "@value".to_string(),
        Value::String(literal.value().to_string()),
    );
    if let Some(language) = literal.language() {
        value.insert("@language".to_string(), Value::String(language.to_string()));
    } else {
        let datatype = literal.datatype().as_str();
        if datatype != XSD_STRING {
            value.insert("@type".to_string(), Value::String(datatype.to_string()));
        }
    }
    Value::Object(value)
}

fn compact_literal_json_value(literal: &Literal) -> Value {
    if literal.language().is_none() && literal.datatype().as_str() == XSD_STRING {
        Value::String(literal.value().to_string())
    } else {
        literal_json_value(literal)
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

/// The RDF dataset quads for the whole graph, including named-graph scoped
/// semantic statements and node properties plus dataset-only reifier metadata.
pub fn dataset_quads(graph: &Graph) -> Vec<Quad> {
    let mut seen = std::collections::HashSet::new();
    dataset_quad_iter(graph)
        .filter(|quad| seen.insert(quad.clone()))
        .collect()
}

/// Explicit history projection: held assertions remain in their original
/// graph, while withdrawn assertions appear only as reifiers in the reserved
/// history graph with their original `GraphScope` carried separately.
pub fn dataset_quads_with_history(graph: &Graph) -> Vec<Quad> {
    let mut quads = dataset_quads(graph);
    let Ok(history_graph) = NamedNode::new(ASSERTION_HISTORY_GRAPH_IRI) else {
        return quads;
    };
    for (_, from, to, payload) in graph.surface_relation_buckets() {
        let (Some(from), Some(to)) = (graph.get_node(from), graph.get_node(to)) else {
            continue;
        };
        let (Ok(subject), Ok(target)) =
            (NamedNode::new(node_id(from)), NamedNode::new(node_id(to)))
        else {
            continue;
        };
        for statement in payload
            .assertions()
            .iter()
            .filter(|statement| statement.withdrawn.is_some())
        {
            push_semantic_statement_metadata_quads(
                &mut quads,
                &subject,
                &statement.predicate,
                target.clone().into(),
                statement,
                GraphName::NamedNode(history_graph.clone()),
                true,
            );
        }
    }
    for (_, from, to, payload) in graph.resource_relations() {
        let (Some(from), Some(to)) = (graph.resource(from), graph.resource(to)) else {
            continue;
        };
        let (Ok(subject), Ok(target)) = (
            NamedNode::new(from.canonical_iri()),
            NamedNode::new(to.canonical_iri()),
        ) else {
            continue;
        };
        for statement in payload
            .assertions()
            .iter()
            .filter(|statement| statement.withdrawn.is_some())
        {
            push_semantic_statement_metadata_quads(
                &mut quads,
                &subject,
                &statement.predicate,
                target.clone().into(),
                statement,
                GraphName::NamedNode(history_graph.clone()),
                true,
            );
        }
    }
    let mut seen = std::collections::HashSet::new();
    quads.retain(|quad| seen.insert(quad.clone()));
    quads
}

fn resource_edge_quads(
    from: &kernel::graph::resource::ResourceNode,
    to: &kernel::graph::resource::ResourceNode,
    payload: &kernel::graph::EdgePayload,
) -> Vec<Quad> {
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

pub(crate) fn dataset_quad_iter(graph: &Graph) -> impl Iterator<Item = Quad> + '_ {
    graph
        .nodes()
        .flat_map(|(key, node)| node_dataset_quads(graph, key, node))
        .chain(
            graph
                .resource_nodes()
                .flat_map(|resource| resource_metadata::quads(graph, resource)),
        )
        .chain(
            graph
                .resource_edges()
                .flat_map(|(from, to, payload)| resource_edge_quads(from, to, payload)),
        )
}

/// Export the complete RDF projection under an inline context.
/// Curated and recognized predicates use short terms; open predicates retain IRIs.
pub fn to_jsonld_compact(graph: &Graph) -> Value {
    jsonld::export(graph, true)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[path = "statement_metadata.rs"]
mod statement_metadata;

use statement_metadata::{push_semantic_statement_metadata_quads, push_statement_metadata_quads};

use statement_metadata::property_literal;
#[cfg(test)]
use statement_metadata::{asserted_at_literal, withdrawal_activity_id};
