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
//! - the **ids** a mere's session lines and their graphs are known by
//!   ([`SessionId`], [`GraphId`]), which the workspace and pandect both name.
//!
//! It names no graph or host type. A domain's grammar, fold and
//! admission policy are the domain's; Pandect supplies the graph domain's.
//! Replication is the resident's (`personal_sync`) today.
//!
//! Split from `pandect::reservoir` under engine split P4 (D3, D9). The stored
//! slot keys and schema strings keep their `pandect` names, because they are
//! data on disk.

pub mod catalog;
pub mod ids;
pub mod paths;
pub mod retained_text;
pub mod sealed;
pub mod shared_root;
pub mod store;

pub use catalog::{
    DomainId, MERE_RECORD_SCHEMA, MereId, MereRecord, RESERVOIR_INDEX_SCHEMA,
    RESERVOIR_INDEX_SLOT, ReservoirError,
};
pub use ids::{GraphId, SessionId};
pub use paths::{MERES_DIR, RESERVOIR_DB_FILENAME, RESERVOIR_DIR, mere_dir, reservoir_dir};
#[cfg(not(target_arch = "wasm32"))]
pub use paths::{open_mere_archive_backend, open_mere_backend, open_reservoir_backend};
#[cfg(not(target_arch = "wasm32"))]
pub use retained_text::open_mere_retained_backend;
pub use retained_text::{
    DEFAULT_KEPT_BYTES, KEEP_SETTING_SCHEMA, KEEP_SETTING_SLOT, KeepPageText, Keeping,
    MAX_KEPT_BYTES, PlainCustody, RETAINED_DB_FILENAME, RetainedCustody, RetainedText,
    retained_path,
};
pub use sealed::{SealedValues, SharedSealer};
pub use store::{MereApplicationAccess, ReservoirStore};
