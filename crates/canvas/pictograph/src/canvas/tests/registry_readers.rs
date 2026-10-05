// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! G2b's done-condition on the canvas (dynamics grammar plan, G2b): with
//! every registry fact's arrangement and its physics and view readers live,
//! the registry computes each once; a change of its key computes it once more,
//! and nothing else.

use kernel::graph::apply::{GraphDelta, apply_graph_delta};
use kernel::graph::fixtures::GraphFixtures;

use super::arrangement_goldens::topic_fixture;
use crate::canvas::channels::{Channel, OrderSource};
use crate::canvas::{Canvas, PhysicsChoice, PhysicsKindSource, PhysicsLaw, PhysicsOverlay};
use crate::signals::{BridgeMetric, ImportanceMetric, RegistryRuns};

const ARRANGEMENTS: [&str; 6] = [
    "kanban.default",
    "kanban.community",
    "timeline.default",
    "phyllotaxis.default",
    "spectral.default",
    "radial.default",
];

/// Every reader, three times over: each arrangement both ways, the channels
/// no canvas arrangement reads, and frames (the rings, the gloss, size by
/// importance and recency, the affinity force, the law).
fn read_everything(canvas: &mut Canvas) {
    for _ in 0..3 {
        for id in ARRANGEMENTS {
            for recent_first in [true, false] {
                canvas.project_arrangement_for_view(id, 800, 600, None, recent_first, 1.0, None);
            }
        }
        for channel in [
            Channel::Order(OrderSource::Recency),
            Channel::Order(OrderSource::Timeline),
            Channel::Rings,
            Channel::Coords,
            Channel::Weight,
            Channel::Importance(ImportanceMetric::Betweenness),
            Channel::Bridges(BridgeMetric::Articulation),
        ] {
            canvas.channel_values(channel);
        }
        for _ in 0..3 {
            canvas.frame(1024, 600);
        }
    }
}

#[test]
fn every_fact_runs_once_with_its_readers_live_and_once_more_when_its_key_moves() {
    let (graph, keys) = topic_fixture();
    let focus = graph.get_node(keys[0]).unwrap().id;
    let mut canvas = Canvas::with_graph(graph);
    assert_eq!(
        canvas.channel_runs(),
        RegistryRuns::default(),
        "nothing read yet"
    );
    assert!(canvas.select_member(focus), "a focus for Radial's rings");
    canvas
        .set_physics_choice(&PhysicsChoice {
            law: PhysicsLaw::Kinds,
            overlays: vec![PhysicsOverlay::DomainCluster],
            kind: PhysicsKindSource::Site,
            groups: PhysicsKindSource::Cluster,
            ..Default::default()
        })
        .expect("not refused");
    canvas.set_size_by_importance(true);
    canvas.set_size_by_recency(true);
    canvas.set_gloss_size_by_importance(true);
    canvas.set_show_community_rings(true);
    canvas.set_show_bridge_rings(true);
    canvas.set_cluster_by_affinity(true);

    read_everything(&mut canvas);
    let once = RegistryRuns {
        community: 1,
        // Betweenness brokers for the rings, articulation points read once.
        bridges: 2,
        affinity: 1,
        // Degree for size and the gloss, betweenness read once.
        importance: 2,
        recency: 1,
        enumeration: 1,
        spectral: 1,
        rings: 1,
        degree_weights: 1,
        sites: 1,
    };
    assert_eq!(
        canvas.channel_runs(),
        once,
        "each fact once, however often read"
    );

    // A visit moves recency's key and nothing else.
    let visited = canvas.graph().get_node(keys[5]).unwrap().id;
    canvas.ingest_graph(|g| {
        apply_graph_delta(
            g,
            GraphDelta::ReplayTouchNodeLastVisitedById {
                node_id: visited,
                timestamp_ms: 1_800_000_000_000,
            },
        );
        true
    });
    read_everything(&mut canvas);
    assert_eq!(
        canvas.channel_runs(),
        RegistryRuns { recency: 2, ..once },
        "a visit reruns recency alone"
    );

    // An edge moves structure: every fact keyed to it runs once more, with
    // its readers live again.
    canvas.ingest_graph(|g| {
        g.assert_semantic_predicate(keys[0], keys[31], "links".to_string());
        true
    });
    read_everything(&mut canvas);
    assert_eq!(
        canvas.channel_runs(),
        RegistryRuns {
            community: 2,
            bridges: 4,
            affinity: 2,
            importance: 4,
            recency: 3,
            enumeration: 2,
            spectral: 2,
            rings: 2,
            degree_weights: 2,
            sites: 2,
        },
        "structure reruns each fact once"
    );
}
