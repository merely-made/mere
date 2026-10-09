// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The two-reading matrix: two independently produced readings, crossed.
//!
//! One reading supplies the rows and another the columns. Every row meets
//! every column in exactly one cell, and the cell says what is there: one or
//! more relations from the row to the column, the same source on both axes,
//! a disclosed value, or, stated rather than left blank, nothing at all. Every
//! cell names the inputs that produced it, so a reader can follow any cell
//! back to an authority.
//!
//! The derivation is pure. A host reads its own authorities into two
//! [`MatrixAxis`] values and supplies a cell resolver; [`derive_matrix`] owns
//! cell kinds, wording, contributor order, and refusals, so every matrix
//! reads the same however its axes were produced (mer3ly site canvas plan,
//! Rulings 142 and 143). cartography's matrix adapter is one such host, over
//! two graph readings; a contact ledger is another, over a value resolver.

use std::collections::BTreeMap;
use std::fmt;

use sceno::{
    Footprint, InstanceId, ProjectedItem, Rect, Representation, Scene, Size2, SourceRef,
    Transform2, Vec2,
};
use serde::{Deserialize, Serialize};

/// The adapter of the sources this derivation itself names: identity,
/// absence, and value cells.
pub const MATRIX_DERIVATION_ADAPTER: &str = "scenomise.matrix-derivation/v1";

/// The provenance a contributor carries when it is an axis input rather than a
/// relation.
pub const AXIS_INPUT_PROVENANCE: &str = "axis input";

/// The distance between cell origins in [`Matrix::scene`].
pub const MATRIX_CELL_SIZE: f32 = 64.0;

/// One source a reading placed on an axis, with the label a reader sees.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatrixAxisSource {
    pub source: SourceRef,
    pub label: String,
}

/// One reading of one authority, ready to cross with another.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatrixAxis {
    /// The authority the reading read, as the host names it.
    pub authority: String,
    /// The authority record the reading read: a checkpoint, a store.
    pub record: String,
    /// The reading that produced this axis.
    pub reading: String,
    /// The reading's focus, for readings that take one.
    pub focus: Option<String>,
    /// The authority's generation or public revision when it was read.
    pub generation: String,
    /// The reading's sources, in the reading's order.
    pub sources: Vec<MatrixAxisSource>,
}

/// What a cell holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatrixCellKind {
    /// One or more relations run from the row to the column.
    Relation,
    /// The row and the column are the same source.
    IdentityMatch,
    /// Nothing is there. Stated, so a reader is never handed a blank.
    Absence,
    /// The resolver disclosed a value for this row and column.
    Value,
}

/// One input that produced a cell.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatrixContributor {
    /// The authority the input came from.
    pub authority: String,
    /// The input: a relation, or an axis source.
    pub source: SourceRef,
    /// How that authority came to hold it; [`AXIS_INPUT_PROVENANCE`] for an
    /// axis source.
    pub provenance: String,
}

/// One row-by-column cell.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatrixCell {
    pub row: SourceRef,
    pub column: SourceRef,
    /// What the cell stands for: the relations it reports, or a source this
    /// derivation names under [`MATRIX_DERIVATION_ADAPTER`].
    pub source: SourceRef,
    pub kind: MatrixCellKind,
    /// The short text a grid shows.
    pub value: String,
    /// The full sentence a reader hears.
    pub description: String,
    /// The inputs that produced the cell, ordered by authority and source.
    pub contributors: Vec<MatrixContributor>,
}

/// Two readings crossed: every row meets every column once, row-major.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Matrix {
    pub rows: MatrixAxis,
    pub columns: MatrixAxis,
    pub cells: Vec<MatrixCell>,
}

/// What a resolver found at one row and column.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CellReading {
    /// No relation runs from the row to the column. The cell is an identity
    /// match when both are the same source, and an absence otherwise.
    NoRelation,
    /// These relations run from the row to the column. An empty list reads as
    /// [`CellReading::NoRelation`].
    Relations(Vec<MatrixContributor>),
    /// The row discloses this value under the column.
    Value(String),
    /// The row discloses no value under the column.
    NoValue,
}

/// Which axis an error concerns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatrixRole {
    Rows,
    Columns,
}

impl fmt::Display for MatrixRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Rows => "row",
            Self::Columns => "column",
        })
    }
}

/// Why a matrix was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MatrixError {
    /// An axis names no authority, record, or reading.
    UnnamedAxis(MatrixRole),
    /// An axis has no sources.
    EmptyAxis(MatrixRole),
    /// A source appears twice on one axis.
    DuplicateSource { axis: MatrixRole, source: SourceRef },
    /// Both axes are the same reading of the same authority at the same
    /// focus, so the matrix would cross a reading with itself.
    IdenticalAxes,
    /// The resolver could not read a cell.
    Unresolved {
        row: SourceRef,
        column: SourceRef,
        message: String,
    },
}

impl fmt::Display for MatrixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnnamedAxis(axis) => {
                write!(
                    f,
                    "the {axis} axis needs an authority, a record, and a reading"
                )
            },
            Self::EmptyAxis(axis) => write!(f, "the {axis} axis produced no sources"),
            Self::DuplicateSource { axis, source } => write!(
                f,
                "the {axis} axis lists {}:{} twice",
                source.adapter, source.id
            ),
            Self::IdenticalAxes => {
                write!(f, "matrix axes must be independently produced readings")
            },
            Self::Unresolved {
                row,
                column,
                message,
            } => write!(
                f,
                "could not read the cell at {} by {}: {message}",
                row.id, column.id
            ),
        }
    }
}

impl std::error::Error for MatrixError {}

/// Cross two readings into a matrix.
///
/// `resolve` is asked once per cell, row-major, and reports relations, a
/// value, or nothing; everything a reader sees is decided here. Axes that are
/// the same reading of the same authority at the same focus are refused, as
/// are empty or unnamed axes and a source listed twice on one axis.
pub fn derive_matrix(
    rows: MatrixAxis,
    columns: MatrixAxis,
    mut resolve: impl FnMut(&MatrixAxisSource, &MatrixAxisSource) -> Result<CellReading, String>,
) -> Result<Matrix, MatrixError> {
    for (axis, role) in [(&rows, MatrixRole::Rows), (&columns, MatrixRole::Columns)] {
        if [&axis.authority, &axis.record, &axis.reading]
            .iter()
            .any(|name| name.trim().is_empty())
        {
            return Err(MatrixError::UnnamedAxis(role));
        }
    }
    if rows.authority == columns.authority
        && rows.reading == columns.reading
        && rows.focus == columns.focus
    {
        return Err(MatrixError::IdenticalAxes);
    }
    for (axis, role) in [(&rows, MatrixRole::Rows), (&columns, MatrixRole::Columns)] {
        if axis.sources.is_empty() {
            return Err(MatrixError::EmptyAxis(role));
        }
        let mut seen = std::collections::HashSet::new();
        if let Some(repeat) = axis
            .sources
            .iter()
            .find(|entry| !seen.insert(&entry.source))
        {
            return Err(MatrixError::DuplicateSource {
                axis: role,
                source: repeat.source.clone(),
            });
        }
    }

    let mut cells = Vec::with_capacity(rows.sources.len() * columns.sources.len());
    for row in &rows.sources {
        for column in &columns.sources {
            let reading = resolve(row, column).map_err(|message| MatrixError::Unresolved {
                row: row.source.clone(),
                column: column.source.clone(),
                message,
            })?;
            cells.push(cell(&rows, row, &columns, column, reading));
        }
    }
    Ok(Matrix {
        rows,
        columns,
        cells,
    })
}

fn cell(
    rows: &MatrixAxis,
    row: &MatrixAxisSource,
    columns: &MatrixAxis,
    column: &MatrixAxisSource,
    reading: CellReading,
) -> MatrixCell {
    let axis_inputs = || {
        ordered([
            MatrixContributor {
                authority: rows.authority.clone(),
                source: row.source.clone(),
                provenance: AXIS_INPUT_PROVENANCE.into(),
            },
            MatrixContributor {
                authority: columns.authority.clone(),
                source: column.source.clone(),
                provenance: AXIS_INPUT_PROVENANCE.into(),
            },
        ])
    };
    let derived = |kind: &str| {
        SourceRef::new(
            MATRIX_DERIVATION_ADAPTER,
            format!("{kind}:{}:{}", row.source.id, column.source.id),
        )
    };
    let (kind, source, value, description, contributors) = match reading {
        CellReading::Relations(relations) if !relations.is_empty() => {
            let contributors = ordered(relations);
            let count = contributors.len();
            let ids = contributors
                .iter()
                .map(|contributor| contributor.source.id.as_str())
                .collect::<Vec<_>>()
                .join("+");
            (
                MatrixCellKind::Relation,
                SourceRef::new(contributors[0].source.adapter.clone(), ids),
                "relation".to_owned(),
                format!(
                    "{count} relation{} from {} to {}",
                    if count == 1 { "" } else { "s" },
                    row.label,
                    column.label
                ),
                contributors,
            )
        },
        CellReading::Relations(_) | CellReading::NoRelation if row.source == column.source => (
            MatrixCellKind::IdentityMatch,
            SourceRef::new(
                MATRIX_DERIVATION_ADAPTER,
                format!("identity:{}", row.source.id),
            ),
            "same source".to_owned(),
            format!("{} is present in both readings", row.label),
            axis_inputs(),
        ),
        CellReading::Relations(_) | CellReading::NoRelation => (
            MatrixCellKind::Absence,
            derived("absence"),
            "no relation".to_owned(),
            format!("No direct relation from {} to {}", row.label, column.label),
            axis_inputs(),
        ),
        CellReading::Value(value) => (
            MatrixCellKind::Value,
            derived("value"),
            value.clone(),
            format!("{} {}: {value}", row.label, column.label),
            axis_inputs(),
        ),
        CellReading::NoValue => (
            MatrixCellKind::Absence,
            derived("absence"),
            "no value".to_owned(),
            format!("{} has no {}", row.label, column.label),
            axis_inputs(),
        ),
    };
    MatrixCell {
        row: row.source.clone(),
        column: column.source.clone(),
        source,
        kind,
        value,
        description,
        contributors,
    }
}

/// Contributors in authority, then source id, then adapter order; the first
/// of two that name the same input wins.
fn ordered(contributors: impl IntoIterator<Item = MatrixContributor>) -> Vec<MatrixContributor> {
    let mut by_key = BTreeMap::new();
    for contributor in contributors {
        by_key
            .entry((
                contributor.authority.clone(),
                contributor.source.id.clone(),
                contributor.source.adapter.clone(),
            ))
            .or_insert(contributor);
    }
    by_key.into_values().collect()
}

impl Matrix {
    /// The cell at `row` and `column`, by axis index.
    pub fn cell(&self, row: usize, column: usize) -> Option<&MatrixCell> {
        if column >= self.columns.sources.len() {
            return None;
        }
        self.cells.get(row * self.columns.sources.len() + column)
    }

    /// The scene instance of the cell at `index` in [`Matrix::cells`].
    ///
    /// The scene places row headings first, then column headings, then cells,
    /// so one source backs both its headings when it is on both axes.
    pub fn cell_instance(&self, index: usize) -> InstanceId {
        InstanceId((self.rows.sources.len() + self.columns.sources.len() + index) as u32)
    }

    /// The scene instance of the row heading at `index`.
    pub fn row_instance(&self, index: usize) -> InstanceId {
        InstanceId(index as u32)
    }

    /// The scene instance of the column heading at `index`.
    pub fn column_instance(&self, index: usize) -> InstanceId {
        InstanceId((self.rows.sources.len() + index) as u32)
    }

    /// Lay the matrix out as a scene: headings on the top and left, one
    /// square per cell, each carrying a `matrix.value` channel that is 0 for
    /// an absence and 1 otherwise. `generation` identifies the two readings'
    /// inputs, which the host knows and this derivation does not.
    pub fn scene(&self, generation: u64) -> Scene {
        let mut scene = Scene::new();
        scene.generation = generation;
        let footprint = Footprint::Rect {
            size: Size2::new(MATRIX_CELL_SIZE - 8.0, MATRIX_CELL_SIZE - 8.0),
        };
        let push = |scene: &mut Scene,
                    source: &SourceRef,
                    at: (usize, usize),
                    kind: &str,
                    layer: i16,
                    channels: Vec<(String, f32)>| {
            let source = scene.intern_source(source.clone());
            scene.items.push(ProjectedItem {
                source,
                space: Scene::WORLD,
                transform: Transform2::translation(
                    at.0 as f32 * MATRIX_CELL_SIZE,
                    at.1 as f32 * MATRIX_CELL_SIZE,
                ),
                footprint: footprint.clone(),
                representation: Representation::Open { kind: kind.into() },
                layer,
                visible: true,
                hit: None,
                channels,
            });
        };
        for (index, row) in self.rows.sources.iter().enumerate() {
            push(
                &mut scene,
                &row.source,
                (0, index + 1),
                "matrix.row-heading",
                1,
                Vec::new(),
            );
        }
        for (index, column) in self.columns.sources.iter().enumerate() {
            push(
                &mut scene,
                &column.source,
                (index + 1, 0),
                "matrix.column-heading",
                1,
                Vec::new(),
            );
        }
        let width = self.columns.sources.len();
        for (index, cell) in self.cells.iter().enumerate() {
            let kind = match cell.kind {
                MatrixCellKind::Relation => "matrix.relation-cell",
                MatrixCellKind::IdentityMatch => "matrix.identity-cell",
                MatrixCellKind::Absence => "matrix.absence-cell",
                MatrixCellKind::Value => "matrix.value-cell",
            };
            let value = if cell.kind == MatrixCellKind::Absence {
                0.0
            } else {
                1.0
            };
            push(
                &mut scene,
                &cell.source,
                (index % width + 1, index / width + 1),
                kind,
                0,
                vec![("matrix.value".into(), value)],
            );
        }
        scene.bounds = Rect::new(
            Vec2::ZERO,
            Size2::new(
                (width as f32 + 1.0) * MATRIX_CELL_SIZE,
                (self.rows.sources.len() as f32 + 1.0) * MATRIX_CELL_SIZE,
            ),
        );
        scene
    }
}

#[cfg(test)]
#[path = "matrix_tests.rs"]
mod tests;
