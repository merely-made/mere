// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! How a stored record or sync event names its predicate (declared-lens
//! D16, D28): always by the UUIDv5 of its IRI (RFC 9562 §5.5, the URL
//! namespace), the identity sync and comparison use, and by the IRI text too
//! when the predicate is not a term of the core pack, since a UUIDv5 cannot
//! be reversed. Records written before L7 carry the IRI text alone and still
//! read. The core pack is the one the engine embeds. [`nested`] is the
//! serde helper for a record that nests the pair as `{"uuid", "iri"}`.

use std::fmt;

use uuid::Uuid;

use crate::profile::core;

#[path = "stored_nested.rs"]
pub mod nested;

/// The UUIDv5 a stored record names `predicate` by.
pub fn predicate_uuid(predicate: &str) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_URL, predicate.as_bytes())
}

/// The IRI text a stored record carries beside the UUID: none for a term of
/// the core pack, which resolves the UUID.
pub fn stored_iri(predicate: &str) -> Option<&str> {
    core::terms()
        .entry(predicate)
        .is_none()
        .then_some(predicate)
}

/// Why a stored record's predicate is refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoredPredicateError {
    /// The record names neither a UUID nor an IRI.
    Missing,
    /// The UUID is not the UUIDv5 of the IRI beside it.
    Mismatch { uuid: Uuid, iri: String },
    /// The UUID names no term of the core pack, and no IRI says what it is.
    Unresolved(Uuid),
}

impl fmt::Display for StoredPredicateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => f.write_str("a stored relation names no predicate"),
            Self::Mismatch { uuid, iri } => {
                write!(f, "predicate_uuid {uuid} is not the UUIDv5 of {iri}")
            },
            Self::Unresolved(uuid) => write!(
                f,
                "predicate_uuid {uuid} names no core term and the record carries no IRI"
            ),
        }
    }
}

impl std::error::Error for StoredPredicateError {}

/// The predicate IRI a stored record names, from its UUID and IRI fields.
///
/// # Errors
/// A [`StoredPredicateError`]: neither field, a UUID that is not the IRI's,
/// or a UUID the core pack does not resolve with no IRI beside it.
pub fn read_predicate(
    uuid: Option<Uuid>,
    iri: Option<String>,
) -> Result<String, StoredPredicateError> {
    match (uuid, iri) {
        (None, None) => Err(StoredPredicateError::Missing),
        (None, Some(iri)) => Ok(iri),
        (Some(uuid), Some(iri)) if predicate_uuid(&iri) != uuid => {
            Err(StoredPredicateError::Mismatch { uuid, iri })
        },
        (Some(_), Some(iri)) => Ok(iri),
        (Some(uuid), None) => {
            let terms = core::terms();
            terms
                .id_of_uuid(&uuid)
                .and_then(|id| terms.iri(id))
                .map(str::to_owned)
                .ok_or(StoredPredicateError::Unresolved(uuid))
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::core::ids;

    #[test]
    fn a_core_term_stores_its_uuid_alone_and_reads_back() {
        let cites = core::iri(ids::REL_CITES);
        assert_eq!(stored_iri(cites), None);
        assert_eq!(
            read_predicate(Some(predicate_uuid(cites)), None).unwrap(),
            cites
        );
        // The UUID is the one the core pack's table derives.
        assert_eq!(
            core::terms().get(ids::REL_CITES).unwrap().uuid,
            predicate_uuid(cites)
        );
    }

    #[test]
    fn a_non_core_predicate_needs_its_iri() {
        let open = "urn:mere:test:open";
        assert_eq!(stored_iri(open), Some(open));
        assert_eq!(
            read_predicate(Some(predicate_uuid(open)), Some(open.into())).unwrap(),
            open
        );
        assert_eq!(
            read_predicate(Some(predicate_uuid(open)), None),
            Err(StoredPredicateError::Unresolved(predicate_uuid(open)))
        );
    }

    #[test]
    fn a_wrong_uuid_is_refused_and_an_old_record_reads() {
        let cites = core::iri(ids::REL_CITES);
        let quotes = predicate_uuid(core::iri(ids::REL_QUOTES));
        assert!(matches!(
            read_predicate(Some(quotes), Some(cites.into())),
            Err(StoredPredicateError::Mismatch { .. })
        ));
        assert_eq!(read_predicate(None, Some(cites.into())).unwrap(), cites);
        assert_eq!(
            read_predicate(None, None),
            Err(StoredPredicateError::Missing)
        );
    }
}
