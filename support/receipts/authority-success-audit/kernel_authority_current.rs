//! Diagnostic assertions of the observed defects, not desired behavior.
use chartulary::{Classified, Predicated};
use kernel::graph::{Graph, CapturedDelta, SemanticData, SemanticStatementSpec, replay_captured_deltas_onto};
use kernel::graph::apply::{GraphDelta, apply_graph_delta, add_node};
use uuid::Uuid;

#[test]
fn generic_projection_exposes_a_withdrawn_statement_as_current_predicate() {
    let mut data = SemanticData::default();
    let assertion = data.assert_statement(SemanticStatementSpec { predicate: "urn:audit:withdrawn".into(), ..Default::default() });
    assert!(data.withdraw_statement(&assertion.statement_id, kernel::types::AssertionWithdrawal { at_ms: Some(1), by: None }));
    let mut payload = kernel::graph::EdgePayload::new();
    payload.assertions = data;
    assert!(payload.held_semantic_statements().next().is_none());
    assert!(!payload.has_held_topology());
    assert_eq!(Predicated::predicate(&payload), Some("urn:audit:withdrawn"));
    assert!(Classified::class(&payload).predicate().is_some());
}

#[test]
fn adding_the_same_id_at_a_new_url_leaves_the_old_url_index_live() {
    let mut graph = Graph::new();
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let sink = seen.clone();
    graph.set_recorder(Some(std::sync::Arc::new(move |delta| sink.lock().unwrap().push(delta.clone()))));
    let id = Uuid::from_u128(101);
    let first = add_node(&mut graph, Some(id), "https://audit.test/old".into(), Default::default());
    let second = add_node(&mut graph, Some(id), "https://audit.test/new".into(), Default::default());
    assert_eq!(first, second);
    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.get_node(first).unwrap().url(), "https://audit.test/new");
    assert_eq!(graph.get_nodes_by_url("https://audit.test/old").len(), 1);
    assert_eq!(graph.get_nodes_by_url("https://audit.test/new").len(), 1);
    let replayed = kernel::graph::replay_captured_deltas(seen.lock().unwrap().clone());
    assert_eq!(replayed.get_node_by_id(id).unwrap().1.url(), "https://audit.test/old", "the captured insert-if-missing cannot replay the live replacement");
    apply_graph_delta(&mut graph, GraphDelta::RemoveNode { key: first });
    let replacement = add_node(&mut graph, Some(Uuid::from_u128(102)), "https://audit.test/unrelated".into(), Default::default());
    assert_eq!(replacement, first, "arena key is recycled");
    assert_eq!(graph.get_nodes_by_url("https://audit.test/old").len(), 1, "old address now resolves to unrelated node");
}

#[test]
fn writing_one_nodes_import_provenance_erases_another_records_suppression() {
    use kernel::types::{ImportRecord, ImportRecordMembership, NodeImportProvenance};
    let mut graph = Graph::new();
    let hidden_id = Uuid::from_u128(301);
    let visible_id = Uuid::from_u128(302);
    add_node(&mut graph, Some(hidden_id), "https://audit.test/hidden".into(), Default::default());
    let visible = add_node(&mut graph, Some(visible_id), "https://audit.test/visible".into(), Default::default());
    apply_graph_delta(&mut graph, GraphDelta::SetImportRecords { import_records: vec![ImportRecord { record_id: "retained-suppressed".into(), source_id: "old-source".into(), source_label: "Old source".into(), imported_at_secs: 1, memberships: vec![ImportRecordMembership { node_id: hidden_id.to_string(), suppressed: true }] }] });
    assert_eq!(graph.import_records().len(), 1);
    apply_graph_delta(&mut graph, GraphDelta::SetNodeImportProvenance { key: visible, import_provenance: vec![NodeImportProvenance { source_id: "new-source".into(), source_label: "New source".into() }] });
    assert!(graph.import_records().iter().all(|record| record.record_id != "retained-suppressed"));
    assert_eq!(graph.import_records().len(), 1);
}

#[test]
fn malformed_facet_record_is_silently_skipped_during_replay() {
    let mut graph = Graph::new();
    let id = Uuid::from_u128(201);
    let key = add_node(&mut graph, Some(id), "https://audit.test".into(), Default::default());
    let revision = graph.revision();
    replay_captured_deltas_onto(&mut graph, [CapturedDelta::ReplaySetNodeFacetById { node_id: id.to_string(), facet: "audit/value".into(), value_json: "{".into() }]);
    assert_eq!(graph.revision(), revision);
    assert!(graph.facets().get(&id, &chartulary::FacetId::new("audit/value")).is_none());
    assert!(graph.get_node(key).is_some());
}
