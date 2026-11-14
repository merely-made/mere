// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A checkpoint is read back only beside the journal prefix it names.

use super::*;
use kernel::graph::apply::add_node;
use muniment::MemoryBackend;

fn baseline() -> Graph {
    let mut graph = Graph::new();
    for (id, url) in [(1, "https://one.test/"), (2, "https://two.test/")] {
        add_node(
            &mut graph,
            Some(Uuid::from_u128(id)),
            url.into(),
            Default::default(),
        );
    }
    graph
}

/// A recorded session whose first node is titled `title`, checkpointed.
async fn checkpointed(sessions: &MereSessions<MemoryBackend>, title: &str) -> SessionId {
    let author = Author::person("checkpoint");
    let mut session = sessions.begin_recorded(author.clone(), Some(baseline()));
    session
        .apply(
            author,
            vec![CapturedDelta::ReplaySetNodeTitleById {
                node_id: Uuid::from_u128(1).to_string(),
                title: title.into(),
            }],
        )
        .await
        .unwrap();
    session.checkpoint().await.unwrap();
    session.id()
}

fn title_of(session: &GraphSession<MemoryBackend>) -> String {
    session
        .graph()
        .get_node_by_id(Uuid::from_u128(1))
        .unwrap()
        .1
        .title
        .clone()
}

async fn stored(backend: &MemoryBackend, id: SessionId, name: &str) -> Vec<u8> {
    backend
        .get(&Keys::new(id).at(name))
        .await
        .unwrap()
        .expect("stored")
}

async fn put(backend: &MemoryBackend, id: SessionId, name: &str, value: Vec<u8>) {
    let key = Keys::new(id).at(name);
    backend.apply(&[WriteOp::Put { key, value }]).await.unwrap();
}

#[test]
fn a_checkpoint_names_the_journal_prefix_and_the_files_it_reflects_and_opens_beside_them() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = MereSessions::new(backend.clone());
        let id = checkpointed(&sessions, "alpha").await;

        let record: serde_json::Value =
            serde_json::from_slice(&stored(&backend, id, CHECKPOINT).await).unwrap();
        let opened = sessions.open(id).await.unwrap();
        let cursor = opened.journal().live_cursor();
        let digest = opened
            .journal()
            .log()
            .chain_digest(cursor.index())
            .unwrap()
            .unwrap();
        assert_eq!(record["journal_digest"], digest.to_hex());
        assert_eq!(record["journal_digest_version"], 2);
        let contents = contents_digest(
            &stored(&backend, id, GRAPH).await,
            &stored(&backend, id, NODE_FACETS_FILE).await,
        );
        assert_eq!(record["contents_digest"], contents);
        assert_eq!(title_of(&opened), "alpha");
    });
}

/// Point `id`'s checkpoint at `from`'s graph and facets files, naming the
/// journal by `digest` under `version`; `None` writes no version, as every
/// checkpoint before engine-split D154 did.
async fn marked(
    backend: &MemoryBackend,
    id: SessionId,
    from: SessionId,
    digest: &str,
    version: Option<u8>,
) {
    let graph = stored(backend, from, GRAPH).await;
    let facets = stored(backend, from, NODE_FACETS_FILE).await;
    let mut record: serde_json::Value =
        serde_json::from_slice(&stored(backend, id, CHECKPOINT).await).unwrap();
    let fields = record.as_object_mut().unwrap();
    fields.insert("journal_digest".into(), digest.into());
    fields.insert(
        "contents_digest".into(),
        contents_digest(&graph, &facets).into(),
    );
    match version {
        Some(version) => fields.insert("journal_digest_version".into(), version.into()),
        None => fields.remove("journal_digest_version"),
    };
    put(backend, id, GRAPH, graph).await;
    put(backend, id, NODE_FACETS_FILE, facets).await;
    put(
        backend,
        id,
        CHECKPOINT,
        serde_json::to_vec(&record).unwrap(),
    )
    .await;
}

/// The v1 and v2 digests (hex) of `id`'s journal at its live cursor.
async fn digests_of(sessions: &MereSessions<MemoryBackend>, id: SessionId) -> (String, String) {
    let opened = sessions.open(id).await.unwrap();
    let log = opened.journal().log();
    let cursor = opened.journal().live_cursor().index();
    (
        log.prefix_digest(cursor).unwrap().unwrap().to_hex(),
        log.chain_digest(cursor).unwrap().unwrap().to_hex(),
    )
}

#[test]
fn a_v1_era_checkpoint_is_read_by_its_v1_digest_and_rewritten_as_v2() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = MereSessions::new(backend.clone());
        let id = checkpointed(&sessions, "alpha").await;
        // Its files come from another session, so reading them shows as "beta".
        let other = checkpointed(&sessions, "beta").await;
        let (v1, v2) = digests_of(&sessions, id).await;
        assert_ne!(v1, v2);
        marked(&backend, id, other, &v1, None).await;

        let mut opened = sessions.open(id).await.unwrap();
        assert_eq!(title_of(&opened), "beta", "the v1 checkpoint was read");
        assert_eq!(opened.journal().live_cursor(), opened.checkpointed);
        opened.checkpoint_if_behind().await.unwrap();
        let record: serde_json::Value =
            serde_json::from_slice(&stored(&backend, id, CHECKPOINT).await).unwrap();
        assert_eq!(record["journal_digest"], v2.as_str(), "rewritten as v2");
        assert_eq!(record["journal_digest_version"], 2);

        let reopened = sessions.open(id).await.unwrap();
        assert_eq!(title_of(&reopened), "beta", "the v2 checkpoint is read");
        assert!(!reopened.checkpoint_v1, "nothing left to rewrite");
    });
}

#[test]
fn a_checkpoint_whose_digest_is_not_of_its_version_is_replayed_past() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = MereSessions::new(backend.clone());
        let id = checkpointed(&sessions, "alpha").await;
        let other = checkpointed(&sessions, "beta").await;
        let (v1, v2) = digests_of(&sessions, id).await;
        // Positive control: each digest under its own version is read.
        for (digest, version) in [(&v1, None), (&v2, Some(2))] {
            marked(&backend, id, other, digest, version).await;
            assert_eq!(title_of(&sessions.open(id).await.unwrap()), "beta");
        }
        for (digest, version) in [(&v1, Some(2)), (&v2, None), (&v2, Some(3))] {
            marked(&backend, id, other, digest, version).await;
            let opened = sessions.open(id).await.unwrap();
            assert_eq!(title_of(&opened), "alpha", "{version:?} replayed past");
        }
    });
}

#[test]
fn a_checkpoint_taken_beside_another_journal_is_replayed_past() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = MereSessions::new(backend.clone());
        let ours = checkpointed(&sessions, "alpha").await;
        // The same number of entries, from a different history.
        let theirs = checkpointed(&sessions, "beta").await;
        for name in [GRAPH, NODE_FACETS_FILE, CHECKPOINT] {
            put(&backend, ours, name, stored(&backend, theirs, name).await).await;
        }

        let mut opened = sessions
            .open(ours)
            .await
            .expect("not this journal's checkpoint");
        assert_eq!(title_of(&opened), "alpha");
        opened.checkpoint().await.unwrap();
        assert_eq!(title_of(&sessions.open(ours).await.unwrap()), "alpha");
    });
}

#[test]
fn a_checkpoint_whose_graph_and_facets_were_replaced_is_replayed_past() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = MereSessions::new(backend.clone());
        let id = checkpointed(&sessions, "alpha").await;
        // The audit's probe: only the two files change, the record is the session's own.
        let mut foreign = Graph::new();
        add_node(
            &mut foreign,
            Some(Uuid::from_u128(999)),
            "https://foreign.test/".into(),
            Default::default(),
        );
        put(
            &backend,
            id,
            GRAPH,
            serde_json::to_vec(&foreign.to_snapshot()).unwrap(),
        )
        .await;
        put(
            &backend,
            id,
            NODE_FACETS_FILE,
            serde_json::to_vec(foreign.facets()).unwrap(),
        )
        .await;

        let mut opened = sessions.open(id).await.expect("replayed past, not refused");
        assert_eq!(title_of(&opened), "alpha");
        assert!(
            opened
                .graph()
                .get_node_by_id(Uuid::from_u128(999))
                .is_none()
        );

        opened.checkpoint().await.unwrap();
        let reopened = sessions.open(id).await.unwrap();
        assert_eq!(title_of(&reopened), "alpha");
        assert!(
            reopened
                .graph()
                .get_node_by_id(Uuid::from_u128(999))
                .is_none()
        );
    });
}

#[test]
fn a_checkpoint_naming_no_files_is_replayed_past() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = MereSessions::new(backend.clone());
        let id = checkpointed(&sessions, "alpha").await;
        let mut record: serde_json::Value =
            serde_json::from_slice(&stored(&backend, id, CHECKPOINT).await).unwrap();
        record.as_object_mut().unwrap().remove("contents_digest");
        put(
            &backend,
            id,
            CHECKPOINT,
            serde_json::to_vec(&record).unwrap(),
        )
        .await;
        let mut foreign = Graph::new();
        add_node(
            &mut foreign,
            Some(Uuid::from_u128(999)),
            "https://foreign.test/".into(),
            Default::default(),
        );
        put(
            &backend,
            id,
            GRAPH,
            serde_json::to_vec(&foreign.to_snapshot()).unwrap(),
        )
        .await;

        let opened = sessions.open(id).await.unwrap();
        assert_eq!(title_of(&opened), "alpha");
        assert!(
            opened
                .graph()
                .get_node_by_id(Uuid::from_u128(999))
                .is_none()
        );
    });
}

#[test]
fn a_checkpoint_without_a_journal_digest_is_ignored_for_a_full_replay_and_the_next_names_the_journal()
 {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = MereSessions::new(backend.clone());
        let id = checkpointed(&sessions, "alpha").await;
        let mut record: serde_json::Value =
            serde_json::from_slice(&stored(&backend, id, CHECKPOINT).await).unwrap();
        record.as_object_mut().unwrap().remove("journal_digest");
        put(
            &backend,
            id,
            CHECKPOINT,
            serde_json::to_vec(&record).unwrap(),
        )
        .await;
        // Its graph is another's: a checkpoint that names no journal is not trusted.
        let mut foreign = Graph::new();
        add_node(
            &mut foreign,
            Some(Uuid::from_u128(999)),
            "https://foreign.test/".into(),
            Default::default(),
        );
        put(
            &backend,
            id,
            GRAPH,
            serde_json::to_vec(&foreign.to_snapshot()).unwrap(),
        )
        .await;
        put(
            &backend,
            id,
            NODE_FACETS_FILE,
            serde_json::to_vec(foreign.facets()).unwrap(),
        )
        .await;

        let mut opened = sessions
            .open(id)
            .await
            .expect("a digest-less checkpoint is skipped");
        assert_eq!(title_of(&opened), "alpha");
        assert!(
            opened
                .graph()
                .get_node_by_id(Uuid::from_u128(999))
                .is_none()
        );

        opened.checkpoint().await.unwrap();
        let record: serde_json::Value =
            serde_json::from_slice(&stored(&backend, id, CHECKPOINT).await).unwrap();
        assert!(
            record["journal_digest"].is_string(),
            "the new checkpoint names its journal"
        );
        let reopened = sessions.open(id).await.unwrap();
        assert_eq!(title_of(&reopened), "alpha");
        assert!(
            reopened
                .graph()
                .get_node_by_id(Uuid::from_u128(999))
                .is_none()
        );
    });
}
