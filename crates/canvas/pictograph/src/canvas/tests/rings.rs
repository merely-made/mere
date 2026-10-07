// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Revision-gated recompute for the overlay rings: the arrangement memo, the
//! community-ring partition, and the bridge-ring broker set.

use super::*;

fn set_address(canvas: &mut Canvas, key: NodeKey, url: &str) {
    canvas.ingest_graph(|graph| {
        kernel::graph::apply::apply_graph_delta(
            graph,
            kernel::graph::apply::GraphDelta::SetNodeUrl {
                key,
                new_url: url.into(),
            },
        );
        true
    });
}

#[test]
fn strategy_cache_tracks_visible_payload_and_resource_rebinding() {
    use kernel::graph::SemanticStatementSpec;
    let (graph, [a, alias_a, b, _, unbound]) = crate::canvas::build_tests::shown_content_graph();
    let mut canvas = Canvas::with_graph(graph);
    canvas.note_strategy_computed("kanban.default", 800, 600, None);
    let revision = canvas.graph().revision();
    let url_groups = canvas.graph().url_grouping_revision();
    let footprint_revision = canvas.strategy_footprint_revision;
    let extents = canvas.strategy_extents();
    set_address(
        &mut canvas,
        unbound,
        "https://unbound.surface.test/elsewhere",
    );
    assert_ne!(
        canvas.graph().revision(),
        revision,
        "new empty page still advances graph truth"
    );
    assert_eq!(canvas.graph().url_grouping_revision(), url_groups);
    assert_eq!(canvas.strategy_extents(), extents);
    assert_eq!(canvas.strategy_footprint_revision, footprint_revision);
    assert_eq!(canvas.graph().projected_relations().count(), 13);
    assert!(!canvas.needs_strategy_recompute("kanban.default", 800, 600, None));
    assert!(
        !canvas.needs_strategy_recompute("kanban.default", 800, 600, None),
        "steady frame reuses the stamp"
    );

    let claim = canvas
        .graph()
        .get_edge(canvas.graph().find_edge_key(a, b).unwrap())
        .unwrap()
        .semantic_statements()[0]
        .clone();
    canvas.ingest_graph(|graph| {
        graph
            .assert_semantic_statement(
                a,
                b,
                SemanticStatementSpec {
                    predicate: claim.predicate,
                    recognized_sub_kind: claim.recognized_sub_kind,
                    label: Some("edited metadata".into()),
                    graph_scope: claim.graph_scope,
                    provenance_iri: claim.provenance_iri,
                    asserted_at_ms: claim.asserted_at_ms,
                },
            )
            .unwrap()
            .1
            .changed
    });
    assert_eq!(
        canvas.graph().projected_relations().count(),
        13,
        "classifier rows did not change"
    );
    assert!(
        canvas.needs_strategy_recompute("kanban.default", 800, 600, None),
        "same-kind payload edit invalidates"
    );
    canvas.note_strategy_computed("kanban.default", 800, 600, None);
    assert!(!canvas.needs_strategy_recompute("kanban.default", 800, 600, None));

    set_address(
        &mut canvas,
        alias_a,
        "https://alias-a.surface.test/elsewhere",
    );
    assert_eq!(canvas.graph().url_grouping_revision(), url_groups);
    assert_eq!(
        canvas.graph().projected_relations().count(),
        7,
        "rebind removes six lifted rows"
    );
    assert_eq!(
        canvas.graph().resource_relations().count(),
        2,
        "original resource truth survives"
    );
    assert!(
        canvas.needs_strategy_recompute("kanban.default", 800, 600, None),
        "visible content topology changed"
    );
}

#[test]
fn restored_score_tracks_visible_inputs_and_same_revision_graph_swap_clears_cache() {
    let (graph, [a, alias_a, _, _, unbound]) = crate::canvas::build_tests::shown_content_graph();
    let mut canvas = Canvas::with_graph(graph);
    let mut extents = canvas.strategy_extents();
    extents.insert(a, (64.0, 64.0));
    let score =
        ::cartography::project_spiral_score(canvas.graph(), Some(&extents), None, false).score;
    let footprint_revision = canvas.strategy_footprint_revision;
    assert!(canvas.restore_projection_score(score.clone()));
    assert_eq!(canvas.node_size(a), 64.0);
    assert_ne!(canvas.strategy_footprint_revision, footprint_revision);
    assert!(!canvas.needs_strategy_recompute("phyllotaxis.default", 800, 600, None));
    let id = canvas.graph().get_node(a).unwrap().id;
    canvas.set_node_size(id, 80.0);
    assert!(
        canvas.needs_strategy_recompute("phyllotaxis.default", 800, 600, None),
        "a later real footprint change releases restored placement"
    );
    assert!(canvas.restore_projection_score(score));
    assert!(!canvas.needs_strategy_recompute("phyllotaxis.default", 800, 600, None));
    set_address(
        &mut canvas,
        unbound,
        "https://unbound.surface.test/elsewhere",
    );
    assert!(
        !canvas.needs_strategy_recompute("phyllotaxis.default", 800, 600, None),
        "empty page binding keeps restored placement"
    );
    set_address(
        &mut canvas,
        alias_a,
        "https://alias-a.surface.test/elsewhere",
    );
    assert!(
        canvas.needs_strategy_recompute("phyllotaxis.default", 800, 600, None),
        "visible content rebind releases restored placement"
    );

    let mut first = Graph::new();
    first.add_node("https://first.test/".into(), Default::default());
    let mut second = Graph::new();
    second.add_node("https://second.test/".into(), Default::default());
    assert_eq!(first.revision(), second.revision());
    let mut canvas = Canvas::with_graph(first);
    canvas.note_strategy_computed("grid.default", 800, 600, None);
    assert!(!canvas.needs_strategy_recompute("grid.default", 800, 600, None));
    canvas.set_graph(second);
    assert!(
        canvas.needs_strategy_recompute("grid.default", 800, 600, None),
        "equal revision is not graph identity"
    );
}

#[test]
fn strategy_cache_tracks_replayed_raw_predicate_edits_with_identical_classifier_rows() {
    use kernel::graph::apply::{GraphDelta, apply_graph_delta};
    const SOURCE: &str = "https://source.test/";
    const OLD: &str = "https://predicate.test/old";
    const NEW: &str = "https://predicate.test/new";
    let mut graph = Graph::new();
    let a = graph.add_node("https://a.test/".into(), Default::default());
    let b = graph.add_node("https://b.test/".into(), Default::default());
    graph
        .assert_semantic_statement(
            a,
            b,
            kernel::graph::SemanticStatementSpec {
                predicate: OLD.into(),
                provenance_iri: Some(SOURCE.into()),
                ..Default::default()
            },
        )
        .unwrap();
    let from_id = graph.get_node(a).unwrap().id;
    let to_id = graph.get_node(b).unwrap().id;
    let mut canvas = Canvas::with_graph(graph);
    let rows: Vec<_> = canvas
        .graph()
        .projected_relations()
        .map(|(_, row)| row)
        .collect();
    canvas.note_strategy_computed("grid.default", 800, 600, None);
    assert!(!canvas.needs_strategy_recompute("grid.default", 800, 600, None));
    let edit = || GraphDelta::ReplaySetEdgeSemanticPredicateByIds {
        from_id,
        to_id,
        predicate: Some(NEW.into()),
        asserter_iri: SOURCE.into(),
    };
    canvas.ingest_graph(|graph| {
        apply_graph_delta(graph, edit());
        true
    });
    assert_eq!(
        canvas
            .graph()
            .projected_relations()
            .map(|(_, row)| row)
            .collect::<Vec<_>>(),
        rows
    );
    let payload = canvas
        .graph()
        .get_edge(canvas.graph().find_edge_key(a, b).unwrap())
        .unwrap();
    assert_eq!(payload.semantic_statements()[0].predicate, NEW);
    assert!(
        canvas.needs_strategy_recompute("grid.default", 800, 600, None),
        "replay payload edit invalidates warm cache"
    );
    canvas.note_strategy_computed("grid.default", 800, 600, None);
    let revision = canvas.graph().revision();
    canvas.ingest_graph(|graph| {
        apply_graph_delta(graph, edit());
        true
    });
    assert_eq!(
        canvas.graph().revision(),
        revision,
        "identical replay edit is a no-op"
    );
    assert!(!canvas.needs_strategy_recompute("grid.default", 800, 600, None));
}

#[test]
fn strategy_cache_tracks_exact_assertion_insertions_and_retractions() {
    use kernel::graph::SemanticStatement;
    let (graph, [a, _, b, _, _]) = crate::canvas::build_tests::shown_content_graph();
    let mut canvas = Canvas::with_graph(graph);
    let exact = |id: &str| SemanticStatement {
        statement_id: id.into(),
        predicate: kernel::graph::predicate_iri(SemanticSubKind::UserGrouped).into(),
        recognized_sub_kind: Some(SemanticSubKind::UserGrouped),
        label: Some(id.into()),
        graph_scope: Default::default(),
        provenance_iri: Some(format!("https://source.test/{id}")),
        asserted_at_ms: Some(17),
    };
    canvas.note_strategy_computed("grid.default", 800, 600, None);
    for (id, count) in [("cache-alice", 14), ("cache-bob", 15)] {
        assert!(!canvas.needs_strategy_recompute("grid.default", 800, 600, None));
        canvas.ingest_graph(|graph| {
            graph
                .assert_persisted_semantic_statement(a, b, exact(id))
                .is_some()
        });
        assert_eq!(canvas.graph().projected_relations().count(), count);
        assert!(
            canvas.needs_strategy_recompute("grid.default", 800, 600, None),
            "a distinct carried assertion adds multiplicity"
        );
        canvas.note_strategy_computed("grid.default", 800, 600, None);
        let revision = canvas.graph().revision();
        canvas.ingest_graph(|graph| {
            graph
                .assert_persisted_semantic_statement(a, b, exact(id))
                .is_some()
        });
        assert_eq!(canvas.graph().revision(), revision);
        assert_eq!(canvas.graph().projected_relations().count(), count);
        assert!(
            !canvas.needs_strategy_recompute("grid.default", 800, 600, None),
            "exact duplicate keeps the cached multiplicity"
        );
    }
    canvas.ingest_graph(|graph| graph.retract_semantic_statement(a, b, "cache-alice"));
    assert_eq!(canvas.graph().projected_relations().count(), 14);
    assert!(canvas.needs_strategy_recompute("grid.default", 800, 600, None));
    let payload = canvas
        .graph()
        .get_edge(canvas.graph().find_edge_key(a, b).unwrap())
        .unwrap();
    assert!(
        payload
            .semantic_statements()
            .iter()
            .any(|statement| statement.statement_id == "cache-bob")
    );
    assert!(
        !payload
            .semantic_statements()
            .iter()
            .any(|statement| statement.statement_id == "cache-alice")
    );
    canvas.note_strategy_computed("grid.default", 800, 600, None);
    let revision = canvas.graph().revision();
    assert!(!canvas.ingest_graph(|graph| graph.retract_semantic_statement(a, b, "missing-handle")));
    assert_eq!(canvas.graph().revision(), revision);
    assert!(!canvas.needs_strategy_recompute("grid.default", 800, 600, None));
    canvas.ingest_graph(|graph| graph.retract_semantic_statement(a, b, "cache-bob"));
    assert_eq!(canvas.graph().projected_relations().count(), 13);
    assert!(canvas.needs_strategy_recompute("grid.default", 800, 600, None));
    assert_eq!(canvas.graph().resource_relations().count(), 2);
}

#[test]
fn arrangement_recompute_is_gated_on_its_inputs() {
    let mut graph = Graph::new();
    let a = graph.add_node(
        "https://a.example".to_string(),
        PortablePoint::new(0.0, 0.0),
    );
    let b = graph.add_node(
        "https://b.example".to_string(),
        PortablePoint::new(1.0, 0.0),
    );
    graph.assert_semantic_predicate(a, b, "links".to_string());
    let mut canvas = Canvas::with_graph(graph);
    let ak = canvas
        .graph()
        .get_node_by_url("https://a.example")
        .unwrap()
        .0;
    let bk = canvas
        .graph()
        .get_node_by_url("https://b.example")
        .unwrap()
        .0;

    // First time there is no recorded layout, so a recompute is needed; after noting it, the same
    // inputs skip (an analytic layout is computed once, not per frame).
    assert!(
        canvas.needs_strategy_recompute("grid.default", 800, 600, None),
        "first compute"
    );
    canvas.note_strategy_computed("grid.default", 800, 600, None);
    assert!(
        !canvas.needs_strategy_recompute("grid.default", 800, 600, None),
        "unchanged => skip"
    );

    // A viewport change re-triggers.
    assert!(
        canvas.needs_strategy_recompute("grid.default", 1024, 600, None),
        "viewport change"
    );

    // A structural change (the kernel revision moves) re-triggers.
    canvas.ingest_graph(|g| {
        g.add_node(
            "https://c.example".to_string(),
            PortablePoint::new(2.0, 0.0),
        );
        true
    });
    assert!(
        canvas.needs_strategy_recompute("grid.default", 800, 600, None),
        "revision moved"
    );

    // A non-focus strategy ignores the focus, so a selection change does not invalidate it.
    canvas.note_strategy_computed("grid.default", 800, 600, None);
    assert!(
        !canvas.needs_strategy_recompute("grid.default", 800, 600, Some(ak)),
        "grid ignores focus, so a selection change does not force a recompute"
    );

    // Host-grouped kanban shares the ordinary cache. A same-host navigation
    // keeps its column, while a cross-host move advances the graph's narrow
    // URL-grouping dependency without disturbing structural caches.
    canvas.note_strategy_computed("kanban.default", 800, 600, None);
    assert!(
        !canvas.needs_strategy_recompute("kanban.default", 800, 600, None),
        "unchanged kanban no longer recomputes every frame"
    );
    canvas.ingest_graph(|g| {
        kernel::graph::apply::apply_graph_delta(
            g,
            kernel::graph::apply::GraphDelta::SetNodeUrl {
                key: ak,
                new_url: "https://a.example/elsewhere".to_string(),
            },
        );
        true
    });
    assert!(
        !canvas.needs_strategy_recompute("kanban.default", 800, 600, None),
        "same-host navigation leaves kanban columns and the cache intact"
    );
    canvas.ingest_graph(|g| {
        kernel::graph::apply::apply_graph_delta(
            g,
            kernel::graph::apply::GraphDelta::SetNodeUrl {
                key: ak,
                new_url: "https://elsewhere.example/".to_string(),
            },
        );
        true
    });
    assert!(
        canvas.needs_strategy_recompute("kanban.default", 800, 600, None),
        "a host move invalidates the kanban grouping"
    );

    canvas.note_strategy_computed("grid.default", 800, 600, None);
    canvas.ingest_graph(|g| {
        kernel::graph::apply::apply_graph_delta(
            g,
            kernel::graph::apply::GraphDelta::SetNodeUrl {
                key: bk,
                new_url: "https://another-site.example/".to_string(),
            },
        );
        true
    });
    assert!(
        !canvas.needs_strategy_recompute("grid.default", 800, 600, None),
        "the URL-authority dependency is selected only for by-site kanban"
    );
    canvas.note_strategy_computed("grid.default", 800, 600, None);
    canvas.set_node_size(canvas.graph().get_node(ak).unwrap().id, 80.0);
    assert!(
        canvas.needs_strategy_recompute("grid.default", 800, 600, None),
        "resolved face footprint changes invalidate extent-aware layouts"
    );

    // Radial is focus-driven, so a focus change DOES re-trigger it.
    canvas.note_strategy_computed("radial.default", 800, 600, Some(ak));
    assert!(
        canvas.needs_strategy_recompute("radial.default", 800, 600, Some(bk)),
        "radial re-centers on a focus change"
    );
}

#[test]
fn community_rings_toggle_computes_the_partition_on_frame() {
    // Two triangles {0,1,2} and {3,4,5} joined by a bridge => two communities.
    let mut graph = Graph::new();
    let n: Vec<NodeKey> = (0..6)
        .map(|i| {
            graph.add_node(
                format!("https://{i}.example"),
                PortablePoint::new(i as f32, 0.0),
            )
        })
        .collect();
    for &(a, b) in &[(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)] {
        graph.assert_semantic_predicate(n[a], n[b], "links".to_string());
    }
    let mut canvas = Canvas::with_graph(graph);
    assert!(
        canvas.community().is_none(),
        "no partition until a consumer asks for it"
    );

    // Turning the rings on makes the frame compute the partition and run the ring paint path.
    canvas.set_show_community_rings(true);
    let _ = canvas.frame(800, 600);
    assert!(canvas.show_community_rings(), "the toggle is on");
    assert_eq!(
        canvas.community().map(|c| c.clusters.len()),
        Some(2),
        "the ring frame computed the two-community partition"
    );
}

#[test]
fn bridge_rings_toggle_computes_the_broker_on_frame() {
    // Bowtie: triangles {0,1,2} and {2,3,4} share the broker node 2.
    let mut graph = Graph::new();
    let n: Vec<NodeKey> = (0..5)
        .map(|i| {
            graph.add_node(
                format!("https://{i}.example"),
                PortablePoint::new(i as f32, 0.0),
            )
        })
        .collect();
    for &(a, b) in &[(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (4, 2)] {
        graph.assert_semantic_predicate(n[a], n[b], "links".to_string());
    }
    let mut canvas = Canvas::with_graph(graph);
    let k2 = canvas
        .graph()
        .get_node_by_url("https://2.example")
        .unwrap()
        .0;
    assert!(
        canvas.bridges().is_none(),
        "no bridges until the toggle asks for them"
    );

    canvas.set_show_bridge_rings(true);
    let _ = canvas.frame(800, 600);
    assert!(canvas.show_bridge_rings(), "the toggle is on");
    assert_eq!(
        canvas.bridges().unwrap().bridges,
        vec![k2],
        "the broker is the only bridge"
    );
}

#[test]
fn bridge_metric_switch_recomputes_under_the_new_metric() {
    // A 4-cycle distinguishes the metrics: every node is a (tied) betweenness broker, but none is a
    // cut vertex (the cycle is 2-connected). Switching the metric invalidates the cache and
    // recomputes, flipping the set. (Graph signals — articulation points.)
    let mut graph = Graph::new();
    let n: Vec<NodeKey> = (0..4)
        .map(|i| {
            graph.add_node(
                format!("https://{i}.example"),
                PortablePoint::new(i as f32, 0.0),
            )
        })
        .collect();
    for &(a, b) in &[(0, 1), (1, 2), (2, 3), (3, 0)] {
        graph.assert_semantic_predicate(n[a], n[b], "links".to_string());
    }
    let mut canvas = Canvas::with_graph(graph);
    canvas.set_show_bridge_rings(true);

    // Default metric (betweenness): every cycle node is a tied broker.
    let _ = canvas.frame(800, 600);
    assert_eq!(
        canvas.bridges().unwrap().bridges.len(),
        4,
        "all four cycle nodes are tied betweenness brokers"
    );

    // Switching to articulation invalidates the cache; the cycle has no cut vertex.
    canvas.set_bridge_metric(crate::signals::BridgeMetric::Articulation);
    let _ = canvas.frame(800, 600);
    assert!(
        canvas.bridges().unwrap().bridges.is_empty(),
        "a 2-connected cycle has no articulation point"
    );
    assert_eq!(
        canvas.bridge_metric(),
        crate::signals::BridgeMetric::Articulation
    );
}
