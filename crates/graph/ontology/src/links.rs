// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A two-way index over the named relationships in one admitted source.

use crate::{Profile, TermKind};
use mere::profile::{core::CORE_PACK_ID, lens::SelectionRecord};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Entry {
    links: Vec<usize>,
    types: Vec<String>,
}

/// The direction and spelling supplied by the record.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize)]
pub struct RecordLink {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

impl RecordLink {
    pub(crate) fn membership(subject: &str, class: &str) -> Self {
        Self {
            subject: subject.into(),
            predicate: RDF_TYPE.into(),
            object: class.into(),
        }
    }
}

/// Each statement is stored once. Both endpoints reference the same evidence.
/// Construct a separate index for each source so identical claims from two
/// records retain independent custody and attribution.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RecordIndex {
    selection: Option<Box<SelectionRecord>>,
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
        Self::from_named(profile, statements)
    }

    /// Read only named resource claims in the default graph of this owner's admitted dataset.
    pub fn from_dataset(profile: &Profile, dataset: &mere::dataset::IndexedDataset) -> Self {
        let statements = dataset.quads(mere::dataset::View::Live).filter_map(|q| {
            if !q.graph_name.is_default_graph() {
                return None;
            }
            let oxrdf::NamedOrBlankNode::NamedNode(subject) = q.subject else {
                return None;
            };
            let oxrdf::Term::NamedNode(object) = q.object else {
                return None;
            };
            Some((
                subject.into_string(),
                q.predicate.into_string(),
                object.into_string(),
            ))
        });
        Self::from_named(profile, statements)
    }

    fn from_named<S: AsRef<str> + Into<String>>(
        profile: &Profile,
        statements: impl IntoIterator<Item = (S, S, S)>,
    ) -> Self {
        let mut claims = BTreeSet::new();
        let mut memberships = BTreeSet::<(String, String)>::new();
        for (subject, predicate, object) in statements {
            if predicate.as_ref() == RDF_TYPE {
                if profile
                    .terms
                    .entry(object.as_ref())
                    .is_some_and(|term| term.kind == TermKind::Class && term.pack != CORE_PACK_ID)
                {
                    memberships.insert((subject.into(), object.into()));
                }
            } else if profile
                .terms
                .entry(predicate.as_ref())
                .is_some_and(|term| term.kind == TermKind::Predicate && term.pack != CORE_PACK_ID)
            {
                claims.insert(RecordLink {
                    subject: subject.into(),
                    predicate: predicate.into(),
                    object: object.into(),
                });
            }
        }
        let mut index = Self {
            selection: Some(Box::new(profile.selection.clone())),
            ..Self::default()
        };
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

    pub(crate) fn built_with(&self, selection: &SelectionRecord) -> bool {
        self.selection.as_deref() == Some(selection)
    }

    pub fn records(&self) -> impl ExactSizeIterator<Item = &str> {
        self.endpoints.keys().map(String::as_str)
    }

    /// The named statements admitted by this profile, each in its original
    /// direction and emitted once. Includes class membership.
    pub fn statements(&self) -> impl Iterator<Item = RecordLink> + '_ {
        self.claims
            .iter()
            .cloned()
            .chain(self.classes.iter().flat_map(|(class, members)| {
                members
                    .iter()
                    .map(move |member| RecordLink::membership(member, class))
            }))
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
