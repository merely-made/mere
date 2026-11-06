// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A mere's retained page text, owned by the reservoir.
//!
//! The records are Eidetic's `PageTextStore` and `ResourceCaptureStore`
//! formats, the ones the browser extension writes (`blob/<blake3>`,
//! `page-text/<url hash>`, `resource-capture/<id>/page-text/<hash>`), so one
//! reader serves both writers. The store lives beside the mere's archive
//! (`<reservoir>/meres/<id>/retained.redb`) and is opened where the archive is:
//! the reservoir's owner opens it and lends it to routes. An application never
//! opens the file.
//!
//! Two things sit here with the store because they are about the store, not
//! about any one route:
//!
//! - the **"Keep page text" setting**, stored in the store itself under
//!   [`KEEP_SETTING_SLOT`]. It is off until a person turns it on, and it means
//!   what the browser's control means: pages the person loads, never background
//!   fetches, bounded per page, with excluded origins.
//! - the **custody gate**. A store over a sealed backend is sealed by that
//!   backend (the same `Backend` seam the graph sessions use), and a locked or
//!   unavailable custody pauses writing instead of falling back to plaintext.
//!   The gate asks a [`RetainedCustody`] before every write; reads are never
//!   gated here because a sealed backend refuses them by itself.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use identity::PersonaId;
use muniment::{Backend, JsonSlots};
use serde::{Deserialize, Serialize};

use crate::reservoir::{MereId, ReservoirError, mere_dir};

/// The retained store's file inside a mere's directory.
pub const RETAINED_DB_FILENAME: &str = "retained.redb";
/// The slot the "Keep page text" setting lives in.
pub const KEEP_SETTING_SLOT: &str = "retained-text/keep-page-text/v1";
/// Schema of the stored setting.
pub const KEEP_SETTING_SCHEMA: &str = "pandect.retained-text.keep/v1";
/// The most text one page may keep, whatever the setting asks for.
pub const MAX_KEPT_BYTES: usize = 256 * 1024;
/// The default per-page bound.
pub const DEFAULT_KEPT_BYTES: usize = 128 * 1024;

/// Where a mere's retained page text lives.
pub fn retained_path(shared_root: &Path, persona: PersonaId, mere: MereId) -> PathBuf {
    mere_dir(shared_root, persona, mere).join(RETAINED_DB_FILENAME)
}

/// Open mere `mere`'s retained page-text backend, creating it if absent.
///
/// Only the reservoir's owner, who holds [`crate::open_reservoir_backend`]'s
/// lock, should call this; redb's own lock refuses a second opener at once.
#[cfg(not(target_arch = "wasm32"))]
pub fn open_mere_retained_backend(
    shared_root: &Path,
    persona: PersonaId,
    mere: MereId,
) -> Result<muniment::RedbBackend, ReservoirError> {
    let path = retained_path(shared_root, persona, mere);
    std::fs::create_dir_all(path.parent().expect("mere directory")).map_err(|error| {
        ReservoirError::Open {
            path: path.clone(),
            reason: error.to_string(),
        }
    })?;
    muniment::RedbBackend::open(&path).map_err(|error| ReservoirError::Open {
        path,
        reason: error.to_string(),
    })
}

/// The person's "Keep page text" choice for one mere. Off by default.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeepPageText {
    pub enabled: bool,
    /// Per-page byte bound; `0` means [`DEFAULT_KEPT_BYTES`], and nothing
    /// exceeds [`MAX_KEPT_BYTES`].
    #[serde(default)]
    pub max_bytes: usize,
    #[serde(default)]
    pub excluded_origins: BTreeSet<String>,
}

#[derive(Serialize, Deserialize)]
struct StoredKeep {
    schema: String,
    #[serde(flatten)]
    setting: KeepPageText,
}

/// Whether the store may be written now.
pub trait RetainedCustody: Send + Sync {
    /// `Ok` when a write may proceed; `Err` carries the reason a person reads.
    /// A sealed store reports locked or unavailable custody here, so writing
    /// pauses instead of falling back to plaintext.
    fn writable(&self) -> Result<(), String>;
}

/// A store whose backend is not sealed: nothing to wait for. This is every
/// mere's store today; sealing a mere is a custody decision made where the
/// backend is composed, not here.
pub struct PlainCustody;

impl RetainedCustody for PlainCustody {
    fn writable(&self) -> Result<(), String> {
        Ok(())
    }
}

/// What a write is allowed to do right now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Keeping {
    /// The setting is off; nothing is written.
    Off,
    /// On, and the store accepts writes.
    On,
    /// On, but custody refuses writes; nothing is written.
    Paused(String),
}

/// A mere's retained page-text store: its backend, its setting and its gate.
///
/// `B` is a muniment backend handle (a clone shares the store). Cloning the
/// store clones the handle and the gate.
#[derive(Clone)]
pub struct RetainedText<B> {
    backend: B,
    custody: Arc<dyn RetainedCustody>,
}

impl<B: Backend + Clone> RetainedText<B> {
    pub fn new(backend: B, custody: Arc<dyn RetainedCustody>) -> Self {
        Self { backend, custody }
    }

    /// The backend the page-text records are read through.
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// The stored setting; off when nothing has been chosen.
    pub async fn setting(&self) -> Result<KeepPageText, String> {
        let stored: Option<StoredKeep> = JsonSlots::new(self.backend.clone())
            .load(KEEP_SETTING_SLOT)
            .await
            .map_err(|error| error.to_string())?;
        match stored {
            None => Ok(KeepPageText::default()),
            Some(stored) if stored.schema == KEEP_SETTING_SCHEMA => Ok(stored.setting),
            Some(_) => Err("the Keep page text setting has an unknown schema".into()),
        }
    }

    /// Record the setting. Writing the setting is itself a write the gate
    /// governs.
    pub async fn set_setting(&self, setting: KeepPageText) -> Result<(), String> {
        self.custody.writable()?;
        JsonSlots::new(self.backend.clone())
            .save(
                KEEP_SETTING_SLOT,
                &StoredKeep {
                    schema: KEEP_SETTING_SCHEMA.into(),
                    setting,
                },
            )
            .await
            .map_err(|error| error.to_string())
    }

    /// The setting and the gate, as a person should see them.
    pub async fn keeping(&self) -> Result<Keeping, String> {
        if !self.setting().await?.enabled {
            return Ok(Keeping::Off);
        }
        Ok(match self.custody.writable() {
            Ok(()) => Keeping::On,
            Err(reason) => Keeping::Paused(reason),
        })
    }

    /// Ask permission to write page text: `Ok` with the setting when it is on
    /// and custody allows, otherwise the reason it is not. The caller writes
    /// only on `Ok`.
    pub async fn permit_write(&self) -> Result<KeepPageText, Keeping> {
        let setting = self.setting().await.map_err(Keeping::Paused)?;
        if !setting.enabled {
            return Err(Keeping::Off);
        }
        self.custody.writable().map_err(Keeping::Paused)?;
        Ok(setting)
    }
}
