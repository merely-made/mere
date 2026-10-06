// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! How a vault lock reaches the holders of vault-derived keys (vault lock
//! plan, rulings 1, 13 and 31).
//!
//! A holder registers with the resident host. Its hooks run synchronously:
//! the host's lock returns only after every holder has dropped its keys, and
//! an unlock re-derives them from the vault before anyone is told. Observers
//! that only need to know (a UI, the status route) watch the host's lock
//! state instead.

use personae::{IdentityError, IdentityProvider};

/// Something that keeps keys derived from the vault.
pub trait VaultLockHolder: Send + Sync {
    /// What the log calls it.
    fn name(&self) -> &str;

    /// Drop every vault-derived key now. Infallible: a lock never waits on
    /// a holder's opinion.
    fn lock(&self);

    /// Re-derive the keys from the vault just unlocked. An error leaves the
    /// holder locked, and the host relocks everything.
    fn unlock(&self, vault: &dyn IdentityProvider) -> Result<(), IdentityError>;
}
