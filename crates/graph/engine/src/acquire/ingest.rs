// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! JSON-LD ingest (linked-data plan Phase 2): `application/ld+json` → a
//! [`GraphContribution`].
//!
//! [`from_jsonld`] is a **pure** parse — bytes through `oxjsonld` (sync,
//! wasm-light) to RDF triples, grouped into the contribution. It mirrors
//! [`crate::to_jsonld`]: a resource-valued predicate becomes an edge, `rdf:type`
//! becomes a node type, and the curated literals `schema:name` / `schema:keywords`
//! become a node's title / tags; every other literal lands in the node property
//! bag, carrying datatype and language-tag metadata when present. Blank nodes are
//! skolemized to a `urn:mere:bnode:` IRI.
//!
//! [`apply_contribution`] materializes a contribution into a [`Graph`]: it
//! creates a surface per subject/object (or reuses one matched by URL), binds
//! new subjects to exact-IRI resources and asserts
//! each edge — a recognized predicate as a typed `Semantic` edge (sub-kind +
//! canonical IRI), an unrecognized one as an **open-predicate** edge via
//! `Graph::assert_semantic_predicate`. It is `not(wasm32)` because `add_node`
//! mints a UUID; a wasm host materializes from the same contribution with
//! `add_node_with_id`.
//! Page extractors supply explicit per-subject [`SubjectIdentity::Page`] intent
//! through [`apply_contribution_with_identity`]. Contribution transport does
//! not infer identity from a document base or an RDF type.
//!
//! The parser expands `@type` values and can resolve explicitly cached remote
//! contexts without network access. Class-IRI projection policy and complete
//! named-graph and assertion metadata are shared across all supported formats.

use crate::model::{AssertionBasis, AssertionWithdrawal, GraphScope, NodeProperty};
use crate::reifier::statement_id_from_reifier;
use crate::{SCHEMA_KEYWORDS, SCHEMA_NAME};
use oxjsonld::{JsonLdParser, JsonLdRemoteDocument};
use oxrdf::{NamedOrBlankNode, Quad, Term};
use std::collections::BTreeMap;

/// `rdf:type` — the predicate JSON-LD `@type` expands to.
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
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
const WITHDRAWAL_ACTIVITY_PREFIX: &str = "urn:mere:withdrawal-activity:v1:";

/// How a newly bound subject identifies its resource at the apply boundary.
/// Existing shown-resource associations retain their prepared identity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SubjectIdentity {
    /// Preserve the prepared subject IRI, including fragment and case.
    #[default]
    ExactIri,
    /// A caller-identified page uses the common URL canonicalizer.
    Page,
}

/// A node described by an ingested JSON-LD document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeContribution {
    /// The subject IRI (a URL, or a skolemized `urn:` IRI for a blank node).
    pub id: String,
    /// `@type` IRIs — applied as `rdf:type` classifications by
    /// [`apply_contribution`].
    pub types: Vec<String>,
    /// `schema:name`, if present.
    pub title: Option<String>,
    /// `schema:keywords` values.
    pub tags: Vec<String>,
    /// Non-curated literal properties: every literal beyond `schema:name` /
    /// `schema:keywords`, including datatype/lang fidelity when present.
    pub properties: Vec<NodeProperty>,
}

impl NodeContribution {
    /// An empty contribution for one subject.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            types: Vec::new(),
            title: None,
            tags: Vec::new(),
            properties: Vec::new(),
        }
    }
}

/// A predicate edge: `subject —predicate→ object`, all IRIs. Statement
/// metadata (fact handle, label, provenance, assertion time) arrives via an
/// RDF 1.2 reifier over the triple; absent for plain edges.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EdgeContribution {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub graph_scope: GraphScope,
    /// The exact fact handle a legacy or encoded Mere reifier carried.
    /// Preserving it through apply makes the dataset round trip id-stable.
    pub statement_id: Option<String>,
    pub label: Option<String>,
    pub provenance_iri: Option<String>,
    /// First assertion record time, distinct from later confirmations.
    pub first_asserted_ms: Option<u64>,
    /// Latest assertion/confirmation time; retained under the legacy name.
    pub asserted_at_ms: Option<u64>,
    /// Retained tombstone metadata when this fact was withdrawn.
    pub withdrawn: Option<AssertionWithdrawal>,
    /// Reason metadata; `label` remains its compatibility projection.
    pub basis: AssertionBasis,
}

/// The result of parsing a JSON-LD document: the nodes it describes and the
/// predicate edges between them. Every IRI referenced by an edge also appears as
/// a node, so the contribution is self-contained.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphContribution {
    pub nodes: Vec<NodeContribution>,
    pub edges: Vec<EdgeContribution>,
}

/// Parser evidence the importer reads beside a contribution.
pub mod envelope;
pub use envelope::ImportEnvelope;

/// RDF ingest failure (JSON-LD, N-Quads, or TriG).
#[derive(Debug)]
pub enum IngestError {
    /// RDF syntax, expansion, or reserved Mere reifier data was invalid.
    Parse(String),
}

impl std::fmt::Display for IngestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IngestError::Parse(msg) => write!(f, "RDF parse error: {msg}"),
        }
    }
}

impl std::error::Error for IngestError {}

/// A stable per-document namespace (FNV-1a over the bytes, hex) that scopes a
/// blank node's skolemized IRI to its source document. oxjsonld already assigns
/// each blank a unique label per parse, so this is defensive scoping rather than
/// strict collision-avoidance; full re-ingest idempotency for blanks would need
/// RDF canonicalization (URDNA2015), which is out of scope.
fn doc_namespace(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// Skolemize a blank-node id to a Mere IRI, scoped to its source document via
/// `namespace`.
fn skolemize(namespace: &str, blank_id: &str) -> String {
    format!("urn:mere:bnode:{namespace}:{blank_id}")
}

/// Canonicalize a schema.org IRI to its `https` form. The schema.org `@context`
/// uses `@vocab: http://schema.org/`, so a document referencing it expands terms
/// to `http://schema.org/…`; Mere keys on (and stores) `https://schema.org/…`, so
/// the two flavors of schema.org document unify.
fn normalize_schema_org(iri: &str) -> String {
    match iri.strip_prefix("http://schema.org/") {
        Some(rest) => format!("https://schema.org/{rest}"),
        None => iri.to_string(),
    }
}

/// Literal predicates promoted to a node's title, across the curated
/// vocabularies (schema.org, Dublin Core, ActivityStreams). Last one seen wins.
const TITLE_PREDICATES: &[&str] = &[
    SCHEMA_NAME,                                  // schema:name
    "http://purl.org/dc/terms/title",             // dcterms:title
    "http://purl.org/dc/elements/1.1/title",      // dc:title
    "https://www.w3.org/ns/activitystreams#name", // as:name
];

/// Literal predicates promoted to a node's tags.
const TAGS_PREDICATES: &[&str] = &[
    SCHEMA_KEYWORDS,                           // schema:keywords
    "http://purl.org/dc/terms/subject",        // dcterms:subject
    "http://purl.org/dc/elements/1.1/subject", // dc:subject
];

fn is_title_predicate(predicate: &str) -> bool {
    TITLE_PREDICATES.contains(&predicate)
}

fn is_tags_predicate(predicate: &str) -> bool {
    TAGS_PREDICATES.contains(&predicate)
}

/// The IRI for a subject term (skolemizing a blank node within `namespace`).
fn subject_iri(subject: &NamedOrBlankNode, namespace: &str) -> String {
    match subject {
        NamedOrBlankNode::NamedNode(node) => node.as_str().to_string(),
        NamedOrBlankNode::BlankNode(node) => skolemize(namespace, node.as_str()),
    }
}

/// Route a resource-valued object: `rdf:type` adds a node type, anything else is
/// an edge.
fn route_resource(
    nodes: &mut BTreeMap<String, NodeContribution>,
    edges: &mut Vec<EdgeContribution>,
    subject: &str,
    predicate: &str,
    object: String,
    graph_scope: GraphScope,
) {
    if predicate == RDF_TYPE && graph_scope == GraphScope::Default {
        nodes
            .get_mut(subject)
            .expect("subject inserted before routing")
            .types
            .push(object);
    } else {
        edges.push(EdgeContribution {
            subject: subject.to_string(),
            predicate: predicate.to_string(),
            object,
            graph_scope,
            statement_id: None,
            label: None,
            provenance_iri: None,
            first_asserted_ms: None,
            asserted_at_ms: None,
            withdrawn: None,
            basis: AssertionBasis::default(),
        });
    }
}

fn scope_from_graph_name(graph_name: &oxrdf::GraphName, namespace: &str) -> GraphScope {
    match graph_name {
        oxrdf::GraphName::DefaultGraph => GraphScope::Default,
        oxrdf::GraphName::NamedNode(node) => match node.as_str() {
            "https://mere.computer/ns/graph#source" => GraphScope::Source,
            "https://mere.computer/ns/graph#user" => GraphScope::User,
            "https://mere.computer/ns/graph#agent" => GraphScope::Agent,
            "https://mere.computer/ns/graph#moot" => GraphScope::Moot,
            iri => GraphScope::Custom(iri.to_string()),
        },
        oxrdf::GraphName::BlankNode(node) => {
            GraphScope::Custom(skolemize(namespace, node.as_str()))
        },
    }
}

/// Parse an `application/ld+json` document into a [`GraphContribution`]. Pure: no
/// graph, no kernel mutation. Expects inline-`@context` or expanded JSON-LD; a
/// remote `@context` is not fetched (a bundled-context loader is a later step).
pub fn from_jsonld(bytes: &[u8]) -> Result<GraphContribution, IngestError> {
    from_jsonld_envelope(bytes).map(ImportEnvelope::into_contribution)
}

/// Parse JSON-LD with definition evidence for [`apply_import`].
pub fn from_jsonld_envelope(bytes: &[u8]) -> Result<ImportEnvelope, IngestError> {
    collect_contribution(JsonLdParser::new().for_slice(bytes), &doc_namespace(bytes))
}

/// Ingest a set of RDF quads directly — the dataset path (`dataset_quads` /
/// Turtle / N-Quads), and the half of the Phase 2 round-trip gate that JSON-LD
/// represents with classic records: RDF 1.2 reifier metadata (triple terms) is recognized here
/// and attached to the matching edge / property contributions.
pub fn from_quads(
    quads: impl IntoIterator<Item = Quad>,
    namespace: &str,
) -> Result<GraphContribution, IngestError> {
    from_quads_envelope(quads, namespace).map(ImportEnvelope::into_contribution)
}

/// Parse RDF quads while retaining profile definition and assertion evidence.
pub fn from_quads_envelope(
    quads: impl IntoIterator<Item = Quad>,
    namespace: &str,
) -> Result<ImportEnvelope, IngestError> {
    collect_contribution(
        quads.into_iter().map(Ok::<Quad, std::convert::Infallible>),
        namespace,
    )
}

/// Like [`from_jsonld`], but a remote `@context` is resolved from `contexts`
/// instead of the network. A context URL absent from the cache is an error, so
/// ingest never makes a request. The cache is the seam for bundling schema.org /
/// Dublin Core / ActivityStreams; populating it (the asset-weight decision) is a
/// separate step.
pub fn from_jsonld_with_contexts(
    bytes: &[u8],
    contexts: ContextCache,
) -> Result<GraphContribution, IngestError> {
    from_jsonld_with_contexts_and_base_iri(bytes, contexts, None)
}

/// Like [`from_jsonld_with_contexts`], while resolving relative IRIs against
/// `base_iri` when the caller has retained the document's resolved address.
///
/// The base is an input to JSON-LD's expansion algorithm, not a source-identity
/// claim. Callers that do not own a resolved document address must pass `None`;
/// this parser never infers one or fetches it from the network.
pub fn from_jsonld_with_contexts_and_base_iri(
    bytes: &[u8],
    contexts: ContextCache,
    base_iri: Option<&str>,
) -> Result<GraphContribution, IngestError> {
    from_jsonld_envelope_with_contexts_and_base_iri(bytes, contexts, base_iri)
        .map(ImportEnvelope::into_contribution)
}

/// Parse JSON-LD using the offline cache and retain its import evidence.
pub fn from_jsonld_envelope_with_contexts(
    bytes: &[u8],
    contexts: ContextCache,
) -> Result<ImportEnvelope, IngestError> {
    from_jsonld_envelope_with_contexts_and_base_iri(bytes, contexts, None)
}

/// Like [`from_jsonld_envelope_with_contexts`], using a caller-owned document base.
pub fn from_jsonld_envelope_with_contexts_and_base_iri(
    bytes: &[u8],
    contexts: ContextCache,
    base_iri: Option<&str>,
) -> Result<ImportEnvelope, IngestError> {
    let namespace = doc_namespace(bytes);
    let parser = match base_iri {
        Some(base_iri) => JsonLdParser::new()
            .with_base_iri(base_iri)
            .map_err(|error| IngestError::Parse(format!("invalid JSON-LD base IRI: {error}")))?,
        None => JsonLdParser::new(),
    };
    let quads = parser
        .for_slice(bytes)
        .with_load_document_callback(offline_loader(contexts));
    collect_contribution(quads, &namespace)
}

/// The document loader that serves `@context` documents from `contexts` and
/// refuses everything else, so no parse makes a request.
#[allow(clippy::type_complexity)]
pub(crate) fn offline_loader(
    contexts: ContextCache,
) -> impl Fn(
    &str,
    &oxjsonld::JsonLdLoadDocumentOptions,
) -> Result<JsonLdRemoteDocument, Box<dyn std::error::Error + Send + Sync>>
+ Send
+ Sync
+ std::panic::UnwindSafe
+ std::panic::RefUnwindSafe
+ 'static {
    move |url, _options| {
        contexts
            .get(url)
            .map(|document| JsonLdRemoteDocument {
                document: document.to_vec(),
                document_url: url.to_string(),
            })
            .ok_or_else(|| -> Box<dyn std::error::Error + Send + Sync> {
                format!("refused remote @context (not in the bundled cache): {url}").into()
            })
    }
}

/// Scan a JSON-LD document for the remote `@context` URLs it references — the
/// string entries of any `@context`, top-level or nested, including inside
/// `@graph`. A host fetches these (minus the ones the bundled packs already
/// cover, see [`is_bundled_context`]) and folds them into a [`ContextCache`]
/// before ingest, so an unbundled vocabulary resolves. Pure and network-free;
/// returns `[]` for non-JSON input.
pub fn referenced_context_urls(body: &[u8]) -> Vec<String> {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return Vec::new();
    };
    let mut urls = Vec::new();
    collect_context_urls(&value, &mut urls);
    urls.sort();
    urls.dedup();
    urls
}

/// Walk every object for an `@context`, collecting its remote string references.
fn collect_context_urls(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(ctx) = map.get("@context") {
                collect_context_strings(ctx, out);
            }
            for (key, child) in map {
                if key != "@context" {
                    collect_context_urls(child, out);
                }
            }
        },
        serde_json::Value::Array(items) => items.iter().for_each(|i| collect_context_urls(i, out)),
        _ => {},
    }
}

/// A `@context` value is a URL string, an array (URL strings + inline objects), or
/// an inline object. Collect only the remote (`http(s)`) string references.
fn collect_context_strings(ctx: &serde_json::Value, out: &mut Vec<String>) {
    match ctx {
        serde_json::Value::String(s) if s.starts_with("http://") || s.starts_with("https://") => {
            out.push(s.clone());
        },
        serde_json::Value::Array(items) => {
            items.iter().for_each(|i| collect_context_strings(i, out))
        },
        _ => {},
    }
}
const RDF_REIFIES: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies";
const RDFS_LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";
const PROV_WAS_ATTRIBUTED_TO: &str = "http://www.w3.org/ns/prov#wasAttributedTo";
const PROV_GENERATED_AT_TIME: &str = "http://www.w3.org/ns/prov#generatedAtTime";

/// One lifted RDF 1.2 reifier: the fact it names + the metadata its other
/// quads carried.
struct ReifiedStatement {
    statement_id: Option<String>,
    subject: String,
    predicate: String,
    object: ReifiedObject,
    graph_scope: GraphScope,
    label: Option<String>,
    provenance_iri: Option<String>,
    first_asserted_ms: Option<u64>,
    last_asserted_ms: Option<u64>,
    lifecycle_version: bool,
    method_iri: Option<String>,
    method_version: Option<String>,
    evidence: Vec<String>,
    withdrawal_activity: Option<String>,
    withdrawal_actor: Option<String>,
    withdrawal_at_ms: Option<u64>,
    asserted_graph_kind: Option<String>,
    asserted_graph_name: Option<String>,
}

#[derive(PartialEq, Eq)]
enum ReifiedObject {
    Resource(String),
    Literal {
        value: String,
        datatype: String,
        lang: Option<String>,
    },
}

/// Parse an `xsd:dateTime` lexical back to unix ms — the inverse of the
/// export's `prov:generatedAtTime` formatting.
pub(crate) fn parse_xsd_datetime_ms(lexical: &str) -> Option<u64> {
    let parsed =
        time::OffsetDateTime::parse(lexical, &time::format_description::well_known::Rfc3339)
            .ok()?;
    u64::try_from(parsed.unix_timestamp_nanos() / 1_000_000).ok()
}

fn parse_metadata_time(lexical: &str) -> Result<u64, IngestError> {
    parse_xsd_datetime_ms(lexical).ok_or_else(|| {
        IngestError::Parse(format!(
            "invalid nonnegative xsd:dateTime value {lexical:?}"
        ))
    })
}

/// RDF graph order is not meaningful. Lifecycle and basis properties have one
/// value, so repeated equal quads are harmless while conflicting values fail
/// deterministically instead of allowing the last encountered quad to win.
fn set_once_or_equal<T: PartialEq>(
    slot: &mut Option<T>,
    value: T,
    field: &str,
) -> Result<(), IngestError> {
    if let Some(existing) = slot.as_ref() {
        if existing != &value {
            return Err(IngestError::Parse(format!("conflicting {field} metadata")));
        }
    } else {
        *slot = Some(value);
    }
    Ok(())
}

fn restore_graph_scope(kind: Option<&str>, name: Option<&str>) -> Result<GraphScope, IngestError> {
    match (kind, name) {
        (Some("default"), None) => Ok(GraphScope::Default),
        (Some("source"), None) => Ok(GraphScope::Source),
        (Some("user"), None) => Ok(GraphScope::User),
        (Some("agent"), None) => Ok(GraphScope::Agent),
        (Some("moot"), None) => Ok(GraphScope::Moot),
        (Some("custom"), Some(name)) => Ok(GraphScope::Custom(name.to_string())),
        _ => Err(IngestError::Parse(
            "incomplete or invalid original assertion graph scope marker".into(),
        )),
    }
}

fn is_withdrawal_activity_metadata(quad: &Quad, namespace: &str) -> bool {
    subject_iri(&quad.subject, namespace).starts_with(WITHDRAWAL_ACTIVITY_PREFIX)
        && matches!(quad.predicate.as_str(), RDF_TYPE | PROV_WAS_ASSOCIATED_WITH)
}

fn parse_withdrawal_activity_id(activity: &str) -> Option<String> {
    let encoded = activity.strip_prefix(WITHDRAWAL_ACTIVITY_PREFIX)?;
    if encoded.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::with_capacity(encoded.len() / 2);
    for pair in encoded.as_bytes().chunks_exact(2) {
        let high = char::from(pair[0]).to_digit(16)?;
        let low = char::from(pair[1]).to_digit(16)?;
        bytes.push(((high << 4) | low) as u8);
    }
    String::from_utf8(bytes).ok()
}

/// A bundled JSON-LD `@context` cache for offline ingest: a map of context URL to
/// its document bytes. [`from_jsonld_with_contexts`] serves these and refuses any
/// URL not present, so a document's remote `@context` never hits the network.
#[derive(Clone, Default)]
pub struct ContextCache {
    documents: std::sync::Arc<std::collections::HashMap<String, Vec<u8>>>,
}

/// The URL Mere's curated context is served at.
const MERE_CONTEXT_URL: &str = "https://mere.computer/ns/context";

/// Mere's curated context (the "minimal" pack): `schema:name` / `schema:keywords`
/// plus the `mere:` relation prefix.
const MERE_MINIMAL_CONTEXT: &[u8] = br#"{"@context":{"name":"https://schema.org/name","keywords":"https://schema.org/keywords","mere":"https://mere.computer/ns/rel#"}}"#;

/// The vendored schema.org context (see `assets/NOTICE`; CC BY-SA 3.0).
#[cfg(feature = "bundled-contexts")]
const SCHEMA_ORG_CONTEXT: &[u8] = include_bytes!("../../assets/schema.org.context.jsonld");

/// The `@context` URLs a document may reference for schema.org.
#[cfg(feature = "bundled-contexts")]
const SCHEMA_ORG_URLS: &[&str] = &[
    "https://schema.org/",
    "https://schema.org",
    "http://schema.org/",
    "http://schema.org",
];

/// The vendored ActivityStreams 2.0 context (see `assets/NOTICE`; W3C license).
#[cfg(feature = "bundled-contexts")]
const ACTIVITYSTREAMS_CONTEXT: &[u8] =
    include_bytes!("../../assets/activitystreams.context.jsonld");

#[cfg(feature = "bundled-contexts")]
const ACTIVITYSTREAMS_URLS: &[&str] = &[
    "https://www.w3.org/ns/activitystreams",
    "http://www.w3.org/ns/activitystreams",
];

/// A small Mere-authored Dublin Core context (MPL; not a third-party asset).
#[cfg(feature = "bundled-contexts")]
const DUBLIN_CORE_CONTEXT: &[u8] = include_bytes!("../../assets/dublin-core.context.jsonld");

#[cfg(feature = "bundled-contexts")]
const DUBLIN_CORE_URLS: &[&str] = &[
    "http://purl.org/dc/terms/",
    "http://purl.org/dc/elements/1.1/",
];

#[path = "ingest/contribution.rs"]
mod contribution;

use contribution::collect_contribution;

#[path = "ingest/contexts.rs"]
mod contexts;

pub use contexts::is_bundled_context;
