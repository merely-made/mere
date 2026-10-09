// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The identity surface's secret-free read model.
//!
//! Views that render *about* secrets and never contain them. They may name
//! public keys, public carrier facts, vault posture and grant summaries. No
//! field can carry a seed, private key, vault root, cleartext signing
//! payload or wrapped epoch material. Moved from castellan (dramatis repo
//! plan, ruling D24), which re-exports them and keeps the loader that fills
//! [`CarryView`] from pandect.

use personae::signing::{PendingSigningRequest, SigningRecord};
use serde::{Deserialize, Serialize};

/// Storage protection selected for the resident vault.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VaultProtectionView {
    OsProtected,
    Passphrase,
    Ephemeral,
}

/// Whether the resident vault may currently perform private operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VaultLockView {
    Locked,
    Unlocked,
}

/// Whether Graphshell has taken ownership of an SSH agent endpoint.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentListenerView {
    /// H4 must retain the standalone agent while the replacement is unproved.
    StandaloneRetained,
    /// A nonstandard receipt endpoint is active, without changing user config.
    ReceiptEndpoint { endpoint: String },
    /// The standard endpoint is active after cutover proof.
    StandardEndpoint { endpoint: String },
}

/// Vault state safe to disclose to Graphshell clients.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultView {
    pub protection: VaultProtectionView,
    pub lock: VaultLockView,
    pub agent: AgentListenerView,
}

/// One public profile summary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileView {
    pub id: String,
    pub display_name: String,
    pub selected: bool,
    pub slot_count: usize,
    pub master_public_fingerprint: String,
}

/// Public half and recovery posture of one SSH vault slot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SshKeyView {
    pub profile: String,
    pub fingerprint: String,
    pub comment: String,
    pub public_openssh: String,
    pub lineage: String,
    pub device_loss_note: String,
    pub unlock_policy: String,
}

/// Public device roster entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceView {
    pub device_id: String,
    pub label: String,
    pub mode: String,
    pub exposure: String,
    pub public_key_fingerprint: String,
    pub revoked: bool,
    pub grant_ref: Option<String>,
}

/// Secret-free summary of one signed device grant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceGrantView {
    pub device_id: String,
    pub grant_ref: Option<String>,
    pub signature_valid: Option<bool>,
    pub issued_at_ms: u64,
    pub expires_at_ms: Option<u64>,
    pub personas: Vec<String>,
    pub scopes: Vec<String>,
    pub attenuations: Vec<String>,
    pub wrapped_epoch_count: usize,
}

/// Carry state loaded from pandect without importing secret bridges.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CarryView {
    pub recovery_policy: Option<String>,
    pub personas: Vec<String>,
    pub devices: Vec<DeviceView>,
    pub grants: Vec<DeviceGrantView>,
    pub unavailable: Vec<String>,
}

/// Complete identity surface disclosed by the native host.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentitySurfaceSnapshot {
    pub vault: VaultView,
    pub profiles: Vec<ProfileView>,
    pub ssh_keys: Vec<SshKeyView>,
    pub carry: CarryView,
    pub pending_signing: Vec<PendingSigningRequest>,
    pub signing_history: Vec<SigningRecord>,
}

impl IdentitySurfaceSnapshot {
    /// Serialize only the public read model.
    pub fn to_public_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}
