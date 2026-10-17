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

/// Statements-over-schema ingest (Phase 0): knot `rel` links → `Semantic`
/// edges. The apply half of inker's pure `link_statements` walk; relocated
/// here 2026-07-10 (inker-adoption plan) so inker stays kernel-free.
pub mod statements;

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
pub use statements::{StatementOutcome, apply_link_statements, resolve_rel};
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

fn property_literal(property: &NodeProperty) -> Literal {
    if let Some(lang) = property.lang.as_deref()
        && let Ok(literal) = Literal::new_language_tagged_literal(property.value.clone(), lang)
    {
        return literal;
    }
    if let Some(datatype) = property.datatype.as_deref()
        && datatype != XSD_STRING
        && let Ok(datatype) = NamedNode::new(datatype)
    {
        return Literal::new_typed_literal(property.value.clone(), datatype);
    }
    Literal::new_simple_literal(property.value.clone())
}

fn asserted_at_literal(asserted_at_ms: u64) -> Option<Literal> {
    let timestamp =
        OffsetDateTime::from_unix_timestamp_nanos(i128::from(asserted_at_ms) * 1_000_000).ok()?;
    let lexical = timestamp.format(&Rfc3339).ok()?;
    let datatype = NamedNode::new(XSD_DATETIME).ok()?;
    Some(Literal::new_typed_literal(lexical, datatype))
}

#[allow(
    clippy::too_many_arguments,
    reason = "the helper mirrors the complete RDF statement metadata tuple"
)]
fn push_statement_metadata_quads(
    quads: &mut Vec<Quad>,
    subject: &NamedNode,
    statement_id: &str,
    predicate: &str,
    object: Term,
    graph_scope: &GraphScope,
    label: Option<&str>,
    provenance_iri: Option<&str>,
    asserted_at_ms: Option<u64>,
) {
    let Ok(reifier) = NamedNode::new(statement_reifier_id(statement_id)) else {
        return;
    };
    let (Ok(predicate_node), Some(graph_name), Ok(rdf_reifies)) = (
        NamedNode::new(predicate),
        graph_name_for_scope(graph_scope),
        NamedNode::new(RDF_REIFIES),
    ) else {
        return;
    };
    quads.push(Quad::new(
        reifier.clone(),
        rdf_reifies,
        Term::from(Triple::new(subject.clone(), predicate_node, object)),
        graph_name.clone(),
    ));
    if let Some(label) = label
        && let Ok(predicate) = NamedNode::new(RDFS_LABEL)
    {
        quads.push(Quad::new(
            reifier.clone(),
            predicate,
            Term::from(Literal::new_simple_literal(label)),
            graph_name.clone(),
        ));
    }
    if let Some(provenance_iri) = provenance_iri
        && let (Ok(predicate), Ok(agent)) = (
            NamedNode::new(PROV_WAS_ATTRIBUTED_TO),
            NamedNode::new(provenance_iri),
        )
    {
        quads.push(Quad::new(
            reifier.clone(),
            predicate,
            Term::from(agent),
            graph_name.clone(),
        ));
    }
    if let Some(asserted_at_ms) = asserted_at_ms
        && let (Ok(predicate), Some(literal)) = (
            NamedNode::new(PROV_GENERATED_AT_TIME),
            asserted_at_literal(asserted_at_ms),
        )
    {
        quads.push(Quad::new(
            reifier,
            predicate,
            Term::from(literal),
            graph_name,
        ));
    }
}

fn push_semantic_statement_metadata_quads(
    quads: &mut Vec<Quad>,
    subject: &NamedNode,
    predicate: &str,
    object: Term,
    statement: &SemanticStatement,
    graph_name: GraphName,
    include_original_scope: bool,
) {
    let Ok(reifier) = NamedNode::new(statement_reifier_id(&statement.statement_id)) else {
        return;
    };
    let Ok(predicate_node) = NamedNode::new(predicate) else {
        return;
    };
    let Ok(rdf_reifies) = NamedNode::new(RDF_REIFIES) else {
        return;
    };

    quads.push(Quad::new(
        reifier.clone(),
        rdf_reifies,
        Term::from(Triple::new(subject.clone(), predicate_node, object)),
        graph_name.clone(),
    ));

    if statement.first_asserted_ms != statement.asserted_at_ms {
        push_reifier_literal(
            quads,
            &reifier,
            MERE_LIFECYCLE_VERSION,
            "1".into(),
            &graph_name,
        );
    }

    if include_original_scope {
        let (kind, name) = graph_scope_marker(&statement.graph_scope);
        push_reifier_literal(
            quads,
            &reifier,
            MERE_ASSERTED_GRAPH_KIND,
            kind.into(),
            &graph_name,
        );
        if let Some(name) = name {
            push_reifier_literal(quads, &reifier, MERE_ASSERTED_GRAPH_NAME, name, &graph_name);
        }
    }

    if let Some(label) = statement.basis.note.as_deref()
        && let Ok(predicate) = NamedNode::new(RDFS_LABEL)
    {
        quads.push(Quad::new(
            reifier.clone(),
            predicate,
            Term::from(Literal::new_simple_literal(label)),
            graph_name.clone(),
        ));
    }
    if let Some(provenance_iri) = statement.provenance_iri.as_deref()
        && let (Ok(predicate), Ok(agent)) = (
            NamedNode::new(PROV_WAS_ATTRIBUTED_TO),
            NamedNode::new(provenance_iri),
        )
    {
        quads.push(Quad::new(
            reifier.clone(),
            predicate,
            Term::from(agent),
            graph_name.clone(),
        ));
    }
    if let Some(first_asserted_ms) = statement.first_asserted_ms
        && let (Ok(predicate), Some(literal)) = (
            NamedNode::new(PROV_GENERATED_AT_TIME),
            asserted_at_literal(first_asserted_ms),
        )
    {
        quads.push(Quad::new(
            reifier.clone(),
            predicate,
            Term::from(literal),
            graph_name.clone(),
        ));
    }
    if let Some(last_asserted_ms) = statement.asserted_at_ms
        && let (Ok(predicate), Some(literal)) = (
            NamedNode::new(MERE_LAST_ASSERTED_AT),
            asserted_at_literal(last_asserted_ms),
        )
    {
        quads.push(Quad::new(
            reifier.clone(),
            predicate,
            Term::from(literal),
            graph_name.clone(),
        ));
    }
    if let Some(method) = &statement.basis.method {
        if let (Ok(predicate), Ok(method_iri)) = (
            NamedNode::new(PROV_WAS_GENERATED_BY),
            NamedNode::new(method.iri.clone()),
        ) {
            quads.push(Quad::new(
                reifier.clone(),
                predicate,
                Term::from(method_iri),
                graph_name.clone(),
            ));
        }
        if let Some(version) = &method.version {
            push_reifier_literal(
                quads,
                &reifier,
                MERE_METHOD_VERSION,
                version.clone(),
                &graph_name,
            );
        }
    }
    for evidence in &statement.basis.evidence {
        if let (Ok(predicate), Ok(evidence)) = (
            NamedNode::new(PROV_WAS_DERIVED_FROM),
            NamedNode::new(evidence.clone()),
        ) {
            quads.push(Quad::new(
                reifier.clone(),
                predicate,
                Term::from(evidence),
                graph_name.clone(),
            ));
        }
    }
    if let Some(withdrawal) = &statement.withdrawn {
        if let Ok(predicate) = NamedNode::new(PROV_WAS_INVALIDATED_BY)
            && let Ok(activity) = NamedNode::new(withdrawal_activity_id(&statement.statement_id))
        {
            quads.push(Quad::new(
                reifier.clone(),
                predicate,
                Term::from(activity.clone()),
                graph_name.clone(),
            ));
            if let Ok(predicate) = NamedNode::new(RDF_TYPE) {
                quads.push(Quad::new(
                    activity.clone(),
                    predicate,
                    Term::from(NamedNode::new_unchecked(PROV_ACTIVITY)),
                    graph_name.clone(),
                ));
            }
            if let Some(by) = &withdrawal.by
                && let (Ok(predicate), Ok(agent)) = (
                    NamedNode::new(PROV_WAS_ASSOCIATED_WITH),
                    NamedNode::new(by.clone()),
                )
            {
                quads.push(Quad::new(
                    activity,
                    predicate,
                    Term::from(agent),
                    graph_name.clone(),
                ));
            }
        }
        if let Some(at_ms) = withdrawal.at_ms
            && let (Ok(predicate), Some(literal)) = (
                NamedNode::new(PROV_INVALIDATED_AT_TIME),
                asserted_at_literal(at_ms),
            )
        {
            quads.push(Quad::new(
                reifier,
                predicate,
                Term::from(literal),
                graph_name,
            ));
        }
    }
}

fn push_reifier_literal(
    quads: &mut Vec<Quad>,
    reifier: &NamedNode,
    predicate: &str,
    value: String,
    graph_name: &GraphName,
) {
    if let Ok(predicate) = NamedNode::new(predicate) {
        quads.push(Quad::new(
            reifier.clone(),
            predicate,
            Term::from(Literal::new_simple_literal(value)),
            graph_name.clone(),
        ));
    }
}

fn withdrawal_activity_id(statement_id: &str) -> String {
    let mut encoded = String::from(WITHDRAWAL_ACTIVITY_PREFIX);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in statement_id.bytes() {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 15)]));
    }
    encoded
}

fn graph_scope_marker(scope: &GraphScope) -> (&'static str, Option<String>) {
    match scope {
        GraphScope::Default => ("default", None),
        GraphScope::Source => ("source", None),
        GraphScope::User => ("user", None),
        GraphScope::Agent => ("agent", None),
        GraphScope::Moot => ("moot", None),
        GraphScope::Custom(name) => ("custom", Some(name.clone())),
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
mod tests {
    #[cfg(feature = "query")]
    use super::node_quads;
    use super::{
        ASSERTION_HISTORY_GRAPH_IRI, GraphContribution, MERE_ASSERTED_GRAPH_KIND,
        MERE_ASSERTED_GRAPH_NAME, MERE_LAST_ASSERTED_AT, MERE_LIFECYCLE_VERSION,
        MERE_METHOD_VERSION, PROV_GENERATED_AT_TIME, PROV_INVALIDATED_AT_TIME,
        PROV_WAS_ASSOCIATED_WITH, PROV_WAS_ATTRIBUTED_TO, PROV_WAS_DERIVED_FROM,
        PROV_WAS_GENERATED_BY, PROV_WAS_INVALIDATED_BY, RDF_REIFIES, RDFS_LABEL,
        apply_contribution, apply_import, dataset_quads_with_history, from_jsonld,
        from_nquads_envelope, node_id, statement_reifier_id, to_jsonld, to_jsonld_compact,
        to_jsonld_with_history, to_nquads_with_history, to_trig_with_history,
        withdrawal_activity_id,
    };
    use super::{dataset_quads, node_direct_quads};
    use crate::from_trig_envelope;
    use kernel::graph::fixtures::GraphFixtures;
    use kernel::graph::{EdgeAssertion, Graph, SemanticSubKind};
    use kernel::types::{GraphScope, NodeProperty};
    use oxrdf::{GraphName, NamedNode, Term};
    use serde_json::json;

    #[test]
    fn type_export_uses_affirmative_statuses_and_retains_review_records() {
        use kernel::types::{
            ClassificationProvenance, ClassificationScheme, ClassificationStatus,
            NodeClassification,
        };

        for (status, exported) in [
            (ClassificationStatus::Accepted, true),
            (ClassificationStatus::Verified, true),
            (ClassificationStatus::Imported, true),
            (ClassificationStatus::Suggested, false),
            (ClassificationStatus::Rejected, false),
        ] {
            let mut graph = Graph::new();
            let key = graph.add_node("https://classification.test".into(), Default::default());
            graph.get_node_mut(key).unwrap().title = "Classification control".into();
            let record = NodeClassification {
                scheme: ClassificationScheme::Custom("rdf:type".into()),
                value: "https://schema.org/Article".into(),
                label: Some("Article".into()),
                confidence: 0.8,
                provenance: ClassificationProvenance::UserAuthored,
                status,
                primary: true,
            };
            assert!(graph.add_node_classifications(key, vec![record.clone()]));
            let types: Vec<_> = super::dataset_quads(&graph)
                .into_iter()
                .filter(|quad| quad.predicate.as_str() == super::RDF_TYPE)
                .collect();
            assert_eq!(types.len(), usize::from(exported), "{record:?}");
            if exported {
                assert_eq!(types[0].object.to_string(), "<https://schema.org/Article>");
            }
            let exported_bytes = serde_json::to_vec(&to_jsonld(&graph)).unwrap();
            let contribution = from_jsonld(&exported_bytes).expect("JSON-LD export reingests");
            let node = contribution
                .nodes
                .iter()
                .find(|node| node.id == "https://classification.test")
                .expect("exported node remains");
            assert_eq!(node.title.as_deref(), Some("Classification control"));
            assert_eq!(node.types.contains(&record.value), exported, "{record:?}");
            #[cfg(feature = "query")]
            {
                let rows = super::query::sparql(
                    &graph,
                    "SELECT ?type WHERE { <https://classification.test> a ?type }",
                )
                .expect("classification query");
                assert_eq!(rows.rows.len(), usize::from(exported), "{record:?}");
                if exported {
                    assert_eq!(rows.rows[0], vec![Some(record.value.clone())]);
                }
            }
            assert_eq!(graph.node_classifications(key).unwrap(), vec![record]);
        }
    }

    /// A graph with one recognized edge (A cites B, canonical IRI), one raw
    /// open-predicate edge (A → C, `schema:citation`), and curated literals on A.
    fn seed() -> Graph {
        let mut graph = Graph::new();
        let a = graph.add_node("https://a.test".to_string(), Default::default());
        let b = graph.add_node("https://b.test".to_string(), Default::default());
        let c = graph.add_node("https://c.test".to_string(), Default::default());

        graph.assert_relation(
            a,
            b,
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Cites,
                label: None,
                decay_progress: None,
            },
        );
        let e = graph
            .assert_relation(
                a,
                c,
                EdgeAssertion::Semantic {
                    sub_kind: SemanticSubKind::Cites,
                    label: None,
                    decay_progress: None,
                },
            )
            .expect("edge");
        graph
            .get_relation_mut(e)
            .expect("payload")
            .set_semantic_predicate(Some("https://schema.org/citation".to_string()));

        let node_a = graph.get_node_mut(a).expect("node a");
        node_a.title = "Article A".to_string();
        node_a.tags.insert("research".to_string());
        graph
    }

    #[test]
    fn node_direct_quads_export_each_outgoing_parallel_bucket_without_reversing_incoming() {
        let mut source = Graph::new();
        let a = source.add_node("https://a.test".to_string(), Default::default());
        let b = source.add_node("https://b.test".to_string(), Default::default());

        source.assert_relation(
            a,
            b,
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::UserGrouped,
                label: None,
                decay_progress: None,
            },
        );
        source.assert_relation(
            b,
            a,
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::UserGrouped,
                label: None,
                decay_progress: None,
            },
        );

        let mut snapshot = source.to_snapshot();
        let mut parallel = snapshot
            .edges
            .iter()
            .find(|edge| edge.from_node_id == source.get_node(a).unwrap().id.to_string())
            .expect("outgoing source bucket")
            .clone();
        let statement = parallel
            .retained_assertions_mut()
            .first_mut()
            .expect("persisted outgoing assertion");
        statement.statement_id = "parallel-outgoing-assertion".into();
        statement.predicate = "https://example.test/parallel-outgoing".into();
        statement.recognized_sub_kind = None;
        statement.graph_scope = GraphScope::Custom("urn:graph:parallel".into());
        snapshot.edges.push(parallel);

        let incoming = snapshot
            .edges
            .iter_mut()
            .find(|edge| edge.from_node_id == source.get_node(b).unwrap().id.to_string())
            .expect("incoming-only bucket")
            .retained_assertions_mut()
            .first_mut()
            .expect("persisted incoming assertion");
        incoming.statement_id = "incoming-only-assertion".into();
        incoming.predicate = "https://example.test/incoming-only".into();
        incoming.recognized_sub_kind = None;
        incoming.graph_scope = GraphScope::Custom("urn:graph:incoming".into());

        let graph = Graph::try_from_recorded_snapshot(&snapshot).expect("restore parallel buckets");
        let (a, node_a) = graph.get_node_by_url("https://a.test").expect("node A");
        let outgoing = super::node_direct_quads(&graph, a, node_a);
        let predicates: Vec<_> = outgoing
            .iter()
            .map(|quad| quad.predicate.as_str())
            .collect();

        assert!(
            predicates.contains(&kernel::graph::predicate_iri(SemanticSubKind::UserGrouped)),
            "outgoing predicates: {predicates:?}"
        );
        assert!(predicates.contains(&"https://example.test/parallel-outgoing"));
        assert!(!predicates.contains(&"https://example.test/incoming-only"));

        let a_id = oxrdf::NamedNode::new("https://a.test").unwrap();
        let b_id = oxrdf::NamedNode::new("https://b.test").unwrap();
        let parallel_graph =
            oxrdf::GraphName::NamedNode(oxrdf::NamedNode::new("urn:graph:parallel").unwrap());
        let incoming_graph =
            oxrdf::GraphName::NamedNode(oxrdf::NamedNode::new("urn:graph:incoming").unwrap());
        let dataset = super::dataset_quads(&graph);
        assert!(dataset.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(a_id.clone())
                && quad.predicate.as_str() == "https://example.test/parallel-outgoing"
                && quad.object == oxrdf::Term::from(b_id.clone())
                && quad.graph_name == parallel_graph
        }));
        assert!(dataset.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(b_id.clone())
                && quad.predicate.as_str() == "https://example.test/incoming-only"
                && quad.object == oxrdf::Term::from(a_id.clone())
                && quad.graph_name == incoming_graph
        }));
        assert!(!dataset.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(a_id.clone())
                && quad.predicate.as_str() == "https://example.test/incoming-only"
                && quad.object == oxrdf::Term::from(b_id.clone())
        }));
    }

    #[test]
    fn default_export_omits_withdrawn_surface_and_resource_assertions() {
        use kernel::graph::{RelationKey, SemanticStatementSpec, predicate_iri};

        let mut graph = Graph::new();
        let a = graph.add_node("https://a.test".to_string(), Default::default());
        let b = graph.add_node("https://b.test".to_string(), Default::default());
        let surface_predicate = predicate_iri(SemanticSubKind::UserGrouped);
        let (_, surface) = graph
            .try_assert_semantic_statement(
                a,
                b,
                SemanticStatementSpec {
                    predicate: surface_predicate.into(),
                    recognized_sub_kind: Some(SemanticSubKind::UserGrouped),
                    provenance_iri: Some("urn:author:surface".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        let resource_predicate = predicate_iri(SemanticSubKind::Cites);
        let (resource_owner, resource) = graph
            .try_assert_semantic_statement(
                a,
                b,
                SemanticStatementSpec {
                    predicate: resource_predicate.into(),
                    recognized_sub_kind: Some(SemanticSubKind::Cites),
                    provenance_iri: Some("urn:author:resource".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(matches!(resource_owner, RelationKey::Resource(_)));

        assert!(graph.withdraw_semantic_assertion(
            &surface.statement_id,
            Some(10),
            Some("urn:author:surface".into()),
        ));
        assert!(graph.withdraw_semantic_assertion(
            &resource.statement_id,
            Some(11),
            Some("urn:author:resource".into()),
        ));

        let (a, node_a) = graph.get_node_by_url("https://a.test").expect("node A");
        let direct = node_direct_quads(&graph, a, node_a);
        assert!(
            !direct
                .iter()
                .any(|quad| quad.predicate.as_str() == surface_predicate)
        );
        let dataset = dataset_quads(&graph);
        assert!(!dataset.iter().any(|quad| {
            quad.predicate.as_str() == surface_predicate
                || quad.predicate.as_str() == resource_predicate
        }));
        let surface_reifier = NamedNode::new(statement_reifier_id(&surface.statement_id)).unwrap();
        assert!(dataset.iter().all(|quad| {
            quad.subject != oxrdf::NamedOrBlankNode::from(surface_reifier.clone())
        }));
        assert!(dataset.iter().all(|quad| {
            !matches!(&quad.object, Term::Triple(triple)
                if triple.predicate.as_str() == surface_predicate
                    || triple.predicate.as_str() == resource_predicate)
        }));
        assert!(
            dataset
                .iter()
                .all(|quad| quad.predicate.as_str() != RDF_REIFIES)
        );

        let history = dataset_quads_with_history(&graph);
        let history_graph =
            GraphName::NamedNode(NamedNode::new(ASSERTION_HISTORY_GRAPH_IRI).unwrap());
        let resource_reifier =
            NamedNode::new(statement_reifier_id(&resource.statement_id)).unwrap();
        for (reifier, expected_predicate) in [
            (surface_reifier.clone(), surface_predicate),
            (resource_reifier.clone(), resource_predicate),
        ] {
            assert!(history.iter().any(|quad| {
                quad.subject == oxrdf::NamedOrBlankNode::from(reifier.clone())
                    && quad.predicate.as_str() == RDF_REIFIES
                    && quad.graph_name == history_graph
                    && matches!(&quad.object, Term::Triple(triple)
                        if triple.predicate.as_str() == expected_predicate)
            }));
            assert!(history.iter().all(|quad| {
                quad.subject != oxrdf::NamedOrBlankNode::from(reifier.clone())
                    || quad.graph_name == history_graph
            }));
        }
    }

    #[test]
    fn all_family_assertions_export_distinct_reifiers_and_withdrawal_history() {
        use kernel::graph::{ContainmentSubKind, ImportedSubKind, RelationKind, SemanticStatement};
        use kernel::types::{AssertionBasis, AssertionMethod};

        let mut graph = Graph::new();
        let from = graph.add_node("https://family.test/from".into(), Default::default());
        let to = graph.add_node("https://family.test/to".into(), Default::default());
        let predicate = RelationKind::Containment(ContainmentSubKind::CollectionMember)
            .predicate_iri()
            .expect("canonical containment predicate")
            .to_owned();
        let actor = "urn:family:actor";
        let make_statement =
            |id: &str, note: &str, method: &str, evidence: &str, at| SemanticStatement {
                statement_id: id.into(),
                predicate: predicate.clone(),
                recognized_sub_kind: None,
                label: Some(note.into()),
                basis: AssertionBasis {
                    note: Some(note.into()),
                    method: Some(AssertionMethod {
                        iri: method.into(),
                        version: Some("1".into()),
                    }),
                    evidence: vec![evidence.into()],
                },
                graph_scope: GraphScope::Custom("urn:family:scope".into()),
                provenance_iri: Some(actor.into()),
                first_asserted_ms: Some(at),
                asserted_at_ms: Some(at),
                withdrawn: None,
            };
        for statement in [
            make_statement(
                "family:primary",
                "primary reason",
                "urn:method:primary",
                "urn:evidence:primary",
                10,
            ),
            make_statement(
                "family:independent",
                "independent reason",
                "urn:method:independent",
                "urn:evidence:independent",
                11,
            ),
            make_statement(
                "family:withdrawn",
                "withdrawn reason",
                "urn:method:withdrawn",
                "urn:evidence:withdrawn",
                12,
            ),
        ] {
            graph
                .try_assert_persisted_relation_statement(from, to, statement)
                .expect("distinct exact family assertions are retained");
        }
        let resource_id = "family:resource-control";
        let resource_kind = RelationKind::Imported(ImportedSubKind::SharedCollection);
        graph
            .try_assert_persisted_relation_statement(
                from,
                to,
                SemanticStatement {
                    statement_id: resource_id.into(),
                    predicate: resource_kind
                        .predicate_iri()
                        .expect("canonical imported predicate")
                        .into(),
                    recognized_sub_kind: None,
                    label: Some("resource family control".into()),
                    basis: AssertionBasis {
                        note: Some("resource family control".into()),
                        method: None,
                        evidence: vec!["urn:evidence:resource-family".into()],
                    },
                    graph_scope: GraphScope::Custom("urn:family:scope".into()),
                    provenance_iri: Some(actor.into()),
                    first_asserted_ms: Some(13),
                    asserted_at_ms: Some(13),
                    withdrawn: None,
                },
            )
            .expect("exact resource-family assertion is retained");
        assert!(graph.withdraw_semantic_assertion(
            "family:withdrawn",
            Some(20),
            Some("urn:family:withdrawer".into()),
        ));

        let held = dataset_quads(&graph);
        let expanded = to_jsonld(&graph).to_string();
        assert!(expanded.contains(&statement_reifier_id("family:primary")));
        assert!(expanded.contains(&statement_reifier_id("family:independent")));
        assert!(expanded.contains(&statement_reifier_id(resource_id)));
        assert!(!expanded.contains(&statement_reifier_id("family:withdrawn")));
        #[cfg(feature = "query")]
        {
            let rows = super::query::sparql(
                &graph,
                &format!(
                    "SELECT ?target WHERE {{ GRAPH <urn:family:scope> {{ <https://family.test/from> <{predicate}> ?target }} }}"
                ),
            )
            .expect("SPARQL reads the canonical family assertion projection");
            assert_eq!(rows.rows, vec![vec![Some("https://family.test/to".into())]]);
        }
        let scope = GraphName::NamedNode(NamedNode::new("urn:family:scope").unwrap());
        let from_id = NamedNode::new("https://family.test/from").unwrap();
        let to_id = NamedNode::new("https://family.test/to").unwrap();
        assert!(held.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(from_id.clone())
                && quad.predicate.as_str() == predicate
                && quad.object == Term::NamedNode(to_id.clone())
                && quad.graph_name == scope
        }));
        // RDF facts are a set, while independent source reasons retain one
        // reifier each with their own immutable IDs and metadata.
        for (id, note, method, evidence) in [
            (
                "family:primary",
                "primary reason",
                "urn:method:primary",
                "urn:evidence:primary",
            ),
            (
                "family:independent",
                "independent reason",
                "urn:method:independent",
                "urn:evidence:independent",
            ),
        ] {
            let reifier = NamedNode::new(statement_reifier_id(id)).unwrap();
            assert!(held.iter().any(|quad| {
                quad.subject == oxrdf::NamedOrBlankNode::from(reifier.clone())
                    && quad.predicate.as_str() == RDF_REIFIES
            }));
            assert!(held.iter().any(|quad| {
                quad.subject == oxrdf::NamedOrBlankNode::from(reifier.clone())
                    && quad.predicate.as_str() == RDFS_LABEL
                    && matches!(&quad.object, Term::Literal(value) if value.value() == note)
            }));
            assert!(held.iter().any(|quad| {
                quad.subject == oxrdf::NamedOrBlankNode::from(reifier.clone())
                    && quad.predicate.as_str() == PROV_WAS_GENERATED_BY
                    && quad.object == Term::NamedNode(NamedNode::new(method).unwrap())
            }));
            assert!(held.iter().any(|quad| {
                quad.subject == oxrdf::NamedOrBlankNode::from(reifier.clone())
                    && quad.predicate.as_str() == MERE_METHOD_VERSION
                    && matches!(&quad.object, Term::Literal(value) if value.value() == "1")
            }));
            assert!(held.iter().any(|quad| {
                quad.subject == oxrdf::NamedOrBlankNode::from(reifier.clone())
                    && quad.predicate.as_str() == PROV_WAS_DERIVED_FROM
                    && quad.object == Term::NamedNode(NamedNode::new(evidence).unwrap())
            }));
            assert!(held.iter().any(|quad| {
                quad.subject == oxrdf::NamedOrBlankNode::from(reifier.clone())
                    && quad.predicate.as_str() == PROV_WAS_ATTRIBUTED_TO
                    && quad.object == Term::NamedNode(NamedNode::new(actor).unwrap())
            }));
        }
        let resource_reifier = NamedNode::new(statement_reifier_id(resource_id)).unwrap();
        let resource_predicate = resource_kind.predicate_iri().unwrap();
        assert!(held.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(from_id.clone())
                && quad.predicate.as_str() == resource_predicate
                && quad.object == Term::NamedNode(to_id.clone())
                && quad.graph_name == scope
        }));
        assert!(held.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(resource_reifier.clone())
                && quad.predicate.as_str() == RDF_REIFIES
        }));
        assert!(held.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(resource_reifier.clone())
                && quad.predicate.as_str() == PROV_WAS_ATTRIBUTED_TO
                && quad.object == Term::NamedNode(NamedNode::new(actor).unwrap())
        }));
        let withdrawn_reifier = NamedNode::new(statement_reifier_id("family:withdrawn")).unwrap();
        assert!(held.iter().all(|quad| {
            quad.subject != oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
        }));

        let history = dataset_quads_with_history(&graph);
        assert!(
            to_jsonld_with_history(&graph)
                .to_string()
                .contains(&statement_reifier_id("family:withdrawn"))
        );
        let history_graph =
            GraphName::NamedNode(NamedNode::new(ASSERTION_HISTORY_GRAPH_IRI).unwrap());
        assert!(history.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
                && quad.predicate.as_str() == RDF_REIFIES
                && quad.graph_name == history_graph
                && matches!(&quad.object, Term::Triple(triple)
                    if triple.predicate.as_str() == predicate)
        }));
        let activity = NamedNode::new(withdrawal_activity_id("family:withdrawn")).unwrap();
        assert!(history.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
                && quad.predicate.as_str() == PROV_WAS_INVALIDATED_BY
                && quad.object == Term::NamedNode(activity.clone())
                && quad.graph_name == history_graph
        }));
        assert!(history.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
                && quad.predicate.as_str() == PROV_WAS_ATTRIBUTED_TO
                && quad.object == Term::NamedNode(NamedNode::new(actor).unwrap())
                && quad.graph_name == history_graph
        }));
        assert!(history.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
                && quad.predicate.as_str() == PROV_INVALIDATED_AT_TIME
                && quad.graph_name == history_graph
        }));
        assert!(history.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(activity.clone())
                && quad.predicate.as_str() == PROV_WAS_ASSOCIATED_WITH
                && quad.object == Term::NamedNode(NamedNode::new("urn:family:withdrawer").unwrap())
                && quad.graph_name == history_graph
        }));
        assert!(history.iter().all(|quad| {
            !(quad.subject == oxrdf::NamedOrBlankNode::from(from_id.clone())
                && quad.predicate.as_str() == predicate
                && quad.object == Term::NamedNode(to_id.clone())
                && quad.graph_name == history_graph)
        }));
    }

    #[test]
    fn explicit_history_roundtrips_lifecycle_basis_and_original_scope() {
        use kernel::graph::{Author, SemanticStatement};
        use kernel::types::{AssertionBasis, AssertionMethod, AssertionWithdrawal};

        let mut graph = Graph::new();
        let from = graph.add_node("https://history.test/from".into(), Default::default());
        let to = graph.add_node("https://history.test/to".into(), Default::default());
        let predicate = kernel::graph::predicate_iri(SemanticSubKind::Cites).to_string();
        let id = "rdf:history/withdrawn:one";
        let basis = AssertionBasis {
            note: Some("checked against source".into()),
            method: Some(AssertionMethod {
                iri: "urn:method:extractor".into(),
                version: Some("2.1".into()),
            }),
            evidence: vec!["https://evidence.test/item".into()],
        };
        let withdrawn = SemanticStatement {
            statement_id: id.into(),
            predicate: predicate.clone(),
            recognized_sub_kind: Some(SemanticSubKind::Cites),
            label: basis.note.clone(),
            basis: basis.clone(),
            graph_scope: GraphScope::Custom("https://scope.test/original".into()),
            provenance_iri: Some("urn:author:asserted".into()),
            first_asserted_ms: Some(10),
            asserted_at_ms: Some(20),
            withdrawn: Some(AssertionWithdrawal {
                at_ms: Some(30),
                by: Some("urn:author:withdrew".into()),
            }),
        };
        graph.write_as(Author::person("urn:author:host"), |graph| {
            graph
                .try_assert_persisted_semantic_statement(from, to, withdrawn)
                .expect("store exact tombstone");
        });

        let undated_id = "history:undated-default";
        let undated_predicate = "urn:predicate:undated-history";
        let undated = SemanticStatement {
            statement_id: undated_id.into(),
            predicate: undated_predicate.into(),
            recognized_sub_kind: None,
            label: None,
            basis: AssertionBasis::default(),
            graph_scope: GraphScope::Default,
            provenance_iri: None,
            first_asserted_ms: None,
            asserted_at_ms: None,
            withdrawn: Some(AssertionWithdrawal {
                at_ms: None,
                by: None,
            }),
        };
        graph
            .try_assert_persisted_semantic_statement(from, to, undated)
            .expect("store undated default-scope tombstone");

        let held_id = "held:partial-time";
        let mut held = SemanticStatement::new(
            predicate.clone(),
            Some(SemanticSubKind::Cites),
            None,
            GraphScope::Default,
            Some("urn:author:held".into()),
            Some(42),
        );
        held.statement_id = held_id.into();
        held.first_asserted_ms = None;
        graph
            .try_assert_persisted_semantic_statement(from, to, held)
            .expect("store held partial-time control");

        let default_dataset = dataset_quads(&graph);
        let withdrawn_reifier = NamedNode::new(statement_reifier_id(id)).unwrap();
        let from_id = node_id(graph.get_node(from).unwrap());
        let to_id = node_id(graph.get_node(to).unwrap());
        assert!(default_dataset.iter().all(|quad| {
            quad.subject != oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
        }));
        assert!(default_dataset.iter().any(|quad| {
            quad.subject.to_string() == format!("<{from_id}>")
                && quad.predicate.as_str() == predicate
                && quad.object == Term::NamedNode(NamedNode::new_unchecked(to_id.clone()))
        }));
        let held_reifier = NamedNode::new(statement_reifier_id(held_id)).unwrap();
        assert!(default_dataset.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(held_reifier.clone())
                && quad.predicate.as_str() == MERE_LIFECYCLE_VERSION
                && matches!(&quad.object, Term::Literal(literal) if literal.value() == "1")
        }));

        let history = dataset_quads_with_history(&graph);
        let history_graph =
            GraphName::NamedNode(NamedNode::new(ASSERTION_HISTORY_GRAPH_IRI).unwrap());
        assert!(history.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
                && quad.predicate.as_str() == RDF_REIFIES
                && quad.graph_name == history_graph
        }));
        let undated_reifier = NamedNode::new(statement_reifier_id(undated_id)).unwrap();
        assert!(history.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(undated_reifier.clone())
                && quad.predicate.as_str() == MERE_ASSERTED_GRAPH_KIND
                && matches!(&quad.object, Term::Literal(literal) if literal.value() == "default")
        }));
        assert!(history.iter().all(|quad| {
            quad.subject != oxrdf::NamedOrBlankNode::from(undated_reifier.clone())
                || !matches!(
                    quad.predicate.as_str(),
                    PROV_GENERATED_AT_TIME | MERE_LAST_ASSERTED_AT | PROV_INVALIDATED_AT_TIME
                )
        }));
        assert!(history.iter().all(|quad| {
            !(quad.subject.to_string() == format!("<{from_id}>")
                && quad.predicate.as_str() == predicate
                && quad.object == Term::NamedNode(NamedNode::new_unchecked(to_id.clone()))
                && quad.graph_name
                    == GraphName::NamedNode(NamedNode::new("https://scope.test/original").unwrap()))
        }));
        assert!(history.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
                && quad.predicate.as_str() == MERE_ASSERTED_GRAPH_KIND
                && matches!(&quad.object, Term::Literal(literal) if literal.value() == "custom")
                && quad.graph_name == history_graph
        }));
        assert!(history.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
                && quad.predicate.as_str() == MERE_ASSERTED_GRAPH_NAME
                && matches!(&quad.object, Term::Literal(literal) if literal.value() == "https://scope.test/original")
                && quad.graph_name == history_graph
        }));
        let activity = NamedNode::new(withdrawal_activity_id(id)).unwrap();
        assert!(history.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
                && quad.predicate.as_str() == PROV_WAS_INVALIDATED_BY
                && quad.object == Term::NamedNode(activity.clone())
        }));
        assert!(history.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(activity.clone())
                && quad.predicate.as_str() == PROV_WAS_ASSOCIATED_WITH
                && quad.object == Term::NamedNode(NamedNode::new_unchecked("urn:author:withdrew"))
        }));
        assert!(history.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
                && quad.predicate.as_str() == MERE_METHOD_VERSION
                && matches!(&quad.object, Term::Literal(literal) if literal.value() == "2.1")
        }));

        let nquads = to_nquads_with_history(&graph);
        let envelope = from_nquads_envelope(&nquads, "history-nquads").unwrap();
        let mut restored = Graph::new();
        apply_import(&mut restored, &envelope);
        let (_, restored_tombstone) = restored.find_semantic_statement(id).unwrap();
        assert_eq!(restored_tombstone.first_asserted_ms, Some(10));
        assert_eq!(restored_tombstone.asserted_at_ms, Some(20));
        assert_eq!(
            restored_tombstone.withdrawn.as_ref().unwrap().at_ms,
            Some(30)
        );
        assert_eq!(
            restored_tombstone.withdrawn.as_ref().unwrap().by.as_deref(),
            Some("urn:author:withdrew")
        );
        assert_eq!(
            restored_tombstone.graph_scope,
            GraphScope::Custom("https://scope.test/original".into())
        );
        assert_eq!(restored_tombstone.basis, basis);
        let (_, restored_undated) = restored.find_semantic_statement(undated_id).unwrap();
        assert_eq!(restored_undated.first_asserted_ms, None);
        assert_eq!(restored_undated.asserted_at_ms, None);
        assert_eq!(restored_undated.withdrawn.as_ref().unwrap().at_ms, None);
        assert_eq!(restored_undated.withdrawn.as_ref().unwrap().by, None);
        assert_eq!(restored_undated.graph_scope, GraphScope::Default);
        let (_, restored_held) = restored.find_semantic_statement(held_id).unwrap();
        assert_eq!(restored_held.first_asserted_ms, None);
        assert_eq!(restored_held.asserted_at_ms, Some(42));

        let trig = to_trig_with_history(&graph).expect("serialize explicit history TriG");
        let envelope = from_trig_envelope(&trig, "history-trig").unwrap();
        let mut restored_trig = Graph::new();
        apply_import(&mut restored_trig, &envelope);
        let (_, restored_trig_tombstone) = restored_trig.find_semantic_statement(id).unwrap();
        assert_eq!(restored_trig_tombstone.first_asserted_ms, Some(10));
        assert_eq!(restored_trig_tombstone.asserted_at_ms, Some(20));
        assert_eq!(
            restored_trig_tombstone.withdrawn.as_ref().unwrap().at_ms,
            Some(30)
        );
        assert_eq!(
            restored_trig_tombstone.graph_scope,
            GraphScope::Custom("https://scope.test/original".into())
        );
        assert_eq!(restored_trig_tombstone.basis, basis);

        let jsonld = to_jsonld_with_history(&graph);
        let envelope =
            crate::from_jsonld_envelope(serde_json::to_string(&jsonld).unwrap().as_bytes())
                .unwrap();
        let mut restored_jsonld = Graph::new();
        apply_import(&mut restored_jsonld, &envelope);
        let (_, restored_tombstone) = restored_jsonld.find_semantic_statement(id).unwrap();
        assert_eq!(
            restored_tombstone.withdrawn.as_ref().unwrap().at_ms,
            Some(30)
        );
        assert_eq!(
            restored_tombstone.graph_scope,
            GraphScope::Custom("https://scope.test/original".into())
        );
    }

    fn assert_property_fields(
        property: &NodeProperty,
        predicate: &str,
        value: &str,
        datatype: Option<&str>,
        lang: Option<&str>,
        graph_scope: GraphScope,
    ) {
        assert_eq!(property.predicate, predicate);
        assert_eq!(property.value, value);
        assert_eq!(property.datatype.as_deref(), datatype);
        assert_eq!(property.lang.as_deref(), lang);
        assert_eq!(property.graph_scope, graph_scope);
        assert!(!property.statement_id.is_empty());
        assert_eq!(property.provenance_iri, None);
        assert_eq!(property.asserted_at_ms, None);
    }

    #[test]
    fn empty_graph_exports_empty_array() {
        assert_eq!(to_jsonld(&Graph::new()), json!([]));
    }

    #[test]
    fn properties_round_trip_via_the_property_bag() {
        // A non-curated literal survives ingest → graph property bag → export.
        let doc = br#"{"@id":"https://a.test","https://schema.org/datePublished":[{"@value":"2026-06-02"}]}"#;
        let contribution = from_jsonld(doc).expect("parse");
        let node = contribution
            .nodes
            .iter()
            .find(|n| n.id == "https://a.test")
            .expect("node a");
        assert_eq!(node.properties.len(), 1);
        assert_property_fields(
            &node.properties[0],
            "https://schema.org/datePublished",
            "2026-06-02",
            None,
            None,
            GraphScope::Default,
        );

        let mut graph = Graph::new();
        apply_contribution(&mut graph, &contribution);
        let exported = to_jsonld(&graph);
        let a = exported
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["@id"] == json!("https://a.test"))
            .expect("exported node a");
        assert_eq!(
            a["https://schema.org/datePublished"],
            json!([{ "@value": "2026-06-02" }])
        );
    }

    #[test]
    fn typed_and_language_tagged_properties_round_trip_via_the_property_bag() {
        let doc = br#"{
            "@id":"https://a.test",
            "https://schema.org/datePublished":[{"@value":"2026-06-02","@type":"http://www.w3.org/2001/XMLSchema#date"}],
            "https://schema.org/headline":[{"@value":"Bonjour","@language":"fr"}]
        }"#;
        let contribution = from_jsonld(doc).expect("parse");
        let node = contribution
            .nodes
            .iter()
            .find(|n| n.id == "https://a.test")
            .expect("node a");
        assert_eq!(node.properties.len(), 2);
        assert_property_fields(
            &node.properties[0],
            "https://schema.org/datePublished",
            "2026-06-02",
            Some("http://www.w3.org/2001/XMLSchema#date"),
            None,
            GraphScope::Default,
        );
        assert_property_fields(
            &node.properties[1],
            "https://schema.org/headline",
            "Bonjour",
            None,
            Some("fr"),
            GraphScope::Default,
        );

        let mut graph = Graph::new();
        apply_contribution(&mut graph, &contribution);
        let exported = to_jsonld(&graph);
        let a = exported
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["@id"] == json!("https://a.test"))
            .expect("exported node a");
        assert_eq!(
            a["https://schema.org/datePublished"],
            json!([{
                "@value": "2026-06-02",
                "@type": "http://www.w3.org/2001/XMLSchema#date"
            }])
        );
        assert_eq!(
            a["https://schema.org/headline"],
            json!([{
                "@value": "Bonjour",
                "@language": "fr"
            }])
        );
    }

    #[test]
    fn type_round_trips_via_rdf_type_classification() {
        // @type → node types on ingest, applied as an `rdf:type` classification,
        // re-exported as @type.
        let doc = br#"{"@id":"https://a.test","@type":["https://schema.org/Article"]}"#;
        let contribution = from_jsonld(doc).expect("parse");
        let node = contribution
            .nodes
            .iter()
            .find(|n| n.id == "https://a.test")
            .expect("node a");
        assert_eq!(node.types, vec!["https://schema.org/Article".to_string()]);

        let mut graph = Graph::new();
        apply_contribution(&mut graph, &contribution);
        let exported = to_jsonld(&graph);
        let a = exported
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["@id"] == json!("https://a.test"))
            .expect("exported node a");
        assert_eq!(a["@type"], json!(["https://schema.org/Article"]));
    }

    fn instance_objects(document: &serde_json::Value, reifier_count: usize) -> serde_json::Value {
        let nodes = document.as_array().expect("expanded node array");
        let is_reifier = |node: &&serde_json::Value| {
            node["@type"].as_array().is_some_and(|types| {
                types.contains(&serde_json::Value::String(
                    crate::jsonld::RDF_STATEMENT.into(),
                ))
            })
        };
        assert_eq!(nodes.iter().filter(is_reifier).count(), reifier_count);
        serde_json::Value::Array(
            nodes
                .iter()
                .filter(|node| !is_reifier(node))
                .cloned()
                .collect(),
        )
    }

    #[test]
    fn exports_recognized_and_raw_predicates_with_literals() {
        assert_eq!(
            instance_objects(&to_jsonld(&seed()), 2),
            json!([
                {
                    "@id": "https://a.test",
                    "https://schema.org/name": [{ "@value": "Article A" }],
                    "https://schema.org/keywords": [{ "@value": "research" }],
                    "https://mere.computer/ns/rel#cites": [{ "@id": "https://b.test" }],
                    "https://schema.org/citation": [{ "@id": "https://c.test" }]
                },
                { "@id": "https://b.test" },
                { "@id": "https://c.test" }
            ])
        );
    }

    #[test]
    fn exports_multiple_statements_on_one_node_pair() {
        let mut graph = Graph::new();
        let a = graph.add_node("https://a.test".to_string(), Default::default());
        let b = graph.add_node("https://b.test".to_string(), Default::default());

        graph.assert_relation(
            a,
            b,
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Cites,
                label: None,
                decay_progress: None,
            },
        );
        graph.assert_semantic_predicate(a, b, "https://schema.org/citation".to_string());

        assert_eq!(
            instance_objects(&to_jsonld(&graph), 2),
            json!([
                {
                    "@id": "https://a.test",
                    "https://mere.computer/ns/rel#cites": [{ "@id": "https://b.test" }],
                    "https://schema.org/citation": [{ "@id": "https://b.test" }]
                },
                { "@id": "https://b.test" }
            ])
        );
    }

    #[test]
    fn compact_export_uses_terms_and_keeps_raw_iris() {
        let compact = to_jsonld_compact(&seed());
        // A recognized relation + the curated literals are short terms backed by
        // the inline context.
        assert_eq!(
            compact["@context"]["cites"],
            json!("https://mere.computer/ns/rel#cites")
        );
        assert_eq!(
            compact["@context"]["name"],
            json!("https://schema.org/name")
        );
        let a = compact["@graph"]
            .as_array()
            .expect("@graph array")
            .iter()
            .find(|n| n["@id"] == json!("https://a.test"))
            .expect("node a");
        assert_eq!(a["name"], json!("Article A"));
        assert_eq!(a["cites"], json!({ "@id": "https://b.test" }));
        // The raw predicate keeps its full IRI as the key, not a context term.
        assert_eq!(
            a["https://schema.org/citation"],
            json!({ "@id": "https://c.test" })
        );
        assert!(compact["@context"].get("citation").is_none());
    }

    #[test]
    fn expanded_export_round_trips_through_ingest() {
        let graph = seed();
        let doc = serde_json::to_vec(&to_jsonld(&graph)).expect("serialize");
        assert_round_trip(&from_jsonld(&doc).expect("round-trip parse"), &graph);
    }

    #[test]
    fn compact_export_round_trips_through_ingest() {
        let graph = seed();
        let doc = serde_json::to_vec(&to_jsonld_compact(&graph)).expect("serialize");
        assert_round_trip(&from_jsonld(&doc).expect("round-trip parse"), &graph);
    }

    /// Both export forms must ingest back to the same logical content: A's curated
    /// literals, the recognized `cites` edge (canonical IRI), and the raw
    /// `schema:citation` edge.
    /// THE Phase 2 losslessness gate (petgraph-RDF plan): a graph exercising
    /// the full profile construct matrix — typed + language-tagged literals,
    /// named graph scopes, statement metadata (label / provenance / assertion
    /// time via RDF 1.2 reifiers), two asserters of one triple term and
    /// two differently-scoped statements on one
    /// pair, a recognized (CiTO-mapped) and a raw predicate, `rdf:type`,
    /// curated title/tags — projects to quads, re-ingests through the quad
    /// path into a FRESH graph, and projects identically: a normalized
    /// (sorted N-Quads) dataset compare, id-stable through the reifier
    /// handles. "Lossless under the profile" as a checked property.
    #[test]
    fn dataset_round_trip_is_lossless_under_the_profile() {
        use super::{GRAPH_SCOPE_USER, RDF_REIFIES};
        use kernel::graph::SemanticStatementSpec;
        use oxrdf::{GraphName, NamedNode, Term};

        let mut graph = Graph::new();
        let a = graph.add_node("https://a.test".to_string(), Default::default());
        let b = graph.add_node("https://b.test".to_string(), Default::default());
        let c = graph.add_node("https://c.test".to_string(), Default::default());

        // Curated fast-path literals.
        graph.get_node_mut(a).expect("a").title = "Article A".to_string();
        assert!(graph.insert_node_tags(a, vec!["research".into()]));

        // A recognized predicate with full statement metadata, plus a second
        // statement on the SAME pair in a different named graph.
        graph
            .assert_semantic_statement(
                a,
                b,
                SemanticStatementSpec {
                    predicate: "https://mere.computer/ns/rel#cites".to_string(),
                    recognized_sub_kind: Some(SemanticSubKind::Cites),
                    label: Some("cited in the intro".to_string()),
                    basis: Default::default(),
                    graph_scope: GraphScope::User,
                    provenance_iri: Some("https://persona.test/mark".to_string()),
                    asserted_at_ms: Some(1_720_000_000_000),
                },
            )
            .expect("cites statement");
        graph
            .assert_semantic_statement(
                a,
                b,
                SemanticStatementSpec {
                    predicate: "https://mere.computer/ns/rel#cites".to_string(),
                    recognized_sub_kind: Some(SemanticSubKind::Cites),
                    graph_scope: GraphScope::Source,
                    ..Default::default()
                },
            )
            .expect("source-scoped statement");
        // A raw (unrecognized) predicate.
        graph
            .assert_semantic_statement(
                a,
                c,
                SemanticStatementSpec {
                    predicate: "https://example.test/vocab#inspiredBy".to_string(),
                    asserted_at_ms: Some(1_720_000_100_000),
                    ..Default::default()
                },
            )
            .expect("raw predicate statement");

        // The same triple term and scope has two independently attributable
        // assertion handles; neither reifier may overwrite the other.
        graph
            .assert_semantic_statement(
                a,
                b,
                SemanticStatementSpec {
                    predicate: "https://mere.computer/ns/rel#cites".to_string(),
                    recognized_sub_kind: Some(SemanticSubKind::Cites),
                    label: Some("also cited by Alice".to_string()),
                    basis: Default::default(),
                    graph_scope: GraphScope::User,
                    provenance_iri: Some("https://persona.test/alice".to_string()),
                    asserted_at_ms: Some(1_720_000_050_000),
                },
            )
            .expect("second asserter statement");

        // Typed + language-tagged + scoped literals with metadata.
        let mut published = NodeProperty::new(
            "https://schema.org/datePublished".to_string(),
            "2026-07-04".to_string(),
        )
        .with_graph_scope(GraphScope::User);
        published.datatype = Some("http://www.w3.org/2001/XMLSchema#date".to_string());
        published.provenance_iri = Some("https://persona.test/mark".to_string());
        published.asserted_at_ms = Some(1_720_000_200_000);
        kernel::graph::apply::apply_graph_delta(
            &mut graph,
            kernel::graph::apply::GraphDelta::AppendNodeProperty {
                key: a,
                property: published,
            },
        );
        let mut greeting = NodeProperty::new(
            "https://schema.org/description".to_string(),
            "bonjour".to_string(),
        );
        greeting.lang = Some("fr".to_string());
        kernel::graph::apply::apply_graph_delta(
            &mut graph,
            kernel::graph::apply::GraphDelta::AppendNodeProperty {
                key: a,
                property: greeting,
            },
        );

        // rdf:type via classification (through the delta, the public write).
        let _ = kernel::graph::apply::apply_graph_delta(
            &mut graph,
            kernel::graph::apply::GraphDelta::AddNodeClassification {
                key: a,
                classification: kernel::types::NodeClassification {
                    scheme: kernel::types::ClassificationScheme::Custom("rdf:type".to_string()),
                    value: "https://schema.org/Article".to_string(),
                    label: None,
                    confidence: 1.0,
                    provenance: kernel::types::ClassificationProvenance::Imported,
                    status: kernel::types::ClassificationStatus::Imported,
                    primary: false,
                },
            },
        );

        let normalized = |graph: &Graph| -> Vec<String> {
            let mut lines: Vec<String> = crate::dataset_quads(graph)
                .iter()
                .map(|quad| format!("{quad} ."))
                .collect();
            lines.sort();
            lines
        };
        let exported = normalized(&graph);

        let reifiers: Vec<_> = crate::dataset_quads(&graph)
            .into_iter()
            .filter(|quad| {
                quad.predicate.as_str() == RDF_REIFIES
                    && quad.graph_name
                        == GraphName::from(NamedNode::new(GRAPH_SCOPE_USER).expect("user scope"))
                    && matches!(&quad.object, Term::Triple(triple)
                        if triple.predicate.as_str() == "https://mere.computer/ns/rel#cites")
            })
            .collect();
        assert_eq!(reifiers.len(), 2, "one reifier per asserter");
        assert_ne!(reifiers[0].subject, reifiers[1].subject);
        assert_eq!(
            reifiers[0].object, reifiers[1].object,
            "one shared triple term"
        );

        let contribution = crate::ingest::from_quads_envelope(crate::dataset_quads(&graph), "gate")
            .expect("quad ingest");
        let mut reimported = Graph::new();
        let outcome = crate::ingest::apply_import(&mut reimported, &contribution);
        assert!(outcome.edges_skipped == 0, "self-contained contribution");
        assert_eq!(
            outcome.edges_asserted, 5,
            "all four relation handles and the tagging assertion survive"
        );

        let reexported = normalized(&reimported);
        assert_eq!(
            exported, reexported,
            "RDF -> kernel -> RDF is byte-stable as a normalized dataset"
        );
    }

    fn assert_round_trip(contribution: &GraphContribution, graph: &Graph) {
        let a = contribution
            .nodes
            .iter()
            .find(|n| n.id == "https://a.test")
            .expect("node a");
        assert_eq!(a.title.as_deref(), Some("Article A"));
        assert_eq!(a.tags, vec!["research".to_string()]);
        let expected = crate::from_quads(crate::dataset_quads(graph), "round-trip oracle")
            .expect("native RDF contribution");
        assert_eq!(
            contribution, &expected,
            "JSON-LD preserves the exact assertion metadata"
        );
    }

    #[cfg(feature = "query")]
    #[test]
    fn sparql_selects_a_literal_and_an_edge_over_node_quads() {
        let graph = seed();

        // A curated literal: A's schema:name.
        let names = crate::query::sparql(
            &graph,
            "SELECT ?name WHERE { <https://a.test> <https://schema.org/name> ?name }",
        )
        .expect("name query");
        assert_eq!(names.variables, vec!["name".to_string()]);
        assert_eq!(names.rows, vec![vec![Some("Article A".to_string())]]);

        // A recognized semantic edge: A cites B (canonical Mere IRI).
        let cites = crate::query::sparql(
            &graph,
            "SELECT ?t WHERE { <https://a.test> <https://mere.computer/ns/rel#cites> ?t }",
        )
        .expect("edge query");
        assert_eq!(cites.rows, vec![vec![Some("https://b.test".to_string())]]);
    }

    #[cfg(feature = "query")]
    #[test]
    fn sparql_graph_clause_sees_scoped_statements_and_properties() {
        let mut graph = Graph::new();
        let a = graph.add_node("https://a.test".to_string(), Default::default());
        let b = graph.add_node("https://b.test".to_string(), Default::default());

        kernel::graph::apply::assert_semantic_relation_in_scope(
            &mut graph,
            a,
            b,
            SemanticSubKind::Cites,
            None,
            GraphScope::Source,
        );
        assert!(graph.append_node_properties(
            a,
            vec![
            NodeProperty::new(
                "https://schema.org/datePublished".to_string(),
                "2026-07-04".to_string(),
            )
            .with_graph_scope(GraphScope::User),
        ]
        ));

        let scoped_edge = crate::query::sparql(
            &graph,
            "SELECT ?t WHERE { GRAPH <https://mere.computer/ns/graph#source> { <https://a.test> <https://mere.computer/ns/rel#cites> ?t } }",
        )
        .expect("scoped edge query");
        assert_eq!(
            scoped_edge.rows,
            vec![vec![Some("https://b.test".to_string())]]
        );

        let scoped_property = crate::query::sparql(
            &graph,
            "SELECT ?v WHERE { GRAPH <https://mere.computer/ns/graph#user> { <https://a.test> <https://schema.org/datePublished> ?v } }",
        )
        .expect("scoped property query");
        assert_eq!(
            scoped_property.rows,
            vec![vec![Some("2026-07-04".to_string())]]
        );
    }

    #[cfg(feature = "query")]
    #[test]
    fn sparql_exposes_reifier_metadata_but_node_quads_stay_clean() {
        let mut graph = Graph::new();
        let a = graph.add_node("https://a.test".to_string(), Default::default());
        let b = graph.add_node("https://b.test".to_string(), Default::default());

        kernel::graph::apply::assert_semantic_relation_in_scope(
            &mut graph,
            a,
            b,
            SemanticSubKind::Cites,
            Some("cites".to_string()),
            GraphScope::Source,
        );

        let edge = graph
            .projected_relations_between(a, b)
            .next()
            .expect("semantic edge")
            .0;
        let payload = graph.get_relation_mut(edge).expect("payload");
        let statement = payload
            .assertions
            .statements
            .iter_mut()
            .next()
            .expect("semantic statement");
        statement.statement_id = "stmt-edge-1".to_string();
        statement.provenance_iri = Some("https://people.test/alice".to_string());
        statement.asserted_at_ms = Some(1_720_000_000_123);

        let mut property = NodeProperty::new(
            "https://schema.org/datePublished".to_string(),
            "2026-07-04".to_string(),
        )
        .with_graph_scope(GraphScope::User)
        .with_metadata(
            Some("https://people.test/bob".to_string()),
            Some(1_720_000_100_456),
        );
        property.statement_id = "stmt-prop-1".to_string();
        assert!(graph.append_node_properties(a, vec![property]));

        let (_, node_a) = graph.get_node_by_url("https://a.test").expect("node a");
        assert!(
            node_quads(&graph, a, node_a)
                .iter()
                .all(|quad| quad.predicate.as_str() != RDF_REIFIES)
        );

        let edge_metadata = crate::query::sparql(
            &graph,
            "SELECT ?prov ?label WHERE {
                GRAPH <https://mere.computer/ns/graph#source> {
                    <urn:mere:statement:stmt-edge-1> <http://www.w3.org/ns/prov#wasAttributedTo> ?prov ;
                                                     <http://www.w3.org/2000/01/rdf-schema#label> ?label .
                }
            }",
        )
        .expect("edge metadata query");
        assert_eq!(
            edge_metadata.rows,
            vec![vec![
                Some("https://people.test/alice".to_string()),
                Some("cites".to_string()),
            ]]
        );

        let property_metadata = crate::query::sparql(
            &graph,
            "SELECT ?prov WHERE {
                GRAPH <https://mere.computer/ns/graph#user> {
                    <urn:mere:statement:stmt-prop-1> <http://www.w3.org/ns/prov#wasAttributedTo> ?prov .
                }
            }",
        )
        .expect("property metadata query");
        assert_eq!(
            property_metadata.rows,
            vec![vec![Some("https://people.test/bob".to_string())]]
        );
    }
}
