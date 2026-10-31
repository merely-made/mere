// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Custody-owned public metadata retained across lock and snapshot cloning.
//!
//! The shared personae transport vocabulary remains unchanged. Retaining its
//! inline `Option<String>` inside the vault allowed the unused None payload
//! to carry stale stack secrets into the heap, also through derived Clone.
//! This representation uses `Option<Box<String>>`: the absent value is a
//! fully defined null pointer, with no dormant string words to copy.
//! See https://doc.rust-lang.org/std/option/index.html#representation.

use super::{CredentialLineage, Ed25519PublicKey, ProfileId, UnlockTier};

/// Public profile facts kept by the custody owner, including while locked.
///
/// This is an in-memory retention type, not an on-disk or IPC format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetainedPublicProfile {
    /// Stable profile identity.
    pub id: ProfileId,
    /// Public display name.
    pub display_name: String,
    /// Public signing key.
    pub master_public_key: Ed25519PublicKey,
    /// Sorted descriptions with no private slot payloads.
    pub slots: Vec<RetainedSlotSummary>,
}

/// Public slot identity with an optional instance that has no inactive payload.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RetainedSlotKey {
    /// Protocol identifier.
    pub mod_id: String,
    /// None and Some(empty) remain distinct. The sized Box gives None a
    /// defined all-zero pointer representation, including when cloned.
    pub instance: Option<Box<String>>,
}

/// A slot's public facts retained without its secret bytes or bootstrap state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetainedSlotSummary {
    /// Protocol and instance identity.
    pub key: RetainedSlotKey,
    /// Protocol kind.
    pub kind: String,
    /// Recovery semantics.
    pub lineage: CredentialLineage,
    /// Consent policy.
    pub unlock_tier: UnlockTier,
}
