// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Snapshot round-trip basics — graph, edges, edge types, semantic
//! sub-kinds, favicon, thumbnail, UUID, duplicate-URL handling.

use super::super::*;

fn resource_payload(graph: &Graph, from: NodeKey, to: NodeKey) -> &EdgePayload {
    let key = graph
        .find_resource_edge_key(
            graph.shown_resource_id(from).unwrap(),
            graph.shown_resource_id(to).unwrap(),
        )
        .expect("resource edge survives");
    graph.get_resource_edge(key).unwrap()
}

fn hyperlink() -> EdgeAssertion {
    EdgeAssertion::Semantic {
        sub_kind: SemanticSubKind::Hyperlink,
        label: None,
    }
}

fn user_grouped(label: Option<&str>) -> EdgeAssertion {
    EdgeAssertion::Semantic {
        sub_kind: SemanticSubKind::UserGrouped,
        label: label.map(str::to_string),
    }
}

fn cites() -> EdgeAssertion {
    EdgeAssertion::Semantic {
        sub_kind: SemanticSubKind::Cites,
        label: None,
    }
}

#[test]
fn assert_semantic_predicate_creates_open_predicate_edge() {
    let mut graph = Graph::new();
    let a = graph.add_node("https://a.test/".to_string(), Point2D::new(0.0, 0.0));
    let b = graph.add_node("https://b.test/".to_string(), Point2D::new(0.0, 0.0));
    let key = graph
        .assert_semantic_predicate(a, b, "https://schema.org/citation".to_string())
        .expect("edge created");
    let payload = graph.get_relation(key).expect("payload");
    // Carries no sub-kinds, yet still reports the Semantic family.
    assert!(payload.has_relation(RelationSelector::Family(EdgeFamily::Semantic)));
    assert!(!payload.is_empty());
    assert!(
        payload
            .semantic_data()
            .is_some_and(|d| d.sub_kinds.is_empty())
    );
    assert_eq!(
        payload.semantic_data().and_then(|d| d.predicate.as_deref()),
        Some("https://schema.org/citation")
    );
}

#[test]
fn open_predicate_only_edge_survives_snapshot_roundtrip() {
    let mut graph = Graph::new();
    let a = graph.add_node("https://a.test/".to_string(), Point2D::new(0.0, 0.0));
    let b = graph.add_node("https://b.test/".to_string(), Point2D::new(0.0, 0.0));
    graph.assert_semantic_predicate(a, b, "https://schema.org/citation".to_string());

    let restored = Graph::from_snapshot(&graph.to_snapshot());

    assert_eq!(restored.edge_count(), 0);
    assert_eq!(restored.resource_relations().count(), 1);
    let (ra, _) = restored.get_node_by_url("https://a.test/").unwrap();
    let (rb, _) = restored.get_node_by_url("https://b.test/").unwrap();
    let payload = resource_payload(&restored, ra, rb);
    assert!(payload.has_relation(RelationSelector::Family(EdgeFamily::Semantic)));
    assert_eq!(
        payload.semantic_data().and_then(|d| d.predicate.as_deref()),
        Some("https://schema.org/citation")
    );
}

#[test]
fn test_snapshot_roundtrip() {
    let mut graph = Graph::new();
    let n1 = graph.add_node("https://a.com".to_string(), Point2D::new(10.0, 20.0));
    let n2 = graph.add_node("https://b.com".to_string(), Point2D::new(30.0, 40.0));
    let _ = graph.assert_relation(n1, n2, hyperlink());

    graph.get_node_mut(n1).unwrap().title = "Site A".to_string();
    graph.set_node_pinned(n2, true);

    let snapshot = graph.to_snapshot();
    let facets = graph.facets().clone();
    let mut restored = Graph::from_snapshot(&snapshot);
    restored.overlay_facets(facets).unwrap();

    assert_eq!(restored.node_count(), 2);
    assert_eq!(restored.edge_count(), 0);
    assert_eq!(restored.resource_relations().count(), 1);

    let (_, ra) = restored.get_node_by_url("https://a.com").unwrap();
    assert_eq!(ra.title, "Site A");
    // Positions are no longer graph truth (not a node field, not in the snapshot);
    // they live in the cartography sidecar. (Position gut.)

    let (rb, _) = restored.get_node_by_url("https://b.com").unwrap();
    assert_eq!(restored.node_is_pinned(rb), Some(true));
}

#[test]
fn test_snapshot_empty_graph() {
    let graph = Graph::new();
    let snapshot = graph.to_snapshot();
    let restored = Graph::from_snapshot(&snapshot);

    assert_eq!(restored.node_count(), 0);
    assert_eq!(restored.edge_count(), 0);
}

#[test]
fn test_snapshot_preserves_edge_types() {
    let mut graph = Graph::new();
    let n1 = graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
    let n2 = graph.add_node("https://b.com".to_string(), Point2D::new(100.0, 0.0));
    let n3 = graph.add_node("https://c.com".to_string(), Point2D::new(200.0, 0.0));
    let _ = graph.assert_relation(n1, n2, hyperlink());
    let _ = graph.append_traversal(n2, n1, NavigationTrigger::LinkClick, Some(1_000_000));
    let _ = graph.assert_relation(n1, n3, user_grouped(None));

    let snapshot = graph.to_snapshot();
    let restored = Graph::from_snapshot(&snapshot);

    assert_eq!(restored.edge_count(), 2);
    assert_eq!(restored.resource_relations().count(), 1);

    let has_hyperlink = resource_payload(&restored, n1, n2)
        .has_relation(RelationSelector::Semantic(SemanticSubKind::Hyperlink));
    let has_history = restored
        .find_edge_key(n2, n1)
        .and_then(|edge_key| restored.get_edge(edge_key))
        .is_some_and(|payload| {
            payload.has_relation(RelationSelector::Family(EdgeFamily::Traversal))
        });
    let has_user_grouped = restored
        .find_edge_key(n1, n3)
        .and_then(|edge_key| restored.get_edge(edge_key))
        .is_some_and(|payload| {
            payload.has_relation(RelationSelector::Semantic(SemanticSubKind::UserGrouped))
        });
    assert!(has_hyperlink);
    assert!(has_history);
    assert!(has_user_grouped);
}

#[test]
fn test_snapshot_preserves_user_grouped_edge_label() {
    let mut graph = Graph::new();
    let from = graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
    let to = graph.add_node("https://b.com".to_string(), Point2D::new(100.0, 0.0));
    graph
        .assert_relation(from, to, user_grouped(Some("tab-group")))
        .unwrap();

    let snapshot = graph.to_snapshot();
    let restored = Graph::from_snapshot(&snapshot);
    let edge_key = restored.find_edge_key(from, to).unwrap();
    let payload = restored.get_edge(edge_key).unwrap();
    assert_eq!(payload.label(), Some("tab-group"));
}

#[test]
fn test_snapshot_preserves_generic_semantic_relations() {
    let mut graph = Graph::new();
    let from = graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
    let to = graph.add_node("https://b.com".to_string(), Point2D::new(100.0, 0.0));
    graph
        .assert_relation(
            from,
            to,
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::CanonicalMirrorOf,
                label: Some("profile".to_string()),
            },
        )
        .expect("canonical mirror relation should be asserted");

    let snapshot = graph.to_snapshot();
    let restored = Graph::from_snapshot(&snapshot);
    let payload = resource_payload(&restored, from, to);
    assert!(payload.has_relation(RelationSelector::Semantic(
        SemanticSubKind::CanonicalMirrorOf,
    )));
    assert_eq!(payload.label(), Some("profile"));
}

#[test]
fn test_snapshot_preserves_favicon_data() {
    let mut graph = Graph::new();
    let key = graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
    let favicon = crate::types::ImageRef::new([255u8; 32], 1, 1);
    if let Some(node) = graph.get_node_mut(key) {
        node.set_image(crate::types::ImageRole::Favicon, favicon);
    }

    let snapshot = graph.to_snapshot();
    let restored = Graph::from_snapshot(&snapshot);
    let (_, restored_node) = restored.get_node_by_url("https://a.com").unwrap();
    assert_eq!(restored_node.favicon(), Some(&favicon));
}

#[test]
fn test_snapshot_preserves_thumbnail_data() {
    let mut graph = Graph::new();
    let key = graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
    let thumbnail = crate::types::ImageRef::new([137u8; 32], 64, 48);
    if let Some(node) = graph.get_node_mut(key) {
        node.set_image(crate::types::ImageRole::Preview, thumbnail);
    }

    let snapshot = graph.to_snapshot();
    let restored = Graph::from_snapshot(&snapshot);
    let (_, restored_node) = restored.get_node_by_url("https://a.com").unwrap();
    assert_eq!(restored_node.preview(), Some(&thumbnail));
}

#[test]
fn test_snapshot_preserves_uuid_identity() {
    let mut graph = Graph::new();
    let key = graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
    let node_id = graph.get_node(key).unwrap().id;

    let snapshot = graph.to_snapshot();
    let restored = Graph::from_snapshot(&snapshot);
    let (_, restored_node) = restored.get_node_by_id(node_id).unwrap();
    assert_eq!(restored_node.url(), "https://a.com");
}

// --- TEST-3: from_snapshot edge cases ---

#[test]
fn test_snapshot_edge_with_missing_url_is_dropped() {
    use crate::persistence::{GraphSnapshot, PersistedAddress, PersistedEdge, PersistedNode};

    let snapshot = GraphSnapshot {
        assertion_schema: 0,
        nodes: vec![PersistedNode {
            node_id: Uuid::new_v4().to_string(),
            url: "https://a.com".to_string(),
            cached_host: None,
            title: String::new(),
            body: None,
            tags: vec![],
            tag_presentation: NodeTagPresentationState::default(),
            import_provenance: vec![],
            is_pinned: false,
            images: Default::default(),
            legacy_thumbnail_png: None,
            legacy_thumbnail_width: 0,
            legacy_thumbnail_height: 0,
            legacy_favicon_rgba: None,
            legacy_favicon_width: 0,
            legacy_favicon_height: 0,
            session_state: None,
            mime_hint: None,
            address: PersistedAddress::Http("https://a.com".to_string()),
            classifications: Vec::new(),
            frame_layout_hints: Vec::new(),
            frame_split_offer_suppressed: false,
            properties: Vec::new(),
            derivations: Vec::new(),
            last_session_visited: 0,
            nested: None,
            content_hash: None,
        }],
        edges: vec![PersistedEdge {
            assertions: None,
            from_node_id: Uuid::new_v4().to_string(),
            to_node_id: Uuid::new_v4().to_string(),
            families: vec![PersistedEdgeFamily::Semantic],
            semantic: Some(PersistedSemanticEdgeData {
                sub_kinds: vec![PersistedSemanticSubKind::Hyperlink],
                label: None,
                agent_decay_progress: None,
                predicate: None,
                statements: vec![],
            }),
            traversal: None,
            containment: None,
            arrangement: None,
            imported: None,
            provenance: None,
        }],
        import_records: vec![],
        timestamp_secs: 0,
        fields: vec![],
        couplings: vec![],
        navigation: SharedNavigationMemory::empty(),
        resources: vec![],
        resource_edges: vec![],
        shown_resources: vec![],
    };

    let graph = Graph::from_snapshot(&snapshot);

    // Node should be restored, edge should be silently dropped
    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn test_snapshot_duplicate_urls_last_wins() {
    use crate::persistence::{GraphSnapshot, PersistedAddress, PersistedNode};

    let snapshot = GraphSnapshot {
        assertion_schema: 0,
        nodes: vec![
            PersistedNode {
                node_id: Uuid::new_v4().to_string(),
                url: "https://same.com".to_string(),
                cached_host: None,
                title: "First".to_string(),
                body: None,
                tags: vec![],
                tag_presentation: NodeTagPresentationState::default(),
                import_provenance: vec![],
                is_pinned: false,
                images: Default::default(),
                legacy_thumbnail_png: None,
                legacy_thumbnail_width: 0,
                legacy_thumbnail_height: 0,
                legacy_favicon_rgba: None,
                legacy_favicon_width: 0,
                legacy_favicon_height: 0,
                session_state: None,
                mime_hint: None,
                address: PersistedAddress::Http("https://same.com".to_string()),
                classifications: Vec::new(),
                frame_layout_hints: Vec::new(),
                frame_split_offer_suppressed: false,
                properties: Vec::new(),
                derivations: Vec::new(),
                last_session_visited: 0,
                nested: None,
                content_hash: None,
            },
            PersistedNode {
                node_id: Uuid::new_v4().to_string(),
                url: "https://same.com".to_string(),
                cached_host: None,
                title: "Second".to_string(),
                body: None,
                tags: vec![],
                tag_presentation: NodeTagPresentationState::default(),
                import_provenance: vec![],
                is_pinned: false,
                images: Default::default(),
                legacy_thumbnail_png: None,
                legacy_thumbnail_width: 0,
                legacy_thumbnail_height: 0,
                legacy_favicon_rgba: None,
                legacy_favicon_width: 0,
                legacy_favicon_height: 0,
                session_state: None,
                mime_hint: None,
                address: PersistedAddress::Http("https://same.com".to_string()),
                classifications: Vec::new(),
                frame_layout_hints: Vec::new(),
                frame_split_offer_suppressed: false,
                properties: Vec::new(),
                derivations: Vec::new(),
                last_session_visited: 0,
                nested: None,
                content_hash: None,
            },
        ],
        edges: vec![],
        import_records: vec![],
        timestamp_secs: 0,
        fields: vec![],
        couplings: vec![],
        navigation: SharedNavigationMemory::empty(),
        resources: vec![],
        resource_edges: vec![],
        shown_resources: vec![],
    };

    let graph = Graph::from_snapshot(&snapshot);

    // Both nodes are created and lookup keeps last inserted semantics.
    assert_eq!(graph.node_count(), 2);
    let (_, node) = graph.get_node_by_url("https://same.com").unwrap();
    assert_eq!(node.title, "Second");
}

// --- Field layer (field-system Phase 2) ---

#[test]
fn test_snapshot_without_field_layer_loads_empty() {
    // A pre-field-layer snapshot (JSON missing `fields`/`couplings`) must still
    // load, with an empty field layer — the additive `#[serde(default)]` migration.
    let json = r#"{"nodes":[],"edges":[],"import_records":[],"timestamp_secs":0}"#;
    let snapshot: crate::persistence::GraphSnapshot = serde_json::from_str(json).unwrap();
    assert!(snapshot.fields.is_empty());
    assert!(snapshot.couplings.is_empty());

    let restored = Graph::from_snapshot(&snapshot);
    assert_eq!(restored.fields().count(), 0);
    assert_eq!(restored.couplings().count(), 0);
}

#[test]
fn every_authored_containment_sub_kind_survives_snapshot_roundtrip() {
    // The writer persists all seven containment sub-kinds; the loader once
    // restored only UrlPath and Domain and dropped the rest, so an authored
    // CollectionMember edge round-tripped to nothing.
    //
    // UrlPath and Domain are the two DERIVED sub-kinds: the loader ends with
    // `rebuild_derived_containment_relations`, which retracts them graph-wide
    // and re-derives them child-to-parent from the URLs. They are therefore
    // not restored verbatim, and are covered separately below. The other five
    // are authored and exist only in the snapshot, so dropping any of them
    // loses user data.
    let authored = [
        ContainmentSubKind::FileSystem,
        ContainmentSubKind::UserFolder,
        ContainmentSubKind::ClipSource,
        ContainmentSubKind::NotebookSection,
        ContainmentSubKind::CollectionMember,
    ];

    let mut graph = Graph::new();
    let parent = graph.add_node("https://box.test/".to_string(), Point2D::new(0.0, 0.0));
    let child = graph.add_node("https://box.test/item".to_string(), Point2D::new(0.0, 0.0));
    for sub_kind in authored {
        let _ = graph.assert_relation(parent, child, EdgeAssertion::Containment { sub_kind });
    }

    let restored = Graph::from_snapshot(&graph.to_snapshot());

    let (rparent, _) = restored.get_node_by_url("https://box.test/").unwrap();
    let (rchild, _) = restored.get_node_by_url("https://box.test/item").unwrap();
    let key = restored
        .find_edge_key(rparent, rchild)
        .expect("the authored containment edge survives the round trip");
    let surface_sub_kinds = restored
        .get_edge(key)
        .expect("payload survives")
        .containment_data()
        .expect("the containment sidecar survives")
        .sub_kinds
        .clone();

    for sub_kind in authored {
        let restored_sub_kinds =
            match built_in_relation_stratum(RelationKind::Containment(sub_kind)) {
                GraphStratum::Surface => &surface_sub_kinds,
                GraphStratum::Resource => {
                    &resource_payload(&restored, rparent, rchild)
                        .containment_data()
                        .unwrap()
                        .sub_kinds
                },
            };
        assert!(
            restored_sub_kinds.contains(&sub_kind),
            "{sub_kind:?} was dropped by the snapshot loader"
        );
    }
}

#[test]
fn derived_containment_is_rebuilt_child_to_parent_on_load() {
    // The companion to the authored case: UrlPath is not carried across as
    // written but re-derived from the URLs, in the canonical child-to-parent
    // direction, by the rebuild at the end of the load.
    let mut graph = Graph::new();
    let _parent = graph.add_node("https://box.test/".to_string(), Point2D::new(0.0, 0.0));
    let _child = graph.add_node("https://box.test/item".to_string(), Point2D::new(0.0, 0.0));

    let restored = Graph::from_snapshot(&graph.to_snapshot());

    let (rparent, _) = restored.get_node_by_url("https://box.test/").unwrap();
    let (rchild, _) = restored.get_node_by_url("https://box.test/item").unwrap();
    let key = restored
        .find_edge_key(rchild, rparent)
        .expect("the derived containment edge runs child to parent");
    let sub_kinds = restored
        .get_edge(key)
        .expect("payload exists")
        .containment_data()
        .expect("the derived containment sidecar exists")
        .sub_kinds
        .clone();
    assert!(
        sub_kinds.contains(&ContainmentSubKind::UrlPath),
        "the URL-path parent relation is re-derived on load, got {sub_kinds:?}"
    );
}

/// A snapshot dated by its caller reads no clock, so one graph's snapshots
/// compare equal across a wall-clock second; the clock-dated snapshot, taken
/// the same way, is the control that differs (reservoir plan ruling 47).
#[test]
fn a_dated_snapshot_compares_equal_across_a_second_tick() {
    let mut graph = Graph::new();
    let a = graph.add_node("https://a.test/".to_string(), Point2D::new(0.0, 0.0));
    let b = graph.add_node("https://b.test/".to_string(), Point2D::new(0.0, 0.0));
    graph.assert_relation(a, b, hyperlink());
    let state = |snapshot: crate::persistence::GraphSnapshot| {
        serde_json::to_value((snapshot, graph.facets())).unwrap()
    };

    let dated_before = state(graph.to_snapshot_at(0));
    let clock_before = state(graph.to_snapshot());
    let second = clock_before[0]["timestamp_secs"].as_u64().unwrap();
    while graph.to_snapshot().timestamp_secs == second {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert_ne!(
        state(graph.to_snapshot()),
        clock_before,
        "control: the clock-dated snapshot moved with the tick"
    );
    assert_eq!(state(graph.to_snapshot_at(0)), dated_before);
}

#[path = "snapshot_basic/payloads.rs"]
mod payloads;
