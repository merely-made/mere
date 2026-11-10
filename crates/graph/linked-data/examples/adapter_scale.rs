// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Scale measurement for the SPARQL dataset.
//!
//! Builds a chain graph of N nodes (each node `next`-links to its successor and
//! carries one literal property), then times four queries three ways: through
//! `linked_data::query::sparql` (which indexes the graph on every call), over a
//! held `IndexedDataset` built once (`sparql_on`), and over an `oxrdf::Dataset`
//! materialized from the same projection, kept as the parity column. It also
//! prints the index build time and the index's `memory_bytes()`.
//!
//! `cargo run --release -p mere-rdf --features query --example adapter_scale -- 1000 10000 100000`
//!
//! A query that took longer than the budget at one scale is skipped at the next.
//! The borrowed adapter the first measurement timed was deleted with P2.

use kernel::graph::fixtures::GraphFixtures;
use kernel::graph::{Graph, SemanticStatementSpec};
use kernel::types::NodeProperty;
use linked_data::query::{IndexedDataset, View, sparql, sparql_on};
use spareval::{QueryEvaluator, QueryResults};
use spargebra::SparqlParser;
use std::time::{Duration, Instant};

const NEXT: &str = "https://example.test/vocab#next";
const BUDGET: Duration = Duration::from_secs(20);

const QUERIES: &[(&str, &str)] = &[
    (
        "bound predicate",
        "SELECT (COUNT(*) AS ?n) WHERE { ?s <https://example.test/vocab#next> ?o }",
    ),
    (
        "point lookup",
        "SELECT ?o WHERE { <https://n.test/42> <https://example.test/vocab#next> ?o }",
    ),
    (
        "two-hop join",
        "SELECT (COUNT(*) AS ?n) WHERE { ?a <https://example.test/vocab#next> ?b . ?b <https://example.test/vocab#next> ?c }",
    ),
    ("full scan", "SELECT (COUNT(*) AS ?n) WHERE { ?s ?p ?o }"),
];

fn build(n: usize) -> Graph {
    let mut graph = Graph::new();
    let t = Instant::now();
    let keys: Vec<_> = (0..n)
        .map(|i| graph.add_node(format!("https://n.test/{i}"), Default::default()))
        .collect();
    let nodes_ms = t.elapsed().as_secs_f64() * 1000.0;
    let t = Instant::now();
    for (i, key) in keys.iter().enumerate() {
        let to = keys[(i + 1) % n];
        graph.assert_semantic_statement(
            *key,
            to,
            SemanticStatementSpec {
                predicate: NEXT.to_string(),
                ..Default::default()
            },
        );
    }
    let statements_ms = t.elapsed().as_secs_f64() * 1000.0;
    let t = Instant::now();
    for (i, key) in keys.iter().enumerate() {
        let property = NodeProperty::new(
            "https://schema.org/position".to_string(),
            i.to_string(),
        );
        graph.append_node_properties(*key, vec![property]);
    }
    let properties_ms = t.elapsed().as_secs_f64() * 1000.0;
    eprintln!("build {n}: add_node {nodes_ms:.1} ms, assert_semantic_statement {statements_ms:.1} ms, append_node_properties {properties_ms:.1} ms");
    graph
}

fn materialized_rows(dataset: &oxrdf::Dataset, query: &str) -> Result<usize, String> {
    let query = SparqlParser::new()
        .parse_query(query)
        .map_err(|e| e.to_string())?;
    let results = QueryEvaluator::new()
        .prepare(&query)
        .execute(dataset)
        .map_err(|e| e.to_string())?;
    match results {
        QueryResults::Solutions(solutions) => {
            let mut count = 0;
            for solution in solutions {
                solution.map_err(|e| e.to_string())?;
                count += 1;
            }
            Ok(count)
        },
        QueryResults::Boolean(_) => Ok(1),
        QueryResults::Graph(_) => Err("graph results unsupported".to_string()),
    }
}

fn ms(d: Duration) -> String {
    format!("{:.2}", d.as_secs_f64() * 1000.0)
}

fn main() {
    let scales: Vec<usize> = std::env::args()
        .skip(1)
        .filter_map(|a| a.parse().ok())
        .collect();
    let scales = if scales.is_empty() {
        vec![1_000, 10_000, 100_000]
    } else {
        scales
    };
    let mut over_budget = vec![false; QUERIES.len()];

    println!(
        "scale\tquads\tgraph_build_ms\tindex_build_ms\tindex_bytes\tmaterialize_ms\tquery\tsparql_call_ms\tindexed_ms\tmaterialized_ms\trows"
    );
    for n in scales {
        let t = Instant::now();
        let graph = build(n);
        let build_ms = t.elapsed();

        let t = Instant::now();
        let quads = linked_data::dataset_quads(&graph);
        let quad_count = quads.len();
        let dataset: oxrdf::Dataset = quads.into_iter().collect();
        let materialize_ms = t.elapsed();

        let t = Instant::now();
        let indexed = IndexedDataset::for_graph(&graph);
        let index_ms = t.elapsed();
        let prefix = format!(
            "{n}\t{quad_count}\t{}\t{}\t{}\t{}",
            ms(build_ms),
            ms(index_ms),
            indexed.memory_bytes(),
            ms(materialize_ms)
        );

        for (i, (name, query)) in QUERIES.iter().enumerate() {
            if over_budget[i] {
                println!("{prefix}\t{name}\tskipped\t\t\t");
                continue;
            }
            let t = Instant::now();
            let call = sparql(&graph, query).map(|r| r.rows.len());
            let call_ms = t.elapsed();
            let coverage = graph.coverage_note();
            let t = Instant::now();
            let held = sparql_on(&indexed, query, View::Live, coverage)
                .map(|r| r.rows.len());
            let indexed_ms = t.elapsed();
            let t = Instant::now();
            let materialized = materialized_rows(&dataset, query);
            let materialized_ms = t.elapsed();
            let rows = match (&call, &held, &materialized) {
                (Ok(a), Ok(b), Ok(c)) if a == b && b == c => a.to_string(),
                (a, b, c) => format!("MISMATCH call={a:?} indexed={b:?} materialized={c:?}"),
            };
            println!(
                "{prefix}\t{name}\t{}\t{}\t{}\t{rows}",
                ms(call_ms),
                ms(indexed_ms),
                ms(materialized_ms)
            );
            if call_ms > BUDGET {
                over_budget[i] = true;
            }
        }
    }
}
