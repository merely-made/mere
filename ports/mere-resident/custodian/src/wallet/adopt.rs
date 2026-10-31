// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Moving an existing wallet and credential catalog out of Djinn's data root
//! into the custodian's. The files keep their relative paths and their bytes,
//! so every sealed record, derivation and public key reopens unchanged; only
//! the directory they sit in changes.

use std::path::{Path, PathBuf};
use std::{fs, io};

use pandect::engine_profile_store::PERSONAS_DIR;
use pandect::wallet_store::{IDENTITY_DIR, PERSONA_EPOCH_BRIDGE_FILENAME, PERSONA_WALLET_FILENAME};

use crate::host::CATALOG_DIR;

/// What an adoption did.
#[derive(Debug, Default)]
pub struct Adopted {
    /// The relative paths now held in the custodian's data root.
    pub files: Vec<PathBuf>,
    /// Copies that could not be removed from the source, with why. Each is
    /// still readable where it was.
    pub left: Vec<(PathBuf, String)>,
}

/// Every file of the wallet and the credential catalog under `root`, as
/// paths relative to it, sorted:
/// - all of `identity/`: the manifests, the seed, the device secret, the
///   wrapping keys, and the grants with their epoch records;
/// - each persona's `wallet.json` and `private-epoch-bridge.json`, and
///   nothing else in `personas/<uuid>/`, which also holds Djinn's own data;
/// - all of `castellan/`, the credential catalog.
///
/// A symlink or other irregular entry is refused: it could point out of the
/// root.
pub fn wallet_files(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for dir in [IDENTITY_DIR, CATALOG_DIR] {
        collect(root, Path::new(dir), &mut files)?;
    }
    let personas = Path::new(PERSONAS_DIR);
    let Some(entries) = read_dir_if_present(&root.join(personas))? else {
        files.sort();
        return Ok(files);
    };
    for entry in entries {
        let entry = entry?;
        if !entry.file_name().to_string_lossy().parse::<uuid::Uuid>().is_ok_and(|_| entry.file_type().is_ok_and(|kind| kind.is_dir())) {
            continue;
        }
        for name in [PERSONA_WALLET_FILENAME, PERSONA_EPOCH_BRIDGE_FILENAME] {
            let relative = personas.join(entry.file_name()).join(name);
            match fs::symlink_metadata(root.join(&relative)) {
                Ok(meta) if meta.is_file() => files.push(relative),
                Ok(_) => return Err(irregular(&relative)),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {},
                Err(error) => return Err(error),
            }
        }
    }
    files.sort();
    Ok(files)
}

fn irregular(relative: &Path) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, format!("{} is not a regular file", relative.display()))
}

fn read_dir_if_present(dir: &Path) -> io::Result<Option<fs::ReadDir>> {
    match fs::read_dir(dir) {
        Ok(entries) => Ok(Some(entries)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn collect(root: &Path, relative: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    let Some(entries) = read_dir_if_present(&root.join(relative))? else {
        return Ok(());
    };
    for entry in entries {
        let entry = entry?;
        let child = relative.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_dir() {
            collect(root, &child, files)?;
        } else if kind.is_file() {
            files.push(child);
        } else {
            return Err(irregular(&child));
        }
    }
    Ok(())
}

/// Carry the wallet and catalog under `source` into `dest`, prove the copy
/// opens to the same wallet, then remove the source copies.
pub fn adopt_files(source: &Path, dest: &Path) -> Result<Adopted, String> {
    let files = carry_files(source, dest)?;
    let left = retire_files(source, dest, &files);
    Ok(Adopted { files, left })
}

/// The carrying half of [`adopt_files`]: copy, then prove the copy.
///
/// Refused, with nothing changed, when `source` holds neither wallet nor
/// catalog, when `dest` already holds either, or when the two overlap. If
/// the copy differs from the source in any byte, or opens to another seed,
/// persona index, roster or manifest, it is removed again. The source is
/// only read.
pub fn carry_files(source: &Path, dest: &Path) -> Result<Vec<PathBuf>, String> {
    let described = |error: io::Error| error.to_string();
    let files = wallet_files(source).map_err(described)?;
    if files.is_empty() {
        return Err(format!("no wallet or credential catalog under {}", source.display()));
    }
    if !wallet_files(dest).map_err(described)?.is_empty() {
        return Err("the custodian's data root already holds a wallet or credential catalog".into());
    }
    let (from, to) = (resolved(source).map_err(described)?, resolved(dest).map_err(described)?);
    if from.starts_with(&to) || to.starts_with(&from) {
        return Err("the source and the custodian's data root overlap".into());
    }
    fs::create_dir_all(dest).map_err(described)?;
    let mut written = Vec::new();
    let carried = copy_all(&files, source, dest, &mut written).and_then(|()| same_wallet(source, dest));
    if let Err(reason) = carried {
        for path in &written {
            let _ = fs::remove_file(path);
        }
        prune(dest, &files);
        return Err(format!("the copy was not made: {reason}"));
    }
    Ok(files)
}

/// The removing half of [`adopt_files`]: delete each carried file from
/// `source`, but only while it still equals the copy in `dest`. Returns the
/// files that stayed, with why; each is still readable where it was.
pub fn retire_files(source: &Path, dest: &Path, files: &[PathBuf]) -> Vec<(PathBuf, String)> {
    let mut left = Vec::new();
    for relative in files {
        let removed = match (fs::read(source.join(relative)), fs::read(dest.join(relative))) {
            (Ok(kept), Ok(copy)) if kept == copy => fs::remove_file(source.join(relative)).map_err(|e| e.to_string()),
            (Ok(_), _) => Err("it changed after it was carried".to_string()),
            (Err(error), _) => Err(error.to_string()),
        };
        if let Err(reason) = removed {
            left.push((relative.clone(), reason));
        }
    }
    prune(source, files);
    left
}

/// `path` with its longest existing ancestor resolved, so an unmade
/// directory still compares against the ones that exist.
fn resolved(path: &Path) -> io::Result<PathBuf> {
    let mut tail = Vec::new();
    let mut probe = path;
    while !probe.exists() {
        match (probe.file_name(), probe.parent()) {
            (Some(name), Some(parent)) => {
                tail.push(name);
                probe = parent;
            },
            _ => break,
        }
    }
    let mut base = fs::canonicalize(if probe.as_os_str().is_empty() { Path::new(".") } else { probe })?;
    base.extend(tail.iter().rev());
    Ok(base)
}

fn copy_all(files: &[PathBuf], source: &Path, dest: &Path, written: &mut Vec<PathBuf>) -> Result<(), String> {
    for relative in files {
        let bytes = fs::read(source.join(relative)).map_err(|e| format!("read {}: {e}", relative.display()))?;
        let target = dest.join(relative);
        if let Some(parent) = target.parent() {
            create_private_dir(parent).map_err(|e| e.to_string())?;
        }
        let staged = target.with_file_name(format!("{}.adopting", relative.file_name().unwrap_or_default().to_string_lossy()));
        write_private(&staged, &bytes).map_err(|e| format!("write {}: {e}", relative.display()))?;
        written.push(staged.clone());
        fs::rename(&staged, &target).map_err(|e| e.to_string())?;
        written.push(target.clone());
        if fs::read(&target).map_err(|e| e.to_string())? != bytes {
            return Err(format!("{} differs after the copy", relative.display()));
        }
    }
    Ok(())
}

/// The wallet the two roots hold is the same one: the seed, the persona
/// index and each persona's manifest, the identity manifest and the roster.
fn same_wallet(source: &Path, dest: &Path) -> Result<(), String> {
    use castellan::custody::wallet::load_identity_seed_read_only;
    use pandect::wallet_store::{list_personas, load_device_roster, load_identity_wallet, load_persona_wallet};
    let described = |error: io::Error| error.to_string();
    let (kept, copy) = (list_personas(source).map_err(described)?, list_personas(dest).map_err(described)?);
    if kept != copy {
        return Err("the persona index differs".into());
    }
    if load_identity_seed_read_only(source).map_err(described)? != load_identity_seed_read_only(dest).map_err(described)? {
        return Err("the identity seed differs".into());
    }
    if json(&load_identity_wallet(source).map_err(described)?) != json(&load_identity_wallet(dest).map_err(described)?) {
        return Err("the identity manifest differs".into());
    }
    if json(&load_device_roster(source).map_err(described)?) != json(&load_device_roster(dest).map_err(described)?) {
        return Err("the device roster differs".into());
    }
    for persona in kept {
        if json(&load_persona_wallet(source, persona).map_err(described)?) != json(&load_persona_wallet(dest, persona).map_err(described)?) {
            return Err(format!("persona {} manifest differs", persona.as_uuid()));
        }
    }
    Ok(())
}

fn json<T: serde::Serialize>(record: &Option<T>) -> serde_json::Value {
    serde_json::to_value(record).unwrap_or(serde_json::Value::Null)
}

/// Remove directories the carried `files` leave empty, innermost first.
/// A directory that still holds anything else stays.
fn prune(root: &Path, files: &[PathBuf]) {
    let mut dirs: Vec<PathBuf> = files.iter().flat_map(|file| file.ancestors().skip(1).map(Path::to_path_buf)).filter(|dir| !dir.as_os_str().is_empty()).collect();
    dirs.sort_by_key(|dir| std::cmp::Reverse(dir.components().count()));
    dirs.dedup();
    for dir in dirs {
        let _ = fs::remove_dir(root.join(dir));
    }
}

#[cfg(unix)]
fn create_private_dir(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    fs::DirBuilder::new().recursive(true).mode(0o700).create(path)
}

#[cfg(not(unix))]
fn create_private_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}

fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    use std::io::Write;
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}
