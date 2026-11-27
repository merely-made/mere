// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::test_surface::{PendingLinks, Retention as PendingLinkRetention, Surface};
use mere::model::{
    CoverageLayer, CoverageLimit, CoverageNote, FrozenSelection, NodeProperty,
    PersistedResourceFacet, PersistedResourceRecord, RESOURCE_PROPERTIES, ResourceNode,
};
use mere::projection::RdfSource;
use muniment::MemoryBackend;

type Sessions = MereSessions<MemoryBackend, Surface>;

/// Two surfaces, a statement between them, and one link held pending.
fn baseline(retention: PendingLinkRetention) -> Surface {
    let mut graph = Surface::default();
    let (source, shown) = (Uuid::from_u128(1), Uuid::from_u128(2));
    graph.show(source, "https://example.org/source");
    graph.show(shown, "https://example.org/held");
    graph.assert(source, shown, "https://example.org/vocab#cites");
    graph.queue_pending_link("https://example.org/absent");
    graph.set_pending_link_retention(retention);
    graph
}

fn truth(graph: &Surface) -> serde_json::Value {
    serde_json::to_value(graph.to_snapshot()).unwrap()
}

#[test]
fn session_only_history_and_reopen_release_inputs_without_losing_truth() {
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let sessions = Sessions::new_with(store);
        let mut session = sessions.begin_recorded_with(
            Author::user(),
            Some(baseline(PendingLinkRetention::SessionOnly)),
        );
        let id = session.manifest().session_id;
        let before = truth(session.graph());
        assert_eq!(session.graph().pending_links().len(), 1);
        assert!(session.graph_at(Seq(0)).unwrap().pending_links().is_empty());
        session.flush(SystemTime::now()).await.unwrap();
        drop(session);
        let reopened = sessions.open_with(id).await.unwrap();
        assert!(reopened.graph().pending_links().is_empty());
        assert_eq!(truth(reopened.graph()), before);
        assert_eq!(reopened.journal().live_cursor(), Seq(0));
    });
}

#[test]
fn until_purged_reopens_and_purge_stores_only_a_cache_change() {
    pollster::block_on(async {
        let sessions = Sessions::new_with(MemoryBackend::new());
        let mut session = sessions.begin_recorded_with(
            Author::user(),
            Some(baseline(PendingLinkRetention::UntilPurged)),
        );
        let id = session.manifest().session_id;
        let before = truth(session.graph());
        let inputs: PendingLinks = session.graph().pending_link_state().clone();
        session.flush(SystemTime::now()).await.unwrap();
        drop(session);
        let mut reopened = sessions.open_with(id).await.unwrap();
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
        let reopened = sessions.open_with(id).await.unwrap();
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
        let sessions = Sessions::new_with(MemoryBackend::new());
        let mut session = sessions.begin_recorded_with(
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
                graph.set_known_coverage(CoverageNote {
                    limits: vec![CoverageLimit::new(
                        CoverageLayer::Disclosure,
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
        let sessions = Sessions::new_with(backend.clone());
        let mut session = sessions.begin_recorded_with(
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
            .edit_now(Author::user(), |graph| graph.queue_pending_link(&source))
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
                .open_with(session.id())
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
        let sessions = Sessions::new_with(backend.clone());
        let mut session = sessions.begin_recorded_with(
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
            sessions.open_with(id).await,
            Err(SessionError::Corrupt(_))
        ));
    });
}

#[test]
fn cache_returning_to_saved_state_still_writes_after_an_older_batch() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = Sessions::new_with(backend.clone());
        let mut session = sessions.begin_recorded_with(
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
            .edit_now(Author::user(), |graph| graph.queue_pending_link(&original))
            .unwrap();
        let newer = session.pending(SystemTime::now()).unwrap();
        backend.apply(older.ops()).await.unwrap();
        backend.apply(newer.ops()).await.unwrap();
        session.stored(newer);
        session.stored(older);
        assert!(!session.has_unstored());
        assert_eq!(
            sessions
                .open_with(id)
                .await
                .unwrap()
                .graph()
                .pending_link_state(),
            session.graph().pending_link_state()
        );
    });
}

/// A resource owning a frozen selection of `members`, and its record.
fn frozen_owner(members: Vec<Uuid>) -> (Uuid, FrozenSelection, PersistedResourceRecord) {
    let selection = FrozenSelection::v1(
        serde_json::json!({"kind":"saved-selection"}),
        0,
        members,
        Vec::new(),
    );
    let iri = format!("urn:uuid:{}", Uuid::from_u128(0xf0));
    let record = PersistedResourceRecord {
        canonical_iri: iri.clone(),
        facets: vec![PersistedResourceFacet {
            facet: mere::model::FROZEN_SELECTION.into(),
            value_json: serde_json::to_string(&selection).unwrap(),
        }],
    };
    (ResourceNode::for_term(&iri).id(), selection, record)
}

#[test]
fn frozen_resource_selection_and_annotations_survive_recorded_save_reopen() {
    pollster::block_on(async {
        let sessions = Sessions::new_with(MemoryBackend::new());
        let base = baseline(PendingLinkRetention::SessionOnly);
        let members: Vec<_> = base.resources().resource_nodes().map(|node| node.id()).collect();
        let mut session = sessions.begin_recorded_with(Author::user(), Some(base));
        let id = session.manifest().session_id;
        let (owner, frozen, mut record) = frozen_owner(members);
        session
            .edit_now(Author::user(), |graph| {
                graph
                    .write_resource_record(&Author::user(), owner, Some(record.clone()))
                    .unwrap();
            })
            .unwrap();
        assert_eq!(
            session.graph().resources().resource(owner).unwrap().nested_selection(),
            Some(&frozen)
        );
        let annotation = NodeProperty {
            statement_id: "frozen-annotation".into(),
            predicate: "urn:test:annotation".into(),
            value: "keep this".into(),
            datatype: None,
            lang: None,
            graph_scope: Default::default(),
            provenance_iri: Some("urn:test:user".into()),
            asserted_at_ms: Some(53),
        };
        record.facets.push(PersistedResourceFacet {
            facet: RESOURCE_PROPERTIES.into(),
            value_json: serde_json::to_string(&[annotation]).unwrap(),
        });
        session
            .edit_now(Author::user(), |graph| {
                graph
                    .write_resource_record(&Author::user(), owner, Some(record))
                    .unwrap();
            })
            .unwrap();
        session.flush(SystemTime::now()).await.unwrap();
        drop(session);
        let reopened = sessions.open_with(id).await.unwrap();
        let resources = reopened.graph().resources();
        assert_eq!(
            resources.resource(owner).unwrap().nested_selection(),
            Some(&frozen)
        );
        assert_eq!(
            resources.resource_properties(owner)[0].statement_id,
            "frozen-annotation"
        );
        assert_eq!(resources.resource_properties(owner)[0].value, "keep this");
        assert!(
            reopened
                .graph()
                .coverage_for_resource_refs(frozen.members())
                .limits
                .is_empty()
        );
    });
}
