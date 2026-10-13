// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::{BTreeMap, VecDeque};
use stickleback::inventory::{Inventory, InventoryLimits, ObjectDigest, RetainedEntry, Snapshot};

use super::IngressError;

/// A volatile owner-held availability window after domain admission. This is
/// neither the durable operation store nor a receipt promising future custody.
pub(super) struct Records {
    inventory: Inventory,
    bytes: BTreeMap<ObjectDigest, Vec<u8>>,
    fifo: VecDeque<ObjectDigest>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifth_original_evicts_only_availability_and_expiry_removes_exact_bytes() {
        let mut records = Records::new([1; 32]);
        let originals: Vec<_> = (0..5).map(|i| vec![i; 16 * 1024]).collect();
        for original in &originals {
            records.retain(original.clone(), 0, 10).unwrap();
        }
        assert_eq!(records.snapshot(0).unwrap().len(), 4);
        assert!(
            records
                .original(blake3::hash(&originals[0]).as_bytes(), 0)
                .unwrap()
                .is_none()
        );
        for original in originals.iter().skip(1) {
            assert_eq!(
                records
                    .original(blake3::hash(original).as_bytes(), 9)
                    .unwrap()
                    .as_ref(),
                Some(original)
            );
        }
        assert!(records.snapshot(10).unwrap().is_empty());
        assert!(records.bytes.is_empty());
        assert!(records.fifo.is_empty());
    }
}

impl Records {
    pub(super) fn new(mesh: [u8; 32]) -> Self {
        Self {
            inventory: Inventory::new(
                mesh,
                InventoryLimits {
                    entries: 4,
                    retained_bytes: 64 * 1024,
                    object_bytes: 16 * 1024,
                },
            )
            .unwrap(),
            bytes: BTreeMap::new(),
            fifo: VecDeque::new(),
        }
    }

    fn expire(&mut self, now: u64) -> Result<(), IngressError> {
        for id in self.inventory.expire(now)? {
            self.bytes.remove(&id);
            self.fifo.retain(|p| *p != id);
        }
        Ok(())
    }

    pub(super) fn retain(
        &mut self,
        bytes: Vec<u8>,
        now: u64,
        expires_at: u64,
    ) -> Result<(), IngressError> {
        self.expire(now)?;
        let digest = *blake3::hash(&bytes).as_bytes();
        if !self.bytes.contains_key(&digest) {
            if self.fifo.len() == 4 {
                let old = self.fifo.pop_front().unwrap();
                self.bytes.remove(&old);
                self.inventory.remove(&old)?;
            }
            self.fifo.push_back(digest);
        }
        self.inventory.retain(
            RetainedEntry {
                digest,
                bytes: bytes.len(),
                expires_at,
            },
            now,
        )?;
        self.bytes.insert(digest, bytes);
        Ok(())
    }

    pub(super) fn snapshot(&mut self, now: u64) -> Result<Snapshot, IngressError> {
        self.expire(now)?;
        Ok(self.inventory.snapshot(now))
    }

    pub(super) fn original(
        &mut self,
        id: &ObjectDigest,
        now: u64,
    ) -> Result<Option<Vec<u8>>, IngressError> {
        self.expire(now)?;
        Ok(self.bytes.get(id).cloned())
    }
}
