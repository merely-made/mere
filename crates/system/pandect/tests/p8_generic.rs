// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! pandect's transaction, operation and page-claims families over a graph
//! with no kernel (engine split D257): the minimal surface of D245, publishing
//! through its own hook and writing resource records through the engine's one
//! recorded write.

mod support;

use std::time::{SystemTime, UNIX_EPOCH};

use mere::ContextCache;
use mere::acquire::page_claims::{PageClaims, interpret};
use mere::model::Author;
use muniment::{Backend, MemoryBackend};
use oxrdf::NamedNode;
use pandect::{GraphSession, MereSessions, SessionError, SessionId};
use support::{PUBLICATION, Surface, UNPUBLISHABLE, generation_of, head_key, show_edits};
use uuid::Uuid;

type Sessions = MereSessions<MemoryBackend, Surface>;

async fn minted(sessions: &Sessions) -> (SessionId, GraphSession<MemoryBackend, Surface>) {
    let id = sessions
        .mint(Author::person("p8-generic"), None)
        .await
        .unwrap()
        .session_id;
    (id, sessions.open_with(id).await.unwrap())
}

async fn stored(backend: &MemoryBackend) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    for key in backend.list("").await.unwrap() {
        let value = backend.get(&key).await.unwrap().unwrap();
        out.push((key, value));
    }
    out.sort();
    out
}

#[test]
fn a_published_transaction_writes_the_hooks_publication_with_its_edit() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = Sessions::new_with(backend.clone()).with_publication(&PUBLICATION);
        let (id, mut session) = minted(&sessions).await;
        let container = *session.manifest().root_graph_id.as_uuid();
        let author = Author::person("p8-generic");
        let (resource, edits) = show_edits(Uuid::from_u128(0xa1), "https://p8.test/published");

        let (applied, generation) = session
            .transact_deltas_published(author.clone(), edits, SystemTime::now())
            .await
            .unwrap();
        assert_eq!((applied.first.0, applied.end.0), (0, 2));
        assert_eq!(session.graph().shown(Uuid::from_u128(0xa1)), Some(resource));
        // The hook named the edited graph, and its write landed with the edit.
        assert_eq!(generation, generation_of(session.graph()));
        assert_eq!(
            backend.get(&head_key(container)).await.unwrap(),
            Some(generation.clone().into_bytes())
        );
        drop(session);
        let reopened = sessions.open_with(id).await.unwrap();
        assert_eq!(reopened.journal().live_cursor().0, 2);
        assert_eq!(generation_of(reopened.graph()), generation);

        // Positive control: the same surface with no hook installed refuses
        // to publish, and the store does not move.
        let bare = Sessions::new_with(backend.clone());
        let mut unhooked = bare.open_with(id).await.unwrap();
        let before = stored(&backend).await;
        let (_, more) = show_edits(Uuid::from_u128(0xa2), "https://p8.test/refused");
        let refused = unhooked
            .transact_deltas_published(author.clone(), more.clone(), SystemTime::now())
            .await;
        assert!(
            matches!(&refused, Err(SessionError::Validation(why)) if why.contains("no publication hook")),
            "{refused:?}"
        );
        assert_eq!(stored(&backend).await, before);
        assert_eq!(unhooked.journal().live_cursor().0, 2);
        // The unpublished path still writes without a hook.
        unhooked
            .transact_deltas(author.clone(), more, SystemTime::now())
            .await
            .unwrap();
        assert_eq!(unhooked.journal().live_cursor().0, 4);
        assert_eq!(
            backend.get(&head_key(container)).await.unwrap(),
            Some(generation.clone().into_bytes()),
            "an unpublished write leaves the publication"
        );
        drop(unhooked);

        // And the hook's own refusal leaves graph, journal and store as they were.
        let mut hooked = sessions.open_with(id).await.unwrap();
        let before = stored(&backend).await;
        let (_, bad) = show_edits(UNPUBLISHABLE, "https://p8.test/unpublishable");
        let refused = hooked
            .transact_deltas_published(author, bad, SystemTime::now())
            .await;
        assert!(
            matches!(&refused, Err(SessionError::Validation(why)) if why == "surface refuses publication"),
            "{refused:?}"
        );
        assert_eq!(stored(&backend).await, before);
        assert_eq!(hooked.graph().shown(UNPUBLISHABLE), None);
        assert_eq!(hooked.journal().live_cursor().0, 4);
    });
}

#[test]
fn an_operation_is_retained_with_its_publication_and_reconciles() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = Sessions::new_with(backend.clone()).with_publication(&PUBLICATION);
        let (id, mut session) = minted(&sessions).await;
        let author = Author::person("p8-generic");
        let operation = [7u8; 32];
        let (_, edits) = show_edits(Uuid::from_u128(0xb1), "https://p8.test/operation");

        let receipt = session
            .transact_operation_published(
                operation,
                b"request".to_vec(),
                b"result".to_vec(),
                author.clone(),
                edits.clone(),
                SystemTime::now(),
            )
            .await
            .unwrap();
        assert_eq!(receipt.generation, generation_of(session.graph()));
        assert_eq!(
            session.operation_receipt(operation).await.unwrap(),
            Some(receipt.clone())
        );
        let revision = session.operation_revision().unwrap();
        let cursor = session.journal().live_cursor();

        // The same request is answered by its receipt, editing nothing twice;
        // another request under the identity is refused.
        let again = session
            .transact_operation_published(
                operation,
                b"request".to_vec(),
                b"result".to_vec(),
                author.clone(),
                edits.clone(),
                SystemTime::now(),
            )
            .await
            .unwrap();
        assert_eq!(again, receipt);
        assert_eq!(session.journal().live_cursor(), cursor);
        assert!(
            session
                .transact_operation_published(
                    operation,
                    b"other".to_vec(),
                    b"result".to_vec(),
                    author,
                    edits,
                    SystemTime::now(),
                )
                .await
                .is_err()
        );

        // Reconciliation reopens through the session's hooks and reads the
        // generation the store holds.
        let mut stale = sessions.open_with(id).await.unwrap();
        assert_eq!(stale.operation_revision().unwrap(), revision);
        let (held, generation) = stale.reconcile_operation(operation).await.unwrap().unwrap();
        assert_eq!(held, receipt);
        assert_eq!(generation, receipt.generation);
        assert_eq!(stale.journal().live_cursor(), cursor);
        assert!(
            stale
                .reconcile_operation([8u8; 32])
                .await
                .unwrap()
                .is_none()
        );

        // Positive control: an unhooked handle has no revision to name.
        let bare = Sessions::new_with(backend).open_with(id).await.unwrap();
        assert!(bare.operation_revision().is_err());
    });
}

#[test]
fn page_claims_are_kept_through_the_engines_recorded_resource_write() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = Sessions::new_with(backend.clone());
        let (id, mut session) = minted(&sessions).await;
        let author = Author::person("p8-generic");
        let source = NamedNode::new_unchecked("urn:fixture:p8-kept-capture");
        let block = r##"{"@id":"#river","http://example.test/source":"surface claim"}"##;
        let claims = interpret(
            &ContextCache::new(),
            "https://example.test/notes",
            [(5, block)],
        );
        let before = session.journal().live_cursor();
        assert!(session.kept_page_claims(&source).unwrap().is_none());

        let kept = session
            .admit_page_claims(author.clone(), &source, &claims, UNIX_EPOCH)
            .await
            .unwrap();
        let cursor = session.journal().live_cursor();
        // One recorded resource write, journaled as the engine's own delta.
        assert_eq!(cursor.0, before.0 + 1);
        assert!(matches!(
            session.journal().entries()[before.index()].delta,
            mere::model::JournalDelta::Resource(
                mere::model::ResourceDelta::ReplaySetResourceRecordById { .. }
            )
        ));
        assert!(
            session
                .pin(before)
                .unwrap()
                .kept_page_claims(&source)
                .unwrap()
                .is_none()
        );
        // An identical retry edits nothing; different claims are refused.
        let retry = session
            .admit_page_claims(author.clone(), &source, &claims, UNIX_EPOCH)
            .await
            .unwrap();
        assert_eq!(retry, kept);
        assert_eq!(session.journal().live_cursor(), cursor);
        assert!(
            session
                .admit_page_claims(author, &source, &PageClaims::default(), UNIX_EPOCH)
                .await
                .is_err()
        );
        assert_eq!(session.journal().live_cursor(), cursor);
        drop(session);

        let reopened = sessions.open_with(id).await.unwrap();
        let read = reopened
            .pin(cursor)
            .unwrap()
            .kept_page_claims(&source)
            .unwrap()
            .unwrap();
        assert_eq!(read, kept);
        assert_eq!(read.claims.blocks[0].order, 5);
    });
}
