// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Retained P3 observations follow their Resource owner, outside graph truth.
use super::*;
use kernel::graph::{PendingLink, PendingLinkState};
use muniment::WriteOp;
pub(super) fn write<B: Backend>(
    store: &AddressableGraphStore<B>,
    state: &PendingLinkState,
    index: &mut PendingIndex,
    ops: &mut Vec<WriteOp>,
) -> Result<(), StoreError> {
    let mut owners: BTreeMap<Uuid, Vec<(usize, PendingLink)>> = BTreeMap::new();
    for (ordinal, link) in state.entries.iter().enumerate() {
        owners
            .entry(link.source_resource)
            .or_default()
            .push((ordinal, link.clone()));
    }
    for (id, links) in owners {
        let bytes = encode(&links)?;
        let digest = hash(&bytes);
        ops.push(WriteOp::Put {
            key: store.key(&format!("pending/{id}/{digest}")),
            value: bytes,
        });
        index.owners.push((id, digest));
    }
    Ok(())
}
pub(super) async fn load<B: Backend>(
    store: &AddressableGraphStore<B>,
    index: &PendingIndex,
    addresses: &BTreeSet<GraphAddress>,
    full: bool,
) -> Result<(PendingLinkState, usize), StoreError> {
    let mut entries = BTreeMap::new();
    let mut bytes = 0;
    for (id, digest) in &index.owners {
        if !full && !addresses.contains(&GraphAddress::Resource(*id)) {
            continue;
        }
        let data = store
            .required(&store.key(&format!("pending/{id}/{digest}")))
            .await?;
        bytes += data.len();
        let links: Vec<(usize, PendingLink)> = checked(&data, digest)?;
        for (ordinal, link) in links {
            if link.source_resource != *id || entries.insert(ordinal, link).is_some() {
                return Err(fail("invalid pending observation ownership/order"));
            }
        }
    }
    Ok((
        PendingLinkState {
            retention: index.retention,
            entries: entries.into_values().collect(),
        },
        bytes,
    ))
}
