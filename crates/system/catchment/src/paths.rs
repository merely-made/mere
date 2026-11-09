// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Where a reservoir and its meres live, and the store lifetimes: opening the
//! index, a mere's session store and its archive.
//!
//! Layout: `<shared root>/personas/<persona>/reservoir/reservoir.redb`, with
//! each mere under `meres/<mere id>/`. The redb file's exclusive lock is the
//! one-owner rule: a second process that opens the same reservoir is refused
//! rather than left waiting.

use std::path::{Path, PathBuf};

use identity::PersonaId;
use sporran::PERSONAS_DIR;

#[cfg(not(target_arch = "wasm32"))]
use crate::catalog::ReservoirError;
use crate::catalog::MereId;

/// The reservoir's directory under a persona's shared-root directory.
pub const RESERVOIR_DIR: &str = "reservoir";
/// The reservoir index database inside [`RESERVOIR_DIR`].
pub const RESERVOIR_DB_FILENAME: &str = "reservoir.redb";
/// The directory under a reservoir that holds each mere's own store.
pub const MERES_DIR: &str = "meres";

/// Where a persona's reservoir lives under the shared root.
pub fn reservoir_dir(shared_root: &Path, persona: PersonaId) -> PathBuf {
    shared_root
        .join(PERSONAS_DIR)
        .join(persona.as_uuid().to_string())
        .join(RESERVOIR_DIR)
}

/// Where mere `mere` keeps its sessions: `<reservoir>/meres/<mere id>/`.
pub fn mere_dir(shared_root: &Path, persona: PersonaId, mere: MereId) -> PathBuf {
    reservoir_dir(shared_root, persona)
        .join(MERES_DIR)
        .join(mere.0.to_string())
}

/// Open the durable backend for a persona's reservoir, creating it if absent.
///
/// This takes the reservoir's exclusive lock. Only the device resident, or an
/// application embedding it, should call this; a second caller is refused with
/// [`ReservoirError::Open`].
#[cfg(not(target_arch = "wasm32"))]
pub fn open_reservoir_backend(
    shared_root: &Path,
    persona: PersonaId,
) -> Result<muniment::RedbBackend, ReservoirError> {
    let dir = reservoir_dir(shared_root, persona);
    std::fs::create_dir_all(&dir).map_err(|error| ReservoirError::Open {
        path: dir.clone(),
        reason: error.to_string(),
    })?;
    open_redb(dir.join(RESERVOIR_DB_FILENAME))
}

/// Open mere `mere`'s store: a muniment directory backend whose keys are the
/// domain's own files (a graph domain's sessions, for one). Only the
/// reservoir's owner, who already holds [`open_reservoir_backend`]'s lock,
/// should call this; the store takes its own lock too, so a second opener is
/// refused at once.
#[cfg(not(target_arch = "wasm32"))]
pub fn open_mere_backend(
    shared_root: &Path,
    persona: PersonaId,
    mere: MereId,
) -> Result<muniment::DirectoryBackend, ReservoirError> {
    let dir = mere_dir(shared_root, persona, mere);
    muniment::DirectoryBackend::open(&dir).map_err(|error| ReservoirError::Open {
        path: dir,
        reason: error.to_string(),
    })
}

/// Eidetic's existing colon-bearing keys cannot be filenames in the portable
/// session directory backend. The resident therefore owns one redb archive
/// beside each mere's session files, through Eidetic's normal Store seam.
#[cfg(not(target_arch = "wasm32"))]
pub fn open_mere_archive_backend(
    shared_root: &Path,
    persona: PersonaId,
    mere: MereId,
) -> Result<muniment::RedbBackend, ReservoirError> {
    open_mere_redb(shared_root, persona, mere, "archive.redb")
}

/// Open a redb file named `file` inside mere `mere`'s directory, creating the
/// directory if absent.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn open_mere_redb(
    shared_root: &Path,
    persona: PersonaId,
    mere: MereId,
    file: &str,
) -> Result<muniment::RedbBackend, ReservoirError> {
    let path = mere_dir(shared_root, persona, mere).join(file);
    std::fs::create_dir_all(path.parent().expect("mere directory")).map_err(|error| {
        ReservoirError::Open {
            path: path.clone(),
            reason: error.to_string(),
        }
    })?;
    open_redb(path)
}

#[cfg(not(target_arch = "wasm32"))]
fn open_redb(path: PathBuf) -> Result<muniment::RedbBackend, ReservoirError> {
    muniment::RedbBackend::open(&path).map_err(|error| ReservoirError::Open {
        path,
        reason: error.to_string(),
    })
}
