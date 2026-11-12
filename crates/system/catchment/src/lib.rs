// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! # catchment
//!
//! The reservoir service: every mere a persona holds, one per data domain.
//! A mere is its data domain's repository, and all of an identity's meres
//! together are its reservoir (TERMINOLOGY, *reservoir*).
//!
//! This crate holds what the reservoir supplies to every domain, graph or not
//! (domain contract, "What the reservoir supplies"):
//!
//! - the **catalog**: which meres a persona holds and which domain each holds
//!   ([`DomainId`], [`MereId`], [`MereRecord`]);
//! - **access**: the owner's per-application decisions ([`MereApplicationAccess`]);
//! - **store lifetimes**: where a reservoir and its meres live and the one
//!   owner that may open them ([`open_reservoir_backend`], [`open_mere_backend`]);
//! - **sealing**: values sealed under custody ([`SealedValues`]) and the
//!   custody gate on a mere's retained page text ([`RetainedText`]);
//! - the **family-shared root** personas live under ([`shared_root`]);
//! - **slot guards**: compare-and-replace over one muniment slot ([`slot_guard`]);
//! - the **ids** a mere's session lines and their graphs are known by
//!   ([`SessionId`], [`GraphId`]), which the workspace and pandect both name.
//!
//! Its default feature is the ids alone; the service is the `store` feature.
//! It names no graph or host type. A domain's grammar, fold and
//! admission policy are the domain's; Pandect supplies the graph domain's.
//! Generic replication is catchment's when the domain adoption plan's M4 builds
//! it; the graph domain's sync is pandect's `personal_sync` (D69, D133).
//!
//! Split from `pandect::reservoir` under engine split P4 (D3, D9). The stored
//! slot keys and schema strings keep their `pandect` names, because they are
//! data on disk.

pub mod ids;
pub use ids::{GraphId, SessionId};

// The reservoir service proper, behind `store`; the default is the ids alone.
#[cfg(feature = "store")]
pub mod catalog;
#[cfg(feature = "store")]
pub mod paths;
#[cfg(feature = "store")]
pub mod retained_text;
#[cfg(feature = "store")]
pub mod sealed;
#[cfg(feature = "store")]
pub mod shared_root;
// The one compare-and-replace over a muniment slot, for every store that
// writes back bytes it loaded (engine split D84).
#[cfg(feature = "store")]
pub mod slot_guard;
#[cfg(feature = "store")]
pub mod store;

#[cfg(feature = "store")]
pub use catalog::{
    DomainId, MERE_RECORD_SCHEMA, MereId, MereRecord, RESERVOIR_INDEX_SCHEMA, RESERVOIR_INDEX_SLOT,
    ReservoirError,
};
#[cfg(feature = "store")]
pub use paths::{MERES_DIR, RESERVOIR_DB_FILENAME, RESERVOIR_DIR, mere_dir, reservoir_dir};
#[cfg(all(feature = "store", not(target_arch = "wasm32")))]
pub use paths::{open_mere_archive_backend, open_mere_backend, open_reservoir_backend};
#[cfg(all(feature = "store", not(target_arch = "wasm32")))]
pub use retained_text::open_mere_retained_backend;
#[cfg(feature = "store")]
pub use retained_text::{
    DEFAULT_KEPT_BYTES, KEEP_SETTING_SCHEMA, KEEP_SETTING_SLOT, KeepPageText, Keeping,
    MAX_KEPT_BYTES, PlainCustody, RETAINED_DB_FILENAME, RetainedCustody, RetainedText,
    retained_path,
};
#[cfg(feature = "store")]
pub use sealed::{SealedValues, SharedSealer};
#[cfg(feature = "store")]
pub use store::{MereApplicationAccess, ReservoirStore};
