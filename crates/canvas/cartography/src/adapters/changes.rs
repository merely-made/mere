// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The `changes` reading over a host-supplied history.
//!
//! cartography declares the adjacent-revision reading
//! ([`ActorScope::AdjacentRevision`], emphasising [`ReadingEmphasis::Change`]);
//! `scenomise`'s history evaluator computes it (mer3ly site canvas plan,
//! Rulings 14, 19 and 142-145). This adapter checks that a profile is that
//! reading and hands it a revision and its predecessor from a
//! `scenomise.host-dataset/v2` history, with the fields the history declares.
//! It reads no [`crate::IntelligenceSignals`].

use std::fmt;

pub use scenomise::history::{
    Change, ComparedFields, OccurrenceChange, RelationshipChange, RevisionChanges, RevisionView,
    classify_revisions,
};
pub use scenomise::host_dataset::{
    HOST_DATASET_SCHEMA_V2, HostDatasetRevisionV2, HostDatasetV2, parse_host_history,
};

use crate::adapters::matrix::ReadingAxis;
use crate::reading::{ActorScope, GraphReadingProfile, ReadingEmphasis};

/// Why the changes reading could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChangesReadingError {
    /// The profile is not an adjacent-revision reading.
    NotAChangesReading { reading: String },
    /// The history has no revision at this index.
    NoRevision { index: usize },
}

impl fmt::Display for ChangesReadingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAChangesReading { reading } => {
                write!(f, "reading {reading} does not compare adjacent revisions")
            },
            Self::NoRevision { index } => write!(f, "the history has no revision {index}"),
        }
    }
}

impl std::error::Error for ChangesReadingError {}

/// Read `profile` over the history's current revision and its predecessor.
pub fn read_changes(
    profile: &GraphReadingProfile,
    history: &HostDatasetV2,
) -> Result<RevisionChanges, ChangesReadingError> {
    read_changes_at(profile, history, history.revisions.len() - 1)
}

/// Read `profile` over the revision at `index` and the one before it, as a
/// checkpoint slider does. The first revision has no predecessor, so every
/// occurrence in it is added.
pub fn read_changes_at(
    profile: &GraphReadingProfile,
    history: &HostDatasetV2,
    index: usize,
) -> Result<RevisionChanges, ChangesReadingError> {
    if profile.actor_scope != ActorScope::AdjacentRevision
        || profile.emphasis != ReadingEmphasis::Change
    {
        return Err(ChangesReadingError::NotAChangesReading {
            reading: profile.id.clone(),
        });
    }
    history
        .changes_at(index)
        .ok_or(ChangesReadingError::NoRevision { index })
}

impl<'a> ReadingAxis<'a> {
    /// A matrix axis over a history's current revision, with its predecessor
    /// for an adjacent-revision reading.
    pub fn of_history(
        authority: &'a str,
        record: &'a str,
        profile: &'a GraphReadingProfile,
        focus: Option<&'a str>,
        history: &'a HostDatasetV2,
        label_field: &'a str,
    ) -> Self {
        let current = history.current();
        let previous = history
            .revisions
            .len()
            .checked_sub(2)
            .map(|index| RevisionView::from(&history.revisions[index]));
        Self {
            authority,
            record,
            profile,
            focus,
            dataset: &current.dataset,
            relationships: &current.relationships,
            previous,
            label_field,
        }
    }
}

#[cfg(test)]
#[path = "changes_tests.rs"]
mod tests;
