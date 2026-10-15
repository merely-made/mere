// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The vault the Secret Service follows (vault lock rulings 10, 67, 68).

/// The resident's vault as the Secret Service sees it. The host hands it in,
/// so castellan names no unlock UI of its own.
pub trait SecretServiceVault: Send + Sync + 'static {
    /// Whether the vault is locked now.
    fn is_locked(&self) -> bool;

    /// Lock changes: `true` while locked.
    fn watch(&self) -> tokio::sync::watch::Receiver<bool>;

    /// Engage the whole vault lock (ruling 68).
    fn lock(&self) -> Result<(), String>;

    /// The resident's own unlock prompt (ruling 67). Blocks until it ends:
    /// `Ok(true)` once unlocked, `Ok(false)` when cancelled.
    fn prompt_unlock(&self) -> Result<bool, String>;
}
