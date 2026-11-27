// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Relations named by IRI (declared-lens D20). A predicate is its IRI; what
//! it is (family, stratum, durability) is its row in a term table, read
//! through the generated ids. [`Vocabulary`] is the table a payload classifies
//! by, [`RelationSelector`] picks relations by family or predicate,
//! [`stored`] is how a stored record names its predicate (D28), and
//! [`legacy`] maps the names older engines stored to the IRIs they meant (D17).

use std::fmt;

use crate::profile::core::{self, ids};
use crate::profile::intern::{Durability, TermId, TermTable};

#[path = "relation/legacy.rs"]
pub mod legacy;
#[path = "relation/stored.rs"]
pub mod stored;

/// The term table a payload classifies its predicates by: the core profile,
/// or a host's table layered on it. A classification context, not content, so
/// two payloads compare equal whatever table each reads.
#[derive(Clone, Copy)]
pub struct Vocabulary(&'static TermTable);

impl Vocabulary {
    /// The process's core profile (D23).
    pub fn core() -> Self {
        Self(core::terms())
    }

    /// A host's table, which must hold the core terms at their core ids.
    pub fn new(terms: &'static TermTable) -> Self {
        Self(terms)
    }

    pub fn terms(self) -> &'static TermTable {
        self.0
    }

    /// The family `predicate` is declared in, if any.
    pub fn family(self, predicate: &str) -> Option<TermId> {
        self.0.family(predicate)
    }

    /// The family a relation counts in: its declared one, else semantic. An
    /// undeclared IRI is an open predicate, which reads as semantic.
    pub fn relation_family(self, predicate: &str) -> TermId {
        self.family(predicate).unwrap_or(ids::FAMILY_SEMANTIC)
    }

    pub fn is_semantic(self, predicate: &str) -> bool {
        self.relation_family(predicate) == ids::FAMILY_SEMANTIC
    }

    /// Whether held records of `predicate` last only while the session does.
    pub fn is_session_only(self, predicate: &str) -> bool {
        self.0.row(predicate).durability == Durability::Session
    }
}

impl Default for Vocabulary {
    fn default() -> Self {
        Self::core()
    }
}

impl PartialEq for Vocabulary {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for Vocabulary {}

impl fmt::Debug for Vocabulary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vocabulary({} terms)", self.0.len())
    }
}

/// Which relations on a pair to read or withdraw: those of one family, or of
/// one predicate. Both are IRIs, so a selector stores as IRIs (D16, D17).
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum RelationSelector {
    /// A family's IRI.
    Family(String),
    /// A predicate's IRI.
    Predicate(String),
}

impl RelationSelector {
    /// Every relation of the core family `family`.
    pub fn family(family: TermId) -> Self {
        Self::Family(core::iri(family).to_owned())
    }

    /// Every relation of `predicate`.
    pub fn predicate(predicate: impl Into<String>) -> Self {
        Self::Predicate(predicate.into())
    }

    /// Whether a relation of `predicate` is selected, read through `vocabulary`.
    pub fn matches(&self, predicate: &str, vocabulary: Vocabulary) -> bool {
        match self {
            Self::Family(family) => {
                vocabulary
                    .terms()
                    .iri(vocabulary.relation_family(predicate))
                    == Some(family)
            },
            Self::Predicate(iri) => iri == predicate,
        }
    }
}

impl<'de> serde::Deserialize<'de> for RelationSelector {
    /// Reads the IRI form, and the family and sub-kind names stored before L7.
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (key, value): (String, String) = legacy::one_entry(deserializer)?;
        Ok(match key.as_str() {
            "Predicate" => Self::Predicate(value),
            "Family" if legacy::is_iri(&value) => Self::Family(value),
            "Family" => Self::Family(legacy::family_iri(&value)),
            family => Self::Predicate(legacy::predicate_iri(family, &value)),
        })
    }
}
