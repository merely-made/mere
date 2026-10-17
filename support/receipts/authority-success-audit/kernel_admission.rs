//! Diagnostic reproductions of divergent admission/replacement paths.
use kernel::graph::{Graph, CapturedDelta};
use kernel::graph::apply::{apply_graph_delta, GraphDelta};
use kernel::persistence::{PersistedResourceRecord, PersistedResourceFacet};

#[test]
fn replay_record_admission_accepts_literal_ownership_that_checked_load_rejects() {
    let mut graph = Graph::new();
    let mut property = kernel::types::NodeProperty::new("urn:audit:predicate".into(), "value".into());
    property.statement_id = "same-owned-handle".into();
    for iri in ["urn:audit:first", "urn:audit:second"] {
        let resource_id = chartulary::resource_id_from_canonical_iri(iri);
        apply_graph_delta(&mut graph, GraphDelta::ReplaySetResourceRecordById { resource_id, record: Some(PersistedResourceRecord { canonical_iri: iri.into(), facets: vec![PersistedResourceFacet { facet: "semantic.properties".into(), value_json: serde_json::to_string(&vec![property.clone()]).unwrap() }] }) });
        assert!(graph.resource(resource_id).is_some());
    }
    assert_eq!(graph.resource_nodes().count(), 2);
    assert!(graph.validate_active_resource_assertion_handles().is_err());
    assert!(Graph::try_from_recorded_snapshot(&graph.to_snapshot()).is_err());
}

#[test]
fn resident_snapshot_replacement_drops_the_live_journal_recorder() {
    let mut graph = Graph::new();
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::<CapturedDelta>::new()));
    let sink = seen.clone();
    graph.set_recorder(Some(std::sync::Arc::new(move |delta| sink.lock().unwrap().push(delta.clone()))));
    assert!(graph.is_recording());
    let supplied = Graph::new();
    graph.replace_loaded_recorded_snapshot(&supplied.to_snapshot(), supplied.facets(), Default::default()).unwrap();
    assert!(!graph.is_recording());
    kernel::graph::apply::add_node(&mut graph, Some(uuid::Uuid::from_u128(888)), "https://audit.test".into(), Default::default());
    assert!(seen.lock().unwrap().is_empty(), "subsequent edit never reaches the previous recorder");
}
