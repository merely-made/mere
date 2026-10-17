//! Diagnostic assertions of observed discrepancies, with a stale-comment control.
use kernel::graph::{Graph, ContainmentSubKind, EdgeAssertion, GraphStratum, RelationSelector, built_in_relation_stratum, RelationKind};
use kernel::graph::apply::{add_node, apply_graph_delta, GraphDelta};
use uuid::Uuid;

#[test]
fn restored_navigation_overrides_a_successful_url_write_and_leaves_both_index_entries() {
    let mut graph = Graph::new();
    let id = Uuid::from_u128(1);
    let key = add_node(&mut graph, Some(id), "https://audit.test/old".into(), Default::default());
    apply_graph_delta(&mut graph, GraphDelta::NavigateNode { key, url: "https://audit.test/old".into() });
    apply_graph_delta(&mut graph, GraphDelta::SetNodeUrl { key, new_url: "https://audit.test/new".into() });
    assert_eq!(graph.get_node(key).unwrap().url(), "https://audit.test/new");
    let snapshot = graph.to_snapshot();
    assert_eq!(snapshot.nodes[0].url, "https://audit.test/new");
    let restored = Graph::try_from_recorded_snapshot(&snapshot).unwrap();
    let key = restored.get_node_key_by_id(id).unwrap();
    assert_eq!(restored.get_node(key).unwrap().url(), "https://audit.test/old");
    assert_eq!(restored.get_node_by_url("https://audit.test/new").unwrap().1.url(), "https://audit.test/old");
    assert_eq!(restored.get_nodes_by_url("https://audit.test/old"), vec![key]);
}

#[test]
fn live_and_loaded_containment_use_different_stores_and_both_get_persisted() {
    let mut graph = Graph::new();
    let parent = add_node(&mut graph, Some(Uuid::from_u128(2)), "https://audit.test/".into(), Default::default());
    let child = add_node(&mut graph, Some(Uuid::from_u128(3)), "https://audit.test/page".into(), Default::default());
    graph.derive_containment_for(child);
    assert_eq!(built_in_relation_stratum(RelationKind::Containment(ContainmentSubKind::UrlPath)), GraphStratum::Resource);
    assert!(graph.find_edge_key(child, parent).is_none());
    assert!(graph.resource_relations().any(|(_,_,_,p)| p.has_relation(RelationSelector::Containment(ContainmentSubKind::UrlPath))));
    let saved = graph.to_snapshot();
    assert_eq!(saved.resource_edges.len(), 1);
    let restored = Graph::try_from_snapshot(&saved).unwrap();
    assert!(restored.find_edge_key(child, parent).is_some(), "load invents a Surface copy alongside the persisted Resource relation");
    assert_eq!(restored.resource_relations().count(), 1);
    assert!(restored.to_snapshot().edges.iter().any(|edge| edge.containment.is_some()));
}

#[test]
fn checked_snapshot_load_panics_on_unvalidated_navigation_indices() {
    let mut graph = Graph::new();
    let key = add_node(&mut graph, Some(Uuid::from_u128(4)), "https://audit.test".into(), Default::default());
    apply_graph_delta(&mut graph, GraphDelta::NavigateNode { key, url: "https://audit.test".into() });
    let mut value = serde_json::to_value(graph.to_snapshot()).unwrap();
    value["navigation"]["snapshot"]["visits"][0]["entry"] = serde_json::json!(999999);
    let snapshot = serde_json::from_value(value).unwrap();
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| Graph::try_from_snapshot(&snapshot))).is_err());
}

#[test]
fn collection_membership_restore_is_a_positive_control_for_stale_test_comment() {
    let mut graph = Graph::new();
    let parent = add_node(&mut graph, Some(Uuid::from_u128(5)), "https://parent.test".into(), Default::default());
    let child = add_node(&mut graph, Some(Uuid::from_u128(6)), "https://child.test".into(), Default::default());
    apply_graph_delta(&mut graph, GraphDelta::AssertRelation { from: parent, to: child, assertion: EdgeAssertion::Containment { sub_kind: ContainmentSubKind::CollectionMember }, asserter_iri: "urn:audit".into() });
    let restored = Graph::try_from_snapshot(&graph.to_snapshot()).unwrap();
    assert!(restored.get_edge(restored.find_edge_key(parent, child).unwrap()).unwrap().has_relation(RelationSelector::Containment(ContainmentSubKind::CollectionMember)));
}
