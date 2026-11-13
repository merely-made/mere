// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Interning (declared-lens §4): term IRIs to dense ids and UUIDv5
//! identities, axis rows as the observed enums plus a carrier that keeps every
//! declared chain verbatim (D31), and the fallback chain, then the core
//! pack's default (D32).

use std::collections::BTreeMap;

use eidetic::pack::{LensBody, Term, TermKind};
use uuid::Uuid;

use super::load::{Diagnostic, Limits, LoadRefusal};
use super::pack::DecodedPack;
use crate::model::GraphStratum;

/// A term's dense id within one profile. The core pack's ids are fixed by
/// build.rs ([`super::core::ids`]), since the core pack interns first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermId(pub u32);

/// An axis whose values the engine observes (D6, D31).
pub trait Observed: Sized + Copy {
    /// The axis IRI.
    const AXIS: &'static str;
    /// The observed value an IRI names, if the engine knows it.
    fn from_iri(iri: &str) -> Option<Self>;
}

macro_rules! observed {
    ($(#[$m:meta])* $name:ident = $axis:literal { $($(#[$vm:meta])* $variant:ident = $iri:literal),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum $name { $($(#[$vm])* $variant),+ }
        impl Observed for $name {
            const AXIS: &'static str = $axis;
            fn from_iri(iri: &str) -> Option<Self> {
                match iri { $($iri => Some(Self::$variant),)+ _ => None }
            }
        }
    };
}

impl Observed for GraphStratum {
    const AXIS: &'static str = "https://mere.computer/ns/axis#stratum";
    fn from_iri(iri: &str) -> Option<Self> {
        match iri {
            "https://mere.computer/ns/stratum#resource" => Some(Self::Resource),
            "https://mere.computer/ns/stratum#surface" => Some(Self::Surface),
            _ => None,
        }
    }
}

observed!(
    /// How long a held record lasts.
    Durability = "https://mere.computer/ns/axis#durability" {
        RetainedUntilCompaction = "https://mere.computer/ns/durability#retained-until-compaction",
        Session = "https://mere.computer/ns/durability#session",
    }
);
observed!(
    /// Who may withdraw an assertion (link model).
    Retractability = "https://mere.computer/ns/axis#retractability" {
        Asserter = "https://mere.computer/ns/retractability#asserter",
    }
);
observed!(
    /// Whether the relation projects to RDF.
    Projectability = "https://mere.computer/ns/axis#projectability" {
        Projectable = "https://mere.computer/ns/projectability#projectable",
        NotProjectable = "https://mere.computer/ns/projectability#not-projectable",
    }
);
observed!(
    /// What the relation contributes to a layout force.
    Force = "https://mere.computer/ns/axis#force" {
        None = "https://mere.computer/ns/force#none",
    }
);
observed!(
    /// Whether the relation reads the same both ways.
    Symmetry = "https://mere.computer/ns/axis#symmetry" {
        Directed = "https://mere.computer/ns/symmetry#directed",
        Symmetric = "https://mere.computer/ns/symmetry#symmetric",
    }
);
observed!(
    /// The presentation class; how it is drawn is the host's.
    PresentationClass = "https://mere.computer/ns/axis#presentation" {
        Generic = "https://mere.computer/ns/presentation#generic",
    }
);
observed!(
    /// Whether facts of this term are derived (D26).
    Derivation = "https://mere.computer/ns/axis#derivation" {
        NotDerived = "https://mere.computer/ns/derivation#not-derived",
        Derived = "https://mere.computer/ns/derivation#derived",
    }
);

/// The axes the engine observes, by IRI.
pub const OBSERVED_AXES: [&str; 8] = [
    GraphStratum::AXIS,
    Durability::AXIS,
    Retractability::AXIS,
    Projectability::AXIS,
    Force::AXIS,
    Symmetry::AXIS,
    PresentationClass::AXIS,
    Derivation::AXIS,
];

/// A term's effective axis values, and the chains it declared (the carrier).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AxisRow {
    pub stratum: GraphStratum,
    pub durability: Durability,
    pub retractability: Retractability,
    pub projectability: Projectability,
    pub force: Force,
    pub symmetry: Symmetry,
    pub presentation: PresentationClass,
    pub derivation: Derivation,
    /// Every axis the row declared, observed or not, verbatim.
    pub declared: BTreeMap<String, Vec<String>>,
}

/// One interned term.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TermEntry {
    pub iri: String,
    /// UUIDv5 of the IRI in the URL namespace (RFC 9562 §5.5), derived, never stored in a pack.
    pub uuid: Uuid,
    pub kind: TermKind,
    pub family: Option<String>,
    pub handler: Option<String>,
    /// The pack whose row this is, by id.
    pub pack: String,
    pub row: AxisRow,
}

/// Dense id to term; IRI to id; the core defaults for unknown IRIs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TermTable {
    entries: Vec<TermEntry>,
    by_iri: BTreeMap<String, TermId>,
    defaults: AxisRow,
}

impl TermTable {
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn id(&self, iri: &str) -> Option<TermId> {
        self.by_iri.get(iri).copied()
    }
    pub fn get(&self, id: TermId) -> Option<&TermEntry> {
        self.entries.get(id.0 as usize)
    }
    pub fn entry(&self, iri: &str) -> Option<&TermEntry> {
        self.id(iri).and_then(|id| self.get(id))
    }
    pub fn iter(&self) -> impl Iterator<Item = (TermId, &TermEntry)> {
        (0u32..).map(TermId).zip(&self.entries)
    }
    /// The axis row of `iri`, or the core defaults for an undeclared IRI.
    pub fn row(&self, iri: &str) -> &AxisRow {
        self.entry(iri).map_or(&self.defaults, |e| &e.row)
    }
    /// The stratum of `iri`: its row's, else the core default.
    pub fn stratum(&self, iri: &str) -> GraphStratum {
        self.row(iri).stratum
    }
}

/// Intern the vocabularies of `packs` (in resolved order, core first) under
/// the root lens's settlements.
pub(crate) fn intern(
    packs: &[DecodedPack],
    lens: &LensBody,
    limits: &Limits,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<TermTable, LoadRefusal> {
    let core = &packs[0];
    let core_defaults = core.vocabulary.as_ref().map(|v| &v.defaults);
    let defaults = default_row(core_defaults, &core.header.id)?;
    let mut winners: Vec<(&Term, &str)> = Vec::new();
    let mut by_iri = BTreeMap::new();
    let mut settled_terms = std::collections::BTreeSet::new();
    for pack in packs {
        let Some(vocabulary) = &pack.vocabulary else {
            continue;
        };
        let id = pack.header.id.as_str();
        if id != core.header.id && !vocabulary.defaults.is_empty() {
            diagnostics.push(Diagnostic::DefaultsIgnored {
                pack: id.to_string(),
            });
        }
        for term in &vocabulary.terms {
            match by_iri.get(&term.iri).copied() {
                None => {
                    if winners.len() >= limits.max_terms {
                        return Err(LoadRefusal::Limit {
                            limit: "terms",
                            max: limits.max_terms,
                        });
                    }
                    by_iri.insert(term.iri.clone(), TermId(winners.len() as u32));
                    winners.push((term, id));
                },
                Some(TermId(i)) => {
                    let (held, holder) = winners[i as usize];
                    if held == term {
                        continue;
                    }
                    let settled = lens.settle.iter().find(|s| s.term == term.iri);
                    settled_terms.insert(term.iri.as_str());
                    match settled.map(|s| s.pack.as_str()) {
                        Some(p) if p == id => winners[i as usize] = (term, id),
                        Some(p) if p == holder => {},
                        _ => {
                            return Err(LoadRefusal::Overlap {
                                term: term.iri.clone(),
                                packs: [holder.to_string(), id.to_string()],
                            });
                        },
                    }
                },
            }
        }
    }
    for s in &lens.settle {
        if !settled_terms.contains(s.term.as_str()) {
            diagnostics.push(Diagnostic::UnusedSettlement {
                term: s.term.clone(),
                pack: s.pack.clone(),
            });
        }
    }
    let entries = winners
        .into_iter()
        .map(|(term, pack)| TermEntry {
            iri: term.iri.clone(),
            uuid: Uuid::new_v5(&Uuid::NAMESPACE_URL, term.iri.as_bytes()),
            kind: term.kind,
            family: term.family.clone(),
            handler: term.handler.clone(),
            pack: pack.to_string(),
            row: row(term, pack, &defaults, diagnostics),
        })
        .collect();
    Ok(TermTable {
        entries,
        by_iri,
        defaults,
    })
}

/// The core pack's defaults as a row; each observed axis must have one (R2).
fn default_row(
    defaults: Option<&BTreeMap<String, String>>,
    core: &str,
) -> Result<AxisRow, LoadRefusal> {
    fn one<T: Observed>(
        d: Option<&BTreeMap<String, String>>,
        core: &str,
    ) -> Result<T, LoadRefusal> {
        d.and_then(|d| d.get(T::AXIS))
            .and_then(|v| T::from_iri(v))
            .ok_or_else(|| LoadRefusal::MissingDefault {
                pack: core.to_string(),
                axis: T::AXIS,
            })
    }
    Ok(AxisRow {
        stratum: one(defaults, core)?,
        durability: one(defaults, core)?,
        retractability: one(defaults, core)?,
        projectability: one(defaults, core)?,
        force: one(defaults, core)?,
        symmetry: one(defaults, core)?,
        presentation: one(defaults, core)?,
        derivation: one(defaults, core)?,
        declared: BTreeMap::new(),
    })
}

fn row(term: &Term, pack: &str, d: &AxisRow, diagnostics: &mut Vec<Diagnostic>) -> AxisRow {
    let mut walk = Walk {
        term,
        pack,
        diagnostics,
    };
    let row = AxisRow {
        stratum: walk.axis(d.stratum),
        durability: walk.axis(d.durability),
        retractability: walk.axis(d.retractability),
        projectability: walk.axis(d.projectability),
        force: walk.axis(d.force),
        symmetry: walk.axis(d.symmetry),
        presentation: walk.axis(d.presentation),
        derivation: walk.axis(d.derivation),
        declared: term.axes.clone(),
    };
    for axis in term.axes.keys() {
        if !OBSERVED_AXES.contains(&axis.as_str()) {
            walk.note(axis, None);
        }
    }
    row
}

struct Walk<'a> {
    term: &'a Term,
    pack: &'a str,
    diagnostics: &'a mut Vec<Diagnostic>,
}

impl Walk<'_> {
    /// The first observed value in the term's chain, newest first; else the default.
    fn axis<T: Observed>(&mut self, default: T) -> T {
        let Some(chain) = self.term.axes.get(T::AXIS) else {
            return default;
        };
        if let Some(v) = chain.iter().find_map(|v| T::from_iri(v)) {
            return v;
        }
        self.note(T::AXIS, chain.first().cloned());
        default
    }

    fn note(&mut self, axis: &str, value: Option<String>) {
        let (pack, term) = (self.pack.to_string(), self.term.iri.clone());
        self.diagnostics.push(match value {
            Some(value) => Diagnostic::UnobservedValue {
                pack,
                term,
                axis: axis.to_string(),
                value,
            },
            None => Diagnostic::UnobservedAxis {
                pack,
                term,
                axis: axis.to_string(),
            },
        });
    }
}
