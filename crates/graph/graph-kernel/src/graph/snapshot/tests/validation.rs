// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn checked_snapshot_recorded_loader_preserves_same_host_resource_containment_and_surface_handles() {
    let mut graph = Graph::new();
    let parent = graph.add_node_with_id(
        Uuid::from_u128(1),
        "https://a.test/".into(),
        Point2D::zero(),
    );
    let child = graph.add_node_with_id(
        Uuid::from_u128(2),
        "https://a.test/child".into(),
        Point2D::zero(),
    );
    let parent_resource = ResourceNode::new("https://a.test/");
    let child_resource = ResourceNode::new("https://a.test/child");
    for resource in [&parent_resource, &child_resource] {
        assert!(graph.set_resource_record(
            resource.id(),
            Some(PersistedResourceRecord {
                canonical_iri: resource.canonical_iri().into(),
                facets: vec![],
            })
        ));
    }
    assert!(graph.set_shown_resource(Uuid::from_u128(1), Some(parent_resource.id())));
    assert!(graph.set_shown_resource(Uuid::from_u128(2), Some(child_resource.id())));
    let mut resource_relation = EdgePayload::new();
    for sub_kind in [ContainmentSubKind::Domain, ContainmentSubKind::UrlPath] {
        resource_relation.assert_relation(EdgeAssertion::Containment { sub_kind });
    }
    assert!(graph.set_resource_edges_between(
        child_resource.id(),
        parent_resource.id(),
        &[persisted_edge_for_ids(
            child_resource.id(),
            parent_resource.id(),
            &resource_relation
        ),]
    ));
    for (id, author, time) in [
        ("held-a", "https://alice.test/", 11),
        ("held-b", "https://bob.test/", 22),
    ] {
        let mut payload = EdgePayload::new();
        payload.push_persisted_semantic_statement(SemanticStatement {
            statement_id: id.into(),
            predicate: predicate_iri(SemanticSubKind::UserGrouped).into(),
            recognized_sub_kind: Some(SemanticSubKind::UserGrouped),
            label: Some(id.into()),
            graph_scope: GraphScope::User,
            provenance_iri: Some(author.into()),
            asserted_at_ms: Some(time),
            first_asserted_ms: Some(time),
            withdrawn: None,
            basis: Default::default(),
        });
        graph.inner.connect(child, parent, payload);
    }
    let snapshot = graph.to_snapshot();
    let recorded = Graph::try_from_recorded_snapshot(&snapshot).unwrap();
    assert_eq!(
        recorded.to_snapshot().resource_edges,
        snapshot.resource_edges
    );
    assert_eq!(
        recorded.to_snapshot().edges,
        snapshot.edges,
        "parallel held surface records remain exact"
    );
    assert_eq!(recorded.inner.edge_count(), 2);
    assert!(
        recorded
            .relations()
            .all(|row| !matches!(row.kind, RelationKind::Containment(_))),
        "recorded resource containment does not acquire Surface authority"
    );
    let legacy = Graph::try_from_snapshot(&snapshot).unwrap();
    for kind in [ContainmentSubKind::Domain, ContainmentSubKind::UrlPath] {
        assert!(
            legacy
                .relations()
                .any(|row| row.kind == RelationKind::Containment(kind)),
            "the existing loader still derives legacy containment"
        );
    }
    assert_eq!(legacy.to_snapshot().resource_edges, snapshot.resource_edges);
    let mut invalid = snapshot.clone();
    invalid.resource_edges[0].from_node_id = Uuid::from_u128(99).to_string();
    let before = serde_json::to_value(&invalid).unwrap();
    let legacy_error = Graph::try_from_snapshot(&invalid)
        .err()
        .expect("legacy checked boundary rejects invalid resource");
    let recorded_error = Graph::try_from_recorded_snapshot(&invalid)
        .err()
        .expect("recorded checked boundary rejects invalid resource");
    assert_eq!(legacy_error, recorded_error);
    assert!(matches!(
        recorded_error,
        ResourceSnapshotError::MissingResource { .. }
    ));
    assert_eq!(serde_json::to_value(&invalid).unwrap(), before);
    assert!(Graph::try_from_recorded_snapshot(&snapshot).is_ok());
}

#[test]
fn checked_snapshot_compatibility_wrappers_stop_before_partial_materialization() {
    let (valid, surface, from, to) = resource_snapshot_fixture();
    assert_resource_snapshot_restores(&valid, surface, from, to);
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&valid).unwrap();
    let graph = rkyv::from_bytes::<Graph, rkyv::rancor::Error>(&bytes).unwrap();
    assert_eq!(graph.to_snapshot().resource_edges, valid.resource_edges);

    let mut invalid = valid.clone();
    let mut conflicting = invalid.resource_edges[0].clone();
    conflicting.from_node_id = to.to_string();
    conflicting.to_node_id = from.to_string();
    invalid.resource_edges.push(conflicting);
    let error = checked_rejection(&invalid).to_string();
    let stopped = std::panic::catch_unwind(|| Graph::from_snapshot(&invalid));
    let panic = match stopped {
        Err(panic) => panic,
        Ok(_) => panic!("compatibility wrapper returned partial graph"),
    };
    let message = panic.downcast_ref::<String>().unwrap();
    assert!(message.contains(&error));
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&invalid).unwrap();
    assert!(
        std::panic::catch_unwind(|| { rkyv::from_bytes::<Graph, rkyv::rancor::Error>(&bytes) })
            .is_err()
    );
    assert!(Graph::try_from_snapshot(&valid).is_ok());
}

#[test]
fn checked_resource_load_accepts_identical_duplicates_and_legacy_empty_columns() {
    let (mut snapshot, surface, from, to) = resource_snapshot_fixture();
    assert!(Graph::try_from_snapshot(&snapshot).is_ok());
    snapshot.resources.push(snapshot.resources[0].clone());
    snapshot
        .shown_resources
        .push(snapshot.shown_resources[0].clone());
    let graph = Graph::try_from_snapshot(&snapshot).unwrap();
    assert_eq!(graph.resources.node_count(), 2);
    assert_eq!(graph.resources.edge_count(), 2);
    assert_eq!(graph.shown_resources.get(&surface), Some(&from));
    assert_eq!(
        graph.persisted_resource_edges_between(from, to),
        snapshot.resource_edges
    );
    snapshot.resources.clear();
    snapshot.resource_edges.clear();
    snapshot.shown_resources.clear();
    let graph = Graph::try_from_snapshot(&snapshot).unwrap();
    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.resources.node_count(), 0);
    let mut json = serde_json::to_value(&snapshot).unwrap();
    for column in ["resources", "resource_edges", "shown_resources"] {
        json.as_object_mut().unwrap().remove(column);
    }
    let legacy = serde_json::from_value::<GraphSnapshot>(json).unwrap();
    assert!(Graph::try_from_snapshot(&legacy).is_ok());
}

#[test]
fn checked_resource_load_rejects_invalid_facets_and_conflicting_records() {
    let (base, _, _, _) = resource_snapshot_fixture();
    assert!(Graph::try_from_snapshot(&base).is_ok());
    let mut invalid = base.clone();
    invalid.resources[0].facets[0].value_json = "{".into();
    assert!(matches!(
        checked_rejection(&invalid),
        ResourceSnapshotError::InvalidFacet { .. }
    ));
    let mut duplicate = base.clone();
    let facet = duplicate.resources[0].facets[0].clone();
    duplicate.resources[0].facets.push(facet);
    assert!(matches!(
        checked_rejection(&duplicate),
        ResourceSnapshotError::DuplicateFacet { .. }
    ));
    let mut conflict = base.clone();
    let mut record = conflict.resources[0].clone();
    record.facets[0].value_json = "false".into();
    conflict.resources.push(record);
    assert!(matches!(
        checked_rejection(&conflict),
        ResourceSnapshotError::ConflictingResource { .. }
    ));
}

#[test]
fn checked_resource_load_rejects_invalid_endpoints_and_unhandled_semantic_data() {
    let (base, _, from, to) = resource_snapshot_fixture();
    assert!(Graph::try_from_snapshot(&base).is_ok());
    let mut invalid = base.clone();
    invalid.resource_edges[0].from_node_id = "invalid".into();
    assert!(matches!(
        checked_rejection(&invalid),
        ResourceSnapshotError::InvalidUuid { .. }
    ));
    let mut noncanonical = base.clone();
    noncanonical.resource_edges[0].from_node_id = from.to_string().to_uppercase();
    assert_ne!(
        noncanonical.resource_edges[0].from_node_id,
        from.to_string()
    );
    assert!(matches!(
        checked_rejection(&noncanonical),
        ResourceSnapshotError::NonCanonicalUuid { .. }
    ));
    let mut unknown = base.clone();
    unknown.resource_edges[0].to_node_id = Uuid::from_u128(123).to_string();
    assert!(matches!(
        checked_rejection(&unknown),
        ResourceSnapshotError::MissingResource { .. }
    ));
    let mut aggregate = base.clone();
    aggregate.assertion_schema = 0;
    let edge = &mut aggregate.resource_edges[0];
    edge.assertions = None;
    edge.semantic = Some(PersistedSemanticEdgeData {
        sub_kinds: vec![PersistedSemanticSubKind::Cites],
        ..Default::default()
    });
    assert!(matches!(
        checked_rejection(&aggregate),
        ResourceSnapshotError::LegacySemanticAggregate { .. }
    ));
    for handle in ["", "  ", "opaque\nhandle"] {
        let mut opaque = base.clone();
        opaque.resource_edges[0].retained_assertions_mut()[0].statement_id = handle.into();
        let expected = opaque.resource_edges[0].retained_assertions()[0].clone();
        let graph = Graph::try_from_snapshot(&opaque).unwrap();
        assert_eq!(
            graph.persisted_resource_edges_between(from, to)[0].retained_assertions()[0],
            expected
        );
    }
    let mut empty_sidecar = base.clone();
    empty_sidecar.resource_edges[1].assertions = Some(vec![]);
    let graph = Graph::try_from_snapshot(&empty_sidecar).unwrap();
    assert_eq!(graph.resources.edge_count(), 2);
    assert_eq!(
        graph.persisted_resource_edges_between(from, to)[1].provenance,
        base.resource_edges[1].provenance
    );
}

#[test]
fn checked_resource_load_rejects_reused_handles_and_conflicting_shown_resources() {
    let (base, surface, from, to) = resource_snapshot_fixture();
    assert!(Graph::try_from_snapshot(&base).is_ok());
    let mut changed_metadata = base.clone();
    let mut statement = changed_metadata.resource_edges[0].retained_assertions()[0].clone();
    statement.asserted_at_ms = Some(999);
    changed_metadata.resource_edges[0]
        .assertions
        .as_mut()
        .unwrap()
        .push(statement);
    assert!(matches!(
        checked_rejection(&changed_metadata),
        ResourceSnapshotError::RepeatedAssertionHandle(_)
    ));
    let mut changed_pair = base.clone();
    let mut reverse = changed_pair.resource_edges[0].clone();
    reverse.from_node_id = to.to_string();
    reverse.to_node_id = from.to_string();
    changed_pair.resource_edges.push(reverse);
    assert!(matches!(
        checked_rejection(&changed_pair),
        ResourceSnapshotError::RepeatedAssertionHandle(_)
    ));
    let mut active_surface = base.clone();
    let mut surface_edge = active_surface.resource_edges[0].clone();
    surface_edge.from_node_id = surface.to_string();
    surface_edge.to_node_id = surface.to_string();
    active_surface.edges.push(surface_edge);
    assert!(matches!(
        checked_rejection(&active_surface),
        ResourceSnapshotError::RepeatedAssertionHandle(_)
    ));
    let mut orphan_surface = active_surface.clone();
    orphan_surface.edges[0].to_node_id = Uuid::from_u128(123).to_string();
    assert!(matches!(
        checked_rejection(&orphan_surface),
        ResourceSnapshotError::MissingSurface { .. }
    ));
    // Schema 0 retains the historical permissive orphan handling.
    orphan_surface.assertion_schema = 0;
    let restored = Graph::try_from_snapshot(&orphan_surface).unwrap();
    assert_eq!(
        restored.persisted_resource_edges_between(from, to),
        base.resource_edges
    );
    assert_eq!(restored.edge_count(), 0);
    assert_eq!(
        orphan_surface.edges[0].assertions,
        active_surface.edges[0].assertions
    );
    let mut uppercase_surface = active_surface.clone();
    uppercase_surface.edges[0].from_node_id = surface.to_string().to_uppercase();
    assert!(matches!(
        checked_rejection(&uppercase_surface),
        ResourceSnapshotError::NonCanonicalUuid { .. }
    ));
    let mut shown_conflict = base.clone();
    shown_conflict.shown_resources.push(PersistedShownResource {
        surface_id: surface.to_string(),
        resource_id: to.to_string(),
    });
    assert!(matches!(
        checked_rejection(&shown_conflict),
        ResourceSnapshotError::ConflictingShownResource { .. }
    ));
    let mut missing_surface = base.clone();
    missing_surface.shown_resources[0].surface_id = Uuid::from_u128(123).to_string();
    assert!(matches!(
        checked_rejection(&missing_surface),
        ResourceSnapshotError::MissingSurface { .. }
    ));
    let mut missing_resource = base.clone();
    missing_resource.shown_resources[0].resource_id = Uuid::from_u128(123).to_string();
    assert!(matches!(
        checked_rejection(&missing_resource),
        ResourceSnapshotError::MissingResource { .. }
    ));
    let mut invalid_mapping = base.clone();
    invalid_mapping.shown_resources[0].resource_id = "invalid".into();
    assert!(matches!(
        checked_rejection(&invalid_mapping),
        ResourceSnapshotError::InvalidUuid { .. }
    ));
}
