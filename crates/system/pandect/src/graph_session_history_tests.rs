// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Engine-split D161: `graph_at` starts from the held checkpoint, and a
//! session-level import with a refused edit commits nothing. Over the shared
//! test surface (D272).
use super::*;
use crate::test_surface::{Delta, Surface, edges_edit, show_edits};
use mere::session::ReplayPoint;
use muniment::MemoryBackend;

type Session = GraphSession<MemoryBackend, Surface>;
type Sessions = MereSessions<MemoryBackend, Surface>;

fn person() -> Author {
    Author::person("persona-a").via("turnstone")
}

fn restorer() -> Author {
    Author::person("persona-b").via("knot-editor")
}

/// Surface `id` showing `https://{id}.test/`: the resource, then the surface.
fn add(id: u128) -> Vec<Delta> {
    show_edits(Uuid::from_u128(id), &format!("https://{id}.test/")).1
}

/// Surface `id` showing the resource named `title`.
fn retitle(id: u128, title: &str) -> Vec<Delta> {
    show_edits(Uuid::from_u128(id), &format!("https://{title}.test/")).1
}

/// A graph's whole persisted state: its snapshot and its facets.
fn whole(graph: &Surface) -> (serde_json::Value, serde_json::Value) {
    (
        serde_json::to_value(graph.to_snapshot()).unwrap(),
        serde_json::to_value(graph.facets()).unwrap(),
    )
}

/// `graph_at` as it read before D161: the baseline and the whole prefix.
fn from_baseline(session: &Session, cursor: Seq) -> Surface {
    let mut graph = session
        .journal
        .snapshot_at_from(&session.baseline, cursor)
        .unwrap()
        .unwrap();
    graph.validate_handles().unwrap();
    graph.set_known_coverage(session.graph.known_coverage().clone());
    graph
}

/// Where `graph_at(cursor)` starts: the held checkpoint's cursor, or the
/// baseline's zero.
fn start(session: &Session, cursor: Seq) -> Seq {
    session
        .replay_point_for(cursor)
        .map_or(Seq(0), |point| point.cursor())
}

/// Every cursor's `graph_at` equals the from-baseline replay, and starts from
/// the held checkpoint exactly when one lies at or before it.
fn every_cursor_matches(session: &Session) {
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
        assert_eq!(
            whole(&session.graph_at(cursor).unwrap()),
            whole(&from_baseline(session, cursor)),
            "graph_at({at}) equals the from-baseline replay"
        );
    }
}

/// The pair 1 -> 2 as one exact row carrying `statements`, each with the
/// given source (`None` is a source the replay attributes).
fn exact(template: &Surface, statements: &[(&str, Option<&str>)]) -> Vec<Delta> {
    let (one, two) = (Uuid::from_u128(1), Uuid::from_u128(2));
    let mut edges = template.rows_between(one, two);
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
    let (from, to) = (
        template.shown(one).unwrap(),
        template.shown(two).unwrap(),
    );
    vec![edges_edit(from, to, edges)]
}

/// Record `deltas` under `author` and bring the live graph to the replay.
fn record(session: &mut Session, author: Author, deltas: Vec<Delta>) {
    for delta in deltas {
        session.journal.record_as(author.clone(), delta);
    }
    session.graph = from_baseline(session, session.journal.live_cursor());
}

async fn checkpoint(session: &mut Session) {
    session
        .store(SystemTime::now(), Vec::new(), true)
        .await
        .unwrap();
}

/// A session whose journal mints a statement with no recorded source, drops
/// it, and has another author restore it, with checkpoints between: replay
/// keeps the minting author only if it carries attribution across them.
async fn attributed_session(store: &MemoryBackend) -> (SessionId, Surface) {
    let mere = Sessions::new_with(store.clone());
    let id = mere.mint(person(), None).await.unwrap().session_id;
    let mut session = mere.open_with(id).await.unwrap();
    let mut template = Surface::default();
    template.show(Uuid::from_u128(1), "https://1.test/");
    template.show(Uuid::from_u128(2), "https://2.test/");
    template.assert(
        Uuid::from_u128(1),
        Uuid::from_u128(2),
        "https://history.test/vocab#cites",
    );
    session.baseline = template.clone();
    session.write_baseline = true;
    session.head_stored = false;
    session.graph = template.clone();
    let source = Some("https://source.test/page");
    let minted = [("kept", source), ("minted", None)];
    every_cursor_matches(&session);

    record(&mut session, person(), exact(&template, &minted));
    record(&mut session, person(), add(3));
    checkpoint(&mut session).await;
    every_cursor_matches(&session);
    record(
        &mut session,
        person(),
        exact(&template, &[("kept", source)]),
    );
    record(&mut session, restorer(), retitle(3, "three"));
    checkpoint(&mut session).await;
    every_cursor_matches(&session);
    record(&mut session, restorer(), exact(&template, &minted));
    every_cursor_matches(&session);

    // Control: the checkpoint's graph replayed on without the attribution it
    // carried gives the restoring author, so the comparison above sees it.
    let point = session.replay_point.as_ref().unwrap().point.clone();
    assert_eq!(point.cursor(), Seq(6));
    let mut bare = point.graph().clone();
    session.journal.replay_from(Seq(6), &mut bare).unwrap();
    assert_ne!(
        whole(&bare),
        whole(&session.graph_at(Seq(7)).unwrap()),
        "control: replay from the checkpoint without its attribution differs"
    );
    let edges = session
        .graph_at(Seq(7))
        .unwrap()
        .rows_between(Uuid::from_u128(1), Uuid::from_u128(2));
    let restored = edges[0]
        .retained_assertions()
        .iter()
        .find(|row| row.statement_id == "minted")
        .unwrap();
    assert_eq!(
        restored.provenance_iri.as_deref(),
        Some(person().asserter_iri().as_str()),
        "restoration keeps the minting author"
    );

    record(&mut session, restorer(), add(4));
    checkpoint(&mut session).await;
    record(&mut session, person(), retitle(4, "four"));
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
        let reopened = Session::open_with(store, id).await.unwrap();
        assert_eq!(whole(reopened.graph()), whole(&live));
        assert_eq!(
            reopened
                .replay_point
                .as_ref()
                .map(|held| held.point.cursor()),
            Some(Seq(9))
        );
        every_cursor_matches(&reopened);
    });
}

#[test]
fn a_cursor_before_the_held_checkpoint_replays_from_the_baseline() {
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let mere = Sessions::new_with(store.clone());
        let id = mere.mint(person(), None).await.unwrap().session_id;
        let mut session = mere.open_with(id).await.unwrap();
        for n in 0..2 {
            session.apply(person(), add(n)).await.unwrap();
        }
        session.checkpoint().await.unwrap();
        session.apply(person(), add(2)).await.unwrap();
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
        let mere = Sessions::new_with(store.clone());
        let id = mere.mint(person(), None).await.unwrap().session_id;
        let mut session = mere.open_with(id).await.unwrap();
        for n in 1..=3 {
            session.apply(person(), add(n)).await.unwrap();
        }
        session.checkpoint().await.unwrap();
        session.apply(person(), retitle(1, "one")).await.unwrap();
        let live = session.journal.live_cursor();
        let good = session.replay_point.clone().unwrap();
        let at = good.point.cursor();
        assert!(Seq(0) < at && at < live, "a checkpoint inside the journal");
        let expected = whole(&from_baseline(&session, live));

        // A point at the same cursor built over another baseline: wrong
        // content, so whichever start graph_at takes shows in its result.
        let mut other = Surface::default();
        other.show(Uuid::from_u128(99), "https://99.test/");
        let wrong = session
            .journal
            .replay_point_from(&other, at)
            .unwrap()
            .unwrap();
        let held = |point: ReplayPoint<Surface>, digest: Option<String>| {
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

/// Surfaces 1 -> 2 with one statement between their resources, and its handle.
fn related() -> (Surface, String) {
    let mut graph = Surface::default();
    let (one, two) = (Uuid::from_u128(1), Uuid::from_u128(2));
    graph.show(one, "https://1.test/");
    graph.show(two, "https://2.test/");
    let handle = graph.assert(one, two, "https://import.test/vocab#next");
    (graph, handle)
}

/// An import of three edits whose middle one puts `handle` on the pair
/// 2 -> 1: surface 7's resource, the relation, then surface 7.
fn import(graph: &Surface, handle: &str) -> Vec<Delta> {
    let (one, two) = (Uuid::from_u128(1), Uuid::from_u128(2));
    let mut rows = graph.rows_between(one, two);
    for statement in rows[0].assertions.as_mut().unwrap() {
        statement.statement_id = handle.into();
    }
    let relation = edges_edit(graph.shown(two).unwrap(), graph.shown(one).unwrap(), rows);
    let (_, mut seven) = show_edits(Uuid::from_u128(7), "https://imported.test/");
    let resource = seven.remove(0);
    vec![resource, relation, seven.remove(0)]
}

#[test]
fn an_import_with_a_refused_edit_commits_nothing_and_names_the_handle() {
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let mere = Sessions::new_with(store.clone());
        let (graph, handle) = related();
        let mut session = mere.begin_with(person(), Some(graph));
        session.flush(SystemTime::now()).await.unwrap();
        let before = (
            whole(session.graph()),
            session.journal.live_cursor(),
            session.changes().len(),
            session.revision(),
        );
        let stored = stored_bytes(&store).await;

        let Err(refused) = session.apply_now(person(), import(session.graph(), &handle)) else {
            panic!("an import taking a held handle is refused");
        };
        assert!(refused.to_string().contains(&handle), "{refused}");
        assert!(refused.to_string().contains("edit 1"), "{refused}");
        assert_eq!(
            (
                whole(session.graph()),
                session.journal.live_cursor(),
                session.changes().len(),
                session.revision(),
            ),
            before,
            "nothing of the change is applied or recorded"
        );
        assert!(!session.has_unstored());
        session.flush(SystemTime::now()).await.unwrap();
        assert_eq!(stored_bytes(&store).await, stored, "nothing is stored");

        // Control: the same import with a fresh handle commits whole.
        let applied = session
            .apply_now(person(), import(session.graph(), "fresh-import"))
            .unwrap();
        assert!(applied.end > applied.first);
        let graph = session.graph();
        assert_eq!(
            graph.iri_of(Uuid::from_u128(7)).as_deref(),
            Some("https://imported.test/")
        );
        assert!(graph.statement("fresh-import").is_some());
    });
}

async fn stored_bytes(store: &MemoryBackend) -> Vec<(String, Vec<u8>)> {
    let mut keys = store.list("").await.unwrap();
    keys.sort();
    let mut bytes = Vec::new();
    for key in keys {
        bytes.push((key.clone(), store.get(&key).await.unwrap().unwrap()));
    }
    bytes
}

#[path = "graph_session_refusal_tests.rs"]
mod refusal;
