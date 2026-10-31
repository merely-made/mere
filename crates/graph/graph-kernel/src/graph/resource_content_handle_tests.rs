// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Replayed literal handles meet the same rule the checked open path applies.
use super::apply::{GraphDelta, apply_graph_delta};
use super::node_facets::SEMANTIC_PROPERTIES;
use super::*;
use crate::persistence::{PersistedResourceFacet, PersistedResourceRecord};
use crate::types::NodeProperty;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

fn record(iri: &str, properties: &[NodeProperty]) -> PersistedResourceRecord {
    PersistedResourceRecord {
        canonical_iri: iri.into(),
        facets: vec![PersistedResourceFacet {
            facet: super::resource_content::RESOURCE_PROPERTIES.into(),
            value_json: serde_json::to_string(properties).unwrap(),
        }],
    }
}

fn set_record(graph: &mut Graph, iri: &str, properties: &[NodeProperty]) {
    apply_graph_delta(
        graph,
        GraphDelta::ReplaySetResourceRecordById {
            resource_id: chartulary::resource_id_from_canonical_iri(iri),
            record: Some(record(iri, properties)),
        },
    );
}

fn reopens(graph: &Graph) -> bool {
    graph.validate_active_resource_assertion_handles().is_ok()
        && Graph::try_from_recorded_snapshot(&graph.to_snapshot()).is_ok()
}

#[test]
fn replayed_literal_handles_are_refused_where_checked_open_refuses_them() {
    let mut graph = Graph::new();
    let seen = Arc::new(Mutex::new(Vec::<CapturedDelta>::new()));
    let sink = seen.clone();
    graph.set_recorder(Some(Arc::new(move |delta| {
        sink.lock().unwrap().push(delta.clone())
    })));
    let mut held = NodeProperty::new("urn:audit:predicate".into(), "value".into());
    held.statement_id = "owned-handle".into();
    let (first, second) = ("urn:audit:first", "urn:audit:second");

    // Resource records: a second owner of a held handle is refused and unrecorded.
    set_record(&mut graph, first, &[held.clone()]);
    let recorded = seen.lock().unwrap().len();
    set_record(&mut graph, second, &[held.clone()]);
    assert_eq!(graph.resource_nodes().count(), 1);
    assert_eq!(seen.lock().unwrap().len(), recorded);
    assert!(reopens(&graph));

    // The owner may restate its own literal; a distinct handle has no owner.
    set_record(&mut graph, first, &[held.clone()]);
    let mut distinct = held.clone();
    distinct.statement_id = "distinct-handle".into();
    set_record(&mut graph, second, &[distinct]);
    assert_eq!(graph.resource_nodes().count(), 2);
    assert!(reopens(&graph));

    // Retained Surface literals: no delta kind may reuse a Resource-held handle.
    let surface = Uuid::from_u128(7);
    let key = apply::add_node(
        &mut graph,
        Some(surface),
        "https://surface.test/".into(),
        Default::default(),
    );
    let before = graph.facets().clone();
    let recorded = seen.lock().unwrap().len();
    for delta in [
        GraphDelta::ReplaySetNodeFacetById {
            node_id: surface,
            facet: SEMANTIC_PROPERTIES.into(),
            value: serde_json::to_value(vec![held.clone()]).unwrap(),
        },
        GraphDelta::SetNodeFacet {
            key,
            facet: SEMANTIC_PROPERTIES.into(),
            value: serde_json::to_value(vec![held.clone()]).unwrap(),
        },
        GraphDelta::ReplayAppendNodePropertyById {
            node_id: surface,
            property: held.clone(),
        },
    ] {
        apply_graph_delta(&mut graph, delta);
        assert_eq!(graph.facets(), &before);
        assert!(reopens(&graph));
    }
    assert_eq!(seen.lock().unwrap().len(), recorded);

    // Surface-only literals, and other facets, keep their historical grammar.
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplaySetNodeFacetById {
            node_id: surface,
            facet: SEMANTIC_PROPERTIES.into(),
            value: serde_json::to_value(vec![NodeProperty::new(
                "urn:audit:other".into(),
                "x".into(),
            )])
            .unwrap(),
        },
    );
    assert_eq!(graph.legacy_node_properties(key).unwrap().len(), 1);
    assert!(reopens(&graph));
}

/// A journal entry the graph refuses is reported, with the graph holding only
/// the entries before it, rather than dropped into a quietly different replay.
#[test]
fn checked_replay_reports_a_refused_literal_handle_instead_of_dropping_it() {
    let mut graph = Graph::new();
    let mut held = NodeProperty::new("urn:replay:literal".into(), "held".into());
    held.statement_id = "shared replay handle".into();
    set_record(&mut graph, "urn:owner", &[held.clone()]);
    let before = serde_json::to_value(graph.to_snapshot()).unwrap();
    let colliding = CapturedDelta::ReplaySetResourceRecordById {
        resource_id: chartulary::resource_id_from_canonical_iri("urn:other").to_string(),
        record: Some(record("urn:other", &[held])),
    };
    let refused = super::replay_captured_deltas_onto_checked(&mut graph, [colliding.clone()])
        .expect_err("a colliding literal handle is refused on replay");
    assert_eq!(refused.position, 0);
    assert!(
        refused.to_string().contains("shared replay handle"),
        "the refusal names the handle: {refused}"
    );
    assert_eq!(
        serde_json::to_value(graph.to_snapshot()).unwrap(),
        before,
        "a refused entry changes nothing"
    );
    assert!(graph.validate_active_resource_assertion_handles().is_ok());
    let mut journal = super::journal::GraphJournal::default();
    journal.record_as(super::Author::person("test"), colliding);
    assert!(journal.snapshot_at_from(&graph, super::Seq(1)).is_err());
}

#[test]
fn installing_facets_that_steal_a_resource_handle_is_refused_and_leaves_the_store() {
    use super::resource_content::ContentError;
    use super::snapshot::ResourceSnapshotError;

    let mut graph = Graph::new();
    let mut held = NodeProperty::new("urn:audit:predicate".into(), "value".into());
    held.statement_id = "owned-handle".into();
    set_record(&mut graph, "urn:audit:first", &[held.clone()]);
    let surface = Uuid::from_u128(7);
    apply::add_node(
        &mut graph,
        Some(surface),
        "https://surface.test/".into(),
        Default::default(),
    );
    let before = graph.facets().clone();
    let revision = graph.visit_revision();

    let carrying = |property: &NodeProperty| {
        let mut facets = NodeFacetStore::new();
        facets
            .set(
                surface,
                chartulary::FacetId::new(SEMANTIC_PROPERTIES),
                serde_json::to_value(vec![property.clone()]).unwrap(),
                &chartulary::AcceptAll,
            )
            .unwrap();
        facets
    };
    let stolen = carrying(&held);
    let refusal = Err(ResourceSnapshotError::InvalidFacets(
        ContentError::HandleCollision("owned-handle".into()),
    ));
    assert_eq!(graph.overlay_facets(stolen.clone()), refusal);
    assert_eq!(graph.install_facets(stolen), refusal);
    assert_eq!(graph.facets(), &before);
    assert_eq!(graph.visit_revision(), revision);
    assert!(reopens(&graph));

    // A handle no Resource holds installs, and replaces rather than merges.
    let other = NodeProperty::new("urn:audit:other".into(), "x".into());
    graph.overlay_facets(carrying(&other)).unwrap();
    let key = graph.get_node_key_by_id(surface).unwrap();
    assert_eq!(graph.legacy_node_properties(key), Some(vec![other]));
    graph.install_facets(NodeFacetStore::new()).unwrap();
    assert_eq!(graph.facets(), &NodeFacetStore::new());
    assert!(graph.visit_revision() > revision);
}
