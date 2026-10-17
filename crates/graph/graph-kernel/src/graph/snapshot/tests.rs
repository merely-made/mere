// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use euclid::default::Point2D;
use strum::IntoEnumIterator;
use uuid::Uuid;

use super::from::restore_persisted_payload;
use super::{ResourceSnapshotError, payload_from_persisted, persisted_edge_for_ids};
use crate::graph::*;
use crate::persistence::{
    GraphSnapshot, PersistedEdge, PersistedResourceFacet, PersistedResourceRecord,
    PersistedSemanticEdgeData, PersistedSemanticSubKind, PersistedShownResource,
};
use crate::types::GraphScope;

fn complete_payload() -> EdgePayload {
    let mut payload = EdgePayload::new();
    for sub_kind in all_semantic_sub_kinds() {
        for (asserter, time) in [("https://alice.test/", 11), ("https://bob.test/", 22)] {
            payload.push_persisted_semantic_statement(SemanticStatement {
                statement_id: format!("claim-{sub_kind:?}-{time}"),
                predicate: predicate_iri(sub_kind).to_owned(),
                recognized_sub_kind: Some(sub_kind),
                label: Some(format!("label-{sub_kind:?}-{time}")),
                graph_scope: GraphScope::Custom("https://scope.test/claims".into()),
                provenance_iri: Some(asserter.into()),
                asserted_at_ms: Some(time),
                first_asserted_ms: Some(time),
                withdrawn: None,
                basis: Default::default(),
            });
        }
    }
    payload.push_persisted_semantic_statement(SemanticStatement {
        statement_id: "raw-claim".into(),
        predicate: "https://vocabulary.test/ns#refutes".into(),
        recognized_sub_kind: None,
        label: Some("raw label".into()),
        graph_scope: GraphScope::Source,
        provenance_iri: None,
        asserted_at_ms: Some(33),
        first_asserted_ms: Some(33),
        withdrawn: None,
        basis: Default::default(),
    });
    for sub_kind in ArrangementSubKind::iter() {
        payload.assert_relation(EdgeAssertion::Arrangement { sub_kind });
    }
    for sub_kind in ContainmentSubKind::iter() {
        payload.assert_relation(EdgeAssertion::Containment { sub_kind });
    }
    for sub_kind in ImportedSubKind::iter() {
        payload.assert_relation(EdgeAssertion::Imported { sub_kind });
    }
    for sub_kind in ProvenanceSubKind::iter() {
        payload.assert_relation(EdgeAssertion::Provenance { sub_kind });
    }
    for trigger in [
        NavigationTrigger::Unknown,
        NavigationTrigger::LinkClick,
        NavigationTrigger::Back,
        NavigationTrigger::Forward,
        NavigationTrigger::AddressBarEntry,
        NavigationTrigger::PanePromotion,
        NavigationTrigger::Programmatic,
        NavigationTrigger::Redirect,
        NavigationTrigger::ReopenSession,
        NavigationTrigger::JumpAnchor,
        NavigationTrigger::InPageSearchJump,
        NavigationTrigger::ImportedHistory,
    ] {
        payload.push_traversal(Traversal {
            timestamp_ms: 55,
            trigger,
        });
    }
    payload
}

#[test]
fn shared_payload_conversion_preserves_every_sidecar_and_separate_assertions() {
    let from = Uuid::from_u128(1);
    let to = Uuid::from_u128(2);
    let payload = complete_payload();
    let encoded = persisted_edge_for_ids(from, to, &payload);
    assert_eq!(encoded.from_node_id, from.to_string());
    assert_eq!(encoded.to_node_id, to.to_string());
    assert_eq!(encoded.families.len(), 6);
    let decoded = payload_from_persisted(&encoded);
    for sub_kind in [ArrangementSubKind::TileGroup, ArrangementSubKind::SplitPair] {
        assert!(payload.has_relation(RelationSelector::Arrangement(sub_kind)));
        assert!(!decoded.has_relation(RelationSelector::Arrangement(sub_kind)));
    }
    assert!(decoded.has_relation(RelationSelector::Arrangement(
        ArrangementSubKind::FrameMember
    )));
    let mut durable = payload.clone();
    durable.assertions.statements.retain(|statement| !matches!(
        statement.relation_kind(), RelationKind::Arrangement(kind) if kind.durability() != RelationDurability::Durable
    ));
    durable.assertions.rebuild_compat();
    assert_eq!(decoded, durable);
    assert_eq!(persisted_edge_for_ids(from, to, &decoded), encoded);
    let cites = decoded
        .semantic_statements()
        .iter()
        .filter(|claim| claim.recognized_sub_kind == Some(SemanticSubKind::Cites))
        .collect::<Vec<_>>();
    assert_eq!(cites.len(), 2);
    assert_ne!(cites[0].statement_id, cites[1].statement_id);
    assert_ne!(cites[0].provenance_iri, cites[1].provenance_iri);
    assert_ne!(cites[0].asserted_at_ms, cites[1].asserted_at_ms);
    assert!(
        decoded
            .semantic_statements()
            .iter()
            .any(|claim| claim.statement_id == "raw-claim" && claim.provenance_iri.is_none())
    );
    assert!(
        payload_from_persisted(&persisted_edge_for_ids(from, to, &EdgePayload::new())).is_empty()
    );
    let mut merged = EdgePayload::new();
    assert!(restore_persisted_payload(&mut merged, &encoded) > 0);
    assert_eq!(restore_persisted_payload(&mut merged, &encoded), 0);
    assert_eq!(merged, durable);
}

#[test]
fn surface_restore_retains_revision_bumps_and_normalization_boundary() {
    let mut graph = Graph::new();
    let from = graph.add_node("https://a.test/".into(), Point2D::new(0.0, 0.0));
    let to = graph.add_node("https://b.test/".into(), Point2D::new(0.0, 0.0));
    let from_id = graph.get_node(from).unwrap().id;
    let to_id = graph.get_node(to).unwrap().id;
    let encoded = persisted_edge_for_ids(from_id, to_id, &complete_payload());
    let mut expected = EdgePayload::new();
    let changes = restore_persisted_payload(&mut expected, &encoded);
    let revision = graph.revision();
    graph.restore_persisted_edge(from, to, &encoded);
    assert_eq!(graph.revision(), revision + changes as u64);
    assert_eq!(
        graph.get_edge(graph.find_edge_key(from, to).unwrap()),
        Some(&expected)
    );
    graph.restore_persisted_edge(from, to, &encoded);
    assert_eq!(graph.revision(), revision + changes as u64);
    let restored = Graph::from_snapshot(&graph.to_snapshot());
    let claims = restored
        .get_edge(restored.find_edge_key(from, to).unwrap())
        .unwrap()
        .semantic_statements();
    assert!(claims.iter().any(|claim| claim.statement_id == "raw-claim"
        && claim.provenance_iri.as_deref() == Some(edge_data::UNKNOWN_LEGACY_ASSERTER_IRI)));
    assert!(
        claims
            .iter()
            .any(|claim| claim.provenance_iri.as_deref() == Some("https://alice.test/"))
    );
}

fn resource_snapshot_fixture() -> (GraphSnapshot, Uuid, Uuid, Uuid) {
    let mut graph = Graph::new();
    let surface = graph.add_node("https://surface.test/".into(), Point2D::new(0.0, 0.0));
    let surface_id = graph.get_node(surface).unwrap().id;
    let first = PersistedResourceRecord {
        canonical_iri: "https://vocabulary.test/ns#Topic".into(),
        facets: vec![PersistedResourceFacet {
            facet: "foreign.review".into(),
            value_json: r#"{"review":"suggested","nested":[1,true,null]}"#.into(),
        }],
    };
    let second = PersistedResourceRecord {
        canonical_iri: "https://target.test/item?id=7".into(),
        facets: Vec::new(),
    };
    let from = chartulary::resource_id_from_canonical_iri(&first.canonical_iri);
    let to = chartulary::resource_id_from_canonical_iri(&second.canonical_iri);
    assert_ne!(from, chartulary::resource_id(&first.canonical_iri));
    assert!(graph.set_resource_record(from, Some(first)));
    assert!(graph.set_resource_record(to, Some(second)));
    let mut payload = complete_payload();
    for claim in &mut payload.assertions.statements {
        claim.normalize_legacy_asserter();
    }
    let first_edge = persisted_edge_for_ids(from, to, &payload);
    let mut other = EdgePayload::new();
    other.assert_relation(EdgeAssertion::Provenance {
        sub_kind: ProvenanceSubKind::ExtractedFrom,
    });
    let second_edge = persisted_edge_for_ids(from, to, &other);
    assert!(graph.set_resource_edges_between(from, to, &[first_edge, second_edge]));
    assert!(graph.set_shown_resource(surface_id, Some(from)));
    (graph.to_snapshot(), surface_id, from, to)
}

fn assert_resource_snapshot_restores(
    snapshot: &GraphSnapshot,
    surface: Uuid,
    from: Uuid,
    to: Uuid,
) {
    let mut graph = Graph::from_snapshot(snapshot);
    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.resources.node_count(), 2);
    assert_eq!(graph.resources.edge_count(), 2);
    assert_eq!(graph.shown_resources.get(&surface), Some(&from));
    assert_eq!(
        graph.resource_record(from).unwrap().canonical_iri,
        "https://vocabulary.test/ns#Topic"
    );
    assert_eq!(
        graph.resource_record(from).unwrap().facets,
        snapshot.resources[0].facets
    );
    assert_eq!(
        graph.persisted_resource_edges_between(from, to),
        snapshot.resource_edges
    );
    assert!(graph.persisted_resource_edges_between(to, from).is_empty());
    assert!(graph.resource_record(Uuid::from_u128(123)).is_none());
    let saved = graph.to_snapshot();
    assert_eq!(saved.resources, snapshot.resources);
    assert_eq!(saved.resource_edges, snapshot.resource_edges);
    assert_eq!(saved.shown_resources, snapshot.shown_resources);
    let revision = graph.revision();
    for id in [from, to] {
        let record = graph.resource_record(id).unwrap();
        assert!(!graph.set_resource_record(id, Some(record)));
    }
    let edges = graph.persisted_resource_edges_between(from, to);
    assert!(!graph.set_resource_edges_between(from, to, &edges));
    assert!(!graph.set_shown_resource(surface, Some(from)));
    assert_eq!(graph.revision(), revision);
}

#[test]
fn resource_snapshot_json_and_rkyv_preserve_records_pairs_and_shown_resources() {
    let (snapshot, surface, from, to) = resource_snapshot_fixture();
    let json = serde_json::to_string(&snapshot).unwrap();
    let decoded: GraphSnapshot = serde_json::from_str(&json).unwrap();
    assert_resource_snapshot_restores(&decoded, surface, from, to);
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&snapshot).unwrap();
    let decoded = rkyv::from_bytes::<GraphSnapshot, rkyv::rancor::Error>(&bytes).unwrap();
    assert_resource_snapshot_restores(&decoded, surface, from, to);
}

#[test]
fn resource_snapshot_columns_and_assertion_metadata_are_postcard_compatible() {
    type Columns = (
        Vec<PersistedResourceRecord>,
        Vec<PersistedEdge>,
        Vec<PersistedShownResource>,
    );
    let (snapshot, surface, from, to) = resource_snapshot_fixture();
    let columns = (
        snapshot.resources.clone(),
        snapshot.resource_edges.clone(),
        snapshot.shown_resources.clone(),
    );
    let bytes = postcard::to_allocvec(&columns).unwrap();
    let decoded: Columns = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(decoded, columns);
    let rebuilt = GraphSnapshot {
        resources: decoded.0,
        resource_edges: decoded.1,
        shown_resources: decoded.2,
        ..snapshot
    };
    assert_resource_snapshot_restores(&rebuilt, surface, from, to);
}

#[test]
fn legacy_json_keeps_surfaces_and_has_no_resource_auto_population() {
    let (snapshot, surface, from, to) = resource_snapshot_fixture();
    assert_resource_snapshot_restores(&snapshot, surface, from, to);
    let mut legacy = serde_json::to_value(&snapshot).unwrap();
    let object = legacy.as_object_mut().unwrap();
    for column in ["resources", "resource_edges", "shown_resources"] {
        assert!(object.remove(column).is_some());
    }
    let decoded: GraphSnapshot = serde_json::from_value(legacy).unwrap();
    assert!(decoded.resources.is_empty());
    assert!(decoded.resource_edges.is_empty());
    assert!(decoded.shown_resources.is_empty());
    let graph = Graph::from_snapshot(&decoded);
    assert!(graph.get_node_key_by_id(surface).is_some());
    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.resources.node_count(), 0);
    assert_eq!(graph.resources.edge_count(), 0);
    assert!(graph.shown_resources.is_empty());
}

#[test]
fn resource_snapshot_normalizes_only_missing_legacy_attribution() {
    let (mut snapshot, _, from, to) = resource_snapshot_fixture();
    // Exercise the schema-0 semantic sidecar decoder explicitly. Canonical
    // snapshots carry all-family assertion records and do not use this field.
    snapshot.assertion_schema = 0;
    for edge in &mut snapshot.resource_edges {
        let statements: Vec<_> = edge
            .assertions
            .take()
            .unwrap_or_default()
            .into_iter()
            .filter(|statement| {
                statement.recognized_sub_kind.is_some()
                    || statement.predicate == "https://vocabulary.test/ns#refutes"
            })
            .collect();
        if statements.is_empty() {
            continue;
        }
        edge.semantic = Some(PersistedSemanticEdgeData {
            sub_kinds: all_semantic_sub_kinds()
                .map(|kind| match kind {
                    SemanticSubKind::Hyperlink => PersistedSemanticSubKind::Hyperlink,
                    SemanticSubKind::UserGrouped => PersistedSemanticSubKind::UserGrouped,
                    SemanticSubKind::AgentDerived => PersistedSemanticSubKind::AgentDerived,
                    SemanticSubKind::Cites => PersistedSemanticSubKind::Cites,
                    SemanticSubKind::Quotes => PersistedSemanticSubKind::Quotes,
                    SemanticSubKind::Summarizes => PersistedSemanticSubKind::Summarizes,
                    SemanticSubKind::Elaborates => PersistedSemanticSubKind::Elaborates,
                    SemanticSubKind::ExampleOf => PersistedSemanticSubKind::ExampleOf,
                    SemanticSubKind::Supports => PersistedSemanticSubKind::Supports,
                    SemanticSubKind::Contradicts => PersistedSemanticSubKind::Contradicts,
                    SemanticSubKind::Questions => PersistedSemanticSubKind::Questions,
                    SemanticSubKind::SameEntityAs => PersistedSemanticSubKind::SameEntityAs,
                    SemanticSubKind::DuplicateOf => PersistedSemanticSubKind::DuplicateOf,
                    SemanticSubKind::CanonicalMirrorOf => {
                        PersistedSemanticSubKind::CanonicalMirrorOf
                    },
                    SemanticSubKind::DependsOn => PersistedSemanticSubKind::DependsOn,
                    SemanticSubKind::Blocks => PersistedSemanticSubKind::Blocks,
                    SemanticSubKind::NextStep => PersistedSemanticSubKind::NextStep,
                })
                .collect(),
            statements,
            ..Default::default()
        });
    }
    let raw = snapshot
        .resource_edges
        .iter_mut()
        .filter_map(|edge| edge.semantic.as_mut())
        .flat_map(|semantic| semantic.statements.iter_mut())
        .find(|statement| statement.statement_id == "raw-claim")
        .unwrap();
    raw.provenance_iri = None;
    let graph = Graph::from_snapshot(&snapshot);
    let claims = graph
        .persisted_resource_edges_between(from, to)
        .into_iter()
        .flat_map(|edge| edge.retained_assertions().to_vec())
        .collect::<Vec<_>>();
    assert!(claims.iter().any(|claim| claim.statement_id == "raw-claim"
        && claim.provenance_iri.as_deref() == Some(edge_data::UNKNOWN_LEGACY_ASSERTER_IRI)));
    assert!(
        claims
            .iter()
            .any(|claim| claim.provenance_iri.as_deref() == Some("https://alice.test/"))
    );
    assert_eq!(claims.len(), all_semantic_sub_kinds().count() * 2 + 1);
}

fn checked_rejection(snapshot: &GraphSnapshot) -> ResourceSnapshotError {
    let before = serde_json::to_value(snapshot).unwrap();
    let error = match Graph::try_from_snapshot(snapshot) {
        Err(error) => error,
        Ok(_) => panic!("invalid resource columns accepted"),
    };
    assert_eq!(serde_json::to_value(snapshot).unwrap(), before);
    assert!(!error.to_string().is_empty());
    error
}

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
        ResourceSnapshotError::ConflictingAssertion { .. }
    ));
    let mut changed_pair = base.clone();
    let mut reverse = changed_pair.resource_edges[0].clone();
    reverse.from_node_id = to.to_string();
    reverse.to_node_id = from.to_string();
    changed_pair.resource_edges.push(reverse);
    assert!(matches!(
        checked_rejection(&changed_pair),
        ResourceSnapshotError::ConflictingAssertion { .. }
    ));
    let mut active_surface = base.clone();
    let mut surface_edge = active_surface.resource_edges[0].clone();
    surface_edge.from_node_id = surface.to_string();
    surface_edge.to_node_id = surface.to_string();
    active_surface.edges.push(surface_edge);
    assert!(matches!(
        checked_rejection(&active_surface),
        ResourceSnapshotError::ActiveSurfaceAssertion { .. }
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
