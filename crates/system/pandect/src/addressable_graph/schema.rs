// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::tree::{TreeRef, Trees};
use super::*;
use hodograph::{EntrySnapshot, OwnerSnapshot, StemmaSnapshot, VisitSnapshot};
use kernel::graph::history::NodeHistoryOwner;
use kernel::persistence::{GraphSnapshot, PersistedEdge, PersistedNode, PersistedResourceRecord};
pub(super) type Navigation = StemmaSnapshot<String, String, NodeHistoryOwner, ()>;
pub(super) type Entry = EntrySnapshot<String, String>;
pub(super) type Visit = VisitSnapshot<()>;
pub(super) type Owner = OwnerSnapshot<NodeHistoryOwner>;
#[derive(Clone, Serialize, Deserialize)]
pub(super) struct IndexRow {
    pub address: GraphAddress,
    pub hash: String,
    pub shown: Option<Uuid>,
    pub pinned: bool,
    pub declaration: bool,
}
/// A catalog in memory, and versions 1 and 2's stored form. From version 3
/// the lists are stored as trees a [`Root`] names (engine split D177).
#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Catalog {
    pub version: u8,
    pub globals: GraphSnapshot,
    pub entries: Vec<IndexRow>,
    pub navigation: NavigationIndex,
    pub orphan_facets: NodeFacetStore,
    pub pending: PendingIndex,
    /// Built once per catalog: by `parse_catalog` for a stored one, on the
    /// first lookup for a prepared one. Not stored.
    #[serde(skip)]
    pub index: std::sync::OnceLock<RowIndex>,
    /// The trees the lists are stored as (version 3). Not stored.
    #[serde(skip)]
    pub trees: Trees,
}
/// A version 3 catalog as stored: its lists as trees, the rest whole. Its
/// digest is the generation, and it names every row through the trees.
#[derive(Serialize, Deserialize)]
pub(super) struct Root {
    pub version: u8,
    pub globals: GraphSnapshot,
    pub rows: TreeRef,
    pub navigation: NavigationRoot,
    pub orphan_facets: NodeFacetStore,
    pub pending: PendingRoot,
}
#[derive(Serialize, Deserialize)]
pub(super) struct NavigationRoot {
    pub entries: TreeRef,
    pub visits: TreeRef,
    /// Each owner's digest with its identity.
    pub owners: TreeRef,
    pub digest: String,
}
#[derive(Serialize, Deserialize)]
pub(super) struct PendingRoot {
    pub retention: kernel::graph::PendingLinkRetention,
    pub owners: TreeRef,
}
/// Each row's position by address, and the Surfaces showing each Resource in
/// row order (engine split D177: `row` was a scan, so a full reader's
/// selection was quadratic, 23.6 s of a 32.8 s open at 100k).
#[derive(Clone, Default)]
pub(super) struct RowIndex {
    at: BTreeMap<GraphAddress, usize>,
    showing: BTreeMap<Uuid, Vec<usize>>,
}
impl RowIndex {
    fn of(entries: &[IndexRow]) -> Self {
        let mut index = Self::default();
        for (i, row) in entries.iter().enumerate() {
            // The first row wins, as the scan's `find` did.
            index.at.entry(row.address).or_insert(i);
            if let Some(resource) = row.shown {
                index.showing.entry(resource).or_default().push(i);
            }
        }
        index
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub(super) struct PendingIndex {
    pub retention: kernel::graph::PendingLinkRetention,
    pub owners: Vec<(Uuid, String)>,
}
#[derive(Clone, Serialize, Deserialize)]
pub(super) struct NavigationIndex {
    pub entries: Vec<String>,
    pub visits: Vec<String>,
    pub owners: Vec<String>,
    pub identities: Vec<NodeHistoryOwner>,
    pub digest: String,
}
/// The catalog version this binary writes. Version 2 records list their
/// edges without version 1's snapshot index (engine split D175); version 3
/// stores the catalog's lists as content-defined trees (D177) and its records
/// as version 2 does.
pub(super) const VERSION: u8 = 3;
/// A stored node. Each edge with both endpoints in the stratum is listed by
/// both endpoints' records, in snapshot order; the reader merges the two copies
/// by pair and restores the snapshot's order from content (D175).
#[derive(Clone, Serialize, Deserialize)]
pub(super) enum StoredNode {
    Surface {
        node: PersistedNode,
        facets: Vec<(String, serde_json::Value)>,
        edges: Vec<PersistedEdge>,
    },
    Resource {
        record: PersistedResourceRecord,
        edges: Vec<PersistedEdge>,
    },
}
impl StoredNode {
    pub fn edges(&self) -> &[PersistedEdge] {
        match self {
            Self::Surface { edges, .. } | Self::Resource { edges, .. } => edges,
        }
    }
}
/// A version 1 record: each edge with its position in the snapshot's edge
/// column. It still loads, the position ignored; the next publication
/// rewrites it.
#[derive(Deserialize)]
#[cfg_attr(test, derive(Serialize))]
pub(super) enum StoredNodeV1 {
    Surface {
        node: PersistedNode,
        facets: Vec<(String, serde_json::Value)>,
        edges: Vec<(usize, PersistedEdge)>,
    },
    Resource {
        record: PersistedResourceRecord,
        edges: Vec<(usize, PersistedEdge)>,
    },
}
impl From<StoredNodeV1> for StoredNode {
    fn from(node: StoredNodeV1) -> Self {
        let plain =
            |edges: Vec<(usize, PersistedEdge)>| edges.into_iter().map(|(_, e)| e).collect();
        match node {
            StoredNodeV1::Surface {
                node,
                facets,
                edges,
            } => Self::Surface {
                node,
                facets,
                edges: plain(edges),
            },
            StoredNodeV1::Resource { record, edges } => Self::Resource {
                record,
                edges: plain(edges),
            },
        }
    }
}
pub(super) fn fail(message: impl Into<String>) -> StoreError {
    StoreError::Codec(message.into())
}
pub(super) fn encode<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, StoreError> {
    serde_json::to_vec(value).map_err(|e| fail(e.to_string()))
}
pub(super) fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, StoreError> {
    serde_json::from_slice(bytes).map_err(|e| fail(e.to_string()))
}
pub(super) fn hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}
pub(super) fn checked<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    digest: &str,
) -> Result<T, StoreError> {
    if hash(bytes) != digest {
        return Err(fail("addressable record checksum mismatch"));
    }
    decode(bytes)
}
/// A stored catalog: whole (versions 1 and 2) or a root naming trees (3).
pub(super) enum Stored {
    Whole(Box<Catalog>),
    Root(Box<Root>),
}
/// The catalog stored at `generation`, checked against it, decoded as its
/// version wrote it. A whole one is [finished](finish); a root's trees are
/// still to read.
pub(super) fn parse_catalog(bytes: &[u8], generation: &str) -> Result<Stored, StoreError> {
    #[derive(Deserialize)]
    struct Version {
        version: u8,
    }
    if hash(bytes) != generation {
        return Err(fail("addressable record checksum mismatch"));
    }
    match decode::<Version>(bytes)?.version {
        1 | 2 => finish(decode(bytes)?).map(|catalog| Stored::Whole(Box::new(catalog))),
        3 => Ok(Stored::Root(Box::new(decode(bytes)?))),
        _ => Err(fail("unsupported addressable graph version")),
    }
}
/// `catalog` with its row index built: one row per address.
pub(super) fn finish(catalog: Catalog) -> Result<Catalog, StoreError> {
    let index = RowIndex::of(&catalog.entries);
    if index.at.len() != catalog.entries.len() {
        return Err(fail("duplicate catalog address"));
    }
    let _ = catalog.index.set(index);
    Ok(catalog)
}
impl Catalog {
    fn index(&self) -> &RowIndex {
        self.index.get_or_init(|| RowIndex::of(&self.entries))
    }
    pub fn row(&self, address: GraphAddress) -> Option<&IndexRow> {
        let at = *self.index().at.get(&address)?;
        Some(&self.entries[at])
    }
    /// The rows of the Surfaces showing `resource`, in row order.
    pub fn showing(&self, resource: Uuid) -> impl Iterator<Item = &IndexRow> {
        let rows = self.index().showing.get(&resource);
        rows.into_iter().flatten().map(|&i| &self.entries[i])
    }
}
impl<B: Backend> AddressableGraphStore<B> {
    pub(super) fn record_key(&self, address: GraphAddress, digest: &str) -> String {
        let layer = match address {
            GraphAddress::Surface(_) => "surface",
            GraphAddress::Resource(_) => "resource",
        };
        self.key(&format!("nodes/{layer}/{}/{digest}", address.id()))
    }
    pub(super) fn chunk_key(&self, digest: &str) -> String {
        self.key(&format!("chunks/{digest}"))
    }
    pub(super) async fn required(&self, key: &str) -> Result<Vec<u8>, StoreError> {
        self.backend
            .get(key)
            .await?
            .ok_or_else(|| fail(format!("missing addressable record: {key}")))
    }
    pub(super) async fn catalog(&self) -> Result<(String, Catalog, usize), StoreError> {
        let generation: String = decode(&self.required(&self.key("head")).await?)?;
        let bytes = self
            .required(&self.key(&format!("catalog/{generation}")))
            .await?;
        let mut size = bytes.len();
        let catalog = match parse_catalog(&bytes, &generation)? {
            Stored::Whole(catalog) => *catalog,
            Stored::Root(root) => tree::load_catalog(self, *root, &mut size).await?,
        };
        Ok((generation, catalog, size))
    }
    /// The record `row` names, decoded as its catalog's `version` wrote it.
    pub(super) async fn node(&self, row: &IndexRow, version: u8) -> Result<StoredNode, StoreError> {
        let bytes = self
            .required(&self.record_key(row.address, &row.hash))
            .await?;
        match version {
            1 => checked::<StoredNodeV1>(&bytes, &row.hash).map(StoredNode::from),
            _ => checked(&bytes, &row.hash),
        }
    }
}
