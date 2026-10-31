// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Borrowed protection for private sibling lanes on a personal graph.
//!
//! The personal-sync host alone changes membership and installs epochs. A
//! sibling can seal and open bytes, but cannot export keys or create a group.

use std::{
    collections::BTreeSet,
    sync::{Arc, RwLock},
};

use stickleback::{DataKeyring, GroupCiphertext};

pub(crate) struct SealingState {
    pub keys: Option<Arc<DataKeyring>>,
    pub current_roots: BTreeSet<[u8; 32]>,
    pub historical_roots: BTreeSet<[u8; 32]>,
    pub can_write: bool,
    pub revision: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_reads_require_a_current_seat_and_current_local_key() {
        let root = [17; 32];
        let mut keys = DataKeyring::new();
        keys.rotate_random().unwrap();
        let state = Arc::new(RwLock::new(SealingState {
            keys: Some(Arc::new(keys)),
            current_roots: BTreeSet::from([root]),
            historical_roots: BTreeSet::from([root]),
            can_write: true,
            revision: 1,
        }));
        let seals = PersonalGraphSeals(state.clone());
        assert!(seals.current_reader(&root));
        assert!(!seals.current_reader(&[18; 32]));
        state.write().unwrap().current_roots.clear();
        assert!(
            !seals.current_reader(&root),
            "old readable history does not authorize stage work"
        );
        state.write().unwrap().current_roots.insert(root);
        state.write().unwrap().can_write = false;
        assert!(!seals.current_reader(&root));
        state.write().unwrap().can_write = true;
        state.write().unwrap().keys = None;
        assert!(!seals.current_reader(&root));
    }
}

/// Live graph protection. Retained epochs have exactly the personal graph's
/// history semantics; retiring a reader cannot revoke bytes it already read.
#[derive(Clone)]
pub struct PersonalGraphSeals(pub(crate) Arc<RwLock<SealingState>>);

impl PersonalGraphSeals {
    /// Live private compute requires both this device's current key and the
    /// peer's current seat. Historical readability cannot open a new stage.
    pub fn current_reader(&self, root: &[u8; 32]) -> bool {
        let state = self.0.read().unwrap();
        state.can_write
            && state
                .keys
                .as_ref()
                .is_some_and(|keys| keys.current_epoch().is_some())
            && state.current_roots.contains(root)
    }
    /// Changes with installed epochs and group membership.
    pub fn revision(&self) -> u64 {
        self.0.read().unwrap().revision
    }

    /// Whether this device holds a current seat and a data epoch.
    pub fn can_write(&self) -> bool {
        self.0.read().unwrap().can_write
    }

    /// Current epochs require a current key-group seat. Old epochs remain
    /// admissible for attested roots known through the retained key agreement,
    /// so a newly keyed device can receive history written by a retired reader.
    /// Transport reachability alone confers no authority.
    pub fn admits(&self, root: &[u8; 32], sealed: &GroupCiphertext) -> bool {
        let state = self.0.read().unwrap();
        state.current_roots.contains(root)
            || (state.historical_roots.contains(root)
                && state.keys.as_ref().is_some_and(|keys| {
                    keys.contains(&sealed.epoch) && keys.current_epoch() != Some(sealed.epoch)
                }))
    }

    /// Seal under the current graph epoch, refusing an unkeyed or removed seat.
    pub fn seal(&self, clear: &[u8]) -> Result<GroupCiphertext, String> {
        let state = self.0.read().unwrap();
        if !state.can_write {
            return Err("this device is not a keyed reader of the personal graph".into());
        }
        state
            .keys
            .as_ref()
            .ok_or("personal graph has no data key")?
            .seal_random(clear)
            .map_err(|error| error.to_string())
    }

    /// Read a retained graph epoch, including history from before revocation.
    pub fn open(&self, sealed: &GroupCiphertext) -> Result<Vec<u8>, String> {
        self.0
            .read()
            .unwrap()
            .keys
            .as_ref()
            .ok_or("this device has no personal graph data key")?
            .open(sealed)
            .map_err(|error| error.to_string())
    }
}
