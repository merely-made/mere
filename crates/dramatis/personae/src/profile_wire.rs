// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shared plaintext wire shape for stored profiles.
//!
//! Both on-disk backends ([`crate::PassphraseEncryptedStorage`] and
//! [`crate::SealedProfileStorage`]) serialize a [`crate::vault::Profile`]
//! to this serde shape before encrypting. One definition so the two
//! at-rest formats cannot drift structurally; the encryption envelopes
//! around it differ per backend and are versioned there.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::vault::{CredentialLineage, IdentitySlot, ProtocolKey, SecretBytes, UnlockTier};

/// Plaintext inner shape — what a backend serializes then encrypts.
///
/// Zeroizes on drop (vault lock ruling 6): every load and save builds one.
#[derive(Debug, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub(crate) struct PlaintextProfile {
    #[zeroize(skip)]
    pub(crate) display_name: String,
    /// 32-byte master signing-key seed.
    pub(crate) master_seed: [u8; 32],
    /// Slots, encoded.
    pub(crate) slots: Vec<PlaintextSlot>,
}

/// Plaintext slot — same structural shape as [`IdentitySlot`] but with
/// `Vec<u8>` for the secret payload (since `SecretBytes` doesn't impl
/// serde).
#[derive(Debug, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub(crate) struct PlaintextSlot {
    #[zeroize(skip)]
    pub(crate) mod_id: String,
    #[zeroize(skip)]
    pub(crate) instance: Option<String>,
    #[zeroize(skip)]
    pub(crate) kind: String,
    #[serde(deserialize_with = "crate::zeroizing_json::bytes")]
    pub(crate) payload: Vec<u8>,
    /// Set iff Bootstrap-category slot. None for Direct.
    #[zeroize(skip)]
    pub(crate) state_dir: Option<PathBuf>,
    #[zeroize(skip)]
    pub(crate) is_bootstrap: bool,
    #[zeroize(skip)]
    pub(crate) lineage: CredentialLineage,
    #[zeroize(skip)]
    pub(crate) unlock_tier: UnlockTier,
}

pub(crate) fn slot_to_plaintext(key: &ProtocolKey, slot: &IdentitySlot) -> PlaintextSlot {
    match slot {
        IdentitySlot::Direct {
            kind,
            payload,
            lineage,
            unlock_tier,
        } => PlaintextSlot {
            mod_id: key.mod_id.clone(),
            instance: key.instance.clone(),
            kind: kind.clone(),
            payload: payload.as_slice().to_vec(),
            state_dir: None,
            is_bootstrap: false,
            lineage: *lineage,
            unlock_tier: *unlock_tier,
        },
        IdentitySlot::Bootstrap {
            kind,
            bootstrap,
            state_dir,
            lineage,
            unlock_tier,
        } => PlaintextSlot {
            mod_id: key.mod_id.clone(),
            instance: key.instance.clone(),
            kind: kind.clone(),
            payload: bootstrap.as_slice().to_vec(),
            state_dir: Some(state_dir.clone()),
            is_bootstrap: true,
            lineage: *lineage,
            unlock_tier: *unlock_tier,
        },
    }
}

/// Moves the payload out rather than copying it, so no uncleared copy is
/// freed; what is left in `p` zeroizes when it drops.
pub(crate) fn plaintext_to_slot(p: &mut PlaintextSlot) -> (ProtocolKey, IdentitySlot) {
    let key = ProtocolKey {
        mod_id: p.mod_id.clone(),
        instance: p.instance.clone(),
    };
    let payload = SecretBytes::new(std::mem::take(&mut p.payload));
    let slot = if p.is_bootstrap {
        IdentitySlot::Bootstrap {
            kind: p.kind.clone(),
            bootstrap: payload,
            state_dir: p.state_dir.clone().unwrap_or_else(|| PathBuf::from(".")),
            lineage: p.lineage,
            unlock_tier: p.unlock_tier,
        }
    } else {
        IdentitySlot::Direct {
            kind: p.kind.clone(),
            payload,
            lineage: p.lineage,
            unlock_tier: p.unlock_tier,
        }
    };
    (key, slot)
}
