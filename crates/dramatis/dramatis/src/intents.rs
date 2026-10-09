// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The identity surface's intent names and their typed, secret-free payloads.
//!
//! Moved from castellan's projection (dramatis repo plan, ruling D24), which
//! re-exports them. The names keep their `castellan.*` values: they cross
//! the projection protocol to admitted browsers, and renaming is a wire
//! vocabulary change with its own blast radius. See the keeper founding
//! plan.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const SIGNING_APPROVE_ONCE_INTENT: &str = "castellan.signing.approve-once";
pub const SIGNING_APPROVE_IDLE_INTENT: &str = "castellan.signing.approve-until-idle";
pub const SIGNING_DENY_INTENT: &str = "castellan.signing.deny";
pub const SIGNING_DECISION_SCHEMA: &str = "castellan.signing-decision/v1";
pub const SSH_GENERATE_INTENT: &str = "castellan.ssh.generate";
pub const SSH_GENERATE_SCHEMA: &str = "castellan.ssh.generate/v1";
pub const SSH_IMPORT_NATIVE_INTENT: &str = "castellan.ssh.import-native";
pub const SSH_IMPORT_NATIVE_SCHEMA: &str = "castellan.ssh.import-native/v1";
pub const SSH_REMOVE_INTENT: &str = "castellan.ssh.remove";
pub const SSH_REMOVE_SCHEMA: &str = "castellan.ssh.remove/v1";
pub const DEVICE_REVOKE_INTENT: &str = "castellan.device.revoke";
pub const DEVICE_REVOKE_SCHEMA: &str = "castellan.device.revoke/v1";
pub const PROFILE_SWITCH_INTENT: &str = "castellan.profile.switch";
pub const PROFILE_SWITCH_SCHEMA: &str = "castellan.profile.switch/v1";
pub const PROFILE_CREATE_INTENT: &str = "castellan.profile.create";
pub const PROFILE_CREATE_SCHEMA: &str = "castellan.profile.create/v1";
/// Lock the vault; portable (vault lock ruling 10).
pub const VAULT_LOCK_INTENT: &str = "castellan.vault.lock";
/// [`LockVaultIntentV1`]'s schema.
pub const VAULT_LOCK_SCHEMA: &str = "castellan.vault.lock/v1";
/// Unlock the vault; native only, and refused as an intent (ruling 9).
pub const VAULT_UNLOCK_INTENT: &str = "castellan.vault.unlock";
/// The Unlock action's schema; it carries nothing.
pub const VAULT_UNLOCK_SCHEMA: &str = "castellan.vault.unlock/v1";

/// Lock the vault (vault lock ruling 10). Portable: any admitted surface may
/// lock, since locking only takes authority away. It carries nothing.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockVaultIntentV1 {}

/// Typed, secret-free signing decision payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SigningDecisionIntentV1 {
    pub request_id: Uuid,
}

/// User-selected unlock policy for a generated or natively imported key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SshUnlockPolicyIntentV1 {
    Session,
    ShortTtl { idle_seconds: u32 },
    PerUse,
}

/// Safe input for generating a key entirely inside the resident host.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenerateSshKeyIntentV1 {
    pub comment: String,
    pub unlock_policy: SshUnlockPolicyIntentV1,
}

/// Options accompanying a native file-picker handoff.
///
/// The private key is intentionally absent. The native picker passes the
/// parsed key object directly to `PersonaeHost::import_ssh_private`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportSshKeyNativeIntentV1 {
    pub unlock_policy: SshUnlockPolicyIntentV1,
}

/// Explicit removal of one public SSH fingerprint.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoveSshKeyIntentV1 {
    pub fingerprint: String,
    pub confirmed: bool,
}

/// Explicit revocation of one delegated device at the live carry authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevokeDeviceIntentV1 {
    pub device_id: Uuid,
    pub confirmed: bool,
}

/// Mint a new persona in the vault.
///
/// The last thing in the family nothing could do: every application's picker
/// has a create row, and each one told the user to go and run the
/// `personae-vault` CLI.
///
/// Minting is the port's act, per the port law: identity is a capability the
/// stack owns (`personae`), castellan is its port, and applications compose
/// it. This intent briefly lived in graphshell — the first application with an
/// identity surface — and moved home with the keeper founding.
///
/// Creating is deliberately not switching. A persona minted for another device
/// or another purpose is not one you are necessarily becoming, so the new
/// card arrives carrying the ordinary switch action and the choice stays the
/// user's.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateProfileIntentV1 {
    /// The id, which is also what the remembered choice and the settings
    /// filenames are keyed by. Constrained at the host; see
    /// `PersonaeHost::create_profile`.
    pub id: String,
    /// What to show. Free text, unlike the id.
    pub display_name: String,
}

/// Switch the resident host to another persona, live.
///
/// The one identity mutation that was missing: the projection could show which
/// persona the host speaks as and could not change it. No confirmation field —
/// switching is not destructive (the previous persona keeps everything and is
/// one switch away), and a wrong guess corrects itself the same way it was
/// made.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SwitchProfileIntentV1 {
    /// The profile id, as carried by the profile card this action rides on.
    pub profile: String,
}
