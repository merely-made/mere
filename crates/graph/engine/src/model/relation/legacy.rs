// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The names engines before L7 stored, read as the IRIs they meant (D17).
//! Conversion, not dispatch (D21): a family name is its family term's local
//! name, and a sub-kind name is the local name of the term in that family
//! that matches it, compared without case or separators, so `FileSystem`
//! finds `containment.filesystem`. A family the core pack does not hold (a
//! workspace one) takes the scheme its IRIs were minted by, for the host's
//! own table to recognise.

use serde::Deserialize as _;
use serde::de::{Deserializer, Error as _};

use crate::profile::core::{self, ids};

fn namespace(iri: &str) -> &str {
    iri.rsplit_once('#').map_or("", |(ns, _)| ns)
}

/// `https://mere.computer/ns/family#`, from the generated constants.
fn family_namespace() -> String {
    format!("{}#", namespace(core::iri(ids::FAMILY_SEMANTIC)))
}

/// `https://mere.computer/ns/rel#`, from the generated constants.
fn predicate_namespace() -> String {
    format!("{}#", namespace(core::iri(ids::REL_CITES)))
}

fn folded(name: &str) -> String {
    name.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn kebab(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            out.push('-');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

/// Whether a stored value is an IRI rather than a name.
pub fn is_iri(value: &str) -> bool {
    value.contains(':')
}

/// The family IRI a stored family name (`Containment`) meant.
pub fn family_iri(name: &str) -> String {
    format!("{}{}", family_namespace(), name.to_ascii_lowercase())
}

/// The predicate IRI a stored family and sub-kind name pair meant.
pub fn predicate_iri(family: &str, sub_kind: &str) -> String {
    let semantic = core::iri(ids::FAMILY_SEMANTIC);
    let family_iri = family_iri(family);
    let dotted = (family_iri != semantic).then(|| format!("{}.", family.to_ascii_lowercase()));
    let terms = core::terms();
    if let Some(family_id) = terms.id(&family_iri) {
        let wanted = folded(sub_kind);
        let found = terms.iter().find(|(_, e)| {
            e.family_id == Some(family_id)
                && e.iri.rsplit_once('#').is_some_and(|(_, local)| {
                    let local = dotted
                        .as_deref()
                        .and_then(|d| local.strip_prefix(d))
                        .unwrap_or(local);
                    folded(local) == wanted
                })
        });
        if let Some((_, entry)) = found {
            return entry.iri.clone();
        }
    }
    format!(
        "{}{}{}",
        predicate_namespace(),
        dotted.unwrap_or_default(),
        kebab(sub_kind)
    )
}

/// One externally tagged entry whose value is a string: `{"Family": "Semantic"}`.
pub fn one_entry<'de, D: Deserializer<'de>>(deserializer: D) -> Result<(String, String), D::Error> {
    let map = std::collections::BTreeMap::<String, String>::deserialize(deserializer)?;
    let mut entries = map.into_iter();
    match (entries.next(), entries.next()) {
        (Some(entry), None) => Ok(entry),
        _ => Err(D::Error::custom("a selector holds exactly one entry")),
    }
}

/// An assertion as engines before L7 stored it, `{"Containment": {"sub_kind":
/// "UserFolder"}}`, read as its predicate IRI and label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacyAssertion {
    pub predicate: String,
    pub label: Option<String>,
}

impl<'de> serde::Deserialize<'de> for LegacyAssertion {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Body {
            sub_kind: String,
            #[serde(default)]
            label: Option<String>,
            // Older semantic assertions carried a decay state; it is dropped.
            #[serde(default)]
            #[allow(dead_code)]
            agent_decay_progress: Option<f32>,
        }
        let map = std::collections::BTreeMap::<String, Body>::deserialize(deserializer)?;
        let mut entries = map.into_iter();
        match (entries.next(), entries.next()) {
            (Some((family, body)), None) => Ok(Self {
                predicate: predicate_iri(&family, &body.sub_kind),
                label: body.label,
            }),
            _ => Err(D::Error::custom("an assertion holds exactly one family")),
        }
    }
}

#[cfg(test)]
#[path = "legacy_tests.rs"]
mod tests;
