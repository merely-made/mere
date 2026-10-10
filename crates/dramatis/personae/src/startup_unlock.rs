// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The startup unlock policy: a plain device setting.
//!
//! The ladder that acts on it (the `AutoOs` root loaders and the persisted
//! lock) is custody and lives in castellan (dramatis repo plan, DR-B); the
//! mode stays here because device settings store it.

use serde::{Deserialize, Serialize};

/// Startup unlock policy for the identity vault.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StartupUnlockMode {
    /// Use OS-protected local storage to auto-unlock the local vault root.
    #[default]
    AutoOs,
    /// Require an explicit passphrase prompt during startup.
    Prompt,
    /// Start locked and bring sync/private lanes up after an explicit unlock.
    Locked,
}
