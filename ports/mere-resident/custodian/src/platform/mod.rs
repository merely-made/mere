// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One launcher, one peer-identity check and one confinement recipe per OS
//! (§5.2). Everything else is shared. The service manager starts the
//! custodian with a fixed configuration path; nothing here reads a
//! caller-chosen executable, library path or environment. Confinement is
//! applied while the process is still single-threaded and before the first
//! byte of client input. In protected mode a missing mechanism stops the
//! launch; development mode reports what it could not apply.

use std::path::PathBuf;

#[cfg(windows)]
pub mod hello;
#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(windows)]
pub mod windows;

/// What the confined custodian may touch.
pub struct Confinement {
    /// The vault directory: read and write.
    pub state: PathBuf,
    /// The configuration file: read only.
    pub config: PathBuf,
    /// Refuse to run unless every mechanism applied.
    pub required: bool,
}

/// What was actually applied, for the startup log and status.
#[derive(Debug, Default)]
pub struct Applied {
    pub notes: Vec<String>,
}

/// Apply this OS's recipe.
pub fn confine(plan: &Confinement) -> Result<Applied, String> {
    #[cfg(target_os = "linux")]
    return linux::confine(plan);
    #[cfg(target_os = "macos")]
    return macos::confine(plan);
    #[cfg(windows)]
    return windows::confine(plan);
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    {
        if plan.required {
            return Err("no confinement recipe for this OS".into());
        }
        Ok(Applied {
            notes: vec!["no confinement recipe for this OS".into()],
        })
    }
}
