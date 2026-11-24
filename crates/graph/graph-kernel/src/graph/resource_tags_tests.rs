// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! `resource_tag_labels` walks a resource's own edges; these pin it to the
//! full Resource-edge scan it replaced.
use super::resource::TAGGED_WITH_IRI;
use super::*;
use euclid::default::Point2D;
use std::collections::HashSet;
use uuid::Uuid;

/// The scan `resource_tag_labels` used before: every Resource edge, filtered
/// to the ones leaving `id`.
fn scanned(graph: &Graph, id: Uuid) -> HashSet<String> {
    graph
        .resource_edges()
        .filter(|(from, _, payload)| {
            from.id() == id
                && payload
                    .held_semantic_statements()
                    .any(|statement| statement.predicate == TAGGED_WITH_IRI)
        })
        .filter_map(|(_, to, _)| graph.resource_tag_concept(to.id()).map(|c| c.label))
        .collect()
}

/// The same scan without the `from` filter: what a walk that strayed past the
/// resource's own edges would return.
fn scanned_any_source(graph: &Graph) -> HashSet<String> {
    graph
        .resource_edges()
        .filter(|(_, _, payload)| {
            payload
                .held_semantic_statements()
                .any(|statement| statement.predicate == TAGGED_WITH_IRI)
        })
        .filter_map(|(_, to, _)| graph.resource_tag_concept(to.id()).map(|c| c.label))
        .collect()
}

#[test]
fn resource_tag_labels_walk_equals_the_full_scan() {
    let mut graph = Graph::new();
    let keys: Vec<NodeKey> = (0..5)
        .map(|i| {
            let key = graph.add_node(format!("https://tags.test/{i}"), Point2D::zero());
            graph.refresh_surface_resource(key);
            key
        })
        .collect();
    assert!(graph.insert_node_tags(keys[0], vec!["red".into(), "blue".into()]));
    assert!(graph.insert_node_tags(keys[1], vec!["blue".into()]));
    assert!(graph.insert_node_tags(keys[2], vec!["green".into(), "red".into(), "cat".into()]));
    // keys[3] and keys[4] carry no tags; non-tag relations run between all.
    for pair in keys.windows(2) {
        graph.assert_semantic_statement(
            pair[0],
            pair[1],
            SemanticStatementSpec {
                predicate: "https://tags.test/vocab#next".into(),
                ..Default::default()
            },
        );
    }

    let resources: Vec<Uuid> = graph.resource_nodes().map(|r| r.id()).collect();
    assert!(resources.len() > keys.len(), "concepts are resources too");
    for id in &resources {
        assert_eq!(graph.resource_tag_labels(*id), scanned(&graph, *id), "{id}");
    }
    assert!(graph.resource_tag_labels(Uuid::from_u128(7)).is_empty());

    let shown = |key| graph.shown_resource_id(key).expect("shown");
    let set = |labels: &[&str]| labels.iter().map(|l| l.to_string()).collect::<HashSet<_>>();
    assert_eq!(
        graph.resource_tag_labels(shown(keys[0])),
        set(&["red", "blue"])
    );
    assert_eq!(
        graph.resource_tag_labels(shown(keys[2])),
        set(&["green", "red", "cat"])
    );
    assert!(graph.resource_tag_labels(shown(keys[3])).is_empty());

    // Control: the comparison distinguishes resources, and a walk that read
    // other resources' tag edges would fail it.
    assert_ne!(
        graph.resource_tag_labels(shown(keys[0])),
        scanned(&graph, shown(keys[1]))
    );
    assert_ne!(
        scanned_any_source(&graph),
        graph.resource_tag_labels(shown(keys[0]))
    );
}
