// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// Replay reproduces the live graph exactly, clock-read visit stamps and
/// minted statement ids included: the journal keeps what was read or
/// minted (reservoir plan V2).
#[test]
fn replay_reproduces_visit_stamps_and_statement_ids() {
    let (journal, hook) = journal_capture_hook();
    set_captured_delta_hook(Some(hook));
    let mut live = Graph::new();
    let key = |graph: &mut Graph, id: u128| match apply_graph_delta(
        graph,
        GraphDelta::AddNode {
            id: Some(Uuid::from_u128(id)),
            url: format!("https://{id}.test/"),
            position: Point2D::new(0.0, 0.0),
        },
    ) {
        GraphDeltaResult::NodeAdded(key) => key,
        other => panic!("expected NodeAdded, got {other:?}"),
    };
    let a = key(&mut live, 1);
    // Let the clock move, so a replay's own stamp would differ.
    std::thread::sleep(std::time::Duration::from_millis(2));
    let b = key(&mut live, 2);
    apply_graph_delta(
        &mut live,
        GraphDelta::AssertRelation {
            asserter_iri: crate::graph::journal::Author::user().asserter_iri(),
            from: a,
            to: b,
            assertion: EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Cites,
                label: None,
            },
        },
    );
    apply_graph_delta(
        &mut live,
        GraphDelta::AssertSemanticPredicate {
            asserter_iri: crate::graph::journal::Author::user().asserter_iri(),
            from: b,
            to: a,
            predicate: "https://schema.org/about".into(),
        },
    );
    set_captured_delta_hook(None);

    std::thread::sleep(std::time::Duration::from_millis(2));
    let replayed = journal.lock().unwrap().replay();
    for id in [1u128, 2] {
        let visited = |graph: &Graph| {
            let (key, _) = graph.get_node_by_id(Uuid::from_u128(id)).unwrap();
            graph.node_last_visited(key)
        };
        assert!(
            visited(&live).is_some(),
            "creating node {id} stamped a visit"
        );
        assert_eq!(
            visited(&replayed),
            visited(&live),
            "node {id}'s visit stamp"
        );
    }
    let key_of = |graph: &Graph, key| {
        let id = live.get_node(key).unwrap().id;
        graph.get_node_key_by_id(id).unwrap()
    };
    for (from, to) in [(a, b), (b, a)] {
        let resource_pair = (
            live.shown_resource_id(from).unwrap(),
            live.shown_resource_id(to).unwrap(),
        );
        assert!(live.persisted_edges_between(from, to).is_empty());
        assert!(
            !live
                .persisted_resource_edges_between(resource_pair.0, resource_pair.1)
                .is_empty(),
            "the live Resource graph holds a minted statement there"
        );
        assert_eq!(
            replayed.shown_resource_id(key_of(&replayed, from)),
            Some(resource_pair.0)
        );
        assert_eq!(
            replayed.shown_resource_id(key_of(&replayed, to)),
            Some(resource_pair.1)
        );
        assert_eq!(
            replayed.persisted_resource_edges_between(resource_pair.0, resource_pair.1),
            live.persisted_resource_edges_between(resource_pair.0, resource_pair.1),
            "statement ids survive replay"
        );
    }
}

/// Replay records nothing: a scrub or a catch-up re-applies history, it
/// does not make new history. The same run first proves both instruments
/// see a live edit, so the silence after is the replay's.
#[test]
fn replay_records_nothing() {
    let (hooked, hook) = journal_capture_hook();
    set_captured_delta_hook(Some(hook));
    let seen = Arc::new(Mutex::new(0usize));
    let sink = Arc::clone(&seen);
    let mut session = Graph::new();
    session.set_recorder(Some(Arc::new(move |_: &CapturedDelta| {
        *sink.lock().unwrap() += 1;
    })));
    apply_graph_delta(
        &mut session,
        GraphDelta::AddNode {
            id: Some(Uuid::from_u128(1)),
            url: "https://a.test/".to_string(),
            position: Point2D::new(0.0, 0.0),
        },
    );
    // A live add records the surface, visit, resource and shown binding.
    assert_live_add_captures(
        &hooked
            .lock()
            .unwrap()
            .entries()
            .iter()
            .map(|entry| entry.delta.clone())
            .collect::<Vec<_>>(),
        1,
        "https://a.test/",
    );
    assert_eq!(
        hooked.lock().unwrap().len(),
        4,
        "the thread hook sees live edits"
    );
    assert_eq!(
        *seen.lock().unwrap(),
        4,
        "the graph's recorder sees live edits"
    );

    let history = hooked.lock().unwrap().clone();
    let scrubbed = history.snapshot_at(history.live_cursor()).unwrap().unwrap();
    replay_captured_deltas_onto(&mut session, [add(2, "https://b.test/")]);
    assert_eq!(scrubbed.node_count(), 1);
    assert_eq!(session.node_count(), 2);
    assert_eq!(
        hooked.lock().unwrap().len(),
        4,
        "replay reached the thread hook"
    );
    assert_eq!(
        *seen.lock().unwrap(),
        4,
        "replay reached the graph's recorder"
    );

    // Both instruments are back once the replay ends.
    assert!(session.is_recording());
    apply_graph_delta(
        &mut session,
        GraphDelta::AddNode {
            id: Some(Uuid::from_u128(3)),
            url: "https://c.test/".to_string(),
            position: Point2D::new(0.0, 0.0),
        },
    );
    let captures = hooked
        .lock()
        .unwrap()
        .entries()
        .iter()
        .map(|entry| entry.delta.clone())
        .collect::<Vec<_>>();
    assert_eq!(captures.len(), 8);
    assert_live_add_captures(&captures[4..], 3, "https://c.test/");
    assert_eq!(*seen.lock().unwrap(), 8);
    set_captured_delta_hook(None);
}

#[test]
fn records_and_replays_reconstruct_the_graph() {
    let mut journal = GraphJournal::new();
    journal.record(add(1, "https://a.test/"));
    journal.record(add(2, "https://b.test/"));
    journal.record(CapturedDelta::ReplayAssertRelationByIds {
        from_id: Uuid::from_u128(1).to_string(),
        to_id: Uuid::from_u128(2).to_string(),
        assertion: EdgeAssertion::Semantic {
            sub_kind: SemanticSubKind::Cites,
            label: None,
        },
    });
    journal.record(CapturedDelta::ReplaySetNodeTitleById {
        node_id: Uuid::from_u128(1).to_string(),
        title: "Paper A".to_string(),
    });

    let graph = journal.replay();
    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.relations().count(), 1, "the cites relation replayed");
    let (_, node) = graph
        .get_node_by_id(Uuid::from_u128(1))
        .expect("node a present");
    assert_eq!(node.title, "Paper A", "the content edit replayed too");
}

#[test]
fn prefix_cursors_replay_historical_graphs_without_touching_live_history() {
    let mut journal = GraphJournal::new();
    journal.record(add(1, "https://a.test/"));
    journal.record(add(2, "https://b.test/"));
    journal.record(CapturedDelta::ReplayAssertRelationByIds {
        from_id: Uuid::from_u128(1).to_string(),
        to_id: Uuid::from_u128(2).to_string(),
        assertion: EdgeAssertion::Semantic {
            sub_kind: SemanticSubKind::Cites,
            label: None,
        },
    });

    assert_eq!(journal.earliest_cursor(), Seq(0));
    assert_eq!(journal.live_cursor(), Seq(3));
    assert_eq!(
        journal.snapshot_at(Seq(0)).unwrap().unwrap().node_count(),
        0
    );
    assert_eq!(
        journal.snapshot_at(Seq(1)).unwrap().unwrap().node_count(),
        1
    );
    let historical = journal.snapshot_at(Seq(2)).unwrap().expect("known prefix");
    assert_eq!(historical.node_count(), 2);
    assert_eq!(historical.relations().count(), 0);
    assert_eq!(
        journal
            .snapshot_at(Seq(3))
            .unwrap()
            .unwrap()
            .relations()
            .count(),
        1
    );
    assert!(
        journal.snapshot_at(Seq(4)).unwrap().is_none(),
        "stale cursors refuse"
    );
    assert_eq!(
        journal.replay().relations().count(),
        1,
        "live history remains live"
    );
    assert_eq!(journal.sequence_ticks(3), vec![Seq(0), Seq(1), Seq(3)]);
}
