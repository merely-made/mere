// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Address existing navigation rows without retaining every owner's histories.
use super::*;
use kernel::graph::history::{NodeHistoryOwner, SharedNavigationMemory};
use muniment::WriteOp;
pub(super) fn write<B: Backend>(
    store: &AddressableGraphStore<B>,
    nav: &Navigation,
    index: &mut NavigationIndex,
    ops: &mut Vec<WriteOp>,
) -> Result<(), StoreError> {
    validate(nav)?;
    for row in &nav.entries {
        index.entries.push(append(store, "entry", row, ops)?);
    }
    for row in &nav.visits {
        index.visits.push(append(store, "visit", row, ops)?);
    }
    for row in &nav.owners {
        index.owners.push(append(store, "owner", row, ops)?);
        index.identities.push(row.identity.clone());
    }
    Ok(())
}
fn append<B: Backend, T: Serialize>(
    store: &AddressableGraphStore<B>,
    kind: &str,
    row: &T,
    ops: &mut Vec<WriteOp>,
) -> Result<String, StoreError> {
    let bytes = encode(row)?;
    let digest = hash(&bytes);
    ops.push(WriteOp::Put {
        key: store.key(&format!("navigation/{kind}/{digest}")),
        value: bytes,
    });
    Ok(digest)
}
fn validate(nav: &Navigation) -> Result<(), StoreError> {
    let visits = nav.visits.len();
    let owners = nav.owners.len();
    for v in &nav.visits {
        if v.entry >= nav.entries.len()
            || v.parent.is_some_and(|i| i >= visits)
            || v.children.iter().any(|i| *i >= visits)
            || v.bindings
                .iter()
                .any(|b| b.owner >= owners || b.forward_child.is_some_and(|i| i >= visits))
        {
            return Err(fail("invalid navigation visit reference"));
        }
    }
    for o in &nav.owners {
        if [o.origin, o.current, o.pending_origin_parent]
            .into_iter()
            .flatten()
            .any(|i| i >= visits)
            || o.owned_visits.iter().any(|i| *i >= visits)
            || o.creator.is_some_and(|i| i >= owners)
        {
            return Err(fail("invalid navigation owner reference"));
        }
    }
    Ok(())
}
async fn row<B: Backend, T: serde::de::DeserializeOwned>(
    store: &AddressableGraphStore<B>,
    kind: &str,
    hashes: &[String],
    idx: usize,
    bytes: &mut usize,
) -> Result<T, StoreError> {
    let digest = hashes
        .get(idx)
        .ok_or_else(|| fail("navigation index out of bounds"))?;
    let data = store
        .required(&store.key(&format!("navigation/{kind}/{digest}")))
        .await?;
    *bytes += data.len();
    checked(&data, digest)
}
pub(super) async fn load<B: Backend>(
    store: &AddressableGraphStore<B>,
    index: &NavigationIndex,
    addresses: &BTreeSet<GraphAddress>,
    full: bool,
) -> Result<(SharedNavigationMemory, usize), StoreError> {
    if index.identities.len() != index.owners.len() {
        return Err(fail("navigation identity index mismatch"));
    }
    let mut bytes = 0;
    let mut owners: BTreeMap<usize, Owner> = BTreeMap::new();
    let mut visits: BTreeMap<usize, Visit> = BTreeMap::new();
    let mut entries: BTreeMap<usize, Entry> = BTreeMap::new();
    let complete: BTreeSet<usize> = index
        .identities
        .iter()
        .enumerate()
        .filter(|(_, owner)| {
            full || match owner {
                NodeHistoryOwner::Node(id) => Uuid::parse_str(id)
                    .is_ok_and(|id| addresses.contains(&GraphAddress::Surface(id))),
            }
        })
        .map(|(i, _)| i)
        .collect();
    let mut pending_owners = complete.clone();
    let mut pending_visits = BTreeSet::new();
    if full {
        pending_visits.extend(0..index.visits.len());
    }
    while !pending_owners.is_empty() || !pending_visits.is_empty() {
        while let Some(i) = pending_owners.pop_first() {
            if owners.contains_key(&i) {
                continue;
            }
            let owner: Owner = row(store, "owner", &index.owners, i, &mut bytes).await?;
            if index.identities.get(i) != Some(&owner.identity) {
                return Err(fail("navigation owner identity mismatch"));
            }
            if let Some(c) = owner.creator {
                pending_owners.insert(c);
            }
            if complete.contains(&i) {
                pending_visits.extend(owner.owned_visits.iter().copied());
                pending_visits.extend(
                    [owner.origin, owner.current, owner.pending_origin_parent]
                        .into_iter()
                        .flatten(),
                );
            }
            owners.insert(i, owner);
        }
        while let Some(i) = pending_visits.pop_first() {
            if visits.contains_key(&i) {
                continue;
            }
            let visit: Visit = row(store, "visit", &index.visits, i, &mut bytes).await?;
            if let Some(p) = visit.parent {
                pending_visits.insert(p);
            }
            for b in &visit.bindings {
                pending_owners.insert(b.owner);
            }
            visits.insert(i, visit);
        }
    }
    let needed_entries: BTreeSet<_> = if full {
        (0..index.entries.len()).collect()
    } else {
        visits.values().map(|v| v.entry).collect()
    };
    for i in needed_entries {
        entries.insert(i, row(store, "entry", &index.entries, i, &mut bytes).await?);
    }
    let em: BTreeMap<_, _> = entries
        .keys()
        .enumerate()
        .map(|(new, old)| (*old, new))
        .collect();
    let vm: BTreeMap<_, _> = visits
        .keys()
        .enumerate()
        .map(|(new, old)| (*old, new))
        .collect();
    let om: BTreeMap<_, _> = owners
        .keys()
        .enumerate()
        .map(|(new, old)| (*old, new))
        .collect();
    let optional =
        |old: Option<usize>, map: &BTreeMap<usize, usize>| old.and_then(|i| map.get(&i).copied());
    let mut nav = Navigation {
        entries: entries.into_values().collect(),
        visits: vec![],
        owners: vec![],
    };
    for (_, mut v) in visits {
        v.entry = *em
            .get(&v.entry)
            .ok_or_else(|| fail("missing navigation entry"))?;
        v.parent = optional(v.parent, &vm);
        v.children = v
            .children
            .into_iter()
            .filter_map(|i| vm.get(&i).copied())
            .collect();
        v.bindings.retain_mut(|b| {
            if let Some(i) = om.get(&b.owner) {
                b.owner = *i;
                b.forward_child = optional(b.forward_child, &vm);
                true
            } else {
                false
            }
        });
        nav.visits.push(v);
    }
    for (_, mut o) in owners {
        o.origin = optional(o.origin, &vm);
        o.current = optional(o.current, &vm);
        o.pending_origin_parent = optional(o.pending_origin_parent, &vm);
        o.creator = optional(o.creator, &om);
        o.owned_visits = o
            .owned_visits
            .into_iter()
            .filter_map(|i| vm.get(&i).copied())
            .collect();
        nav.owners.push(o);
    }
    validate(&nav)?;
    if full && hash(&encode(&nav)?) != index.digest {
        return Err(fail("full navigation digest mismatch"));
    }
    Ok((SharedNavigationMemory::from_snapshot(nav), bytes))
}
