// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The store error type.

use std::fmt;

/// A storage failure.
#[derive(Debug, Clone, PartialEq)]
pub enum StoreError {
    /// Another revised writer retains the database lifetime lock.
    OwnershipBusy,
    /// The platform cannot establish mandatory exclusive admission.
    OwnershipUnavailable(String),
    /// A legacy connection blocked cutover; the refused request has drained.
    UpgradeBlocked,
    /// Cancellation completed. A committed version upgrade is reported honestly.
    OpenCanceled { schema_upgraded: bool },
    /// An existing database cannot satisfy this store's schema without alteration.
    InvalidSchema(String),
    /// The admitted connection was invalidated; fresh authoritative reopen is required.
    OwnershipLost(String),
    /// The host backend (filesystem, OPFS, redb, fjall, ...) failed. The string
    /// is the backend's own error, stringified so the seam stays backend-agnostic.
    Backend(String),
    /// Encoding or decoding a typed value through a [`Codec`](crate::Codec) failed.
    Codec(String),
    /// This backend cannot honor [`Backend::transact`](crate::Backend::transact):
    /// it has no way to run a caller's read-then-write closure against one
    /// consistent snapshot and commit atomically. Distinct from `Backend` so a
    /// caller can match on it and fall back deliberately (e.g. refuse custody
    /// operations rather than risk a stale read) instead of treating it as an
    /// ordinary I/O failure.
    NotTransactional,
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OwnershipBusy => write!(f, "another page owns this database"),
            Self::OwnershipUnavailable(reason) => {
                write!(f, "exclusive ownership unavailable: {reason}")
            },
            Self::UpgradeBlocked => write!(
                f,
                "legacy database connection blocked cutover; request drained"
            ),
            Self::OpenCanceled { schema_upgraded } => write!(
                f,
                "open canceled; schema upgrade committed: {schema_upgraded}"
            ),
            Self::InvalidSchema(reason) => write!(f, "database schema: {reason}"),
            Self::OwnershipLost(reason) => write!(f, "database owner lost: {reason}"),
            StoreError::Backend(msg) => write!(f, "backend: {msg}"),
            StoreError::Codec(msg) => write!(f, "codec: {msg}"),
            StoreError::NotTransactional => {
                write!(f, "backend does not support transactional read-then-write")
            },
        }
    }
}

impl std::error::Error for StoreError {}
