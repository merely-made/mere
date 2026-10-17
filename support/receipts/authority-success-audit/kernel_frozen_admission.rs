// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
// Diagnostic assertions describe divergent admission policies, not desired behavior.
use kernel::graph::{FrozenLinkSource, Graph, GraphStratum, SemanticStatementSpec};
use kernel::graph::apply::{add_node, apply_graph_delta, GraphDelta};
use uuid::Uuid;

#[test]
fn a_valid_parallel_assertion_copy_freezes_as_a_selection_but_not_as_a_link() {
    let mut graph = Graph::new();
    let a = add_node(&mut graph, Some(Uuid::from_u128(0xad101)), "https://audit.test/a".into(), Default::default());
    let b = add_node(&mut graph, Some(Uuid::from_u128(0xad102)), "https://audit.test/b".into(), Default::default());
    graph.try_assert_semantic_statement(a, b, SemanticStatementSpec {
        predicate: "urn:audit:claim".into(),
        provenance_iri: Some("urn:audit:source".into()),
        asserted_at_ms: Some(1),
        ..Default::default()
    }).unwrap();
    let from = graph.shown_resource_id(a).unwrap();
    let to = graph.shown_resource_id(b).unwrap();
    let edge = graph.to_snapshot().resource_edges[0].clone();
    let source = || FrozenLinkSource::Unavailable { reason: "isolated diagnostic source".into() };
    // The same operation with one bucket is a positive control.
    graph.clone().freeze_link_with_nonce(GraphStratum::Resource, from, to, source(), Uuid::from_u128(0xad103)).unwrap();
    apply_graph_delta(&mut graph, GraphDelta::ReplaySetResourceEdgesByIds {
        from_resource_id: from, to_resource_id: to, edges: vec![edge.clone(), edge]
    });
    assert_eq!(graph.resource_edges().count(), 2);
    let snapshot = graph.to_snapshot();
    Graph::try_from_snapshot(&snapshot).expect("current snapshot admission allows exact duplicate copies on one pair");
    let selected = graph.freeze_resource_selection_with_nonce(serde_json::json!({}), vec![from, to], Uuid::from_u128(0xad104)).unwrap();
    assert_eq!(graph.open_frozen_selection(selected).unwrap().selection.statements().len(), 2);
    let before = graph.to_snapshot();
    let error = graph.freeze_link_with_nonce(GraphStratum::Resource, from, to, source(), Uuid::from_u128(0xad105)).unwrap_err();
    assert_eq!(error, "duplicate frozen assertion handle");
    assert_eq!(graph.to_snapshot().resources, before.resources, "link refusal leaves Resources unchanged");
}
