// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use kernel::geometry::PortablePoint;
use kernel::graph::fixtures::GraphFixtures;
use uuid::Uuid;

fn triangle_graph() -> (Graph, [NodeKey; 3]) {
    let mut graph = Graph::new();
    let a = graph.add_node_with_id(
        Uuid::from_u128(1),
        "test://a".into(),
        PortablePoint::new(0.0, 0.0),
    );
    let b = graph.add_node_with_id(
        Uuid::from_u128(2),
        "test://b".into(),
        PortablePoint::new(100.0, 0.0),
    );
    let c = graph.add_node_with_id(
        Uuid::from_u128(3),
        "test://c".into(),
        PortablePoint::new(50.0, 86.6),
    );
    (graph, [a, b, c])
}

#[test]
fn coverage_refreshes_without_recomputing_channels_or_geometry() {
    use kernel::graph::{CoverageLayer, CoverageLimit, CoverageNote};
    let (mut graph, _) = triangle_graph();
    let mut registry = ChannelRegistry::new();
    for id in ["phyllotaxis.default", "grid.default", "kanban.default"] {
        graph.set_known_coverage(Default::default());
        let before = project_canvas_strategy_with_score(
            &mut registry,
            id,
            &graph,
            None,
            800,
            600,
            None,
            None,
            false,
        );
        let runs = registry.runs();
        let note = CoverageNote {
            limits: CoverageLayer::ALL
                .into_iter()
                .map(|layer| CoverageLimit::new(layer, "host boundary"))
                .collect(),
        };
        graph.set_known_coverage(note.clone());
        let after = project_canvas_strategy_with_score(
            &mut registry,
            id,
            &graph,
            None,
            800,
            600,
            None,
            None,
            false,
        );
        let mut before_positions = before.positions;
        let mut after_positions = after.positions;
        before_positions.sort_by_key(|(key, _)| *key);
        after_positions.sort_by_key(|(key, _)| *key);
        assert_eq!(before_positions, after_positions);
        assert_eq!(before.score, after.score);
        assert_eq!(registry.runs(), runs);
        assert!(
            note.limits
                .iter()
                .all(|limit| after.coverage.limits.contains(limit)),
            "{id}"
        );
    }
    for (id, reason) in [
        ("radial.default", "projection focus unavailable"),
        ("unknown", "projection strategy unavailable"),
    ] {
        let result = project_canvas_strategy_with_score(
            &mut registry,
            id,
            &graph,
            None,
            800,
            600,
            None,
            None,
            false,
        );
        assert!(result.positions.is_empty());
        assert!(
            result
                .coverage
                .limits
                .iter()
                .any(|limit| limit.layer == CoverageLayer::Projection && limit.reason == reason)
        );
        assert!(
            graph
                .known_coverage()
                .limits
                .iter()
                .all(|limit| result.coverage.limits.contains(limit))
        );
    }
}

#[test]
fn project_spiral_uses_the_portable_score_solver() {
    let (graph, _) = triangle_graph();
    let projection = project_canvas_dispatch(
        &mut ChannelRegistry::new(),
        "phyllotaxis.default",
        &graph,
        None,
        800,
        600,
        None,
        None,
        false,
    );
    assert_eq!(projection.nodes.len(), 3);
    assert_eq!(
        projection.metadata.strategy_id.as_deref(),
        Some("phyllotaxis.default")
    );
    assert!(projection.metadata.settled);
}

#[test]
fn kanban_community_groups_each_cluster_into_its_own_column() {
    // Two triangles {0,1,2} and {3,4,5} joined by one bridge edge => two Louvain communities,
    // so the cluster-kanban board lays them out as two columns.
    let mut graph = Graph::new();
    let n: Vec<NodeKey> = (0..6)
        .map(|i| graph.add_node(format!("test://{i}"), PortablePoint::new(i as f32, 0.0)))
        .collect();
    for &(a, b) in &[(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)] {
        graph.assert_semantic_predicate(n[a], n[b], "links".to_string());
    }
    let positions = project_canvas_strategy(
        &mut ChannelRegistry::new(),
        "kanban.community",
        &graph,
        None,
        800,
        600,
        None,
        None,
        false,
    );
    assert_eq!(positions.len(), 6, "every node is placed");
    let x_of = |key: NodeKey| positions.iter().find(|(k, _)| *k == key).unwrap().1.x;
    assert!(
        (x_of(n[0]) - x_of(n[1])).abs() < 0.001,
        "triangle A shares one column"
    );
    assert!((x_of(n[0]) - x_of(n[2])).abs() < 0.001);
    assert!(
        (x_of(n[3]) - x_of(n[4])).abs() < 0.001,
        "triangle B shares one column"
    );
    assert!(
        (x_of(n[0]) - x_of(n[3])).abs() > 0.001,
        "the two communities are distinct columns"
    );
}

#[test]
fn project_canvas_strategy_radial_centers_on_focus_and_no_ops_without_one() {
    let (graph, [a, _, _]) = triangle_graph();
    // With a focus, radial lays out the whole graph (focus at center).
    let with_focus = project_canvas_strategy(
        &mut ChannelRegistry::new(),
        "radial.default",
        &graph,
        Some(a),
        800,
        600,
        None,
        None,
        false,
    );
    assert_eq!(
        with_focus.len(),
        3,
        "radial projects every node around the focus"
    );
    let focus_pos = with_focus.iter().find(|(k, _)| *k == a).unwrap().1;
    assert!(
        focus_pos.x.abs() < 0.001 && focus_pos.y.abs() < 0.001,
        "the focus sits at the radial center"
    );
    // Without a focus there is nothing to center on, so it leaves the layout alone.
    let no_focus = project_canvas_strategy(
        &mut ChannelRegistry::new(),
        "radial.default",
        &graph,
        None,
        800,
        600,
        None,
        None,
        false,
    );
    assert!(no_focus.is_empty(), "radial without a selection no-ops");
}

#[test]
fn signal_overlays_builds_halos_in_cluster_order_then_bridge_emphasis() {
    use cartography::{BridgeNodes, Cluster, ClusterSet, Overlay};
    let (a, b, c) = (NodeKey::new(0), NodeKey::new(1), NodeKey::new(2));
    let clusters = ClusterSet {
        clusters: vec![
            Cluster {
                id: "c0".into(),
                label: Some("A".into()),
                members: vec![a, b],
                confidence: 1.0,
            },
            Cluster {
                id: "c1".into(),
                label: None,
                members: vec![c],
                confidence: 1.0,
            },
        ],
    };
    let bridges = BridgeNodes { bridges: vec![b] };
    let overlays = signal_overlays(Some(&clusters), Some(&bridges));
    assert_eq!(overlays.len(), 3, "two halos + one bridge emphasis");
    assert!(
        matches!(&overlays[0], Overlay::ClusterHalo { members, .. } if *members == vec![a, b]),
        "the first halo is the first cluster (order preserved for colour matching)"
    );
    assert!(matches!(&overlays[1], Overlay::ClusterHalo { cluster_id, .. } if cluster_id == "c1"));
    assert!(matches!(&overlays[2], Overlay::BridgeEmphasis { node, .. } if *node == b));
}

#[test]
fn signal_overlays_empty_without_signals() {
    assert!(
        signal_overlays(None, None).is_empty(),
        "no signals => no overlays"
    );
}

#[test]
fn project_canvas_lens_carries_the_community_halos() {
    // Two triangles + a bridge: the lens projection carries the two community halos on the
    // overlay channel, so the gloss can paint the clusters at its own (spectral) positions.
    let mut graph = Graph::new();
    let n: Vec<NodeKey> = (0..6)
        .map(|i| graph.add_node(format!("test://{i}"), PortablePoint::new(i as f32, 0.0)))
        .collect();
    for &(a, b) in &[(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)] {
        graph.assert_semantic_predicate(n[a], n[b], "links".to_string());
    }
    let clusters = crate::signals::community_louvain(&graph);
    let lens = project_canvas_lens(
        &mut ChannelRegistry::new(),
        "spectral.default",
        &graph,
        None,
        800,
        600,
        Some(&clusters),
        None,
    );
    assert_eq!(lens.nodes.len(), 6, "spectral lays out every node");
    let halos = lens
        .overlays
        .iter()
        .filter(|o| matches!(o, cartography::Overlay::ClusterHalo { .. }))
        .count();
    assert_eq!(halos, 2, "two communities => two halos ride the projection");
    // The positions-only path is unchanged (it drops the overlays).
    let positions = project_canvas_strategy(
        &mut ChannelRegistry::new(),
        "spectral.default",
        &graph,
        None,
        800,
        600,
        Some(&clusters),
        None,
        false,
    );
    assert_eq!(
        positions.len(),
        6,
        "the positions wrapper still returns every node"
    );
}

#[test]
fn project_canvas_subgraph_lays_out_only_the_scope() {
    // Path a-b-c-d; scope to {a, b}. The subgraph projection returns positions only for the
    // scoped nodes, keyed by the ORIGINAL graph's keys (remapped via stable id). (P6c.)
    let mut graph = Graph::new();
    let n: Vec<NodeKey> = (0..4)
        .map(|i| graph.add_node(format!("test://{i}"), PortablePoint::new(i as f32, 0.0)))
        .collect();
    for &(a, b) in &[(0, 1), (1, 2), (2, 3)] {
        graph.assert_relation(
            n[a],
            n[b],
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Hyperlink,
                label: None,
            },
        );
    }
    let scope = vec![n[0], n[1]];
    let positions = project_canvas_subgraph(&graph, &scope, "spectral.default", None, 800, 600);
    assert_eq!(positions.len(), 2, "only the scoped nodes are laid out");
    let keys: HashSet<NodeKey> = positions.iter().map(|(k, _)| *k).collect();
    assert!(
        keys.contains(&n[0]) && keys.contains(&n[1]),
        "positions are keyed by the original graph keys"
    );
    assert!(
        !keys.contains(&n[2]) && !keys.contains(&n[3]),
        "out-of-scope nodes are absent from the lens"
    );
}

#[test]
fn build_projection_request_threads_intent_from_options() {
    let (graph, [a, _, _]) = triangle_graph();
    let signals = IntelligenceSignals::default();
    let options = CartographySceneOptions::minimap(120.0, 90.0).with_focus(a);
    let request = build_projection_request(&graph, &signals, &options);
    assert_eq!(request.intent.form_factor, FormFactor::Minimap);
    assert_eq!(request.intent.focus, Some(a));
    assert_eq!(
        request.intent.target_size,
        TargetSize::Logical {
            width: 120.0,
            height: 90.0
        }
    );
}

#[test]
fn options_default_is_canvas_form_factor() {
    let opts = CartographySceneOptions::default();
    assert_eq!(opts.form_factor, FormFactor::Canvas);
    assert_eq!(opts.dimension, ProjectionDimension::TwoD);
    assert_eq!(opts.target_size, TargetSize::Default);
}
