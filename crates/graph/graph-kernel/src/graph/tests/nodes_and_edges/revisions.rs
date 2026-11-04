// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn revision_advances_on_structural_change_only() {
    let mut graph = Graph::new();
    assert_eq!(graph.revision(), 0, "a fresh graph starts at revision 0");

    let a = graph.add_node("https://a.example".to_string(), Point2D::new(0.0, 0.0));
    let after_a = graph.revision();
    assert!(after_a > 0, "adding a node advances the revision");

    let b = graph.add_node("https://b.example".to_string(), Point2D::new(1.0, 0.0));
    assert!(
        graph.revision() > after_a,
        "a second node advances it again"
    );

    // A new edge (relation) is structural.
    let before_edge = graph.revision();
    graph.assert_relation(a, b, hyperlink());
    let after_edge = graph.revision();
    assert!(after_edge > before_edge, "a new edge advances the revision");

    // A content edit (the node url) is not structural: the revision holds, so a structural cache
    // is not needlessly invalidated by navigation/renames.
    graph.update_node_url(a, "https://a2.example".to_string());
    assert_eq!(
        graph.revision(),
        after_edge,
        "a url edit is content, not structure"
    );

    let url_groups = graph.url_grouping_revision();
    graph.update_node_url(a, "https://a2.example/another-page".to_string());
    assert_eq!(
        graph.url_grouping_revision(),
        url_groups,
        "a same-host URL edit does not move a host-grouped projection"
    );
    graph.update_node_url(a, "https://other.example/".to_string());
    assert!(
        graph.url_grouping_revision() > url_groups,
        "a host move advances the narrow URL-grouping dependency"
    );
    let url_groups = graph.url_grouping_revision();
    graph.update_node_url(a, "https://other.example:8443/".to_string());
    assert!(
        graph.url_grouping_revision() > url_groups,
        "a port is part of Canvas's site authority grouping"
    );

    // Removing a node is structural.
    graph.remove_node(b);
    assert!(
        graph.revision() > after_edge,
        "removing a node advances the revision"
    );
}

#[test]
fn content_revision_advances_on_node_text_and_membership_only() {
    let mut graph = Graph::new();
    assert_eq!(graph.content_revision(), 0);
    let a = graph.add_node("https://a.example".to_string(), Point2D::new(0.0, 0.0));
    let b = graph.add_node("https://b.example".to_string(), Point2D::new(1.0, 0.0));
    let added = graph.content_revision();
    assert!(added >= 2, "each added node is new text");

    // An edge is structure: the structural revision moves, content holds
    // (and the structural counter moving shows the edge registered).
    let structure = graph.revision();
    graph.assert_relation(a, b, hyperlink());
    assert!(graph.revision() > structure, "the edge is structure");
    assert_eq!(graph.content_revision(), added, "an edge is not content");

    // A title edit is content, and leaves structure alone.
    let structure = graph.revision();
    assert!(graph.set_node_title(a, "Saturn's rings".to_string()));
    let titled = graph.content_revision();
    assert!(titled > added, "a title edit is content");
    assert_eq!(graph.revision(), structure, "a title edit is not structure");
    // The same title again changes nothing.
    assert!(!graph.set_node_title(a, "Saturn's rings".to_string()));
    assert_eq!(
        graph.content_revision(),
        titled,
        "an unchanged title is not an edit"
    );

    // A primary URL move is content; the same URL again is not.
    graph.update_node_url(b, "https://b.example/page".to_string());
    let moved = graph.content_revision();
    assert!(moved > titled, "a URL move is content");
    graph.update_node_url(b, "https://b.example/page".to_string());
    assert_eq!(
        graph.content_revision(),
        moved,
        "the same URL is not a move"
    );

    // Tags are not the text the revision covers.
    assert!(graph.insert_node_tag(a, "astronomy".to_string()));
    assert_eq!(graph.content_revision(), moved, "a tag is not title or URL");

    // A node leaving takes its text with it.
    graph.remove_node(b);
    assert!(
        graph.content_revision() > moved,
        "removing a node is content"
    );
}

#[test]
fn visit_revision_advances_on_visit_history_only() {
    use crate::graph::apply::{GraphDelta, apply_graph_delta};
    let mut graph = Graph::new();
    let a = graph.add_node("https://a.example".to_string(), Point2D::new(0.0, 0.0));
    let b = graph.add_node("https://b.example".to_string(), Point2D::new(1.0, 0.0));
    let added = graph.visit_revision();
    assert!(added >= 2, "each added node's creation visit is a visit");

    // A visit moves it, and neither structure nor text.
    let (structure, content) = (graph.revision(), graph.content_revision());
    assert!(graph.set_node_last_visited_at_ms(a, 1_000));
    let visited = graph.visit_revision();
    assert!(visited > added, "a visit is a visit");
    assert_eq!(graph.revision(), structure, "a visit is not structure");
    assert_eq!(graph.content_revision(), content, "a visit is not text");
    // The same timestamp again changes nothing.
    assert!(!graph.set_node_last_visited_at_ms(a, 1_000));
    assert_eq!(graph.visit_revision(), visited, "an unchanged visit");

    // Structure, text and other facets leave it alone.
    graph.assert_relation(a, b, hyperlink());
    assert!(graph.set_node_title(a, "Saturn".to_string()));
    assert!(graph.insert_node_tag(a, "astronomy".to_string()));
    assert_eq!(graph.visit_revision(), visited, "not a visit");

    // The replay delta and a generic facet delta naming visit history both
    // move it; a generic delta naming another facet does not.
    let node_id = graph.get_node(b).unwrap().id;
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplayTouchNodeLastVisitedById {
            node_id,
            timestamp_ms: 2_000,
        },
    );
    let replayed = graph.visit_revision();
    assert!(replayed > visited, "the replay delta is a visit");
    apply_graph_delta(
        &mut graph,
        GraphDelta::SetNodeFacet {
            key: b,
            facet: "custom.note".to_string(),
            value: serde_json::json!({"text": "hello"}),
        },
    );
    assert_eq!(graph.visit_revision(), replayed, "another facet");
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplaySetNodeFacetById {
            node_id,
            facet: crate::graph::node_facets::VISIT_HISTORY.to_string(),
            value: serde_json::json!({"last_visited_ms": 3_000, "last_session_visited": 0}),
        },
    );
    let generic = graph.visit_revision();
    assert!(generic > replayed, "a generic write of visit history");
    assert_eq!(
        graph
            .node_last_visited(b)
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis()),
        Some(3_000),
        "the generic write is the one the reader sees"
    );

    // Mutable facet access may touch visits, so it counts as one.
    let _ = graph.facets_mut();
    assert!(graph.visit_revision() > generic, "raw facet access");
}
