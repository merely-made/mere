// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Djinn as a client of the independently launched custodian (custody
//! isolation plan, §5.6 and H1).
//!
//! The custodian is found through an installed client file that the desktop
//! user cannot write: on Unix it and its directory must be root-owned and
//! not group- or world-writable, so a same-user process cannot point Djinn
//! at an impostor. The file names the endpoint and the OS identity the
//! custodian must present; the client checks that identity before its
//! hello. Every answer is typed. A lost or refusing custodian is reported
//! as such and never replaced by an in-process key: identity stays pending.
//!
//! [`crate::custody_host::remote::RemoteHost`] is the client built on it;
//! every vault use in Djinn goes through that host (H2).

use std::path::{Path, PathBuf};

use custody_wire::ServerExpectation;
use serde::Deserialize;

/// The installed client file.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstalledCustodian {
    /// A Unix socket path, or a Windows pipe name.
    pub endpoint: String,
    /// The custodian's OS identity.
    pub server: ServerExpectation,
    /// Where the custodian serves the SSH agent, for the resident's status.
    #[serde(default)]
    pub agent: Option<String>,
}

static FIXTURE: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// Where the installed client file lives for this user.
pub fn installed_path() -> PathBuf {
    #[cfg(target_os = "linux")]
    return PathBuf::from(format!("/etc/djinn-custodian/clients/{}.json", custody_wire::peer::effective_uid()));
    #[cfg(target_os = "macos")]
    return PathBuf::from(format!(
        "/Library/Application Support/Merely/djinn-custodian/clients/{}.json",
        custody_wire::peer::effective_uid()
    ));
    #[cfg(windows)]
    return PathBuf::from(r"C:\ProgramData\djinn-custodian\client.json");
    #[allow(unreachable_code)]
    PathBuf::new()
}

impl InstalledCustodian {
    /// The installed client file, if this device has a custodian. A file
    /// the desktop user could have written is refused, not ignored. A
    /// harness build may name its own fixture file instead (`--custodian`).
    pub fn load() -> Result<Option<Self>, String> {
        if let Some(fixture) = FIXTURE.get() {
            return Self::from_path(fixture, false).map(Some);
        }
        // A harness build reaches only the custodian it was handed, never
        // the device's own.
        if cfg!(feature = "passphrase-fd") {
            return Ok(None);
        }
        let path = installed_path();
        if !path.exists() {
            return Ok(None);
        }
        Self::from_path(&path, true).map(Some)
    }

    /// Use a fixture client file in place of the installed one. Only builds
    /// with `passphrase-fd` (the harness) expose this.
    #[cfg(feature = "passphrase-fd")]
    pub fn use_fixture(path: PathBuf) {
        let _ = FIXTURE.set(path);
    }

    /// Read a client file. `protected` requires root ownership on Unix.
    pub fn from_path(path: &Path, protected: bool) -> Result<Self, String> {
        #[cfg(unix)]
        if protected {
            use std::os::unix::fs::MetadataExt;
            for checked in [Some(path), path.parent()].into_iter().flatten() {
                let meta = std::fs::metadata(checked).map_err(|e| format!("{}: {e}", checked.display()))?;
                if meta.uid() != 0 || meta.mode() & 0o022 != 0 {
                    return Err(format!(
                        "{} must be root-owned and not group- or world-writable",
                        checked.display()
                    ));
                }
            }
        }
        let _ = protected;
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))
    }
}

/// Whether a persona salt is a Graphshell guest-delegation scope key: the
/// H1 signing consumer.
pub fn is_graphshell_delegation(salt: &[u8]) -> bool {
    let prefix = insigne::delegation::delegation_signing_prefix(GRAPHSHELL_AUDIENCE);
    salt.len() > prefix.len() && salt.starts_with(&prefix)
}

/// The delegation domain the H1 signing consumer uses.
pub const GRAPHSHELL_AUDIENCE: &str = "mere.graphshell";
