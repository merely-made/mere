// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Records list their edges without the snapshot index; the reader merges each
//! edge's two copies by pair and rebuilds `to_snapshot_at`'s columns; version 1
//! stores still load and the next publication rewrites them (engine split
//! D175).
use super::tests::linked;
use super::*;
use kernel::graph::SemanticStatementSpec;
use kernel::graph::apply::{self, GraphDelta, apply_graph_delta};
use muniment::MemoryBackend;
use std::collections::BTreeSet;

const GRAPH: Uuid = Uuid::from_u128(0xd175);

fn store(backend: &MemoryBackend) -> AddressableGraphStore<MemoryBackend> {
    AddressableGraphStore::new(backend.clone(), GRAPH)
}

/// Publish `graph` in the shape `version` writes, collecting as a publication
/// does.
async fn publish(backend: &MemoryBackend, graph: &Graph, version: u8) -> String {
    let store = store(backend);
    let prepared = store.prepare_with(graph, indexed_links, version).unwrap();
    let generation = prepared.generation.clone();
    store.commit_staged(store.stage(prepared)).await.unwrap();
    generation
}

/// The edge columns as bytes, the form the byte-for-byte checks compare.
fn columns(snapshot: &GraphSnapshot) -> Vec<u8> {
    encode(&(&snapshot.edges, &snapshot.resource_edges)).unwrap()
}

/// `read`'s edge columns are `want`'s, byte for byte; a failure names the
/// first byte that differs rather than printing both columns.
fn same_columns(read: &GraphSnapshot, want: &GraphSnapshot) {
    let (read, want) = (columns(read), columns(want));
    let at = read.iter().zip(&want).position(|(a, b)| a != b);
    assert!(
        read == want,
        "edge columns differ at byte {at:?} ({} read, {} wanted)",
        read.len(),
        want.len()
    );
}

/// `to_snapshot_at`'s columns, keeping only edges with both ends in `resident`.
fn expected(graph: &Graph, resident: &BTreeSet<GraphAddress>) -> GraphSnapshot {
    let mut snapshot = graph.to_snapshot_at(0);
    let kept = |edge: &PersistedEdge, stratum: fn(Uuid) -> GraphAddress| {
        [&edge.from_node_id, &edge.to_node_id]
            .iter()
            .all(|id| resident.contains(&stratum(Uuid::parse_str(id).unwrap())))
    };
    snapshot.edges.retain(|e| kept(e, GraphAddress::Surface));
    snapshot
        .resource_edges
        .retain(|e| kept(e, GraphAddress::Resource));
    snapshot
}

/// The snapshot a reader materializes from `view`'s records, before the
/// kernel reloads (and so reorders) it.
async fn materialized(view: &ResidentGraph<MemoryBackend>) -> GraphSnapshot {
    let catalog = view.catalog.as_ref().unwrap();
    let full = view.policy == ResidencyPolicy::Full;
    read::materialize(&view.store, catalog, &view.records, full)
        .await
        .unwrap()
        .0
}

/// [`linked`] with a Resource pair holding two edges, listed against the
/// order their content would sort in.
fn fixture() -> (Graph, (Uuid, Uuid)) {
    let mut graph = linked(12);
    let resource = |graph: &Graph, n| {
        let key = graph.get_node_key_by_id(Uuid::from_u128(n)).unwrap();
        graph.shown_resource_id(key).unwrap()
    };
    let pair = (resource(&graph, 4), resource(&graph, 5));
    graph
        .try_assert_semantic_statement_by_resource_ids(
            pair.0,
            pair.1,
            SemanticStatementSpec {
                predicate: "urn:test:resource-also".into(),
                ..Default::default()
            },
        )
        .unwrap();
    let between = |graph: &Graph| -> Vec<PersistedEdge> {
        let (from, to) = (pair.0.to_string(), pair.1.to_string());
        let snapshot = graph.to_snapshot_at(0);
        (snapshot.resource_edges.into_iter())
            .filter(|e| e.from_node_id == from && e.to_node_id == to)
            .collect()
    };
    let [joined] = between(&graph).try_into().unwrap();
    let mut split: Vec<PersistedEdge> = (joined.retained_assertions().iter())
        .map(|statement| {
            let mut edge = joined.clone();
            edge.assertions = Some(vec![statement.clone()]);
            if let Some(semantic) = &mut edge.semantic {
                semantic
                    .statements
                    .retain(|s| s.statement_id == statement.statement_id);
            }
            edge
        })
        .collect();
    assert_eq!(split.len(), 2);
    split.sort_by_key(|e| std::cmp::Reverse(encode(e).unwrap()));
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplaySetResourceEdgesByIds {
            from_resource_id: pair.0,
            to_resource_id: pair.1,
            edges: split.clone(),
        },
    );
    assert_eq!(between(&graph), split, "two edges in one pair, as set");
    (graph, pair)
}

#[test]
fn readers_rebuild_the_writers_columns_from_version_2_records() {
    pollster::block_on(async {
        let (graph, pair) = fixture();
        let source = graph.to_snapshot_at(0);
        // The instrument sees a chain, a fan and self relations among Surface
        // edges, and a Resource pair with two edges.
        assert!(source.edges.len() >= 23, "{}", source.edges.len());
        assert!(source.edges.iter().any(|e| e.from_node_id == e.to_node_id));
        assert!(
            source
                .resource_edges
                .iter()
                .any(|e| e.from_node_id == e.to_node_id)
        );
        let backend = MemoryBackend::new();
        publish(&backend, &graph, VERSION).await;
        let (_, catalog, _) = store(&backend).catalog().await.unwrap();
        assert_eq!(catalog.version, 2);
        // Version 2 records carry no index: none decodes in version 1's shape
        // unless it has no edges.
        for row in &catalog.entries {
            let bytes = (backend.get(&store(&backend).record_key(row.address, &row.hash)))
                .await
                .unwrap()
                .unwrap();
            let old = decode::<StoredNodeV1>(&bytes).map(StoredNode::from);
            assert!(old.is_err() || old.unwrap().edges().is_empty());
        }

        // A fresh full reader.
        let full = store(&backend)
            .open(ResidencyRoots::default())
            .await
            .unwrap();
        same_columns(&materialized(&full).await, &source);
        same_columns(&full.graph().to_snapshot_at(0), &source);

        // A fresh one-hop reader around the Surface showing the pair's start.
        store(&backend)
            .set_policy(ResidencyPolicy::Neighborhood { hops: 1 })
            .await
            .unwrap();
        let roots = ResidencyRoots {
            projection_surfaces: BTreeSet::from([Uuid::from_u128(4)]),
            ..Default::default()
        };
        let partial = store(&backend).open(roots).await.unwrap();
        let resident: BTreeSet<_> = partial.records.keys().copied().collect();
        let want = expected(&graph, &resident);
        assert!(resident.contains(&GraphAddress::Resource(pair.0)));
        assert!(resident.contains(&GraphAddress::Resource(pair.1)));
        assert!(
            !want.edges.is_empty() && want.edges.len() < source.edges.len(),
            "a partial view: {} of {}",
            want.edges.len(),
            source.edges.len()
        );
        assert!(want.resource_edges.len() < source.resource_edges.len());
        same_columns(&materialized(&partial).await, &want);
    });
}

/// The key the one sweep leaves.
fn swept() -> String {
    format!("graphs/{GRAPH}/resident/v1/swept")
}

#[test]
fn version_1_stores_read_the_same_and_the_next_publication_rewrites_them() {
    pollster::block_on(async {
        let (graph, _) = fixture();
        let source = graph.to_snapshot_at(0);
        let backend = MemoryBackend::new();
        publish(&backend, &graph, 1).await;
        let (_, old, _) = store(&backend).catalog().await.unwrap();
        assert_eq!(old.version, 1);

        // A fresh full reader opens version 1 records to the same graph.
        let view = store(&backend)
            .open(ResidencyRoots::default())
            .await
            .unwrap();
        same_columns(&materialized(&view).await, &source);
        let read = view.graph().to_snapshot_at(0);
        assert!(
            encode(&read).unwrap() == encode(&source).unwrap(),
            "the same graph"
        );
        assert_eq!(view.graph().facets(), graph.facets());

        // The next publication leaves only what its head names, every record
        // in version 2's shape.
        let mut next = graph.clone();
        let key = next.get_node_key_by_id(Uuid::from_u128(7)).unwrap();
        apply_graph_delta(
            &mut next,
            GraphDelta::SetNodeTitle {
                key,
                title: "next".into(),
            },
        );
        let prepared = store(&backend).prepare_recorded_graph(&next).unwrap();
        let mut want: BTreeSet<String> = (prepared.ops().iter())
            .map(|op| match op {
                muniment::WriteOp::Put { key, .. } => key.clone(),
                muniment::WriteOp::Delete { key } => panic!("prepared delete {key}"),
            })
            .collect();
        want.insert(swept());
        let generation = publish(&backend, &next, VERSION).await;
        let held: BTreeSet<String> = (backend.list(&store(&backend).key("")).await.unwrap())
            .into_iter()
            .collect();
        assert_eq!(held, want);
        let (head, new, _) = store(&backend).catalog().await.unwrap();
        assert_eq!((head.as_str(), new.version), (generation.as_str(), 2));
        let edged = old
            .entries
            .iter()
            .filter(|row| new.row(row.address).unwrap().hash != row.hash)
            .count();
        // Every record with an edge was rewritten (plus the retitled one).
        assert!(edged > source.nodes.len(), "{edged} records rewritten");
        let view = store(&backend)
            .open(ResidencyRoots::default())
            .await
            .unwrap();
        same_columns(&materialized(&view).await, &next.to_snapshot_at(0));
    });
}

/// `count` Surfaces each showing a Resource, and a statement from each
/// Resource to the next, the last closing the chain: the bench's chain graph.
fn chain(count: u128) -> Graph {
    let mut graph = Graph::new();
    let keys: Vec<_> = (1..=count)
        .map(|i| {
            apply::add_node(
                &mut graph,
                Some(Uuid::from_u128(i)),
                format!("https://chain.test/{i}"),
                Default::default(),
            )
        })
        .collect();
    let resources: Vec<_> = (keys.iter())
        .map(|key| graph.shown_resource_id(*key).unwrap())
        .collect();
    for (i, from) in resources.iter().enumerate() {
        let to = resources[(i + 1) % resources.len()];
        state(&mut graph, *from, to, "urn:test:next");
    }
    graph
}

fn state(graph: &mut Graph, from: Uuid, to: Uuid, predicate: &str) {
    graph
        .try_assert_semantic_statement_by_resource_ids(
            from,
            to,
            SemanticStatementSpec {
                predicate: predicate.into(),
                ..Default::default()
            },
        )
        .unwrap();
}

/// The record keys `after`'s publication writes over `before`'s: through the
/// publishing writer, or version 1's when `version` is 1.
fn rewritten(before: &Graph, after: &Graph, version: u8) -> BTreeSet<String> {
    let store = store(&MemoryBackend::new());
    let keys = |graph| -> BTreeSet<String> {
        let prepared = match version {
            1 => store.prepare_with(graph, indexed_links, 1),
            _ => store.prepare_recorded_graph(graph),
        }
        .unwrap();
        (prepared.ops.into_iter())
            .filter_map(|op| match op {
                muniment::WriteOp::Put { key, .. } if key.contains("/nodes/") => Some(key),
                _ => None,
            })
            .collect()
    };
    let old = keys(before);
    keys(after).difference(&old).cloned().collect()
}

#[test]
fn an_insert_between_existing_nodes_writes_only_its_two_records() {
    let before = chain(40);
    let snapshot = before.to_snapshot_at(0);
    // A new pair from the Resource that sorts first, so version 1's index
    // shifts for nearly every edge after it.
    let mut ids: Vec<Uuid> = (snapshot.resources.iter())
        .map(|r| kernel::graph::ResourceNode::for_term(&r.canonical_iri).id())
        .collect();
    ids.sort();
    let pairs: BTreeSet<(String, String)> = (snapshot.resource_edges.iter())
        .map(|e| (e.from_node_id.clone(), e.to_node_id.clone()))
        .collect();
    let to = (ids[1..].iter())
        .find(|to| !pairs.contains(&(ids[0].to_string(), to.to_string())))
        .copied()
        .unwrap();
    let mut after = before.clone();
    state(&mut after, ids[0], to, "urn:test:inserted");
    let plain = rewritten(&before, &after, VERSION);
    let indexed = rewritten(&before, &after, 1);
    // The instrument: version 1's writer rewrites far more than the pair.
    assert!(indexed.len() >= 20, "version 1 rewrote {}", indexed.len());
    let endpoints = BTreeSet::from([ids[0], to]);
    let changed: BTreeSet<Uuid> = (plain.iter())
        .map(|key| Uuid::parse_str(key.rsplit('/').nth(1).unwrap()).unwrap())
        .collect();
    assert_eq!(
        (plain.len(), changed),
        (2, endpoints),
        "version 2 rewrote {plain:?}; version 1 rewrote {} of {}",
        indexed.len(),
        snapshot.nodes.len() + snapshot.resources.len()
    );
    println!(
        "insert at 40: version 2 rewrote {}, version 1 rewrote {} of {} records",
        plain.len(),
        indexed.len(),
        snapshot.nodes.len() + snapshot.resources.len()
    );
}

/// What a full read of `records` under `view`'s catalog answers.
async fn refused(
    view: &ResidentGraph<MemoryBackend>,
    records: &BTreeMap<GraphAddress, StoredNode>,
) -> String {
    let catalog = view.catalog.as_ref().unwrap();
    match read::materialize(&view.store, catalog, records, true).await {
        Err(error) => error.to_string(),
        Ok(_) => "admitted".into(),
    }
}

#[test]
fn copies_of_an_edge_that_disagree_are_refused() {
    pollster::block_on(async {
        let (graph, pair) = fixture();
        let backend = MemoryBackend::new();
        publish(&backend, &graph, VERSION).await;
        let view = store(&backend)
            .open(ResidencyRoots::default())
            .await
            .unwrap();
        // One endpoint lists the pair's two edges in the other order.
        let mut swapped = view.records.clone();
        let Some(StoredNode::Resource { edges, .. }) =
            swapped.get_mut(&GraphAddress::Resource(pair.0))
        else {
            panic!("the pair's start is a resident Resource");
        };
        let at: Vec<usize> = (0..edges.len())
            .filter(|i| edges[*i].to_node_id == pair.1.to_string())
            .collect();
        assert_eq!(at.len(), 2);
        edges.swap(at[0], at[1]);
        let answer = refused(&view, &swapped).await;
        assert!(
            answer.contains("inconsistent incident relation copies"),
            "{answer}"
        );
        // One endpoint's copy of a Surface edge says something else.
        let mut changed = view.records.clone();
        let Some(StoredNode::Surface { edges, .. }) =
            changed.get_mut(&GraphAddress::Surface(Uuid::from_u128(3)))
        else {
            panic!("Surface 3 is resident");
        };
        let edge = (edges.iter_mut())
            .find(|e| e.from_node_id != e.to_node_id)
            .unwrap();
        edge.assertions = Some(vec![]);
        let answer = refused(&view, &changed).await;
        assert!(
            answer.contains("inconsistent incident relation copies"),
            "{answer}"
        );
        // The untouched records are admitted.
        assert_eq!(refused(&view, &view.records).await, "admitted");
    });
}

#[test]
fn an_edge_one_resident_endpoint_omits_is_refused() {
    pollster::block_on(async {
        let (graph, pair) = fixture();
        let backend = MemoryBackend::new();
        publish(&backend, &graph, VERSION).await;
        let view = store(&backend)
            .open(ResidencyRoots::default())
            .await
            .unwrap();
        // Surface 3's record omits one of its edges to another Surface.
        let mut omitted = view.records.clone();
        let Some(StoredNode::Surface { edges, .. }) =
            omitted.get_mut(&GraphAddress::Surface(Uuid::from_u128(3)))
        else {
            panic!("Surface 3 is resident");
        };
        let at = (edges.iter())
            .position(|e| e.from_node_id != e.to_node_id)
            .unwrap();
        edges.remove(at);
        let answer = refused(&view, &omitted).await;
        assert!(answer.contains("listed by one endpoint"), "{answer}");
        // The pair's start omits both of the pair's edges.
        let mut omitted = view.records.clone();
        let Some(StoredNode::Resource { edges, .. }) =
            omitted.get_mut(&GraphAddress::Resource(pair.0))
        else {
            panic!("the pair's start is a resident Resource");
        };
        edges.retain(|e| e.to_node_id != pair.1.to_string());
        let answer = refused(&view, &omitted).await;
        assert!(answer.contains("listed by one endpoint"), "{answer}");
        assert_eq!(refused(&view, &view.records).await, "admitted");

        // A one-hop view: resident records list edges to items that are not
        // resident, which only one endpoint can list. Accepted.
        store(&backend)
            .set_policy(ResidencyPolicy::Neighborhood { hops: 1 })
            .await
            .unwrap();
        let roots = ResidencyRoots {
            projection_surfaces: BTreeSet::from([Uuid::from_u128(4)]),
            ..Default::default()
        };
        let partial = store(&backend).open(roots).await.unwrap();
        let resident: BTreeSet<_> = partial.records.keys().copied().collect();
        let outward = (partial.records.iter())
            .flat_map(|(address, record)| record.edges().iter().map(move |e| (address, e)))
            .filter(|(address, e)| {
                let stratum = |id: &String| {
                    let id = Uuid::parse_str(id).unwrap();
                    match address {
                        GraphAddress::Surface(_) => GraphAddress::Surface(id),
                        GraphAddress::Resource(_) => GraphAddress::Resource(id),
                    }
                };
                !resident.contains(&stratum(&e.from_node_id))
                    || !resident.contains(&stratum(&e.to_node_id))
            })
            .count();
        assert!(outward > 0, "the view has edges leaving it");
        let catalog = partial.catalog.as_ref().unwrap();
        let read = read::materialize(&partial.store, catalog, &partial.records, false)
            .await
            .unwrap()
            .0;
        same_columns(&read, &expected(&graph, &resident));
    });
}
