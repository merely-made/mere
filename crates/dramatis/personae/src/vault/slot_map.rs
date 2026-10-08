// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A profile's slots, in storage that is wiped whole before it is freed
//! (vault lock ruling 71).
//!
//! A slot moved into a collection carries its padding with it, and padding
//! can hold stale stack bytes, a seed among them. A `HashMap` frees its table
//! uncleared when it drops or grows. This is a small vector instead: on
//! drop, after a removal and before giving up a buffer it outgrew, the
//! whole buffer is zeroed, spare capacity included. A profile holds a
//! handful of slots, so lookups scan.

use std::mem::MaybeUninit;

use zeroize::Zeroize;

use super::{IdentitySlot, ProtocolKey};

/// Per-protocol slots keyed by [`ProtocolKey`], at most one per key.
#[derive(Default)]
pub struct SlotMap(Vec<(ProtocolKey, IdentitySlot)>);

/// Zero every byte past the length: what removals and moves left behind.
fn wipe_spare(slots: &mut Vec<(ProtocolKey, IdentitySlot)>) {
    let spare: &mut [MaybeUninit<(ProtocolKey, IdentitySlot)>] = slots.spare_capacity_mut();
    spare.zeroize();
}

impl SlotMap {
    /// No slots.
    pub fn new() -> Self {
        Self::default()
    }

    /// Room for `capacity` slots before the first growth.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    /// How many slots.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether there are none.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn position(&self, key: &ProtocolKey) -> Option<usize> {
        self.0.iter().position(|(k, _)| k == key)
    }

    /// The slot for `key`.
    pub fn get(&self, key: &ProtocolKey) -> Option<&IdentitySlot> {
        self.position(key).map(|i| &self.0[i].1)
    }

    /// Whether `key` has a slot.
    pub fn contains_key(&self, key: &ProtocolKey) -> bool {
        self.position(key).is_some()
    }

    /// Set the slot for `key`, returning the one it replaces.
    pub fn insert(&mut self, key: ProtocolKey, slot: IdentitySlot) -> Option<IdentitySlot> {
        if let Some(i) = self.position(&key) {
            return Some(std::mem::replace(&mut self.0[i].1, slot));
        }
        if self.0.len() == self.0.capacity() {
            // Grow by hand, so the outgrown buffer is wiped before it is freed.
            let mut bigger = Vec::with_capacity((self.0.capacity() * 2).max(4));
            bigger.append(&mut self.0);
            wipe_spare(&mut self.0);
            self.0 = bigger;
        }
        self.0.push((key, slot));
        None
    }

    /// Remove the slot for `key`.
    pub fn remove(&mut self, key: &ProtocolKey) -> Option<IdentitySlot> {
        let i = self.position(key)?;
        let (_, slot) = self.0.remove(i);
        wipe_spare(&mut self.0);
        Some(slot)
    }

    /// Each key and its slot.
    pub fn iter(&self) -> impl Iterator<Item = (&ProtocolKey, &IdentitySlot)> {
        self.0.iter().map(|(key, slot)| (key, slot))
    }

    /// Each key.
    pub fn keys(&self) -> impl Iterator<Item = &ProtocolKey> {
        self.0.iter().map(|(key, _)| key)
    }

    /// Each slot.
    pub fn values(&self) -> impl Iterator<Item = &IdentitySlot> {
        self.0.iter().map(|(_, slot)| slot)
    }
}

impl std::ops::Index<&ProtocolKey> for SlotMap {
    type Output = IdentitySlot;

    /// The slot for `key`; panics when there is none, as a map's index does.
    fn index(&self, key: &ProtocolKey) -> &IdentitySlot {
        self.get(key).expect("no slot for this protocol key")
    }
}

impl Drop for SlotMap {
    fn drop(&mut self) {
        // The slots' own secrets zeroize as they drop; then the buffer.
        self.0.clear();
        wipe_spare(&mut self.0);
    }
}

impl FromIterator<(ProtocolKey, IdentitySlot)> for SlotMap {
    fn from_iter<I: IntoIterator<Item = (ProtocolKey, IdentitySlot)>>(iter: I) -> Self {
        let mut slots = Self::new();
        slots.extend(iter);
        slots
    }
}

impl Extend<(ProtocolKey, IdentitySlot)> for SlotMap {
    fn extend<I: IntoIterator<Item = (ProtocolKey, IdentitySlot)>>(&mut self, iter: I) {
        for (key, slot) in iter {
            self.insert(key, slot);
        }
    }
}

impl<'a> IntoIterator for &'a SlotMap {
    type Item = (&'a ProtocolKey, &'a IdentitySlot);
    type IntoIter = std::iter::Map<
        std::slice::Iter<'a, (ProtocolKey, IdentitySlot)>,
        fn(&'a (ProtocolKey, IdentitySlot)) -> (&'a ProtocolKey, &'a IdentitySlot),
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().map(|(key, slot)| (key, slot))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::{CredentialLineage, SecretBytes, UnlockTier};

    fn key(name: &str) -> ProtocolKey {
        ProtocolKey::new(name, Some("default".into()))
    }

    fn slot(byte: u8) -> IdentitySlot {
        IdentitySlot::Direct {
            kind: "nostr".into(),
            payload: SecretBytes::new(vec![byte; 32]),
            lineage: CredentialLineage::LocallyGeneratedExternallyRegistered,
            unlock_tier: UnlockTier::Session,
        }
    }

    fn payload(slot: &IdentitySlot) -> u8 {
        match slot {
            IdentitySlot::Direct { payload, .. } => payload.as_slice()[0],
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_behaves_as_a_map_through_growth_and_removal() {
        let mut slots = SlotMap::new();
        for n in 0..9u8 {
            assert!(slots.insert(key(&format!("p{n}")), slot(n)).is_none());
        }
        assert_eq!(slots.len(), 9);
        let replaced = slots.insert(key("p3"), slot(33)).unwrap();
        assert_eq!(payload(&replaced), 3);
        assert_eq!(payload(slots.get(&key("p3")).unwrap()), 33);
        assert_eq!(payload(&slots.remove(&key("p0")).unwrap()), 0);
        assert!(!slots.contains_key(&key("p0")));
        assert!(slots.remove(&key("p0")).is_none());
        assert_eq!(slots.keys().count(), 8);
        assert_eq!(payload(&slots[&key("p8")]), 8);
        let collected: SlotMap = (0..3u8).map(|n| (key(&format!("c{n}")), slot(n))).collect();
        assert_eq!(collected.len(), 3);
        assert_eq!((&collected).into_iter().count(), 3);
    }
}
