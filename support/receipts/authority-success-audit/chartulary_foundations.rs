//! Diagnostic reproductions: assertions describe observed defects, not desired semantics.
use chartulary::{Author, Container, EditSpec, Graph, GraphLog, LogId, Relation, RelationClass};
use chartulary::stemma::{EntryPrivacy, Stemma, TransitionKind};
use muniment::{Backend, JsonSlots, MemoryBackend};

fn who() -> Author { Author::new("audit") }
fn relation() -> Relation { Relation::new(RelationClass::open("urn:audit:related")) }

#[test]
fn snapshot_can_override_the_authoritative_durable_journal() {
    pollster::block_on(async {
        let slots = JsonSlots::new(MemoryBackend::new());
        let mut graph = GraphLog::<Container, Relation>::new();
        graph.insert_node(&who(), Container::new("durable"));
        graph.save_log(&slots, "log").await.unwrap();
        graph.insert_node(&who(), Container::new("not-in-durable-log"));
        graph.snapshot(&slots, "snap").await.unwrap();
        let full = GraphLog::<Container, Relation>::load_full(&slots, "log").await.unwrap();
        let checkpointed = GraphLog::<Container, Relation>::load_checkpointed(&slots, "log", "snap").await.unwrap();
        assert_eq!(full.graph().node_count(), 1);
        assert_eq!(checkpointed.graph().node_count(), 2);
        assert_eq!(checkpointed.revision(), full.revision());
    });
}

#[test]
fn deleting_a_creator_breaks_snapshotting_of_the_surviving_child() {
    let mut memory = Stemma::<String, String, String, ()>::new();
    let creator = memory.ensure_owner("creator".into(), None);
    let entry = memory.resolve_or_create_entry("a".into(), "a".into(), 1, EntryPrivacy::LocalOnly);
    memory.visit_entry(creator, entry, (), TransitionKind::UrlTyped, 1).unwrap();
    let child = memory.ensure_owner("child".into(), Some(creator));
    memory.visit_entry(child, entry, (), TransitionKind::TabSpawn, 2).unwrap();
    memory.delete_owner(creator).unwrap();
    assert!(memory.owner(child).is_some());
    assert!(std::panic::catch_unwind(|| memory.to_snapshot()).is_err());
}

#[test]
fn removing_one_bearer_archives_the_other_bearers_live_graph() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let slots = JsonSlots::new(backend.clone());
        let nested_id = LogId::new("shared-child");
        let mut nested = GraphLog::<Container, Relation>::with_id(nested_id.clone());
        nested.insert_node(&who(), Container::new("content"));
        nested.save_log(&slots, &chartulary::nested::log_slot(&nested_id)).await.unwrap();
        let mut parent = GraphLog::<Container, Relation>::new();
        parent.insert_node(&who(), Container::new("first").with_nested(nested_id.clone()));
        parent.insert_node(&who(), Container::new("second").with_nested(nested_id.clone()));
        parent.remove_bearing_node(&slots, &who(), &"first".into()).await.unwrap();
        assert!(parent.graph().get(&"second".into()).is_some());
        assert!(backend.get(&chartulary::nested::log_slot(&nested_id)).await.unwrap().is_none());
        parent.recover_archived_bearings(&slots, &who()).await.unwrap();
        assert!(parent.graph().get(&"second".into()).is_none());
    });
}

#[test]
fn batch_preflight_accepts_a_disconnect_of_an_edge_removed_earlier_in_the_batch() {
    let mut graph = GraphLog::<Container, Relation>::new();
    graph.insert_node(&who(), Container::new("a"));
    graph.insert_node(&who(), Container::new("b"));
    let edge = graph.connect(&who(), &"a".into(), &"b".into(), relation()).unwrap();
    let result = graph.commit_batch(who(), graph.revision(), vec![EditSpec::RemoveNode("a".into()), EditSpec::Disconnect(edge)]);
    assert!(result.is_ok());
    assert_eq!(graph.log().entries().last().unwrap().edits.len(), 2);
    assert_eq!(graph.graph().edge_count(), 0);
}

#[test]
fn nquads_export_leaves_a_literal_carriage_return_unescaped() {
    let mut graph = Graph::<Container, Relation>::new();
    graph.insert(Container::new("a").with_address("https://audit.test/a").with_title("one\rtwo"));
    let output = chartulary::rdf::to_nquads(&graph);
    assert!(output.contains("\"one\rtwo\""));
}

#[test]
fn deserializing_duplicate_node_ids_preserves_two_nodes_but_indexes_only_one() {
    let mut graph = Graph::<Container, Relation>::new();
    graph.insert(Container::new("a"));
    graph.insert(Container::new("b"));
    let mut serialized = serde_json::to_value(&graph).unwrap();
    serialized["nodes"][1]["id"] = serde_json::json!("a");
    let loaded: Graph<Container, Relation> = serde_json::from_value(serialized).unwrap();
    assert_eq!(loaded.node_count(), 2);
    assert_eq!(loaded.nodes().filter(|(_, n)| n.id == "a").count(), 2);
    assert!(loaded.key_of(&"b".into()).is_none());
}
