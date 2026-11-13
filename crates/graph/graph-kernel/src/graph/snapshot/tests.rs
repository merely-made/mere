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

pub(super) fn complete_payload() -> EdgePayload {
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

pub(super) fn resource_snapshot_fixture() -> (GraphSnapshot, Uuid, Uuid, Uuid) {
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
    let from = portolan::resource_id_from_canonical_iri(&first.canonical_iri);
    let to = portolan::resource_id_from_canonical_iri(&second.canonical_iri);
    assert_ne!(from, portolan::resource_id(&first.canonical_iri));
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

/// Q013: the saved node address owns the current address on load; a stale
/// navigation cursor must not undo a successful `SetNodeUrl`.
#[test]
fn recorded_load_keeps_a_set_node_url_over_the_navigation_cursor() {
    use crate::graph::apply::{GraphDelta, add_node, apply_graph_delta};
    let mut graph = Graph::new();
    let id = Uuid::from_u128(1);
    let key = add_node(
        &mut graph,
        Some(id),
        "https://audit.test/old".into(),
        Default::default(),
    );
    apply_graph_delta(
        &mut graph,
        GraphDelta::NavigateNode {
            key,
            url: "https://audit.test/old".into(),
        },
    );
    apply_graph_delta(
        &mut graph,
        GraphDelta::SetNodeUrl {
            key,
            new_url: "https://audit.test/new".into(),
        },
    );
    let restored = Graph::try_from_recorded_snapshot(&graph.to_snapshot()).unwrap();
    let key = restored.get_node_key_by_id(id).unwrap();
    assert_eq!(
        restored.get_node(key).unwrap().url(),
        "https://audit.test/new"
    );
    assert_eq!(
        restored.get_nodes_by_url("https://audit.test/new"),
        vec![key]
    );
    assert!(
        restored
            .get_nodes_by_url("https://audit.test/old")
            .is_empty()
    );
}

/// Q015: every index in the navigation snapshot names a row of its own column,
/// so each checked load refuses one that names none instead of panicking when
/// Stemma hydrates it.
#[test]
fn checked_loads_refuse_a_navigation_index_that_names_no_row() {
    use crate::graph::apply::add_node;
    use hodograph::{StemmaSnapshot, TransitionKind};
    type Nav = StemmaSnapshot<String, String, NodeHistoryOwner, ()>;
    const BAD: usize = 999_999;

    let (a, b) = (Uuid::from_u128(1), Uuid::from_u128(2));
    let mut nav = SharedNavigationMemory::empty();
    nav.record_visit(a, "https://audit.test/a", TransitionKind::UrlTyped, 1);
    nav.record_visit(a, "https://audit.test/b", TransitionKind::LinkClick, 2);
    nav.back(a, 3);
    nav.spawn(b, a);
    nav.record_visit(b, "https://audit.test/c", TransitionKind::UrlTyped, 4);
    let mut graph = Graph::new();
    for (id, url) in [(a, "https://audit.test/a"), (b, "https://audit.test/c")] {
        add_node(&mut graph, Some(id), url.into(), Default::default());
    }
    let mut valid = graph.to_snapshot();
    valid.navigation = nav;
    assert!(Graph::try_from_snapshot(&valid).is_ok());
    assert!(Graph::try_from_recorded_snapshot(&valid).is_ok());

    let corruptions: [(&str, fn(&mut Nav)); 10] = [
        ("visit entry", |n| n.visits[0].entry = BAD),
        ("visit parent", |n| n.visits[1].parent = Some(BAD)),
        ("visit child", |n| n.visits[0].children[0] = BAD),
        ("binding owner", |n| {
            n.visits
                .iter_mut()
                .flat_map(|v| &mut v.bindings)
                .next()
                .unwrap()
                .owner = BAD
        }),
        ("binding forward child", |n| {
            let binding = n
                .visits
                .iter_mut()
                .flat_map(|v| &mut v.bindings)
                .next()
                .unwrap();
            binding.forward_child = Some(BAD)
        }),
        ("owner origin", |n| n.owners[0].origin = Some(BAD)),
        ("owner current", |n| n.owners[0].current = Some(BAD)),
        ("owner creator", |n| n.owners[1].creator = Some(BAD)),
        ("owner pending origin parent", |n| {
            n.owners[1].pending_origin_parent = Some(BAD)
        }),
        ("owner owned visit", |n| n.owners[0].owned_visits[0] = BAD),
    ];
    for (what, corrupt) in corruptions {
        let mut snapshot = valid.clone();
        let mut nav = snapshot.navigation.snapshot().clone();
        corrupt(&mut nav);
        snapshot.navigation = SharedNavigationMemory::from_snapshot(nav);
        let loads = [
            Graph::try_from_snapshot(&snapshot),
            Graph::try_from_recorded_snapshot(&snapshot),
        ];
        for load in loads {
            assert!(
                matches!(load, Err(ResourceSnapshotError::InvalidNavigation(_))),
                "{what}"
            );
        }
    }
}

#[path = "tests/validation.rs"]
mod validation;
