// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use kernel::graph::apply::{GraphDelta, apply_graph_delta};
use kernel::graph::{
    CapturedDelta, EdgeAssertion, NavigationTrigger, ProvenanceSubKind, SemanticSubKind,
};

#[test]
fn projected_signal_buckets_keep_surface_edges_aliases_and_numerical_parity() {
    let (mut graph, nodes) = graph_from_edges(5, &[]);
    let (hub, leaf, alias, surface_leaf, isolated) =
        (nodes[0], nodes[1], nodes[2], nodes[3], nodes[4]);
    graph
        .assert_relation(
            hub,
            leaf,
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Hyperlink,
                label: None,
                decay_progress: None,
            },
        )
        .unwrap();
    graph
        .assert_relation(
            hub,
            leaf,
            EdgeAssertion::Provenance {
                sub_kind: ProvenanceSubKind::ClippedFrom,
            },
        )
        .unwrap();
    apply_graph_delta(
        &mut graph,
        GraphDelta::AppendTraversal {
            from: surface_leaf,
            to: hub,
            trigger: NavigationTrigger::Programmatic,
            timestamp_ms: Some(42),
        },
    );
    let resource = graph.shown_resource_id(leaf).unwrap();
    let surface_id = graph.get_node(alias).unwrap().id.to_string();
    kernel::graph::replay_captured_deltas_onto(
        &mut graph,
        [CapturedDelta::ReplaySetShownResourceById {
            surface_id,
            resource_id: Some(resource.to_string()),
        }],
    );
    assert_eq!(graph.shown_resource_id(alias), Some(resource));
    assert_eq!(graph.resource_relations().count(), 1);
    assert_eq!(graph.relations().count(), 1);
    assert_eq!(
        graph.projected_relations().count(),
        5,
        "two Resource families lift to both aliases beside Surface Traversal"
    );
    assert_eq!(
        TopologyView::neighbors_undirected(&graph, hub).count(),
        3,
        "signals count buckets, not relation classifier rows"
    );
    let weights = degree_importance(&graph);
    assert_eq!(weights.lookup(hub), Some(1.0));
    for node in [leaf, alias, surface_leaf] {
        assert_eq!(weights.lookup(node), Some(1.0 / 3.0));
    }
    assert_eq!(weights.lookup(isolated), Some(0.0));
    assert_eq!(betweenness_importance(&graph).lookup(hub), Some(1.0));
    assert_eq!(aps(&graph), vec![hub]);
    assert_eq!(
        affinity_of(&structural_affinity(&graph, 0.0), leaf, alias),
        Some(1.0)
    );
    assert_eq!(
        affinity_of(&structural_affinity(&graph, 0.0), alias, isolated),
        None
    );

    // The same exact buckets held wholly on Surfaces feed unchanged algorithms.
    let mut recorded = graph.to_snapshot();
    let content = recorded.resource_edges[0].clone();
    for target in [leaf, alias] {
        let mut edge = content.clone();
        edge.from_node_id = graph.get_node(hub).unwrap().id.to_string();
        edge.to_node_id = graph.get_node(target).unwrap().id.to_string();
        recorded.edges.push(edge);
    }
    recorded.resources.clear();
    recorded.resource_edges.clear();
    recorded.shown_resources.clear();
    let surface = Graph::try_from_recorded_snapshot(&recorded).unwrap();
    for metric in [ImportanceMetric::Degree, ImportanceMetric::Betweenness] {
        let projected = importance(&graph, metric);
        let held = importance(&surface, metric);
        for node in &nodes {
            assert_eq!(projected.lookup(*node), held.lookup(*node));
        }
    }
    assert_eq!(community_louvain(&graph), community_louvain(&surface));
    assert_eq!(
        structural_affinity(&graph, 0.0),
        structural_affinity(&surface, 0.0)
    );
    assert_eq!(aps(&graph), aps(&surface));
    for metric in [BridgeMetric::Articulation, BridgeMetric::Betweenness] {
        let mut projected = bridges(&graph, metric, 0.5).bridges;
        let mut held = bridges(&surface, metric, 0.5).bridges;
        projected.sort();
        held.sort();
        assert_eq!(projected, held);
    }

    for node in [leaf, alias] {
        let surface_id = graph.get_node(node).unwrap().id.to_string();
        kernel::graph::replay_captured_deltas_onto(
            &mut graph,
            [CapturedDelta::ReplaySetShownResourceById {
                surface_id,
                resource_id: None,
            }],
        );
        assert_eq!(graph.shown_resource_id(node), None);
    }
    assert_eq!(
        TopologyView::neighbors_undirected(&graph, hub).collect::<Vec<_>>(),
        [surface_leaf]
    );
    assert_eq!(
        graph.resource_relations().count(),
        1,
        "unshown content is retained"
    );
    let weights = degree_importance(&graph);
    assert_eq!(weights.lookup(hub), Some(1.0));
    assert_eq!(weights.lookup(surface_leaf), Some(1.0));
    for node in [leaf, alias, isolated] {
        assert_eq!(weights.lookup(node), Some(0.0));
    }
    assert!(aps(&graph).is_empty());
    assert!(structural_affinity(&graph, 0.0).pairs.is_empty());
    assert_eq!(community_louvain(&graph).clusters.len(), 4);
}

#[test]
fn projected_signal_self_loops_keep_legacy_incident_bucket_count() {
    let (mut graph, nodes) = graph_from_edges(2, &[]);
    let id = graph.get_node(nodes[0]).unwrap().id.to_string();
    kernel::graph::replay_captured_deltas_onto(
        &mut graph,
        [CapturedDelta::ReplayAssertRelationByIds {
            from_id: id.clone(),
            to_id: id,
            assertion: EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::UserGrouped,
                label: None,
                decay_progress: None,
            },
        }],
    );
    let old_count = Graph::neighbors_undirected(&graph, nodes[0]).count();
    assert_eq!(old_count, 1);
    assert_eq!(
        TopologyView::neighbors_undirected(&graph, nodes[0]).count(),
        old_count
    );
    graph
        .assert_relation(
            nodes[0],
            nodes[0],
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Hyperlink,
                label: None,
                decay_progress: None,
            },
        )
        .unwrap();
    assert_eq!(graph.resource_relations().count(), 1);
    assert_eq!(
        TopologyView::neighbors_undirected(&graph, nodes[0]).count(),
        2,
        "each Surface or Resource self-loop bucket is counted once"
    );
    assert_eq!(degree_importance(&graph).lookup(nodes[0]), Some(1.0));
    assert_eq!(degree_importance(&graph).lookup(nodes[1]), Some(0.0));
    assert!(structural_affinity(&graph, 0.0).pairs.is_empty());
    assert!(articulation_points(&graph).bridges.is_empty());
}
