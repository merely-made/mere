// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Snapshot round-trips for import provenance, classifications,
//! imported / provenance edge sub-kinds, import-records replacement
//! + suppression state.

use super::super::*;

#[test]
fn test_snapshot_roundtrip_preserves_import_provenance() {
    let mut graph = Graph::new();
    let key = graph.add_node("https://example.com".to_string(), Point2D::new(0.0, 0.0));
    assert!(graph.set_node_import_provenance(
        key,
        vec![NodeImportProvenance {
            source_id: "import:firefox-bookmarks".to_string(),
            source_label: "Firefox bookmarks".to_string(),
        }],
    ));

    let facets = graph.facets().clone();
    let mut restored = Graph::from_snapshot(&graph.to_snapshot());
    restored.overlay_facets(facets).unwrap();
    let (key, _) = restored.get_node_by_url("https://example.com").unwrap();
    assert_eq!(
        restored.node_import_provenance(key).unwrap(),
        vec![NodeImportProvenance {
            source_id: "import:firefox-bookmarks".to_string(),
            source_label: "Firefox bookmarks".to_string(),
        }]
    );
}

#[test]
fn test_snapshot_roundtrip_preserves_classifications() {
    let mut graph = Graph::new();
    let key = graph.add_node("https://example.com".to_string(), Point2D::new(0.0, 0.0));

    let classification = NodeClassification {
        scheme: ClassificationScheme::Udc,
        value: "udc:519.6".to_string(),
        label: Some("Computational mathematics".to_string()),
        confidence: 0.9,
        provenance: ClassificationProvenance::UserAuthored,
        status: ClassificationStatus::Accepted,
        primary: true,
    };
    assert!(graph.add_node_classification(key, classification.clone()));

    let facets = graph.facets().clone();
    let mut restored = Graph::from_snapshot(&graph.to_snapshot());
    restored.overlay_facets(facets).unwrap();
    let (key, _) = restored.get_node_by_url("https://example.com").unwrap();
    let classifications = restored.node_classifications(key).unwrap();
    assert_eq!(classifications.len(), 1);
    let c = &classifications[0];
    assert_eq!(c.scheme, ClassificationScheme::Udc);
    assert_eq!(c.value, "udc:519.6");
    assert_eq!(c.label.as_deref(), Some("Computational mathematics"));
    assert!((c.confidence - 0.9).abs() < 1e-6);
    assert_eq!(c.provenance, ClassificationProvenance::UserAuthored);
    assert_eq!(c.status, ClassificationStatus::Accepted);
    assert!(c.primary);
}

#[test]
fn test_snapshot_roundtrip_preserves_imported_and_provenance_edge_sub_kinds() {
    let mut graph = Graph::new();
    let from = graph.add_node("https://from.example".to_string(), Point2D::new(0.0, 0.0));
    let to = graph.add_node("https://to.example".to_string(), Point2D::new(10.0, 0.0));

    let _ = graph.assert_relation(
        from,
        to,
        EdgeAssertion::Imported {
            sub_kind: ImportedSubKind::BookmarkFolder,
        },
    );
    let _ = graph.assert_relation(
        from,
        to,
        EdgeAssertion::Provenance {
            sub_kind: ProvenanceSubKind::ClippedFrom,
        },
    );

    let restored = Graph::from_snapshot(&graph.to_snapshot());
    let edge_key = restored
        .find_edge_key(from, to)
        .expect("restored imported/provenance edge");
    let payload = restored.get_edge(edge_key).expect("restored payload");

    assert!(payload.has_relation(RelationSelector::Imported(ImportedSubKind::BookmarkFolder)));
    assert!(!payload.has_relation(RelationSelector::Provenance(ProvenanceSubKind::ClippedFrom)));
    let resource = restored
        .find_resource_edge_key(
            restored.shown_resource_id(from).unwrap(),
            restored.shown_resource_id(to).unwrap(),
        )
        .unwrap();
    assert!(
        restored
            .get_resource_edge(resource)
            .unwrap()
            .has_relation(RelationSelector::Provenance(ProvenanceSubKind::ClippedFrom))
    );
}

#[test]
fn test_snapshot_roundtrip_preserves_import_records_and_suppression_state() {
    let mut graph = Graph::new();
    let included = graph.add_node(
        "https://included.example".to_string(),
        Point2D::new(0.0, 0.0),
    );
    let suppressed = graph.add_node(
        "https://suppressed.example".to_string(),
        Point2D::new(10.0, 0.0),
    );
    let included_id = graph
        .get_node(included)
        .expect("included node")
        .id
        .to_string();
    let suppressed_id = graph
        .get_node(suppressed)
        .expect("suppressed node")
        .id
        .to_string();
    assert!(graph.set_import_records(vec![ImportRecord {
        record_id: "import-record:firefox-bookmarks-2026-03-17".to_string(),
        source_id: "import:firefox-bookmarks".to_string(),
        source_label: "Firefox bookmarks".to_string(),
        imported_at_secs: 1_763_500_800,
        memberships: vec![
            ImportRecordMembership {
                node_id: included_id,
                suppressed: false,
            },
            ImportRecordMembership {
                node_id: suppressed_id,
                suppressed: true,
            },
        ],
    }]));

    let restored = Graph::from_snapshot(&graph.to_snapshot());
    let restored_records = restored.import_records();
    assert_eq!(restored_records.len(), 1);
    assert_eq!(
        restored_records[0].record_id,
        "import-record:firefox-bookmarks-2026-03-17"
    );
    assert_eq!(restored_records[0].memberships.len(), 2);
    assert!(
        restored_records[0]
            .memberships
            .iter()
            .any(|membership| membership.suppressed)
    );
    assert_eq!(
        restored
            .node_import_provenance(included)
            .expect("active provenance should exist")
            .len(),
        1
    );
    assert!(
        restored
            .node_import_provenance(suppressed)
            .expect("suppressed provenance should resolve to empty slice")
            .is_empty()
    );
}

#[test]
fn test_delete_import_record_removes_derived_provenance() {
    let mut graph = Graph::new();
    let key = graph.add_node("https://example.com".to_string(), Point2D::new(0.0, 0.0));
    let node_id = graph.get_node(key).expect("node").id.to_string();
    assert!(graph.set_import_records(vec![ImportRecord {
        record_id: "import-record:test".to_string(),
        source_id: "import:test".to_string(),
        source_label: "Test import".to_string(),
        imported_at_secs: 1_763_500_800,
        memberships: vec![ImportRecordMembership {
            node_id,
            suppressed: false,
        }],
    }]));

    assert!(graph.delete_import_record("import-record:test"));
    assert!(graph.import_records().is_empty());
    assert!(
        graph
            .node_import_provenance(key)
            .expect("node provenance slice")
            .is_empty()
    );
}

#[test]
fn test_suppress_import_record_membership_updates_node_projection() {
    let mut graph = Graph::new();
    let active = graph.add_node("https://active.example".to_string(), Point2D::new(0.0, 0.0));
    let peer = graph.add_node("https://peer.example".to_string(), Point2D::new(10.0, 0.0));
    let active_id = graph.get_node(active).expect("active").id.to_string();
    let peer_id = graph.get_node(peer).expect("peer").id.to_string();
    assert!(graph.set_import_records(vec![ImportRecord {
        record_id: "import-record:test".to_string(),
        source_id: "import:test".to_string(),
        source_label: "Test import".to_string(),
        imported_at_secs: 1_763_500_800,
        memberships: vec![
            ImportRecordMembership {
                node_id: active_id,
                suppressed: false,
            },
            ImportRecordMembership {
                node_id: peer_id,
                suppressed: false,
            },
        ],
    }]));

    assert!(graph.set_import_record_membership_suppressed("import-record:test", active, true,));
    assert!(
        graph
            .node_import_provenance(active)
            .expect("active provenance slice")
            .is_empty()
    );
    assert_eq!(
        graph.import_record_member_keys("import-record:test"),
        vec![peer]
    );
}

fn node_id(graph: &Graph, key: NodeKey) -> String {
    graph.get_node(key).expect("node").id.to_string()
}

/// `Some(suppressed)` when the record holds the node.
fn suppression(graph: &Graph, record_id: &str, node_id: &str) -> Option<bool> {
    let record = graph
        .import_records()
        .iter()
        .find(|record| record.record_id == record_id)?;
    let membership = record
        .memberships
        .iter()
        .find(|membership| membership.node_id == node_id)?;
    Some(membership.suppressed)
}

#[test]
fn node_provenance_write_changes_only_that_nodes_memberships() {
    let mut graph = Graph::new();
    let hidden = graph.add_node("https://hidden.example".to_string(), Point2D::new(0.0, 0.0));
    let peer = graph.add_node("https://peer.example".to_string(), Point2D::new(10.0, 0.0));
    let writer = graph.add_node(
        "https://writer.example".to_string(),
        Point2D::new(20.0, 0.0),
    );
    let (hidden_id, peer_id, writer_id) = (
        node_id(&graph, hidden),
        node_id(&graph, peer),
        node_id(&graph, writer),
    );
    let member = |node_id: &str, suppressed| ImportRecordMembership {
        node_id: node_id.to_string(),
        suppressed,
    };
    assert!(graph.set_import_records(vec![ImportRecord {
        record_id: "import-record:old".to_string(),
        source_id: "old".to_string(),
        source_label: "Old import".to_string(),
        imported_at_secs: 1,
        memberships: vec![member(&hidden_id, true), member(&peer_id, false)],
    }]));

    let new_source = NodeImportProvenance {
        source_id: "new".to_string(),
        source_label: "New import".to_string(),
    };
    assert!(graph.set_node_import_provenance(writer, vec![new_source.clone()]));
    assert_eq!(
        suppression(&graph, "import-record:old", &hidden_id),
        Some(true)
    );
    assert_eq!(
        suppression(&graph, "import-record:old", &peer_id),
        Some(false)
    );
    assert_eq!(suppression(&graph, "import-record:old", &writer_id), None);
    assert_eq!(
        suppression(&graph, "import-record:new", &writer_id),
        Some(false)
    );
    assert_eq!(graph.node_import_provenance(writer), Some(vec![new_source]));
    assert!(
        graph
            .node_import_provenance(hidden)
            .expect("hidden")
            .is_empty()
    );

    assert!(graph.set_node_import_provenance(writer, vec![]));
    assert_eq!(graph.import_records().len(), 1);
    assert_eq!(
        suppression(&graph, "import-record:old", &hidden_id),
        Some(true)
    );
    assert_eq!(
        suppression(&graph, "import-record:old", &peer_id),
        Some(false)
    );
    assert!(
        graph
            .node_import_provenance(writer)
            .expect("writer")
            .is_empty()
    );
}
