// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A publication prepared through the endpoint maps is byte-identical to one
//! prepared through the scans they replaced (engine split D171).
use super::*;
use kernel::graph::apply::{self, GraphDelta, apply_graph_delta};
use kernel::graph::fixtures::GraphFixtures;
use kernel::graph::{Graph, SemanticStatementSpec};
use muniment::MemoryBackend;

/// Surfaces in a chain with edges back to the first and a self relation,
/// Resources in a chain with a self relation, every Surface showing one.
pub(super) fn linked(count: u128) -> Graph {
    let mut graph = Graph::new();
    let keys: Vec<_> = (1..=count)
        .map(|i| {
            apply::add_node(
                &mut graph,
                Some(Uuid::from_u128(i)),
                format!("https://links.test/{i}"),
                Default::default(),
            )
        })
        .collect();
    let author = kernel::graph::Author::person("links-author");
    for predicate in ["urn:test:next", "urn:test:first", "urn:test:itself"] {
        graph
            .write_as(author.clone(), |g| {
                g.declare_predicate(predicate, kernel::graph::GraphStratum::Surface)
            })
            .unwrap();
    }
    let mut relate = |from: usize, to: usize, predicate: &str| {
        assert!(
            graph
                .assert_semantic_predicate(keys[from], keys[to], predicate.into())
                .is_some()
        );
    };
    for i in 1..keys.len() {
        relate(i - 1, i, "urn:test:next");
        relate(i, 0, "urn:test:first");
    }
    relate(1, 1, "urn:test:itself");
    let resources: Vec<_> = keys
        .iter()
        .map(|key| graph.shown_resource_id(*key).unwrap())
        .collect();
    let pairs = resources.windows(2).map(|pair| (pair[0], pair[1]));
    for (from, to) in pairs.chain([(resources[2], resources[2])]) {
        graph
            .try_assert_semantic_statement_by_resource_ids(
                from,
                to,
                SemanticStatementSpec {
                    predicate: "urn:test:resource-next".into(),
                    ..Default::default()
                },
            )
            .unwrap();
    }
    graph
}

#[test]
fn indexed_links_publish_the_bytes_the_scans_did() {
    let graph = linked(12);
    let snapshot = graph.to_snapshot_at(0);
    // The instrument sees every kind of link the maps answer for.
    let counts = (
        snapshot.edges.len(),
        snapshot.resource_edges.len(),
        snapshot.shown_resources.len(),
    );
    assert!(
        counts.0 >= 23 && counts.1 >= 12 && counts.2 == 12,
        "{counts:?}"
    );
    let store = AddressableGraphStore::new(MemoryBackend::new(), Uuid::from_u128(0xd171));
    let scanned = store.prepare_with(&graph, scanned_links, VERSION).unwrap();
    let indexed = store.prepare_recorded_graph(&graph).unwrap();
    assert_eq!(indexed.generation, scanned.generation);
    assert!(indexed.ops == scanned.ops, "byte-identical writes");
    for node in &snapshot.nodes {
        let id = Uuid::parse_str(&node.node_id).unwrap();
        assert!(indexed.shown_resource(id).is_some());
    }

    // Control: a changed graph publishes another generation.
    let mut changed = graph.clone();
    let key = changed.get_node_key_by_id(Uuid::from_u128(5)).unwrap();
    apply_graph_delta(
        &mut changed,
        GraphDelta::SetNodeTitle {
            key,
            title: "moved".into(),
        },
    );
    let moved = store.prepare_recorded_graph(&changed).unwrap();
    assert_ne!(moved.generation, indexed.generation);
    assert_eq!(
        moved.generation,
        store
            .prepare_with(&changed, scanned_links, VERSION)
            .unwrap()
            .generation
    );
}

/// The catalog's row index answers as the scans it replaced (engine split
/// D177), refuses a duplicate address as the scan's check did, and a reader's
/// footprint counts the catalog's stored bytes.
#[test]
fn row_index_answers_as_the_scans_did() {
    pollster::block_on(async {
        let graph = linked(12);
        let backend = MemoryBackend::new();
        let id = Uuid::from_u128(0xd177);
        let store = AddressableGraphStore::new(backend.clone(), id);
        let generation = store.write_recorded_graph(&graph).await.unwrap();
        let (_, catalog, size) = store.catalog().await.unwrap();
        let prepared = store.prepare_recorded_graph(&graph).unwrap();
        let mut showing = 0;
        for row in &catalog.entries {
            let scanned = catalog.entries.iter().find(|e| e.address == row.address);
            assert_eq!(
                catalog.row(row.address).unwrap().hash,
                scanned.unwrap().hash
            );
            assert_eq!(prepared.record_digest(row.address), Some(row.hash.as_str()));
            let other = match row.address {
                GraphAddress::Surface(id) => GraphAddress::Resource(id),
                GraphAddress::Resource(id) => GraphAddress::Surface(id),
            };
            assert!(catalog.row(other).is_none());
            let mine: Vec<_> = catalog
                .showing(row.address.id())
                .map(|r| r.address)
                .collect();
            let scanned: Vec<_> = (catalog.entries.iter())
                .filter(|e| e.shown == Some(row.address.id()))
                .map(|e| e.address)
                .collect();
            assert_eq!(mine, scanned);
            showing += mine.len();
        }
        assert_eq!(showing, 12);

        let key = store.key(&format!("catalog/{generation}"));
        assert_eq!(backend.get(&key).await.unwrap().unwrap().len(), size);
        assert_eq!(encode(&catalog).unwrap().len(), size);
        let view = AddressableGraphStore::new(backend.clone(), id)
            .open(ResidencyRoots::default())
            .await
            .unwrap();
        assert_eq!(view.footprint().catalog_bytes, size);

        // Control: a repeated address is refused.
        let mut twice = catalog.clone();
        twice.entries.push(catalog.entries[0].clone());
        let bytes = encode(&twice).unwrap();
        let refused = parse_catalog(&bytes, &hash(&bytes)).err().unwrap();
        assert!(refused.to_string().contains("duplicate catalog address"));
    });
}
