// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Checked load admits one row per assertion handle.
use super::ResourceSnapshotError;
use crate::graph::Graph;
use crate::graph::assertion_handles_tests::claimed_pair;
use crate::persistence::GraphSnapshot;
use uuid::Uuid;

fn refused(snapshot: &GraphSnapshot) -> String {
    let loads = [
        Graph::try_from_snapshot(snapshot).err(),
        Graph::try_from_recorded_snapshot(snapshot).err(),
    ];
    for error in &loads {
        assert!(matches!(
            error,
            Some(ResourceSnapshotError::RepeatedAssertionHandle(_))
        ));
    }
    loads[0].as_ref().unwrap().to_string()
}

#[test]
fn checked_load_refuses_a_resource_row_copied_verbatim_and_keeps_the_original() {
    let (graph, ..) = claimed_pair();
    let snapshot = graph.to_snapshot();
    assert!(Graph::try_from_snapshot(&snapshot).is_ok());
    assert!(Graph::try_from_recorded_snapshot(&snapshot).is_ok());
    let mut copied = snapshot;
    copied.resource_edges.push(copied.resource_edges[0].clone());
    assert!(refused(&copied).contains("one handle means one row"));
}

#[test]
fn schema_zero_surface_rows_may_repeat_a_handle_but_a_resource_row_may_not_reuse_it() {
    let (graph, a, b) = claimed_pair();
    let mut snapshot = graph.to_snapshot();
    let resource_row = snapshot.resource_edges.remove(0);
    let mut surface_row = resource_row.clone();
    surface_row.from_node_id = Uuid::from_u128(1).to_string();
    surface_row.to_node_id = Uuid::from_u128(2).to_string();
    snapshot.edges = vec![surface_row.clone(), surface_row];
    snapshot.assertion_schema = 0;
    assert!(Graph::try_from_snapshot(&snapshot).is_ok());
    snapshot.assertion_schema = 1;
    refused(&snapshot);

    snapshot.assertion_schema = 0;
    snapshot.resource_edges = vec![resource_row];
    assert_eq!(snapshot.resource_edges[0].from_node_id, a.to_string());
    assert_eq!(snapshot.resource_edges[0].to_node_id, b.to_string());
    refused(&snapshot);
}
