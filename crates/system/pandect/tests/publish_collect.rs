// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A publication deletes, in its own transaction, what the head it supersedes
//! named and the new head does not, on both publishing paths, so a
//! single-owner store holds exactly what the head names; the first such
//! publication sweeps what earlier ones left, once, and leaves a host's keys;
//! a shared store collects
//! nothing (engine split D173).
#![cfg(not(target_arch = "wasm32"))]

use kernel::graph::Graph;
use kernel::graph::apply::GraphDelta;
use muniment::{Backend, MemoryBackend, StoreError, TransactFn, WriteOp};
use pandect::addressable_graph::{AddressableGraphStore, ResidencyPolicy, ResidencyRoots};
use pandect::{Author, CapturedDelta, GraphSession, MereSessions};
use std::collections::BTreeSet;
use std::time::SystemTime;
use uuid::Uuid;

/// A stand-in for a shared store: it forwards every call to memory but does
/// not forward `single_owner`, so it reads as shared, as IndexedDB does.
#[derive(Clone, Default)]
struct Shared(MemoryBackend);
#[async_trait::async_trait]
impl Backend for Shared {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        self.0.get(key).await
    }
    async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
        self.0.put(key, bytes).await
    }
    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        self.0.delete(key).await
    }
    async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        self.0.list(prefix).await
    }
    async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, StoreError> {
        self.0.scan(start, end).await
    }
    async fn apply(&self, ops: &[WriteOp]) -> Result<(), StoreError> {
        self.0.apply(ops).await
    }
    async fn transact(&self, f: TransactFn) -> Result<(), StoreError> {
        self.0.transact(f).await
    }
}

fn author() -> Author {
    Author::person("collect").via("graphshell")
}

fn prefix(graph_id: Uuid) -> String {
    format!("graphs/{graph_id}/resident/v1/")
}

/// Every key the store holds under the graph's resident prefix.
async fn stored(backend: &MemoryBackend, graph_id: Uuid) -> BTreeSet<String> {
    backend
        .list(&prefix(graph_id))
        .await
        .unwrap()
        .into_iter()
        .collect()
}

/// The keys a publication of `graph` writes: everything its catalog names,
/// the catalog itself, and `head`.
fn publication_keys(graph_id: Uuid, graph: &Graph) -> BTreeSet<String> {
    AddressableGraphStore::new(MemoryBackend::new(), graph_id)
        .prepare_recorded_graph(graph)
        .unwrap()
        .ops()
        .iter()
        .map(|op| match op {
            WriteOp::Put { key, .. } => key.clone(),
            WriteOp::Delete { key } => panic!("a prepared publication deletes {key}"),
        })
        .collect()
}

/// A fresh full reader opens `generation` and reads back `graph`.
async fn reads_back(backend: &MemoryBackend, graph_id: Uuid, graph: &Graph, generation: &str) {
    let view = AddressableGraphStore::new(backend.clone(), graph_id)
        .open(ResidencyRoots::default())
        .await
        .unwrap();
    assert_eq!(view.generation(), generation);
    let (read, source) = (view.graph().to_snapshot_at(0), graph.to_snapshot_at(0));
    assert_eq!(read.nodes.len(), source.nodes.len());
    assert_eq!(read.edges, source.edges);
    assert_eq!(read.resources, source.resources);
    assert_eq!(read.resource_edges, source.resource_edges);
    assert_eq!(view.graph().facets(), graph.facets());
}

/// The one graph a session published, by its keys.
async fn published_graph(backend: &MemoryBackend) -> Uuid {
    let ids: BTreeSet<_> = (backend.list("graphs/").await.unwrap().iter())
        .map(|key| key.split('/').nth(1).unwrap().to_owned())
        .collect();
    assert_eq!(ids.len(), 1, "{ids:?}");
    Uuid::parse_str(ids.first().unwrap()).unwrap()
}

fn adds(range: std::ops::RangeInclusive<u128>) -> Vec<GraphDelta> {
    range
        .map(|n| {
            CapturedDelta::ReplayAddNodeWithIdIfMissing {
                id: Uuid::from_u128(n).to_string(),
                url: format!("https://collect.test/{n}"),
                position: [0.0, 0.0],
            }
            .replay_as(&author())
            .into_write()
            .unwrap()
        })
        .collect()
}

fn title<B: Backend>(session: &GraphSession<B>, node: u128, title: &str) -> Vec<GraphDelta> {
    let key = session
        .graph()
        .get_node_key_by_id(Uuid::from_u128(node))
        .unwrap();
    vec![GraphDelta::SetNodeTitle {
        key,
        title: title.into(),
    }]
}

fn navigate(node: u128, tick: u64) -> Vec<GraphDelta> {
    vec![GraphDelta::ReplayNavigateNodeById {
        node_id: Uuid::from_u128(node),
        url: format!("https://collect.test/{node}/{tick}"),
        transition: kernel::graph::NodeHistoryTransitionKind::UrlTyped,
        timestamp_ms: tick,
        last_session_visited: 0,
    }]
}

fn remove<B: Backend>(session: &GraphSession<B>, node: u128) -> Vec<GraphDelta> {
    let key = session
        .graph()
        .get_node_key_by_id(Uuid::from_u128(node))
        .unwrap();
    vec![GraphDelta::RemoveNode { key }]
}

/// One publication on the commit path.
async fn commit<B: Backend + Clone + Sync>(
    session: &mut GraphSession<B>,
    deltas: Vec<GraphDelta>,
) -> String {
    let at = SystemTime::UNIX_EPOCH;
    session
        .transact_deltas_published(author(), deltas, at)
        .await
        .unwrap()
        .1
}

/// One publication on the `write_recorded_graph` path: an unpublished edit,
/// then the checkpoint's publication.
async fn checkpoint<B: Backend + Clone + Sync>(
    session: &mut GraphSession<B>,
    deltas: Vec<GraphDelta>,
) -> String {
    let at = SystemTime::UNIX_EPOCH;
    session.transact_deltas(author(), deltas, at).await.unwrap();
    session.publish_resident_checkpoint().await.unwrap()
}

/// What a collected store holds: a publication's keys and the `swept`
/// record its first collecting publication left.
fn collected(graph_id: Uuid, graph: &Graph) -> BTreeSet<String> {
    let mut keys = publication_keys(graph_id, graph);
    keys.insert(swept(graph_id));
    keys
}

fn swept(graph_id: Uuid) -> String {
    format!("{}swept", prefix(graph_id))
}

/// After each publication the reader opens it and the store holds exactly
/// what the head names, and the `swept` record.
async fn exact(backend: &MemoryBackend, session: &GraphSession<MemoryBackend>, generation: &str) {
    let graph_id = published_graph(backend).await;
    reads_back(backend, graph_id, session.graph(), generation).await;
    assert_eq!(
        stored(backend, graph_id).await,
        collected(graph_id, session.graph())
    );
}

#[test]
fn publications_on_both_paths_leave_exactly_what_the_head_names() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = MereSessions::new(backend.clone());
        let mut session = sessions.begin_recorded(author(), None);
        session.flush(SystemTime::UNIX_EPOCH).await.unwrap();

        let generation = commit(&mut session, adds(1..=8)).await;
        exact(&backend, &session, &generation).await;
        let graph_id = published_graph(&backend).await;

        // Superseded records, navigation rows and catalogs go on both paths.
        let generation = {
            let deltas = title(&session, 3, "first");
            commit(&mut session, deltas)
        }
        .await;
        exact(&backend, &session, &generation).await;
        let generation = {
            let deltas = title(&session, 5, "first");
            checkpoint(&mut session, deltas)
        }
        .await;
        exact(&backend, &session, &generation).await;
        let first = publication_keys(graph_id, session.graph());
        let generation = checkpoint(&mut session, navigate(4, 10)).await;
        exact(&backend, &session, &generation).await;
        let generation = commit(&mut session, navigate(4, 20)).await;
        exact(&backend, &session, &generation).await;
        let generation = {
            let deltas = title(&session, 3, "second");
            commit(&mut session, deltas)
        }
        .await;
        exact(&backend, &session, &generation).await;
        let generation = {
            let deltas = title(&session, 5, "second");
            checkpoint(&mut session, deltas)
        }
        .await;
        exact(&backend, &session, &generation).await;

        // A reverted digest: the record deleted when it was superseded is
        // written again, on each path.
        let generation = {
            let deltas = title(&session, 3, "first");
            commit(&mut session, deltas)
        }
        .await;
        exact(&backend, &session, &generation).await;
        let generation = {
            let deltas = title(&session, 5, "first");
            checkpoint(&mut session, deltas)
        }
        .await;
        exact(&backend, &session, &generation).await;
        let reverted = publication_keys(graph_id, session.graph());
        for node in [3, 5] {
            let surface = format!("/nodes/surface/{}/", Uuid::from_u128(node));
            let back: Vec<_> = (reverted.iter())
                .filter(|key| key.contains(&surface))
                .collect();
            assert_eq!(back.len(), 1);
            assert!(first.contains(back[0]), "{} returns", back[0]);
        }

        // A removed node's records go too.
        let generation = {
            let deltas = remove(&session, 8);
            commit(&mut session, deltas)
        }
        .await;
        exact(&backend, &session, &generation).await;
        assert!(
            !(stored(&backend, graph_id).await.iter())
                .any(|key| key.contains(&Uuid::from_u128(8).to_string()))
        );
        // The same graph again changes nothing and deletes nothing.
        let before = stored(&backend, graph_id).await;
        assert_eq!(
            session.publish_resident_checkpoint().await.unwrap(),
            generation
        );
        assert_eq!(stored(&backend, graph_id).await, before);

        // A cold reopen reads the same publication.
        let reopened = sessions.open(session.id()).await.unwrap();
        reads_back(&backend, graph_id, reopened.graph(), &generation).await;
    });
}

#[test]
fn a_reader_fails_without_a_record_the_head_names() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let mut session = MereSessions::new(backend.clone()).begin_recorded(author(), None);
        session.flush(SystemTime::UNIX_EPOCH).await.unwrap();
        commit(&mut session, adds(1..=4)).await;
        let generation = {
            let deltas = title(&session, 2, "changed");
            commit(&mut session, deltas)
        }
        .await;
        let graph_id = published_graph(&backend).await;
        let named = publication_keys(graph_id, session.graph());
        let record = (named.iter())
            .find(|key| key.contains(&format!("/nodes/surface/{}/", Uuid::from_u128(2))))
            .unwrap()
            .clone();
        let bytes = backend.get(&record).await.unwrap().unwrap();
        backend.delete(&record).await.unwrap();
        let store = AddressableGraphStore::new(backend.clone(), graph_id);
        let error = match store.open(ResidencyRoots::default()).await {
            Ok(_) => panic!("a reader opened without {record}"),
            Err(error) => error.to_string(),
        };
        assert!(error.contains("missing addressable record"), "{error}");
        backend.put(&record, &bytes).await.unwrap();
        reads_back(&backend, graph_id, session.graph(), &generation).await;
    });
}

#[test]
fn a_shared_store_collects_nothing() {
    pollster::block_on(async {
        let shared = Shared::default();
        let backend = shared.0.clone();
        let sessions = MereSessions::new(shared.clone());
        let mut session = sessions.begin_recorded(author(), None);
        session.flush(SystemTime::UNIX_EPOCH).await.unwrap();
        let first = commit(&mut session, adds(1..=4)).await;
        let graph_id = published_graph(&backend).await;
        let original = publication_keys(graph_id, session.graph());
        let second = {
            let deltas = title(&session, 2, "changed");
            commit(&mut session, deltas)
        }
        .await;
        reads_back(&backend, graph_id, session.graph(), &second).await;
        let third = {
            let deltas = title(&session, 3, "changed");
            checkpoint(&mut session, deltas)
        }
        .await;
        reads_back(&backend, graph_id, session.graph(), &third).await;
        let held = stored(&backend, graph_id).await;
        // Every key any publication wrote is still there, the first catalog
        // and the superseded records included.
        assert!(held.is_superset(&original));
        assert!(held.contains(&format!("{}catalog/{first}", prefix(graph_id))));
        assert!(held.is_superset(&publication_keys(graph_id, session.graph())));
        assert!(!held.contains(&swept(graph_id)));
        assert_eq!(held.len(), original.len() + 4, "{held:#?}");

        // The store path alone, declared shared, keeps the old catalog too.
        let store = AddressableGraphStore::new(shared, graph_id);
        let fourth = store.write_recorded_graph(&Graph::new()).await.unwrap();
        let after = stored(&backend, graph_id).await;
        assert!(after.is_superset(&held));
        reads_back(&backend, graph_id, &Graph::new(), &fourth).await;
    });
}

#[test]
fn garbage_from_before_collection_is_swept_once() {
    pollster::block_on(async {
        // A store published without collection, as every store was before:
        // the same map behind a wrapper that reads as shared.
        let uncollected = Shared::default();
        let backend = uncollected.0.clone();
        let mut session = MereSessions::new(uncollected).begin_recorded(author(), None);
        session.flush(SystemTime::UNIX_EPOCH).await.unwrap();
        commit(&mut session, adds(1..=6)).await;
        let deltas = title(&session, 2, "changed");
        commit(&mut session, deltas).await;
        let deltas = navigate(3, 10);
        checkpoint(&mut session, deltas).await;
        let deltas = remove(&session, 6);
        commit(&mut session, deltas).await;
        let graph_id = published_graph(&backend).await;
        let before = stored(&backend, graph_id).await;
        let named = publication_keys(graph_id, session.graph());
        let garbage = before.difference(&named).count();
        assert!(garbage >= 6, "{garbage} garbage keys");

        // A host's own keys beside the publication: Graphshell keeps its
        // resident settings at `host-settings` next to `policy`.
        let host = format!("{}host-settings", prefix(graph_id));
        backend.put(&host, b"{}").await.unwrap();
        AddressableGraphStore::new(backend.clone(), graph_id)
            .set_policy(ResidencyPolicy::Full)
            .await
            .unwrap();
        let before = stored(&backend, graph_id).await;
        let kept = [host, format!("{}policy", prefix(graph_id))];

        // The owner reopens it single-owner: its first publication sweeps, in
        // its own transaction, every record, navigation row, pending owner and
        // catalog the new head does not name, and nothing else.
        let sessions = MereSessions::new(backend.clone());
        let mut owner = sessions.open(session.id()).await.unwrap();
        assert_eq!(
            stored(&backend, graph_id).await,
            before,
            "open alone sweeps nothing"
        );
        let deltas = title(&owner, 4, "changed");
        let generation = commit(&mut owner, deltas).await;
        reads_back(&backend, graph_id, owner.graph(), &generation).await;
        let mut expected = collected(graph_id, owner.graph());
        expected.extend(kept.iter().cloned());
        assert_eq!(stored(&backend, graph_id).await, expected);

        // Once: a key written outside a publication after the sweep stays.
        let stray = format!(
            "{}nodes/surface/{}/stray",
            prefix(graph_id),
            Uuid::from_u128(4)
        );
        backend.put(&stray, b"stray").await.unwrap();
        let deltas = navigate(5, 20);
        let generation = checkpoint(&mut owner, deltas).await;
        reads_back(&backend, graph_id, owner.graph(), &generation).await;
        let mut expected = collected(graph_id, owner.graph());
        expected.extend(kept);
        expected.insert(stray);
        assert_eq!(stored(&backend, graph_id).await, expected);
    });
}
