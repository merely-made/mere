// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

fn fixture(asserter: Option<&str>) -> (Graph, NodeKey, NodeKey, EdgeKey) {
    let mut graph = Graph::new();
    let from = graph.add_node("https://source.test/".into(), Point2D::zero());
    let to = graph.add_node("https://target.test/".into(), Point2D::zero());
    let mut payload = EdgePayload::new();
    payload.push_persisted_semantic_statement(SemanticStatement {
        statement_id: "carried legacy handle".into(),
        predicate: "https://predicate.test/before".into(),
        recognized_sub_kind: None,
        label: Some("held label".into()),
        graph_scope: GraphScope::User,
        provenance_iri: asserter.map(str::to_owned),
        asserted_at_ms: Some(17),
        first_asserted_ms: Some(17),
        withdrawn: None,
        basis: Default::default(),
    });
    let edge = graph.inner.connect(from, to, payload);
    (graph, from, to, edge)
}

#[test]
fn legacy_predicate_replay_advances_revision_only_for_changed_payload() {
    use super::super::apply::{GraphDelta, GraphDeltaResult, apply_graph_delta};

    let (mut graph, from, to, edge) = fixture(Some("https://author.test/"));
    let original = graph.get_edge(edge).unwrap().semantic_statements()[0].clone();
    let from_id = graph.get_node(from).unwrap().id;
    let to_id = graph.get_node(to).unwrap().id;
    let delta = GraphDelta::ReplaySetEdgeSemanticPredicateByIds {
        from_id,
        to_id,
        predicate: Some("https://predicate.test/after".into()),
        asserter_iri: "https://other-author.test/".into(),
    };
    let before_revision = graph.revision();
    assert!(matches!(
        apply_graph_delta(&mut graph, delta.clone()),
        GraphDeltaResult::NodeMetadataUpdated(true)
    ));
    assert_eq!(graph.revision(), before_revision + 1);
    let mut expected = original;
    expected.predicate = "https://predicate.test/after".into();
    assert_eq!(
        graph.get_edge(edge).unwrap().semantic_statements(),
        &[expected]
    );

    let edited = graph.get_edge(edge).unwrap().clone();
    let edited_revision = graph.revision();
    assert!(matches!(
        apply_graph_delta(&mut graph, delta),
        GraphDeltaResult::NodeMetadataUpdated(true)
    ));
    assert_eq!(graph.get_edge(edge), Some(&edited));
    assert_eq!(
        graph.revision(),
        edited_revision,
        "same input remains a no-op"
    );

    assert!(graph.set_edge_semantic_predicate_as(edge, None, "unused".into()));
    let payload = graph.get_edge(edge).unwrap();
    assert!(payload.held_assertions().next().is_none());
    assert_eq!(payload.semantic_statements().len(), 1);
    assert!(payload.semantic_statements()[0].withdrawn.is_some());
    assert_eq!(graph.revision(), edited_revision + 1);
    let cleared_revision = graph.revision();
    assert!(graph.set_edge_semantic_predicate_as(edge, None, "unused".into()));
    assert_eq!(graph.revision(), cleared_revision);
    assert!(!graph.set_edge_semantic_predicate_as(EdgeKey::new(1_000), None, "unused".into()));
    assert_eq!(graph.revision(), cleared_revision);
}

#[test]
fn legacy_predicate_attribution_repair_alone_advances_revision() {
    let (mut graph, _, _, edge) = fixture(None);
    let original = graph.get_edge(edge).unwrap().semantic_statements()[0].clone();
    let before_revision = graph.revision();
    assert!(graph.set_edge_semantic_predicate_as(
        edge,
        Some(original.predicate.clone()),
        "https://author.test/".into()
    ));
    let mut expected = original;
    expected.provenance_iri = Some("https://author.test/".into());
    assert_eq!(
        graph.get_edge(edge).unwrap().semantic_statements(),
        &[expected]
    );
    assert_eq!(graph.revision(), before_revision + 1);
    let repaired = graph.get_edge(edge).unwrap().clone();
    let repaired_revision = graph.revision();
    assert!(graph.set_edge_semantic_predicate_as(
        edge,
        Some("https://predicate.test/before".into()),
        "https://other-author.test/".into()
    ));
    assert_eq!(graph.get_edge(edge), Some(&repaired));
    assert_eq!(
        graph.revision(),
        repaired_revision,
        "existing attribution stays exact"
    );
}
