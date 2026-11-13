// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! `IndexedDataset::refresh` from captured edits against a fresh `for_graph`.

use std::sync::{Arc, Mutex};

use crate::graph::apply::{
    GraphDelta, add_node, apply_graph_delta, assert_semantic_predicate_in_scope,
};
use crate::graph::capture::CapturedDelta;
use crate::graph::{Graph, NodeKey, SemanticStatementSpec};
use crate::rdf::touched_sources;
use crate::types::{GraphScope, NodeProperty};
use mere::dataset::Refresh;
use mere::model::{ContainmentSubKind, EdgeAssertion};
use mere::query::{IndexedDataset, View, sparql_on};

struct Rng(u64);

impl Rng {
    fn next(&mut self, below: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 33) as usize % below
    }
}

type Sink = Arc<Mutex<Vec<CapturedDelta>>>;

fn recording(graph: &mut Graph) -> Sink {
    let sink: Sink = Arc::default();
    let into = sink.clone();
    graph.set_recorder(Some(Arc::new(move |delta: &CapturedDelta| {
        into.lock().unwrap().push(delta.clone());
    })));
    sink
}

const QUERIES: &[&str] = &[
    "SELECT ?s ?p ?o WHERE { ?s ?p ?o }",
    "SELECT ?g ?s ?p ?o WHERE { GRAPH ?g { ?s ?p ?o } }",
    "SELECT ?a ?b ?c WHERE { ?a ?p ?b . ?b ?q ?c }",
    "SELECT ?r ?t WHERE { ?r <http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies> ?t }",
];

fn rows(dataset: &IndexedDataset, query: &str, view: View) -> Vec<Vec<Option<String>>> {
    let mut rows = sparql_on(dataset, query, view, Default::default())
        .unwrap()
        .rows;
    rows.sort();
    rows
}

/// A containment relation, which stays in the surface graph and projects
/// from `from`'s source with `to`'s IRI (semantic relations go to resources).
fn surface_link(graph: &mut Graph, from: NodeKey, to: NodeKey) {
    let assertion = EdgeAssertion::Containment {
        sub_kind: ContainmentSubKind::UserFolder,
    };
    let asserter_iri = "urn:test:asserter".to_string();
    apply_graph_delta(graph, GraphDelta::AssertRelation { from, to, assertion, asserter_iri });
}

/// One random edit; an asserted statement's id joins `held`.
fn edit(graph: &mut Graph, rng: &mut Rng, keys: &mut Vec<NodeKey>, held: &mut Vec<String>) {
    let pick = |rng: &mut Rng, keys: &[NodeKey]| keys[rng.next(keys.len())];
    match rng.next(22) {
        // Replace the resource a surface shows with another, or clear it.
        20 | 21 => {
            let surface_id = graph.get_node(pick(rng, keys)).unwrap().id;
            let resources: Vec<_> = graph.resources.resource_nodes().map(|r| r.id()).collect();
            let resource_id = (rng.next(2) == 0 && !resources.is_empty())
                .then(|| resources[rng.next(resources.len())]);
            apply_graph_delta(graph, GraphDelta::ReplaySetShownResourceById { surface_id, resource_id });
        },
        0 | 1 => {
            let n = keys.len();
            keys.push(add_node(graph, None, format!("https://n.test/{n}"), Default::default()));
        },
        2..=6 => {
            let (from, to) = (pick(rng, keys), pick(rng, keys));
            let scope = [GraphScope::Default, GraphScope::User, GraphScope::Custom("urn:g:one".into())]
                [rng.next(3)]
            .clone();
            let spec = SemanticStatementSpec {
                predicate: format!("https://v.test/p{}", rng.next(3)),
                graph_scope: scope,
                ..Default::default()
            };
            if let Some((_, outcome)) = graph.assert_semantic_statement(from, to, spec)
                && outcome.changed
            {
                held.push(outcome.statement_id);
            }
        },
        7..=9 if !held.is_empty() => {
            let id = held.swap_remove(rng.next(held.len()));
            graph.withdraw_semantic_assertion(&id, Some(1_000), None);
        },
        10 => {
            let (key, title) = (pick(rng, keys), format!("title {}", rng.next(4)));
            apply_graph_delta(graph, GraphDelta::SetNodeTitle { key, title });
        },
        11 => {
            let (key, tag) = (pick(rng, keys), format!("tag{}", rng.next(3)));
            apply_graph_delta(graph, GraphDelta::InsertNodeTag { key, tag });
        },
        12 => {
            let (key, tag) = (pick(rng, keys), format!("tag{}", rng.next(3)));
            apply_graph_delta(graph, GraphDelta::RemoveNodeTag { key, tag });
        },
        13 | 14 => {
            let key = pick(rng, keys);
            let property = NodeProperty::new("https://schema.org/position".into(), rng.next(5).to_string());
            apply_graph_delta(graph, GraphDelta::AppendNodeProperty { key, property });
        },
        15 => {
            let key = pick(rng, keys);
            let new_url = format!("https://moved.test/{}", rng.next(1_000));
            apply_graph_delta(graph, GraphDelta::SetNodeUrl { key, new_url });
        },
        17 => {
            let (from, to) = (pick(rng, keys), pick(rng, keys));
            let predicate = format!("https://v.test/s{}", rng.next(3));
            assert_semantic_predicate_in_scope(graph, from, to, predicate, GraphScope::User);
        },
        18 | 19 => surface_link(graph, pick(rng, keys), pick(rng, keys)),
        16 if keys.len() > 3 => {
            let key = keys.swap_remove(rng.next(keys.len()));
            apply_graph_delta(graph, GraphDelta::RemoveNode { key });
        },
        _ => {},
    }
}

/// Random edit sequences, each batch refreshed from its captured edits, equal
/// a fresh `for_graph` in both views and answer the same queries.
#[test]
fn refreshed_datasets_match_a_fresh_build() {
    let (mut incremental, mut rebuilt, mut fallbacks) = (0, 0, 0);
    let (mut shown, mut cleared) = (0, 0);
    for seed in 1..=12_u64 {
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ seed.wrapping_mul(0xD1B5_4A32));
        let mut graph = Graph::new();
        let mut keys: Vec<NodeKey> = (0..6)
            .map(|i| add_node(&mut graph, None, format!("https://n.test/{i}"), Default::default()))
            .collect();
        let mut held = Vec::new();
        for _ in 0..8 {
            edit(&mut graph, &mut rng, &mut keys, &mut held);
        }
        let sink = recording(&mut graph);
        let mut dataset = IndexedDataset::for_graph(&graph);
        for step in 0..60 {
            for _ in 0..1 + rng.next(3) {
                edit(&mut graph, &mut rng, &mut keys, &mut held);
            }
            let deltas = std::mem::take(&mut *sink.lock().unwrap());
            for delta in &deltas {
                if let CapturedDelta::ReplaySetShownResourceById { resource_id, .. } = delta {
                    if resource_id.is_some() { shown += 1 } else { cleared += 1 }
                }
            }
            match touched_sources(&deltas) {
                Some(touched) => match dataset.refresh(&graph, &touched) {
                    Refresh::Incremental => incremental += 1,
                    Refresh::Rebuilt => rebuilt += 1,
                    Refresh::Current => {},
                },
                None => {
                    dataset = IndexedDataset::for_graph(&graph);
                    fallbacks += 1;
                },
            }
            assert!(dataset.is_current(&graph), "seed {seed} step {step}");
            if let Err(difference) = dataset.verify_against_fresh(&graph) {
                panic!("seed {seed} step {step}: {difference}; edits {deltas:?}");
            }
            if step % 15 == 14 {
                let fresh = IndexedDataset::for_graph(&graph);
                for query in QUERIES {
                    for view in [View::Live, View::History] {
                        assert_eq!(rows(&dataset, query, view), rows(&fresh, query, view), "{query}");
                    }
                }
            }
        }
    }
    println!("refresh: {incremental} incremental, {rebuilt} rebuilt in refresh, {fallbacks} touched_sources fallbacks; shown resource set {shown}, cleared {cleared}");
    assert!(shown > 0 && cleared > 0, "showing was both replaced and cleared: {shown}, {cleared}");
    assert!(incremental > 300, "most batches refresh in place: {incremental}");
    assert!(fallbacks > 0, "node removals fell back: {fallbacks}");
}

/// A node's new URL renames it in its in-neighbours' containment edges and
/// their reifiers; the refresh re-projects them instead of rebuilding.
#[test]
fn an_iri_change_reprojects_the_in_neighbours() {
    let mut graph = Graph::new();
    let [a, b, c] = ["a", "b", "c"]
        .map(|name| add_node(&mut graph, None, format!("https://n.test/{name}"), Default::default()));
    let spec = |predicate: &str| SemanticStatementSpec {
        predicate: predicate.into(),
        graph_scope: GraphScope::User,
        ..Default::default()
    };
    // A Surface edge, whose quads live in `a`'s source and name `c`'s IRI.
    surface_link(&mut graph, a, c);
    graph.assert_semantic_statement(a, c, spec("https://v.test/held")).unwrap();
    let (_, gone) = graph.assert_semantic_statement(b, c, spec("https://v.test/gone")).unwrap();
    assert!(graph.withdraw_semantic_assertion(&gone.statement_id, Some(1_000), None));
    let sink = recording(&mut graph);
    let mut dataset = IndexedDataset::for_graph(&graph);
    let new_url = "https://moved.test/c".to_string();
    apply_graph_delta(&mut graph, GraphDelta::SetNodeUrl { key: c, new_url });
    let deltas = std::mem::take(&mut *sink.lock().unwrap());
    let touched = touched_sources(&deltas).expect("a URL edit refreshes in place");
    assert_eq!(dataset.refresh(&graph, &touched), Refresh::Incremental);
    dataset.verify_against_fresh(&graph).unwrap();
}

/// The source split is the whole projection: indexing by source equals
/// indexing the held and withdrawn walks.
#[test]
fn indexing_by_source_equals_the_whole_walk() {
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    let mut graph = Graph::new();
    let mut keys: Vec<NodeKey> = (0..8)
        .map(|i| add_node(&mut graph, None, format!("https://n.test/{i}"), Default::default()))
        .collect();
    let mut held = Vec::new();
    for _ in 0..200 {
        edit(&mut graph, &mut rng, &mut keys, &mut held);
    }
    let by_source = IndexedDataset::for_graph(&graph);
    let walk = mere::dataset_quads(&graph).into_iter().map(|quad| (quad, true));
    let withdrawn = mere::RdfSource::withdrawn_reifier_quads(&graph)
        .into_iter()
        .map(|quad| (quad, false));
    let whole = IndexedDataset::from_quads(walk.chain(withdrawn));
    for query in QUERIES {
        for view in [View::Live, View::History] {
            assert_eq!(rows(&by_source, query, view), rows(&whole, query, view), "{query}");
        }
    }
    assert_eq!(by_source.len(), whole.len());
}

/// A refresh not told of a node edit leaves the dataset wrong, which the
/// check catches: the instrument can see a missed source.
#[test]
fn a_missed_source_is_caught_by_the_check() {
    let mut graph = Graph::new();
    let key = add_node(&mut graph, None, "https://n.test/a".into(), Default::default());
    let mut dataset = IndexedDataset::for_graph(&graph);
    apply_graph_delta(&mut graph, GraphDelta::SetNodeTitle { key, title: "A".into() });
    assert_eq!(dataset.refresh(&graph, &[]), Refresh::Incremental);
    assert!(dataset.verify_against_fresh(&graph).is_err());
}

/// Two nodes at one URL with one title emit the same row; retitling one must
/// keep the row the other still emits.
#[test]
fn a_row_two_sources_emit_stays_until_both_drop_it() {
    let mut graph = Graph::new();
    let url = "https://n.test/same".to_string();
    let a = add_node(&mut graph, None, url.clone(), Default::default());
    let b = add_node(&mut graph, None, url, Default::default());
    assert_ne!(a, b, "two nodes at one URL");
    for key in [a, b] {
        apply_graph_delta(&mut graph, GraphDelta::SetNodeTitle { key, title: "Same".into() });
    }
    let sink = recording(&mut graph);
    let mut dataset = IndexedDataset::for_graph(&graph);
    for (key, title) in [(a, "A"), (b, "B"), (a, "Same")] {
        apply_graph_delta(&mut graph, GraphDelta::SetNodeTitle { key, title: title.into() });
        let deltas = std::mem::take(&mut *sink.lock().unwrap());
        let touched = touched_sources(&deltas).expect("a title edit refreshes in place");
        assert_eq!(dataset.refresh(&graph, &touched), Refresh::Incremental);
        dataset.verify_against_fresh(&graph).unwrap();
    }
}
