// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

#[cfg(feature = "query")]
use super::node_quads;
use super::{
    ASSERTION_HISTORY_GRAPH_IRI, GraphContribution, MERE_ASSERTED_GRAPH_KIND,
    MERE_ASSERTED_GRAPH_NAME, MERE_LAST_ASSERTED_AT, MERE_LIFECYCLE_VERSION, MERE_METHOD_VERSION,
    PROV_GENERATED_AT_TIME, PROV_INVALIDATED_AT_TIME, PROV_WAS_ASSOCIATED_WITH,
    PROV_WAS_ATTRIBUTED_TO, PROV_WAS_DERIVED_FROM, PROV_WAS_GENERATED_BY, PROV_WAS_INVALIDATED_BY,
    RDF_REIFIES, RDFS_LABEL, apply_contribution, apply_import, dataset_quads_with_history,
    from_jsonld, from_nquads_envelope, node_id, statement_reifier_id, to_jsonld, to_jsonld_compact,
    to_jsonld_with_history, to_nquads_with_history, to_trig_with_history, withdrawal_activity_id,
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
        ClassificationProvenance, ClassificationScheme, ClassificationStatus, NodeClassification,
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
        },
    );
    let e = graph
        .assert_relation(
            a,
            c,
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Cites,
                label: None,
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
        },
    );
    source.assert_relation(
        b,
        a,
        EdgeAssertion::Semantic {
            sub_kind: SemanticSubKind::UserGrouped,
            label: None,
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

fn instance_objects(document: &serde_json::Value, reifier_count: usize) -> serde_json::Value {
    let nodes = document.as_array().expect("expanded node array");
    let is_reifier = |node: &&serde_json::Value| {
        node["@type"].as_array().is_some_and(|types| {
            types.contains(&serde_json::Value::String(
                crate::formats::jsonld::RDF_STATEMENT.into(),
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

#[path = "tests/history.rs"]
mod history;

#[path = "tests/roundtrip.rs"]
mod roundtrip;

#[cfg(feature = "query")]
#[path = "tests/sparql.rs"]
mod sparql;
