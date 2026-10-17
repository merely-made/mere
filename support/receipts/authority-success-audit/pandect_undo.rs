//! Diagnostic: a normal session undo reaches the dangling navigation creator.
use kernel::graph::{Graph, apply::{add_node, apply_graph_delta, GraphDelta}};
use pandect::{Author, ChangeKind, GraphId, GraphSession, GraphSessionManifest, SessionId};
use uuid::Uuid;

#[test]
fn undoing_parent_navigation_panics_when_a_child_anchors_its_history() {
    let mut graph = Graph::new();
    let parent = add_node(&mut graph, Some(Uuid::from_u128(81)), "https://audit.test/parent".into(), Default::default());
    apply_graph_delta(&mut graph, GraphDelta::NavigateNode { key: parent, url: "https://audit.test/parent".into() });
    let child = add_node(&mut graph, Some(Uuid::from_u128(82)), "https://audit.test/child".into(), Default::default());
    apply_graph_delta(&mut graph, GraphDelta::BranchHistory { child, parent });
    apply_graph_delta(&mut graph, GraphDelta::NavigateNode { key: child, url: "https://audit.test/child".into() });
    let manifest = GraphSessionManifest::new(SessionId(Uuid::from_u128(83)), GraphId(Uuid::from_u128(84)));
    let author = Author::person("audit");
    let mut session = GraphSession::new_recorded(muniment::MemoryBackend::new(), manifest, Some(graph), author.clone(), ChangeKind::Minted);
    // Shared binding rows have random HashMap order on hydration. The session
    // may refuse an otherwise valid navigation when live and replay order differ.
    let mut accepted = false;
    for _ in 0..64 {
        if session.edit_now(author.clone(), |graph| {
            apply_graph_delta(graph, GraphDelta::NavigateNode { key: parent, url: "https://audit.test/later".into() });
        }).is_ok() { accepted = true; break; }
    }
    assert!(accepted, "at least one candidate navigation passes the replay check");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| session.undo_now(author)));
    assert!(result.is_err(), "the advertised fallible undo API panics");
    assert_eq!(session.graph().get_node(parent).unwrap().url(), "https://audit.test/later", "control: candidate isolation preserves the live graph");
    assert!(session.graph().get_node(child).is_some());
}

#[test]
fn a_recorded_branch_reopens_with_invented_surface_containment() {
    pollster::block_on(async {
        let mut graph = Graph::new();
        let _parent = add_node(&mut graph, Some(Uuid::from_u128(91)), "https://audit.test/".into(), Default::default());
        let child = add_node(&mut graph, Some(Uuid::from_u128(92)), "https://audit.test/child".into(), Default::default());
        graph.derive_containment_for(child);
        assert_eq!(graph.edge_count(), 0);
        assert!(graph.resource_relations().count() > 0);
        let sessions = pandect::MereSessions::new(muniment::MemoryBackend::new());
        let author = Author::person("audit");
        let mut parent = sessions.begin_recorded(author.clone(), Some(graph));
        parent.flush(std::time::SystemTime::UNIX_EPOCH).await.unwrap();
        let staged = sessions.begin_branch_at(&parent, parent.journal().live_cursor(), author.clone()).unwrap();
        assert_eq!(staged.graph().edge_count(), 0);
        assert!(staged.placement().is_none(), "copy path loses recorded profile");
        let branch = sessions.branch_at(&parent, parent.journal().live_cursor(), author).await.unwrap();
        let restored = sessions.open(branch.session_id).await.unwrap();
        assert!(restored.graph().edge_count() > 0, "unqualified restore adds Surface containment");
        assert_eq!(restored.graph().resource_relations().count(), parent.graph().resource_relations().count());
        let control = sessions.open(parent.id()).await.unwrap();
        assert_eq!(control.graph().edge_count(), 0, "recorded parent restore preserves its stores");
    });
}

#[test]
fn navigation_replay_equality_changes_with_unordered_navigation_sets() {
    use std::sync::{Arc, Mutex};
    let mut baseline = Graph::new();
    let parent = add_node(&mut baseline, Some(Uuid::from_u128(101)), "https://audit.test/parent".into(), Default::default());
    apply_graph_delta(&mut baseline, GraphDelta::NavigateNode { key: parent, url: "https://audit.test/parent".into() });
    let child = add_node(&mut baseline, Some(Uuid::from_u128(102)), "https://audit.test/child".into(), Default::default());
    apply_graph_delta(&mut baseline, GraphDelta::BranchHistory { child, parent });
    apply_graph_delta(&mut baseline, GraphDelta::NavigateNode { key: child, url: "https://audit.test/child".into() });
    let state = |graph: &Graph| serde_json::to_value((graph.to_snapshot_at(0), graph.facets())).unwrap();
    let normalize = |mut value: serde_json::Value| {
        for visit in value[0]["navigation"]["snapshot"]["visits"].as_array_mut().unwrap() {
            visit["bindings"].as_array_mut().unwrap().sort_by_cached_key(|binding| serde_json::to_string(binding).unwrap());
        }
        for owner in value[0]["navigation"]["snapshot"]["owners"].as_array_mut().unwrap() {
            owner["owned_visits"].as_array_mut().unwrap().sort_by_key(|visit| visit.as_u64().unwrap());
        }
        value
    };
    let mut mismatches = 0;
    let mut rejected = 0;
    for _ in 0..64 {
        let mut live = baseline.clone();
        let mut replayed = baseline.clone();
        let captures = Arc::new(Mutex::new(Vec::new()));
        let sink = captures.clone();
        live.set_recorder(Some(Arc::new(move |delta| sink.lock().unwrap().push(delta.clone()))));
        apply_graph_delta(&mut live, GraphDelta::NavigateNode { key: parent, url: "https://audit.test/later".into() });
        kernel::graph::replay_captured_deltas_onto(&mut replayed, captures.lock().unwrap().clone());
        let (a, b) = (state(&live), state(&replayed));
        mismatches += usize::from(a != b);
        assert_eq!(normalize(a), normalize(b), "only unordered navigation collections differ");
        let manifest = GraphSessionManifest::new(SessionId(Uuid::from_u128(103)), GraphId(Uuid::from_u128(104)));
        let author = Author::person("audit");
        let mut session = GraphSession::new_recorded(muniment::MemoryBackend::new(), manifest, Some(baseline.clone()), author.clone(), ChangeKind::Minted);
        let result = session.edit_now(author, |graph| {
            apply_graph_delta(graph, GraphDelta::NavigateNode { key: parent, url: "https://audit.test/later".into() });
        });
        rejected += usize::from(matches!(result, Err(pandect::SessionError::NotReplayable(_))));
    }
    println!("raw live/replay mismatches from navigation order: {mismatches}/64; ordinary session navigations rejected: {rejected}/64");
    assert!(mismatches > 0);
    assert!(rejected > 0);
}
