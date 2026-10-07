// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The platform layer: read-only process and pipe views, and the job that
//! takes spawned processes down with the harness.

use std::path::PathBuf;

#[cfg(not(windows))]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
pub use unix::*;
#[cfg(windows)]
pub use windows::*;

/// One process as the platform lists it.
#[derive(Clone, Debug)]
pub struct ProcessEntry {
    pub pid: u32,
    pub parent: u32,
    pub name: String,
    pub path: Option<PathBuf>,
    /// Creation time in the platform's own units (FILETIME ticks on Windows,
    /// clock ticks since boot on Linux): compared, never converted.
    pub started: Option<u64>,
}

/// Whether the process `pid` that started at `started` is still running. A
/// reused id with another start time is not it.
pub fn alive(pid: u32, started_at: u64) -> bool {
    started(pid) == Some(started_at)
}
