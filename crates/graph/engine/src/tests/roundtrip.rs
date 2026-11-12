// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn properties_round_trip_via_the_property_bag() {
    // A non-curated literal survives ingest → graph property bag → export.
    let doc =
        br#"{"@id":"https://a.test","https://schema.org/datePublished":[{"@value":"2026-06-02"}]}"#;
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
    use super::super::{GRAPH_SCOPE_USER, RDF_REIFIES};
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

    let contribution = crate::acquire::ingest::from_quads_envelope(crate::dataset_quads(&graph), "gate")
        .expect("quad ingest");
    let mut reimported = Graph::new();
    let outcome = crate::acquire::ingest::apply_import(&mut reimported, &contribution);
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
