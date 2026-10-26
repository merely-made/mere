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
