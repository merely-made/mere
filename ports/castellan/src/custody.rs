// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Secret custody (dramatis repo plan, DR-B; rulings D3, D8, D9 and D22).
//!
//! Moved here from personae: the identity vault and its profiles and slots
//! ([`vault`]), its passphrase and OS-sealed storages, the passphrase-wrapped
//! root, the startup unlock ladder and its persisted lock, Windows Hello
//! ([`unlock`]), the vault directory's bootstrap and roster, the sealed-record
//! identity provider, and, behind `agent`, the SSH agent, its approval broker,
//! signing, and SSH keys as vault slots. Moved from pandect, behind `wallet`:
//! the wallet's identity seed, delegated-device identity, remote-auth wrapping
//! keys and private-epoch bridge, and the grant flows that sign or wrap with
//! them ([`wallet`]).
//!
//! personae keeps the plain types these name ([`personae::vault`]), the
//! keypairs and derivation, `seal_bytes` and the sealed-record store, which
//! take a key the caller holds.

#[cfg(feature = "agent")]
pub mod agent;
pub mod bootstrap;
#[cfg(feature = "agent")]
pub mod broker;
pub mod passphrase_root;
pub mod passphrase_storage;
mod profile_wire;
pub mod roster;
pub mod sealed_profile_storage;
pub mod sealed_provider;
#[cfg(feature = "agent")]
pub mod ssh_face;
#[cfg(feature = "agent")]
pub mod ssh_krl;
#[cfg(feature = "agent")]
pub mod ssh_sign;
#[cfg(feature = "agent")]
pub mod ssh_slot;
pub mod startup_unlock;
pub mod unlock;
pub mod vault;
#[cfg(feature = "wallet")]
pub mod wallet;

#[cfg(feature = "agent")]
pub use broker::ApprovalBroker;
pub use passphrase_root::{
    PassphraseWrappedRoot, change_passphrase, load_passphrase_root, passphrase_root_exists,
    save_passphrase_root, unwrap_vault_root, wrap_vault_root,
};
pub use passphrase_storage::PassphraseEncryptedStorage;
pub use roster::{OpenedVault, open_shared};
pub use sealed_profile_storage::{AUTO_UNLOCK_ROOT_FILE, SealedProfileStorage};
pub use sealed_provider::SealedIdentityProvider;
pub use startup_unlock::{
    PERSISTED_LOCK_FILE, auto_unlock_backend_available, clear_persisted_lock,
    load_existing_auto_unlock_root, load_existing_auto_unlock_root_after_presence,
    load_or_create_auto_unlock_root, lock_persisted, persist_lock,
};
pub use unlock::{OsPresence, UnlockMethod, UnlockMethods};
pub use vault::{
    IdentitySlot, IdentityStorage, IdentityVault, InMemoryStorage, Profile, SecretBytes, SlotMap,
};
