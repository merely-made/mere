// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A traversal event's minted id, attribution and browsing session
//! (engine-split D195, D198, D202) survive the snapshot codecs and journal
//! replay; an idle gap or a manual split opens a new browsing session.

use std::sync::{Arc, Mutex};

use super::super::*;
use crate::graph::apply::{GraphDelta, GraphDeltaResult, apply_graph_delta};
use crate::graph::capture::{CapturedDelta, replay_captured_deltas_onto_checked};
use crate::persistence::GraphSnapshot;

const ALICE: &str = "urn:test:alice";

fn traversals(graph: &Graph, from: NodeKey, to: NodeKey) -> Vec<Traversal> {
    graph
        .find_edge_key(from, to)
        .and_then(|edge| graph.get_edge(edge))
        .map(|payload| payload.traversals().to_vec())
        .unwrap_or_default()
}

fn two_clicks() -> (Graph, NodeKey, NodeKey) {
    let mut graph = Graph::new();
    let a = graph.add_node("https://a.test/".into(), Point2D::new(0.0, 0.0));
    let b = graph.add_node("https://b.test/".into(), Point2D::new(1.0, 0.0));
    graph.write_as(Author::person(ALICE), |graph| {
        assert!(graph.append_traversal(a, b, NavigationTrigger::LinkClick, Some(7)));
        assert!(graph.append_traversal(a, b, NavigationTrigger::LinkClick, Some(7)));
    });
    (graph, a, b)
}

#[test]
fn identical_clicks_get_distinct_ids_and_name_their_recorder() {
    let (graph, a, b) = two_clicks();
    let events = traversals(&graph, a, b);
    assert_eq!(events.len(), 2);
    assert_ne!(events[0].id, events[1].id);
    assert_eq!(
        (events[0].timestamp_ms, events[0].trigger),
        (events[1].timestamp_ms, events[1].trigger)
    );
    for event in &events {
        assert_eq!(event.recorded_by, Author::person(ALICE).asserter_iri());
    }
}

#[test]
fn snapshot_codecs_keep_traversal_ids_and_attribution() {
    let (graph, a, b) = two_clicks();
    let expected = traversals(&graph, a, b);
    let snapshot = graph.to_snapshot();

    let rkyv_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&snapshot).unwrap();
    let from_rkyv = rkyv::from_bytes::<GraphSnapshot, rkyv::rancor::Error>(&rkyv_bytes).unwrap();
    let from_json: GraphSnapshot =
        serde_json::from_str(&serde_json::to_string(&snapshot).unwrap()).unwrap();
    for decoded in [from_rkyv, from_json] {
        let restored = Graph::try_from_snapshot(&decoded).unwrap();
        assert_eq!(traversals(&restored, a, b), expected);
    }

    // Control: the comparison sees an id that changed.
    let mut altered = snapshot.clone();
    let record = altered
        .edges
        .iter_mut()
        .find_map(|edge| edge.traversal.as_mut())
        .expect("the snapshot holds the traversal edge");
    record.traversals[0].id = uuid::Uuid::from_u128(1);
    let restored = Graph::try_from_snapshot(&altered).unwrap();
    assert_ne!(traversals(&restored, a, b), expected);
}

#[test]
fn journal_replay_restores_the_minted_id_not_a_fresh_one() {
    let captured = Arc::new(Mutex::new(Vec::<CapturedDelta>::new()));
    let sink = captured.clone();
    let mut graph = Graph::new();
    graph.set_recorder(Some(Arc::new(move |delta: &CapturedDelta| {
        sink.lock().unwrap().push(delta.clone());
    })));
    let a = crate::graph::apply::add_node(
        &mut graph,
        None,
        "https://a.test/".into(),
        Point2D::new(0.0, 0.0),
    );
    let b = crate::graph::apply::add_node(
        &mut graph,
        None,
        "https://b.test/".into(),
        Point2D::new(1.0, 0.0),
    );
    graph.write_as(Author::person(ALICE), |graph| {
        let outcome = apply_graph_delta(
            graph,
            GraphDelta::AppendTraversal {
                from: a,
                to: b,
                trigger: NavigationTrigger::Back,
                timestamp_ms: None,
            },
        );
        assert!(matches!(outcome, GraphDeltaResult::TraversalAppended(true)));
    });
    let expected = traversals(&graph, a, b);
    assert_eq!(expected.len(), 1);

    let entries = captured.lock().unwrap().clone();
    let mut replayed = Graph::new();
    replay_captured_deltas_onto_checked(&mut replayed, entries).unwrap();
    let (a_id, b_id) = (graph.get_node(a).unwrap().id, graph.get_node(b).unwrap().id);
    let (ra, rb) = (
        replayed.get_node_key_by_id(a_id).unwrap(),
        replayed.get_node_key_by_id(b_id).unwrap(),
    );
    assert_eq!(traversals(&replayed, ra, rb), expected);
    assert_eq!(
        replayed.open_browsing_session(),
        Some(expected[0].browsing_session)
    );
}

fn sessions(graph: &Graph, from: NodeKey, to: NodeKey) -> Vec<uuid::Uuid> {
    traversals(graph, from, to)
        .iter()
        .map(|event| event.browsing_session)
        .collect()
}

#[test]
fn an_idle_gap_reaching_the_setting_opens_a_new_browsing_session() {
    let mut graph = Graph::new();
    assert_eq!(
        graph.browsing_idle_gap_ms(),
        crate::graph::browsing_session::DEFAULT_BROWSING_IDLE_GAP_MS
    );
    graph.set_browsing_idle_gap_ms(1_000);
    let a = graph.add_node("https://a.test/".into(), Point2D::new(0.0, 0.0));
    let b = graph.add_node("https://b.test/".into(), Point2D::new(1.0, 0.0));
    for at in [0, 999, 1_999, 2_500] {
        assert!(graph.append_traversal(a, b, NavigationTrigger::LinkClick, Some(at)));
    }
    let s = sessions(&graph, a, b);
    assert_eq!(s[0], s[1], "999 ms is inside a 1000 ms gap");
    assert_ne!(s[1], s[2], "1000 ms without a traversal ends the session");
    assert_eq!(s[2], s[3]);
    assert_eq!(graph.open_browsing_session(), Some(s[3]));
}

#[test]
fn a_manual_split_opens_a_new_session_for_the_next_traversal() {
    let mut graph = Graph::new();
    let a = graph.add_node("https://a.test/".into(), Point2D::new(0.0, 0.0));
    let b = graph.add_node("https://b.test/".into(), Point2D::new(1.0, 0.0));
    assert!(graph.append_traversal(a, b, NavigationTrigger::LinkClick, Some(10)));
    graph.split_browsing_session();
    assert_eq!(graph.open_browsing_session(), None);
    assert!(graph.append_traversal(a, b, NavigationTrigger::LinkClick, Some(11)));
    assert!(graph.append_traversal(a, b, NavigationTrigger::LinkClick, Some(12)));
    let s = sessions(&graph, a, b);
    assert_ne!(s[0], s[1]);
    assert_eq!(s[1], s[2]);
}

#[test]
fn a_reopened_graph_continues_its_latest_browsing_session() {
    let (graph, a, b) = two_clicks();
    let latest = sessions(&graph, a, b)[1];
    let mut restored = Graph::try_from_snapshot(&graph.to_snapshot()).unwrap();
    assert_eq!(restored.open_browsing_session(), Some(latest));
    assert!(restored.append_traversal(a, b, NavigationTrigger::Forward, Some(8)));
    assert_eq!(sessions(&restored, a, b)[2], latest);
    // Control: past the gap the reopened graph opens a new session.
    let late = 8 + restored.browsing_idle_gap_ms();
    assert!(restored.append_traversal(a, b, NavigationTrigger::Forward, Some(late)));
    assert_ne!(sessions(&restored, a, b)[3], latest);
}
