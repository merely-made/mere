// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[cfg(feature = "query")]
#[test]
fn sparql_selects_a_literal_and_an_edge_over_node_quads() {
    let graph = seed();

    // A curated literal: A's schema:name.
    let names = mere::query::sparql(
        &graph,
        "SELECT ?name WHERE { <https://a.test> <https://schema.org/name> ?name }",
    )
    .expect("name query");
    assert_eq!(names.variables, vec!["name".to_string()]);
    assert_eq!(names.rows, vec![vec![Some("Article A".to_string())]]);

    // A recognized semantic edge: A cites B (canonical Mere IRI).
    let cites = mere::query::sparql(
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

    crate::graph::apply::assert_semantic_relation_in_scope(
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

    let scoped_edge = mere::query::sparql(
        &graph,
        "SELECT ?t WHERE { GRAPH <https://mere.computer/ns/graph#source> { <https://a.test> <https://mere.computer/ns/rel#cites> ?t } }",
    )
    .expect("scoped edge query");
    assert_eq!(
        scoped_edge.rows,
        vec![vec![Some("https://b.test".to_string())]]
    );

    let scoped_property = mere::query::sparql(
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

    crate::graph::apply::assert_semantic_relation_in_scope(
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

    let edge_metadata = mere::query::sparql(
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

    let property_metadata = mere::query::sparql(
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
