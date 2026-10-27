// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A resident edit and its addressable publication share one atomic write
//! (resident host plan, Task 3 under Ruling 1). The fault backend stands in
//! for storage that refuses, or that commits and then reports an error.
#![cfg(not(target_arch = "wasm32"))]

use muniment::{Backend, MemoryBackend, StoreError, TransactFn, WriteOp};
use pandect::addressable_graph::ResidencyRoots;
use pandect::{Author, CapturedDelta, MereSessions, SessionError};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::SystemTime;
use uuid::Uuid;

const NORMAL: u8 = 0;
const REFUSE: u8 = 1;
const COMMIT_THEN_ERROR: u8 = 2;

#[derive(Clone)]
struct FaultBackend {
    inner: MemoryBackend,
    mode: Arc<AtomicU8>,
}
impl FaultBackend {
    fn new() -> Self {
        Self {
            inner: MemoryBackend::new(),
            mode: Arc::new(AtomicU8::new(NORMAL)),
        }
    }
    fn set(&self, mode: u8) {
        self.mode.store(mode, Ordering::SeqCst);
    }
}
#[async_trait::async_trait]
impl Backend for FaultBackend {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        self.inner.get(key).await
    }
    async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
        self.inner.put(key, bytes).await
    }
    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        self.inner.delete(key).await
    }
    async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        self.inner.list(prefix).await
    }
    async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, StoreError> {
        self.inner.scan(start, end).await
    }
    async fn apply(&self, ops: &[WriteOp]) -> Result<(), StoreError> {
        self.inner.apply(ops).await
    }
    async fn transact(&self, f: TransactFn) -> Result<(), StoreError> {
        match self.mode.load(Ordering::SeqCst) {
            REFUSE => Err(StoreError::Backend("injected transaction refusal".into())),
            COMMIT_THEN_ERROR => {
                self.inner.transact(f).await?;
                Err(StoreError::Backend("injected error after commit".into()))
            },
            _ => self.inner.transact(f).await,
        }
    }
}

async fn bytes(store: &MemoryBackend) -> BTreeMap<String, Vec<u8>> {
    let mut all = BTreeMap::new();
    for key in store.list("").await.unwrap() {
        all.insert(key.clone(), store.get(&key).await.unwrap().unwrap());
    }
    all
}

fn author() -> Author {
    Author::person("resident-test").via("graphshell")
}

fn add(n: u128) -> Vec<kernel::graph::apply::GraphDelta> {
    vec![
        CapturedDelta::ReplayAddNodeWithIdIfMissing {
            id: Uuid::from_u128(n).to_string(),
            url: format!("https://resident.test/{n}"),
            position: [0.0, 0.0],
        }
        .replay_as(&author())
        .into_write()
        .unwrap(),
    ]
}

async fn durable(
    backend: &FaultBackend,
) -> (pandect::GraphSession<FaultBackend>, MereSessions<FaultBackend>) {
    let sessions = MereSessions::new(backend.clone());
    let mut session = sessions.begin_recorded(author(), None);
    session.flush(SystemTime::UNIX_EPOCH).await.unwrap();
    (session, sessions)
}

#[test]
fn an_edit_and_its_publication_commit_together_or_not_at_all() {
    pollster::block_on(async {
        let backend = FaultBackend::new();
        let (mut session, sessions) = durable(&backend).await;
        let before = bytes(&backend.inner).await;
        let revision = session.revision();
        let cursor = session.journal().live_cursor();
        backend.set(REFUSE);
        assert!(
            session
                .transact_deltas_published(author(), add(1), SystemTime::UNIX_EPOCH)
                .await
                .unwrap_err()
                .to_string()
                .contains("injected transaction refusal")
        );
        // Neither the edit nor a publication exists, and the session is untouched.
        assert_eq!(bytes(&backend.inner).await, before);
        assert_eq!(session.graph().node_count(), 0);
        assert_eq!(session.revision(), revision);
        assert_eq!(session.journal().live_cursor(), cursor);
        backend.set(NORMAL);
        let (applied, generation) = session
            .transact_deltas_published(author(), add(1), SystemTime::UNIX_EPOCH)
            .await
            .unwrap();
        assert!(applied.end > applied.first);
        assert!(!session.has_unstored());
        let view = sessions
            .open_resident_graph(session.id(), ResidencyRoots::default())
            .await
            .unwrap();
        assert_eq!(view.generation(), generation);
        assert_eq!(view.graph().node_count(), 1);
        // The next edit moves the publication with the session.
        let (_, second) = session
            .transact_deltas_published(author(), add(2), SystemTime::UNIX_EPOCH)
            .await
            .unwrap();
        assert_ne!(second, generation);
        let view = sessions
            .open_resident_graph(session.id(), ResidencyRoots::default())
            .await
            .unwrap();
        assert_eq!(view.generation(), second);
        assert_eq!(view.graph().node_count(), 2);
    });
}

#[test]
fn a_commit_that_reports_an_error_is_refused_on_retry_and_never_repeated() {
    pollster::block_on(async {
        let backend = FaultBackend::new();
        let (mut session, sessions) = durable(&backend).await;
        let id = session.id();
        let changes = session.changes().len();
        backend.set(COMMIT_THEN_ERROR);
        assert!(
            session
                .transact_deltas_published(author(), add(1), SystemTime::UNIX_EPOCH)
                .await
                .unwrap_err()
                .to_string()
                .contains("injected error after commit")
        );
        // The host kept its acknowledged state; storage holds the edit.
        assert_eq!(session.graph().node_count(), 0);
        let committed = bytes(&backend.inner).await;
        // Retrying the same command is refused, visibly, and writes nothing.
        backend.set(NORMAL);
        let retry = session
            .transact_deltas_published(author(), add(1), SystemTime::UNIX_EPOCH)
            .await
            .unwrap_err();
        assert!(
            matches!(&retry, SessionError::Validation(m) if m.contains("Session changed before commit")),
            "{retry:?}"
        );
        assert_eq!(bytes(&backend.inner).await, committed);
        // Reopening shows the truth: one edit, one history entry, a matching publication.
        let reopened = sessions.open(id).await.unwrap();
        assert_eq!(reopened.graph().node_count(), 1);
        assert_eq!(reopened.changes().len(), changes + 1);
        let view = sessions
            .open_resident_graph(id, ResidencyRoots::default())
            .await
            .unwrap();
        assert_eq!(view.graph().node_count(), 1);
        // The reopened host is the one that can continue.
        let mut reopened = reopened;
        reopened
            .transact_deltas_published(author(), add(2), SystemTime::UNIX_EPOCH)
            .await
            .unwrap();
        assert_eq!(reopened.graph().node_count(), 2);
    });
}

#[test]
fn a_competing_writer_and_an_unqualified_session_leave_the_publication_alone() {
    pollster::block_on(async {
        let backend = FaultBackend::new();
        let (mut session, sessions) = durable(&backend).await;
        let mut competing = sessions.open(session.id()).await.unwrap();
        competing
            .transact_deltas_published(author(), add(7), SystemTime::UNIX_EPOCH)
            .await
            .unwrap();
        let durable_state = bytes(&backend.inner).await;
        assert!(
            session
                .transact_deltas_published(author(), add(8), SystemTime::UNIX_EPOCH)
                .await
                .unwrap_err()
                .to_string()
                .contains("Session changed before commit")
        );
        assert_eq!(bytes(&backend.inner).await, durable_state);
        assert_eq!(session.graph().node_count(), 0);

        // A session without recorded placement cannot publish; nothing is written.
        let legacy_backend = FaultBackend::new();
        let legacy_sessions = MereSessions::new(legacy_backend.clone());
        // An arbitrary baseline leaves placement unqualified.
        let mut legacy = legacy_sessions.begin(author(), Some(kernel::graph::Graph::new()));
        legacy.flush(SystemTime::UNIX_EPOCH).await.unwrap();
        let state = bytes(&legacy_backend.inner).await;
        let refused = legacy
            .transact_deltas_published(author(), add(9), SystemTime::UNIX_EPOCH)
            .await
            .unwrap_err();
        assert!(
            matches!(&refused, SessionError::Validation(m) if m.contains("qualified recorded placement")),
            "{refused:?}"
        );
        assert_eq!(bytes(&legacy_backend.inner).await, state);
        assert_eq!(legacy.graph().node_count(), 0);
    });
}

#[test]
fn undo_and_redo_publish_with_the_history_they_move() {
    pollster::block_on(async {
        let backend = FaultBackend::new();
        let (mut session, sessions) = durable(&backend).await;
        let (_, edited) = session
            .transact_deltas_published(author(), add(1), SystemTime::UNIX_EPOCH)
            .await
            .unwrap();
        let (reverted, undone) = session
            .transact_history_published(author(), false, SystemTime::UNIX_EPOCH)
            .await
            .unwrap();
        assert!(reverted.is_some());
        assert_ne!(undone, edited);
        let view = sessions
            .open_resident_graph(session.id(), ResidencyRoots::default())
            .await
            .unwrap();
        assert_eq!(view.generation(), undone);
        assert_eq!(view.graph().node_count(), 0);
        let (redone, again) = session
            .transact_history_published(author(), true, SystemTime::UNIX_EPOCH)
            .await
            .unwrap();
        assert!(redone.is_some());
        assert_eq!(again, edited, "the same graph publishes the same generation");
        backend.set(REFUSE);
        let held = bytes(&backend.inner).await;
        assert!(
            session
                .transact_history_published(author(), false, SystemTime::UNIX_EPOCH)
                .await
                .is_err()
        );
        assert_eq!(bytes(&backend.inner).await, held);
        assert_eq!(session.graph().node_count(), 1);
    });
}

fn pin(surface: Uuid, digest: &str) -> pandect::live_graph::ObservedCommand {
    use pandect::addressable_graph::GraphAddress;
    use pandect::live_graph::{LiveCheck, LiveCommand, ObservedCommand};
    ObservedCommand {
        command: LiveCommand::SetPinned {
            surface,
            pinned: true,
        },
        checks: vec![LiveCheck::RecordUnchanged {
            address: GraphAddress::Surface(surface),
            digest: digest.into(),
        }],
    }
}

#[test]
fn a_live_edit_is_observed_published_and_refused_whole_on_any_fault() {
    pollster::block_on(async {
        let backend = FaultBackend::new();
        let (mut session, sessions) = durable(&backend).await;
        let id = Uuid::from_u128(5);
        session
            .transact_deltas_published(author(), add(5), SystemTime::UNIX_EPOCH)
            .await
            .unwrap();
        // The observation is read from the published reader, as a display would.
        let view = sessions
            .open_resident_graph(session.id(), ResidencyRoots::default())
            .await
            .unwrap();
        let digest = view
            .record_digest(pandect::addressable_graph::GraphAddress::Surface(id))
            .unwrap()
            .to_string();
        let before = bytes(&backend.inner).await;
        // A stale observation is refused before any write.
        let stale = session
            .transact_live(author(), vec![pin(id, "not-the-record")], SystemTime::UNIX_EPOCH)
            .await
            .unwrap_err();
        assert!(matches!(stale, pandect::live_graph::LiveEditError::Stale(_)), "{stale:?}");
        // A storage refusal leaves the session, history and storage alone.
        backend.set(REFUSE);
        let cursor = session.journal().live_cursor();
        assert!(
            session
                .transact_live(author(), vec![pin(id, &digest)], SystemTime::UNIX_EPOCH)
                .await
                .is_err()
        );
        assert_eq!(bytes(&backend.inner).await, before);
        assert_eq!(session.journal().live_cursor(), cursor);
        let key = session.graph().get_node_key_by_id(id).unwrap();
        assert_eq!(session.graph().node_is_pinned(key), Some(false));
        // The same observation, once storage accepts, applies and publishes.
        backend.set(NORMAL);
        let (live, generation) = session
            .transact_live(author(), vec![pin(id, &digest)], SystemTime::UNIX_EPOCH)
            .await
            .unwrap();
        assert!(live.applied.end > live.applied.first);
        let key = session.graph().get_node_key_by_id(id).unwrap();
        assert_eq!(session.graph().node_is_pinned(key), Some(true));
        let view = sessions
            .open_resident_graph(session.id(), ResidencyRoots::default())
            .await
            .unwrap();
        assert_eq!(view.generation(), generation);
        // The observation is spent: the record changed, so it cannot be replayed.
        let replay = session
            .transact_live(author(), vec![pin(id, &digest)], SystemTime::UNIX_EPOCH)
            .await
            .unwrap_err();
        assert!(matches!(replay, pandect::live_graph::LiveEditError::Stale(_)), "{replay:?}");
    });
}
