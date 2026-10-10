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

// The historical goldens used bare Surface nodes and Surface assertions.
// Live Resource fixtures remain the default for current producer tests.
fn fixture_node(
    graph: &mut Graph,
    legacy_surface: bool,
    id: Uuid,
    url: String,
    position: PortablePoint,
) -> NodeKey {
    if legacy_surface {
        graph.add_node_with_id(id, url, position)
    } else {
        add_node(graph, Some(id), url, position)
    }
}

fn fixture_link(graph: &mut Graph, legacy_surface: bool, from: NodeKey, to: NodeKey) {
    if legacy_surface {
        let from_id = graph.get_node(from).unwrap().id;
        let to_id = graph.get_node(to).unwrap().id;
        let asserter_iri = graph.write_author().asserter_iri();
        apply_graph_delta(
            graph,
            GraphDelta::ReplayAssertSemanticPredicateByIds {
                from_id,
                to_id,
                predicate: "links".into(),
                asserter_iri,
            },
        );
    } else {
        graph.assert_semantic_predicate(from, to, "links".into());
    }
}

/// The topic fixture's 32 titles, sites and links, with fixed ids and visit
/// times (pairs of nodes share a visit, so the recency order meets ties).
pub(crate) fn topic_fixture() -> (Graph, Vec<NodeKey>) {
    topic_fixture_in_store(false)
}

fn topic_fixture_in_store(legacy_surface: bool) -> (Graph, Vec<NodeKey>) {
    let mut graph = Graph::new();
    let keys: Vec<NodeKey> = (0..32)
        .map(|i| {
            let key = fixture_node(
                &mut graph,
                legacy_surface,
                Uuid::from_u128(0x7000 + i as u128),
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
            fixture_link(&mut graph, legacy_surface, keys[a], keys[b]);
        }
        for (a, b) in [(0, 4), (1, 5), (2, 6), (3, 7)] {
            fixture_link(
                &mut graph,
                legacy_surface,
                keys[members[a]],
                keys[members[b]],
            );
        }
    }
    for (a, b) in [(0, 2), (3, 5), (6, 12), (15, 1)] {
        fixture_link(&mut graph, legacy_surface, keys[a], keys[b]);
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
    generated_fixture_in_store(n, false)
}

fn generated_fixture_in_store(n: usize, legacy_surface: bool) -> (Graph, Vec<NodeKey>) {
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
            let key = fixture_node(
                &mut graph,
                legacy_surface,
                Uuid::from_u128(0x5000_0000 + i as u128),
                format!("https://site{}.example/{i}", i % 9),
                PortablePoint::zero(),
            );
            assert!(graph.set_node_title(key, format!("Generated page {i}")));
            key
        })
        .collect();
    for i in 1..n {
        let parent = (next() as usize) % i;
        fixture_link(&mut graph, legacy_surface, keys[i], keys[parent]);
    }
    for _ in 0..n / 2 {
        let (a, b) = ((next() as usize) % n, (next() as usize) % n);
        if a != b {
            fixture_link(&mut graph, legacy_surface, keys[a], keys[b]);
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
    arrangement_projections(graph, keys)
        .into_iter()
        .map(|(name, projection)| {
            let score = projection.score.as_ref().map(score_hash).unwrap_or(0);
            (name, positions_hash(&projection.positions), score)
        })
        .collect()
}

fn arrangement_projections(
    graph: &Graph,
    keys: &[NodeKey],
) -> Vec<(String, crate::canvas::CanvasStrategyProjection)> {
    let extents = extents(keys);
    // One registry for the one graph, as a host holds it across calls (F87).
    let mut registry = crate::signals::ChannelRegistry::new();
    let mut out = Vec::new();
    let mut record = |name: String, projection: crate::canvas::CanvasStrategyProjection| {
        out.push((name, projection));
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
                    &mut registry,
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
            &mut registry,
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
    // no canvas strategy picks it, so it is projected through cartography,
    // handed the order, the rings and the weights from the registry as a host
    // does (G2b moved the producers out of cartography; F84 and F86 key them).
    {
        use cartography::LayoutStrategy;
        let signals = registry.disclose(
            graph,
            &[
                cartography::ORDER_TIMELINE,
                cartography::RINGS_FOCUS,
                cartography::WEIGHT_DEGREE,
            ],
            Some(keys[0]),
        );
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
            crate::canvas::CanvasStrategyProjection {
                coverage: Default::default(),
                positions,
                score: None,
            },
        ));
    }
    out
}

/// The goldens, taken on `7ea4b77f` before anything moved (G2b step 1).
#[rustfmt::skip]
const HISTORICAL_GOLDENS: &[(&str, &str, u64, u64)] = &[
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

// Captured independently from untouched main e5a24a4af on Linux. Keep the
// historical captures too; their arithmetic differs from this platform's
// bit hashes. The paired-store test below qualifies the new routing directly.
#[rustfmt::skip]
const LINUX_GOLDENS: &[(&str, &str, u64, u64)] = &[
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
    ("topic", "radial weighted focus 0", 0x143223deca19b751, 0x0000000000000000),
    ("generated 500", "phyllotaxis.default recent_first true", 0xe35641c388bd0762, 0xbf7d39e9cb5455cd),
    ("generated 500", "phyllotaxis.default recent_first false", 0x05cc902e1c85a84e, 0x18f34e3ca4421cfd),
    ("generated 500", "grid.default recent_first true", 0x2c8eb12fa9da7098, 0x0000000000000000),
    ("generated 500", "spectral.default recent_first true", 0xd0d787a1609d27d1, 0x0000000000000000),
    ("generated 500", "penrose.default recent_first true", 0x6c7bd7d4d9651718, 0x0000000000000000),
    ("generated 500", "lsystem.default recent_first true", 0xf5f0f54024d08202, 0x0000000000000000),
    ("generated 500", "kanban.default recent_first true", 0x8e1d4d588a6c9fb9, 0x0000000000000000),
    ("generated 500", "kanban.community recent_first true", 0x0ef74f7fc29541f7, 0x0000000000000000),
    ("generated 500", "timeline.default recent_first true", 0x9b32cf772f256e3b, 0x0000000000000000),
    ("generated 500", "radial.default focus 0", 0xa15022c25bb7cea3, 0x0000000000000000),
    ("generated 500", "radial weighted focus 0", 0xb6d38886135c5dc3, 0x0000000000000000),
];

const GOLDENS: &[(&str, &str, u64, u64)] = if cfg!(target_os = "linux") {
    LINUX_GOLDENS
} else {
    HISTORICAL_GOLDENS
};

#[test]
fn every_arrangement_matches_its_golden_on_both_fixtures() {
    let mut taken = Vec::new();
    for (fixture, (graph, keys)) in [
        ("topic", topic_fixture_in_store(true)),
        ("generated 500", generated_fixture_in_store(500, true)),
    ] {
        for (name, positions, score) in arrangement_hashes(&graph, &keys) {
            println!("    (\"{fixture}\", \"{name}\", 0x{positions:016x}, 0x{score:016x}),");
            taken.push((fixture, name, positions, score));
        }
    }
    // The fixtures are fixed: the same hashes twice in one run.
    let (graph, keys) = topic_fixture_in_store(true);
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

#[test]
fn resource_and_surface_topologies_have_identical_arrangements() {
    for (legacy, live) in [
        (topic_fixture_in_store(true), topic_fixture_in_store(false)),
        (
            generated_fixture_in_store(500, true),
            generated_fixture_in_store(500, false),
        ),
    ] {
        let (legacy, legacy_keys) = legacy;
        let (live, live_keys) = live;
        assert!(legacy.resource_nodes().next().is_none());
        assert!(live.resource_edges().next().is_some());
        assert_eq!(
            live.relations().count(),
            0,
            "former Surface-only producers see nothing"
        );
        for ((name, mut before), (other, mut after)) in
            arrangement_projections(&legacy, &legacy_keys)
                .into_iter()
                .zip(arrangement_projections(&live, &live_keys))
        {
            assert_eq!(name, other);
            assert_eq!(
                positions_hash(&before.positions),
                positions_hash(&after.positions),
                "{name} geometry"
            );
            // Revisions count mutations to different stores. Every other score
            // field must be byte-for-byte identical, not merely visually close.
            if let Some(score) = &mut before.score {
                score.generation = 0;
            }
            if let Some(score) = &mut after.score {
                score.generation = 0;
            }
            assert_eq!(
                serde_json::to_value(before.score).unwrap(),
                serde_json::to_value(after.score).unwrap(),
                "{name} score"
            );
        }
    }
}
