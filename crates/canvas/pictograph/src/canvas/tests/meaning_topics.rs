// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A fixture graph with known topics, for the Meaning channel's purity
//! receipts (dynamics grammar plan, G2). Shared by the lib tests and the
//! device receipt (`tests/meaning_device.rs`, through `#[path]`).
//!
//! Thirty-two page titles, eight on each of four topics, written for the
//! receipt. The topics are crossed by everything else a channel could read:
//! node `i` has topic `i / 8`, site `i % 4` (two of every topic per site), and
//! sits in structural community `(i / 2) % 4` (a ring of eight with its four
//! diameters, four bridges between communities), so site, cluster and meaning are
//! three different partitions. Some words cross topics on purpose
//! ("backyard", "vegetable", "explained", "how"), so a lexical embedder is
//! not handed the answer.

use std::collections::HashMap;

use kernel::geometry::PortablePoint;
use kernel::graph::fixtures::GraphFixtures;
use kernel::graph::{Graph, NodeKey};

pub const TOPICS: [&str; 4] = ["astronomy", "cooking", "gardening", "programming"];

pub const SITES: [&str; 4] = ["news", "wiki", "blog", "forum"];

pub const TITLES: [[&str; 8]; 4] = [
    [
        "Jupiter's moons through a backyard telescope",
        "How neutron stars form after a supernova",
        "The James Webb Space Telescope's first deep field",
        "Why Mars looks red in the night sky",
        "A beginner's map of the constellations",
        "Measuring the distance to faraway galaxies",
        "Saturn's rings explained",
        "Watching a total solar eclipse safely",
    ],
    [
        "Slow-braised beef short ribs with red wine",
        "How to make fresh pasta dough by hand",
        "Crispy roast potatoes with garlic and rosemary",
        "A simple weeknight vegetable curry",
        "Baking sourdough bread from a starter",
        "Pan-seared salmon with lemon butter sauce",
        "Homemade chicken stock for soups",
        "Chocolate chip cookies that stay chewy",
    ],
    [
        "When to plant tomato seedlings outdoors",
        "Composting kitchen scraps in a backyard bin",
        "Pruning roses in late winter",
        "Keeping aphids off your vegetable beds",
        "Choosing perennials for a shady border",
        "How often to water container herbs",
        "Improving clay soil with organic matter",
        "Starting a raised vegetable garden",
    ],
    [
        "Understanding ownership and borrowing in Rust",
        "Writing async network services with Tokio",
        "Error handling with Result and the question mark operator",
        "Generic traits and lifetimes explained",
        "Profiling a slow binary with flamegraphs",
        "Cargo workspaces for multi-crate projects",
        "Safe concurrency with channels and mutexes",
        "Compiling Rust to WebAssembly for the browser",
    ],
];

/// The structural community node `i` sits in.
pub fn community(i: usize) -> usize {
    (i / 2) % 4
}

/// The fixture: the graph, its keys in index order, and each key's topic.
pub fn topic_graph() -> (Graph, Vec<NodeKey>, HashMap<NodeKey, usize>) {
    let mut graph = Graph::new();
    let mut keys = Vec::new();
    for i in 0..32 {
        let key = graph.add_node(
            format!("https://{}.example/{i}", SITES[i % 4]),
            PortablePoint::new((i % 8) as f32 * 40.0, (i / 8) as f32 * 40.0),
        );
        assert!(graph.set_node_title(key, TITLES[i / 8][i % 8].to_string()));
        keys.push(key);
    }
    for c in 0..4 {
        let members: Vec<usize> = (0..32).filter(|&i| community(i) == c).collect();
        for k in 0..members.len() {
            let (a, b) = (members[k], members[(k + 1) % members.len()]);
            graph.assert_semantic_predicate(keys[a], keys[b], "links".to_string());
        }
        for (a, b) in [(0, 4), (1, 5), (2, 6), (3, 7)] {
            graph.assert_semantic_predicate(
                keys[members[a]],
                keys[members[b]],
                "links".to_string(),
            );
        }
    }
    for (a, b) in [(0, 2), (3, 5), (6, 12), (15, 1)] {
        graph.assert_semantic_predicate(keys[a], keys[b], "links".to_string());
    }
    let topics = keys.iter().enumerate().map(|(i, &k)| (k, i / 8)).collect();
    (graph, keys, topics)
}

/// The share of nodes whose group's most common label is their own: each
/// group counts its largest label.
pub fn purity(groups: &[(NodeKey, u32)], labels: &HashMap<NodeKey, usize>) -> f64 {
    let mut counts: HashMap<u32, HashMap<usize, usize>> = HashMap::new();
    for (key, group) in groups {
        *counts
            .entry(*group)
            .or_default()
            .entry(labels[key])
            .or_default() += 1;
    }
    let hits: usize = counts
        .values()
        .map(|by_label| by_label.values().copied().max().unwrap_or(0))
        .sum();
    hits as f64 / groups.len().max(1) as f64
}

/// Purity with the roles swapped: each label counts its largest group, so a
/// partition into singletons scores low here where it scores 1 above.
pub fn inverse_purity(groups: &[(NodeKey, u32)], labels: &HashMap<NodeKey, usize>) -> f64 {
    let as_labels: HashMap<NodeKey, usize> =
        groups.iter().map(|(k, g)| (*k, *g as usize)).collect();
    let as_groups: Vec<(NodeKey, u32)> =
        groups.iter().map(|(k, _)| (*k, labels[k] as u32)).collect();
    purity(&as_groups, &as_labels)
}

/// The fixture's topics dealt out again by a fixed shuffle, eight of each: a
/// labelling no embedding can know, for the purity measure's control.
pub fn shuffled_topics(keys: &[NodeKey]) -> HashMap<NodeKey, usize> {
    let mut labels: Vec<usize> = (0..keys.len()).map(|i| i / 8).collect();
    let mut state: u64 = 0x5EED;
    for i in (1..labels.len()).rev() {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let j = (state >> 33) as usize % (i + 1);
        labels.swap(i, j);
    }
    keys.iter().copied().zip(labels).collect()
}

/// The harmonic mean of purity and inverse purity: high only when the groups
/// are both pure and whole.
pub fn f_measure(groups: &[(NodeKey, u32)], labels: &HashMap<NodeKey, usize>) -> f64 {
    let (p, i) = (purity(groups, labels), inverse_purity(groups, labels));
    if p + i == 0.0 {
        0.0
    } else {
        2.0 * p * i / (p + i)
    }
}

/// Groups as a partition: the member sets, order-free.
pub fn partition(groups: &[(NodeKey, u32)]) -> Vec<Vec<NodeKey>> {
    let mut by: HashMap<u32, Vec<NodeKey>> = HashMap::new();
    for (key, group) in groups {
        by.entry(*group).or_default().push(*key);
    }
    let mut sets: Vec<Vec<NodeKey>> = by
        .into_values()
        .map(|mut members| {
            members.sort_by_key(|k| k.index());
            members
        })
        .collect();
    sets.sort_by_key(|members| members[0].index());
    sets
}
