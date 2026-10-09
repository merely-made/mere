// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use chartulary::stemma::{EntrySnapshot, OwnerSnapshot, StemmaSnapshot, VisitSnapshot};
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
#[derive(Clone, Serialize, Deserialize)]
pub(super) enum StoredNode {
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
impl StoredNode {
    pub fn edges(&self) -> &[(usize, PersistedEdge)] {
        match self {
            Self::Surface { edges, .. } | Self::Resource { edges, .. } => edges,
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
        let catalog: Catalog = checked(&bytes, &generation)?;
        if catalog.version != 1 {
            return Err(fail("unsupported addressable graph version"));
        }
        let unique: BTreeSet<_> = catalog.entries.iter().map(|e| e.address).collect();
        if unique.len() != catalog.entries.len() {
            return Err(fail("duplicate catalog address"));
        }
        Ok((generation, catalog, bytes.len()))
    }
    pub(super) async fn node(&self, row: &IndexRow) -> Result<StoredNode, StoreError> {
        let bytes = self
            .required(&self.record_key(row.address, &row.hash))
            .await?;
        checked(&bytes, &row.hash)
    }
}
