// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The public results of the identity surface's intents.
//!
//! Plain facts with no key material. Moved from castellan's authority
//! (dramatis repo plan, ruling D31), which re-exports them and keeps the
//! error it raises.

use serde::{Deserialize, Serialize};

/// Public result of a native SSH key mutation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SshKeyMutationReceipt {
    pub operation: SshKeyMutationKind,
    pub fingerprint: String,
    pub comment: String,
    pub public_openssh: String,
    pub unlock_policy: String,
    /// The fingerprint was already held. Since ruling 54 the held slot is
    /// left untouched, and comment and unlock policy describe it as held.
    pub replaced_existing: bool,
}

/// Mutation kind safe to retain in a local receipt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SshKeyMutationKind {
    Generated,
    Imported,
    Removed,
}

/// Result of applying one local identity control.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentityIntentOutcome {
    SigningDecision,
    SshKeyMutation(SshKeyMutationReceipt),
    DeviceRevocation(DeviceRevocationReceipt),
    ProfileSwitch(ProfileSwitchReceipt),
    ProfileCreated(ProfileCreatedReceipt),
    /// The vault and every holder locked.
    VaultLocked,
}

/// Public facts produced by minting a persona. No key material: the master
/// public key is reported as a fingerprint, the way the profile cards do.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileCreatedReceipt {
    pub id: String,
    pub display_name: String,
    pub master_public_fingerprint: String,
}

/// Public facts produced by a live profile switch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileSwitchReceipt {
    /// The persona the host now speaks as.
    pub profile: String,
    /// Whether the choice was written beside the vault for the rest of the
    /// family, or only applied to this host (an ephemeral vault has no
    /// beside). Shown, not guessed: "everyone follows" and "just here" are
    /// different promises.
    pub remembered: bool,
}

/// Public facts produced by a carry-authority revocation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceRevocationReceipt {
    pub device_id: String,
    pub already_revoked: bool,
    pub rotated_personas: Vec<String>,
    pub refreshed_devices: Vec<String>,
}
