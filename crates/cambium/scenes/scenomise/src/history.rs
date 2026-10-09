// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Source history: what changed between two revisions of one source.
//!
//! The evaluator behind cartography's adjacent-revision (`changes`) reading
//! (mer3ly site canvas plan, Rulings 14, 19 and 142-145). Given a revision and
//! its predecessor, every occurrence is added, updated, stable, or removed,
//! and so is every relationship:
//!
//! - **added**: present now, absent before;
//! - **updated**: present in both, with a compared field's value or an
//!   incident relationship that differs;
//! - **stable**: present in both, with neither;
//! - **removed**: present before, absent now. A removed relationship is
//!   reported only when both of its endpoints still exist in the current
//!   revision (Ruling 147b): one that vanished with an endpoint is told by
//!   that endpoint's removal.
//!
//! Occurrences are matched by `occurrence_id` and relationships by `id`. By
//! default every disclosed field is compared (Ruling 145); a host may narrow
//! that to the fields its readers treat as change. A relationship is compared
//! by default on everything it discloses except the revision it was
//! disclosed at, since that differs between any two revisions by
//! construction; a host may narrow that too, to parts named in
//! [`RELATIONSHIP_FIELDS`] (Ruling 146).
//!
//! Current occurrences come first, in the current dataset's order, then
//! removed ones in the predecessor's order; relationships likewise.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use sceno::SourceRef;
use serde::{Deserialize, Serialize};

use crate::host_dataset::{HostDatasetRevisionV2, HostDatasetV2};
use crate::projection::{DisclosedRelationship, ProjectionDataset, ProjectionValue};
use scenograph::PublicSourceRevision;

/// How an occurrence or relationship changed between two revisions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Change {
    Added,
    Updated,
    Stable,
    Removed,
}

/// Which fields decide whether an occurrence was updated.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ComparedFields {
    /// Every disclosed field.
    #[default]
    All,
    /// Only these fields.
    Only(BTreeSet<String>),
}

impl ComparedFields {
    /// The fields a history declares, or every field when it declares none.
    pub fn of(history: &HostDatasetV2) -> Self {
        match &history.compared_fields {
            Some(fields) => Self::Only(fields.iter().cloned().collect()),
            None => Self::All,
        }
    }

    fn compares(&self, field: &str) -> bool {
        match self {
            Self::All => true,
            Self::Only(fields) => fields.contains(field),
        }
    }
}

/// The parts of a relationship a host may compare (Ruling 146). Its `id` is
/// its identity and always counts; its source binding and source revision
/// never do.
pub const RELATIONSHIP_FIELDS: &[&str] = &[
    "endpoints",
    "kind",
    "label",
    "explanation",
    "provenance.method",
    "provenance.method_version",
    "provenance.provider",
    "provenance.evidence",
];

/// What decides an update: occurrence fields, and the compared parts of
/// incident relationships.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Comparison {
    /// The occurrence fields compared.
    pub fields: ComparedFields,
    /// The relationship parts compared, named from [`RELATIONSHIP_FIELDS`].
    pub relationship_fields: ComparedFields,
}

impl Comparison {
    /// What a history declares, defaulting each half to everything.
    pub fn of(history: &HostDatasetV2) -> Self {
        Self {
            fields: ComparedFields::of(history),
            relationship_fields: match &history.compared_relationship_fields {
                Some(parts) => ComparedFields::Only(parts.iter().cloned().collect()),
                None => ComparedFields::All,
            },
        }
    }
}

/// One revision of a source, as the evaluator reads it.
#[derive(Clone, Copy, Debug)]
pub struct RevisionView<'a> {
    pub dataset: &'a ProjectionDataset,
    pub relationships: &'a [DisclosedRelationship],
}

impl<'a> From<&'a HostDatasetRevisionV2> for RevisionView<'a> {
    fn from(entry: &'a HostDatasetRevisionV2) -> Self {
        Self {
            dataset: &entry.dataset,
            relationships: &entry.relationships,
        }
    }
}

/// How one occurrence changed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OccurrenceChange {
    pub occurrence_id: String,
    pub source: SourceRef,
    pub change: Change,
    /// For an update, the compared fields whose values differ, by name.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<String>,
    /// For an update, whether its incident relationships differ.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub relationships_differ: bool,
}

/// How one relationship changed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipChange {
    pub id: String,
    pub from_occurrence: String,
    pub to_occurrence: String,
    pub change: Change,
}

/// Everything that changed from one revision to the next.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RevisionChanges {
    /// The predecessor, or none when the revision is the first.
    pub previous: Option<PublicSourceRevision>,
    pub current: PublicSourceRevision,
    pub occurrences: Vec<OccurrenceChange>,
    pub relationships: Vec<RelationshipChange>,
}

impl RevisionChanges {
    /// The change recorded for an occurrence.
    pub fn occurrence(&self, id: &str) -> Option<Change> {
        self.occurrences
            .iter()
            .find(|entry| entry.occurrence_id == id)
            .map(|entry| entry.change)
    }

    /// How many occurrences changed each way.
    pub fn counts(&self) -> BTreeMap<Change, usize> {
        let mut counts = BTreeMap::new();
        for entry in &self.occurrences {
            *counts.entry(entry.change).or_default() += 1;
        }
        counts
    }
}

/// Classify `current` against `previous`. With no predecessor every current
/// occurrence and relationship is added.
pub fn classify_revisions(
    previous: Option<RevisionView<'_>>,
    current: RevisionView<'_>,
    comparison: &Comparison,
) -> RevisionChanges {
    let fields = &comparison.fields;
    let signature = |relationship: &DisclosedRelationship| {
        signature(relationship, &comparison.relationship_fields)
    };
    let before = previous
        .map(|view| {
            view.dataset
                .occurrences
                .iter()
                .map(|occurrence| (occurrence.occurrence_id.as_str(), occurrence))
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();
    let before_incident = incident(previous.map_or(&[], |view| view.relationships), &signature);
    let after_incident = incident(current.relationships, &signature);
    let current_ids = current
        .dataset
        .occurrences
        .iter()
        .map(|occurrence| occurrence.occurrence_id.as_str())
        .collect::<HashSet<_>>();

    let mut occurrences = Vec::new();
    for occurrence in &current.dataset.occurrences {
        let id = occurrence.occurrence_id.as_str();
        let Some(prior) = before.get(id) else {
            occurrences.push(OccurrenceChange {
                occurrence_id: id.to_owned(),
                source: occurrence.source.clone(),
                change: Change::Added,
                fields: Vec::new(),
                relationships_differ: false,
            });
            continue;
        };
        let changed = changed_fields(&prior.values, &occurrence.values, fields);
        let relationships_differ = before_incident.get(id) != after_incident.get(id);
        let change = if changed.is_empty() && !relationships_differ {
            Change::Stable
        } else {
            Change::Updated
        };
        occurrences.push(OccurrenceChange {
            occurrence_id: id.to_owned(),
            source: occurrence.source.clone(),
            change,
            fields: changed,
            relationships_differ,
        });
    }
    if let Some(previous) = previous {
        occurrences.extend(
            previous
                .dataset
                .occurrences
                .iter()
                .filter(|occurrence| !current_ids.contains(occurrence.occurrence_id.as_str()))
                .map(|occurrence| OccurrenceChange {
                    occurrence_id: occurrence.occurrence_id.clone(),
                    source: occurrence.source.clone(),
                    change: Change::Removed,
                    fields: Vec::new(),
                    relationships_differ: false,
                }),
        );
    }

    let before_relationships = previous
        .map(|view| {
            view.relationships
                .iter()
                .map(|relationship| (relationship.id.as_str(), relationship))
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();
    let mut relationships = current
        .relationships
        .iter()
        .map(|relationship| RelationshipChange {
            id: relationship.id.clone(),
            from_occurrence: relationship.from_occurrence.clone(),
            to_occurrence: relationship.to_occurrence.clone(),
            change: match before_relationships.get(relationship.id.as_str()) {
                None => Change::Added,
                Some(prior) if signature(prior) != signature(relationship) => Change::Updated,
                Some(_) => Change::Stable,
            },
        })
        .collect::<Vec<_>>();
    if let Some(previous) = previous {
        let current_relationships = current
            .relationships
            .iter()
            .map(|relationship| relationship.id.as_str())
            .collect::<HashSet<_>>();
        relationships.extend(
            previous
                .relationships
                .iter()
                .filter(|relationship| {
                    !current_relationships.contains(relationship.id.as_str())
                        && current_ids.contains(relationship.from_occurrence.as_str())
                        && current_ids.contains(relationship.to_occurrence.as_str())
                })
                .map(|relationship| RelationshipChange {
                    id: relationship.id.clone(),
                    from_occurrence: relationship.from_occurrence.clone(),
                    to_occurrence: relationship.to_occurrence.clone(),
                    change: Change::Removed,
                }),
        );
    }

    RevisionChanges {
        previous: previous.map(|view| view.dataset.revision.clone()),
        current: current.dataset.revision.clone(),
        occurrences,
        relationships,
    }
}

impl HostDatasetV2 {
    /// The current revision classified against its predecessor, comparing
    /// the fields this history declares.
    pub fn changes(&self) -> RevisionChanges {
        self.changes_at(self.revisions.len() - 1)
            .expect("a parsed history has a current revision")
    }

    /// The revision at `index` classified against the one before it.
    pub fn changes_at(&self, index: usize) -> Option<RevisionChanges> {
        let current = self.revisions.get(index)?;
        let previous = index
            .checked_sub(1)
            .and_then(|previous| self.revisions.get(previous));
        Some(classify_revisions(
            previous.map(RevisionView::from),
            current.into(),
            &Comparison::of(self),
        ))
    }
}

fn changed_fields(
    before: &BTreeMap<String, ProjectionValue>,
    after: &BTreeMap<String, ProjectionValue>,
    fields: &ComparedFields,
) -> Vec<String> {
    before
        .keys()
        .chain(after.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|field| fields.compares(field) && before.get(*field) != after.get(*field))
        .cloned()
        .collect()
}

/// A relationship's identity and its compared parts. Its source binding and
/// source revision are never compared.
fn signature(relationship: &DisclosedRelationship, parts: &ComparedFields) -> String {
    let provenance = &relationship.provenance;
    let evidence = provenance
        .evidence
        .iter()
        .map(|source| format!("{}\u{1f}{}", source.adapter, source.id))
        .collect::<Vec<_>>()
        .join("\u{1e}");
    let endpoints = format!(
        "{}\u{1f}{}",
        relationship.from_occurrence, relationship.to_occurrence
    );
    let version = provenance.method_version.to_string();
    let mut signature = relationship.id.clone();
    for (part, value) in [
        ("endpoints", endpoints.as_str()),
        ("kind", &relationship.kind),
        ("label", &relationship.label),
        ("explanation", &relationship.explanation),
        ("provenance.method", &provenance.method),
        ("provenance.method_version", &version),
        ("provenance.provider", &provenance.provider),
        ("provenance.evidence", &evidence),
    ] {
        signature.push_str("\u{1f}\u{1f}");
        if parts.compares(part) {
            signature.push_str(value);
        }
    }
    signature
}

/// Each occurrence's incident relationship signatures, sorted.
fn incident<'a>(
    relationships: &'a [DisclosedRelationship],
    signature: &impl Fn(&DisclosedRelationship) -> String,
) -> HashMap<&'a str, Vec<String>> {
    let mut by_occurrence = HashMap::<&str, Vec<String>>::new();
    for relationship in relationships {
        let signature = signature(relationship);
        for end in [&relationship.from_occurrence, &relationship.to_occurrence] {
            by_occurrence
                .entry(end.as_str())
                .or_default()
                .push(signature.clone());
        }
    }
    for signatures in by_occurrence.values_mut() {
        signatures.sort();
    }
    by_occurrence
}

#[cfg(test)]
#[path = "history_tests.rs"]
mod tests;
