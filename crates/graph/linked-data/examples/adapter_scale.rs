// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Scale measurement for the borrowed SPARQL adapter.
//!
//! Builds a chain graph of N nodes (each node `next`-links to its successor and
//! carries one literal property), then times four queries two ways: through the
//! borrowed kernel adapter (`linked_data::query::sparql`) and over an indexed
//! in-memory `oxrdf::Dataset` materialized from the same projection. The second
//! is the floor an indexed dataset could reach.
//!
//! `cargo run --release -p mere-rdf --features query --example adapter_scale -- 1000 10000 100000`
//!
//! A query that took longer than the budget at one scale is skipped at the next.

use kernel::graph::fixtures::GraphFixtures;
use kernel::graph::{Graph, SemanticStatementSpec};
use kernel::types::NodeProperty;
use linked_data::query::sparql;
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
    format!("{:.1}", d.as_secs_f64() * 1000.0)
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

    println!("scale\tquads\tbuild_ms\tmaterialize_ms\tquery\tadapter_ms\tindexed_ms\trows");
    for n in scales {
        let t = Instant::now();
        let graph = build(n);
        let build_ms = t.elapsed();

        let t = Instant::now();
        let quads = linked_data::dataset_quads(&graph);
        let quad_count = quads.len();
        let dataset: oxrdf::Dataset = quads.into_iter().collect();
        let materialize_ms = t.elapsed();

        for (i, (name, query)) in QUERIES.iter().enumerate() {
            if over_budget[i] {
                println!("{n}\t{quad_count}\t{}\t{}\t{name}\tskipped\t\t", ms(build_ms), ms(materialize_ms));
                continue;
            }
            let t = Instant::now();
            let adapter = sparql(&graph, query).map(|r| r.rows.len());
            let adapter_ms = t.elapsed();
            let t = Instant::now();
            let indexed = materialized_rows(&dataset, query);
            let indexed_ms = t.elapsed();
            let rows = match (&adapter, &indexed) {
                (Ok(a), Ok(b)) if a == b => a.to_string(),
                (a, b) => format!("MISMATCH adapter={a:?} indexed={b:?}"),
            };
            println!(
                "{n}\t{quad_count}\t{}\t{}\t{name}\t{}\t{}\t{rows}",
                ms(build_ms),
                ms(materialize_ms),
                ms(adapter_ms),
                ms(indexed_ms)
            );
            if adapter_ms > BUDGET {
                over_budget[i] = true;
            }
        }
    }
}
