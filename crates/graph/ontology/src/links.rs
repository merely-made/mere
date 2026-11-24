// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A two-way index over the named relationships in one admitted source.

use crate::{Profile, TermKind};
use mere::profile::core::CORE_PACK_ID;
use std::collections::{BTreeMap, BTreeSet};

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Entry {
    links: Vec<usize>,
    types: Vec<String>,
}

/// The direction and spelling supplied by the record.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RecordLink {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

/// Each statement is stored once. Both endpoints reference the same evidence.
/// Construct a separate index for each source so identical claims from two
/// records retain independent custody and attribution.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RecordIndex {
    claims: Vec<RecordLink>,
    endpoints: BTreeMap<String, Entry>,
    classes: BTreeMap<String, Vec<String>>,
}

impl RecordIndex {
    /// Supply named resource-to-resource statements in their original direction.
    /// Vocabulary membership comes from the locally admitted profile.
    pub fn new<'a>(
        profile: &Profile,
        statements: impl IntoIterator<Item = (&'a str, &'a str, &'a str)>,
    ) -> Self {
        let mut claims = BTreeSet::new();
        let mut memberships = BTreeSet::new();
        for (subject, predicate, object) in statements {
            if predicate == RDF_TYPE {
                if profile
                    .terms
                    .entry(object)
                    .is_some_and(|term| term.kind == TermKind::Class && term.pack != CORE_PACK_ID)
                {
                    memberships.insert((subject.to_owned(), object.to_owned()));
                }
            } else if profile
                .terms
                .entry(predicate)
                .is_some_and(|term| term.kind == TermKind::Predicate && term.pack != CORE_PACK_ID)
            {
                claims.insert(RecordLink {
                    subject: subject.to_owned(),
                    predicate: predicate.to_owned(),
                    object: object.to_owned(),
                });
            }
        }
        let mut index = Self::default();
        for (record, class) in memberships {
            index
                .endpoints
                .entry(record.clone())
                .or_default()
                .types
                .push(class.clone());
            index.classes.entry(class).or_default().push(record);
        }
        for claim in claims {
            let ordinal = index.claims.len();
            index
                .endpoints
                .entry(claim.subject.clone())
                .or_default()
                .links
                .push(ordinal);
            if claim.object != claim.subject {
                index
                    .endpoints
                    .entry(claim.object.clone())
                    .or_default()
                    .links
                    .push(ordinal);
            }
            index.claims.push(claim);
        }
        index
    }

    pub fn records(&self) -> impl ExactSizeIterator<Item = &str> {
        self.endpoints.keys().map(String::as_str)
    }

    pub fn classes(&self) -> impl ExactSizeIterator<Item = &str> {
        self.classes.keys().map(String::as_str)
    }

    pub fn members(&self, class: &str) -> &[String] {
        self.classes.get(class).map_or(&[], Vec::as_slice)
    }

    pub fn types(&self, record: &str) -> &[String] {
        self.endpoints
            .get(record)
            .map_or(&[], |e| e.types.as_slice())
    }

    pub fn links(&self, record: &str) -> impl ExactSizeIterator<Item = &RecordLink> {
        self.endpoints
            .get(record)
            .map_or(&[][..], |e| e.links.as_slice())
            .iter()
            .map(|&i| &self.claims[i])
    }
}
