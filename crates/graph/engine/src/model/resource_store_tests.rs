// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::model::SemanticStatementSpec;

fn put(store: &mut ResourceStore, resource: &ResourceNode, note: &str) -> Option<ResourceDelta> {
    let record = PersistedResourceRecord {
        canonical_iri: resource.canonical_iri().to_owned(),
        facets: vec![super::super::PersistedResourceFacet {
            facet: "test.note".into(),
            value_json: serde_json::json!(note).to_string(),
        }],
    };
    store.put_record(resource.id(), record)
}

fn claim(predicate: &str) -> EdgePayload {
    let mut payload = EdgePayload::new();
    payload.assert_semantic_statement(SemanticStatementSpec {
        predicate: predicate.into(),
        provenance_iri: Some("urn:test:asserter".into()),
        ..Default::default()
    });
    payload
}

#[test]
fn a_record_write_returns_the_state_it_made_once() {
    let mut store = ResourceStore::new();
    let resource = ResourceNode::for_term("urn:test:a");
    let id = resource.id();
    let delta = put(&mut store, &resource, "one").unwrap();
    assert_eq!(delta, store.record_delta(id));
    let ResourceDelta::ReplaySetResourceRecordById { record, .. } = &delta else {
        panic!("a record delta: {delta:?}");
    };
    assert_eq!(record.as_ref().unwrap().facets[0].value_json, "\"one\"");
    // Unchanged: no delta. Control: a changed facet makes one.
    assert_eq!(put(&mut store, &resource, "one"), None);
    assert!(put(&mut store, &resource, "two").is_some());
}

#[test]
fn a_resource_an_edge_touches_is_not_removed() {
    let mut store = ResourceStore::new();
    let (a, b) = (
        ResourceNode::for_term("urn:test:a"),
        ResourceNode::for_term("urn:test:b"),
    );
    let (ia, ib) = (a.id(), b.id());
    put(&mut store, &a, "a");
    put(&mut store, &b, "b");
    store
        .replace_edges(ia, ib, vec![claim("urn:test:p")])
        .unwrap();
    assert_eq!(store.remove_record(ia), None);
    assert!(store.resource(ia).is_some());
    // Control: once the pair is empty the resource goes, and the delta says so.
    store.replace_edges(ia, ib, Vec::new()).unwrap();
    let removed = store.remove_record(ia).unwrap();
    assert_eq!(
        removed,
        ResourceDelta::ReplaySetResourceRecordById {
            resource_id: ia.to_string(),
            record: None
        }
    );
    assert!(store.resource(ia).is_none());
}

#[test]
fn an_edge_write_returns_the_pair_in_order_once() {
    let mut store = ResourceStore::new();
    let (a, b) = (
        ResourceNode::for_term("urn:test:a"),
        ResourceNode::for_term("urn:test:b"),
    );
    let (ia, ib) = (a.id(), b.id());
    assert_eq!(
        store.replace_edges(ia, ib, vec![claim("urn:test:p")]),
        None,
        "no endpoints"
    );
    put(&mut store, &a, "a");
    put(&mut store, &b, "b");
    let payloads = vec![claim("urn:test:p"), claim("urn:test:q")];
    let delta = store.replace_edges(ia, ib, payloads.clone()).unwrap();
    assert_eq!(delta, store.pair_delta(ia, ib));
    let expected: Vec<_> = payloads
        .iter()
        .map(|payload| persisted_edge_for_ids(ia, ib, payload))
        .collect();
    assert_eq!(store.persisted_edges_between(ia, ib), expected);
    assert_eq!(store.replace_edges(ia, ib, payloads), None);
}
