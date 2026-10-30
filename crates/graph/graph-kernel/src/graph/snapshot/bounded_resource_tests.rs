// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::tests::{complete_payload, resource_snapshot_fixture};
use crate::graph::resource::ResourceEdgesBoundError;
use crate::graph::resource_pair_bounds::BorrowedEdge;
use crate::graph::{EdgePayload, Graph};
use crate::persistence::PersistedEdge;
use crate::types::AssertionWithdrawal;
use uuid::Uuid;

fn fixture() -> (Graph, Uuid, Uuid, Vec<EdgePayload>) {
    let (snapshot, _, from, other) = resource_snapshot_fixture();
    let mut graph = Graph::try_from_snapshot(&snapshot).unwrap();
    let mut payload = complete_payload();
    for statement in &mut payload.assertions.statements {
        statement.statement_id = format!("live-{}", statement.statement_id);
        statement.normalize_legacy_asserter();
    }
    payload.assertions.statements[0].withdrawn = Some(AssertionWithdrawal {
        at_ms: Some(99),
        by: Some("https://withdrawer.test/".into()),
    });
    let mut second = EdgePayload::new();
    second.push_persisted_semantic_statement(payload.assertions.statements[1].clone());
    second.assertions.statements[0].statement_id = "different-handle".into();
    let key = graph.resources.key_of(&from).unwrap();
    graph.resources.connect(key, key, payload.clone());
    graph.resources.connect(key, key, second.clone());
    let raw = graph
        .resources
        .inner()
        .edges_connecting(key, key)
        .map(|edge| edge.weight().clone())
        .collect();
    (graph, from, other, raw)
}

#[test]
fn live_capture_survives_pair_replacement_with_session_assertions() {
    let (mut graph, from, _, raw) = fixture();
    let mut captured = graph
        .persisted_resource_edges_between_bounded(from, from, 2, usize::MAX)
        .unwrap();
    assert_eq!(
        captured
            .iter()
            .map(|edge| edge.retained_assertions().len())
            .sum::<usize>(),
        raw.iter()
            .map(|payload| payload.assertions().len())
            .sum::<usize>()
    );
    let mut added = captured[0].clone();
    added.assertions.as_mut().unwrap().truncate(1);
    added.retained_assertions_mut()[0].statement_id = "fresh-recovery-handle".into();
    captured.push(added);
    assert!(graph.set_resource_edges_between(from, from, &captured));
    assert_eq!(
        graph.persisted_resource_edges_between_bounded(from, from, 3, usize::MAX),
        Ok(captured)
    );
}

fn live_edges(from: Uuid, raw: &[EdgePayload]) -> Vec<PersistedEdge> {
    raw.iter()
        .map(|payload| {
            let mut edge = super::persisted_edge_for_ids(from, from, payload);
            edge.assertions = Some(
                payload
                    .assertions()
                    .iter()
                    .map(super::to::persisted_assertion)
                    .collect(),
            );
            edge
        })
        .collect()
}

#[test]
fn bounded_pair_preserves_all_live_assertions_and_accepts_exact_limits() {
    let (graph, from, _, raw) = fixture();
    let durable = graph
        .to_snapshot()
        .resource_edges
        .into_iter()
        .filter(|edge| edge.from_node_id == from.to_string() && edge.to_node_id == from.to_string())
        .collect::<Vec<_>>();
    let expected = live_edges(from, &raw);
    assert!(
        expected
            .iter()
            .map(|edge| edge.retained_assertions().len())
            .sum::<usize>()
            > durable
                .iter()
                .map(|edge| edge.retained_assertions().len())
                .sum::<usize>()
    );
    let bytes = serde_json::to_vec(&raw)
        .unwrap()
        .len()
        .max(serde_json::to_vec(&expected).unwrap().len());
    assert_eq!(
        graph.persisted_resource_edges_between_bounded(from, from, expected.len(), bytes),
        Ok(expected)
    );
    assert_eq!(
        graph.persisted_resource_edges_between_bounded(from, from, 1, bytes),
        Err(ResourceEdgesBoundError::EdgeLimitExceeded)
    );
    assert_eq!(
        graph.persisted_resource_edges_between_bounded(from, from, 2, bytes - 1),
        Err(ResourceEdgesBoundError::ByteLimitExceeded)
    );
}

#[test]
fn borrowed_preflight_serializes_exactly_like_live_conversion() {
    let (_, from, _, raw) = fixture();
    for payload in raw {
        let borrowed = BorrowedEdge::new(from, from, &payload);
        let converted = live_edges(from, std::slice::from_ref(&payload))
            .pop()
            .unwrap();
        assert_eq!(
            serde_json::to_vec(&borrowed).unwrap(),
            serde_json::to_vec(&converted).unwrap()
        );
    }
}

#[test]
fn persisted_bytes_can_be_larger_than_raw_bytes() {
    let (mut graph, _, other, _) = fixture();
    let key = graph.resources.key_of(&other).unwrap();
    let payload = EdgePayload::new();
    let raw_bytes = serde_json::to_vec(&[&payload]).unwrap().len();
    let expected = vec![super::persisted_edge_for_ids(other, other, &payload)];
    let persisted_bytes = serde_json::to_vec(&expected).unwrap().len();
    assert!(persisted_bytes > raw_bytes);
    graph.resources.connect(key, key, payload);
    assert_eq!(
        graph.persisted_resource_edges_between_bounded(other, other, 1, persisted_bytes),
        Ok(expected)
    );
    assert_eq!(
        graph.persisted_resource_edges_between_bounded(other, other, 1, persisted_bytes - 1),
        Err(ResourceEdgesBoundError::ByteLimitExceeded)
    );
}

#[test]
fn bounded_pair_refuses_missing_endpoints_and_ignores_other_large_bucket() {
    let (mut graph, from, other, raw) = fixture();
    let key = graph.resources.key_of(&other).unwrap();
    let mut huge = EdgePayload::new();
    huge.push_persisted_semantic_statement(raw[0].assertions.statements[0].clone());
    huge.assertions.statements[0].label = Some("x".repeat(1_000_000));
    let source = graph.resources.key_of(&from).unwrap();
    graph.resources.connect(source, key, huge);
    let expected = live_edges(from, &raw);
    let bytes = serde_json::to_vec(&raw)
        .unwrap()
        .len()
        .max(serde_json::to_vec(&expected).unwrap().len());
    assert_eq!(
        graph.persisted_resource_edges_between_bounded(from, from, 2, bytes),
        Ok(expected)
    );
    assert_eq!(
        graph.persisted_resource_edges_between_bounded(from, other, 0, 1),
        Err(ResourceEdgesBoundError::EdgeLimitExceeded)
    );
    assert_eq!(
        graph.persisted_resource_edges_between_bounded(from, Uuid::nil(), 0, 0),
        Err(ResourceEdgesBoundError::MissingEndpoint)
    );
    assert_eq!(
        graph.persisted_resource_edges_between_bounded(other, from, 0, 2),
        Ok(vec![])
    );
    assert_eq!(
        graph.persisted_resource_edges_between_bounded(other, from, 0, 1),
        Err(ResourceEdgesBoundError::ByteLimitExceeded)
    );
}
