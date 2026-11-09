// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The catalog's names: a data domain, the mere that holds it, and the record
//! the index keeps for each.
//!
//! Ids are derived. A mere's id is a UUIDv5 of the persona and the domain, so
//! the same domain always names the same mere, on every device of that persona.

use std::fmt;
use std::path::PathBuf;

use identity::PersonaId;
use muniment::StoreError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The slot the index lives in. The `pandect/` prefix is the stored key and
/// stays: changing it would orphan every existing reservoir.
pub const RESERVOIR_INDEX_SLOT: &str = "pandect/reservoir/index/v1";
/// Schema of the index document.
pub const RESERVOIR_INDEX_SCHEMA: &str = "pandect.reservoir.index/v1";
/// Schema of one mere record.
pub const MERE_RECORD_SCHEMA: &str = "pandect.reservoir.mere/v1";

/// The namespace every derived mere id is minted under. Fixed forever: changing
/// it would give every existing mere a new id.
const MERE_ID_NAMESPACE: Uuid = Uuid::from_u128(0x6d65_7265_2d72_6573_6572_766f_6972_0001);

pub(crate) const MAX_DOMAIN_LEN: usize = 64;

/// A data domain's stable identifier, such as `divination` or `browsing`.
///
/// Lowercase ASCII letters and digits, with `-` and `.` between them; at most
/// 64 bytes. Stable identifiers are what let two applications find the same
/// mere (reservoir plan, §7).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct DomainId(String);

impl DomainId {
    pub fn new(id: impl Into<String>) -> Result<Self, ReservoirError> {
        let id = id.into();
        if valid_domain(&id) {
            Ok(Self(id))
        } else {
            Err(ReservoirError::InvalidDomain(id))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn valid_domain(id: &str) -> bool {
    let bytes = id.as_bytes();
    let edge = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    !bytes.is_empty()
        && bytes.len() <= MAX_DOMAIN_LEN
        && edge(bytes[0])
        && edge(bytes[bytes.len() - 1])
        && bytes.iter().all(|&b| edge(b) || b == b'-' || b == b'.')
}

impl TryFrom<String> for DomainId {
    type Error = ReservoirError;
    fn try_from(id: String) -> Result<Self, Self::Error> {
        Self::new(id)
    }
}

impl From<DomainId> for String {
    fn from(id: DomainId) -> Self {
        id.0
    }
}

impl fmt::Display for DomainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A mere's id: derived from its persona and domain, never chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MereId(pub Uuid);

impl MereId {
    /// The id of `domain`'s mere in `persona`'s reservoir.
    pub fn derive(persona: PersonaId, domain: &DomainId) -> Self {
        let mut name = persona.as_uuid().as_bytes().to_vec();
        name.push(b'/');
        name.extend_from_slice(domain.as_str().as_bytes());
        Self(Uuid::new_v5(&MERE_ID_NAMESPACE, &name))
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for MereId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// One mere in the reservoir.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MereRecord {
    pub schema: String,
    pub id: MereId,
    pub persona: PersonaId,
    pub domain: DomainId,
    pub created_at_ms: u64,
}

/// Why the reservoir refused.
#[derive(Debug)]
pub enum ReservoirError {
    /// A domain identifier broke the naming rule.
    InvalidDomain(String),
    /// The stored index belongs to another persona.
    PersonaMismatch {
        expected: PersonaId,
        found: PersonaId,
    },
    /// A stored record fails its own identity rules.
    Corrupt(String),
    /// The reservoir could not be opened, including because another process
    /// already owns it.
    Open { path: PathBuf, reason: String },
    /// The store failed underneath.
    Store(StoreError),
}

impl fmt::Display for ReservoirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDomain(id) => write!(
                f,
                "invalid data domain {id:?}: use 1-64 lowercase letters or digits, with '-' or '.' between them"
            ),
            Self::PersonaMismatch { expected, found } => write!(
                f,
                "reservoir belongs to persona {}, not {}",
                found.as_uuid(),
                expected.as_uuid()
            ),
            Self::Corrupt(reason) => write!(f, "reservoir index is corrupt: {reason}"),
            Self::Open { path, reason } => {
                write!(
                    f,
                    "could not open the reservoir at {}: {reason}",
                    path.display()
                )
            },
            Self::Store(error) => write!(f, "reservoir store failed: {error}"),
        }
    }
}

impl std::error::Error for ReservoirError {}

impl From<StoreError> for ReservoirError {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}
