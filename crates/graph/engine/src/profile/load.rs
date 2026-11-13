// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The loader's entry point (declared-lens §4): strict decode, digest, COSE
//! verdict and policy, resolution, declare-and-verify, interning, hand-off.
//! It does no I/O; refusals and diagnostics are typed data (R8, engine-split
//! D26), each located by pack id and, where one applies, term IRI.

use std::collections::BTreeSet;

use eidetic::pack::{HeaderError, Pin, Refusal, SignerReview, TrustPolicy};

use super::core::{CORE_PACK, CORE_PACK_ID};
use super::intern::{TermTable, intern};
use super::lens::{LensChoices, ResolvedPin, SelectionRecord, choices};
use super::pack::{declare_and_verify, decode};
use super::resolve::{PackRef, PackSource, resolve};

/// Bounds on the work a load may do (Stack 8); each is a typed refusal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Limits {
    pub max_pack_bytes: usize,
    pub max_packs: usize,
    pub max_terms: usize,
}

impl Default for Limits {
    /// Room for the 1,000-term pack L1 costed, sixty-four packs and a
    /// hundred thousand terms.
    fn default() -> Self {
        Self {
            max_pack_bytes: 4 << 20,
            max_packs: 64,
            max_terms: 100_000,
        }
    }
}

/// Why a load is refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadRefusal {
    /// The bytes offered as `pack` are not a canonical `mere.pack/2` document.
    Decode { pack: String, error: HeaderError },
    /// An offer for `requested` is the pack `found`.
    WrongId { requested: String, found: String },
    /// No offered document matches the pin.
    DigestMismatch { pin: Pin },
    /// The lens `pack` has no lens body.
    NotALens { pack: String },
    /// Refused by the trust policy: `Broken`, or unsigned or unaccepted across trust.
    Trust { pack: String, refusal: Refusal },
    /// These packs require each other.
    Cycle { packs: Vec<String> },
    /// `pack` is needed at two incompatible versions: the one held and a bound it misses.
    TwoVersions { pack: String, versions: [String; 2] },
    /// No offered version of `pack` satisfies `version`, required by `required_by`.
    Unsatisfied {
        pack: String,
        version: String,
        required_by: String,
    },
    /// Two offered documents of `pack` share one version.
    Ambiguous { pack: String, version: String },
    /// Two packs declare `term` with different rows and the lens settles neither.
    Overlap { term: String, packs: [String; 2] },
    /// `pack` declares `term` outside every namespace it provides.
    Undeclared { pack: String, term: String },
    /// `pack` names a handler for `term` the host does not know.
    UnknownHandler {
        pack: String,
        term: String,
        handler: String,
    },
    /// The core pack `pack` sets no observed default for `axis` (R2).
    MissingDefault { pack: String, axis: &'static str },
    /// A bound in [`Limits`] was reached.
    Limit { limit: &'static str, max: usize },
}

/// Accepted, with a note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Diagnostic {
    /// A chain held no value the engine observes; the core default applies (D32).
    UnobservedValue {
        pack: String,
        term: String,
        axis: String,
        value: String,
    },
    /// An axis the engine does not observe; the row keeps it (D31).
    UnobservedAxis {
        pack: String,
        term: String,
        axis: String,
    },
    /// Defaults outside the core pack are not honoured (D13).
    DefaultsIgnored { pack: String },
    /// A settlement no overlap needed.
    UnusedSettlement { term: String, pack: String },
    /// Something the pack carries that a later phase interprets.
    NotYetInterpreted { pack: String, what: String },
    /// A selected pack is itself a lens; only the root lens's body governs,
    /// so its select, settle and presentation choices are not applied.
    NestedLensIgnored { pack: String },
}

/// The loaded semantic profile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub selection: SelectionRecord,
    pub terms: TermTable,
    /// Capability IRIs the packs require of their environment (D16), for the host.
    pub capabilities: BTreeSet<String>,
    pub lens: LensChoices,
    pub diagnostics: Vec<Diagnostic>,
}

/// Load the lens `lens` over the core pack.
///
/// # Errors
///
/// A [`LoadRefusal`] naming the rule and where.
pub fn load<R: SignerReview>(
    lens: &PackRef,
    source: &dyn PackSource,
    policy: &TrustPolicy<R>,
    limits: &Limits,
) -> Result<Profile, LoadRefusal> {
    let core = decode("the core pack", CORE_PACK, limits)?;
    let resolution = resolve(lens, core, source, policy, limits)?;
    let mut diagnostics = Vec::new();
    for chosen in &resolution.packs {
        declare_and_verify(&chosen.pack, source, &mut diagnostics)?;
        let id = &chosen.pack.header.id;
        if chosen.pack.lens.is_some() && *id != resolution.lens && *id != CORE_PACK_ID {
            diagnostics.push(Diagnostic::NestedLensIgnored { pack: id.clone() });
        }
    }
    let root = resolution
        .packs
        .iter()
        .find(|c| c.pack.header.id == resolution.lens)
        .map(|c| &c.pack);
    let (lens_pin, body) = match root.and_then(|p| p.lens.as_ref().map(|b| (&p.pin, b))) {
        Some(found) => found,
        None => {
            return Err(LoadRefusal::NotALens {
                pack: resolution.lens,
            });
        },
    };
    let packs: Vec<_> = resolution.packs.iter().map(|c| c.pack.clone()).collect();
    let terms = intern(&packs, body, limits, &mut diagnostics)?;
    let lens = choices(&resolution.lens, body, &mut diagnostics);
    let selection = SelectionRecord {
        lens: lens_pin.clone(),
        pins: resolution
            .packs
            .iter()
            .map(|c| ResolvedPin {
                pin: c.pack.pin.clone(),
                admitted: c.admitted.clone(),
            })
            .collect(),
    };
    Ok(Profile {
        selection,
        terms,
        capabilities: resolution.capabilities,
        lens,
        diagnostics,
    })
}

/// A source that offers nothing and knows no handler: the core pack alone.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoPacks;

impl PackSource for NoPacks {
    fn offer(&self, _id: &str) -> Vec<super::resolve::Offered> {
        Vec::new()
    }
    fn handler_known(&self, _id: &str) -> bool {
        false
    }
}
