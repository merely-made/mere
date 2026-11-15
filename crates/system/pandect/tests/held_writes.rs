// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A publication leaves out every write the current head already holds, on
//! both publishing paths, and a full reader opens each one (engine split
//! D171). The counting backend records every batch a store applies or commits.
#![cfg(not(target_arch = "wasm32"))]

use kernel::graph::Graph;
use kernel::graph::apply::{self, GraphDelta, apply_graph_delta};
use muniment::{Backend, MemoryBackend, StoreError, TransactFn, WriteOp};
use pandect::addressable_graph::{AddressableGraphStore, GraphAddress, ResidencyRoots};
use pandect::{Author, CapturedDelta, MereSessions};
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Clone, Default)]
struct Counting {
    inner: MemoryBackend,
    batches: Arc<Mutex<Vec<Vec<WriteOp>>>>,
}
impl Counting {
    /// The keys the last batch put under `prefix`.
    fn last_puts(&self, prefix: &str) -> BTreeSet<String> {
        let batches = self.batches.lock().unwrap();
        batches
            .last()
            .unwrap()
            .iter()
            .filter_map(|op| match op {
                WriteOp::Put { key, .. } if key.starts_with(prefix) => Some(key.clone()),
                _ => None,
            })
            .collect()
    }
}
#[async_trait::async_trait]
impl Backend for Counting {
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
        self.batches.lock().unwrap().push(ops.to_vec());
        self.inner.apply(ops).await
    }
    fn single_owner(&self) -> bool {
        self.inner.single_owner()
    }
    async fn transact(&self, f: TransactFn) -> Result<(), StoreError> {
        let batches = self.batches.clone();
        self.inner
            .transact(Box::new(move |reader| {
                let ops = f(reader);
                batches.lock().unwrap().push(ops.clone());
                ops
            }))
            .await
    }
}

fn prefix(graph_id: Uuid) -> String {
    format!("graphs/{graph_id}/resident/v1/")
}

/// The keys a publication of `after` must write over one of `before`: each
/// record whose digest moved, the new catalog and the head.
fn expected_writes(graph_id: Uuid, before: &Graph, after: &Graph) -> BTreeSet<String> {
    let store = AddressableGraphStore::new(MemoryBackend::new(), graph_id);
    let (old, new) = (
        store.prepare_recorded_graph(before).unwrap(),
        store.prepare_recorded_graph(after).unwrap(),
    );
    let mut keys = BTreeSet::from([
        format!("{}head", prefix(graph_id)),
        format!("{}catalog/{}", prefix(graph_id), new.generation()),
    ]);
    let snapshot = after.to_snapshot_at(0);
    let surfaces = (snapshot.nodes.iter())
        .map(|node| GraphAddress::Surface(Uuid::parse_str(&node.node_id).unwrap()));
    let resources = after
        .resource_nodes()
        .map(|r| GraphAddress::Resource(r.id()));
    for address in surfaces.chain(resources) {
        let digest = new.record_digest(address).unwrap();
        if old.record_digest(address) != Some(digest) {
            let layer = match address {
                GraphAddress::Surface(_) => "surface",
                GraphAddress::Resource(_) => "resource",
            };
            let id = address.id();
            keys.insert(format!("{}nodes/{layer}/{id}/{digest}", prefix(graph_id)));
        }
    }
    keys
}

/// The puts of a store's first collecting publication: `writes` and the
/// record of its one sweep (engine split D173).
fn first_collecting(graph_id: Uuid, mut writes: BTreeSet<String>) -> BTreeSet<String> {
    writes.insert(format!("{}swept", prefix(graph_id)));
    writes
}

/// A full reader opens the head at `generation` and sees `graph`.
async fn reads_back(backend: &Counting, graph_id: Uuid, graph: &Graph, generation: &str) {
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

fn graph(count: u128) -> Graph {
    let mut graph = Graph::new();
    for i in 1..=count {
        apply::add_node(
            &mut graph,
            Some(Uuid::from_u128(i)),
            format!("https://held.test/{i}"),
            Default::default(),
        );
    }
    graph
}

fn retitled(graph: &Graph, node: u128) -> Graph {
    let mut changed = graph.clone();
    let key = changed.get_node_key_by_id(Uuid::from_u128(node)).unwrap();
    apply_graph_delta(
        &mut changed,
        GraphDelta::SetNodeTitle {
            key,
            title: "changed".into(),
        },
    );
    changed
}

#[test]
fn write_recorded_graph_leaves_out_what_the_head_holds() {
    pollster::block_on(async {
        let backend = Counting::default();
        let graph_id = Uuid::from_u128(0xd171);
        let store = AddressableGraphStore::new(backend.clone(), graph_id);
        let source = graph(8);
        let first = store.write_recorded_graph(&source).await.unwrap();
        // The first publication writes everything: 16 records, catalog, head.
        let everything = expected_writes(graph_id, &Graph::new(), &source);
        assert_eq!(everything.len(), 18);
        assert_eq!(
            backend.last_puts(&prefix(graph_id)),
            first_collecting(graph_id, everything)
        );
        reads_back(&backend, graph_id, &source, &first).await;

        // The same graph again: only the head.
        assert_eq!(store.write_recorded_graph(&source).await.unwrap(), first);
        assert_eq!(
            backend.last_puts(&prefix(graph_id)),
            BTreeSet::from([format!("{}head", prefix(graph_id))])
        );
        reads_back(&backend, graph_id, &source, &first).await;

        // One Surface changed: its record, the catalog and the head.
        let changed = retitled(&source, 3);
        let expected = expected_writes(graph_id, &source, &changed);
        assert_eq!(expected.len(), 3, "{expected:?}");
        let second = store.write_recorded_graph(&changed).await.unwrap();
        assert_ne!(second, first);
        assert_eq!(backend.last_puts(&prefix(graph_id)), expected);
        reads_back(&backend, graph_id, &changed, &second).await;
    });
}

/// The one graph a session published, by its keys.
async fn published_graph(backend: &Counting) -> Uuid {
    let ids: BTreeSet<_> = backend
        .list("graphs/")
        .await
        .unwrap()
        .iter()
        .map(|key| key.split('/').nth(1).unwrap().to_owned())
        .collect();
    assert_eq!(ids.len(), 1, "{ids:?}");
    Uuid::parse_str(ids.first().unwrap()).unwrap()
}

fn author() -> Author {
    Author::person("held-writes").via("graphshell")
}

#[test]
fn a_published_commit_leaves_out_what_the_head_holds() {
    pollster::block_on(async {
        let backend = Counting::default();
        let sessions = MereSessions::new(backend.clone());
        let mut session = sessions.begin_recorded(author(), None);
        session.flush(SystemTime::UNIX_EPOCH).await.unwrap();
        let adds = (1..=8)
            .map(|n: u128| {
                CapturedDelta::ReplayAddNodeWithIdIfMissing {
                    id: Uuid::from_u128(n).to_string(),
                    url: format!("https://held.test/{n}"),
                    position: [0.0, 0.0],
                }
                .replay_as(&author())
                .into_write()
                .unwrap()
            })
            .collect();
        let empty = session.graph().clone();
        let (_, first) = session
            .transact_deltas_published(author(), adds, SystemTime::UNIX_EPOCH)
            .await
            .unwrap();
        let graph_id = published_graph(&backend).await;
        assert_eq!(
            backend.last_puts(&prefix(graph_id)),
            first_collecting(graph_id, expected_writes(graph_id, &empty, session.graph()))
        );
        reads_back(&backend, graph_id, session.graph(), &first).await;

        // One Surface retitled in a published commit: its record, the catalog
        // and the head join the commit, nothing else of the publication.
        let before = session.graph().clone();
        let key = before.get_node_key_by_id(Uuid::from_u128(3)).unwrap();
        let (_, second) = session
            .transact_deltas_published(
                author(),
                vec![GraphDelta::SetNodeTitle {
                    key,
                    title: "changed".into(),
                }],
                SystemTime::UNIX_EPOCH,
            )
            .await
            .unwrap();
        let expected = expected_writes(graph_id, &before, session.graph());
        assert_eq!(expected.len(), 3, "{expected:?}");
        assert_eq!(backend.last_puts(&prefix(graph_id)), expected);
        reads_back(&backend, graph_id, session.graph(), &second).await;
        // A cold reopen of the session reads the same publication.
        let reopened = sessions.open(session.id()).await.unwrap();
        reads_back(&backend, graph_id, reopened.graph(), &second).await;
    });
}
