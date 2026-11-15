// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use kernel::graph::{CoverageLayer, ResourceNode};
use kernel::persistence::{GraphSnapshot, PersistedEdge, PersistedShownResource};
use muniment::WriteOp;
use std::collections::{HashMap, HashSet};

/// Each node's edges and shown binding, found through maps built once per
/// prepare instead of a scan of every edge and binding per node (engine split
/// D171: the Resource edge scan was 97% of a 100k publication).
struct NodeLinks<'a> {
    snapshot: &'a GraphSnapshot,
    lookup: Lookup<'a>,
}
enum Lookup<'a> {
    Maps {
        edges: HashMap<&'a str, Vec<usize>>,
        resource_edges: HashMap<&'a str, Vec<usize>>,
        shown: HashMap<&'a str, &'a PersistedShownResource>,
    },
    /// The scans the maps replaced, kept so a test can pin that both write
    /// the same bytes.
    #[cfg(test)]
    Scan,
}
/// The links of `snapshot`, its maps built once.
fn indexed_links(snapshot: &GraphSnapshot) -> NodeLinks<'_> {
    let mut shown = HashMap::new();
    for binding in &snapshot.shown_resources {
        // The first binding wins, as the scan's `find` did.
        shown.entry(binding.surface_id.as_str()).or_insert(binding);
    }
    let lookup = Lookup::Maps {
        edges: by_endpoint(&snapshot.edges),
        resource_edges: by_endpoint(&snapshot.resource_edges),
        shown,
    };
    NodeLinks { snapshot, lookup }
}
#[cfg(test)]
fn scanned_links(snapshot: &GraphSnapshot) -> NodeLinks<'_> {
    NodeLinks {
        snapshot,
        lookup: Lookup::Scan,
    }
}
impl<'a> NodeLinks<'a> {
    fn surface_edges(&self, id: &str) -> Vec<(usize, PersistedEdge)> {
        match &self.lookup {
            Lookup::Maps { edges, .. } => listed(&self.snapshot.edges, edges, id),
            #[cfg(test)]
            Lookup::Scan => scanned(&self.snapshot.edges, id),
        }
    }
    fn resource_edges(&self, id: &str) -> Vec<(usize, PersistedEdge)> {
        match &self.lookup {
            Lookup::Maps { resource_edges, .. } => {
                listed(&self.snapshot.resource_edges, resource_edges, id)
            },
            #[cfg(test)]
            Lookup::Scan => scanned(&self.snapshot.resource_edges, id),
        }
    }
    fn shown(&self, id: &str) -> Option<&'a PersistedShownResource> {
        match &self.lookup {
            Lookup::Maps { shown, .. } => shown.get(id).copied(),
            #[cfg(test)]
            Lookup::Scan => self
                .snapshot
                .shown_resources
                .iter()
                .find(|b| b.surface_id == id),
        }
    }
}
/// Edge positions by endpoint, each list in edge order; a self-loop once.
fn by_endpoint(edges: &[PersistedEdge]) -> HashMap<&str, Vec<usize>> {
    let mut map: HashMap<&str, Vec<usize>> = HashMap::new();
    for (i, edge) in edges.iter().enumerate() {
        map.entry(edge.from_node_id.as_str()).or_default().push(i);
        if edge.to_node_id != edge.from_node_id {
            map.entry(edge.to_node_id.as_str()).or_default().push(i);
        }
    }
    map
}
fn listed(
    edges: &[PersistedEdge],
    index: &HashMap<&str, Vec<usize>>,
    id: &str,
) -> Vec<(usize, PersistedEdge)> {
    index
        .get(id)
        .map(|at| at.iter().map(|&i| (i, edges[i].clone())).collect())
        .unwrap_or_default()
}
#[cfg(test)]
fn scanned(edges: &[PersistedEdge], id: &str) -> Vec<(usize, PersistedEdge)> {
    edges
        .iter()
        .enumerate()
        .filter(|(_, e)| e.from_node_id == id || e.to_node_id == id)
        .map(|(i, e)| (i, e.clone()))
        .collect()
}
/// Refuse a residency-limited view, which cannot become complete truth.
pub(crate) fn refuse_residency_limited(graph: &Graph) -> Result<(), StoreError> {
    if graph
        .known_coverage()
        .limits
        .iter()
        .any(|l| l.layer == CoverageLayer::Residency)
    {
        return Err(fail(
            "cannot publish a residency-limited view as complete truth",
        ));
    }
    Ok(())
}
#[cfg(test)]
thread_local! {
    /// Publications prepared on this thread, for tests that count them.
    pub(crate) static PREPARES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
/// A complete publication prepared without touching storage. The caller owns
/// when, and in which batch, its writes are applied.
pub struct PreparedGraphPublication {
    ops: Vec<WriteOp>,
    generation: String,
    catalog: Catalog,
}
impl PreparedGraphPublication {
    pub fn ops(&self) -> &[WriteOp] {
        &self.ops
    }
    pub fn generation(&self) -> &str {
        &self.generation
    }
    /// The content hash of the complete stored record at `address`.
    pub fn record_digest(&self, address: GraphAddress) -> Option<&str> {
        self.catalog.row(address).map(|row| row.hash.as_str())
    }
    /// The Resource a Surface shows, from the complete catalog.
    pub fn shown_resource(&self, surface: Uuid) -> Option<Uuid> {
        self.catalog.row(GraphAddress::Surface(surface))?.shown
    }
}
impl<B: Backend> AddressableGraphStore<B> {
    /// Publish immutable records and catalog, then their head pointer, leaving
    /// out what the current head already holds. Existing resident readers
    /// retain their prior generation until an explicit refresh.
    pub async fn write_recorded_graph(&self, graph: &Graph) -> Result<String, StoreError> {
        let prepared = self.prepare_recorded_graph(graph)?;
        let (ops, generation) = self.unheld_writes(prepared).await;
        self.backend.apply(&ops).await?;
        Ok(generation)
    }
    /// The writes of `prepared` less every put whose key the current head's
    /// catalog already names, and its generation. Those keys are content
    /// addressed, so the head naming one means the store holds its bytes: the
    /// rule in the [module docs](super) that nothing deletes or rewrites a
    /// record the head names is what makes leaving it out safe. With no head,
    /// or one that cannot be read, nothing is held and every write stays.
    pub(crate) async fn unheld_writes(
        &self,
        prepared: PreparedGraphPublication,
    ) -> (Vec<WriteOp>, String) {
        let held = self.held_keys().await;
        let mut ops = prepared.ops;
        ops.retain(|op| !matches!(op, WriteOp::Put { key, .. } if held.contains(key)));
        (ops, prepared.generation)
    }
    /// The content-addressed keys the current head's catalog names: its
    /// records, navigation rows, pending owners and the catalog itself.
    async fn held_keys(&self) -> HashSet<String> {
        let mut held = HashSet::new();
        let Ok((generation, catalog, _)) = self.catalog().await else {
            return held;
        };
        held.insert(self.key(&format!("catalog/{generation}")));
        for row in &catalog.entries {
            held.insert(self.record_key(row.address, &row.hash));
        }
        for (kind, digests) in [
            ("entry", &catalog.navigation.entries),
            ("visit", &catalog.navigation.visits),
            ("owner", &catalog.navigation.owners),
        ] {
            for digest in digests {
                held.insert(self.key(&format!("navigation/{kind}/{digest}")));
            }
        }
        for (id, digest) in &catalog.pending.owners {
            held.insert(self.key(&format!("pending/{id}/{digest}")));
        }
        held
    }
    /// Encode a complete publication without writing. Refuses a residency-limited
    /// view, which cannot become complete truth.
    pub fn prepare_recorded_graph(
        &self,
        graph: &Graph,
    ) -> Result<PreparedGraphPublication, StoreError> {
        self.prepare_with(graph, indexed_links)
    }
    fn prepare_with(
        &self,
        graph: &Graph,
        links: for<'a> fn(&'a GraphSnapshot) -> NodeLinks<'a>,
    ) -> Result<PreparedGraphPublication, StoreError> {
        refuse_residency_limited(graph)?;
        #[cfg(test)]
        PREPARES.with(|count| count.set(count.get() + 1));
        let snapshot = graph.to_snapshot_at(0);
        let mut loaded =
            Graph::try_from_recorded_snapshot(&snapshot).map_err(|e| fail(e.to_string()))?;
        loaded
            .install_facets(graph.facets().clone())
            .map_err(|e| fail(e.to_string()))?;
        let mut catalog = Catalog {
            version: 1,
            globals: snapshot.clone(),
            entries: vec![],
            navigation: NavigationIndex {
                entries: vec![],
                visits: vec![],
                owners: vec![],
                identities: vec![],
                digest: hash(&encode(snapshot.navigation.snapshot())?),
            },
            orphan_facets: graph.facets().clone(),
            pending: PendingIndex {
                retention: graph.pending_link_state().retention,
                owners: vec![],
            },
        };
        catalog.globals.nodes.clear();
        catalog.globals.edges.clear();
        catalog.globals.resources.clear();
        catalog.globals.resource_edges.clear();
        catalog.globals.shown_resources.clear();
        catalog.globals.navigation = Default::default();
        let links = links(&snapshot);
        let mut ops = vec![];
        for node in &snapshot.nodes {
            let id = Uuid::parse_str(&node.node_id).map_err(|e| fail(e.to_string()))?;
            catalog.orphan_facets.remove_node(&id);
            let facets = graph
                .facets()
                .facets_of(&id)
                .map(|f| {
                    f.iter()
                        .map(|(k, v)| (k.as_str().to_owned(), v.clone()))
                        .collect()
                })
                .unwrap_or_default();
            let edges = links.surface_edges(&node.node_id);
            let row = IndexRow {
                address: GraphAddress::Surface(id),
                hash: String::new(),
                shown: links
                    .shown(&node.node_id)
                    .map(|b| Uuid::parse_str(&b.resource_id))
                    .transpose()
                    .map_err(|e| fail(e.to_string()))?,
                pinned: graph
                    .get_node_key_by_id(id)
                    .and_then(|k| graph.node_is_pinned(k))
                    .unwrap_or(false),
                declaration: false,
            };
            self.append_node(
                &mut catalog,
                &mut ops,
                row,
                StoredNode::Surface {
                    node: node.clone(),
                    facets,
                    edges,
                },
            )?;
        }
        for record in &snapshot.resources {
            let id = ResourceNode::for_term(&record.canonical_iri).id();
            let edges = links.resource_edges(&id.to_string());
            let declaration = record.facets.iter().any(|f| {
                f.facet == kernel::graph::predicate_declarations::PREDICATE_DECLARATIONS_FACET
            });
            self.append_node(
                &mut catalog,
                &mut ops,
                IndexRow {
                    address: GraphAddress::Resource(id),
                    hash: String::new(),
                    shown: None,
                    pinned: false,
                    declaration,
                },
                StoredNode::Resource {
                    record: record.clone(),
                    edges,
                },
            )?;
        }
        pending::write(
            self,
            &graph.retained_pending_link_state(),
            &mut catalog.pending,
            &mut ops,
        )?;
        navigation::write(
            self,
            snapshot.navigation.snapshot(),
            &mut catalog.navigation,
            &mut ops,
        )?;
        let bytes = encode(&catalog)?;
        let generation = hash(&bytes);
        ops.push(WriteOp::Put {
            key: self.key(&format!("catalog/{generation}")),
            value: bytes,
        });
        ops.push(WriteOp::Put {
            key: self.key("head"),
            value: encode(&generation)?,
        });
        Ok(PreparedGraphPublication {
            ops,
            generation,
            catalog,
        })
    }
    fn append_node(
        &self,
        catalog: &mut Catalog,
        ops: &mut Vec<WriteOp>,
        mut row: IndexRow,
        node: StoredNode,
    ) -> Result<(), StoreError> {
        let bytes = encode(&node)?;
        row.hash = hash(&bytes);
        ops.push(WriteOp::Put {
            key: self.record_key(row.address, &row.hash),
            value: bytes,
        });
        catalog.entries.push(row);
        Ok(())
    }
}
#[cfg(test)]
#[path = "write_tests.rs"]
mod tests;
