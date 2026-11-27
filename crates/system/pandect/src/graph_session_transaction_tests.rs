// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::test_surface::{Delta, Surface, show_edits};
use muniment::MemoryBackend;

type Sessions = MereSessions<MemoryBackend, Surface>;

fn person() -> Author {
    Author::person("persona-a").via("turnstone")
}

fn add(id: u128) -> Vec<Delta> {
    show_edits(Uuid::from_u128(id), &format!("https://{id}.test/")).1
}

fn holds(session: &GraphSession<MemoryBackend, Surface>, id: u128) -> bool {
    session.graph().shown(Uuid::from_u128(id)).is_some()
}

#[test]
fn a_transaction_carries_a_direct_writer_through_the_replay_check() {
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let mere = Sessions::new_with(store.clone());
        let id = mere.mint(person(), None).await.unwrap().session_id;
        let mut session = mere.open_with(id).await.unwrap();
        session.apply(person(), add(1)).await.unwrap();
        let (one, transacted) = (Uuid::from_u128(1), "https://transacted.test/");
        let at = SystemTime::UNIX_EPOCH;

        session
            .transact_change(at, false, |candidate| {
                candidate
                    .edit_now(person(), |graph| graph.show(one, transacted))
                    .map(|_| ())
            })
            .await
            .unwrap();
        let shown = |graph: &Surface| graph.iri_of(one).unwrap();
        assert_eq!(shown(session.graph()), transacted);
        assert!(!session.has_unstored());
        let reopened = mere.open_with(id).await.unwrap();
        assert_eq!(shown(reopened.graph()), transacted);
        assert_eq!(reopened.journal().len(), session.journal().len());

        // A writer that skips the capture is refused, whole, and leaves no trace.
        let before = (session.journal().len(), session.changes().len());
        let refused = session
            .transact_change(at, false, |candidate| {
                candidate
                    .edit_now(person(), |graph| {
                        graph.show_unrecorded(one, "https://unrecorded.test/");
                    })
                    .map(|_| ())
            })
            .await;
        assert!(matches!(refused, Err(SessionError::NotReplayable(_))));
        assert_eq!((session.journal().len(), session.changes().len()), before);
        assert_ne!(shown(session.graph()), "https://unrecorded.test/");
        let reopened = mere.open_with(id).await.unwrap();
        assert_eq!(reopened.journal().len(), before.0);
    });
}

#[test]
fn two_handles_flushing_from_one_saved_point_refuse_the_stale_one() {
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let mere = Sessions::new_with(store.clone());
        let id = mere.mint(person(), None).await.unwrap().session_id;
        let mut first = mere.open_with(id).await.unwrap();
        let mut second = mere.open_with(id).await.unwrap();
        first.apply_now(person(), add(1)).unwrap();
        second.apply_now(person(), add(2)).unwrap();

        first.flush(SystemTime::UNIX_EPOCH).await.unwrap();
        let refused = second.flush(SystemTime::UNIX_EPOCH).await;
        assert!(matches!(
            refused,
            Err(SessionError::Validation(ref why)) if why.contains("Session changed before commit")
        ));
        assert!(second.has_unstored(), "the refused edit stays pending");
        assert!(holds(&second, 2));
        let stored = mere.open_with(id).await.unwrap();
        assert!(holds(&stored, 1) && !holds(&stored, 2));

        let mut rebased = mere.open_with(id).await.unwrap();
        rebased.apply_now(person(), add(2)).unwrap();
        rebased.flush(SystemTime::UNIX_EPOCH).await.unwrap();
        let stored = mere.open_with(id).await.unwrap();
        assert!(holds(&stored, 1) && holds(&stored, 2));
    });
}

#[test]
fn a_transaction_carries_the_handles_unstored_entries() {
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let mere = Sessions::new_with(store.clone());
        let id = mere.mint(person(), None).await.unwrap().session_id;
        let mut session = mere.open_with(id).await.unwrap();
        session.apply_now(person(), add(1)).unwrap();
        assert!(session.has_unstored());

        session
            .transact_deltas(person(), add_delta(2), SystemTime::UNIX_EPOCH)
            .await
            .unwrap();
        assert!(!session.has_unstored());
        let stored = mere.open_with(id).await.unwrap();
        assert!(holds(&stored, 1) && holds(&stored, 2));
    });
}

fn add_delta(id: u128) -> Vec<Delta> {
    add(id)
        .iter()
        .map(|delta| Surface::edit_of(delta, &person()).unwrap())
        .collect()
}

#[test]
fn a_candidate_commits_extra_writes_and_refuses_a_key_that_changed() {
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let mere = Sessions::new_with(store.clone());
        let id = mere.mint(person(), None).await.unwrap().session_id;
        let mut session = mere.open_with(id).await.unwrap();
        store.put("side/ledger", b"one").await.unwrap();
        let extra = |read: &[u8], write: &[u8]| CommitExtra {
            ops: vec![WriteOp::Put {
                key: "side/ledger".into(),
                value: write.to_vec(),
            }],
            expect: vec![("side/ledger".into(), Some(read.to_vec()))],
            decide: None,
        };
        let at = SystemTime::UNIX_EPOCH;

        let mut candidate = session.detached_candidate();
        candidate.apply_now(person(), add(1)).unwrap();
        // The key the extra write was decided against moves under the candidate.
        store.put("side/ledger", b"other").await.unwrap();
        let refused = session
            .commit_candidate(candidate, at, false, extra(b"one", b"two"))
            .await;
        assert!(matches!(refused, Err(SessionError::Validation(_))));
        assert!(!holds(&session, 1) && !session.has_unstored());
        assert_eq!(store.get("side/ledger").await.unwrap().unwrap(), b"other");
        assert!(!holds(&mere.open_with(id).await.unwrap(), 1));

        let mut candidate = session.detached_candidate();
        candidate.apply_now(person(), add(1)).unwrap();
        session
            .commit_candidate(candidate, at, false, extra(b"other", b"two"))
            .await
            .unwrap();
        assert!(holds(&session, 1) && !session.has_unstored());
        assert_eq!(store.get("side/ledger").await.unwrap().unwrap(), b"two");
        assert!(holds(&mere.open_with(id).await.unwrap(), 1));
    });
}
