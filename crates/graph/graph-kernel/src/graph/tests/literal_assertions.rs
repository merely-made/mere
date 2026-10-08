// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::super::*;
use crate::graph::apply::{GraphDelta, GraphDeltaResult, apply_graph_delta};
use crate::graph::capture::{CapturedDelta, replay_captured_deltas_onto};
use crate::types::NodeProperty;
use std::sync::{Arc, Mutex};

fn property(id: &str, asserter: &str, time: u64) -> NodeProperty {
    let mut property = NodeProperty::new("urn:literal:score".into(), "42".into())
        .with_graph_scope(GraphScope::Source)
        .with_metadata(Some(asserter.into()), Some(time));
    property.statement_id = id.into();
    property.datatype = Some("http://www.w3.org/2001/XMLSchema#integer".into());
    property
}

fn assert_literal_writers_keep_asserters_and_first_ids(batched: bool) {
    let mut graph = Graph::new();
    let key = graph.add_node("https://literal.test/".into(), Default::default());
    let baseline = graph.clone();
    let captures = Arc::new(Mutex::new(Vec::new()));
    let sink = captures.clone();
    graph.set_recorder(Some(Arc::new(move |delta| {
        sink.lock().unwrap().push(delta.clone());
    })));
    let write = |graph: &mut Graph, property: NodeProperty| {
        if batched {
            graph.append_node_properties(key, vec![property])
        } else {
            matches!(
                apply_graph_delta(graph, GraphDelta::AppendNodeProperty { key, property }),
                GraphDeltaResult::NodeMetadataUpdated(true)
            )
        }
    };
    let alice = property("alice-first", "urn:people:alice", 10);
    let bob = property("bob-first", "urn:people:bob", 20);
    assert!(write(&mut graph, alice.clone()));
    assert!(
        !write(&mut graph, alice.clone()),
        "identical reingest is a no-op"
    );
    assert!(write(&mut graph, bob.clone()));
    assert_eq!(
        graph.node_properties(key).unwrap(),
        vec![alice.clone(), bob.clone()]
    );
    assert!(write(
        &mut graph,
        property("incoming-id", "urn:people:alice", 30)
    ));
    let mut updated = alice;
    updated.asserted_at_ms = Some(30);
    let expected = vec![updated.clone(), bob];
    assert_eq!(graph.node_properties(key).unwrap(), expected);
    assert_eq!(captures.lock().unwrap().len(), 3);
    match captures.lock().unwrap().last().unwrap() {
        CapturedDelta::ReplayAppendNodePropertyById { property, .. } => {
            assert_eq!(property, &updated, "capture names the stored first handle");
        },
        other => panic!("unexpected literal capture: {other:?}"),
    }
    let mut replayed = baseline;
    replay_captured_deltas_onto(&mut replayed, captures.lock().unwrap().iter().cloned());
    assert_eq!(replayed.node_properties(key).unwrap(), expected);

    let mut other_scope = updated.clone();
    other_scope.statement_id = "alice-user-scope".into();
    other_scope.graph_scope = GraphScope::User;
    assert!(write(&mut graph, other_scope));
    let mut other_literal = updated;
    other_literal.statement_id = "alice-other-value".into();
    other_literal.value = "43".into();
    assert!(write(&mut graph, other_literal));
    assert_eq!(graph.node_properties(key).unwrap().len(), 4);
}

#[test]
fn single_literal_writer_keeps_asserters_and_replays_first_handles() {
    assert_literal_writers_keep_asserters_and_first_ids(false);
}

#[test]
fn batched_literal_writer_keeps_asserters_and_replays_first_handles() {
    assert_literal_writers_keep_asserters_and_first_ids(true);
}
