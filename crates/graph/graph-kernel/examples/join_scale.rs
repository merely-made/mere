// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The surface/resource join at scale (engine split P5 L3): each surface shows
//! its own resource, and the resources form a `cites` ring. Times the join
//! reads (`shown_resource_id` then the resource, and `surface_ids_showing_resource`),
//! the resource edge walk, the composed RDF projection, the held index build,
//! and one edit followed by `is_current` and a rebuild. Medians of seven runs.
//!
//! `cargo run --release -p mere-kernel --features query --example join_scale -- 10000 100000`

use kernel::graph::apply::add_node;
use kernel::graph::{Graph, NodeKey, SemanticStatementSpec, SemanticSubKind, predicate_iri};
use mere::query::IndexedDataset;
use std::time::Instant;
use uuid::Uuid;

const RUNS: usize = 7;

fn median(mut samples: Vec<f64>) -> f64 {
    samples.sort_by(f64::total_cmp);
    samples[samples.len() / 2]
}

fn time<T>(mut run: impl FnMut() -> T) -> (f64, T) {
    let mut samples = Vec::with_capacity(RUNS);
    let mut last = None;
    for _ in 0..RUNS {
        let t = Instant::now();
        let out = std::hint::black_box(run());
        samples.push(t.elapsed().as_secs_f64() * 1000.0);
        last = Some(out);
    }
    (median(samples), last.expect("at least one run"))
}

fn cites(at: u64) -> SemanticStatementSpec {
    SemanticStatementSpec {
        predicate: predicate_iri(SemanticSubKind::Cites).into(),
        provenance_iri: Some("urn:scale:asserter".into()),
        asserted_at_ms: Some(at),
        ..Default::default()
    }
}

fn build(n: usize) -> (Graph, Vec<NodeKey>, Vec<Uuid>) {
    let mut graph = Graph::new();
    let t = Instant::now();
    let keys: Vec<NodeKey> = (0..n)
        .map(|i| {
            add_node(
                &mut graph,
                None,
                format!("https://n.test/{i}"),
                Default::default(),
            )
        })
        .collect();
    let surfaces_ms = t.elapsed().as_secs_f64() * 1000.0;
    let resources: Vec<Uuid> = keys
        .iter()
        .map(|key| graph.shown_resource_id(*key).expect("bound resource"))
        .collect();
    let t = Instant::now();
    for (i, from) in resources.iter().enumerate() {
        graph
            .try_assert_semantic_statement_by_resource_ids(*from, resources[(i + 1) % n], cites(1))
            .expect("resource statement");
    }
    let statements_ms = t.elapsed().as_secs_f64() * 1000.0;
    println!(
        "{n}\tbuild\tadd_node+bind {surfaces_ms:.1} ms\tresource asserts {statements_ms:.1} ms"
    );
    (graph, keys, resources)
}

fn main() {
    let scales: Vec<usize> = std::env::args()
        .skip(1)
        .filter_map(|a| a.parse().ok())
        .collect();
    let scales = if scales.is_empty() {
        vec![10_000, 100_000]
    } else {
        scales
    };
    println!("scale\tphase\tmedian_ms\tcount");
    for n in scales {
        let (mut graph, keys, resources) = build(n);
        let (ms, count) = time(|| {
            keys.iter()
                .filter_map(|key| graph.shown_resource_id(*key))
                .filter_map(|id| graph.resource(id))
                .map(|resource| resource.canonical_iri().len())
                .count()
        });
        println!("{n}\tshown_resource_id+resource\t{ms:.3}\t{count}");
        let (ms, count) = time(|| {
            resources
                .iter()
                .map(|id| graph.surface_ids_showing_resource(*id).len())
                .sum::<usize>()
        });
        println!("{n}\tsurface_ids_showing_resource\t{ms:.3}\t{count}");
        let (ms, count) = time(|| {
            graph
                .resource_edges()
                .map(|(_, _, payload)| payload.held_assertions().count())
                .sum::<usize>()
        });
        println!("{n}\tresource_edges held\t{ms:.3}\t{count}");
        let (ms, count) = time(|| mere::dataset_quads(&graph).len());
        println!("{n}\tdataset_quads\t{ms:.3}\t{count}");
        let (ms, dataset) = time(|| IndexedDataset::for_graph(&graph));
        println!("{n}\tfor_graph\t{ms:.3}\t{}", dataset.len());
        let mut samples = Vec::with_capacity(RUNS);
        for run in 0..RUNS {
            let held = IndexedDataset::for_graph(&graph);
            let t = Instant::now();
            graph
                .try_assert_semantic_statement_by_resource_ids(
                    resources[run],
                    resources[(run + 2) % n],
                    cites(2 + run as u64),
                )
                .expect("edit");
            assert!(!held.is_current(&graph));
            let rebuilt = IndexedDataset::for_graph(&graph);
            assert!(rebuilt.is_current(&graph));
            samples.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        println!(
            "{n}\tedit+is_current+rebuild\t{:.3}\t{RUNS}",
            median(samples)
        );
    }
}
