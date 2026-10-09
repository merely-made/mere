// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::graph::apply::add_node;
use crate::persistence::PersistedResourceFacet;
use serde_json::json;
use std::sync::{Arc, Mutex};

fn source() -> (Graph, Uuid, Uuid, String) {
    let mut graph = Graph::new();
    let a = add_node(
        &mut graph,
        None,
        "https://example.test/a".into(),
        Default::default(),
    );
    let b = add_node(
        &mut graph,
        None,
        "https://example.test/b".into(),
        Default::default(),
    );
    let a = graph.shown_resource_id(a).unwrap();
    let b = graph.shown_resource_id(b).unwrap();
    let (_, claim) = graph
        .try_assert_semantic_statement_by_resource_ids(
            a,
            b,
            SemanticStatementSpec {
                predicate: "urn:test:claim".into(),
                label: Some("exact label".into()),
                graph_scope: crate::types::GraphScope::Custom("urn:test:scope".into()),
                provenance_iri: Some("urn:test:author".into()),
                asserted_at_ms: Some(41),
                ..Default::default()
            },
        )
        .unwrap();
    (graph, a, b, claim.statement_id)
}

#[test]
fn frozen_resource_bears_exact_immutable_nested_selection_and_surfaces_show_it() {
    let (mut graph, a, b, claim) = source();
    let revision = graph.revision();
    let mut replayed = graph.clone();
    let captures = Arc::new(Mutex::new(Vec::new()));
    let sink = captures.clone();
    graph.set_recorder(Some(Arc::new(move |d| {
        sink.lock().unwrap().push(d.clone())
    })));
    let spec = json!({"query":"SELECT ?item WHERE { ?item ?p ?o }"});
    let owner = graph
        .freeze_resource_selection(spec.clone(), vec![b, a, a])
        .unwrap();
    let frozen = graph
        .resource(owner)
        .unwrap()
        .nested_selection()
        .unwrap()
        .clone();
    assert_eq!(frozen.spec(), &spec);
    assert_eq!(frozen.revision(), revision);
    assert_eq!(frozen.members().len(), 2);
    assert!(frozen.members().contains(&a));
    assert_eq!(frozen.statements().len(), 1);
    let statement = &frozen.statements()[0].semantic.as_ref().unwrap().statements[0];
    assert_eq!(statement.statement_id, claim);
    assert_eq!(statement.label.as_deref(), Some("exact label"));
    assert_eq!(statement.asserted_at_ms, Some(41));
    assert_eq!(statement.provenance_iri.as_deref(), Some("urn:test:author"));
    assert_eq!(
        statement.graph_scope,
        crate::types::GraphScope::Custom("urn:test:scope".into())
    );
    assert!(graph.retract_assertion(&claim));
    assert_eq!(
        graph.resource(owner).unwrap().nested_selection(),
        Some(&frozen)
    );
    let iri = graph.resource(owner).unwrap().canonical_iri().to_string();
    let surface = add_node(&mut graph, None, iri, Default::default());
    assert_eq!(graph.shown_resource_id(surface), Some(owner));
    let reopened = Graph::try_from_snapshot(&graph.to_snapshot()).unwrap();
    assert_eq!(
        reopened.resource(owner).unwrap().nested_selection(),
        Some(&frozen)
    );
    capture::replay_captured_deltas_onto(&mut replayed, captures.lock().unwrap().clone());
    assert_eq!(
        replayed.resource(owner).unwrap().nested_selection(),
        Some(&frozen)
    );
}

#[test]
fn transferred_owner_opens_missing_refs_with_coverage_and_refuses_payload_replacement() {
    let (mut graph, a, b, _) = source();
    let owner = graph
        .freeze_resource_selection(json!({"kind":"saved"}), vec![a, b])
        .unwrap();
    let record = graph.resource_record(owner).unwrap();
    let mut receiver = Graph::new();
    assert!(receiver.set_resource_record(owner, Some(record.clone())));
    assert_eq!(
        receiver.resource_nodes().count(),
        1,
        "refs do not invent member payloads"
    );
    let opened = receiver.open_frozen_selection(owner).unwrap();
    assert!(
        opened
            .coverage
            .limits
            .iter()
            .any(|l| l.layer == CoverageLayer::Possession && l.count == Some(2))
    );
    let mut limit = CoverageLimit::new(CoverageLayer::Residency, "held but not loaded");
    limit.resources = vec![a, b];
    receiver.set_known_coverage(CoverageNote {
        limits: vec![limit],
    });
    let opened = receiver.open_frozen_selection(owner).unwrap();
    assert!(
        !opened
            .coverage
            .limits
            .iter()
            .any(|l| l.layer == CoverageLayer::Possession)
    );
    assert!(
        opened
            .coverage
            .limits
            .iter()
            .any(|l| l.layer == CoverageLayer::Residency)
    );
    let before = receiver.resource_record(owner).unwrap();
    let revision = receiver.revision();
    let mut changed = record.clone();
    let facet = changed
        .facets
        .iter_mut()
        .find(|f| f.facet == frozen_selection::FROZEN_SELECTION)
        .unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&facet.value_json).unwrap();
    value["revision"] = json!(999);
    facet.value_json = value.to_string();
    assert!(!receiver.set_resource_record(owner, Some(changed)));
    let mut cleared = record.clone();
    cleared
        .facets
        .retain(|f| f.facet != frozen_selection::FROZEN_SELECTION);
    assert!(!receiver.set_resource_record(owner, Some(cleared)));
    assert_eq!(receiver.resource_record(owner), Some(before));
    assert_eq!(receiver.revision(), revision);
    // Ordinary Resource annotations may grow without changing the nested value.
    let mut annotated = record;
    annotated.facets.push(PersistedResourceFacet {
        facet: "test.annotation".into(),
        value_json: "\"kept\"".into(),
    });
    assert!(receiver.set_resource_record(owner, Some(annotated)));
    assert_eq!(
        receiver.resource(owner).unwrap().nested_selection(),
        graph.resource(owner).unwrap().nested_selection()
    );
}

#[test]
fn invalid_frozen_member_refusal_is_atomic_and_records_nothing() {
    let (mut graph, _, _, _) = source();
    let revision = graph.revision();
    let count = graph.resource_nodes().count();
    assert!(
        graph
            .freeze_resource_selection(json!({}), vec![Uuid::nil()])
            .is_err()
    );
    assert_eq!(graph.revision(), revision);
    assert_eq!(graph.resource_nodes().count(), count);
}

#[test]
fn malformed_nested_records_refuse_before_admission_and_nonce_reuse_is_atomic() {
    let (mut graph, a, b, _) = source();
    let nonce = Uuid::from_u128(17);
    let owner = graph
        .freeze_resource_selection_with_nonce(json!({}), vec![a, b], nonce)
        .unwrap();
    let revision = graph.revision();
    assert!(
        graph
            .freeze_resource_selection_with_nonce(json!({}), vec![a, b], nonce)
            .is_err()
    );
    assert_eq!(graph.revision(), revision);
    let record = graph.resource_record(owner).unwrap();
    for mutate in 0..6 {
        let mut invalid = record.clone();
        let facet = invalid
            .facets
            .iter_mut()
            .find(|f| f.facet == frozen_selection::FROZEN_SELECTION)
            .unwrap();
        let mut value: serde_json::Value = serde_json::from_str(&facet.value_json).unwrap();
        match mutate {
            0 => value["version"] = json!(2),
            1 => value["members"] = json!([a, a]),
            2 => value["statements"][0]["to_node_id"] = json!(Uuid::nil().to_string()),
            3 => value["statements"][0]["traversal"] = json!({"records":[]}),
            4 => value["future_payload"] = json!("must not be silently dropped"),
            _ => {
                value["statements"][0]["semantic"]["statements"][0]["future_payload"] =
                    json!("nested unknown must refuse")
            },
        }
        facet.value_json = value.to_string();
        let mut receiver = Graph::new();
        assert!(
            !receiver.set_resource_record(owner, Some(invalid)),
            "mutation {mutate}"
        );
        assert_eq!(receiver.resource_nodes().count(), 0);
    }
}

#[test]
fn immutable_frozen_carrier_refuses_wire_changes_hidden_by_float_decoding() {
    let (mut graph, a, b, _) = source();
    graph
        .try_assert_semantic_statement_by_resource_ids(
            a,
            b,
            SemanticStatementSpec {
                predicate: predicate_iri(SemanticSubKind::AgentDerived).into(),
                recognized_sub_kind: Some(SemanticSubKind::AgentDerived),
                ..Default::default()
            },
        )
        .unwrap();
    let owner = graph
        .freeze_resource_selection(json!({}), vec![a, b])
        .unwrap();
    let mut record = graph.resource_record(owner).unwrap();
    let mut receiver = Graph::new();
    assert!(receiver.set_resource_record(owner, Some(record.clone())));
    let before = receiver.resource_record(owner).unwrap();
    let revision = receiver.revision();
    let facet = record
        .facets
        .iter_mut()
        .find(|f| f.facet == frozen_selection::FROZEN_SELECTION)
        .unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&facet.value_json).unwrap();
    value["statements"][0]["semantic"]["agent_decay_progress"] = json!(1e-50_f64);
    facet.value_json = value.to_string();
    assert!(
        !receiver.set_resource_record(owner, Some(record)),
        "immutable carrier must preserve raw nested value, even when f32 decoding rounds it away"
    );
    assert_eq!(receiver.resource_record(owner), Some(before));
    assert_eq!(receiver.revision(), revision);
}

#[test]
fn frozen_selection_copies_parallel_rows_self_relations_and_opaque_handles_exactly() {
    let (mut graph, a, b, original) = source();
    let first = graph.persisted_resource_edges_between(a, b)[0].clone();
    let mut second = first.clone();
    let statement = &mut second.semantic.as_mut().unwrap().statements[0];
    statement.statement_id = String::new();
    statement.label = Some("parallel variant".into());
    statement.provenance_iri = Some("urn:test:other-author".into());
    statement.asserted_at_ms = Some(0);
    assert!(graph.set_resource_edges_between(a, b, &[first, second]));
    graph
        .try_assert_semantic_statement_by_resource_ids(
            a,
            a,
            SemanticStatementSpec {
                predicate: "urn:test:self".into(),
                ..Default::default()
            },
        )
        .unwrap();
    let mut self_row = graph.persisted_resource_edges_between(a, a)[0].clone();
    self_row.semantic.as_mut().unwrap().statements[0].statement_id = "opaque\nself handle".into();
    assert!(graph.set_resource_edges_between(a, a, &[self_row]));
    let expected = graph.to_snapshot().resource_edges;
    let owner = graph
        .freeze_resource_selection(json!({}), vec![a, b])
        .unwrap();
    let frozen = graph
        .resource(owner)
        .unwrap()
        .nested_selection()
        .unwrap()
        .clone();
    fn rows(edges: &[crate::persistence::PersistedEdge]) -> Vec<String> {
        let mut rows: Vec<_> = edges
            .iter()
            .map(|e| serde_json::to_string(e).unwrap())
            .collect();
        rows.sort();
        rows
    }
    assert_eq!(rows(frozen.statements()), rows(&expected));
    assert_eq!(frozen.statements().len(), 3);
    let handles: std::collections::BTreeSet<_> = frozen
        .statements()
        .iter()
        .flat_map(|e| &e.semantic.as_ref().unwrap().statements)
        .map(|s| s.statement_id.as_str())
        .collect();
    assert!(handles.contains(original.as_str()));
    assert!(handles.contains(""));
    assert!(handles.contains("opaque\nself handle"));
    assert!(graph.retract_assertion(&original));
    assert_eq!(
        graph.resource(owner).unwrap().nested_selection(),
        Some(&frozen)
    );
}

#[test]
fn frozen_wire_accepts_known_default_metadata_but_retained_owner_cannot_drop_it() {
    let (mut graph, a, b, _) = source();
    let owner = graph
        .freeze_resource_selection(json!({}), vec![a, b])
        .unwrap();
    let mut record = graph.resource_record(owner).unwrap();
    let facet = record
        .facets
        .iter_mut()
        .find(|f| f.facet == frozen_selection::FROZEN_SELECTION)
        .unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&facet.value_json).unwrap();
    value["statements"][0]["semantic"]["statements"][0]["recognized_sub_kind"] =
        serde_json::Value::Null;
    facet.value_json = value.to_string();
    let mut receiver = Graph::new();
    assert!(
        receiver.set_resource_record(owner, Some(record.clone())),
        "known null metadata remains supported"
    );
    let before = receiver.resource_record(owner).unwrap();
    value["statements"][0]["semantic"]["statements"][0]
        .as_object_mut()
        .unwrap()
        .remove("recognized_sub_kind");
    record
        .facets
        .iter_mut()
        .find(|f| f.facet == frozen_selection::FROZEN_SELECTION)
        .unwrap()
        .value_json = value.to_string();
    assert!(
        !receiver.set_resource_record(owner, Some(record)),
        "the retained immutable carrier cannot lose an explicit field"
    );
    assert_eq!(receiver.resource_record(owner), Some(before));
}
