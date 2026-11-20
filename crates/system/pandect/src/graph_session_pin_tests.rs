// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Engine-split D163: `pin` returns an owned view at its cursor, named by the
//! session's v2 chain digest there, released on drop.
use super::*;
use kernel::graph::SemanticStatementSpec;
use mere::dataset::View;
use muniment::MemoryBackend;
use std::collections::BTreeSet;

fn person() -> Author {
    Author::person("persona-a").via("turnstone")
}

fn add(id: u128) -> CapturedDelta {
    CapturedDelta::ReplayAddNodeWithIdIfMissing {
        id: Uuid::from_u128(id).to_string(),
        url: format!("https://{id}.test/"),
        position: [0.0, 0.0],
    }
}

/// Assert statement `predicate` from node `from` to node `to` as one edit.
fn relate(session: &mut GraphSession<MemoryBackend>, from: u128, to: u128, predicate: &str) {
    session
        .edit_now(person(), |graph| {
            let key = |id| graph.get_node_key_by_id(Uuid::from_u128(id)).unwrap();
            let (from, to) = (key(from), key(to));
            let spec = SemanticStatementSpec {
                predicate: format!("https://pin.test/vocab#{predicate}"),
                ..Default::default()
            };
            graph.assert_semantic_statement(from, to, spec).unwrap();
        })
        .unwrap();
}

/// A graph's persisted state, undated and without the visit clock, which a
/// replay between a node's creation and its journaled stamp reads afresh.
fn state(graph: &Graph) -> (serde_json::Value, serde_json::Value) {
    let mut snapshot = graph.to_snapshot();
    snapshot.timestamp_secs = 0;
    let mut facets = serde_json::to_value(graph.facets()).unwrap();
    if let Some(nodes) = facets.get_mut("nodes").and_then(|n| n.as_object_mut()) {
        for node in nodes.values_mut() {
            if let Some(visit) = node.get_mut("visit.history") {
                visit["last_visited_ms"] = 0.into();
            }
        }
    }
    (serde_json::to_value(&snapshot).unwrap(), facets)
}

/// Every row of `dataset`, live and withdrawn, as sortable text.
fn rows(dataset: &IndexedDataset) -> BTreeSet<String> {
    [View::Live, View::History]
        .into_iter()
        .flat_map(|view| {
            dataset
                .quads(view)
                .map(move |quad| format!("{view:?} {quad}"))
        })
        .collect()
}

/// `graph_at` as it read before D161: the baseline and the whole prefix.
fn from_baseline(session: &GraphSession<MemoryBackend>, cursor: Seq) -> Graph {
    let mut graph = session
        .journal
        .snapshot_at_from(&session.baseline, cursor)
        .unwrap()
        .unwrap();
    graph.validate_active_resource_assertion_handles().unwrap();
    graph.set_known_coverage(session.graph.known_coverage().clone());
    graph
}

/// A session with nodes 1 to 4 and a chain of statements, stored, its live
/// cursor checkpointed.
async fn chained() -> GraphSession<MemoryBackend> {
    let mere = MereSessions::new(MemoryBackend::new());
    let id = mere.mint(person(), None).await.unwrap().session_id;
    let mut session = mere.open(id).await.unwrap();
    session
        .apply(person(), (1..=4).map(add).collect())
        .await
        .unwrap();
    relate(&mut session, 1, 2, "next");
    relate(&mut session, 2, 3, "next");
    session.checkpoint().await.unwrap();
    session
}

#[test]
fn a_pin_reads_its_cursor_after_live_moves() {
    pollster::block_on(async {
        let mut session = chained().await;
        let at = session.journal.live_cursor();
        let then = (
            state(session.graph()),
            rows(&IndexedDataset::for_graph(session.graph())),
        );
        let pin = session.pin(at).unwrap();
        assert_eq!((pin.session(), pin.cursor()), (session.id(), at));
        let revision = session.live_revision().unwrap();
        assert_eq!(pin.revision_identity(), revision);

        relate(&mut session, 3, 4, "later");
        session.apply(person(), vec![add(5)]).await.unwrap();
        assert_ne!(session.live_revision().unwrap(), revision);
        assert_eq!(pin.revision_identity(), revision);
        let live = rows(&IndexedDataset::for_graph(session.graph()));
        assert_ne!(live, then.1, "control: live moved");
        assert_ne!(state(session.graph()), then.0, "control: live moved");

        assert_eq!(
            state(pin.graph()),
            then.0,
            "the pin's graph is the cursor's"
        );
        assert_eq!(
            rows(pin.dataset()),
            then.1,
            "the pin's quads are the cursor's"
        );
        pin.dataset().verify_against_fresh(pin.graph()).unwrap();
    });
}

#[test]
fn the_pin_digest_is_the_chain_digest_at_its_cursor() {
    pollster::block_on(async {
        let mut session = chained().await;
        let at = session.journal.live_cursor();
        let stored: Checkpoint = read(&session.slots, &session.keys.at(CHECKPOINT))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.cursor, at);
        assert_eq!(stored.journal_digest_version, Some(JOURNAL_DIGEST_VERSION));
        relate(&mut session, 3, 4, "later");

        let chain = |cursor| journal_digest(&session.digests, &session.journal, cursor).unwrap();
        let pin = session.pin(at).unwrap();
        assert_eq!(Some(pin.digest()), stored.journal_digest.as_deref());
        assert_eq!(Some(pin.digest().to_owned()), chain(at));
        let next = session.pin(Seq(at.0 + 1)).unwrap();
        assert_eq!(Some(next.digest().to_owned()), chain(Seq(at.0 + 1)));
        assert_ne!(
            next.digest(),
            pin.digest(),
            "control: the next prefix differs"
        );

        let past = Seq(session.journal.live_cursor().0 + 1);
        assert!(matches!(
            session.pin(past),
            Err(SessionError::CursorOutOfRange { .. })
        ));
    });
}

#[test]
fn dropping_a_pin_frees_it() {
    pollster::block_on(async {
        let session = chained().await;
        let pin = session.pin(Seq(2)).unwrap();
        let held = pin.downgrade();
        assert_eq!(held.strong_count(), 1, "only the holder holds it");
        assert!(held.upgrade().is_some(), "control: alive while held");
        drop(pin);
        assert_eq!(held.strong_count(), 0);
        assert!(held.upgrade().is_none(), "dropping the pin frees it");
    });
}

#[test]
fn pins_at_different_cursors_differ_and_at_one_cursor_agree() {
    pollster::block_on(async {
        let session = chained().await;
        let live = session.journal.live_cursor();
        let (a, b, again) = (
            session.pin(Seq(1)).unwrap(),
            session.pin(live).unwrap(),
            session.pin(Seq(1)).unwrap(),
        );
        assert_ne!(a.digest(), b.digest());
        assert_ne!(state(a.graph()), state(b.graph()));
        assert_ne!(rows(a.dataset()), rows(b.dataset()));

        // Positive control: two pins at one cursor are equal and separate.
        assert_eq!(a.digest(), again.digest());
        assert_eq!(state(a.graph()), state(again.graph()));
        assert_eq!(rows(a.dataset()), rows(again.dataset()));
        assert!(!std::ptr::eq(a.graph(), again.graph()));
    });
}

#[test]
fn pins_either_side_of_the_checkpoint_equal_the_baseline_replay() {
    pollster::block_on(async {
        let mut session = chained().await;
        relate(&mut session, 3, 4, "later");
        let held = session.replay_point.as_ref().unwrap().point.cursor();
        let (before, after) = (Seq(held.0 - 1), session.journal.live_cursor());
        assert!(after > held);
        let start = |cursor| session.replay_point_for(cursor).map(|point| point.cursor());
        assert_eq!(start(before), None, "before: replays from the baseline");
        assert_eq!(start(after), Some(held), "after: starts at the checkpoint");

        for cursor in [before, held, after] {
            let pin = session.pin(cursor).unwrap();
            let expected = from_baseline(&session, cursor);
            assert_eq!(state(pin.graph()), state(&expected), "at {}", cursor.0);
            pin.dataset().verify_against_fresh(&expected).unwrap();
        }
        let (early, late) = (session.pin(before).unwrap(), session.pin(after).unwrap());
        assert!(
            early.dataset().verify_against_fresh(late.graph()).is_err(),
            "control: the comparison sees the cursors apart"
        );
    });
}

#[test]
fn a_pin_indexes_on_its_first_read() {
    pollster::block_on(async {
        let mut session = chained().await;
        let at = session.journal.live_cursor();
        let expected = rows(&IndexedDataset::for_graph(&session.graph_at(at).unwrap()));
        let pin = session.pin(at).unwrap();
        assert!(!pin.is_indexed(), "no index until dataset()");
        relate(&mut session, 3, 4, "later");
        assert!(state(pin.graph()) != state(session.graph()));
        assert!(!pin.is_indexed(), "reading the graph builds no index");

        let first = rows(pin.dataset());
        assert!(pin.is_indexed(), "control: dataset() builds it");
        assert_eq!(first, expected, "the cursor's quads, though live moved");
        assert_eq!(rows(pin.dataset()), first, "the same quads after");
        assert!(std::ptr::eq(pin.dataset(), pin.dataset()), "built once");
    });
}
