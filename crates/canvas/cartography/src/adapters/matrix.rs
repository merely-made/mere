// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The two-reading matrix as a graph reading.
//!
//! Two [`GraphReadingProfile`]s, each applied to a host-disclosed dataset,
//! become the two axes of `scenomise`'s shared matrix derivation, which owns
//! the cells, their wording, and their refusals (mer3ly site canvas plan,
//! Ruling 142). This adapter does only the reading half: it evaluates each
//! profile's actor scope over its dataset, and it reports the disclosed
//! relationships that run from a row's source to a column's source, from
//! either axis's authority.
//!
//! The readings are evaluated over the host dataset seam (S1's
//! [`ProjectionDataset`] and [`DisclosedRelationship`]) rather than over a
//! kernel graph, because a two-reading matrix crosses authorities a single
//! graph does not hold. Nothing here reads [`crate::IntelligenceSignals`].

use std::collections::{HashMap, HashSet};
use std::fmt;

use sceno::SourceRef;
use scenomise::projection::{DisclosedRelationship, ProjectionDataset, ProjectionOccurrence};

pub use scenomise::matrix::{
    AXIS_INPUT_PROVENANCE, CellReading, MATRIX_CELL_SIZE, MATRIX_DERIVATION_ADAPTER, Matrix,
    MatrixAxis, MatrixAxisSource, MatrixCell, MatrixCellKind, MatrixContributor, MatrixError,
    MatrixRole, derive_matrix,
};

use scenomise::history::{Comparison, RevisionView, classify_revisions};

use crate::reading::{ActorScope, GraphReadingProfile, ReadingEmphasis, ReadingSurface};

/// The id of the two-reading matrix profile.
pub const TWO_READING_MATRIX: &str = "two-reading-matrix";

/// The adapter under which a matrix cell names a disclosed relationship.
pub const DISCLOSED_RELATIONSHIP_ADAPTER: &str = "cartography.disclosed-relationship/v1";

/// The two-reading matrix, beside the one-authority `matrix` reading: rows
/// from one reading and columns from another, each possibly of another
/// authority. Its actor scope is each axis's own, so the profile's is `All`.
pub fn two_reading_matrix_profile() -> GraphReadingProfile {
    GraphReadingProfile {
        id: TWO_READING_MATRIX.to_owned(),
        label: "Two-reading matrix".to_owned(),
        description: "Two independently produced readings crossed: one supplies the rows, \
                      the other the columns, and each cell reports the direct relations \
                      between them."
            .to_owned(),
        actor_scope: ActorScope::All,
        surface: ReadingSurface::RelationMatrix,
        emphasis: ReadingEmphasis::Relation,
        default_arrangement: None,
        arrangement_locked: true,
    }
}

/// One reading of one host dataset, as one matrix axis.
#[derive(Clone, Copy, Debug)]
pub struct ReadingAxis<'a> {
    /// The authority, as the host names it in contributors.
    pub authority: &'a str,
    /// The authority record that was read.
    pub record: &'a str,
    /// The reading, which must select actors (a spatial reading).
    pub profile: &'a GraphReadingProfile,
    /// The focus occurrence, for a focus reading. Required by a focus
    /// reading and never guessed; a host supplies its own default (Ruling
    /// 147a).
    pub focus: Option<&'a str>,
    /// The revision read: the authority's current dataset and relationships.
    pub dataset: &'a ProjectionDataset,
    pub relationships: &'a [DisclosedRelationship],
    /// The revision before it, which an adjacent-revision reading compares
    /// against. Without one, that reading selects every current actor, as
    /// for a first revision. [`ReadingAxis::of_history`] fills it.
    pub previous: Option<RevisionView<'a>>,
    /// The text field whose value labels a source; the occurrence id when
    /// absent.
    pub label_field: &'a str,
}

/// Why a two-reading matrix could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MatrixReadingError {
    /// The axis's profile is itself a matrix, not a reading of actors.
    NotAnActorReading { axis: MatrixRole, reading: String },
    /// A focus reading without a focus that names a disclosed occurrence.
    /// There is deliberately no fallback (site canvas plan, Ruling 147a): a
    /// host that wants a default focus, as the site does, supplies it.
    MissingFocus { axis: MatrixRole },
    /// The shared derivation refused the axes.
    Matrix(MatrixError),
}

impl fmt::Display for MatrixReadingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAnActorReading { axis, reading } => {
                write!(
                    f,
                    "the {axis} axis reading {reading} does not select actors"
                )
            },
            Self::MissingFocus { axis } => {
                write!(f, "the {axis} axis needs a focus that names an occurrence")
            },
            Self::Matrix(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for MatrixReadingError {}

impl From<MatrixError> for MatrixReadingError {
    fn from(error: MatrixError) -> Self {
        Self::Matrix(error)
    }
}

/// Read two axes and cross them through the shared derivation.
///
/// A relation cell reports every disclosed relationship, from either axis's
/// dataset, whose endpoints are the row's and the column's sources.
pub fn project_two_reading_matrix(
    rows: &ReadingAxis<'_>,
    columns: &ReadingAxis<'_>,
) -> Result<Matrix, MatrixReadingError> {
    let row_axis = read_axis(rows, MatrixRole::Rows)?;
    let column_axis = read_axis(columns, MatrixRole::Columns)?;
    let mut relations: HashMap<(SourceRef, SourceRef), Vec<MatrixContributor>> = HashMap::new();
    for input in [rows, columns] {
        let sources = input
            .dataset
            .occurrences
            .iter()
            .map(|occurrence| (occurrence.occurrence_id.as_str(), &occurrence.source))
            .collect::<HashMap<_, _>>();
        for relationship in input.relationships {
            let (Some(from), Some(to)) = (
                sources.get(relationship.from_occurrence.as_str()),
                sources.get(relationship.to_occurrence.as_str()),
            ) else {
                continue;
            };
            relations
                .entry(((*from).clone(), (*to).clone()))
                .or_default()
                .push(MatrixContributor {
                    authority: input.authority.to_owned(),
                    source: SourceRef::new(DISCLOSED_RELATIONSHIP_ADAPTER, &relationship.id),
                    provenance: relationship.provenance.method.clone(),
                });
        }
    }
    Ok(derive_matrix(row_axis, column_axis, |row, column| {
        Ok(
            match relations.get(&(row.source.clone(), column.source.clone())) {
                Some(found) => CellReading::Relations(found.clone()),
                None => CellReading::NoRelation,
            },
        )
    })?)
}

/// Evaluate one axis's actor scope over its dataset.
fn read_axis(input: &ReadingAxis<'_>, role: MatrixRole) -> Result<MatrixAxis, MatrixReadingError> {
    if input.profile.surface != ReadingSurface::Spatial {
        return Err(MatrixReadingError::NotAnActorReading {
            axis: role,
            reading: input.profile.id.clone(),
        });
    }
    let occurrences = &input.dataset.occurrences;
    let sources = match input.profile.actor_scope {
        // The current actors and then the removed ones, as the changes
        // reading lists them; the comparison itself does not choose actors.
        ActorScope::AdjacentRevision => {
            let current = RevisionView {
                dataset: input.dataset,
                relationships: input.relationships,
            };
            let changes = classify_revisions(input.previous, current, &Comparison::default());
            let mut by_id = input
                .previous
                .iter()
                .flat_map(|previous| &previous.dataset.occurrences)
                .chain(occurrences)
                .map(|occurrence| (occurrence.occurrence_id.as_str(), occurrence))
                .collect::<HashMap<_, _>>();
            changes
                .occurrences
                .iter()
                .filter_map(|entry| by_id.remove(entry.occurrence_id.as_str()))
                .map(|occurrence| axis_source(input, occurrence))
                .collect()
        },
        ActorScope::All => occurrences
            .iter()
            .map(|occurrence| axis_source(input, occurrence))
            .collect(),
        ActorScope::FocusAndNeighbors => {
            let focus = input
                .focus
                .filter(|focus| {
                    occurrences
                        .iter()
                        .any(|occurrence| occurrence.occurrence_id == *focus)
                })
                .ok_or(MatrixReadingError::MissingFocus { axis: role })?;
            let mut near = HashSet::from([focus]);
            for relationship in input.relationships {
                if relationship.from_occurrence == focus {
                    near.insert(relationship.to_occurrence.as_str());
                }
                if relationship.to_occurrence == focus {
                    near.insert(relationship.from_occurrence.as_str());
                }
            }
            occurrences
                .iter()
                .filter(|occurrence| near.contains(occurrence.occurrence_id.as_str()))
                .map(|occurrence| axis_source(input, occurrence))
                .collect()
        },
    };
    Ok(MatrixAxis {
        authority: input.authority.to_owned(),
        record: input.record.to_owned(),
        reading: input.profile.id.clone(),
        focus: input.focus.map(str::to_owned),
        generation: input.dataset.revision.as_str().to_owned(),
        sources,
    })
}

fn axis_source(input: &ReadingAxis<'_>, occurrence: &ProjectionOccurrence) -> MatrixAxisSource {
    MatrixAxisSource {
        source: occurrence.source.clone(),
        label: occurrence
            .values
            .get(input.label_field)
            .and_then(|value| value.text())
            .unwrap_or(&occurrence.occurrence_id)
            .to_owned(),
    }
}

#[cfg(test)]
#[path = "matrix_tests.rs"]
pub(crate) mod tests;
