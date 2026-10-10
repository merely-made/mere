// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Explicit persistence for authored user themes. This library does not store
//! the host's active theme choice or apply a theme to any running host.

use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::Theme;
use crate::portable::WriteMode;
use crate::theme::registry::{Harmony, ThemeRegistry, ThemeSource};

const LIBRARY_VERSION: u32 = 1;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LibraryDocument {
    version: u32,
    themes: Vec<Theme>,
}

/// A validated snapshot of an authored theme library on disk.
///
/// Saves replace the file atomically after checking that its bytes still
/// match the last successful load/save. A separate `.lock` file coordinates
/// cooperating writers across that replacement; busy writers return an error.
/// External editors that do not use the lock are checked again immediately
/// before replacement, but must cooperate to exclude all concurrent races.
#[derive(Debug)]
pub struct ThemeLibraryStore {
    path: PathBuf,
    themes: Vec<Theme>,
    snapshot: Option<Vec<u8>>,
}

impl ThemeLibraryStore {
    /// Read a library, treating a missing file as an empty library. Malformed,
    /// unsupported or inadmissible libraries return `InvalidData`.
    /// Reading does not create the file or its containing directory.
    pub fn load(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        // Anchor a relative path now so later cwd changes cannot redirect saves.
        let path = if path.is_absolute() {
            path
        } else {
            std::env::current_dir()?.join(path)
        };
        if path.file_name().is_none() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "a theme library path must name a file",
            ));
        }
        let snapshot = read_snapshot(&path)?;
        let themes = match &snapshot {
            Some(bytes) => {
                let document: LibraryDocument = serde_json::from_slice(bytes).map_err(|error| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("invalid theme library: {error}"),
                    )
                })?;
                if document.version != LIBRARY_VERSION {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("unsupported theme library version {}", document.version),
                    ));
                }
                validate(&document.themes)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                document.themes
            },
            None => Vec::new(),
        };
        Ok(Self {
            path,
            themes,
            snapshot,
        })
    }

    /// Authored definitions in their persisted order.
    pub fn themes(&self) -> &[Theme] {
        &self.themes
    }

    /// The absolute file path captured when this store was loaded.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Validate and persist the complete authored library. Invalid definitions,
    /// changed files and failed writes leave this store's snapshot unchanged.
    /// A stale store must be reloaded before it can save again.
    pub fn save(&mut self, themes: &[Theme]) -> io::Result<()> {
        validate(themes).map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
        let document = LibraryDocument {
            version: LIBRARY_VERSION,
            themes: themes.to_vec(),
        };
        let mut bytes = serde_json::to_vec_pretty(&document).map_err(io::Error::other)?;
        bytes.push(b'\n');
        atomic_write(&self.path, &bytes, WriteMode::Replace, Some(&self.snapshot))?;
        self.themes = document.themes;
        self.snapshot = Some(bytes);
        Ok(())
    }
}

fn validate(themes: &[Theme]) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    for theme in themes {
        let id = theme.id.trim().to_ascii_lowercase();
        if !ids.insert(id.clone()) {
            return Err(format!("duplicate authored theme id {id:?}"));
        }
        validate_user_theme(theme)?;
    }
    Ok(())
}

/// Validation shared by the library, portable artifacts and draft constructors.
/// A portable built-in remains an inert definition until explicitly forked.
pub(crate) fn validate_definition(theme: &Theme) -> Result<(), String> {
    let id = theme.id.trim().to_ascii_lowercase();
    if id.is_empty() || theme.name.trim().is_empty() {
        return Err("themes must have a nonblank id and name".into());
    }
    if let Harmony::Locked {
        secondary_deg,
        tertiary_deg,
    } = theme.harmony
        && (!secondary_deg.is_finite() || !tertiary_deg.is_finite())
    {
        return Err(format!("theme {id:?} has nonfinite harmony offsets"));
    }
    ThemeRegistry::default()
        .add_user_theme(theme.clone())
        .map_err(|error| format!("invalid theme {id:?}: {error}"))
}

pub(crate) fn validate_user_theme(theme: &Theme) -> Result<(), String> {
    if theme.source != ThemeSource::User {
        return Err(format!(
            "theme {:?} is not an authored user theme",
            theme.id
        ));
    }
    if ThemeRegistry::default().theme_def(&theme.id).is_some() {
        return Err(format!(
            "authored theme id {:?} collides with a built-in",
            theme.id
        ));
    }
    validate_definition(theme)
}

/// One shared writer for libraries and exported artifacts. An optional
/// expected snapshot protects an already loaded library. Exports capture the
/// current bytes under the same lock, then check them before replacement.
pub(crate) fn atomic_write(
    path: &Path,
    bytes: &[u8],
    mode: WriteMode,
    expected: Option<&Option<Vec<u8>>>,
) -> io::Result<()> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    if path.file_name().is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "an artifact path must name a file",
        ));
    }
    let parent = path.parent().expect("absolute file path has a parent");
    fs::create_dir_all(parent)?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(suffixed_path(&path, ".lock"))?;
    lock.try_lock().map_err(|error| match error {
        TryLockError::WouldBlock => io::Error::new(
            io::ErrorKind::WouldBlock,
            "another writer is saving this artifact",
        ),
        TryLockError::Error(error) => error,
    })?;
    let snapshot = read_snapshot(&path)?;
    if mode == WriteMode::CreateNew && fs::symlink_metadata(&path).is_ok() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "the export path already exists; choose another path or explicitly replace it",
        ));
    }
    if let Some(expected) = expected
        && snapshot != *expected
    {
        return Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            "theme library changed on disk; reload it before saving",
        ));
    }
    let (temp_path, mut temp) = temporary_file(&path)?;
    let result = (|| {
        temp.write_all(bytes)?;
        if snapshot.is_some() {
            temp.set_permissions(fs::metadata(&path)?.permissions())?;
        }
        temp.sync_all()?;
        if read_snapshot(&path)? != snapshot {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "artifact changed on disk before replacement",
            ));
        }
        drop(temp);
        match mode {
            WriteMode::Replace => fs::rename(&temp_path, &path),
            // Publishing a hard link is atomic and refuses an existing path,
            // including one created after the initial existence check.
            WriteMode::CreateNew => fs::hard_link(&temp_path, &path),
        }
    })();
    // On successful CreateNew the complete file is now reachable at its final
    // path; cleanup errors cannot turn a published artifact into a failed save.
    let _ = fs::remove_file(&temp_path);
    result
}

fn read_snapshot(path: &Path) -> io::Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn suffixed_path(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path
        .file_name()
        .expect("library path names a file")
        .to_os_string();
    name.push(suffix);
    path.with_file_name(name)
}

fn temporary_file(path: &Path) -> io::Result<(PathBuf, File)> {
    for _ in 0..32 {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temp_path = suffixed_path(path, &format!(".{}.{}.tmp", std::process::id(), sequence));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(file) => return Ok((temp_path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not create a unique temporary library file",
    ))
}
