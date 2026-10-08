// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use kernel::graph::Graph;
use kernel::types::GraphScope;

/// [`crate::to_jsonld`] emits. One recognized predicate (`rel#cites`) and one
/// raw predicate (`schema:citation`).
const SAMPLE: &[u8] = br#"[
  {
    "@id": "https://a.test/",
    "@type": ["https://schema.org/Article"],
    "https://schema.org/name": [{"@value": "Article A"}],
    "https://schema.org/keywords": [{"@value": "research"}],
    "https://mere.computer/ns/rel#cites": [{"@id": "https://b.test/"}],
    "https://schema.org/citation": [{"@id": "https://c.test/"}]
  }
]"#;

#[test]
fn rdf_subject_identity_is_exact_by_default_and_pages_are_explicit() {
    use kernel::graph::ResourceNode;
    use oxrdf::{GraphName, Literal, NamedNode};
    let terms = ["https://vocab.test/terms#A", "https://vocab.test/terms#B"];
    let pages = [
        "https://PAGE.test:443/item#first",
        "https://page.test/item#second",
    ];
    let quads = terms.into_iter().chain(pages).map(|iri| {
        Quad::new(
            NamedNode::new(iri).unwrap(),
            NamedNode::new(SCHEMA_NAME).unwrap(),
            Literal::new_simple_literal(iri),
            GraphName::DefaultGraph,
        )
    });
    let contribution = from_quads(quads, "identity-input").unwrap();
    assert_eq!(contribution.nodes.len(), 4);
    assert!(contribution.edges.is_empty());
    assert!(
        contribution.nodes.iter().all(|node| node.types.is_empty()),
        "foreign subjects carry no entity/page hint"
    );
    let mut graph = Graph::new();
    let outcome = apply_contribution(&mut graph, &contribution);
    assert_eq!(outcome.nodes_created, 4);
    assert_eq!(outcome.edges_skipped, 0);
    let surface = |iri| graph.get_node_by_url(iri).unwrap().0;
    assert_ne!(
        graph.get_node(surface(terms[0])).unwrap().id,
        graph.get_node(surface(terms[1])).unwrap().id
    );
    let shown = |iri| graph.shown_resource_id(surface(iri)).unwrap();
    assert_ne!(
        shown(terms[0]),
        shown(terms[1]),
        "generic RDF preserves vocabulary fragments"
    );
    assert_ne!(
        shown(pages[0]),
        shown(pages[1]),
        "a URL shape alone does not infer a page subject"
    );
    assert_ne!(shown(terms[0]), shown(pages[0]));
    assert_eq!(graph.resource_nodes().count(), 4);
    let exact = terms.map(ResourceNode::for_term);
    assert_ne!(exact[0].id(), exact[1].id());
    assert!(graph.resource(ResourceNode::new(terms[0]).id()).is_none());
    for iri in terms.into_iter().chain(pages) {
        let resource = graph.resource(shown(iri)).unwrap();
        assert_eq!(resource.id(), ResourceNode::for_term(iri).id());
        assert_eq!(resource.canonical_iri(), iri);
        assert_eq!(graph.get_node(surface(iri)).unwrap().url(), iri);
    }
    let restored = Graph::try_from_snapshot(&graph.to_snapshot()).unwrap();
    assert_eq!(restored.resource_nodes().count(), 4);
    for resource in exact {
        assert_eq!(
            restored.resource(resource.id()).unwrap().canonical_iri(),
            resource.canonical_iri(),
            "exact term storage retains both identities"
        );
    }

    let mut mixed = Graph::new();
    let outcome = apply_contribution_with_identity(&mut mixed, &contribution, |node| {
        if pages.contains(&node.id.as_str()) {
            SubjectIdentity::Page
        } else {
            SubjectIdentity::ExactIri
        }
    });
    assert_eq!(outcome.nodes_created, 4);
    assert_eq!(mixed.node_count(), 4);
    assert_eq!(mixed.resource_nodes().count(), 3);
    let shown = |iri| {
        mixed
            .shown_resource_id(mixed.get_node_by_url(iri).unwrap().0)
            .unwrap()
    };
    assert_ne!(shown(terms[0]), shown(terms[1]));
    assert_eq!(shown(pages[0]), shown(pages[1]));
    assert_eq!(shown(pages[0]), ResourceNode::new(pages[0]).id());
    assert_eq!(
        mixed.resource(shown(pages[0])).unwrap().canonical_iri(),
        "https://page.test/item"
    );
    assert!(mixed.resource(ResourceNode::new(terms[0]).id()).is_none());
    for iri in terms {
        assert_eq!(mixed.resource(shown(iri)).unwrap().canonical_iri(), iri);
    }
    let records = mixed.to_snapshot().resources;
    let bindings = mixed.to_snapshot().shown_resources;
    let repeat =
        apply_contribution_with_identity(&mut mixed, &contribution, |_| SubjectIdentity::Page);
    assert_eq!(repeat.nodes_created, 0);
    assert_eq!(mixed.to_snapshot().resources, records);
    assert_eq!(
        mixed.to_snapshot().shown_resources,
        bindings,
        "existing prepared bindings retain their intent"
    );
}

#[test]
fn exact_subject_binding_replays_without_an_implicit_page_resource() {
    use kernel::graph::ResourceNode;
    use kernel::graph::capture::{CapturedDelta, replay_captured_deltas};
    use std::sync::{Arc, Mutex};

    let iri = "https://vocab.test/terms#Case";
    let contribution = GraphContribution {
        nodes: vec![NodeContribution::new(iri)],
        edges: vec![],
    };
    let mut graph = Graph::new();
    let seen = Arc::new(Mutex::new(Vec::<CapturedDelta>::new()));
    let sink = seen.clone();
    graph.set_recorder(Some(Arc::new(move |delta| {
        sink.lock().unwrap().push(delta.clone())
    })));
    assert_eq!(
        apply_contribution(&mut graph, &contribution).nodes_created,
        1
    );
    let captures = seen.lock().unwrap().clone();
    let surface_id = Graph::node_namespace_id(iri);
    let exact = ResourceNode::for_term(iri);
    assert!(matches!(&captures[..], [
        CapturedDelta::ReplayAddNodeWithIdIfMissing { id, url, .. },
        CapturedDelta::ReplayTouchNodeLastVisitedById { node_id, .. },
        CapturedDelta::ReplaySetResourceRecordById { resource_id, record: Some(record) },
        CapturedDelta::ReplaySetShownResourceById { surface_id: shown_surface, resource_id: Some(shown) },
    ] if id == &surface_id.to_string()
        && url == iri
        && node_id == id
        && resource_id == &exact.id().to_string()
        && record.canonical_iri == iri
        && shown_surface == id
        && shown == resource_id));
    let replayed = replay_captured_deltas(captures);
    assert_eq!(replayed.node_count(), 1);
    assert_eq!(replayed.resource_nodes().count(), 1);
    assert_eq!(replayed.resource(exact.id()).unwrap().canonical_iri(), iri);
    assert!(replayed.resource(ResourceNode::new(iri).id()).is_none());
    assert_eq!(
        replayed.to_snapshot().resources,
        graph.to_snapshot().resources
    );
    assert_eq!(
        replayed.to_snapshot().shown_resources,
        graph.to_snapshot().shown_resources
    );
    assert_eq!(
        replayed.node_last_visited(replayed.get_node_key_by_id(surface_id).unwrap()),
        graph.node_last_visited(graph.get_node_key_by_id(surface_id).unwrap())
    );

    seen.lock().unwrap().clear();
    assert_eq!(
        apply_contribution(&mut graph, &contribution).nodes_created,
        0
    );
    assert!(
        seen.lock().unwrap().is_empty(),
        "same prepared subject is a true no-op"
    );
}

#[test]
fn subject_identity_preserves_existing_resource_metadata_and_shown_binding() {
    use kernel::graph::ResourceNode;
    use kernel::graph::apply::{GraphDelta, apply_graph_delta};
    use kernel::persistence::{PersistedResourceFacet, PersistedResourceRecord};

    let iri = "https://vocab.test/terms#A";
    let prepared = ResourceNode::for_term(iri);
    let record = PersistedResourceRecord {
        canonical_iri: iri.into(),
        facets: vec![PersistedResourceFacet {
            facet: "source.record".into(),
            value_json: r#"{"author":"retained","revision":7}"#.into(),
        }],
    };
    let mut graph = Graph::new();
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplaySetResourceRecordById {
            resource_id: prepared.id(),
            record: Some(record.clone()),
        },
    );
    let surface_id = Graph::node_namespace_id(iri);
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplayAddNodeWithIdIfMissing {
            id: surface_id,
            url: iri.into(),
            position: Default::default(),
        },
    );
    let key = graph.get_node_key_by_id(surface_id).unwrap();
    assert!(graph.shown_resource_id(key).is_none());
    let contribution = GraphContribution {
        nodes: vec![NodeContribution::new(iri)],
        edges: vec![],
    };
    assert_eq!(
        apply_contribution(&mut graph, &contribution).nodes_created,
        0
    );
    assert_eq!(graph.shown_resource_id(key), Some(prepared.id()));
    assert_eq!(
        graph
            .to_snapshot()
            .resources
            .into_iter()
            .find(|resource| resource.canonical_iri == iri),
        Some(record.clone())
    );
    assert_eq!(graph.resource_nodes().count(), 1);
    assert!(graph.resource(ResourceNode::new(iri).id()).is_none());

    let alternate = ResourceNode::for_term("https://prepared.test/#elsewhere");
    let alternate_record = PersistedResourceRecord {
        canonical_iri: alternate.canonical_iri().into(),
        facets: vec![],
    };
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplaySetResourceRecordById {
            resource_id: alternate.id(),
            record: Some(alternate_record.clone()),
        },
    );
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplaySetShownResourceById {
            surface_id,
            resource_id: Some(alternate.id()),
        },
    );
    let before = graph.revision();
    let outcome =
        apply_contribution_with_identity(&mut graph, &contribution, |_| SubjectIdentity::Page);
    assert_eq!(outcome.nodes_created, 0);
    assert_eq!(graph.revision(), before);
    assert_eq!(graph.shown_resource_id(key), Some(alternate.id()));
    let records = graph.to_snapshot().resources;
    assert_eq!(
        records
            .iter()
            .find(|resource| resource.canonical_iri == alternate.canonical_iri()),
        Some(&alternate_record)
    );
    assert_eq!(
        records
            .iter()
            .find(|resource| resource.canonical_iri == iri),
        Some(&record)
    );
    assert_eq!(graph.resource_nodes().count(), 2);
    assert!(graph.resource(ResourceNode::new(iri).id()).is_none());
}

#[test]
fn blank_nodes_skolemize_under_a_document_namespace() {
    // A blank node becomes `urn:mere:bnode:<doc-namespace>:<label>`. The
    // namespace is stable per document content; oxjsonld assigns the label
    // fresh per parse (so blanks are not idempotent across re-ingests without
    // canonicalization), but distinct documents get distinct namespaces.
    let blank = |doc: &[u8]| {
        from_jsonld(doc)
            .unwrap()
            .edges
            .iter()
            .find(|e| e.predicate.ends_with("#cites"))
            .expect("cites edge")
            .object
            .clone()
    };
    let doc_a = br#"{"@context":{"cites":"https://mere.computer/ns/rel#cites"},"@id":"https://a.test/","cites":{"@type":"https://schema.org/Thing"}}"#;
    let doc_b = br#"{"@context":{"cites":"https://mere.computer/ns/rel#cites"},"@id":"https://b.test/","cites":{"@type":"https://schema.org/Thing"}}"#;
    let a = blank(doc_a);
    assert!(a.starts_with("urn:mere:bnode:"), "skolemized: {a}");
    // Everything before the final `:` is the `urn:mere:bnode:<namespace>`
    // prefix; the label after it is what oxjsonld varies per parse.
    let prefix = |iri: &str| iri.rsplit_once(':').map(|(p, _)| p.to_string()).unwrap();
    assert_eq!(
        prefix(&a),
        prefix(&blank(doc_a)),
        "namespace is content-stable"
    );
    assert_ne!(
        prefix(&a),
        prefix(&blank(doc_b)),
        "different doc, different namespace"
    );
}

#[test]
fn from_jsonld_parses_nodes_literals_types_and_edges() {
    let contribution = from_jsonld(SAMPLE).expect("valid JSON-LD");

    assert_eq!(
        contribution.nodes,
        vec![
            NodeContribution {
                id: "https://a.test/".into(),
                types: vec!["https://schema.org/Article".into()],
                title: Some("Article A".into()),
                tags: vec!["research".into()],
                properties: vec![],
            },
            NodeContribution::new("https://b.test/"),
            NodeContribution::new("https://c.test/"),
        ]
    );

    assert_eq!(contribution.edges.len(), 2);
    assert!(contribution.edges.contains(&EdgeContribution {
        subject: "https://a.test/".into(),
        predicate: "https://mere.computer/ns/rel#cites".into(),
        object: "https://b.test/".into(),
        graph_scope: GraphScope::Default,
        statement_id: None,
        label: None,
        provenance_iri: None,
        asserted_at_ms: None,
    }));
    assert!(contribution.edges.contains(&EdgeContribution {
        subject: "https://a.test/".into(),
        predicate: "https://schema.org/citation".into(),
        object: "https://c.test/".into(),
        graph_scope: GraphScope::Default,
        statement_id: None,
        label: None,
        provenance_iri: None,
        asserted_at_ms: None,
    }));
}

#[test]
fn apply_materializes_recognized_and_raw_edges() {
    use kernel::graph::{EdgeFamily, Graph, RelationSelector, SemanticSubKind};

    let contribution = from_jsonld(SAMPLE).expect("valid JSON-LD");
    let mut graph = Graph::new();
    let outcome = apply_contribution(&mut graph, &contribution);

    assert_eq!(outcome.nodes_created, 3);
    assert_eq!(outcome.edges_asserted, 2);
    assert_eq!(outcome.edges_skipped, 0);

    // Curated literals landed on the subject node.
    let (a, node_a) = graph.get_node_by_url("https://a.test/").expect("node a");
    assert_eq!(node_a.title, "Article A");
    assert!(node_a.tags.contains("research"));
    let (b, _) = graph.get_node_by_url("https://b.test/").expect("node b");
    let (c, _) = graph.get_node_by_url("https://c.test/").expect("node c");

    // Recognized predicate → typed Semantic edge with canonical IRI.
    let cites = graph
        .get_edge(graph.find_edge_key(a, b).expect("a→b"))
        .unwrap();
    assert!(cites.has_relation(RelationSelector::Semantic(SemanticSubKind::Cites)));
    assert_eq!(
        cites.semantic_data().and_then(|d| d.predicate.as_deref()),
        Some("https://mere.computer/ns/rel#cites")
    );

    // Raw predicate → open-predicate Semantic edge (no sub-kinds).
    let citation = graph
        .get_edge(graph.find_edge_key(a, c).expect("a→c"))
        .unwrap();
    assert!(citation.has_relation(RelationSelector::Family(EdgeFamily::Semantic)));
    assert!(
        citation
            .semantic_data()
            .is_some_and(|d| d.sub_kinds.is_empty())
    );
    assert_eq!(
        citation
            .semantic_data()
            .and_then(|d| d.predicate.as_deref()),
        Some("https://schema.org/citation")
    );
}

#[test]
fn from_jsonld_preserves_typed_and_language_tagged_literals() {
    let doc = br#"{
      "@id":"https://a.test/",
      "https://schema.org/datePublished":[{"@value":"2026-06-02","@type":"http://www.w3.org/2001/XMLSchema#date"}],
      "https://schema.org/headline":[{"@value":"Bonjour","@language":"fr"}]
    }"#;
    let contribution = from_jsonld(doc).expect("valid JSON-LD");
    let node = contribution
        .nodes
        .iter()
        .find(|n| n.id == "https://a.test/")
        .expect("node a");
    assert_eq!(node.properties.len(), 2);
    assert_eq!(
        node.properties[0].predicate,
        "https://schema.org/datePublished"
    );
    assert_eq!(node.properties[0].value, "2026-06-02");
    assert_eq!(
        node.properties[0].datatype.as_deref(),
        Some("http://www.w3.org/2001/XMLSchema#date")
    );
    assert_eq!(node.properties[0].lang, None);
    assert_eq!(node.properties[0].graph_scope, GraphScope::Default);
    assert!(!node.properties[0].statement_id.is_empty());
    assert_eq!(node.properties[0].provenance_iri, None);
    assert_eq!(node.properties[0].asserted_at_ms, None);

    assert_eq!(node.properties[1].predicate, "https://schema.org/headline");
    assert_eq!(node.properties[1].value, "Bonjour");
    assert_eq!(node.properties[1].datatype, None);
    assert_eq!(node.properties[1].lang.as_deref(), Some("fr"));
    assert_eq!(node.properties[1].graph_scope, GraphScope::Default);
    assert!(!node.properties[1].statement_id.is_empty());
    assert_eq!(node.properties[1].provenance_iri, None);
    assert_eq!(node.properties[1].asserted_at_ms, None);
}

#[test]
fn a_harvested_hyperlink_records_extracted_from_provenance_on_the_target() {
    use kernel::graph::{Graph, ProvenanceSubKind};

    // The shape a materialize / crawl contribution carries: a source page
    // linking to a target (capture plan C3).
    let contribution = GraphContribution {
        nodes: vec![
            NodeContribution::new("https://src.test/"),
            NodeContribution::new("https://dst.test/"),
        ],
        edges: vec![EdgeContribution {
            subject: "https://src.test/".to_string(),
            predicate: "https://mere.computer/ns/rel#hyperlink".to_string(),
            object: "https://dst.test/".to_string(),
            graph_scope: GraphScope::Default,
            statement_id: None,
            label: None,
            provenance_iri: None,
            asserted_at_ms: None,
        }],
    };
    let mut graph = Graph::new();
    apply_contribution(&mut graph, &contribution);

    let (src_key, src) = graph
        .get_node_by_url("https://src.test/")
        .expect("source node");
    let (dst_key, _dst) = graph
        .get_node_by_url("https://dst.test/")
        .expect("target node");

    // The target names the page it was extracted from; the source carries none.
    assert!(
        graph.node_derivations(src_key).unwrap().is_empty(),
        "the source page is not derived"
    );
    let derivations = graph.node_derivations(dst_key).unwrap();
    assert_eq!(derivations.len(), 1, "the target carries one derivation");
    let d = &derivations[0];
    assert_eq!(d.sub_kind, ProvenanceSubKind::ExtractedFrom);
    assert_eq!(
        d.source_node,
        src.id.to_string(),
        "the derivation names the source node"
    );
    assert_eq!(d.source_graph, None, "same-graph derivation");
}

const REMOTE_DOC: &[u8] = br#"{
  "@context": "https://ctx.test/v1",
  "@id": "https://a.test/",
  "name": "Article A",
  "cites": {"@id": "https://b.test/"}
}"#;

const BUNDLED_CONTEXT: &[u8] = br#"{
  "@context": {
    "name": "https://schema.org/name",
    "cites": "https://mere.computer/ns/rel#cites"
  }
}"#;

#[test]
fn context_presets_resolve_mere_docs() {
    let doc = br#"{"@context":"https://mere.computer/ns/context","@id":"https://a.test/","name":"A","mere:cites":{"@id":"https://b.test/"}}"#;
    // minimal + full resolve the Mere context; none refuses the remote URL.
    let resolved = from_jsonld_with_contexts(doc, ContextCache::minimal()).expect("minimal");
    assert!(
        resolved
            .nodes
            .iter()
            .any(|n| n.id == "https://a.test/" && n.title.as_deref() == Some("A"))
    );
    assert!(
        resolved
            .edges
            .iter()
            .any(|e| e.predicate == "https://mere.computer/ns/rel#cites")
    );
    assert!(from_jsonld_with_contexts(doc, ContextCache::full()).is_ok());
    assert!(from_jsonld_with_contexts(doc, ContextCache::new()).is_err());
}

#[cfg(feature = "bundled-contexts")]
#[test]
fn full_pack_resolves_a_schema_org_remote_context() {
    // A page referencing schema.org's remote context resolves offline, and
    // schema.org's http `@vocab` is normalized to https.
    let doc = br#"{"@context":"https://schema.org/","@id":"https://a.test/","name":"Article A","datePublished":"2026-06-02"}"#;
    let contribution =
        from_jsonld_with_contexts(doc, ContextCache::full()).expect("schema.org resolved offline");
    let node = contribution
        .nodes
        .iter()
        .find(|n| n.id == "https://a.test/")
        .expect("node a");
    assert_eq!(node.title.as_deref(), Some("Article A"));
    assert!(node.properties.iter().any(|property| {
        property.predicate == "https://schema.org/datePublished"
            && property.value == "2026-06-02"
            && property.datatype.as_deref() == Some("https://schema.org/Date")
            && property.lang.is_none()
    }));
}

#[cfg(feature = "bundled-contexts")]
#[test]
fn full_pack_resolves_activitystreams() {
    // A fediverse-style object: as:name -> title, as:inReplyTo (an @id term)
    // -> a Semantic edge.
    let doc = br#"{"@context":"https://www.w3.org/ns/activitystreams","@id":"https://x.test/n1","name":"Hello","inReplyTo":"https://y.test/n0"}"#;
    let c = from_jsonld_with_contexts(doc, ContextCache::full()).expect("AS2 resolved offline");
    let node = c
        .nodes
        .iter()
        .find(|n| n.id == "https://x.test/n1")
        .expect("node n1");
    assert_eq!(node.title.as_deref(), Some("Hello"));
    assert!(
        c.edges
            .iter()
            .any(|e| e.object == "https://y.test/n0" && e.predicate.contains("inReplyTo"))
    );
}

#[test]
fn ingested_node_ids_are_deterministic_across_graphs() {
    // Two hosts ingesting the same document mint the same node ids (federation
    // identity), each derived from its `@id`.
    let doc =
        br#"{"@context":{"name":"https://schema.org/name"},"@id":"https://x.test/","name":"X"}"#;
    let contribution = from_jsonld(doc).expect("parsed");
    let mut g1 = Graph::new();
    let mut g2 = Graph::new();
    apply_contribution(&mut g1, &contribution);
    apply_contribution(&mut g2, &contribution);
    let id1 = g1
        .get_node_by_url("https://x.test/")
        .expect("node in g1")
        .1
        .id;
    let id2 = g2
        .get_node_by_url("https://x.test/")
        .expect("node in g2")
        .1
        .id;
    assert_eq!(id1, id2, "same @id yields the same node id on every host");
    assert_eq!(id1, Graph::node_namespace_id("https://x.test/"));
}

#[test]
fn dublin_core_literals_are_recognized() {
    // dcterms:title -> title, dcterms:subject -> tag. Recognition works on an
    // inline prefix, with no remote context fetched.
    let doc = br#"{"@context":{"dcterms":"http://purl.org/dc/terms/"},"@id":"https://d.test/","dcterms:title":"Doc","dcterms:subject":"alpha"}"#;
    let c = from_jsonld(doc).expect("DC parsed");
    let node = c
        .nodes
        .iter()
        .find(|n| n.id == "https://d.test/")
        .expect("node d");
    assert_eq!(node.title.as_deref(), Some("Doc"));
    assert!(node.tags.iter().any(|t| t == "alpha"));
}

#[test]
fn referenced_context_urls_finds_remote_refs() {
    let doc = br#"{"@context":["https://schema.org/",{"x":"https://ex/#x"}],"@graph":[{"@context":"https://www.w3.org/ns/activitystreams","@id":"a"}]}"#;
    let urls = referenced_context_urls(doc);
    assert!(urls.contains(&"https://schema.org/".to_string()));
    assert!(urls.contains(&"https://www.w3.org/ns/activitystreams".to_string()));
    assert_eq!(
        urls.len(),
        2,
        "inline object entries are not URL references"
    );
}

#[test]
fn referenced_context_urls_ignores_inline_and_nonjson() {
    let inline = br#"{"@context":{"name":"https://schema.org/name"},"@id":"a"}"#;
    assert!(
        referenced_context_urls(inline).is_empty(),
        "inline context has no remote ref"
    );
    assert!(referenced_context_urls(b"not json at all").is_empty());
}

#[cfg(feature = "bundled-contexts")]
#[test]
fn is_bundled_context_covers_the_packs() {
    assert!(is_bundled_context("https://schema.org/"));
    assert!(is_bundled_context("https://www.w3.org/ns/activitystreams"));
    assert!(is_bundled_context("http://purl.org/dc/terms/"));
    assert!(!is_bundled_context("https://example.com/ctx"));
}

#[test]
fn bundled_context_expands_a_remote_context() {
    let cache = ContextCache::new().with("https://ctx.test/v1", BUNDLED_CONTEXT);
    let contribution = from_jsonld_with_contexts(REMOTE_DOC, cache).expect("context resolved");

    // `name` expands to schema:name (a title); `cites` to the Mere predicate
    // (an edge) — both via the bundled context, no network.
    let a = contribution
        .nodes
        .iter()
        .find(|n| n.id == "https://a.test/")
        .expect("node a");
    assert_eq!(a.title.as_deref(), Some("Article A"));
    assert!(contribution.edges.contains(&EdgeContribution {
        subject: "https://a.test/".into(),
        predicate: "https://mere.computer/ns/rel#cites".into(),
        object: "https://b.test/".into(),
        graph_scope: GraphScope::Default,
        statement_id: None,
        label: None,
        provenance_iri: None,
        asserted_at_ms: None,
    }));
}

#[test]
fn unbundled_remote_context_is_refused() {
    // Empty cache → the remote @context cannot be resolved → ingest errors.
    assert!(matches!(
        from_jsonld_with_contexts(REMOTE_DOC, ContextCache::new()),
        Err(IngestError::Parse(_))
    ));
    // The network-free parser refuses a remote @context outright.
    assert!(matches!(
        from_jsonld(REMOTE_DOC),
        Err(IngestError::Parse(_))
    ));
}

#[test]
fn every_ingest_assertion_path_supplies_source_or_author() {
    use kernel::graph::Author;
    let author = Author::engine("jsonld", "1");
    for recognized in [false, true] {
        for metadata in [0, 1, 2] {
            for source in [None, Some("https://source.test/")] {
                let contribution = GraphContribution {
                    nodes: vec![
                        NodeContribution::new("https://a.test/"),
                        NodeContribution::new("https://b.test/"),
                    ],
                    edges: vec![EdgeContribution {
                        subject: "https://a.test/".into(),
                        object: "https://b.test/".into(),
                        predicate: if recognized {
                            "https://mere.computer/ns/rel#cites"
                        } else {
                            "https://example.test/rel"
                        }
                        .into(),
                        graph_scope: GraphScope::Source,
                        statement_id: (metadata == 2).then(|| "imported".into()),
                        label: (metadata == 1).then(|| "label".into()),
                        provenance_iri: source.map(str::to_owned),
                        asserted_at_ms: None,
                    }],
                };
                let mut graph = Graph::new();
                let outcome = graph.write_as(author.clone(), |graph| {
                    apply_contribution(graph, &contribution)
                });
                assert_eq!(outcome.edges_asserted, 1);
                assert_eq!(outcome.edges_skipped, 0);
                let from = graph.get_node_by_url("https://a.test/").unwrap().0;
                let to = graph.get_node_by_url("https://b.test/").unwrap().0;
                let statements = graph
                    .get_edge(graph.find_edge_key(from, to).unwrap())
                    .unwrap()
                    .semantic_statements();
                assert_eq!(statements.len(), 1);
                let fallback = author.asserter_iri();
                assert_eq!(
                    statements[0].provenance_iri.as_deref(),
                    Some(source.unwrap_or(&fallback))
                );
                assert_eq!(statements[0].graph_scope, GraphScope::Source);
            }
        }
    }
}
