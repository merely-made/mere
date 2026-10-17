//! Diagnostic reproductions: assertions describe observed defects.
use kernel::graph::Graph;
use kernel::graph::apply::{GraphDelta, apply_graph_delta, add_node};
use uuid::Uuid;

#[test]
fn replacing_a_navigation_parents_history_panics() {
    let mut graph = Graph::new();
    let parent = add_node(&mut graph, Some(Uuid::from_u128(1)), "https://audit.test/parent".into(), Default::default());
    apply_graph_delta(&mut graph, GraphDelta::NavigateNode { key: parent, url: "https://audit.test/parent".into() });
    let child = add_node(&mut graph, Some(Uuid::from_u128(2)), "https://audit.test/child".into(), Default::default());
    apply_graph_delta(&mut graph, GraphDelta::BranchHistory { child, parent });
    apply_graph_delta(&mut graph, GraphDelta::NavigateNode { key: child, url: "https://audit.test/child".into() });
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_graph_delta(&mut graph, GraphDelta::UpdateNodeHistory { key: parent, entries: vec!["https://audit.test/replacement".into()], current_index: 0 });
    }));
    assert!(result.is_err());
    assert!(graph.get_node(parent).is_some());
    assert!(graph.get_node(child).is_some());
}

#[test]
fn ordinary_parent_deletion_preserves_navigation_owner_without_panicking() {
    let mut graph = Graph::new();
    let parent = add_node(&mut graph, Some(Uuid::from_u128(11)), "https://audit.test/parent".into(), Default::default());
    apply_graph_delta(&mut graph, GraphDelta::NavigateNode { key: parent, url: "https://audit.test/parent".into() });
    let child = add_node(&mut graph, Some(Uuid::from_u128(12)), "https://audit.test/child".into(), Default::default());
    apply_graph_delta(&mut graph, GraphDelta::BranchHistory { child, parent });
    apply_graph_delta(&mut graph, GraphDelta::NavigateNode { key: child, url: "https://audit.test/child".into() });
    apply_graph_delta(&mut graph, GraphDelta::RemoveNode { key: parent });
    assert!(graph.get_node(parent).is_none());
    assert!(graph.get_node(child).is_some());
    assert_eq!(graph.table_stats().history_owner_count, 2);
}
