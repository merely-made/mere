// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! E3 probe (recursive-query experiments plan, 2026-09-11): an `ascent` Datalog
//! prototype of graphlet derivation, benchmarked against the kernel's hand-rolled
//! selector-filtered BFS (`derive_members` → `component_members` / `ego_members`).
//!
//! This is a throwaway measurement, not a feature. Delete this file and the
//! `ascent` dev-dependency in `Cargo.toml` together. Run with:
//!
//! ```text
//! cargo test -p graphlets --release -- --ignored ascent_vs_handrolled_bench --nocapture
//! ```

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use ascent::{Dual, ascent};
use euclid::default::Point2D;
use forme::{GraphletKind, GraphletSpec};
use graphlets::derive_members;
use kernel::graph::apply::{GraphDelta, apply_graph_delta};
use kernel::graph::fixtures::GraphFixtures;
use kernel::graph::{
    ContainmentSubKind, EdgeAssertion, EdgeFamily, Graph, NavigationTrigger, NodeKey,
    SemanticSubKind,
};
use uuid::Uuid;

/// Node column: the petgraph index of a kernel node, so the EDB is index-typed
/// (no uuid hashing inside the join loops).
type N = u32;

// ── The rules ────────────────────────────────────────────────────────────────
//
// `edge(a, b, f)` is the EDB: one row per `relations()` row, keyed by family.
// The kernel walk is undirected, so each rule appears twice (a→b and b→a) over
// the *same* edge relation rather than doubling the EDB; ascent still builds a
// second index over `edge` for the reversed join pattern, so memory doubles
// internally either way.
//
// `selected(f)` is the edge projection. The kernel treats an empty selector list
// as "all families"; Datalog has no such default, so the loader materialises all
// six families into `selected` in that case.

ascent! {
    /// Component: unbounded undirected reachability from the seed.
    struct ComponentQ;
    relation edge(N, N, EdgeFamily);
    relation selected(EdgeFamily);
    relation seed(N);
    relation reach(N);

    reach(s) <-- seed(s);
    reach(b) <-- reach(a), edge(a, b, f), selected(f);
    reach(a) <-- reach(b), edge(a, b, f), selected(f);
}

ascent! {
    /// Ego: bounded-radius reachability. `dist` is a min-lattice over hop count
    /// (`Dual<u8>` so the join keeps the smaller distance); the radius bound is a
    /// guard on the current distance, so the fixpoint stops growing at `radius`.
    /// Without the lattice a plain `reach(n, d)` relation would hold every
    /// (node, depth) pair up to the radius — correct but multiplies facts by up
    /// to `radius`.
    struct EgoQ;
    relation edge(N, N, EdgeFamily);
    relation selected(EdgeFamily);
    relation seed(N);
    relation radius(u8);
    lattice dist(N, Dual<u8>);

    dist(s, Dual(0)) <-- seed(s);
    dist(b, Dual(d + 1)) <-- dist(a, ?Dual(d)), radius(r), if d < r, edge(a, b, f), selected(f);
    dist(a, Dual(d + 1)) <-- dist(b, ?Dual(d)), radius(r), if d < r, edge(a, b, f), selected(f);
}

// ── Synthetic graph ──────────────────────────────────────────────────────────

/// Tiny deterministic LCG (Numerical Recipes constants); no new dependency.
struct Lcg(u64);

impl Lcg {
    fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }
    fn below(&mut self, n: u32) -> u32 {
        self.next_u32() % n
    }
    fn unit(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }
}

struct Synth {
    graph: Graph,
    seed_id: Uuid,
    nodes: usize,
    clusters: usize,
    build: Duration,
}

/// `target_relations` assertions spread over `clusters` weakly connected
/// clusters (no inter-cluster edges), ~0.3 nodes per relation, families 40%
/// Semantic / 30% Containment / 30% Traversal. Antiparallel pairs are avoided:
/// the kernel's `edge_matches_selectors` only inspects the first direction it
/// finds, so a→b Semantic plus b→a Containment would make the hand-rolled walk
/// and the rules disagree for reasons unrelated to the engine.
fn build_graph(target_relations: usize, clusters: usize, rng_seed: u64) -> Synth {
    let start = Instant::now();
    let mut rng = Lcg(rng_seed);
    let nodes = (target_relations * 3 / 10).max(clusters * 4);
    let mut graph = Graph::new();
    let keys: Vec<NodeKey> = (0..nodes)
        .map(|i| graph.add_node(format!("https://synth/{i}"), Point2D::new(i as f32, 0.0)))
        .collect();
    let per_cluster = nodes / clusters;
    let mut seen_pairs: HashSet<(usize, usize)> = HashSet::with_capacity(target_relations);
    let mut made = 0usize;
    let mut attempts = 0usize;
    while made < target_relations && attempts < target_relations * 20 {
        attempts += 1;
        let c = made % clusters;
        let lo = c * per_cluster;
        let hi = if c + 1 == clusters {
            nodes
        } else {
            lo + per_cluster
        };
        let span = (hi - lo) as u32;
        let a = lo + rng.below(span) as usize;
        let b = lo + rng.below(span) as usize;
        if a == b || !seen_pairs.insert((a.min(b), a.max(b))) {
            continue;
        }
        let roll = rng.unit();
        let ok = if roll < 0.4 {
            graph
                .assert_relation(
                    keys[a],
                    keys[b],
                    EdgeAssertion::Semantic {
                        sub_kind: SemanticSubKind::Hyperlink,
                        label: None,
                        decay_progress: None,
                    },
                )
                .is_some()
        } else if roll < 0.7 {
            graph
                .assert_relation(
                    keys[a],
                    keys[b],
                    EdgeAssertion::Containment {
                        sub_kind: ContainmentSubKind::Domain,
                    },
                )
                .is_some()
        } else {
            matches!(
                apply_graph_delta(
                    &mut graph,
                    GraphDelta::AppendTraversal {
                        from: keys[a],
                        to: keys[b],
                        trigger: NavigationTrigger::LinkClick,
                        timestamp_ms: Some(made as u64),
                    },
                ),
                kernel::graph::apply::GraphDeltaResult::TraversalAppended(true)
            )
        };
        if ok {
            made += 1;
        }
    }
    assert_eq!(made, target_relations, "could not place every relation");
    // Seed on the cluster-0 node with the most Containment relations so the
    // `[Containment]`-projected Component is a real fraction of its cluster
    // rather than the seed alone.
    let mut containment_degree = vec![0usize; per_cluster];
    for r in graph.relations() {
        if r.kind.family() == EdgeFamily::Containment {
            for k in [r.from, r.to] {
                if let Some(slot) = containment_degree.get_mut(k.index()) {
                    *slot += 1;
                }
            }
        }
    }
    let seed_pos = (0..per_cluster)
        .max_by_key(|&i| containment_degree[i])
        .expect("cluster 0 is non-empty");
    let seed_id = graph.get_node(keys[seed_pos]).expect("seed node").id;
    Synth {
        graph,
        seed_id,
        nodes,
        clusters,
        build: start.elapsed(),
    }
}

// ── EDB extraction ───────────────────────────────────────────────────────────

struct Edb {
    edges: Vec<(N, N, EdgeFamily)>,
    /// petgraph index → node uuid, for turning the IDB back into `GraphMemberId`s.
    ids: HashMap<N, Uuid>,
}

/// One pass over `relations()` plus the index→uuid map. This is the load term
/// the fair comparison charges to the rules engine.
fn extract_edb(graph: &Graph) -> Edb {
    let edges = graph
        .relations()
        .map(|r| (r.from.index() as N, r.to.index() as N, r.kind.family()))
        .collect();
    let ids = graph.nodes().map(|(k, n)| (k.index() as N, n.id)).collect();
    Edb { edges, ids }
}

// ── Queries ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum Shape {
    Component,
    Ego(u8),
}

impl Shape {
    fn label(self) -> String {
        match self {
            Shape::Component => "Component".to_string(),
            Shape::Ego(r) => format!("Ego r={r}"),
        }
    }
    fn kind(self) -> GraphletKind {
        match self {
            Shape::Component => GraphletKind::Component,
            Shape::Ego(radius) => GraphletKind::Ego { radius },
        }
    }
}

fn spec(shape: Shape, seed: Uuid, selectors: &[EdgeFamily]) -> GraphletSpec {
    GraphletSpec {
        kind: shape.kind(),
        anchors: vec![seed.to_string()],
        primary_anchor: Some(seed.to_string()),
        selectors: selectors
            .iter()
            .map(|f| format!("{f:?}").to_ascii_lowercase())
            .collect(),
    }
}

const ALL_FAMILIES: [EdgeFamily; 6] = [
    EdgeFamily::Semantic,
    EdgeFamily::Traversal,
    EdgeFamily::Containment,
    EdgeFamily::Arrangement,
    EdgeFamily::Imported,
    EdgeFamily::Provenance,
];

fn selected_rows(selectors: &[EdgeFamily]) -> Vec<(EdgeFamily,)> {
    let fams: &[EdgeFamily] = if selectors.is_empty() {
        &ALL_FAMILIES
    } else {
        selectors
    };
    fams.iter().map(|&f| (f,)).collect()
}

struct AscentRun {
    /// Clone the EDB into a fresh program and run it once with no seed: that
    /// first `run()` moves the rows into ascent's hash indices. A rules layer
    /// pays this on every derivation unless it keeps the indexed program alive.
    load: Duration,
    /// Add the seed to the already-indexed program and run to fixpoint: the
    /// cost a persistent, pre-indexed rules layer would pay per query.
    query: Duration,
    members: HashSet<Uuid>,
}

/// Load, then query, timing the two separately. ascent programs are re-runnable,
/// but not incrementally: every `run()` begins by moving each relation's `total`
/// index back into `delta` (ascent_macro `ascent_codegen.rs`, "move total
/// versions of dynamic indices to delta"), so the second run re-derives over the
/// whole EDB rather than only from the new seed fact. The `query` number below
/// is therefore O(|EDB|) even for a tiny Ego neighbourhood.
fn run_ascent(edb: &Edb, seed: N, shape: Shape, selectors: &[EdgeFamily]) -> AscentRun {
    let (load, query, members): (Duration, Duration, Vec<N>) = match shape {
        Shape::Component => {
            let t = Instant::now();
            let mut p = ComponentQ {
                edge: edb.edges.clone(),
                selected: selected_rows(selectors),
                ..Default::default()
            };
            p.run();
            let load = t.elapsed();
            let t = Instant::now();
            p.seed = vec![(seed,)];
            p.run();
            let query = t.elapsed();
            (load, query, p.reach.into_iter().map(|(n,)| n).collect())
        },
        Shape::Ego(radius) => {
            let t = Instant::now();
            let mut p = EgoQ {
                edge: edb.edges.clone(),
                selected: selected_rows(selectors),
                radius: vec![(radius,)],
                ..Default::default()
            };
            p.run();
            let load = t.elapsed();
            let t = Instant::now();
            p.seed = vec![(seed,)];
            p.run();
            let query = t.elapsed();
            (load, query, p.dist.into_iter().map(|(n, _)| n).collect())
        },
    };
    AscentRun {
        load,
        query,
        members: members.into_iter().map(|n| edb.ids[&n]).collect(),
    }
}

fn median(mut xs: Vec<Duration>) -> Duration {
    xs.sort();
    xs[xs.len() / 2]
}

fn ms(d: Duration) -> String {
    format!("{:.3}", d.as_secs_f64() * 1e3)
}

// ── The bench ────────────────────────────────────────────────────────────────

#[test]
#[ignore = "E3 probe: wall-clock bench, run explicitly with --ignored --nocapture in release"]
fn ascent_vs_handrolled_bench() {
    const RUNS: usize = 5;
    let sizes: [(usize, usize); 3] = [(1_000, 4), (10_000, 6), (100_000, 8)];
    let shapes = [Shape::Component, Shape::Ego(2), Shape::Ego(4)];
    let projections: [(&str, &[EdgeFamily]); 2] = [
        ("[Containment]", &[EdgeFamily::Containment]),
        ("[] (all)", &[]),
    ];

    println!();
    println!(
        "E3 ascent-vs-handrolled (median of {RUNS}, {} build)",
        if cfg!(debug_assertions) {
            "DEBUG"
        } else {
            "release"
        }
    );
    for (target, clusters) in sizes {
        let synth = build_graph(target, clusters, 0x5EED_u64 + target as u64);
        let graph = &synth.graph;

        let extract_times: Vec<Duration> = (0..RUNS)
            .map(|_| {
                let t = Instant::now();
                let e = extract_edb(graph);
                std::hint::black_box(&e);
                t.elapsed()
            })
            .collect();
        let extract = median(extract_times);
        let edb = extract_edb(graph);
        let seed_idx = graph
            .get_node_key_by_id(synth.seed_id)
            .expect("seed key")
            .index() as N;

        println!();
        println!(
            "graph: {} relation rows, {} nodes, {} clusters, built in {} ms; EDB extraction {} ms ({} edge rows)",
            edb.edges.len(),
            synth.nodes,
            synth.clusters,
            ms(synth.build),
            ms(extract),
            edb.edges.len()
        );
        println!(
            "{:<12} {:<14} {:>8} {:>12} {:>12} {:>12} {:>10} {:>10} {:>9}",
            "query",
            "selectors",
            "members",
            "handrolled",
            "asc load",
            "asc query",
            "cold ratio",
            "warm ratio",
            "extract%"
        );
        for shape in shapes {
            for (label, selectors) in projections {
                let s = spec(shape, synth.seed_id, selectors);
                let mut hand_times = Vec::with_capacity(RUNS);
                let mut hand_set: HashSet<Uuid> = HashSet::new();
                for _ in 0..RUNS {
                    let t = Instant::now();
                    let m = derive_members(graph, &s);
                    hand_times.push(t.elapsed());
                    hand_set = m.into_iter().collect();
                }
                let mut load_times = Vec::with_capacity(RUNS);
                let mut query_times = Vec::with_capacity(RUNS);
                let mut asc_set: HashSet<Uuid> = HashSet::new();
                for _ in 0..RUNS {
                    let r = run_ascent(&edb, seed_idx, shape, selectors);
                    load_times.push(r.load);
                    query_times.push(r.query);
                    asc_set = r.members;
                }
                assert_eq!(
                    hand_set,
                    asc_set,
                    "member sets differ: {} {} at {} relations",
                    shape.label(),
                    label,
                    target
                );
                let hand = median(hand_times);
                let load = median(load_times);
                let query = median(query_times);
                let hand_s = hand.as_secs_f64().max(1e-9);
                let cold_ratio = (extract + load + query).as_secs_f64() / hand_s;
                let warm_ratio = query.as_secs_f64() / hand_s;
                // Extraction as a share of the cold ascent total (extract + load + query).
                let extract_pct =
                    100.0 * extract.as_secs_f64() / (extract + load + query).as_secs_f64();
                println!(
                    "{:<12} {:<14} {:>8} {:>9} ms {:>9} ms {:>9} ms {:>9.1}x {:>9.1}x {:>8.0}%",
                    shape.label(),
                    label,
                    hand_set.len(),
                    ms(hand),
                    ms(load),
                    ms(query),
                    cold_ratio,
                    warm_ratio,
                    extract_pct
                );
            }
        }
    }
}
