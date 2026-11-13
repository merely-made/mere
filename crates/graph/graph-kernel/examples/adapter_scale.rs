// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Scale measurement for the SPARQL dataset.
//!
//! Builds a chain graph of N nodes (each node `next`-links to its successor and
//! carries one literal property), then times four queries three ways: through
//! `mere::query::sparql` (which indexes the graph on every call), over a
//! held `IndexedDataset` built once (`sparql_on`), and over an `oxrdf::Dataset`
//! materialized from the same projection, kept as the parity column. It also
//! prints the index build time and the index's `memory_bytes()`.
//!
//! `cargo run --release -p mere-kernel --features query,fixtures --example adapter_scale -- 1000 10000 100000`
//! (in `mere-kernel` since engine split P5 L3: it builds a kernel graph)
//!
//! A query that took longer than the budget at one scale is skipped at the next.
//! The borrowed adapter the first measurement timed was deleted with P2.
//!
//! After each scale's query rows it prints a second table (`edit_scale` header):
//! per-edit assert and withdraw on the built graph, the refresh after one edit
//! (`is_current`, then rebuild of the held index), and the engine's share of a
//! frame measured headlessly. Each row is the median, min and max over its reps
//! within the run; the recorded figure is the median of several runs. The
//! `refresh_incremental_after_*` rows time `IndexedDataset::refresh` from the
//! edit's captured deltas (quad index plan P4), and `refresh_bytes` is what
//! its per-source record adds beside `index_bytes`.

use kernel::graph::fixtures::GraphFixtures;
use kernel::graph::apply::{GraphDelta, apply_graph_delta};
use kernel::graph::{CapturedDelta, Graph, NodeKey, SemanticStatementSpec};
use kernel::rdf::touched_sources;
use kernel::types::NodeProperty;
use mere::dataset::Refresh;
use mere::query::{IndexedDataset, View, sparql, sparql_on};
use spareval::{QueryEvaluator, QueryResults};
use spargebra::SparqlParser;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
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

fn build(n: usize) -> (Graph, Vec<NodeKey>) {
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
        let property = NodeProperty::new("https://schema.org/position".to_string(), i.to_string());
        graph.append_node_properties(*key, vec![property]);
    }
    let properties_ms = t.elapsed().as_secs_f64() * 1000.0;
    eprintln!(
        "build {n}: add_node {nodes_ms:.1} ms, assert_semantic_statement {statements_ms:.1} ms, append_node_properties {properties_ms:.1} ms"
    );
    (graph, keys)
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
        "scale\tquads\tgraph_build_ms\tindex_build_ms\tindex_bytes\trefresh_bytes\tmaterialize_ms\tquery\tsparql_call_ms\tindexed_ms\tmaterialized_ms\trows"
    );
    for n in scales {
        let t = Instant::now();
        let (mut graph, keys) = build(n);
        let build_ms = t.elapsed();

        let t = Instant::now();
        let quads = mere::dataset_quads(&graph);
        let quad_count = quads.len();
        let dataset: oxrdf::Dataset = quads.into_iter().collect();
        let materialize_ms = t.elapsed();

        let t = Instant::now();
        let mut indexed = IndexedDataset::for_graph(&graph);
        let index_ms = t.elapsed();
        let prefix = format!(
            "{n}\t{quad_count}\t{}\t{}\t{}\t{}\t{}",
            ms(build_ms),
            ms(index_ms),
            indexed.memory_bytes(),
            indexed.refresh_bytes(),
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
            let held = sparql_on(&indexed, query, View::Live, coverage).map(|r| r.rows.len());
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
        edit_scale(n, &mut graph, &keys, &mut indexed);
    }
}

static EDIT_SEQ: AtomicUsize = AtomicUsize::new(0);
const POINT: &str = QUERIES[1].1;
const TWO_HOP: &str = QUERIES[2].1;

fn median_row(scale: usize, name: &str, mut samples: Vec<Duration>) {
    samples.sort();
    let at = |i: usize| format!("{:.4}", samples[i].as_secs_f64() * 1000.0);
    println!(
        "{scale}\t{name}\t{}\t{}\t{}\t{}",
        samples.len(),
        at(samples.len() / 2),
        at(0),
        at(samples.len() - 1)
    );
}

fn edit_spec(tag: &str, k: usize) -> SemanticStatementSpec {
    SemanticStatementSpec {
        predicate: format!("https://example.test/vocab#{tag}{k}"),
        ..Default::default()
    }
}

/// Assert one new statement (a predicate no edge carries yet) on a node pair
/// that has no statement; returns its handle.
fn edit_assert(graph: &mut Graph, keys: &[NodeKey], tag: &str) -> String {
    // A fresh predicate per call: a withdrawn statement is never revived by a
    // repeat assert, so reusing one would change nothing.
    let k = EDIT_SEQ.fetch_add(1, Ordering::Relaxed);
    let n = keys.len();
    let (_, outcome) = graph
        .assert_semantic_statement(keys[k % n], keys[(k + 7) % n], edit_spec(tag, k))
        .expect("edit assert applies");
    assert!(outcome.changed, "edit assert must change the graph");
    outcome.statement_id
}

fn query_rows(indexed: &IndexedDataset, graph: &Graph, query: &str) -> usize {
    sparql_on(indexed, query, View::Live, graph.coverage_note())
        .expect("held query")
        .rows
        .len()
}

/// Per-edit, refresh and headless frame-share measurements on a built graph.
fn edit_scale(n: usize, graph: &mut Graph, keys: &[NodeKey], indexed: &mut IndexedDataset) {
    let rebuilds = if n >= 100_000 { 7 } else { 21 };
    let edits = 51;
    println!("scale\tmeasure\treps\tmedian_ms\tmin_ms\tmax_ms\t# edit_scale");

    // Per-edit assert, then per-edit withdraw of the same statements.
    let mut asserts = Vec::new();
    let mut ids = Vec::new();
    for k in 0..edits {
        let t = Instant::now();
        let id = edit_assert(graph, keys, "assertEdit");
        asserts.push(t.elapsed());
        ids.push(id);
    }
    median_row(n, "edit_assert", asserts);
    let mut withdraws = Vec::new();
    for id in &ids {
        let t = Instant::now();
        let applied = graph.withdraw_semantic_assertion(id, None, None);
        withdraws.push(t.elapsed());
        assert!(applied, "withdraw must apply to a held statement");
    }
    median_row(n, "edit_withdraw", withdraws);

    // Refresh after one edit: is_current (stale), rebuild, is_current (fresh).
    // Two edit kinds: one new assertion, one withdrawal.
    for kind in ["assert", "withdraw"] {
        let (mut stale, mut rebuild, mut fresh, mut total) = (vec![], vec![], vec![], vec![]);
        for k in 0..rebuilds {
            let id = edit_assert(graph, keys, "refreshEdit");
            if kind == "withdraw" {
                // Refresh the index to the assert first, untimed, then withdraw.
                *indexed = IndexedDataset::for_graph(graph);
                assert!(graph.withdraw_semantic_assertion(&id, None, None));
            }
            let t0 = Instant::now();
            let was_current = indexed.is_current(graph);
            let t1 = Instant::now();
            assert!(!was_current, "is_current must be false after an edit");
            *indexed = IndexedDataset::for_graph(graph);
            let t2 = Instant::now();
            let now_current = indexed.is_current(graph);
            let t3 = Instant::now();
            assert!(now_current, "is_current must be true after the rebuild");
            stale.push(t1 - t0);
            rebuild.push(t2 - t1);
            fresh.push(t3 - t2);
            total.push(t3 - t0);
        }
        median_row(n, &format!("is_current_stale_after_{kind}"), stale);
        median_row(n, &format!("rebuild_after_{kind}"), rebuild);
        median_row(n, &format!("is_current_fresh_after_{kind}"), fresh);
        median_row(n, &format!("refresh_total_after_{kind}"), total);
    }

    // Refresh from the captured deltas instead of rebuilding.
    let sink: Arc<Mutex<Vec<CapturedDelta>>> = Arc::default();
    let into = sink.clone();
    graph.set_recorder(Some(Arc::new(move |delta: &CapturedDelta| {
        into.lock().unwrap().push(delta.clone());
    })));
    let refresh = |indexed: &mut IndexedDataset, graph: &Graph| {
        let deltas = std::mem::take(&mut *sink.lock().unwrap());
        let touched = touched_sources(&deltas).expect("statement edits refresh in place");
        let outcome = indexed.refresh(graph, &touched);
        assert_eq!(outcome, Refresh::Incremental, "an edit refreshes in place");
    };
    *indexed = IndexedDataset::for_graph(graph);
    sink.lock().unwrap().clear();
    for kind in ["assert", "withdraw"] {
        let mut samples = vec![];
        for _ in 0..edits {
            let id = edit_assert(graph, keys, "incrementalEdit");
            if kind == "withdraw" {
                refresh(indexed, graph);
                assert!(graph.withdraw_semantic_assertion(&id, None, None));
            }
            let t = Instant::now();
            refresh(indexed, graph);
            samples.push(t.elapsed());
        }
        median_row(n, &format!("refresh_incremental_after_{kind}"), samples);
    }
    // Node edits through the captured path: a title, a re-tag (a tag's label is
    // its identity, so changing it means another concept), and a URL, whose
    // IRI change brings the node's dependents.
    for kind in ["title", "retag", "url"] {
        let mut samples = vec![];
        for k in 0..edits {
            let key = keys[(k * 131) % keys.len()];
            match kind {
                "title" => {
                    let title = format!("title {k}");
                    apply_graph_delta(graph, GraphDelta::SetNodeTitle { key, title });
                },
                "retag" => {
                    let tag = format!("tag {k}");
                    apply_graph_delta(graph, GraphDelta::InsertNodeTag { key, tag });
                    refresh(indexed, graph);
                    let (tag, new) = (format!("tag {k}"), format!("tag {k} renamed"));
                    apply_graph_delta(graph, GraphDelta::RemoveNodeTag { key, tag });
                    apply_graph_delta(graph, GraphDelta::InsertNodeTag { key, tag: new });
                },
                _ => {
                    let new_url = format!("https://moved.test/{k}");
                    apply_graph_delta(graph, GraphDelta::SetNodeUrl { key, new_url });
                },
            }
            let t = Instant::now();
            refresh(indexed, graph);
            samples.push(t.elapsed());
        }
        median_row(n, &format!("refresh_incremental_after_{kind}"), samples);
    }
    indexed
        .verify_against_fresh(graph)
        .expect("the refreshed dataset equals a fresh build");
    graph.set_recorder(None);
    println!("{n}\trefresh_bytes_after_edits\t{}\t\t\t", indexed.refresh_bytes());

    // A frame's engine share, headless. Steady: is_current (true) plus one
    // held query. Edit: is_current (false), rebuild, one held query, after one
    // assert. Two queries: the point lookup and the two-hop join.
    for (name, query, expect) in [("point", POINT, 1usize), ("two_hop", TWO_HOP, 1)] {
        let mut steady = vec![];
        for _ in 0..edits {
            let t = Instant::now();
            assert!(indexed.is_current(graph));
            let rows = query_rows(indexed, graph, query);
            steady.push(t.elapsed());
            assert_eq!(rows, expect, "held {name} row count");
        }
        median_row(n, &format!("frame_steady_{name}"), steady);
        let mut edited = vec![];
        for k in 0..rebuilds {
            edit_assert(graph, keys, &format!("frame{name}"));
            let t = Instant::now();
            if !indexed.is_current(graph) {
                *indexed = IndexedDataset::for_graph(graph);
            }
            let rows = query_rows(indexed, graph, query);
            edited.push(t.elapsed());
            assert_eq!(rows, expect, "held {name} row count after edit");
        }
        median_row(n, &format!("frame_after_edit_{name}"), edited);
    }
}
