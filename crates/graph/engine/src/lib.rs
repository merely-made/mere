// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! # mere
//!
//! The Mere engine, stage modules over the linked-data code (engine split P5 L2)
//! and the model it projects (P5 L3): [`model`] holds the assertion model and the
//! [`ResourceGraph`]. A graph that projects to RDF implements [`RdfSource`]; the
//! resource graph does, and `mere-kernel`'s surface graph does by composing its
//! surface walk with the resource projection here.
//!
//! **Export.** [`to_jsonld`] emits *expanded* JSON-LD (full IRIs, no `@context`);
//! [`to_jsonld_compact`] emits the *compacted* form (a `@graph` under an inline
//! `@context`). Each held exact assertion becomes an RDF predicate and reifier,
//! including semantic, containment, arrangement, imported, and provenance
//! relation kinds. Recognized relation kinds use their canonical Mere IRI;
//! open predicates pass through verbatim.
//!
//! **Ingest.** [`from_jsonld`] parses a document into a [`GraphContribution`]
//! (via `oxjsonld`); [`from_jsonld_with_contexts`] resolves a remote `@context`
//! from a bundled [`ContextCache`] rather than the network. Applying a
//! contribution to a surface graph is the kernel's (`kernel::rdf`).
//!
//! Hosts parse HTML and dispatch preserved `application/ld+json` blocks into
//! these JSON-LD entry points. `mere-document-lanes` owns that Fleece adapter;
//! this crate remains an RDF processor rather than a second HTML parser.
//!
//! Later: `@type` ↔ classification mapping.

#![doc(html_root_url = "https://docs.rs/mere/0.1.0")]

use model::{GraphScope, NodeProperty, SemanticStatement};
use oxrdf::{GraphName, Literal, NamedNode, Quad, Term, Triple};
use serde_json::{Map, Value};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub mod reifier;
mod resource_metadata;

use reifier::statement_reifier_id;

/// Model stage: the assertion model and the resource graph.
pub mod model;

/// The RDF projection: the source trait a graph implements and the resource
/// half of the projection.
pub mod projection;

pub use model::ResourceGraph;
pub use projection::{ProjectionKey, ProjectionSource, RdfSource, SourceQuads};

/// Acquire stage: JSON-LD ingest and the offline `@context` loader.
pub mod acquire;

/// The held dataset: dictionary-encoded quad index, `View`, and `IndexedRead`.
pub mod dataset;

/// Formats stage: Turtle-family and JSON-LD parsers and serializers.
pub mod formats;

/// Profile stage: standard-vocabulary alignment.
pub mod profile;

/// SPARQL query over the graph via spareval evaluating directly over the
/// borrowed RDF projection (the `query` feature).
#[cfg(feature = "query")]
pub mod query;

pub use acquire::ingest::{
    ContextCache, EdgeContribution, GraphContribution, ImportEnvelope, IngestError,
    NodeContribution, SubjectIdentity, from_jsonld, from_jsonld_envelope,
    from_jsonld_envelope_with_contexts, from_jsonld_envelope_with_contexts_and_base_iri,
    from_jsonld_with_contexts, from_jsonld_with_contexts_and_base_iri, from_quads,
    from_quads_envelope, is_bundled_context, referenced_context_urls,
};
pub use formats::parse::{
    JsonLdOptions, ProcessingMode, parse_jsonld, parse_nquads, parse_ntriples, parse_rdfxml,
    parse_trig, parse_turtle,
};
pub use formats::serialize::{
    from_nquads, from_nquads_envelope, from_trig, from_trig_envelope, to_nquads,
    to_nquads_with_history, to_trig, to_trig_with_history,
};
pub use profile::vocab::{Alignment, alignment, vocabulary_alignment_quads};

/// `schema:name` — the curated mapping target for a node's title.
pub const SCHEMA_NAME: &str = "https://schema.org/name";
/// `schema:keywords` — the curated mapping target for a node's tags.
pub const SCHEMA_KEYWORDS: &str = "https://schema.org/keywords";
/// `rdf:type` — the predicate JSON-LD `@type` expands to.
pub const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
pub const RDF_REIFIES: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies";
pub const RDFS_LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";
pub const PROV_WAS_ATTRIBUTED_TO: &str = "http://www.w3.org/ns/prov#wasAttributedTo";
pub const PROV_GENERATED_AT_TIME: &str = "http://www.w3.org/ns/prov#generatedAtTime";
pub const PROV_INVALIDATED_AT_TIME: &str = "http://www.w3.org/ns/prov#invalidatedAtTime";
pub const PROV_WAS_INVALIDATED_BY: &str = "http://www.w3.org/ns/prov#wasInvalidatedBy";
pub const PROV_WAS_ASSOCIATED_WITH: &str = "http://www.w3.org/ns/prov#wasAssociatedWith";
pub const PROV_WAS_GENERATED_BY: &str = "http://www.w3.org/ns/prov#wasGeneratedBy";
pub const PROV_WAS_DERIVED_FROM: &str = "http://www.w3.org/ns/prov#wasDerivedFrom";
pub const PROV_ACTIVITY: &str = "http://www.w3.org/ns/prov#Activity";
pub const MERE_LAST_ASSERTED_AT: &str = "https://mere.computer/ns/lastAssertedAt";
pub const MERE_METHOD_VERSION: &str = "https://mere.computer/ns/methodVersion";
pub const MERE_LIFECYCLE_VERSION: &str = "https://mere.computer/ns/assertionLifecycleVersion";
pub const MERE_ASSERTED_GRAPH_KIND: &str = "https://mere.computer/ns/assertedGraphKind";
pub const MERE_ASSERTED_GRAPH_NAME: &str = "https://mere.computer/ns/assertedGraphName";
/// Reserved named graph containing reifiers emitted by explicit history views.
pub const ASSERTION_HISTORY_GRAPH_IRI: &str = "https://mere.computer/ns/graph#history";
pub const WITHDRAWAL_ACTIVITY_PREFIX: &str = "urn:mere:withdrawal-activity:v1:";
pub const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
pub const XSD_DATETIME: &str = "http://www.w3.org/2001/XMLSchema#dateTime";
pub const GRAPH_SCOPE_SOURCE: &str = "https://mere.computer/ns/graph#source";
pub const GRAPH_SCOPE_USER: &str = "https://mere.computer/ns/graph#user";
pub const GRAPH_SCOPE_AGENT: &str = "https://mere.computer/ns/graph#agent";
pub const GRAPH_SCOPE_MOOT: &str = "https://mere.computer/ns/graph#moot";

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
pub fn to_jsonld<S: RdfSource + ?Sized>(graph: &S) -> Value {
    formats::jsonld::export(graph, false)
}

/// Export JSON-LD with retained withdrawal history in the explicit history graph.
/// Withdrawn fact triples are reified there and are never asserted as base triples.
pub fn to_jsonld_with_history<S: RdfSource + ?Sized>(graph: &S) -> Value {
    formats::jsonld::export_with_projection(graph, false, AssertionProjection::History)
}

/// Pretty-printed [`to_jsonld`], for goldens and human inspection.
pub fn to_jsonld_string<S: RdfSource + ?Sized>(graph: &S) -> String {
    serde_json::to_string_pretty(&to_jsonld(graph))
        .expect("a JSON-LD document of strings is always serializable")
}

/// The named graph a scope projects to, when its IRI is valid.
pub fn graph_name_for_scope(scope: &GraphScope) -> Option<GraphName> {
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
pub fn push_quad(
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

/// The RDF dataset quads for the whole graph, including named-graph scoped
/// semantic statements and node properties plus dataset-only reifier metadata.
pub fn dataset_quads<S: RdfSource + ?Sized>(graph: &S) -> Vec<Quad> {
    let mut seen = std::collections::HashSet::new();
    graph
        .held_quads()
        .filter(|quad| seen.insert(quad.clone()))
        .collect()
}

/// Explicit history projection: held assertions remain in their original
/// graph, while withdrawn assertions appear only as reifiers in the reserved
/// history graph with their original `GraphScope` carried separately.
pub fn dataset_quads_with_history<S: RdfSource + ?Sized>(graph: &S) -> Vec<Quad> {
    let mut quads = dataset_quads(graph);
    quads.extend(graph.withdrawn_reifier_quads());
    let mut seen = std::collections::HashSet::new();
    quads.retain(|quad| seen.insert(quad.clone()));
    quads
}

/// Export the complete RDF projection under an inline context.
/// Curated and recognized predicates use short terms; open predicates retain IRIs.
pub fn to_jsonld_compact<S: RdfSource + ?Sized>(graph: &S) -> Value {
    formats::jsonld::export(graph, true)
}

#[path = "statement_metadata.rs"]
pub mod statement_metadata;

pub use statement_metadata::{
    asserted_at_literal, property_literal, push_semantic_statement_metadata_quads,
    push_statement_metadata_quads, withdrawal_activity_id,
};
