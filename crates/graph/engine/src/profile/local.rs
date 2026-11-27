// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A mere's predicate declarations as a local vocabulary pack layered after
//! the core (declared-lens D24). The declarations stay resource records; at
//! open they compile, in memory, into a pack that goes through the same
//! loader as any other, so a declaration a core term already settles, or two
//! that disagree with no choice made, is refused rather than layered.

use std::collections::BTreeMap;
use std::fmt;

use eidetic::pack::{
    HeaderError, Origin, PACK2_FORMAT, PackHeader, PackKind, Term, TermKind, TypedBody,
    VocabularyBody,
};
use eidetic::schema::TrustEnvelope;

use super::core::DECLARATIONS_PACK_ID;
use super::intern::{Observed, TermTable};
use super::load::{Limits, LoadRefusal, load_local_lens};
use super::pack::PIN_HASH;
use super::resolve::{Offered, PackRef, PackSource};
use crate::model::GraphStratum;
use crate::model::predicate_declarations::{PredicateDeclarationError, PredicateDeclarations};

/// Why a mere's declarations do not layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LocalRefusal {
    /// A predicate's declarations conflict, or name a core term.
    Declaration(PredicateDeclarationError),
    /// The compiled pack did not encode.
    Encode(HeaderError),
    /// The loader refused the compiled pack.
    Load(LoadRefusal),
}

impl fmt::Display for LocalRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Declaration(e) => write!(f, "{e}"),
            Self::Encode(e) => write!(f, "the declarations pack: {e}"),
            Self::Load(e) => write!(f, "the declarations pack: {e:?}"),
        }
    }
}

impl std::error::Error for LocalRefusal {}

/// The declarations pack: one predicate row per declared IRI, carrying its
/// effective stratum. Canonical bytes, unsigned.
///
/// # Errors
/// When a declaration is refused or the pack does not encode.
pub fn declarations_pack<'a>(
    declarations: impl IntoIterator<Item = (&'a str, &'a PredicateDeclarations)>,
) -> Result<Option<Vec<u8>>, LocalRefusal> {
    let mut terms = Vec::new();
    for (predicate, declared) in declarations {
        let stratum = declared
            .effective_stratum(predicate)
            .map_err(LocalRefusal::Declaration)?;
        terms.push(Term {
            iri: predicate.to_owned(),
            kind: TermKind::Predicate,
            family: None,
            handler: None,
            axes: BTreeMap::from([(
                GraphStratum::AXIS.to_owned(),
                vec![stratum.iri().to_owned()],
            )]),
            labels: BTreeMap::new(),
            inverse_labels: None,
        });
    }
    if terms.is_empty() {
        return Ok(None);
    }
    let provides = terms.iter().map(|t| t.iri.clone()).collect();
    let vocabulary = VocabularyBody {
        terms,
        defaults: BTreeMap::new(),
    };
    let header = PackHeader {
        format: PACK2_FORMAT.to_owned(),
        id: DECLARATIONS_PACK_ID.to_owned(),
        version: "0.0.0".to_owned(),
        kind: vec![PackKind::Vocabulary, PackKind::Lens],
        licence: "MPL-2.0".to_owned(),
        credit: Vec::new(),
        requires: Vec::new(),
        provides,
        contributes: Vec::new(),
        scopes: Vec::new(),
        parts: Vec::new(),
        body: BTreeMap::from([
            (
                PackKind::Vocabulary,
                vocabulary.to_value().map_err(|error| {
                    LocalRefusal::Encode(HeaderError::Body {
                        kind: PackKind::Vocabulary,
                        error,
                    })
                })?,
            ),
            (
                PackKind::Lens,
                eidetic::pack::LensBody::default()
                    .to_value()
                    .map_err(|error| {
                        LocalRefusal::Encode(HeaderError::Body {
                            kind: PackKind::Lens,
                            error,
                        })
                    })?,
            ),
        ]),
    };
    header
        .to_canonical()
        .map(Some)
        .map_err(LocalRefusal::Encode)
}

struct Declared<'a>(&'a [u8]);

impl PackSource for Declared<'_> {
    fn offer(&self, id: &str) -> Vec<Offered> {
        if id != DECLARATIONS_PACK_ID {
            return Vec::new();
        }
        vec![Offered {
            canonical: self.0.to_vec(),
            envelope: TrustEnvelope::self_asserted(),
            origin: Origin::Local,
        }]
    }
    fn handler_known(&self, _: &str) -> bool {
        false
    }
}

/// The core terms with a mere's declarations layered after them: the table a
/// session classifies and places its predicates by.
///
/// # Errors
/// When a declaration is refused, or the loader refuses the compiled pack.
pub fn layered<'a>(
    declarations: impl IntoIterator<Item = (&'a str, &'a PredicateDeclarations)>,
) -> Result<TermTable, LocalRefusal> {
    let Some(bytes) = declarations_pack(declarations)? else {
        return Ok(super::core::terms().clone());
    };
    let pin = PackHeader::pin_of(&bytes, PIN_HASH).map_err(LocalRefusal::Encode)?;
    load_local_lens(&PackRef::Pinned(pin), &Declared(&bytes), &Limits::default())
        .map(|profile| profile.terms)
        .map_err(LocalRefusal::Load)
}

#[cfg(test)]
#[path = "tests/local.rs"]
mod tests;
