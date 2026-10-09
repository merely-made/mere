// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The channel registry's receipts (dynamics grammar plan, G2b): each
//! producer runs once per key and once more when its key moves, with the
//! counter shown counting; and the producers that moved from cartography give
//! their pre-migration placements through cartography's adapters.

use std::collections::HashMap;

use cartography::adapters::channels_read;
use cartography::{LayoutStrategy, ORDER_TIMELINE, ProjectionRequest, TargetSize, ViewIntent};
use kernel::geometry::PortablePoint;
use kernel::graph::apply::{GraphDelta, add_node, apply_graph_delta};
use kernel::graph::fixtures::GraphFixtures;
use kernel::graph::{EdgeAssertion, Graph, NodeKey, SemanticSubKind};
use uuid::Uuid;

use super::{BridgeMetric, ChannelRegistry, ImportanceMetric, RegistryRuns, recency};

const EPSILON: f32 = 0.01;

/// cartography's parity fixture: a hub with three spokes, a pair, and a loner.
fn parity_fixture() -> (Graph, Vec<NodeKey>) {
    parity_fixture_in_store(false)
}

fn parity_fixture_in_store(legacy_surface: bool) -> (Graph, Vec<NodeKey>) {
    let mut graph = Graph::new();
    let keys: Vec<NodeKey> = (0..7u8)
        .map(|i| {
            let id = Uuid::from_u128(0x1000_0000_0000_0000_0000_0000_0000_0000u128 + i as u128);
            graph.add_node_with_id(
                id,
                format!("https://parity.example/{i}"),
                PortablePoint::new(0.0, 0.0),
            )
        })
        .collect();
    let hyperlink = || EdgeAssertion::Semantic {
        sub_kind: SemanticSubKind::Hyperlink,
        label: None,
        decay_progress: None,
    };
    let mut assert = |from: NodeKey, to: NodeKey| {
        if legacy_surface {
            let from_id = graph.get_node(from).unwrap().id.to_string();
            let to_id = graph.get_node(to).unwrap().id.to_string();
            kernel::graph::replay_captured_deltas_onto(
                &mut graph,
                [kernel::graph::CapturedDelta::ReplayAssertRelationByIds {
                    from_id,
                    to_id,
                    assertion: hyperlink(),
                }],
            );
        } else {
            graph.assert_relation(from, to, hyperlink());
        }
    };
    for spoke in 1..=3 {
        assert(keys[0], keys[spoke]);
    }
    assert(keys[4], keys[5]);
    (graph, keys)
}

fn placed(projection: &cartography::Projection) -> HashMap<NodeKey, PortablePoint> {
    projection
        .nodes
        .iter()
        .map(|node| (node.node, node.position))
        .collect()
}

fn assert_golden(
    label: &str,
    keys: &[NodeKey],
    projection: &cartography::Projection,
    golden: &[(f32, f32)],
) {
    let by_key = placed(projection);
    assert_eq!(by_key.len(), golden.len(), "{label}");
    for (index, key) in keys.iter().enumerate() {
        let at = by_key[key];
        let (x, y) = golden[index];
        assert!(
            (at.x - x).abs() < EPSILON && (at.y - y).abs() < EPSILON,
            "{label}: node {index} at ({:.4}, {:.4}), golden ({x:.4}, {y:.4})",
            at.x,
            at.y
        );
    }
}

/// cartography's spectral golden, from the registry's coordinates.
#[test]
fn spectral_from_the_registry_matches_the_pre_migration_placement() {
    for legacy_surface in [false, true] {
        let (graph, keys) = parity_fixture_in_store(legacy_surface);
        let mut registry = ChannelRegistry::new();
        let adapter = cartography::adapters::SpectralAdapter::default();
        let signals = registry.disclose(
            &graph,
            channels_read(cartography::adapters::SpectralAdapter::PROJECTION_ID),
            None,
        );
        let projection = adapter.project(&ProjectionRequest {
            graph: &graph,
            signals: &signals,
            intent: ViewIntent {
                target_size: TargetSize::default(),
                ..ViewIntent::default()
            },
        });
        assert_golden(
            "spectral",
            &keys,
            &projection,
            &[
                (146.7027, 34.4556),
                (146.7027, 34.4556),
                (146.7027, 34.4556),
                (146.7027, 34.4556),
                (-152.6299, -228.9111),
                (-152.6299, -228.9111),
                (-281.5509, 320.0),
            ],
        );
    }
}

/// cartography's radial golden, from the registry's rings.
#[test]
fn radial_from_the_registry_matches_the_pre_migration_placement() {
    for legacy_surface in [false, true] {
        let (graph, keys) = parity_fixture_in_store(legacy_surface);
        let mut registry = ChannelRegistry::new();
        assert_eq!(
            registry.rings(&graph, keys[0]).len(),
            4,
            "the hub and its spokes; the rest unreachable"
        );
        let signals = registry.disclose(
            &graph,
            channels_read(cartography::adapters::RadialAdapter::PROJECTION_ID),
            Some(keys[0]),
        );
        let projection =
            cartography::adapters::RadialAdapter::default().project(&ProjectionRequest {
                graph: &graph,
                signals: &signals,
                intent: ViewIntent {
                    target_size: TargetSize::default(),
                    focus: Some(keys[0]),
                    ..ViewIntent::default()
                },
            });
        assert_golden(
            "radial",
            &keys,
            &projection,
            &[
                (0.0, 0.0),
                (120.0, 0.0),
                (-60.0, 103.923),
                (-60.0, -103.923),
                (240.0, 0.0),
                (-120.0, 207.8461),
                (-120.0, -207.8461),
            ],
        );
    }
}

/// Moved from cartography's spiral tests: visits in one tick tie, and the
/// tie goes to the larger stable id.
#[test]
fn equal_recency_uses_stable_identity_order() {
    let mut graph = Graph::new();
    let keys: Vec<_> = (1..=3)
        .map(|id| {
            add_node(
                &mut graph,
                Some(Uuid::from_u128(id)),
                format!("fixture://{id}"),
                PortablePoint::zero(),
            )
        })
        .collect();
    for key in &keys {
        let node_id = graph.get_node(*key).unwrap().id;
        apply_graph_delta(
            &mut graph,
            GraphDelta::ReplayTouchNodeLastVisitedById {
                node_id,
                timestamp_ms: 7,
            },
        );
    }
    let order = recency(&graph).order;
    assert_eq!(order, [keys[2], keys[1], keys[0]]);
    assert!(
        recency(&graph).values.values().all(|value| *value == 1.0),
        "one distinct visit time reads every node as newest"
    );
}

#[cfg(feature = "canvas")]
/// F51: size by recency takes the spiral's `f64` arithmetic. Against the
/// canvas's former `f32` form, on the generated graph with its own visit
/// times and again with millisecond times a few seconds apart, the two agree
/// to within a millionth.
#[test]
fn the_recency_values_agree_with_the_former_f32_form() {
    let former = |graph: &Graph| -> HashMap<NodeKey, f32> {
        let times: Vec<(NodeKey, std::time::SystemTime)> = graph
            .nodes()
            .map(|(key, _)| {
                (
                    key,
                    graph
                        .node_last_visited(key)
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                )
            })
            .collect();
        let oldest = times.iter().map(|(_, t)| *t).min().unwrap();
        let newest = times.iter().map(|(_, t)| *t).max().unwrap();
        let span = newest.duration_since(oldest).unwrap().as_secs_f32();
        times
            .into_iter()
            .map(|(key, t)| (key, t.duration_since(oldest).unwrap().as_secs_f32() / span))
            .collect()
    };
    let (mut graph, keys) = crate::canvas::tests::arrangement_goldens::generated_fixture(500);
    for case in ["the fixture's visits", "millisecond visits"] {
        if case == "millisecond visits" {
            for (i, key) in keys.iter().enumerate() {
                let node_id = graph.get_node(*key).unwrap().id;
                apply_graph_delta(
                    &mut graph,
                    GraphDelta::ReplayTouchNodeLastVisitedById {
                        node_id,
                        timestamp_ms: 1_759_000_000_000 + (i as u64 * 7_919) % 4_000_003,
                    },
                );
            }
        }
        let now = recency(&graph).values;
        let before = former(&graph);
        let (mut worst, mut differing) = (0.0_f32, 0);
        for (key, value) in &before {
            let diff = (value - now[key]).abs();
            worst = worst.max(diff);
            differing += usize::from(diff > 0.0);
        }
        println!(
            "recency on {case}, f64 against the former f32: {differing} of 500 differ, by at most {worst:e}"
        );
        assert!(worst < 1e-6, "{case}: {worst}");
    }
}

fn edge(graph: &mut Graph, a: NodeKey, b: NodeKey) {
    graph.assert_semantic_predicate(a, b, "links".to_string());
}

/// Every producer: one run for one key however often it is read, one more
/// when its key moves, none when an unrelated key moves; and a second
/// registry's direct call shows the counter counts from zero.
#[test]
fn each_producer_runs_once_per_key() {
    let (mut graph, keys) = parity_fixture();
    let mut registry = ChannelRegistry::new();
    let read_all = |registry: &mut ChannelRegistry, graph: &Graph| {
        for _ in 0..3 {
            registry.community(graph);
            registry.bridges(graph, BridgeMetric::Betweenness, 0.5);
            registry.structural_affinity(graph, 0.2);
            registry.importance(graph, ImportanceMetric::Degree);
            registry.importance(graph, ImportanceMetric::Betweenness);
            registry.recency(graph);
            registry.enumeration(graph);
            registry.spectral(graph, 200);
            registry.rings(graph, keys[0]);
            registry.degree_weights(graph);
            registry.sites(graph);
        }
    };
    read_all(&mut registry, &graph);
    let once = RegistryRuns {
        community: 1,
        bridges: 1,
        affinity: 1,
        importance: 2,
        recency: 1,
        enumeration: 1,
        spectral: 1,
        rings: 1,
        degree_weights: 1,
        sites: 1,
    };
    assert_eq!(registry.runs(), once, "each once, read three times");

    // A visit moves recency's key alone.
    let node_id = graph.get_node(keys[6]).unwrap().id;
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplayTouchNodeLastVisitedById {
            node_id,
            timestamp_ms: 9_999,
        },
    );
    read_all(&mut registry, &graph);
    assert_eq!(
        registry.runs(),
        RegistryRuns { recency: 2, ..once },
        "a visit reruns recency only"
    );

    // A recorded URL move retains the shown Resource binding, moving only sites.
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplaySetNodeUrlById {
            node_id,
            new_url: "https://elsewhere.example/6".to_string(),
        },
    );
    read_all(&mut registry, &graph);
    assert_eq!(
        registry.runs(),
        RegistryRuns {
            recency: 2,
            sites: 2,
            ..once
        },
        "a recorded URL move reruns the sites only"
    );

    // An edge moves structure, so every topology fact runs once more.
    edge(&mut graph, keys[5], keys[6]);
    read_all(&mut registry, &graph);
    assert_eq!(
        registry.runs(),
        RegistryRuns {
            community: 2,
            bridges: 2,
            affinity: 2,
            importance: 4,
            recency: 3,
            enumeration: 2,
            spectral: 2,
            rings: 2,
            degree_weights: 2,
            sites: 3,
        },
        "structure reruns everything keyed to it"
    );

    // A parameter is part of the key: another focus, metric, floor or count.
    registry.rings(&graph, keys[4]);
    registry.bridges(&graph, BridgeMetric::Articulation, 0.5);
    registry.structural_affinity(&graph, 0.3);
    registry.spectral(&graph, 50);
    let runs = registry.runs();
    assert_eq!(
        (runs.rings, runs.bridges, runs.affinity, runs.spectral),
        (3, 3, 3, 3)
    );

    // The control: a fresh registry's one direct call counts one.
    let mut fresh = ChannelRegistry::new();
    assert_eq!(fresh.runs(), RegistryRuns::default());
    fresh.spectral(&graph, 200);
    assert_eq!(fresh.runs().spectral, 1, "the counter counts");
}

/// Live navigation changes the Resource projected through a Surface, so cached
/// topology must lose the old page's links and regain them when it returns.
#[test]
fn live_resource_rebinding_invalidates_cached_topology() {
    let (mut graph, keys) = parity_fixture();
    let mut registry = ChannelRegistry::new();
    let before = registry.degree_weights(&graph).clone();
    assert_eq!(before[&keys[5]], 2.0);
    let resource = graph.shown_resource_id(keys[5]).unwrap();
    for (url, expected, runs) in [
        ("https://elsewhere.example/5", 1.0, 2),
        ("https://parity.example/5", before[&keys[5]], 3),
    ] {
        apply_graph_delta(
            &mut graph,
            GraphDelta::SetNodeUrl { key: keys[5], new_url: url.into() },
        );
        assert_eq!(registry.degree_weights(&graph)[&keys[5]], expected);
        let mut fresh = ChannelRegistry::new();
        assert_eq!(registry.degree_weights(&graph), fresh.degree_weights(&graph));
        assert_eq!(registry.runs().degree_weights, runs);
    }
    assert_eq!(graph.shown_resource_id(keys[5]), Some(resource));
    assert_eq!(registry.degree_weights(&graph), &before);
}

/// An off-thread partition is taken once for its revision, and a stale one
/// is not fresh.
#[test]
fn an_offered_partition_counts_once_and_goes_stale_with_structure() {
    let (mut graph, keys) = parity_fixture();
    let mut registry = ChannelRegistry::new();
    let partition = super::community_louvain(&graph);
    assert!(registry.offer_community(graph.revision(), partition.clone()));
    assert!(!registry.offer_community(graph.revision(), partition));
    assert_eq!(registry.runs().community, 1);
    assert!(registry.community_is_fresh(&graph));
    // Reading it now computes nothing.
    registry.community(&graph);
    assert_eq!(registry.runs().community, 1);
    edge(&mut graph, keys[4], keys[6]);
    assert!(!registry.community_is_fresh(&graph));
    assert!(
        registry.community_held().is_some(),
        "the last good one is held"
    );
}

/// One enumeration order, computed once by the registry and carried in the
/// request (F84): a score's ordinals follow `order.timeline` as the request
/// carries it, which the Timeline's axis and the Spiral's graph order also
/// read.
#[test]
fn the_enumeration_channel_is_the_order_every_score_ordinal_follows() {
    let (mut graph, keys) = parity_fixture();
    // A removal and a re-add, so the order is not the order of insertion.
    graph.remove_node(keys[2]);
    graph.add_node_with_id(
        Uuid::from_u128(0x2000),
        "https://parity.example/late".to_string(),
        PortablePoint::new(0.0, 0.0),
    );
    let mut registry = ChannelRegistry::new();
    let order = registry.enumeration(&graph).to_vec();
    let signals = registry.disclose(&graph, &[ORDER_TIMELINE], None);
    let request = ProjectionRequest {
        graph: &graph,
        signals: &signals,
        intent: ViewIntent::default(),
    };
    let (score, score_keys) = cartography::adapters::score_from_request(
        &request,
        sceno::Arrangement::Grid(sceno::Grid::default()),
        &cartography::adapters::Disclosures::default(),
    )
    .expect("the request carries the order");
    assert_eq!(
        registry.runs().enumeration,
        1,
        "one computation, read twice"
    );
    assert_eq!(
        score_keys, order,
        "the score enumerates the channel's order"
    );
    for (index, item) in score.items.iter().enumerate() {
        assert_eq!(item.ordinal as usize, index);
    }
}

#[test]
fn topology_producers_combine_resource_and_surface_buckets() {
    use super::producers::{degree_weights, radial_rings, spectral_coords};
    let (mut graph, keys) = parity_fixture();
    let resource = graph.shown_resource_id(keys[1]).unwrap();
    let alias = graph.add_node_with_id(
        Uuid::from_u128(99),
        "https://parity.example/1#alias".into(),
        PortablePoint::new(0.0, 0.0),
    );
    // Live lifecycle is deliberately explicit in this raw fixture.
    let alias_id = graph.get_node(alias).unwrap().id.to_string();
    kernel::graph::replay_captured_deltas_onto(
        &mut graph,
        [kernel::graph::CapturedDelta::ReplaySetShownResourceById {
            surface_id: alias_id,
            resource_id: Some(resource.to_string()),
        }],
    );
    graph.assert_relation(
        keys[0],
        keys[1],
        EdgeAssertion::Semantic {
            sub_kind: SemanticSubKind::UserGrouped,
            label: None,
            decay_progress: None,
        },
    );
    graph.assert_relation(
        keys[3],
        keys[6],
        EdgeAssertion::Semantic {
            sub_kind: SemanticSubKind::UserGrouped,
            label: None,
            decay_progress: None,
        },
    );
    let rings = radial_rings(&graph, keys[0]);
    assert_eq!(rings[&keys[0]], 0);
    for key in [keys[1], keys[2], keys[3], alias] {
        assert_eq!(rings[&key], 1);
    }
    assert_eq!(rings[&keys[6]], 2);
    assert!(!rings.contains_key(&keys[4]));
    let weights = degree_weights(&graph);
    assert_eq!(
        weights[&keys[0]], 6.0,
        "three resource buckets plus alias lift and one Surface bucket"
    );
    assert_eq!(
        weights[&keys[1]], 3.0,
        "Resource and Surface multiplicity both count"
    );
    assert_eq!(weights[&alias], 2.0, "content lifts to the alias once");
    assert_eq!(
        weights[&keys[6]], 2.0,
        "Surface-only neighbor remains visible"
    );
    assert_eq!(spectral_coords(&graph, 200).len(), graph.node_count());
}
