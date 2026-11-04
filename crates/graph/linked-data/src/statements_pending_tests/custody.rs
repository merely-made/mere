// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn purge_prevents_arrival_derivation_and_source_navigation_keeps_original_ownership() {
    let mut graph = Graph::new();
    let source = kernel::graph::apply::add_node(
        &mut graph,
        None,
        "https://example.org/source".into(),
        Default::default(),
    );
    let original = graph.shown_resource_id(source).unwrap();
    apply_link_statements(
        &mut graph,
        source,
        &[statement("https://example.org/target", "cites")],
    );
    let mut purged = graph.clone();
    let before = truth(&purged);
    assert_eq!(purged.purge_pending_links(), 1);
    assert_eq!(truth(&purged), before);
    kernel::graph::apply::add_node(
        &mut purged,
        None,
        "https://example.org/target".into(),
        Default::default(),
    );
    assert_eq!(purged.resource_relations().count(), 0);

    kernel::graph::apply::apply_graph_delta(
        &mut graph,
        kernel::graph::apply::GraphDelta::NavigateNode {
            key: source,
            url: "https://example.org/later".into(),
        },
    );
    let current = graph.shown_resource_id(source).unwrap();
    assert_ne!(current, original);
    kernel::graph::apply::add_node(
        &mut graph,
        None,
        "https://example.org/target".into(),
        Default::default(),
    );
    let claims: Vec<_> = graph.resource_relations().collect();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].1, original);
    assert_ne!(claims[0].1, current);
    assert!(graph.pending_links().is_empty());
    let before_truth = truth(&graph);
    assert_eq!(graph.purge_pending_links(), 0);
    assert_eq!(truth(&graph), before_truth);
}
