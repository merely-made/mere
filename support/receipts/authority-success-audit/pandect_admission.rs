//! Diagnostic: successful public session write leaves an unreopenable session.
use kernel::graph::{Graph, apply::{add_node, apply_graph_delta, GraphDelta}};
use pandect::{Author, CapturedDelta, MereSessions};
use uuid::Uuid;

#[test]
fn successful_session_write_persists_a_handle_collision_which_open_rejects() {
    pollster::block_on(async {
        let mut graph = Graph::new();
        let first = add_node(&mut graph, Some(Uuid::from_u128(201)), "https://first.test/".into(), Default::default());
        add_node(&mut graph, Some(Uuid::from_u128(202)), "https://second.test/".into(), Default::default());
        let mut property = kernel::types::NodeProperty::new("urn:audit:predicate".into(), "held".into());
        property.statement_id = "audit-owned-resource-handle".into();
        apply_graph_delta(&mut graph, GraphDelta::AppendNodeProperty { key: first, property: property.clone() });
        graph.validate_active_resource_assertion_handles().expect("valid baseline control");
        let sessions = MereSessions::new(muniment::MemoryBackend::new());
        let author = Author::person("audit");
        let mut session = sessions.begin_recorded(author.clone(), Some(graph));
        session.flush(std::time::SystemTime::UNIX_EPOCH).await.unwrap();
        assert!(sessions.open(session.id()).await.is_ok());
        session.apply(author, vec![CapturedDelta::ReplaySetNodeFacetById {
            node_id: Uuid::from_u128(202).to_string(),
            facet: kernel::graph::node_facets::SEMANTIC_PROPERTIES.into(),
            value_json: serde_json::to_string(&vec![property]).unwrap(),
        }]).await.expect("public write reports stored success");
        assert!(session.graph().validate_active_resource_assertion_handles().is_err());
        assert!(session.graph_at(session.journal().live_cursor()).is_none());
        assert!(matches!(sessions.open(session.id()).await, Err(pandect::SessionError::Corrupt(_))), "acknowledged stored session no longer opens");
    });
}
