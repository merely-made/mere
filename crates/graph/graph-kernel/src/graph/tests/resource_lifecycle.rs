// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::graph::apply::{GraphDelta, add_node, apply_graph_delta};
use crate::graph::capture::{CapturedDelta, replay_captured_deltas};
use crate::persistence::{PersistedResourceFacet, PersistedResourceRecord};
use std::sync::{Arc, Mutex};

fn record(graph: &mut Graph) -> Arc<Mutex<Vec<CapturedDelta>>> {
    let captured = Arc::new(Mutex::new(Vec::new()));
    let sink = captured.clone();
    graph.set_recorder(Some(Arc::new(move |delta| {
        sink.lock().unwrap().push(delta.clone());
    })));
    captured
}

fn add_live(graph: &mut Graph, id: u128, url: &str) -> NodeKey {
    add_node(
        graph,
        Some(Uuid::from_u128(id)),
        url.into(),
        Default::default(),
    )
}

fn navigate(graph: &mut Graph, key: NodeKey, url: &str) {
    apply_graph_delta(
        graph,
        GraphDelta::NavigateNode {
            key,
            url: url.into(),
        },
    );
}

fn held_claim(graph: &mut Graph, from: Uuid, to: Uuid) -> PersistedEdge {
    let mut payload = EdgePayload::new();
    payload.push_persisted_semantic_statement(SemanticStatement {
        statement_id: "held-before-navigation".into(),
        predicate: predicate_iri(SemanticSubKind::Cites).into(),
        recognized_sub_kind: Some(SemanticSubKind::Cites),
        label: Some("earlier page claim".into()),
        graph_scope: GraphScope::User,
        provenance_iri: Some("https://author.test/alice".into()),
        asserted_at_ms: Some(17),
    });
    let edge = snapshot::persisted_edge_for_ids(from, to, &payload);
    assert!(graph.set_resource_edges_between(from, to, &[edge.clone()]));
    edge
}

#[test]
fn ensure_binding_is_explicit_and_idempotent_with_missing_surface_control() {
    let mut graph = Graph::new();
    let captured = record(&mut graph);
    let missing = NodeIndex::new(999);
    let revision = graph.revision();
    assert_eq!(graph.ensure_surface_resource(missing), None);
    assert_eq!(graph.refresh_surface_resource(missing), None);
    assert_eq!(graph.revision(), revision);
    assert!(captured.lock().unwrap().is_empty());

    let surface = graph.add_node_with_id(
        Uuid::from_u128(1),
        "https://example.test/page#part".into(),
        Default::default(),
    );
    assert_eq!(
        graph.shown_resource_id(surface),
        None,
        "raw legacy add stays surface-only"
    );
    let resource = chartulary::resource_id("https://example.test/page");
    assert_eq!(graph.ensure_surface_resource(surface), Some(resource));
    assert!(graph.resource(resource).is_some());
    assert_eq!(graph.shown_resource_id(surface), Some(resource));
    assert_eq!(captured.lock().unwrap().len(), 2);
    let revision = graph.revision();
    assert_eq!(graph.ensure_surface_resource(surface), Some(resource));
    assert_eq!(graph.refresh_surface_resource(surface), Some(resource));
    assert_eq!(graph.revision(), revision);
    assert_eq!(
        captured.lock().unwrap().len(),
        2,
        "unchanged binding records nothing"
    );
}

#[test]
fn canonical_alias_surfaces_share_existing_resource_without_erasing_facets() {
    let mut graph = Graph::new();
    let canonical = "https://example.test/page?id=7";
    let resource = ResourceNode::new(canonical);
    let retained = PersistedResourceRecord {
        canonical_iri: canonical.into(),
        facets: vec![PersistedResourceFacet {
            facet: "test.retained".into(),
            value_json: "{\"source\":\"original\"}".into(),
        }],
    };
    assert!(graph.set_resource_record(resource.id(), Some(retained.clone())));
    let captured = record(&mut graph);
    let first = add_live(&mut graph, 1, canonical);
    let alias = add_live(
        &mut graph,
        2,
        "https://EXAMPLE.TEST:443/page?utm_source=news&id=7#part",
    );
    assert_ne!(
        graph.get_node(first).unwrap().id,
        graph.get_node(alias).unwrap().id
    );
    assert_eq!(graph.shown_resource_id(first), Some(resource.id()));
    assert_eq!(graph.shown_resource_id(alias), Some(resource.id()));
    assert_eq!(graph.resource_nodes().count(), 1);
    assert_eq!(graph.resource_record(resource.id()), Some(retained));
    let captured = captured.lock().unwrap();
    assert!(
        !captured
            .iter()
            .any(|delta| matches!(delta, CapturedDelta::ReplaySetResourceRecordById { .. }))
    );
    assert_eq!(
        captured
            .iter()
            .filter(|delta| matches!(delta, CapturedDelta::ReplaySetShownResourceById { .. }))
            .count(),
        2
    );
    for surface in [first, alias] {
        let id = graph.get_node(surface).unwrap().id.to_string();
        let added = captured
            .iter()
            .position(|delta| {
                matches!(delta,
            CapturedDelta::ReplayAddNodeWithIdIfMissing { id: added, .. } if added == &id)
            })
            .unwrap();
        let shown = captured
            .iter()
            .position(|delta| {
                matches!(delta,
            CapturedDelta::ReplaySetShownResourceById { surface_id, .. } if surface_id == &id)
            })
            .unwrap();
        assert!(added < shown);
    }
}

#[test]
fn explicit_refresh_preserves_old_claims_and_ensure_retains_recorded_binding() {
    let mut graph = Graph::new();
    let surface = add_live(&mut graph, 1, "https://page.test/one");
    let target = add_live(&mut graph, 2, "https://target.test/");
    let earlier = graph.shown_resource_id(surface).unwrap();
    let target = graph.shown_resource_id(target).unwrap();
    let held = held_claim(&mut graph, earlier, target);
    let captured = record(&mut graph);
    assert!(
        graph
            .update_node_url(surface, "https://page.test/two".into())
            .is_some()
    );
    assert_eq!(graph.ensure_surface_resource(surface), Some(earlier));
    assert!(
        captured.lock().unwrap().is_empty(),
        "raw URL replay preserves recorded binding"
    );
    let current = chartulary::resource_id("https://page.test/two");
    assert_eq!(graph.refresh_surface_resource(surface), Some(current));
    assert_ne!(current, earlier);
    assert_eq!(graph.shown_resource_id(surface), Some(current));
    assert_eq!(
        graph.persisted_resource_edges_between(earlier, target),
        vec![held]
    );
    assert!(
        graph
            .persisted_resource_edges_between(current, target)
            .is_empty()
    );
    assert_eq!(graph.resource_nodes().count(), 3);
    let captured = captured.lock().unwrap();
    assert!(
        matches!(&captured[0], CapturedDelta::ReplaySetResourceRecordById {
        resource_id, .. } if resource_id == &current.to_string())
    );
    assert!(
        matches!(&captured[1], CapturedDelta::ReplaySetShownResourceById {
        resource_id: Some(resource_id), .. } if resource_id == &current.to_string())
    );
}

#[test]
fn live_navigation_and_history_switch_binding_without_moving_held_assertions() {
    let mut graph = Graph::new();
    let surface = add_live(&mut graph, 1, "https://page.test/one");
    let target = add_live(&mut graph, 2, "https://target.test/");
    let earlier = graph.shown_resource_id(surface).unwrap();
    let target = graph.shown_resource_id(target).unwrap();
    let held = held_claim(&mut graph, earlier, target);
    navigate(&mut graph, surface, "https://page.test/one");
    navigate(&mut graph, surface, "https://page.test/two#part");
    let current = chartulary::resource_id("https://page.test/two");
    assert_eq!(graph.shown_resource_id(surface), Some(current));
    assert_eq!(apply::node_history_forward(&mut graph, surface), None);
    assert_eq!(
        apply::node_history_back(&mut graph, surface).as_deref(),
        Some("https://page.test/one")
    );
    assert_eq!(graph.shown_resource_id(surface), Some(earlier));
    assert_eq!(apply::node_history_back(&mut graph, surface), None);
    assert_eq!(
        apply::node_history_forward(&mut graph, surface).as_deref(),
        Some("https://page.test/two#part")
    );
    assert_eq!(graph.shown_resource_id(surface), Some(current));
    assert_eq!(
        graph.persisted_resource_edges_between(earlier, target),
        vec![held]
    );
    assert!(
        graph
            .persisted_resource_edges_between(current, target)
            .is_empty()
    );
}

#[test]
fn captured_live_lifecycle_replays_exact_resource_records_and_shown_associations() {
    let mut graph = Graph::new();
    let captured = record(&mut graph);
    let surface = add_live(&mut graph, 1, "https://page.test/one");
    let target_surface = add_live(&mut graph, 2, "https://target.test/");
    let earlier = graph.shown_resource_id(surface).unwrap();
    let target = graph.shown_resource_id(target_surface).unwrap();
    let held = held_claim(&mut graph, earlier, target);
    graph.record_delta(&CapturedDelta::ReplaySetResourceEdgesByIds {
        from_resource_id: earlier.to_string(),
        to_resource_id: target.to_string(),
        edges: vec![held.clone()],
    });
    navigate(&mut graph, surface, "https://page.test/one");
    navigate(&mut graph, surface, "https://page.test/two");
    assert!(apply::node_history_back(&mut graph, surface).is_some());
    assert!(apply::node_history_forward(&mut graph, surface).is_some());
    apply_graph_delta(
        &mut graph,
        GraphDelta::SetNodeUrl {
            key: surface,
            new_url: "https://page.test/three?utm_source=test#part".into(),
        },
    );
    let captured = captured.lock().unwrap();
    let created = captured
        .iter()
        .filter_map(|delta| match delta {
            CapturedDelta::ReplaySetResourceRecordById {
                resource_id,
                record: Some(_),
            } => Some(resource_id),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(created.len(), 4);
    for resource_id in created {
        let record = captured
            .iter()
            .position(|delta| {
                matches!(delta,
            CapturedDelta::ReplaySetResourceRecordById { resource_id: id, .. } if id == resource_id)
            })
            .unwrap();
        let shown = captured.iter().position(|delta| matches!(delta,
            CapturedDelta::ReplaySetShownResourceById { resource_id: Some(id), .. } if id == resource_id)).unwrap();
        assert!(record < shown);
        assert!(captured[..record].iter().any(|delta| match delta {
            CapturedDelta::ReplayAddNodeWithIdIfMissing { url, .. }
            | CapturedDelta::ReplayNavigateNodeById { url, .. }
            | CapturedDelta::ReplaySetNodeUrlById { new_url: url, .. } => {
                chartulary::resource_id(url).to_string() == *resource_id
            },
            _ => false,
        }));
    }
    let replayed = replay_captured_deltas(captured.iter().cloned());
    let original = graph.to_snapshot();
    let restored = replayed.to_snapshot();
    assert_eq!(restored.resources, original.resources);
    assert_eq!(restored.resource_edges, original.resource_edges);
    assert_eq!(restored.shown_resources, original.shown_resources);
    assert_eq!(
        replayed.persisted_resource_edges_between(earlier, target),
        vec![held]
    );
    let replayed_surface = replayed.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
    assert_eq!(
        replayed.node_current_url(replayed_surface),
        graph.node_current_url(surface)
    );
    assert_eq!(
        replayed.shown_resource_id(replayed_surface),
        Some(chartulary::resource_id("https://page.test/three"))
    );
}

#[test]
fn old_surface_only_captures_do_not_infer_resource_records_or_bindings() {
    let id = Uuid::from_u128(1).to_string();
    let legacy = vec![
        CapturedDelta::ReplayAddNodeWithIdIfMissing {
            id: id.clone(),
            url: "https://page.test/one".into(),
            position: [0.0, 0.0],
        },
        CapturedDelta::ReplayNavigateNodeById {
            node_id: id.clone(),
            url: "https://page.test/one".into(),
            transition: NodeHistoryTransitionKind::UrlTyped,
            timestamp_ms: 1,
            last_session_visited: 0,
        },
        CapturedDelta::ReplayNavigateNodeById {
            node_id: id.clone(),
            url: "https://page.test/two".into(),
            transition: NodeHistoryTransitionKind::UrlTyped,
            timestamp_ms: 2,
            last_session_visited: 0,
        },
        CapturedDelta::ReplayNodeHistoryBackById {
            node_id: id.clone(),
            timestamp_ms: 3,
        },
        CapturedDelta::ReplayNodeHistoryForwardById {
            node_id: id.clone(),
            timestamp_ms: 4,
        },
        CapturedDelta::ReplaySetNodeUrlById {
            node_id: id,
            new_url: "https://page.test/three".into(),
        },
    ];
    let graph = replay_captured_deltas(legacy);
    let surface = graph.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
    assert_eq!(
        graph
            .get_node(surface)
            .unwrap()
            .primary_address()
            .as_url_str(),
        "https://page.test/three"
    );
    assert_eq!(graph.shown_resource_id(surface), None);
    assert_eq!(graph.resource_nodes().count(), 0);

    let mut live = Graph::new();
    let surface = add_live(&mut live, 1, "https://page.test/one");
    navigate(&mut live, surface, "https://page.test/three");
    assert_eq!(
        live.shown_resource_id(surface),
        Some(chartulary::resource_id("https://page.test/three"))
    );
    assert_eq!(live.resource_nodes().count(), 2);
}
