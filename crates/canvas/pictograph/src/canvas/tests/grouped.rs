// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The grouped layout through the canvas (dynamics grammar plan, G3):
//! Charge's repulsion between groups at weight 16, Springs within (F71), on
//! a graph whose topics no edge structure discloses, read against Springs
//! alone and shuffled topics.
//!
//! The fixture crosses three partitions, as G2's quick topic fixture does
//! (`grammar-g2`, `tests/meaning_topics.rs`, `topic_graph`): node `i` has
//! topic `i / 8`, site `i % 4`, and sits in structural community
//! `(i / 2) % 4` (a ring of eight with four diameters per community, four
//! bridges between). Main has no Meaning channel until G2 merges, so the
//! topics are handed in as the partition (F70); when G2 merges, rows on
//! `groups.meaning` join and G2's `topic_graph` replaces this copy. Every run
//! starts from one seeded scatter, placed as a seeded arrangement, and is
//! read when the bodies rest (F46) or after 6 000 frames. The within-group
//! stress is reported and not asserted: its bar waits on a fixture whose
//! topics have more structure inside them (F72).

use super::*;
use crate::canvas::composition::{PhysicsComposition, PhysicsGrouping};
use seiche::observe::{Separation, group_stress, group_stress_each, separation};

const N: usize = 32;

fn topic(i: usize) -> u32 {
    (i / 8) as u32
}

fn community(i: usize) -> usize {
    (i / 2) % 4
}

/// The fixture's edges, as index pairs.
fn crossed_pairs() -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    for c in 0..4 {
        let members: Vec<usize> = (0..N).filter(|&i| community(i) == c).collect();
        for k in 0..members.len() {
            pairs.push((members[k], members[(k + 1) % members.len()]));
        }
        for (a, b) in [(0, 4), (1, 5), (2, 6), (3, 7)] {
            pairs.push((members[a], members[b]));
        }
    }
    pairs.extend([(0, 2), (3, 5), (6, 12), (15, 1)]);
    pairs
}

/// A seeded scatter in a disc of 300, one point per node index.
fn scatter() -> Vec<PortablePoint> {
    let mut state: u64 = 0x6e0u64;
    let mut unit = || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 40) as f32 / (1u64 << 24) as f32
    };
    (0..N)
        .map(|_| {
            let (r, a) = (300.0 * unit().sqrt(), std::f32::consts::TAU * unit());
            PortablePoint::new(r * a.cos(), r * a.sin())
        })
        .collect()
}

/// The graph over `members` (node indices) and the edges among them.
fn graph_of(members: &[usize]) -> (Graph, Vec<NodeKey>) {
    let sites = ["news", "wiki", "blog", "forum"];
    let mut graph = Graph::new();
    let keys: Vec<NodeKey> = members
        .iter()
        .map(|&i| {
            graph.add_node(
                format!("https://{}.example/{i}", sites[i % 4]),
                PortablePoint::new(0.0, 0.0),
            )
        })
        .collect();
    let at: HashMap<usize, NodeKey> = members.iter().copied().zip(keys.iter().copied()).collect();
    for (a, b) in crossed_pairs() {
        if let (Some(&ka), Some(&kb)) = (at.get(&a), at.get(&b)) {
            graph.assert_relation(ka, kb, hyperlink());
        }
    }
    (graph, keys)
}

/// `labels` dealt out again by a fixed shuffle, each as often as before.
fn shuffled(keys: &[NodeKey], labels: &HashMap<NodeKey, u32>) -> HashMap<NodeKey, u32> {
    let mut values: Vec<u32> = keys.iter().map(|k| labels[k]).collect();
    let mut state: u64 = 0x5EED;
    for i in (1..values.len()).rev() {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        values.swap(i, (state >> 33) as usize % (i + 1));
    }
    keys.iter().copied().zip(values).collect()
}

/// What a run leaves: positions, and how many frames it took to rest.
struct Rest {
    positions: Vec<(NodeKey, (f64, f64))>,
    frames: usize,
}

/// A group per node.
type Groups = HashMap<NodeKey, u32>;

/// How the law slot is filled for a run.
enum Slot {
    /// The picked law (Springs).
    Law,
    /// A composition through the catalog.
    Composition(PhysicsComposition),
}

/// Run `slot` over the graph on `members` from the scatter until rest.
fn rest(members: &[usize], slot: Slot) -> Rest {
    let (graph, keys) = graph_of(members);
    let mut canvas = Canvas::with_graph(graph);
    canvas.resize(1400, 900);
    canvas.set_physics_paused(true);
    canvas.set_layout_strategy(Some("test.scatter".to_string()));
    let scatter = scatter();
    let seed: Vec<_> = members.iter().map(|&i| scatter[i]).collect();
    canvas.apply_strategy_positions(&keys.iter().copied().zip(seed).collect::<Vec<_>>());
    if let Slot::Composition(c) = slot {
        canvas.set_physics_composition(Some(c)).unwrap();
    }
    canvas.set_physics_paused(false);
    let mut frames = 0;
    while canvas.settle_count() == 0 && frames < 6_000 {
        canvas.step_layout();
        frames += 1;
    }
    Rest {
        positions: canvas
            .view
            .positions()
            .map(|(k, p)| (k, (f64::from(p.x), f64::from(p.y))))
            .collect(),
        frames,
    }
}

fn all() -> Vec<usize> {
    (0..N).collect()
}

fn sorted(groups: &HashMap<NodeKey, u32>) -> Vec<(NodeKey, u32)> {
    let mut list: Vec<(NodeKey, u32)> = groups.iter().map(|(k, g)| (*k, *g)).collect();
    list.sort_by_key(|(k, _)| k.index());
    list
}

/// Charge's repulsion between groups at `weight`, Springs within, through
/// the catalog: F71's reading, ruled at weight 16.
fn charge_between(groups: &HashMap<NodeKey, u32>, weight: f32) -> Slot {
    Slot::Composition(PhysicsComposition::Grouped(PhysicsGrouping {
        outer_weight: weight,
        ..PhysicsGrouping::charge_between(sorted(groups))
    }))
}

fn show(v: Option<f64>) -> String {
    v.map_or("none".into(), |v| format!("{v:.4}"))
}

fn line(name: &str, run: &Rest, against: &HashMap<NodeKey, u32>) -> Separation {
    let edges: Vec<(NodeKey, NodeKey)> = {
        let (graph, _) = graph_of(&all());
        crate::canvas::seiche_bridge::visible_relation_edges(
            &graph,
            &std::collections::HashSet::new(),
        )
    };
    let s = separation(&run.positions, against);
    println!(
        "{name}: ratio {:.3} (gap {:.0}, spread {:.0}, nearest gap {:.0}); within-group stress {} \
         at one scale, {} at a scale per group; rest after {} frames",
        s.ratio,
        s.gap,
        s.spread,
        s.nearest_gap,
        show(group_stress(&run.positions, &edges, against)),
        show(group_stress_each(&run.positions, &edges, against)),
        run.frames
    );
    s
}

/// The partitions on main, by node key of the full fixture (keys are dense
/// in index order).
fn partitions() -> (Groups, Groups, Groups, Groups) {
    let (graph, keys) = graph_of(&all());
    let topics = keys
        .iter()
        .enumerate()
        .map(|(i, &k)| (k, topic(i)))
        .collect();
    let sites = keys
        .iter()
        .enumerate()
        .map(|(i, &k)| (k, (i % 4) as u32))
        .collect();
    let clusters = crate::signals::community_louvain(&graph)
        .clusters
        .iter()
        .enumerate()
        .flat_map(|(c, cluster)| cluster.members.iter().map(move |&m| (m, c as u32)))
        .collect();
    let shuffled_topics = shuffled(&keys, &topics);
    (topics, sites, clusters, shuffled_topics)
}

/// Springs run on each topic alone: the other baseline the stress bar may
/// take (F72), a scale per topic.
fn springs_on_each_topic() -> f64 {
    let mut each = Vec::new();
    for t in 0..4u32 {
        let members: Vec<usize> = (0..N).filter(|&i| topic(i) == t).collect();
        let run = rest(&members, Slot::Law);
        let one: HashMap<NodeKey, u32> = run.positions.iter().map(|(k, _)| (*k, 0)).collect();
        let (graph, _) = graph_of(&members);
        let edges = crate::canvas::seiche_bridge::visible_relation_edges(
            &graph,
            &std::collections::HashSet::new(),
        );
        each.push(group_stress_each(&run.positions, &edges, &one).unwrap_or(0.0));
    }
    each.iter().sum::<f64>() / each.len() as f64
}

/// G3's done-condition on the grouped layout, with the topics handed in
/// (F70) and Charge's repulsion between them at weight 16 (F71): the gap
/// between topics exceeds the spread within them; Springs alone does not
/// separate them (the negative control), though it does separate Louvain's
/// clusters, the structure it is drawn from; and the same composition on
/// shuffled topics does not separate the real ones (the shuffled control).
/// The within-topic stress is printed beside Springs alone's and Springs on
/// each topic alone's, its bar open (F72).
#[test]
fn charge_between_topics_and_springs_within_separates_them() {
    let (topics, _, clusters, shuffled_topics) = partitions();
    let springs = rest(&all(), Slot::Law);
    let base = line("springs alone, against topics", &springs, &topics);
    let structural = line("springs alone, against clusters", &springs, &clusters);
    let by_topics = rest(
        &all(),
        Slot::Composition(PhysicsComposition::Grouped(
            PhysicsGrouping::charge_between(sorted(&topics)),
        )),
    );
    let grouped = line(
        "charge between topics at 16, against topics",
        &by_topics,
        &topics,
    );
    let by_shuffled = rest(&all(), charge_between(&shuffled_topics, 16.0));
    let control = line(
        "charge between shuffled topics at 16, against topics",
        &by_shuffled,
        &topics,
    );
    println!(
        "springs on each topic alone: within-topic stress {:.4} at a scale per topic",
        springs_on_each_topic()
    );
    assert!(
        grouped.ratio > 1.0,
        "grouping by topics separates them: {grouped:?}"
    );
    assert!(
        base.ratio < 1.0,
        "Springs alone does not separate topics: {base:?}"
    );
    assert!(
        structural.ratio > 1.0,
        "Springs alone separates the structure's own clusters: {structural:?}"
    );
    assert!(
        control.ratio < 1.0,
        "shuffled topics do not separate the real ones: {control:?}"
    );
}

/// The ruled reading at other weights and on main's other partitions, for
/// the record. Printed only.
#[test]
#[ignore = "a probe beside G3's grouped receipt; prints its table"]
fn probe_grouped_readings() {
    let (topics, sites, clusters, shuffled_topics) = partitions();
    for weight in [1.0, 4.0, 16.0, 64.0] {
        for (name, groups) in [("topics", &topics), ("shuffled topics", &shuffled_topics)] {
            let run = rest(&all(), charge_between(groups, weight));
            line(
                &format!("charge between {name} at {weight}, against topics"),
                &run,
                &topics,
            );
        }
    }
    for (name, groups) in [("clusters", &clusters), ("sites", &sites)] {
        let run = rest(&all(), charge_between(groups, 16.0));
        line(
            &format!("charge between {name} at 16, against topics"),
            &run,
            &topics,
        );
        line(
            &format!("charge between {name} at 16, against {name}"),
            &run,
            groups,
        );
    }
}
