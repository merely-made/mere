// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The vault's roster as a persona picker shows it: plain, secret-free data.
//!
//! Moved from castellan's custody in DR-C (dramatis repo plan, D14), so an
//! application reads the roster from djinn and draws it without naming the
//! crate that opened the vault.

use personae::ProfileId;
use serde::{Deserialize, Serialize};

/// One persona in the vault, as a picker needs to show it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RosterEntry {
    /// Stable id, and what the custodian remembers as the choice.
    pub id: ProfileId,
    /// The name to show.
    pub display_name: String,
    /// How many protocol slots this persona carries. Shown because it is the
    /// one honest signal of which persona is the used one when the display
    /// names are not telling.
    pub slot_count: usize,
    /// Whether this is the persona currently in use.
    pub chosen: bool,
}

/// Everything a persona picker needs from the vault.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Roster {
    /// The vault's personas, sorted by id so the list does not reorder itself
    /// between runs.
    pub entries: Vec<RosterEntry>,
    /// The persona in use, whether or not it exists yet: a fresh vault resolves
    /// to a name that will be minted on first open.
    pub chosen: ProfileId,
    /// What protects the vault, as the custodian reports it.
    /// Shown, never guessed.
    pub description: String,
}

impl Roster {
    /// The entry for the persona in use, absent on a vault that has none yet.
    pub fn chosen_entry(&self) -> Option<&RosterEntry> {
        self.entries.iter().find(|entry| entry.chosen)
    }

    /// Whether the vault holds no personas at all, which is a first run.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
