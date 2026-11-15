// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

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
#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Catalog {
    pub version: u8,
    pub globals: GraphSnapshot,
    pub entries: Vec<IndexRow>,
    pub navigation: NavigationIndex,
    pub orphan_facets: NodeFacetStore,
    pub pending: PendingIndex,
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
/// edges without version 1's snapshot index (engine split D175).
pub(super) const VERSION: u8 = 2;
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
pub(super) fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, StoreError> {
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
/// The catalog stored at `generation`, checked: its digest, a version this
/// binary reads (1 or 2) and one row per address.
pub(super) fn parse_catalog(bytes: &[u8], generation: &str) -> Result<Catalog, StoreError> {
    let catalog: Catalog = checked(bytes, generation)?;
    if !(1..=VERSION).contains(&catalog.version) {
        return Err(fail("unsupported addressable graph version"));
    }
    let unique: BTreeSet<_> = catalog.entries.iter().map(|e| e.address).collect();
    if unique.len() != catalog.entries.len() {
        return Err(fail("duplicate catalog address"));
    }
    Ok(catalog)
}
impl Catalog {
    pub fn row(&self, address: GraphAddress) -> Option<&IndexRow> {
        self.entries.iter().find(|e| e.address == address)
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
        let catalog = parse_catalog(&bytes, &generation)?;
        Ok((generation, catalog, bytes.len()))
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
