// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The vault's plain types: profile ids, protocol keys, recovery lineage,
//! unlock tiers, and the secret-free profile summaries a locked vault still
//! answers.
//!
//! The vault itself (`IdentityVault`, its profiles, slots and storages) is
//! custody and lives in castellan, which only the resident links (dramatis
//! repo plan, DR-B, rulings D3, D9 and D22).

use serde::{Deserialize, Serialize};

use crate::Ed25519PublicKey;

/// Per-user vault directory (`%LOCALAPPDATA%\personae\vault`, or
/// `$XDG_DATA_HOME/personae/vault`).
///
/// A location, not custody: it moved here from castellan's bootstrap in DR-C
/// so a host that only needs the path (an owner-only socket's fallback
/// directory) does not name castellan.
pub fn default_vault_dir() -> std::path::PathBuf {
    use std::path::PathBuf;
    #[cfg(windows)]
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    #[cfg(not(windows))]
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("personae").join("vault")
}

/// Stable identifier for a profile within a vault.
///
/// Profiles are independent: per-profile master keys do not derive from
/// each other. A user with multiple profiles (work / personal / alt)
/// switches between them at the vault level; mods hold per-profile
/// sub-instances (Element-style).
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProfileId(pub String);

/// Compound key for a slot within a profile.
///
/// `mod_id` identifies the protocol mod owning the slot ("nostr",
/// "matrix", "irc", "cable"). `instance` distinguishes multiple slots of
/// the same kind — e.g., two Matrix accounts on different homeservers, or
/// per-cabal Cable keys.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ProtocolKey {
    /// The protocol mod's id.
    pub mod_id: String,
    /// Optional instance discriminator within the mod.
    pub instance: Option<String>,
}

impl ProtocolKey {
    /// Construct a new protocol key.
    pub fn new(mod_id: impl Into<String>, instance: Option<String>) -> Self {
        Self {
            mod_id: mod_id.into(),
            instance,
        }
    }
}

/// Recovery semantics for a slot. See plan §3.4.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CredentialLineage {
    /// Derived locally from master via deterministic salt.
    /// Recoverable from master alone (Cable cabal keys, Mere-native).
    LocallyDerived,
    /// Generated locally and registered with an external authority.
    /// Lost = re-register; recovery phrase does NOT regenerate it
    /// (Matrix device keys).
    LocallyGeneratedExternallyRegistered,
    /// Issued by external authority. Rotates / expires; not meaningfully
    /// backupable (access tokens, refresh JWTs).
    ExternallyIssued,
    /// Externally rooted (CA, identity provider) but locally held. CA
    /// owns revocation (X.509 client certs).
    ExternallyRootedLocallyHeld,
}

impl CredentialLineage {
    /// What losing this device means for a slot of this lineage.
    ///
    /// Plan §3.4 requires surfacing this per slot: a single "the recovery
    /// phrase brings everything back" story is wrong and would mislead
    /// users into thinking externally-registered credentials survive a
    /// device loss. Any vault UI (CLI, pane) shows this verbatim.
    pub fn device_loss_note(&self) -> &'static str {
        match self {
            Self::LocallyDerived => {
                "Recoverable: the master key re-derives this slot deterministically."
            },
            Self::LocallyGeneratedExternallyRegistered => {
                "Not recoverable from the vault alone: register a replacement with the service. \
                 A recovery phrase unlocks the vault; it does not regenerate this key."
            },
            Self::ExternallyIssued => {
                "Not backupable: credentials of this kind rotate and expire by design. \
                 Re-authenticate with the issuer."
            },
            Self::ExternallyRootedLocallyHeld => {
                "Upstream's call: the issuing authority revokes and reissues."
            },
        }
    }
}

/// When the approval broker asks before a slot is exercised, declared at
/// registration. Consent, not custody (vault lock plan, ruling 12): every
/// tier is decrypted while the vault is unlocked.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum UnlockTier {
    /// Exercised without asking while the vault is unlocked.
    Session,
    /// Asked once; the approval is reused until the idle window passes.
    ShortTtl {
        /// Seconds an approval lasts without use.
        idle_seconds: u32,
    },
    /// Asked visibly on every use; refused where no broker can ask.
    PerUse,
}

/// Lightweight summary of a profile, for listing without unlocking the
/// full keypair material.
#[derive(Clone, Debug)]
pub struct ProfileSummary {
    /// The profile id.
    pub id: ProfileId,
    /// Display name.
    pub display_name: String,
    /// Number of slots in the profile (no slot contents).
    pub slot_count: usize,
}

/// What stays visible while the vault is locked: no secret material
/// (vault lock ruling 11).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicProfile {
    /// The profile id.
    pub id: ProfileId,
    /// Display name.
    pub display_name: String,
    /// The master public key.
    pub master_public_key: Ed25519PublicKey,
    /// Each slot's description, without its payload.
    pub slots: Vec<SlotSummary>,
}

/// One slot, described without its secret.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotSummary {
    /// Where the slot lives.
    pub key: ProtocolKey,
    /// The protocol kind.
    pub kind: String,
    /// Recovery lineage.
    pub lineage: CredentialLineage,
    /// Unlock tier (consent, per ruling 12).
    pub unlock_tier: UnlockTier,
}
