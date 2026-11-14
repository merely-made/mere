// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Engine-split D161: `graph_at` starts from the held checkpoint.
use super::*;
use kernel::graph::{EdgeAssertion, SemanticSubKind};
use muniment::MemoryBackend;

fn person() -> Author {
    Author::person("persona-a").via("turnstone")
}

fn restorer() -> Author {
    Author::person("persona-b").via("knot-editor")
}

fn add(id: u128) -> CapturedDelta {
    CapturedDelta::ReplayAddNodeWithIdIfMissing {
        id: Uuid::from_u128(id).to_string(),
        url: format!("https://{id}.test/"),
        position: [0.0, 0.0],
    }
}

fn retitle(id: u128, title: &str) -> CapturedDelta {
    CapturedDelta::ReplaySetNodeTitleById {
        node_id: Uuid::from_u128(id).to_string(),
        title: title.to_string(),
    }
}

/// A graph's whole persisted state: its snapshot, undated, and its facets.
fn whole(graph: &Graph) -> (serde_json::Value, serde_json::Value) {
    let mut snapshot = graph.to_snapshot();
    snapshot.timestamp_secs = 0;
    (
        serde_json::to_value(&snapshot).unwrap(),
        serde_json::to_value(graph.facets()).unwrap(),
    )
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

/// Where `graph_at(cursor)` starts: the held checkpoint's cursor, or the
/// baseline's zero.
fn start(session: &GraphSession<MemoryBackend>, cursor: Seq) -> Seq {
    session
        .replay_point_for(cursor)
        .map_or(Seq(0), |point| point.cursor())
}

/// `whole` without the visit clock: a cursor between a node's creation and
/// its journaled stamp replays by reading the clock, so only there may two
/// replays differ.
fn unclocked(graph: &Graph) -> (serde_json::Value, serde_json::Value) {
    let (snapshot, mut facets) = whole(graph);
    if let Some(nodes) = facets.get_mut("nodes").and_then(|n| n.as_object_mut()) {
        for node in nodes.values_mut() {
            if let Some(visit) = node.get_mut("visit.history") {
                visit["last_visited_ms"] = 0.into();
            }
        }
    }
    (snapshot, facets)
}

/// Every cursor's `graph_at` equals the from-baseline replay, and starts from
/// the held checkpoint exactly when one lies at or before it. Whole at the
/// held checkpoint and the live cursor, where every stamp is journaled; at
/// the others, all but the visit clock.
fn every_cursor_matches(session: &GraphSession<MemoryBackend>) {
    let held = session
        .replay_point
        .as_ref()
        .map(|held| held.point.cursor());
    for at in 0..=session.journal.live_cursor().0 {
        let cursor = Seq(at);
        let expected = match held {
            Some(held) if held <= cursor => held,
            _ => Seq(0),
        };
        assert_eq!(start(session, cursor), expected, "start at {at}");
        let exact = cursor == session.journal.live_cursor() || Some(cursor) == held;
        let state = if exact { whole } else { unclocked };
        assert_eq!(
            state(&session.graph_at(cursor).unwrap()),
            state(&from_baseline(session, cursor)),
            "graph_at({at}) equals the from-baseline replay"
        );
    }
}

/// Add node `id` as a live edit journals it: the node, then its visit stamp.
fn add_stamped(session: &mut GraphSession<MemoryBackend>, author: Author, id: u128) {
    record(session, author.clone(), add(id));
    let delta = CapturedDelta::ReplayTouchNodeLastVisitedById {
        node_id: Uuid::from_u128(id).to_string(),
        timestamp_ms: 1_000 + id as u64,
    };
    record(session, author, delta);
}

/// The legacy Surface pair 1 -> 2 as one exact row carrying `statements`,
/// each with the given source (`None` is a source the replay attributes).
fn exact(template: &Graph, statements: &[(&str, Option<&str>)]) -> CapturedDelta {
    let mut edges = template.to_snapshot().edges;
    let rows = edges[0].assertions.as_mut().unwrap();
    let mut first = rows[0].clone();
    first.asserted_at_ms = Some(100);
    *rows = statements
        .iter()
        .map(|(id, source)| {
            let mut row = first.clone();
            row.statement_id = (*id).into();
            row.provenance_iri = source.map(str::to_owned);
            row
        })
        .collect();
    CapturedDelta::ReplaySetEdgesByIds {
        from_id: Uuid::from_u128(1).to_string(),
        to_id: Uuid::from_u128(2).to_string(),
        edges,
    }
}

/// Record `delta` under `author` and bring the live graph to the replay.
fn record(session: &mut GraphSession<MemoryBackend>, author: Author, delta: CapturedDelta) {
    session.journal.record_as(author, delta);
    session.graph = from_baseline(session, session.journal.live_cursor());
}

async fn checkpoint(session: &mut GraphSession<MemoryBackend>) {
    session
        .store(SystemTime::now(), Vec::new(), true)
        .await
        .unwrap();
}

/// A session whose journal mints a statement with no recorded source, drops
/// it, and has another author restore it, with checkpoints between: replay
/// keeps the minting author only if it carries attribution across them.
async fn attributed_session(store: &MemoryBackend) -> (SessionId, Graph) {
    let mere = MereSessions::new(store.clone());
    let id = mere.mint(person(), None).await.unwrap().session_id;
    let mut session = mere.open(id).await.unwrap();
    let template = kernel::graph::replay_captured_deltas([
        add(1),
        add(2),
        CapturedDelta::ReplayAssertRelationByIds {
            from_id: Uuid::from_u128(1).to_string(),
            to_id: Uuid::from_u128(2).to_string(),
            assertion: EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Cites,
                label: None,
            },
        },
    ]);
    session.baseline = template.clone();
    session.write_baseline = true;
    session.head_stored = false;
    session.graph = template.clone();
    let source = Some("https://source.test/page");
    let minted = [("kept", source), ("minted", None)];
    every_cursor_matches(&session);

    record(&mut session, person(), exact(&template, &minted));
    add_stamped(&mut session, person(), 3);
    checkpoint(&mut session).await;
    every_cursor_matches(&session);
    record(
        &mut session,
        person(),
        exact(&template, &[("kept", source)]),
    );
    record(&mut session, restorer(), retitle(3, "Three"));
    checkpoint(&mut session).await;
    every_cursor_matches(&session);
    record(&mut session, restorer(), exact(&template, &minted));
    every_cursor_matches(&session);

    // Control: the checkpoint's graph replayed on without the attribution it
    // carried gives the restoring author, so the comparison above sees it.
    let point = session.replay_point.as_ref().unwrap().point.clone();
    assert_eq!(point.cursor(), Seq(5));
    let mut bare = point.graph().clone();
    session.journal.replay_from(Seq(5), &mut bare).unwrap();
    assert_ne!(
        whole(&bare),
        whole(&session.graph_at(Seq(6)).unwrap()),
        "control: replay from the checkpoint without its attribution differs"
    );
    let edges = session.graph_at(Seq(6)).unwrap().to_snapshot().edges;
    let restored = edges[0]
        .retained_assertions()
        .into_iter()
        .find(|row| row.statement_id == "minted")
        .unwrap();
    assert_eq!(
        restored.provenance_iri.as_deref(),
        Some(person().asserter_iri().as_str()),
        "restoration keeps the minting author"
    );

    add_stamped(&mut session, restorer(), 4);
    checkpoint(&mut session).await;
    record(&mut session, person(), retitle(4, "Four"));
    every_cursor_matches(&session);
    session.flush(SystemTime::now()).await.unwrap();
    (id, session.graph.clone())
}

#[test]
fn graph_at_starts_from_the_held_checkpoint_and_equals_the_baseline_replay() {
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let (id, live) = attributed_session(&store).await;

        // Reopen holds the stored checkpoint's replay point, read beside the
        // prefix replay open already makes for attribution.
        let reopened = GraphSession::open(store, id).await.unwrap();
        assert_eq!(whole(reopened.graph()), whole(&live));
        assert_eq!(
            reopened
                .replay_point
                .as_ref()
                .map(|held| held.point.cursor()),
            Some(Seq(8))
        );
        every_cursor_matches(&reopened);
    });
}

#[test]
fn a_cursor_before_the_held_checkpoint_replays_from_the_baseline() {
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let mere = MereSessions::new(store.clone());
        let id = mere.mint(person(), None).await.unwrap().session_id;
        let mut session = mere.open(id).await.unwrap();
        for n in 0..2 {
            session.apply(person(), vec![add(n)]).await.unwrap();
        }
        session.checkpoint().await.unwrap();
        session.apply(person(), vec![add(2)]).await.unwrap();
        let held = session.replay_point.as_ref().unwrap().point.cursor();
        assert!(
            held > Seq(1),
            "a checkpoint past the first entries: {held:?}"
        );
        for at in 0..held.0 {
            assert_eq!(start(&session, Seq(at)), Seq(0));
        }
        assert_eq!(
            start(&session, held),
            held,
            "control: at the checkpoint it is used"
        );
        every_cursor_matches(&session);
    });
}

#[test]
fn a_held_checkpoint_whose_digest_does_not_verify_is_skipped() {
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let mere = MereSessions::new(store.clone());
        let id = mere.mint(person(), None).await.unwrap().session_id;
        let mut session = mere.open(id).await.unwrap();
        for n in 1..=3 {
            session.apply(person(), vec![add(n)]).await.unwrap();
        }
        session.checkpoint().await.unwrap();
        session
            .apply(person(), vec![retitle(1, "One")])
            .await
            .unwrap();
        let live = session.journal.live_cursor();
        let good = session.replay_point.clone().unwrap();
        let at = good.point.cursor();
        assert!(Seq(0) < at && at < live, "a checkpoint inside the journal");
        let expected = whole(&from_baseline(&session, live));

        // A point at the same cursor built over another baseline: wrong
        // content, so whichever start graph_at takes shows in its result.
        let wrong = session
            .journal
            .replay_point_from(&kernel::graph::replay_captured_deltas([add(99)]), at)
            .unwrap()
            .unwrap();
        let held = |point: kernel::graph::ReplayPoint, digest: Option<String>| {
            Some(Arc::new(HeldPoint { point, digest }))
        };

        // Positive control: a verified point is used, so its content shows.
        session.replay_point = held(wrong.clone(), good.digest.clone());
        assert_eq!(start(&session, live), at);
        assert_ne!(
            whole(&session.graph_at(live).unwrap()),
            expected,
            "control: graph_at reads a verified checkpoint"
        );

        // A digest that does not name the journal's prefix: skipped.
        session.replay_point = held(wrong, Some("0".repeat(64)));
        assert_eq!(start(&session, live), Seq(0), "the baseline is used");
        assert_eq!(whole(&session.graph_at(live).unwrap()), expected);

        // The good point verifies and is used.
        session.replay_point = Some(good);
        assert_eq!(start(&session, live), at);
        assert_eq!(whole(&session.graph_at(live).unwrap()), expected);
    });
}
