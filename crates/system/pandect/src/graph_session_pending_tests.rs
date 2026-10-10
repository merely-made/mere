// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use kernel::graph::{
    PendingLink, PendingLinkRetention, SemanticStatementSpec, SemanticSubKind, predicate_iri,
};
use muniment::MemoryBackend;

fn baseline(retention: PendingLinkRetention) -> Graph {
    let mut graph = Graph::new();
    let source = kernel::graph::apply::add_node(
        &mut graph,
        None,
        "https://example.org/source".into(),
        Default::default(),
    );
    let shown = kernel::graph::apply::add_node(
        &mut graph,
        None,
        "https://example.org/held".into(),
        Default::default(),
    );
    let source_id = graph.shown_resource_id(source).unwrap();
    graph
        .try_assert_semantic_statement(
            source,
            shown,
            SemanticStatementSpec {
                predicate: predicate_iri(SemanticSubKind::Cites).into(),
                recognized_sub_kind: Some(SemanticSubKind::Cites),
                ..Default::default()
            },
        )
        .unwrap();
    graph.queue_pending_link(PendingLink {
        source_resource: source_id,
        source_surface: None,
        target_iri: "https://example.org/absent".into(),
        statement: SemanticStatementSpec {
            predicate: predicate_iri(SemanticSubKind::Cites).into(),
            recognized_sub_kind: Some(SemanticSubKind::Cites),
            graph_scope: kernel::types::GraphScope::Source,
            provenance_iri: Some("https://example.org/source".into()),
            ..Default::default()
        },
    });
    graph.set_pending_link_retention(retention);
    graph
}

fn truth(graph: &Graph) -> serde_json::Value {
    let mut snapshot = graph.to_snapshot();
    snapshot.timestamp_secs = 0;
    serde_json::to_value(snapshot).unwrap()
}

#[test]
fn session_only_history_and_reopen_release_inputs_without_losing_truth() {
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let sessions = MereSessions::new(store);
        let mut session = sessions.begin_recorded(
            Author::user(),
            Some(baseline(PendingLinkRetention::SessionOnly)),
        );
        let id = session.manifest().session_id;
        let before = truth(session.graph());
        assert_eq!(session.graph().pending_links().len(), 1);
        assert!(session.graph_at(Seq(0)).unwrap().pending_links().is_empty());
        session.flush(SystemTime::now()).await.unwrap();
        drop(session);
        let reopened = sessions.open(id).await.unwrap();
        assert!(reopened.graph().pending_links().is_empty());
        assert_eq!(truth(reopened.graph()), before);
        assert_eq!(reopened.journal().live_cursor(), Seq(0));
    });
}

#[test]
fn until_purged_reopens_and_purge_stores_only_a_cache_change() {
    pollster::block_on(async {
        let sessions = MereSessions::new(MemoryBackend::new());
        let mut session = sessions.begin_recorded(
            Author::user(),
            Some(baseline(PendingLinkRetention::UntilPurged)),
        );
        let id = session.manifest().session_id;
        let before = truth(session.graph());
        let inputs = session.graph().pending_link_state().clone();
        session.flush(SystemTime::now()).await.unwrap();
        drop(session);
        let mut reopened = sessions.open(id).await.unwrap();
        assert_eq!(reopened.graph().pending_link_state(), &inputs);
        let revision = reopened.revision();
        let (_, applied) = reopened
            .edit_now(Author::user(), |graph| graph.purge_pending_links())
            .unwrap();
        assert_eq!(applied.first, applied.end);
        assert!(reopened.revision() > revision);
        assert!(reopened.has_unstored());
        assert_eq!(truth(reopened.graph()), before);
        reopened.flush(SystemTime::now()).await.unwrap();
        drop(reopened);
        let reopened = sessions.open(id).await.unwrap();
        assert!(reopened.graph().pending_links().is_empty());
        assert_eq!(
            reopened.graph().pending_link_state().retention,
            PendingLinkRetention::UntilPurged
        );
        assert_eq!(truth(reopened.graph()), before);
        assert_eq!(reopened.journal().live_cursor(), Seq(0));
    });
}

#[test]
fn cache_observations_are_revised_and_only_retained_entries_are_dirty() {
    pollster::block_on(async {
        let sessions = MereSessions::new(MemoryBackend::new());
        let mut session = sessions.begin_recorded(
            Author::user(),
            Some(baseline(PendingLinkRetention::SessionOnly)),
        );
        session.flush(SystemTime::now()).await.unwrap();
        let at = session.revision();
        let (_, applied) = session
            .edit_now(Author::user(), |graph| graph.purge_pending_links())
            .unwrap();
        assert!(session.revision() > at);
        assert_eq!(applied.first, applied.end);
        assert!(
            !session.has_unstored(),
            "session-only inputs do not dirty durable truth"
        );
        assert!(session.pending(SystemTime::now()).unwrap().is_empty());
        session
            .edit_now(Author::user(), |graph| {
                graph.set_known_coverage(kernel::graph::CoverageNote {
                    limits: vec![kernel::graph::CoverageLimit::new(
                        kernel::graph::CoverageLayer::Disclosure,
                        "host boundary",
                    )],
                })
            })
            .unwrap();
        assert!(!session.has_unstored());
        assert!(session.pending(SystemTime::now()).unwrap().is_empty());
        session
            .edit_now(Author::user(), |graph| {
                graph.set_pending_link_retention(PendingLinkRetention::UntilPurged)
            })
            .unwrap();
        assert!(
            session.has_unstored(),
            "the configured retention policy is durable"
        );
    });
}

#[test]
fn retained_cache_batches_track_post_batch_edits_and_reversed_receipts() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = MereSessions::new(backend.clone());
        let mut session = sessions.begin_recorded(
            Author::user(),
            Some(baseline(PendingLinkRetention::UntilPurged)),
        );
        session.flush(SystemTime::now()).await.unwrap();
        session
            .edit_now(Author::user(), |graph| graph.purge_pending_links())
            .unwrap();
        let older = session.pending(SystemTime::now()).unwrap();
        let source = baseline(PendingLinkRetention::UntilPurged).pending_links()[0].clone();
        session
            .edit_now(Author::user(), |graph| graph.queue_pending_link(source))
            .unwrap();
        backend.apply(older.ops()).await.unwrap();
        session.stored(older);
        assert!(
            session.has_unstored(),
            "editing after a batch remains dirty"
        );
        let older = session.pending(SystemTime::now()).unwrap();
        session
            .edit_now(Author::user(), |graph| graph.purge_pending_links())
            .unwrap();
        let newer = session.pending(SystemTime::now()).unwrap();
        backend.apply(older.ops()).await.unwrap();
        backend.apply(newer.ops()).await.unwrap();
        session.stored(newer);
        session.stored(older);
        assert!(
            !session.has_unstored(),
            "late receipt cannot replace the newer cache receipt"
        );
        assert_eq!(session.journal().live_cursor(), Seq(0));
        assert!(
            sessions
                .open(session.id())
                .await
                .unwrap()
                .graph()
                .pending_links()
                .is_empty()
        );
    });
}

#[test]
fn an_unsupported_retained_cache_version_is_refused() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = MereSessions::new(backend.clone());
        let mut session = sessions.begin_recorded(
            Author::user(),
            Some(baseline(PendingLinkRetention::UntilPurged)),
        );
        session.flush(SystemTime::now()).await.unwrap();
        let id = session.id();
        backend
            .put(
                &Keys::new(id).at(PENDING_LINKS),
                br#"{"version":2,"state":{"retention":"UntilPurged","entries":[]}}"#,
            )
            .await
            .unwrap();
        assert!(matches!(
            sessions.open(id).await,
            Err(SessionError::Corrupt(_))
        ));
    });
}

#[test]
fn cache_returning_to_saved_state_still_writes_after_an_older_batch() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = MereSessions::new(backend.clone());
        let mut session = sessions.begin_recorded(
            Author::user(),
            Some(baseline(PendingLinkRetention::UntilPurged)),
        );
        let id = session.id();
        session.flush(SystemTime::now()).await.unwrap();
        let original = session.graph().pending_links()[0].clone();
        session
            .edit_now(Author::user(), |graph| graph.purge_pending_links())
            .unwrap();
        let older = session.pending(SystemTime::now()).unwrap();
        session
            .edit_now(Author::user(), |graph| graph.queue_pending_link(original))
            .unwrap();
        let newer = session.pending(SystemTime::now()).unwrap();
        backend.apply(older.ops()).await.unwrap();
        backend.apply(newer.ops()).await.unwrap();
        session.stored(newer);
        session.stored(older);
        assert!(!session.has_unstored());
        assert_eq!(
            sessions
                .open(id)
                .await
                .unwrap()
                .graph()
                .pending_link_state(),
            session.graph().pending_link_state()
        );
    });
}

#[test]
fn frozen_resource_selection_and_annotations_survive_recorded_save_reopen() {
    pollster::block_on(async {
        let sessions = MereSessions::new(MemoryBackend::new());
        let base = baseline(PendingLinkRetention::SessionOnly);
        let members: Vec<_> = base.resource_nodes().map(|node| node.id()).collect();
        let mut session = sessions.begin_recorded(Author::user(), Some(base));
        let id = session.manifest().session_id;
        let (owner, _) = session
            .edit_now(Author::user(), |graph| {
                graph
                    .freeze_resource_selection(
                        serde_json::json!({"kind":"saved-selection"}),
                        members,
                    )
                    .unwrap()
            })
            .unwrap();
        let frozen = session
            .graph()
            .resource(owner)
            .unwrap()
            .nested_selection()
            .unwrap()
            .clone();
        session
            .edit_now(Author::user(), |graph| {
                graph
                    .append_resource_properties(
                        owner,
                        vec![kernel::types::NodeProperty {
                            statement_id: "frozen-annotation".into(),
                            predicate: "urn:test:annotation".into(),
                            value: "keep this".into(),
                            datatype: None,
                            lang: None,
                            graph_scope: Default::default(),
                            provenance_iri: Some("urn:test:user".into()),
                            asserted_at_ms: Some(53),
                        }],
                    )
                    .unwrap();
            })
            .unwrap();
        session.flush(SystemTime::now()).await.unwrap();
        drop(session);
        let reopened = sessions.open(id).await.unwrap();
        assert_eq!(
            reopened.graph().resource(owner).unwrap().nested_selection(),
            Some(&frozen)
        );
        assert_eq!(
            reopened.graph().resource_properties(owner)[0].statement_id,
            "frozen-annotation"
        );
        assert_eq!(
            reopened.graph().resource_properties(owner)[0].value,
            "keep this"
        );
        assert!(
            reopened
                .graph()
                .open_frozen_selection(owner)
                .unwrap()
                .coverage
                .limits
                .is_empty()
        );
    });
}
