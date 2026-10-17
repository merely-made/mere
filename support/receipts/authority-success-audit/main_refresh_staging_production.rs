// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Durable staging: hold a verified update on disk until it is applied.
//!
//! T2 of the auto-update plan. Without this, "downloaded, ready to restart"
//! is a claim about bytes held in memory: close the app and the download is
//! gone, and the next launch starts over. Staging writes the verified
//! artifact into an app-owned directory with a small record beside it, so
//! the offer survives a restart — which is the whole point of offering a
//! restart rather than forcing one.
//!
//! ## Re-verification
//!
//! A staged file sits on disk, possibly for days, somewhere another process
//! can reach. [`StagedUpdate::take_verified`] re-hashes it against the
//! digest recorded at download time and refuses on mismatch, so time-of-
//! check/time-of-use is not a hole. Staging therefore requires a manifest
//! that carried a `blake3`; without one there is nothing to re-check
//! against and [`Update::stage`] says so rather than staging unverifiable
//! bytes.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::install::Update;
use crate::release::UpdateFormat;

/// File name of the record describing what is staged.
const RECORD_NAME: &str = "staged.json";
/// File name of the staged artifact itself.
const ARTIFACT_NAME: &str = "staged-artifact";

#[derive(Debug, Serialize, Deserialize)]
struct StagedRecord {
    version: String,
    format: UpdateFormat,
    blake3: String,
    /// Where the running app lives, captured at download time: the app that
    /// applies this may be a fresh process that has not checked a feed.
    extract_path: PathBuf,
}

/// An update sitting on disk, waiting to be applied.
#[derive(Debug)]
pub struct StagedUpdate {
    dir: PathBuf,
    record: StagedRecord,
}

impl StagedUpdate {
    /// The version staged.
    pub fn version(&self) -> &str {
        &self.record.version
    }

    /// The format staged.
    pub fn format(&self) -> UpdateFormat {
        self.record.format
    }

    /// Load whatever is staged in `dir`, if anything.
    ///
    /// A partial or unreadable staging directory reports `Ok(None)` rather
    /// than an error: a half-written stage is not a failure of the app, and
    /// the next check simply offers the update again.
    pub fn load(dir: impl AsRef<Path>) -> Result<Option<Self>> {
        let dir = dir.as_ref().to_path_buf();
        let record_path = dir.join(RECORD_NAME);
        if !record_path.exists() || !dir.join(ARTIFACT_NAME).exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(&record_path)?;
        match serde_json::from_slice::<StagedRecord>(&bytes) {
            Ok(record) => Ok(Some(Self { dir, record })),
            Err(err) => {
                log::warn!("ignoring unreadable staging record {record_path:?}: {err}");
                Ok(None)
            },
        }
    }

    /// Read the staged artifact back, re-checking its digest.
    ///
    /// The bytes were verified (BLAKE3 *and* signature) before staging;
    /// this guards the gap between then and now.
    pub fn take_verified(&self) -> Result<Vec<u8>> {
        let bytes = std::fs::read(self.dir.join(ARTIFACT_NAME))?;
        let actual = blake3::hash(&bytes).to_hex().to_string();
        if !actual.eq_ignore_ascii_case(&self.record.blake3) {
            return Err(Error::Blake3Mismatch {
                expected: self.record.blake3.clone(),
                actual,
            });
        }
        Ok(bytes)
    }

    /// Where the app being replaced lives.
    pub fn extract_path(&self) -> &Path {
        &self.record.extract_path
    }

    /// Delete the staged update.
    ///
    /// Call after a successful apply, or when the user declines: a stale
    /// stage would otherwise be offered forever.
    pub fn discard(&self) -> Result<()> {
        let _ = std::fs::remove_file(self.dir.join(ARTIFACT_NAME));
        let _ = std::fs::remove_file(self.dir.join(RECORD_NAME));
        Ok(())
    }
}

impl Update {
    /// Write verified bytes into `dir` so the offer survives a restart.
    ///
    /// Requires the manifest to have carried a `blake3` digest; there is
    /// nothing to re-verify against otherwise, and staging bytes that
    /// cannot be re-checked would be worse than not staging at all.
    pub fn stage(&self, dir: impl AsRef<Path>, bytes: &[u8]) -> Result<StagedUpdate> {
        let digest = self.blake3.clone().ok_or_else(|| {
            Error::Network(
                "cannot stage an update whose manifest carries no blake3 digest".to_string(),
            )
        })?;
        // Trust nothing that was not checked here: the caller may have got
        // these bytes from anywhere.
        let actual = blake3::hash(bytes).to_hex().to_string();
        if !actual.eq_ignore_ascii_case(&digest) {
            return Err(Error::Blake3Mismatch {
                expected: digest,
                actual,
            });
        }

        let dir = dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir)?;

        // Artifact first, then the record: the record is what `load` keys
        // on, so it must never point at a file that is not fully written.
        let artifact = dir.join(ARTIFACT_NAME);
        let tmp = dir.join("staged-artifact.partial");
        std::fs::write(&tmp, bytes)?;
        std::fs::rename(&tmp, &artifact)?;

        let record = StagedRecord {
            version: self.version.clone(),
            format: self.format,
            blake3: digest,
            extract_path: self.extract_path.clone(),
        };
        std::fs::write(dir.join(RECORD_NAME), serde_json::to_vec_pretty(&record)?)?;

        Ok(StagedUpdate { dir, record })
    }

    /// Install from a staged update, re-verifying it first.
    pub fn install_staged(&self, staged: &StagedUpdate) -> Result<()> {
        let bytes = staged.take_verified()?;
        self.install(bytes)
    }
}

