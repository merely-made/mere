// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Fixture packs, keys, a source and a policy for the loader's tests.

use std::collections::BTreeSet;

use eidetic::pack::{Origin, SignerReview, TrustPolicy, compile_authored, sign_canonical};
use eidetic::schema::TrustEnvelope;
use identity::{Ed25519Keypair, Ed25519PublicKey};

use crate::model::{GraphStratum, RelationKind};
use crate::profile::load::{Limits, LoadRefusal, Profile, load};
use crate::profile::resolve::{Offered, PackRef, PackSource};

pub(crate) const WORKSPACE: &str = include_str!("../fixtures/workspace.toml");
pub(crate) const WORKSPACE_LENS: &str = include_str!("../fixtures/workspace_lens.toml");
pub(crate) const WORKSPACE_ID: &str = "https://mere.computer/packs/test/workspace";
/// Traversal's IRI, minted in the stand-in only (D13).
pub(crate) const TRAVERSAL_IRI: &str = "https://mere.computer/ns/test/workspace#traversal";

pub(crate) fn compiled(text: &str) -> Vec<u8> {
    compile_authored(text)
        .unwrap_or_else(|e| panic!("fixture: {e}"))
        .1
}

pub(crate) fn key(seed: u8) -> Ed25519Keypair {
    Ed25519Keypair::from_seed([seed; 32])
}

/// Packs signed by `key(1)` are the release's; `accepted` lists reviewed (pack, seed) pairs.
pub(crate) struct Review(pub Vec<(String, u8)>);

impl SignerReview for Review {
    fn accepts(&self, pack_id: &str, signer: &Ed25519PublicKey) -> bool {
        self.0
            .iter()
            .any(|(p, s)| p == pack_id && key(*s).public_key() == *signer)
    }
}

pub(crate) fn policy(accepted: Vec<(String, u8)>) -> TrustPolicy<Review> {
    TrustPolicy {
        release_key: key(1).public_key(),
        review: Review(accepted),
    }
}

pub(crate) fn local(canonical: Vec<u8>) -> Offered {
    Offered {
        canonical,
        envelope: TrustEnvelope::self_asserted(),
        origin: Origin::Local,
    }
}

pub(crate) fn signed(canonical: Vec<u8>, seed: u8, origin: Origin) -> Offered {
    let mut envelope = TrustEnvelope::self_asserted();
    envelope
        .signatures
        .push(sign_canonical(&canonical, &key(seed)));
    Offered {
        canonical,
        envelope,
        origin,
    }
}

/// Offers in a fixed order (the proptest shuffles it), and known handlers.
#[derive(Default)]
pub(crate) struct Source {
    pub offers: Vec<Offered>,
    pub handlers: BTreeSet<String>,
}

impl Source {
    pub(crate) fn of(offers: Vec<Offered>) -> Self {
        Self {
            offers,
            handlers: BTreeSet::new(),
        }
    }
}

impl PackSource for Source {
    fn offer(&self, id: &str) -> Vec<Offered> {
        self.offers
            .iter()
            .filter(|o| {
                eidetic::pack::PackHeader::from_canonical(&o.canonical).map_or(true, |h| h.id == id)
            })
            .cloned()
            .collect()
    }
    fn handler_known(&self, id: &str) -> bool {
        self.handlers.contains(id)
    }
}

const PREFIXES: &str = r#"
[prefixes]
pack = "https://example.org/packs/"
ex = "https://example.org/ns#"
axis = "https://mere.computer/ns/axis#"
stratum = "https://mere.computer/ns/stratum#"
"#;

/// A vocabulary pack `pack:<name>` declaring `ex:<term>` rows at a stratum.
pub(crate) fn pack(
    name: &str,
    version: &str,
    requires: &[(&str, &str)],
    terms: &[(&str, &str)],
) -> Vec<u8> {
    let reqs: Vec<String> = requires
        .iter()
        .map(|(p, v)| format!("{{ pack = \"pack:{p}\", version = \"{v}\" }}"))
        .collect();
    let mut text = format!(
        "id = \"pack:{name}\"\nversion = \"{version}\"\nkind = [\"vocabulary\"]\nlicence = \"MPL-2.0\"\n\
         provides = [\"ex:\"]\nrequires = [{}]\n{PREFIXES}",
        reqs.join(", ")
    );
    for (term, stratum) in terms {
        text.push_str(&format!(
            "\n[[vocabulary.terms]]\niri = \"ex:{term}\"\nkind = \"predicate\"\n\
             axes = {{ \"axis:stratum\" = \"stratum:{stratum}\" }}\nlabels = {{}}\n"
        ));
    }
    compiled(&text)
}

/// A lens `pack:<name>` selecting packs and settling terms.
pub(crate) fn lens(name: &str, select: &[(&str, &str)], settle: &[(&str, &str)]) -> Vec<u8> {
    let select: Vec<String> = select
        .iter()
        .map(|(p, v)| format!("{{ pack = \"pack:{p}\", version = \"{v}\" }}"))
        .collect();
    let settle: Vec<String> = settle
        .iter()
        .map(|(t, p)| format!("{{ term = \"ex:{t}\", pack = \"pack:{p}\" }}"))
        .collect();
    compiled(&format!(
        "id = \"pack:{name}\"\nversion = \"1.0.0\"\nkind = [\"lens\"]\nlicence = \"MPL-2.0\"\n{PREFIXES}\n\
         [lens]\nselect = [{}]\nsettle = [{}]\n",
        select.join(", "),
        settle.join(", ")
    ))
}

pub(crate) fn select(name: &str) -> PackRef {
    PackRef::Select(eidetic::pack::Selection {
        pack: format!("https://example.org/packs/{name}"),
        version: "1.0.0".to_string(),
    })
}

pub(crate) fn load_with(lens: &PackRef, source: &Source) -> Result<Profile, LoadRefusal> {
    load(lens, source, &policy(vec![]), &Limits::default())
}

pub(crate) fn workspace_source() -> Source {
    Source::of(vec![
        local(compiled(WORKSPACE)),
        local(compiled(WORKSPACE_LENS)),
    ])
}

pub(crate) fn workspace_lens() -> PackRef {
    PackRef::Select(eidetic::pack::Selection {
        pack: "https://mere.computer/packs/test/workspace-lens".to_string(),
        version: "0.1.0".to_string(),
    })
}

/// The core pack and the workspace stand-in, through the fixture lens.
pub(crate) fn workspace_profile() -> Profile {
    load_with(&workspace_lens(), &workspace_source()).expect("the workspace fixture loads")
}

/// The stratum the pack gives `kind`: its predicate's row, Traversal's minted
/// row, or the core default for an open predicate.
pub(crate) fn pack_stratum(profile: &Profile, kind: RelationKind) -> GraphStratum {
    let iri = match kind {
        RelationKind::Traversal => TRAVERSAL_IRI,
        RelationKind::OpenPredicate => "https://foreign.test/predicate",
        other => other
            .predicate_iri()
            .expect("a sub-kind has a predicate IRI"),
    };
    profile.terms.stratum(iri)
}
