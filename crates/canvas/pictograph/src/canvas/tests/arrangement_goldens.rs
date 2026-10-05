// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! G2b's goldens (dynamics grammar plan, G2b, step 1): every arrangement's
//! output hashed on two fixed fixtures before cartography's disclosures move
//! into the registry, so the move can show it changed no placement. A hash is
//! FNV-1a over the positions as `f32` bits in key order, and over the score's
//! serialized bytes where the arrangement returns one. The fixtures fix every
//! node id and visit time, so nothing depends on the clock or a random id.

use std::collections::HashMap;

use kernel::geometry::PortablePoint;
use kernel::graph::apply::{GraphDelta, add_node, apply_graph_delta};
use kernel::graph::fixtures::GraphFixtures;
use kernel::graph::{Graph, NodeKey};
use uuid::Uuid;

use super::meaning_topics::{SITES, TITLES, community};
use crate::canvas::cartography_scene::{
    CANVAS_LAYOUT_STRATEGIES, project_canvas_strategy_with_score_for_view,
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

fn fnv(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

pub(crate) fn positions_hash(positions: &[(NodeKey, PortablePoint)]) -> u64 {
    let mut sorted = positions.to_vec();
    sorted.sort_by_key(|(key, _)| key.index());
    let mut hash = 0xcbf2_9ce4_8422_2325;
    for (key, at) in sorted {
        fnv(&mut hash, &(key.index() as u64).to_le_bytes());
        fnv(&mut hash, &at.x.to_bits().to_le_bytes());
        fnv(&mut hash, &at.y.to_bits().to_le_bytes());
    }
    hash
}

pub(crate) fn score_hash(score: &sceno::Score) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    fnv(
        &mut hash,
        &serde_json::to_vec(score).expect("a score serializes"),
    );
    hash
}

fn visit(graph: &mut Graph, key: NodeKey, timestamp_ms: u64) {
    let node_id = graph.get_node(key).unwrap().id;
    apply_graph_delta(
        graph,
        GraphDelta::ReplayTouchNodeLastVisitedById {
            node_id,
            timestamp_ms,
        },
    );
}

/// The topic fixture's 32 titles, sites and links, with fixed ids and visit
/// times (pairs of nodes share a visit, so the recency order meets ties).
pub(crate) fn topic_fixture() -> (Graph, Vec<NodeKey>) {
    let mut graph = Graph::new();
    let keys: Vec<NodeKey> = (0..32)
        .map(|i| {
            let key = add_node(
                &mut graph,
                Some(Uuid::from_u128(0x7000 + i as u128)),
                format!("https://{}.example/{i}", SITES[i % 4]),
                PortablePoint::new((i % 8) as f32 * 40.0, (i / 8) as f32 * 40.0),
            );
            assert!(graph.set_node_title(key, TITLES[i / 8][i % 8].to_string()));
            key
        })
        .collect();
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
    for (i, key) in keys.iter().enumerate() {
        visit(
            &mut graph,
            *key,
            1_700_000_000_000 + (i as u64 / 2) * 60_000,
        );
    }
    (graph, keys)
}

/// The generated graph: a seeded tree plus `n / 2` chords over nine sites,
/// with fixed ids and visit times drawn from a fixed sequence (ties
/// included).
pub(crate) fn generated_fixture(n: usize) -> (Graph, Vec<NodeKey>) {
    let mut state: u64 = 0x2545_f491_4f6c_dd1d;
    let mut next = move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        state >> 33
    };
    let mut graph = Graph::new();
    let keys: Vec<NodeKey> = (0..n)
        .map(|i| {
            let key = add_node(
                &mut graph,
                Some(Uuid::from_u128(0x5000_0000 + i as u128)),
                format!("https://site{}.example/{i}", i % 9),
                PortablePoint::zero(),
            );
            assert!(graph.set_node_title(key, format!("Generated page {i}")));
            key
        })
        .collect();
    for i in 1..n {
        let parent = (next() as usize) % i;
        graph.assert_semantic_predicate(keys[i], keys[parent], "links".to_string());
    }
    for _ in 0..n / 2 {
        let (a, b) = ((next() as usize) % n, (next() as usize) % n);
        if a != b {
            graph.assert_semantic_predicate(keys[a], keys[b], "links".to_string());
        }
    }
    for key in &keys {
        visit(&mut graph, *key, 1_700_000_000_000 + (next() % 200) * 1_000);
    }
    (graph, keys)
}

/// Measured faces, fixed: five sizes by index.
fn extents(keys: &[NodeKey]) -> HashMap<NodeKey, (f32, f32)> {
    keys.iter()
        .enumerate()
        .map(|(i, key)| {
            let side = 24.0 + (i % 5) as f32 * 8.0;
            (*key, (side, side))
        })
        .collect()
}

/// Every arrangement's `(name, positions hash, score hash)` on `graph`.
pub(crate) fn arrangement_hashes(graph: &Graph, keys: &[NodeKey]) -> Vec<(String, u64, u64)> {
    let extents = extents(keys);
    let mut out = Vec::new();
    let mut record = |name: String, projection: crate::canvas::CanvasStrategyProjection| {
        let score = projection.score.as_ref().map(score_hash).unwrap_or(0);
        out.push((name, positions_hash(&projection.positions), score));
    };
    for (id, _) in CANVAS_LAYOUT_STRATEGIES {
        let recent = [true, false];
        for recent_first in if *id == "phyllotaxis.default" {
            &recent[..]
        } else {
            &recent[..1]
        } {
            record(
                format!("{id} recent_first {recent_first}"),
                project_canvas_strategy_with_score_for_view(
                    id,
                    graph,
                    None,
                    WIDTH,
                    HEIGHT,
                    None,
                    Some(&extents),
                    *recent_first,
                    1.0,
                    None,
                ),
            );
        }
    }
    record(
        "radial.default focus 0".to_string(),
        project_canvas_strategy_with_score_for_view(
            "radial.default",
            graph,
            Some(keys[0]),
            WIDTH,
            HEIGHT,
            None,
            Some(&extents),
            true,
            1.0,
            None,
        ),
    );
    // Radial's weighted policy is the one reader of the degree weights, and
    // no canvas strategy picks it, so it is projected through cartography.
    {
        use cartography::LayoutStrategy;
        let signals = cartography::IntelligenceSignals::default();
        let mut options = crate::canvas::CartographySceneOptions::canvas_pixels(WIDTH, HEIGHT)
            .with_focus(keys[0]);
        options.extents = Some(extents.clone());
        let request = crate::canvas::build_projection_request(graph, &signals, &options);
        let projection = cartography::adapters::RadialAdapter {
            config: sceno::Radial {
                angular_policy: sceno::RadialAngularPolicy::Weighted,
                ..sceno::Radial::default()
            },
        }
        .project(&request);
        let positions: Vec<(NodeKey, PortablePoint)> = projection
            .nodes
            .iter()
            .map(|node| (node.node, node.position))
            .collect();
        out.push((
            "radial weighted focus 0".to_string(),
            positions_hash(&positions),
            0,
        ));
    }
    out
}

/// The goldens, taken on `7ea4b77f` before anything moved (G2b step 1).
#[rustfmt::skip]
const GOLDENS: &[(&str, &str, u64, u64)] = &[
    ("topic", "phyllotaxis.default recent_first true", 0x11d4ed56b1ecd0da, 0x4b58e36d54859225),
    ("topic", "phyllotaxis.default recent_first false", 0xb8309ba62e9f1b4a, 0x5969b7597f373d27),
    ("topic", "grid.default recent_first true", 0x43ad04ce7ccbfeca, 0x0000000000000000),
    ("topic", "spectral.default recent_first true", 0x4dd47e18a22025f5, 0x0000000000000000),
    ("topic", "penrose.default recent_first true", 0x3cdc98d37bf9b14a, 0x0000000000000000),
    ("topic", "lsystem.default recent_first true", 0x39409c082dbd6d6f, 0x0000000000000000),
    ("topic", "kanban.default recent_first true", 0x4e390e0f086b4059, 0x0000000000000000),
    ("topic", "kanban.community recent_first true", 0x7d9ec76289cca0bd, 0x0000000000000000),
    ("topic", "timeline.default recent_first true", 0x2812629cb6301615, 0x0000000000000000),
    ("topic", "radial.default focus 0", 0x7b86498e83ab96e0, 0x0000000000000000),
    ("topic", "radial weighted focus 0", 0xa7e7b5d148b5387f, 0x0000000000000000),
    ("generated 500", "phyllotaxis.default recent_first true", 0x5fecef5b283f1e94, 0xbf7d39e9cb5455cd),
    ("generated 500", "phyllotaxis.default recent_first false", 0xb91bd9d03607c36c, 0x18f34e3ca4421cfd),
    ("generated 500", "grid.default recent_first true", 0x2c8eb12fa9da7098, 0x0000000000000000),
    ("generated 500", "spectral.default recent_first true", 0xd0d787a1609d27d1, 0x0000000000000000),
    ("generated 500", "penrose.default recent_first true", 0x6c7bd7d4d9651718, 0x0000000000000000),
    ("generated 500", "lsystem.default recent_first true", 0xf5f0f54024d08202, 0x0000000000000000),
    ("generated 500", "kanban.default recent_first true", 0x8e1d4d588a6c9fb9, 0x0000000000000000),
    ("generated 500", "kanban.community recent_first true", 0x0ef74f7fc29541f7, 0x0000000000000000),
    ("generated 500", "timeline.default recent_first true", 0x9b32cf772f256e3b, 0x0000000000000000),
    ("generated 500", "radial.default focus 0", 0xb5efdbb33d9b168b, 0x0000000000000000),
    ("generated 500", "radial weighted focus 0", 0x401f146407bdb99a, 0x0000000000000000),
];

#[test]
fn every_arrangement_matches_its_golden_on_both_fixtures() {
    let mut taken = Vec::new();
    for (fixture, (graph, keys)) in [
        ("topic", topic_fixture()),
        ("generated 500", generated_fixture(500)),
    ] {
        for (name, positions, score) in arrangement_hashes(&graph, &keys) {
            println!("    (\"{fixture}\", \"{name}\", 0x{positions:016x}, 0x{score:016x}),");
            taken.push((fixture, name, positions, score));
        }
    }
    // The fixtures are fixed: the same hashes twice in one run.
    let (graph, keys) = topic_fixture();
    let again = arrangement_hashes(&graph, &keys);
    for (index, (name, positions, score)) in again.iter().enumerate() {
        assert_eq!(
            (&taken[index].1, taken[index].2, taken[index].3),
            (name, *positions, *score),
            "the topic fixture hashes the same twice"
        );
    }
    // A moved position changes a hash: the instrument can tell.
    let mut moved = vec![(keys[0], PortablePoint::new(1.0, 2.0))];
    let before = positions_hash(&moved);
    moved[0].1.x = f32::from_bits(1.0f32.to_bits() + 1);
    assert_ne!(positions_hash(&moved), before, "one ulp moves the hash");

    if GOLDENS.is_empty() {
        panic!("no goldens recorded yet: paste the lines above into GOLDENS");
    }
    assert_eq!(taken.len(), GOLDENS.len(), "one golden per arrangement");
    for ((fixture, name, positions, score), golden) in taken.iter().zip(GOLDENS) {
        assert_eq!(
            (*fixture, name.as_str(), *positions, *score),
            *golden,
            "{fixture} {name} moved from its golden"
        );
    }
}
