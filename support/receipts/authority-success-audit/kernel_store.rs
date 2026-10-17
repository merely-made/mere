//! Diagnostic assertions of observed save/load behavior, not correctness tests.
use kernel::graph::{Graph, apply::{add_node, apply_graph_delta, GraphDelta}};
use uuid::Uuid;

#[test]
fn advertised_graph_save_load_succeeds_but_drops_surface_facets() {
    let directory = std::env::temp_dir().join(format!("mere-audit-kernel-store-{}", std::process::id()));
    std::fs::create_dir(&directory).expect("fresh bounded diagnostic data directory");
    let mut graph = Graph::new();
    let id = Uuid::from_u128(71);
    let key = add_node(&mut graph, Some(id), "https://audit.test/facets".into(), Default::default());
    apply_graph_delta(&mut graph, GraphDelta::SetNodePinned { key, is_pinned: true });
    apply_graph_delta(&mut graph, GraphDelta::SetNodeFacet {
        key, facet: "audit.metadata".into(), value: serde_json::json!({"important": true}),
    });
    assert_eq!(graph.node_is_pinned(key), Some(true));
    assert!(!graph.facets().is_empty());
    kernel::store::save_graph(&directory, &graph).expect("reports successful graph save");
    let loaded = kernel::store::load_graph(&directory).expect("reports successful graph load").unwrap();
    let restored_key = loaded.get_node_key_by_id(id).expect("control: node identity survives");
    assert_eq!(loaded.get_node(restored_key).unwrap().url(), "https://audit.test/facets");
    assert_ne!(loaded.node_is_pinned(restored_key), Some(true));
    assert!(!serde_json::to_string(loaded.facets()).unwrap().contains("audit.metadata"), "supplied optional metadata was discarded");
    std::fs::remove_file(directory.join(kernel::store::GRAPH_FILE)).unwrap();
    std::fs::remove_dir(directory).unwrap();
}
