// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Engine-split D163: `pin` returns an owned view at its cursor, named by the
//! session's v2 chain digest there, released on drop.
use super::*;
use crate::test_surface::{Delta, Surface, show_edits};
use mere::dataset::View;
use muniment::MemoryBackend;
use std::collections::BTreeSet;

type Session = GraphSession<MemoryBackend, Surface>;

fn person() -> Author {
    Author::person("persona-a").via("turnstone")
}

fn add(id: u128) -> Vec<Delta> {
    show_edits(Uuid::from_u128(id), &format!("https://{id}.test/")).1
}

/// Assert statement `predicate` from surface `from` to surface `to` as one edit.
fn relate(session: &mut Session, from: u128, to: u128, predicate: &str) {
    session
        .edit_now(person(), |graph| {
            let predicate = format!("https://pin.test/vocab#{predicate}");
            graph.assert(Uuid::from_u128(from), Uuid::from_u128(to), &predicate);
        })
        .unwrap();
}

/// A graph's persisted state: its snapshot and its facets.
fn state(graph: &Surface) -> (serde_json::Value, serde_json::Value) {
    (
        serde_json::to_value(graph.to_snapshot()).unwrap(),
        serde_json::to_value(graph.facets()).unwrap(),
    )
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

/// A session with surfaces 1 to 4 and a chain of statements, stored, its
/// live cursor checkpointed.
async fn chained() -> Session {
    let mere = MereSessions::<_, Surface>::new_with(MemoryBackend::new());
    let id = mere.mint(person(), None).await.unwrap().session_id;
    let mut session = mere.open_with(id).await.unwrap();
    session
        .apply(person(), (1..=4).flat_map(add).collect())
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
        session.apply(person(), add(5)).await.unwrap();
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

/// Every Resource-stratum quad's line, and every quad of the whole projection.
fn resource_and_all(graph: &Surface) -> (BTreeSet<String>, BTreeSet<String>) {
    let all = graph
        .held_quads()
        .chain(graph.withdrawn_reifier_quads())
        .map(|quad| quad.to_string());
    (
        resource_quads(graph).map(|quad| quad.to_string()).collect(),
        all.collect(),
    )
}

#[test]
fn the_resource_digest_keeps_reifier_rows_and_is_stable() {
    pollster::block_on(async {
        let session = chained().await;
        let live = session.journal.live_cursor();
        let pin = session.pin(live).unwrap();
        let quads: Vec<oxrdf::Quad> = resource_quads(pin.graph()).collect();
        let triple_terms = quads
            .iter()
            .filter(|quad| matches!(quad.object, oxrdf::Term::Triple(_)))
            .count();
        assert!(triple_terms > 0, "the stratum has reifier rows");

        let digest = pin.resource_digest().unwrap().clone();
        assert_eq!(digest.algorithm, RESOURCE_DIGEST_ALGORITHM);
        assert_eq!(digest.version, RESOURCE_DIGEST_VERSION);
        let text = sorted_nquads(quads.clone()).unwrap();
        assert!(text.contains("<<( "), "triple terms in the text");
        let expected = blake3::Hasher::new_derive_key(RESOURCE_DIGEST_DOMAIN)
            .update(text.as_bytes())
            .finalize()
            .to_hex()
            .to_string();
        assert_eq!(digest.digest, expected, "the documented construction");
        let without = sorted_nquads(
            quads
                .into_iter()
                .filter(|quad| !matches!(quad.object, oxrdf::Term::Triple(_))),
        )
        .unwrap();
        assert_ne!(
            digest_nquads(RESOURCE_DIGEST_DOMAIN, &without),
            digest.digest,
            "the reifier rows are in the digest"
        );

        let again = session.pin(live).unwrap();
        assert!(!std::ptr::eq(pin.graph(), again.graph()));
        assert_eq!(
            again.resource_digest().unwrap(),
            &digest,
            "recomputed alike"
        );
        let earlier = session.pin(Seq(live.0 - 1)).unwrap();
        assert_ne!(
            earlier.resource_digest().unwrap().digest,
            digest.digest,
            "control: the cursor before the last statement differs"
        );
    });
}

#[test]
fn the_resource_stratum_without_reifier_rows_reads_as_rdfc() {
    pollster::block_on(async {
        let session = chained().await;
        let pin = session.pin(session.journal.live_cursor()).unwrap();
        let plain: Vec<oxrdf::Quad> = resource_quads(pin.graph())
            .filter(|quad| !matches!(quad.object, oxrdf::Term::Triple(_)))
            .collect();
        assert!(!plain.is_empty());
        let rdfc = mere::canonicalize_quads(&plain, &mere::CanonOptions::default())
            .unwrap()
            .nquads;
        assert_eq!(sorted_nquads(plain).unwrap(), rdfc);
    });
}

/// The surface stratum stands where the kernel's arrangements stood: an edit
/// the projection shows and the Resource stratum does not.
#[test]
fn an_arrangement_edit_keeps_the_resource_digest_and_a_resource_edit_moves_it() {
    pollster::block_on(async {
        let mut session = chained().await;
        let before = session.pin(session.journal().live_cursor()).unwrap();
        let cursor = before.cursor();
        session
            .edit_now(person(), |graph| {
                // A fifth surface shows surface 1's resource: no resource is new.
                graph.show(Uuid::from_u128(5), "https://1.test/");
            })
            .unwrap();
        let arranged = session.pin(session.journal.live_cursor()).unwrap();
        assert!(
            arranged.cursor() > cursor,
            "the arrangement is a journal edit"
        );
        let (resources, all) = resource_and_all(before.graph());
        let (arranged_resources, arranged_all) = resource_and_all(arranged.graph());
        assert_ne!(arranged_all, all, "control: the projection shows it");
        assert_eq!(arranged_resources, resources);
        assert_eq!(
            arranged.resource_digest().unwrap(),
            before.resource_digest().unwrap(),
            "an arrangement edit leaves the Resource digest"
        );

        relate(&mut session, 3, 1, "answers");
        let edited = session.pin(session.journal.live_cursor()).unwrap();
        assert_ne!(
            edited.resource_digest().unwrap(),
            before.resource_digest().unwrap(),
            "a Resource edit moves it"
        );
    });
}

#[test]
fn a_pin_computes_its_resource_digest_once_on_first_ask() {
    pollster::block_on(async {
        let mut session = chained().await;
        let pin = session.pin(session.journal.live_cursor()).unwrap();
        assert_eq!(pin.resource_digests_computed(), 0, "pinning computes none");
        relate(&mut session, 3, 4, "later");
        session.apply(person(), add(5)).await.unwrap();
        let _ = pin.dataset();
        assert_eq!(
            pin.resource_digests_computed(),
            0,
            "edits and the index compute none"
        );

        let first = pin.resource_digest().unwrap();
        assert_eq!(
            pin.resource_digests_computed(),
            1,
            "control: the ask computes it"
        );
        let second = pin.resource_digest().unwrap();
        assert_eq!(pin.resource_digests_computed(), 1, "kept, not recomputed");
        assert!(std::ptr::eq(first, second));

        let other = session.pin(pin.cursor()).unwrap();
        assert_eq!(other.resource_digests_computed(), 0, "each pin its own");
        assert_eq!(other.resource_digest().unwrap(), first);
        assert_eq!(other.resource_digests_computed(), 1);
    });
}

#[test]
fn the_resource_digest_is_domain_separated_and_versioned() {
    let text =
        "<urn:a> <urn:p> <<( <urn:s> <urn:p> <urn:o> )>> .\n<urn:a> <urn:p> <urn:b> <urn:g> .\n";
    // Known answers from an independent BLAKE3 (Python `blake3` 1.0.9,
    // `derive_key_context="mere resource digest v1"`).
    assert_eq!(
        digest_nquads(RESOURCE_DIGEST_DOMAIN, text),
        "b9f03eefecee42a211cb413ae05b9db27b88185029a318973891181392c0cfce"
    );
    assert_eq!(
        digest_nquads(RESOURCE_DIGEST_DOMAIN, ""),
        "4ca1cb55f0bc7ce78d867413037ca62d8a1e4c40c2cc4488f48f050169253b39"
    );
    assert_ne!(
        digest_nquads(RESOURCE_DIGEST_DOMAIN, text),
        blake3::hash(text.as_bytes()).to_hex().to_string(),
        "not a plain hash"
    );
    assert_eq!(
        RESOURCE_DIGEST_DOMAIN,
        format!("mere resource digest v{RESOURCE_DIGEST_VERSION}"),
        "the domain names the version"
    );
    let v1 = digest_nquads(RESOURCE_DIGEST_DOMAIN, text);
    assert_ne!(
        v1,
        digest_nquads("mere resource digest v2", text),
        "a version bump"
    );
    assert_ne!(
        v1,
        digest_nquads("muniment journal prefix digest v2", text),
        "another domain"
    );
    assert_eq!(
        v1,
        digest_nquads(RESOURCE_DIGEST_DOMAIN, text),
        "control: same inputs agree"
    );
}
