// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Deleting what a publication supersedes, in its own transaction (engine
//! split D173). The rules are in the [module docs](super).
use super::*;
use muniment::{TransactionReader, WriteOp};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// The keys a publication names under `prefix`, spelled as its writers spell
/// them: records, navigation rows, pending owners and the catalog itself.
fn named_keys(prefix: &str, generation: &str, catalog: &Catalog) -> HashSet<String> {
    let mut keys = HashSet::new();
    keys.insert(format!("{prefix}catalog/{generation}"));
    for row in &catalog.entries {
        let layer = match row.address {
            GraphAddress::Surface(_) => "surface",
            GraphAddress::Resource(_) => "resource",
        };
        let id = row.address.id();
        keys.insert(format!("{prefix}nodes/{layer}/{id}/{}", row.hash));
    }
    for (kind, digests) in [
        ("entry", &catalog.navigation.entries),
        ("visit", &catalog.navigation.visits),
        ("owner", &catalog.navigation.owners),
    ] {
        for digest in digests {
            keys.insert(format!("{prefix}navigation/{kind}/{digest}"));
        }
    }
    for (id, digest) in &catalog.pending.owners {
        keys.insert(format!("{prefix}pending/{id}/{digest}"));
    }
    keys
}

/// What the head names, read in a transaction. `None` with no head, or one
/// whose catalog cannot be read: then nothing is held and nothing collected.
fn named_by_head(reader: &dyn TransactionReader, prefix: &str) -> Option<HashSet<String>> {
    let generation: String = decode(&reader.get(&format!("{prefix}head")).ok()??).ok()?;
    let bytes = reader
        .get(&format!("{prefix}catalog/{generation}"))
        .ok()??;
    let catalog = parse_catalog(&bytes, &generation).ok()?;
    Some(named_keys(prefix, &generation, &catalog))
}

/// A prepared publication's writes, decided against the head inside the
/// transaction that commits them.
pub(crate) struct StagedPublication {
    ops: Vec<WriteOp>,
    prefix: String,
    /// What the new catalog names, when the store collects.
    collect: Option<HashSet<String>>,
}
impl StagedPublication {
    /// The writes to commit against `reader`'s snapshot: every put less those
    /// the head names (D171), then, when collecting, a delete for each key the
    /// head names that the new catalog does not (D173). Both come from one
    /// read of the head.
    pub(crate) fn decide(self, reader: &dyn TransactionReader) -> Vec<WriteOp> {
        match named_by_head(reader, &self.prefix) {
            Some(held) => self.against(&held),
            None => self.ops,
        }
    }
    fn against(self, held: &HashSet<String>) -> Vec<WriteOp> {
        let mut ops = self.ops;
        ops.retain(|op| !matches!(op, WriteOp::Put { key, .. } if held.contains(key)));
        if let Some(named) = &self.collect {
            let mut superseded: Vec<&String> =
                held.iter().filter(|key| !named.contains(*key)).collect();
            superseded.sort();
            ops.extend(
                superseded
                    .into_iter()
                    .map(|key| WriteOp::Delete { key: key.clone() }),
            );
        }
        ops
    }
}

impl<B: Backend> AddressableGraphStore<B> {
    /// `prepared`, staged to be decided inside the transaction that commits it.
    pub(crate) fn stage(&self, prepared: PreparedGraphPublication) -> StagedPublication {
        let prefix = self.key("");
        let (ops, generation, catalog) = prepared.into_parts();
        let collect =
            (self.backend.single_owner()).then(|| named_keys(&prefix, &generation, &catalog));
        StagedPublication {
            ops,
            prefix,
            collect,
        }
    }
}

/// A transaction needs a `Sync` backend natively, as the session's commit does.
impl<B: crate::graph_session::LegacyTransactionBackend> AddressableGraphStore<B> {
    /// Commit `staged` in one transaction. A backend without transactions
    /// cannot decide under the writer's lock: it skips what the head held when
    /// read just before and deletes nothing.
    pub(super) async fn commit_staged(&self, staged: StagedPublication) -> Result<(), StoreError> {
        let slot = Arc::new(Mutex::new(Some(staged)));
        let taken = slot.clone();
        let committed = self
            .backend
            .transact(Box::new(move |reader: &dyn TransactionReader| {
                let staged = taken.lock().expect("staged publication").take();
                staged.map(|s| s.decide(reader)).unwrap_or_default()
            }))
            .await;
        if committed != Err(StoreError::NotTransactional) {
            return committed;
        }
        let staged = slot.lock().expect("staged publication").take();
        let Some(mut staged) = staged else {
            return Err(StoreError::NotTransactional);
        };
        staged.collect = None;
        let held = match self.catalog().await {
            Ok((generation, catalog, _)) => named_keys(&staged.prefix, &generation, &catalog),
            Err(_) => HashSet::new(),
        };
        self.backend.apply(&staged.against(&held)).await
    }
}
