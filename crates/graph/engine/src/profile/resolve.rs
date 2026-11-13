// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Resolution (declared-lens §3.5, D4, D13): requirements to pins. A bare
//! version matches as Cargo's caret does ([`satisfies`]), the highest match
//! wins, and each refusal is typed: a cycle, two versions of one id, an
//! unsatisfied requirement, and `Broken` or an unaccepted signer by origin.
//! The core pack is resolved first and pinned first; the rest follow in
//! dependency order with ties broken by id, so input order never matters.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use eidetic::pack::{
    Admission, Origin, Pin, Requirement, Selection, SignerReview, TrustPolicy, verify_envelope,
};
use eidetic::schema::TrustEnvelope;

use super::lens::AdmittedAs;
use super::load::{Limits, LoadRefusal};
use super::pack::{DecodedPack, decode};

/// A pack the caller offers: its canonical bytes, its signatures and where it came from.
#[derive(Clone, Debug)]
pub struct Offered {
    pub canonical: Vec<u8>,
    pub envelope: TrustEnvelope,
    pub origin: Origin,
}

/// The caller's packs and handlers. The loader does no I/O (§4).
pub trait PackSource {
    /// Every version of the pack `id` the caller can offer, in any order.
    fn offer(&self, id: &str) -> Vec<Offered>;
    /// Whether the host's activation knows the handler `id` (D21).
    fn handler_known(&self, id: &str) -> bool;
}

/// Which lens to load.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackRef {
    /// The core pack, the user-agent lens (D1).
    Core,
    /// The highest offered version of a lens at or above a lower bound.
    Select(Selection),
    /// Exactly this document, as a selection record names it.
    Pinned(Pin),
}

/// Whether `version` satisfies the lower bound `bound`: SemVer-compatible at
/// or above it, Cargo's caret (D13's reading of D4's "lower bounds"). The one
/// place the matching rule lives.
pub fn satisfies(version: &str, bound: &str) -> bool {
    let (Ok(v), Ok(req)) = (
        semver::Version::parse(version),
        semver::VersionReq::parse(&format!("^{bound}")),
    ) else {
        return false;
    };
    req.matches(&v)
}

/// A resolved pack and how it was admitted.
#[derive(Clone, Debug)]
pub(crate) struct Chosen {
    pub pack: DecodedPack,
    pub admitted: AdmittedAs,
}

/// The resolution: packs in order (core first), the lens's id, and the
/// capabilities the packs require of their environment.
pub(crate) struct Resolution {
    pub packs: Vec<Chosen>,
    pub lens: String,
    pub capabilities: BTreeSet<String>,
}

struct Resolver<'a, R> {
    source: &'a dyn PackSource,
    policy: &'a TrustPolicy<R>,
    limits: &'a Limits,
    chosen: BTreeMap<String, Chosen>,
    edges: BTreeMap<String, BTreeSet<String>>,
    capabilities: BTreeSet<String>,
}

pub(crate) fn resolve<R: SignerReview>(
    lens: &PackRef,
    core: DecodedPack,
    source: &dyn PackSource,
    policy: &TrustPolicy<R>,
    limits: &Limits,
) -> Result<Resolution, LoadRefusal> {
    let core_id = core.header.id.clone();
    let mut r = Resolver {
        source,
        policy,
        limits,
        chosen: BTreeMap::new(),
        edges: BTreeMap::new(),
        capabilities: BTreeSet::new(),
    };
    let admitted = r.admit(&core, &TrustEnvelope::self_asserted(), Origin::Local)?;
    r.chosen.insert(
        core_id.clone(),
        Chosen {
            pack: core,
            admitted,
        },
    );
    let lens_id = match lens {
        PackRef::Core => core_id.clone(),
        PackRef::Select(s) => r.require("the caller", &s.pack, &s.version)?,
        PackRef::Pinned(pin) => r.pinned(pin)?,
    };
    let lens_pack = &r.chosen[&lens_id].pack;
    let Some(body) = &lens_pack.lens else {
        return Err(LoadRefusal::NotALens { pack: lens_id });
    };
    let mut queue: VecDeque<(String, Requirement)> = body
        .select
        .iter()
        .map(|s| {
            let req = Requirement::Pack {
                pack: s.pack.clone(),
                version: s.version.clone(),
            };
            (lens_id.clone(), req)
        })
        .collect();
    let mut expanded = BTreeSet::new();
    let mut pending: Vec<String> = r.chosen.keys().cloned().collect();
    loop {
        for id in pending.drain(..) {
            if expanded.insert(id.clone()) {
                let reqs = r.chosen[&id].pack.header.requires.clone();
                queue.extend(reqs.into_iter().map(|q| (id.clone(), q)));
            }
        }
        let Some((from, req)) = queue.pop_front() else {
            break;
        };
        match req {
            Requirement::Capability { capability } => {
                r.capabilities.insert(capability);
            },
            Requirement::Pack { pack, version } => {
                let to = r.require(&from, &pack, &version)?;
                r.edges.entry(from).or_default().insert(to.clone());
                pending.push(to);
            },
        }
    }
    let order = order(&core_id, &r.chosen, &r.edges)?;
    let mut chosen = r.chosen;
    let packs = order.iter().filter_map(|id| chosen.remove(id)).collect();
    Ok(Resolution {
        packs,
        lens: lens_id,
        capabilities: r.capabilities,
    })
}

impl<R: SignerReview> Resolver<'_, R> {
    /// Resolve `id` at `bound` for `from`, choosing it on first sight.
    fn require(&mut self, from: &str, id: &str, bound: &str) -> Result<String, LoadRefusal> {
        if let Some(held) = self.chosen.get(id) {
            let version = &held.pack.header.version;
            if satisfies(version, bound) {
                return Ok(id.to_string());
            }
            let offered = self.decoded(id)?;
            if offered
                .iter()
                .any(|(p, _)| satisfies(&p.header.version, bound))
            {
                return Err(LoadRefusal::TwoVersions {
                    pack: id.to_string(),
                    versions: [version.clone(), bound.to_string()],
                });
            }
            return Err(unsatisfied(from, id, bound));
        }
        let mut matching: Vec<_> = self
            .decoded(id)?
            .into_iter()
            .filter(|(p, _)| satisfies(&p.header.version, bound))
            .collect();
        matching.sort_by_key(|(p, _)| std::cmp::Reverse(version_of(p)));
        let Some((best, offer)) = matching.first() else {
            return Err(unsatisfied(from, id, bound));
        };
        if matching
            .iter()
            .any(|(p, _)| p.header.version == best.header.version && p.pin != best.pin)
        {
            return Err(LoadRefusal::Ambiguous {
                pack: id.to_string(),
                version: best.header.version.clone(),
            });
        }
        self.choose(best.clone(), offer)
    }

    /// Resolve exactly the document `pin` names.
    fn pinned(&mut self, pin: &Pin) -> Result<String, LoadRefusal> {
        if self
            .chosen
            .get(&pin.id)
            .is_some_and(|held| held.pack.pin == *pin)
        {
            return Ok(pin.id.clone());
        }
        let found = self
            .decoded(&pin.id)?
            .into_iter()
            .find(|(p, _)| p.pin == *pin);
        let (pack, offer) =
            found.ok_or_else(|| LoadRefusal::DigestMismatch { pin: pin.clone() })?;
        self.choose(pack, &offer)
    }

    fn choose(&mut self, pack: DecodedPack, offer: &Offered) -> Result<String, LoadRefusal> {
        if self.chosen.len() >= self.limits.max_packs {
            return Err(LoadRefusal::Limit {
                limit: "packs",
                max: self.limits.max_packs,
            });
        }
        let admitted = self.admit(&pack, &offer.envelope, offer.origin)?;
        let id = pack.header.id.clone();
        self.chosen.insert(id.clone(), Chosen { pack, admitted });
        Ok(id)
    }

    /// Every offer of `id`, strictly decoded; an offer of another id is refused.
    fn decoded(&self, id: &str) -> Result<Vec<(DecodedPack, Offered)>, LoadRefusal> {
        self.source
            .offer(id)
            .into_iter()
            .map(|offer| {
                let pack = decode(id, &offer.canonical, self.limits)?;
                if pack.header.id != id {
                    return Err(LoadRefusal::WrongId {
                        requested: id.to_string(),
                        found: pack.header.id,
                    });
                }
                Ok((pack, offer))
            })
            .collect()
    }

    /// Step 3: the COSE verdict, then the policy (D3, D20).
    fn admit(
        &self,
        pack: &DecodedPack,
        envelope: &TrustEnvelope,
        origin: Origin,
    ) -> Result<AdmittedAs, LoadRefusal> {
        let verdict = verify_envelope(&pack.canonical, envelope);
        match self.policy.admit(&pack.header.id, &verdict, origin) {
            Ok(Admission::Release) => Ok(AdmittedAs::Release),
            Ok(Admission::Reviewed(key)) => Ok(AdmittedAs::Reviewed {
                signer: key.to_bytes().iter().map(|b| format!("{b:02x}")).collect(),
            }),
            Ok(Admission::Local) => Ok(AdmittedAs::Local),
            Err(refusal) => Err(LoadRefusal::Trust {
                pack: pack.header.id.clone(),
                refusal,
            }),
        }
    }
}

fn version_of(pack: &DecodedPack) -> semver::Version {
    // Decoding already checked the version is SemVer 2.0.0.
    semver::Version::parse(&pack.header.version).unwrap_or(semver::Version::new(0, 0, 0))
}

fn unsatisfied(from: &str, id: &str, bound: &str) -> LoadRefusal {
    LoadRefusal::Unsatisfied {
        pack: id.to_string(),
        version: bound.to_string(),
        required_by: from.to_string(),
    }
}

/// Dependencies first, the core pack first of all, ties by id; a cycle is refused.
fn order(
    core: &str,
    chosen: &BTreeMap<String, Chosen>,
    edges: &BTreeMap<String, BTreeSet<String>>,
) -> Result<Vec<String>, LoadRefusal> {
    let mut out = vec![core.to_string()];
    let mut done: BTreeSet<&str> = BTreeSet::from([core]);
    let ready = |id: &str, done: &BTreeSet<&str>| {
        edges
            .get(id)
            .is_none_or(|tos| tos.iter().all(|t| done.contains(t.as_str())))
    };
    // `chosen` iterates by id, so the first ready pack is the least id.
    while let Some(next) = chosen
        .keys()
        .find(|id| !done.contains(id.as_str()) && ready(id, &done))
    {
        done.insert(next);
        out.push(next.clone());
    }
    if out.len() < chosen.len() {
        let packs = chosen
            .keys()
            .filter(|k| !done.contains(k.as_str()))
            .cloned()
            .collect();
        return Err(LoadRefusal::Cycle { packs });
    }
    Ok(out)
}
