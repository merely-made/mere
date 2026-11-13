// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The Mere bodies of a pack (declared-lens §4): strict decoding of the
//! canonical bytes through `eidetic::pack`, the digest pin, and
//! declare-and-verify (Surfaces 3: a pack declares only under the namespaces
//! it provides, and names only handlers the host knows, D21).

use eidetic::pack::{LensBody, Multihash, PackHeader, PackKind, Pin, VocabularyBody};
use eidetic::schema::{Hash, HashFn};

use super::load::{Diagnostic, Limits, LoadRefusal};
use super::resolve::PackSource;

/// The digest function pins are taken under (D22's default).
pub const PIN_HASH: HashFn = HashFn::Blake3;

/// A strictly decoded pack with its pin and typed bodies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedPack {
    pub header: PackHeader,
    pub pin: Pin,
    pub vocabulary: Option<VocabularyBody>,
    pub lens: Option<LensBody>,
    /// The canonical bytes, kept for signature verification.
    pub canonical: Vec<u8>,
}

/// Decode `canonical`, offered as the pack `requested`.
///
/// # Errors
///
/// [`LoadRefusal::Limit`] past `limits.max_pack_bytes`; [`LoadRefusal::Decode`]
/// for bytes that are not a canonical `mere.pack/2` document.
pub fn decode(
    requested: &str,
    canonical: &[u8],
    limits: &Limits,
) -> Result<DecodedPack, LoadRefusal> {
    if canonical.len() > limits.max_pack_bytes {
        return Err(LoadRefusal::Limit {
            limit: "pack bytes",
            max: limits.max_pack_bytes,
        });
    }
    let refuse = |error| LoadRefusal::Decode {
        pack: requested.to_string(),
        error,
    };
    let header = PackHeader::from_canonical(canonical).map_err(refuse)?;
    let pin = Pin {
        id: header.id.clone(),
        version: header.version.clone(),
        digest: Multihash(Hash::digest(PIN_HASH, canonical)),
    };
    let vocabulary = header.vocabulary().map_err(refuse)?;
    let lens = header.lens().map_err(refuse)?;
    Ok(DecodedPack {
        header,
        pin,
        vocabulary,
        lens,
        canonical: canonical.to_vec(),
    })
}

/// Check what the pack declares against what its header says (step 5).
///
/// # Errors
///
/// [`LoadRefusal::Undeclared`] for a term outside every namespace in
/// `provides`; [`LoadRefusal::UnknownHandler`] for a handler the host does not know.
pub fn declare_and_verify(
    pack: &DecodedPack,
    source: &dyn PackSource,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(), LoadRefusal> {
    let id = &pack.header.id;
    for term in pack.vocabulary.iter().flat_map(|v| &v.terms) {
        if !pack
            .header
            .provides
            .iter()
            .any(|ns| term.iri.starts_with(ns.as_str()))
        {
            return Err(LoadRefusal::Undeclared {
                pack: id.clone(),
                term: term.iri.clone(),
            });
        }
        if let Some(handler) = &term.handler
            && !source.handler_known(handler)
        {
            return Err(LoadRefusal::UnknownHandler {
                pack: id.clone(),
                term: term.iri.clone(),
                handler: handler.clone(),
            });
        }
    }
    for kind in &pack.header.kind {
        if !matches!(kind, PackKind::Vocabulary | PackKind::Lens) {
            diagnostics.push(Diagnostic::NotYetInterpreted {
                pack: id.clone(),
                what: format!("the {kind:?} body"),
            });
        }
    }
    Ok(())
}
