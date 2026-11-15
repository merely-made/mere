// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Deleting what a publication supersedes, in its own transaction, and the
//! one sweep the first collecting publication runs (engine split D173,
//! D175). The rules are in the [module docs](super).
use super::*;
use muniment::{TransactionReader, WriteOp};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// The record the one sweep leaves under the graph's prefix.
const SWEPT: &str = "swept";

/// How a publication's writers spell a row's record key under `prefix`.
fn row_key(prefix: &str, row: &IndexRow) -> String {
    let layer = match row.address {
        GraphAddress::Surface(_) => "surface",
        GraphAddress::Resource(_) => "resource",
    };
    let id = row.address.id();
    format!("{prefix}nodes/{layer}/{id}/{}", row.hash)
}
fn navigation_key(prefix: &str, kind: &str, digest: &str) -> String {
    format!("{prefix}navigation/{kind}/{digest}")
}
fn pending_key(prefix: &str, (id, digest): &(Uuid, String)) -> String {
    format!("{prefix}pending/{id}/{digest}")
}
/// The keys each of `catalog`'s lists names, in list order, as its trees
/// hold them: rows, navigation entries, visits and owners, pending owners.
fn list_keys(prefix: &str, catalog: &Catalog) -> [Vec<String>; 5] {
    let nav = &catalog.navigation;
    let spelled = |kind: &str, digests: &[String]| {
        (digests.iter())
            .map(|digest| navigation_key(prefix, kind, digest))
            .collect()
    };
    [
        catalog
            .entries
            .iter()
            .map(|row| row_key(prefix, row))
            .collect(),
        spelled("entry", &nav.entries),
        spelled("visit", &nav.visits),
        spelled("owner", &nav.owners),
        (catalog.pending.owners.iter())
            .map(|owner| pending_key(prefix, owner))
            .collect(),
    ]
}

/// The keys a publication names under `prefix`, spelled as its writers spell
/// them: records, navigation rows, pending owners, the catalog itself and,
/// from version 3, the chunks of its trees.
pub(super) fn named_keys(prefix: &str, generation: &str, catalog: &Catalog) -> HashSet<String> {
    let mut keys: HashSet<String> = list_keys(prefix, catalog).into_iter().flatten().collect();
    keys.insert(format!("{prefix}catalog/{generation}"));
    keys.extend((catalog.trees.digests()).map(|digest| format!("{prefix}chunks/{digest}")));
    keys
}

/// What the head names, read in a transaction. A version 3 head's chunks are
/// read only where the new catalog (`new`) lacks them (D177, "Only changed
/// chunks"): a chunk both name holds the same items, so the keys under it
/// come from `new`. `None` with no head, or one whose catalog cannot be
/// read: then nothing is held and nothing collected.
fn named_by_head(
    reader: &dyn TransactionReader,
    prefix: &str,
    new: &Catalog,
) -> Option<HashSet<String>> {
    let generation: String = decode(&reader.get(&format!("{prefix}head")).ok()??).ok()?;
    let bytes = reader
        .get(&format!("{prefix}catalog/{generation}"))
        .ok()??;
    let root = match parse_catalog(&bytes, &generation).ok()? {
        Stored::Whole(old) => return Some(named_keys(prefix, &generation, &old)),
        Stored::Root(root) => root,
    };
    let mut held = HashSet::from([format!("{prefix}catalog/{generation}")]);
    let chunk = |digest: &str| format!("{prefix}chunks/{digest}");
    let [rows, entries, visits, owners, pending] = list_keys(prefix, new);
    let (trees, nav) = (&new.trees, &root.navigation);
    let navigation =
        |kind: &'static str| move |digest: &String| navigation_key(prefix, kind, digest);
    tree::held(
        reader,
        &chunk,
        &root.rows,
        (&trees.rows, &rows),
        |row| row_key(prefix, row),
        &mut held,
    )?;
    tree::held(
        reader,
        &chunk,
        &nav.entries,
        (&trees.entries, &entries),
        navigation("entry"),
        &mut held,
    )?;
    tree::held(
        reader,
        &chunk,
        &nav.visits,
        (&trees.visits, &visits),
        navigation("visit"),
        &mut held,
    )?;
    tree::held(
        reader,
        &chunk,
        &nav.owners,
        (&trees.owners, &owners),
        |(digest, _): &(String, kernel::graph::history::NodeHistoryOwner)| {
            navigation_key(prefix, "owner", digest)
        },
        &mut held,
    )?;
    let pending_of = |owner: &(Uuid, String)| pending_key(prefix, owner);
    tree::held(
        reader,
        &chunk,
        &root.pending.owners,
        (&trees.pending, &pending),
        pending_of,
        &mut held,
    )?;
    Some(held)
}

/// A prepared publication's writes, decided against the head inside the
/// transaction that commits them.
pub(crate) struct StagedPublication {
    ops: Vec<WriteOp>,
    prefix: String,
    /// The new catalog, whose trees spare the decision reading the head's
    /// unchanged chunks.
    catalog: Catalog,
    /// What the new catalog names, when the store collects.
    collect: Option<HashSet<String>>,
}
impl StagedPublication {
    /// The writes to commit against `reader`'s snapshot: every put less those
    /// the head names (D171), then, when collecting, a delete for each key the
    /// head names that the new catalog does not (D173). Both come from one
    /// read of the head. The first collecting publication on a store also
    /// sweeps what earlier publications left (see [`sweep`]).
    pub(crate) fn decide(mut self, reader: &dyn TransactionReader) -> Vec<WriteOp> {
        let held = named_by_head(reader, &self.prefix, &self.catalog);
        let swept = (self.collect.as_ref()).and_then(|named| sweep(reader, &self.prefix, named));
        if swept.is_some() {
            // The sweep's deletes already cover what the head supersedes.
            self.collect = None;
        }
        let mut ops = match &held {
            Some(held) => self.against(held),
            None => self.ops,
        };
        ops.extend(swept.into_iter().flatten());
        ops
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
            catalog,
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

/// The key families a publication writes under the graph's prefix. The sweep
/// touches nothing else there: `head`, `policy` and a host's own keys beside
/// them (Graphshell's `host-settings`) stay.
const FAMILIES: &[&str] = &["nodes/", "catalog/", "chunks/", "navigation/", "pending/"];

/// The one sweep (D173, "One sweep at open, once"), run by the first
/// collecting publication on a store inside its transaction: a delete for
/// every key in the publication's [families](FAMILIES) the new catalog
/// (`named`) does not name, and the `swept` record that stops it running
/// again. After this transaction the new head is what readers open, so what
/// it does not name there is garbage, whatever the old head was. `None` once
/// swept, or when the store cannot be read here (tried again next time).
fn sweep(
    reader: &dyn TransactionReader,
    prefix: &str,
    named: &HashSet<String>,
) -> Option<Vec<WriteOp>> {
    let swept = format!("{prefix}{SWEPT}");
    if reader.get(&swept).ok()?.is_some() {
        return None;
    }
    let mut garbage = Vec::new();
    for family in FAMILIES.iter() {
        let mut keys = reader.list(&format!("{prefix}{family}")).ok()?;
        keys.retain(|key| !named.contains(key));
        garbage.append(&mut keys);
    }
    garbage.sort();
    let mut ops: Vec<WriteOp> = garbage
        .into_iter()
        .map(|key| WriteOp::Delete { key })
        .collect();
    ops.push(WriteOp::Put {
        key: swept,
        value: encode(&1u8).ok()?,
    });
    Some(ops)
}
