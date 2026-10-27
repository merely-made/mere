// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::canvas::physics_controls::{ArrangementTransition, advance_arrangement};

fn graph(pair: (usize, usize)) -> (Graph, Vec<NodeKey>) {
    let mut graph = Graph::new();
    let nodes: Vec<_> = (0..3)
        .map(|i| {
            graph.add_node_with_id(
                uuid::Uuid::from_u128(500 + i),
                format!("https://{i}.example"),
                Default::default(),
            )
        })
        .collect();
    graph.assert_semantic_predicate(nodes[pair.0], nodes[pair.1], "links".into());
    (graph, nodes)
}

#[test]
fn registry_does_not_reuse_another_graphs_equal_revision() {
    let (first, _) = graph((0, 1));
    let (second, _) = graph((1, 2));
    assert_eq!(first.revision(), second.revision());
    let mut registry = crate::signals::ChannelRegistry::new();
    let old = registry.degree_weights(&first).clone();
    let expected = crate::signals::ChannelRegistry::new()
        .degree_weights(&second)
        .clone();
    assert_ne!(old, expected);
    assert_eq!(registry.degree_weights(&second), &expected);
}

#[test]
fn registry_does_not_reuse_equal_revision_divergent_candidates() {
    let (base, nodes) = graph((0, 1));
    let mut first = base.clone();
    let mut second = base.clone();
    first.assert_semantic_predicate(nodes[0], nodes[2], "links".into());
    second.assert_semantic_predicate(nodes[1], nodes[2], "links".into());
    assert_eq!(first.revision(), second.revision());
    let mut registry = crate::signals::ChannelRegistry::new();
    let old = registry.degree_weights(&first).clone();
    let expected = crate::signals::ChannelRegistry::new()
        .degree_weights(&second)
        .clone();
    assert_ne!(old, expected);
    assert_eq!(registry.degree_weights(&second), &expected);
}

#[test]
fn meaning_refreshes_after_equal_counter_wholesale_ingest() {
    let (first, _) = graph((0, 1));
    let (second, _) = graph((1, 2));
    assert_eq!(first.content_revision(), second.content_revision());
    let mut canvas = Canvas::with_graph(first);
    assert!(canvas.refresh_meaning());
    let runs = canvas.meaning_runs();
    canvas.ingest_graph(|graph| {
        *graph = second;
        true
    });
    assert!(
        canvas.refresh_meaning(),
        "old graph's model snapshot was called fresh"
    );
    assert_eq!(canvas.meaning_runs(), runs + 1);
}

fn transition(canvas: &Canvas, key: NodeKey) -> Option<ArrangementTransition> {
    ArrangementTransition::between(canvas, &[(key, PortablePoint::new(700., 500.))]).unwrap()
}

#[test]
fn delayed_transition_cannot_land_on_a_recreated_member() {
    let (graph, nodes) = graph((0, 1));
    let mut canvas = Canvas::with_graph(graph);
    canvas.set_physics_paused(true);
    let key = nodes[0];
    let id = canvas.graph.get_node(key).unwrap().id;
    let mut pending = transition(&canvas, key);
    assert!(pending.is_some());
    canvas.ingest_graph(|graph| {
        graph.remove_node(key);
        assert_eq!(
            graph.add_node_with_id(id, "https://replacement.example".into(), Default::default()),
            key
        );
        true
    });
    let at = canvas.view.position_of(key);
    let status = advance_arrangement(&mut canvas, &mut pending, "grid.default", 0.);
    assert!(status.is_none_or(|status| !status.starts_with("Arrangement set to")));
    assert!(pending.is_none(), "stale transition remained active");
    assert_eq!(canvas.view.position_of(key), at);
}

#[test]
fn delayed_transition_refuses_another_canvas_over_the_same_graph() {
    let (graph, nodes) = graph((0, 1));
    let source = Canvas::with_graph(graph.clone());
    let mut other = Canvas::with_graph(graph);
    other.set_physics_paused(true);
    let mut pending = transition(&source, nodes[0]);
    let at = other.view.position_of(nodes[0]);
    advance_arrangement(&mut other, &mut pending, "grid.default", 0.);
    assert!(pending.is_none());
    assert_eq!(other.view.position_of(nodes[0]), at);
}

#[test]
fn ordinary_title_edit_keeps_transition_and_topology_cache_identity() {
    let (graph, nodes) = graph((0, 1));
    let mut canvas = Canvas::with_graph(graph);
    canvas.set_physics_paused(true);
    let key = nodes[0];
    let id = canvas.graph.get_node(key).unwrap().id;
    let action = canvas.described_keys.by_member[&id];
    let mut pending = transition(&canvas, key);
    let mut registry = crate::signals::ChannelRegistry::new();
    registry.degree_weights(&canvas.graph);
    let runs = registry.runs();
    let mut edited = canvas.graph.clone();
    edited.set_node_title(key, "Edited title".into());
    canvas.refresh_same_topology_graph(&edited).unwrap();
    registry.degree_weights(&canvas.graph);
    assert_eq!(registry.runs(), runs);
    assert_eq!(canvas.described_keys.by_member[&id], action);
    advance_arrangement(&mut canvas, &mut pending, "grid.default", 0.);
    assert!(pending.is_some());
    assert!(advance_arrangement(&mut canvas, &mut pending, "grid.default", 10000.).is_some());
}

#[test]
fn detached_stop_return_refuses_recreated_targets_before_resolving_slots() {
    let (graph, nodes) = graph((0, 1));
    let mut canvas = Canvas::with_graph(graph);
    let key = nodes[0];
    let id = canvas.graph.get_node(key).unwrap().id;
    let captured = StopReturn::new(
        &canvas,
        vec![(key, PortablePoint::zero())],
        vec![(key, PortablePoint::new(70., 50.))],
        None,
    );
    canvas.ingest_graph(|graph| {
        graph.remove_node(key);
        graph.add_node_with_id(id, "https://replacement.example".into(), Default::default());
        true
    });
    assert!(ArrangementTransition::returning(&canvas, &captured).is_err());
}

#[test]
fn offered_community_must_match_current_source_and_input_version() {
    let (first, nodes) = graph((0, 1));
    let mut second = first.clone();
    second.assert_semantic_predicate(nodes[0], nodes[2], "links".into());
    let mut registry = crate::signals::ChannelRegistry::new();
    let old = crate::signals::community_louvain(&first);
    assert!(!registry.offer_community(&second, first.structural_input_key(), old));
    assert!(registry.community_held().is_none());
    let current = crate::signals::community_louvain(&second);
    assert!(registry.offer_community(&second, second.structural_input_key(), current));
    let runs = registry.runs();
    assert!(
        registry.bind_graph(&Graph::new()),
        "a new empty graph has a different source"
    );
    assert!(registry.community_held().is_none());
    assert_eq!(
        registry.runs(),
        runs,
        "source replacement preserves counters"
    );
}

#[test]
fn memo_and_physics_keys_distinguish_equal_counter_candidates() {
    let (base, nodes) = graph((0, 1));
    let mut first = base.clone();
    let mut second = base;
    first.assert_semantic_predicate(nodes[0], nodes[2], "links".into());
    second.assert_semantic_predicate(nodes[1], nodes[2], "links".into());
    let mut canvas = Canvas::with_graph(first);
    let stamp = canvas.strategy_graph_stamp();
    let physics_key = canvas.physics_view_key();
    canvas.refresh_weighted_edges();
    let weights = canvas.weighted_edges_cache.clone();
    canvas.ingest_graph(|graph| {
        *graph = second;
        true
    });
    assert_ne!(canvas.strategy_graph_stamp(), stamp);
    assert_ne!(canvas.physics_view_key(), physics_key);
    canvas.refresh_weighted_edges();
    assert_ne!(canvas.weighted_edges_cache, weights);
}

#[test]
fn rule_face_result_cannot_reuse_epoch_after_same_uuid_recreation() {
    let (graph, nodes) = graph((0, 1));
    let mut canvas = Canvas::with_graph(graph);
    let epoch = canvas.graph_view_epoch();
    let key = nodes[0];
    let id = canvas.graph.get_node(key).unwrap().id;
    canvas.ingest_graph(|graph| {
        graph.remove_node(key);
        graph.add_node_with_id(id, "https://replacement.example".into(), Default::default());
        true
    });
    assert!(
        canvas
            .set_node_rule_faces(epoch, HashMap::from([(id, sceno::Representation::Glyph)]))
            .is_err()
    );
    assert!(canvas.node_rule_representation(id).is_none());
}
