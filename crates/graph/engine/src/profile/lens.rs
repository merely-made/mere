// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The lens (declared-lens §2, D1): what the root lens selects, settles and
//! presents, and the selection record a load returns. Scenes, the derived
//! layer and reopen carry the record (D4, D14); its canonical CBOR is
//! [`SelectionRecord::to_canonical`].

use eidetic::pack::{CanonicalError, Entail, LensBody, Pin, Presentation, to_canonical};
use serde::{Deserialize, Serialize};

use super::load::Diagnostic;

/// How a resolved pack was admitted (D3, D20).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdmittedAs {
    /// Signed by the release key.
    Release,
    /// Signed by a signer the install review accepted; the key in hex.
    Reviewed { signer: String },
    /// Local: unsigned, or signed by no accepted signer.
    Local,
}

/// One resolved pack.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedPin {
    pub pin: Pin,
    pub admitted: AdmittedAs,
}

/// What a load selected: the lens pin, then every resolved pin in order (the
/// core pack first, then dependencies before dependents, ties by id).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionRecord {
    pub lens: Pin,
    pub pins: Vec<ResolvedPin>,
}

impl SelectionRecord {
    /// The record's canonical CBOR (D5), the bytes a golden fixture compares.
    ///
    /// # Errors
    ///
    /// Never for a record a load built; the codec's refusals otherwise.
    pub fn to_canonical(&self) -> Result<Vec<u8>, CanonicalError> {
        to_canonical(self)
    }
}

/// The root lens's choices the engine hands on.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LensChoices {
    pub entail: Entail,
    pub present: Vec<Presentation>,
    /// Active rule-set IRIs, carried until L4 interprets them.
    pub rules: Vec<String>,
    /// Active shape-set IRIs, carried until L4 interprets them.
    pub shapes: Vec<String>,
}

/// The choices of `body`, the lens `id`, noting what is not interpreted yet.
pub(crate) fn choices(id: &str, body: &LensBody, diagnostics: &mut Vec<Diagnostic>) -> LensChoices {
    for (what, sets) in [("rule sets", &body.rules), ("shape sets", &body.shapes)] {
        if !sets.is_empty() {
            diagnostics.push(Diagnostic::NotYetInterpreted {
                pack: id.to_string(),
                what: what.to_string(),
            });
        }
    }
    LensChoices {
        entail: body.entail,
        present: body.present.clone(),
        rules: body.rules.clone(),
        shapes: body.shapes.clone(),
    }
}
