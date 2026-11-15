// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Catalog version 3 (engine split D177): readers match the writer through
//! trees of several levels, the store holds exactly what the head names, a
//! damaged index chunk fails the reader, collection's decision reads only
//! the chunks the new catalog lacks, version 2 stores are rewritten, and the
//! generation does not depend on edit history.
use super::super::collect::named_keys;
use super::super::tree::{CHUNK_TARGET, TARGET};
use super::tests::linked;
use super::*;
use kernel::graph::apply::{GraphDelta, apply_graph_delta};
use kernel::graph::fixtures::GraphFixtures;
use kernel::graph::{PendingLink, PendingLinkRetention, SemanticStatementSpec};
use muniment::{MemoryBackend, TransactionReader};
use std::cell::RefCell;
use std::collections::{BTreeSet, HashSet};

const GRAPH: Uuid = Uuid::from_u128(0xd177);

fn store(backend: &MemoryBackend) -> AddressableGraphStore<MemoryBackend> {
    AddressableGraphStore::new(backend.clone(), GRAPH)
}

/// Write with chunk target `target` on this thread.
fn with_target(target: u64) {
    TARGET.with(|t| t.set(target));
}

/// Publish `graph` as `version` writes it, collecting as a publication does.
async fn publish(backend: &MemoryBackend, graph: &Graph, version: u8) -> String {
    let store = store(backend);
    let prepared = store.prepare_with(graph, indexed_links, version).unwrap();
    let generation = prepared.generation().to_owned();
    store.commit_staged(store.stage(prepared)).await.unwrap();
    generation
}

/// The keys under the graph's prefix besides `head`, `policy` and `swept`.
async fn stored(backend: &MemoryBackend) -> BTreeSet<String> {
    let store = store(backend);
    let skip = ["head", "policy", "swept"].map(|k| store.key(k));
    let keys = backend.list(&store.key("")).await.unwrap();
    keys.into_iter().filter(|k| !skip.contains(k)).collect()
}

/// What the head names.
async fn named(backend: &MemoryBackend) -> BTreeSet<String> {
    let store = store(backend);
    let (generation, catalog, _) = store.catalog().await.unwrap();
    named_keys(&store.key(""), &generation, &catalog)
        .into_iter()
        .collect()
}

/// A fresh full reader gives `graph`'s snapshot and facets, byte for byte.
async fn reads_back(backend: &MemoryBackend, graph: &Graph) {
    let view = store(backend)
        .open(ResidencyRoots::default())
        .await
        .unwrap();
    let got = encode(&(view.graph().to_snapshot_at(0), view.graph().facets())).unwrap();
    let want = encode(&(graph.to_snapshot_at(0), graph.facets())).unwrap();
    assert!(got == want, "the reader differs from the writer");
    assert_eq!(
        view.graph().pending_link_state(),
        &graph.retained_pending_link_state()
    );
}

fn depths(catalog: &Catalog) -> [u8; 5] {
    let t = &catalog.trees;
    [&t.rows, &t.entries, &t.visits, &t.owners, &t.pending].map(|tree| tree.root().depth)
}

/// The edits the publications make, in turn: a title, a navigation, a
/// statement, a new node, a pending observation, the statement withdrawn.
fn edit(graph: &mut Graph, step: usize) {
    let id = |i: u128| Uuid::from_u128(i);
    match step % 6 {
        0 => {
            let key = graph.get_node_key_by_id(id(3)).unwrap();
            let title = format!("retitled {step}");
            apply_graph_delta(graph, GraphDelta::SetNodeTitle { key, title });
        },
        1 => {
            apply_graph_delta(
                graph,
                GraphDelta::ReplayNavigateNodeById {
                    node_id: id(2 + (step / 6) as u128 % 2),
                    url: format!("https://tree.test/{step}"),
                    transition: kernel::graph::NodeHistoryTransitionKind::UrlTyped,
                    timestamp_ms: 10 + step as u64,
                    last_session_visited: 0,
                },
            );
        },
        2 => {
            let from = graph.get_node_key_by_id(id(1)).unwrap();
            let to = graph.get_node_key_by_id(id(3)).unwrap();
            graph.assert_semantic_predicate(from, to, "urn:test:next".into());
        },
        3 => {
            kernel::graph::apply::add_node(
                graph,
                Some(id(0x1000 + step as u128)),
                format!("https://tree.test/new/{step}"),
                Default::default(),
            );
        },
        4 => {
            let node = id(1 + (step / 6) as u128 % 2);
            let source = graph.shown_resource_id(graph.get_node_key_by_id(node).unwrap());
            graph.queue_pending_link(PendingLink {
                source_resource: source.unwrap(),
                source_surface: Some(node),
                target_iri: format!("https://pending.test/{step}"),
                statement: SemanticStatementSpec {
                    predicate: "urn:test:pending".into(),
                    ..Default::default()
                },
            });
        },
        _ => {
            let key = graph.get_node_key_by_id(id(1)).unwrap();
            apply_graph_delta(
                graph,
                GraphDelta::SetNodeTitle {
                    key,
                    title: format!("again {step}"),
                },
            );
        },
    }
}

#[test]
fn trees_of_several_levels_read_back_and_hold_exactly_the_heads_keys() {
    pollster::block_on(async {
        for (count, target) in [(3, 2), (9, 4), (40, 2), (60, 4), (150, 128)] {
            with_target(target);
            let mut graph = linked(count);
            graph.set_pending_link_retention(PendingLinkRetention::UntilPurged);
            let backend = MemoryBackend::new();
            let mut deepest = 0;
            for step in 0..13 {
                if step > 0 {
                    edit(&mut graph, step);
                }
                publish(&backend, &graph, VERSION).await;
                reads_back(&backend, &graph).await;
                assert_eq!(
                    stored(&backend).await,
                    named(&backend).await,
                    "{count}/{target} step {step}"
                );
                let (_, catalog, _) = store(&backend).catalog().await.unwrap();
                assert_eq!(catalog.version, VERSION);
                deepest = deepest.max(depths(&catalog)[0]);
            }
            let (_, catalog, _) = store(&backend).catalog().await.unwrap();
            let t = &catalog.trees;
            let lists = [&t.entries, &t.visits, &t.owners, &t.pending];
            assert!(lists.iter().all(|tree| tree.root().top.is_some()));
            if count >= 40 && target == 2 {
                assert!(deepest >= 3, "rows depth {deepest}");
                let all = depths(&catalog);
                assert!(all[1] >= 1, "navigation entries depth {all:?}");
            }
        }
        with_target(CHUNK_TARGET);
    });
}

#[test]
fn a_changed_or_missing_index_chunk_fails_the_reader() {
    pollster::block_on(async {
        with_target(2);
        let graph = linked(40);
        let backend = MemoryBackend::new();
        publish(&backend, &graph, VERSION).await;
        let (_, catalog, _) = store(&backend).catalog().await.unwrap();
        let rows = &catalog.trees.rows;
        assert!(rows.root().depth >= 2);
        // The top and an index chunk one level above the leaves.
        let victims = [rows.root().top.unwrap(), rows.level(1)[0].to_owned()];
        for digest in victims {
            let key = store(&backend).chunk_key(&digest);
            let bytes = backend.get(&key).await.unwrap().unwrap();
            let mut changed = bytes.clone();
            let last = changed.len() - 2;
            changed[last] ^= 1;
            backend.put(&key, &changed).await.unwrap();
            let error = store(&backend).open(ResidencyRoots::default()).await.err();
            assert!(error.unwrap().to_string().contains("checksum mismatch"));
            backend.delete(&key).await.unwrap();
            let error = store(&backend).open(ResidencyRoots::default()).await.err();
            assert!(
                error
                    .unwrap()
                    .to_string()
                    .contains("missing addressable record")
            );
            backend.put(&key, &bytes).await.unwrap();
            reads_back(&backend, &graph).await;
        }
        with_target(CHUNK_TARGET);
    });
}

/// A transaction's snapshot of a memory store, recording the keys read.
struct Recording {
    keys: BTreeMap<String, Vec<u8>>,
    read: RefCell<Vec<String>>,
}
impl Recording {
    async fn of(backend: &MemoryBackend) -> Self {
        let mut keys = BTreeMap::new();
        for key in backend.list("").await.unwrap() {
            let bytes = backend.get(&key).await.unwrap().unwrap();
            keys.insert(key, bytes);
        }
        Self {
            keys,
            read: RefCell::new(vec![]),
        }
    }
}
impl TransactionReader for Recording {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        self.read.borrow_mut().push(key.to_owned());
        Ok(self.keys.get(key).cloned())
    }
    fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        Ok(self
            .keys
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect())
    }
}

#[test]
fn the_decision_reads_only_the_chunks_the_new_catalog_lacks() {
    pollster::block_on(async {
        with_target(2);
        let mut graph = linked(60);
        let backend = MemoryBackend::new();
        publish(&backend, &graph, VERSION).await;
        for step in [0, 2, 3] {
            edit(&mut graph, step);
            let store = store(&backend);
            let (_, old, _) = store.catalog().await.unwrap();
            let prepared = store.prepare_recorded_graph(&graph).unwrap();
            let new: HashSet<&str> = prepared.catalog.trees.digests().collect();
            let lacking: BTreeSet<String> = (old.trees.digests())
                .filter(|d| !new.contains(d))
                .map(|d| store.chunk_key(d))
                .collect();
            assert!(!lacking.is_empty() && new.len() > 2 * lacking.len());
            let held = named(&backend).await;
            let want = named_keys(&store.key(""), prepared.generation(), &prepared.catalog);
            let reader = Recording::of(&backend).await;
            let ops = store.stage(prepared).decide(&reader);
            let read: BTreeSet<String> = (reader.read.borrow().iter())
                .filter(|k| k.contains("/chunks/"))
                .cloned()
                .collect();
            assert_eq!(read, lacking, "step {step}");
            // Exactly the puts the head lacks and the deletes it supersedes.
            for op in &ops {
                match op {
                    WriteOp::Put { key, .. } if key.ends_with("/swept") => {},
                    WriteOp::Put { key, .. } => assert!(!held.contains(key), "{key}"),
                    WriteOp::Delete { key } => assert!(held.contains(key) && !want.contains(key)),
                }
            }
            let deleted = ops
                .iter()
                .filter(|op| matches!(op, WriteOp::Delete { .. }))
                .count();
            assert_eq!(deleted, held.iter().filter(|k| !want.contains(*k)).count());
            backend.apply(&ops).await.unwrap();
            assert_eq!(stored(&backend).await, named(&backend).await);
            reads_back(&backend, &graph).await;
        }
        with_target(CHUNK_TARGET);
    });
}

#[test]
fn a_version_2_store_opens_and_is_rewritten_as_version_3() {
    pollster::block_on(async {
        with_target(4);
        let mut graph = linked(30);
        let backend = MemoryBackend::new();
        let old = publish(&backend, &graph, 2).await;
        let (_, catalog, _) = store(&backend).catalog().await.unwrap();
        assert_eq!(catalog.version, 2);
        reads_back(&backend, &graph).await;
        edit(&mut graph, 0);
        let new = publish(&backend, &graph, VERSION).await;
        let (_, catalog, _) = store(&backend).catalog().await.unwrap();
        assert_eq!(catalog.version, 3);
        assert!(catalog.trees.rows.root().depth >= 1);
        let keys = stored(&backend).await;
        assert!(!keys.contains(&store(&backend).key(&format!("catalog/{old}"))));
        assert!(keys.contains(&store(&backend).key(&format!("catalog/{new}"))));
        assert_eq!(keys, named(&backend).await);
        reads_back(&backend, &graph).await;
        with_target(CHUNK_TARGET);
    });
}

#[test]
fn the_generation_does_not_depend_on_edit_history() {
    pollster::block_on(async {
        with_target(2);
        let mut base = linked(40);
        base.set_pending_link_retention(PendingLinkRetention::UntilPurged);
        // The same edits in two orders, the first published after each. (A
        // statement's id and a new node's visit record when they were made, so
        // neither is made here.)
        let (mut first, mut second) = (base.clone(), base.clone());
        let one = MemoryBackend::new();
        publish(&one, &first, VERSION).await;
        for step in [0, 1, 4, 5] {
            edit(&mut first, step);
            publish(&one, &first, VERSION).await;
        }
        for step in [5, 4, 1, 0] {
            edit(&mut second, step);
        }
        let same = |g: &Graph| encode(&(g.to_snapshot_at(0), g.facets())).unwrap();
        assert!(
            same(&first) == same(&second),
            "the two orders give one graph"
        );
        let two = MemoryBackend::new();
        let generation = publish(&two, &second, VERSION).await;
        assert_eq!(store(&one).catalog().await.unwrap().0, generation);
        assert_eq!(stored(&one).await, stored(&two).await);
        // Control: a different graph is another generation.
        edit(&mut second, 2);
        assert_ne!(publish(&two, &second, VERSION).await, generation);
        with_target(CHUNK_TARGET);
    });
}
