// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::{IndexedDataset, View, sparql, sparql_on};
use kernel::graph::fixtures::GraphFixtures;
use kernel::graph::{Graph, SemanticStatementSpec};
use oxrdf::{GraphName, Literal, NamedNode, Quad};

fn named(iri: &str) -> NamedNode {
    NamedNode::new(iri).unwrap()
}

fn rows(dataset: &IndexedDataset, query: &str, view: View) -> Vec<Vec<Option<String>>> {
    let mut rows = sparql_on(dataset, query, view, Default::default())
        .unwrap()
        .rows;
    rows.sort();
    rows
}

#[test]
fn terms_the_dataset_lacks_match_nothing_and_round_trip() {
    let dataset = IndexedDataset::from_quads([(
        Quad::new(
            named("https://a.test/s"),
            named("https://a.test/p"),
            Literal::new_simple_literal("x"),
            GraphName::DefaultGraph,
        ),
        true,
    )]);
    // A constant the dictionary lacks matches no row and does not panic.
    assert!(
        rows(
            &dataset,
            "SELECT ?o WHERE { <https://missing.test/> ?p ?o }",
            View::Live
        )
        .is_empty()
    );
    // VALUES and computed terms externalize back to themselves, and equal
    // computed terms are one id for grouping.
    assert_eq!(
        rows(
            &dataset,
            "SELECT ?v (COUNT(*) AS ?n) WHERE { VALUES ?k { 1 2 3 } BIND(CONCAT(\"new-\", \"term\") AS ?v) } GROUP BY ?v",
            View::Live,
        ),
        vec![vec![Some("new-term".into()), Some("3".into())]]
    );
    assert_eq!(
        rows(
            &dataset,
            "SELECT ?x WHERE { VALUES ?x { <https://missing.test/> } }",
            View::Live
        ),
        vec![vec![Some("https://missing.test/".into())]]
    );
}

#[test]
fn withdrawn_rows_are_read_by_the_history_view_only() {
    let graph_name = named("https://a.test/g");
    let quad = |object: &str| {
        Quad::new(
            named("https://a.test/s"),
            named("https://a.test/p"),
            named(object),
            graph_name.clone(),
        )
    };
    let dataset = IndexedDataset::from_quads([
        (quad("https://a.test/held"), true),
        (quad("https://a.test/gone"), false),
        (quad("https://a.test/held"), false),
    ]);
    let query = "SELECT ?o WHERE { GRAPH ?g { ?s ?p ?o } }";
    assert_eq!(
        rows(&dataset, query, View::Live),
        vec![vec![Some("https://a.test/held".into())]]
    );
    // A quad carried by a held and a withdrawn row is one solution.
    assert_eq!(rows(&dataset, query, View::History).len(), 2);
    assert!(!dataset.is_empty());
    assert!(dataset.memory_bytes() > 0);
}

#[test]
fn for_graph_is_current_until_the_kernel_counters_move() {
    let mut graph = Graph::new();
    let a = graph.add_node("https://a.test/a".into(), Default::default());
    let b = graph.add_node("https://a.test/b".into(), Default::default());
    let dataset = IndexedDataset::for_graph(&graph);
    assert!(dataset.is_current(&graph));
    graph.assert_semantic_statement(
        a,
        b,
        SemanticStatementSpec {
            predicate: "https://a.test/next".into(),
            ..Default::default()
        },
    );
    assert!(!dataset.is_current(&graph), "a mutation moves the key");
    let query = "SELECT ?o WHERE { <https://a.test/a> <https://a.test/next> ?o }";
    assert!(
        rows(&dataset, query, View::Live).is_empty(),
        "the held dataset is the old revision"
    );
    assert_eq!(sparql(&graph, query).unwrap().rows.len(), 1);
    assert!(!IndexedDataset::from_quads([]).is_current(&graph));
}
