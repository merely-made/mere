// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Hocket's own identity, adopted into custody with its fingerprint unchanged
//! (dramatis repo plan, D13).
//!
//! Before DR-C Hocket kept a DPAPI-rooted sealed record of its own: the key
//! behind the contact token musicians had already shared. Hocket no longer
//! opens it; djinn does, once, and places the key in the vault:
//!
//! - under whichever persona already holds that key, if one does (Hocket's
//!   own earlier adoption into the family persona);
//! - otherwise as its own persona, [`APART_PROFILE`], beside the persona in
//!   use, which is never overwritten.
//!
//! The answer is written beside Hocket's data as a public forwarding note,
//! [`MARKER`]: the persona and its public key, nothing secret. Hocket reads it
//! and speaks as that persona through the custody route. The old record is
//! left where it was, for its owner to retire.

use std::path::{Path, PathBuf};

use castellan::authority::PersonaeHost;
use castellan::custody::{IdentityStorage, SealedIdentityProvider, load_existing_auto_unlock_root};
use personae::{IdentityError, ProfileId, SealedRecordStorage};
use serde::{Deserialize, Serialize};

/// Hocket's sealed identity record, relative to its records root.
pub const RECORD: &str = "hocket/local-identity.json";
/// The same record under the product's pre-rename name.
const LEGACY_RECORD: &str = "strophe/local-identity.json";
/// The forwarding note djinn leaves in Hocket's data root.
pub const MARKER: &str = "custody-profile.json";
/// The persona Hocket's key is adopted as when it is kept apart.
pub const APART_PROFILE: &str = "hocket";

/// Where Hocket's identity now lives in custody: public parts only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HocketCustody {
    /// The persona holding Hocket's key.
    pub profile: String,
    /// Its master public key, as Hocket's contact token (64 hex characters).
    pub public_key: String,
}

/// Hocket's platform data root, as Hocket names it.
pub fn data_root() -> Option<PathBuf> {
    if let Some(root) = std::env::var_os("LOCALAPPDATA") {
        return Some(PathBuf::from(root).join("Hocket"));
    }
    if let Some(root) = std::env::var_os("XDG_DATA_HOME") {
        return Some(PathBuf::from(root).join("hocket"));
    }
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share/hocket"))
}

/// Adopt Hocket's record from its platform data root. `None` when there is
/// no record to adopt.
pub fn adopt<S: IdentityStorage + 'static>(
    host: &PersonaeHost<S>,
) -> Result<Option<HocketCustody>, IdentityError> {
    match data_root() {
        Some(root) => adopt_from(host, &root),
        None => Ok(None),
    }
}

/// Adopt the record under `data_root`. Idempotent: a key the vault already
/// holds is found, not imported again.
pub fn adopt_from<S: IdentityStorage + 'static>(
    host: &PersonaeHost<S>,
    data_root: &Path,
) -> Result<Option<HocketCustody>, IdentityError> {
    if host.is_locked() {
        return Err(IdentityError::Locked);
    }
    let Some(root_key) =
        load_existing_auto_unlock_root(data_root.join("personae/auto-unlock-root.json"))?
    else {
        return Ok(None);
    };
    let records = SealedRecordStorage::open_with_key(data_root.join("personae/records"), root_key);
    let path = if records.load_record::<serde_json::Value>(RECORD)?.is_some() {
        RECORD
    } else if records.load_record::<serde_json::Value>(LEGACY_RECORD)?.is_some() {
        LEGACY_RECORD
    } else {
        return Ok(None);
    };
    // The record exists, so this loads it and never mints.
    let record = SealedIdentityProvider::load_or_create(&records, path)?;
    let master = record.master_keypair().clone();
    let public_key = master.public_key();
    let profile = match host.profile_holding(&public_key)? {
        Some(held) => held,
        None => {
            let apart = ProfileId(APART_PROFILE.into());
            host.import_profile(&apart, "Hocket", master)?;
            tracing::info!(profile = APART_PROFILE, "adopted Hocket's identity into custody");
            apart
        },
    };
    let custody = HocketCustody {
        profile: profile.0,
        public_key: public_key
            .to_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    };
    write_marker(data_root, &custody)?;
    Ok(Some(custody))
}

fn write_marker(data_root: &Path, custody: &HocketCustody) -> Result<(), IdentityError> {
    let path = data_root.join(MARKER);
    let temporary = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(custody)
        .map_err(|error| IdentityError::Backend(error.to_string()))?;
    std::fs::write(&temporary, bytes)
        .and_then(|()| std::fs::rename(&temporary, &path))
        .map_err(|error| IdentityError::Backend(format!("write {path:?}: {error}")))
}
