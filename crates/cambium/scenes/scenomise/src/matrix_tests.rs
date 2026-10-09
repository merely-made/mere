// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

fn source(id: &str) -> SourceRef {
    SourceRef::new("host.node", id)
}

fn axis(authority: &str, reading: &str, ids: &[(&str, &str)]) -> MatrixAxis {
    MatrixAxis {
        authority: authority.into(),
        record: format!("{authority}:1"),
        reading: reading.into(),
        focus: None,
        generation: "1".into(),
        sources: ids
            .iter()
            .map(|(id, label)| MatrixAxisSource {
                source: source(id),
                label: (*label).into(),
            })
            .collect(),
    }
}

fn relation(authority: &str, id: &str, provenance: &str) -> MatrixContributor {
    MatrixContributor {
        authority: authority.into(),
        source: SourceRef::new("host.relation", id),
        provenance: provenance.into(),
    }
}

/// Rows a, b; columns b, c. a relates to c twice, through two authorities.
fn relations_matrix() -> Matrix {
    derive_matrix(
        axis("live", "neighbors", &[("a", "Alpha"), ("b", "Beta")]),
        axis("live", "graph", &[("b", "Beta"), ("c", "Gamma")]),
        |row, column| {
            Ok(match (row.source.id.as_str(), column.source.id.as_str()) {
                ("a", "c") => CellReading::Relations(vec![
                    relation("live", "a-cites-c", "curated"),
                    relation("archive", "a-cites-c", "derived"),
                    relation("live", "a-cites-c", "repeated"),
                ]),
                ("b", "c") => CellReading::Relations(Vec::new()),
                _ => CellReading::NoRelation,
            })
        },
    )
    .unwrap()
}

#[test]
fn every_row_meets_every_column_once_row_major() {
    let matrix = relations_matrix();
    assert_eq!(matrix.cells.len(), 4);
    for (index, cell) in matrix.cells.iter().enumerate() {
        assert_eq!(cell.row, matrix.rows.sources[index / 2].source);
        assert_eq!(cell.column, matrix.columns.sources[index % 2].source);
    }
    assert_eq!(matrix.cell(1, 0).unwrap().column, source("b"));
    assert!(matrix.cell(0, 2).is_none());
}

#[test]
fn cells_say_relation_identity_or_absence() {
    let matrix = relations_matrix();
    let kinds = matrix
        .cells
        .iter()
        .map(|cell| cell.kind)
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        [
            MatrixCellKind::Absence,
            MatrixCellKind::Relation,
            MatrixCellKind::IdentityMatch,
            // An empty relation list is no relation.
            MatrixCellKind::Absence,
        ]
    );
    let absence = matrix.cell(0, 0).unwrap();
    assert_eq!(absence.value, "no relation");
    assert_eq!(absence.description, "No direct relation from Alpha to Beta");
    assert_eq!(
        absence.source,
        SourceRef::new(MATRIX_DERIVATION_ADAPTER, "absence:a:b")
    );
    let identity = matrix.cell(1, 0).unwrap();
    assert_eq!(identity.value, "same source");
    assert_eq!(identity.description, "Beta is present in both readings");
    assert_eq!(
        identity.source,
        SourceRef::new(MATRIX_DERIVATION_ADAPTER, "identity:b")
    );
    // One authority on both axes and one source: one contributor.
    assert_eq!(identity.contributors.len(), 1);
    assert_eq!(identity.contributors[0].provenance, AXIS_INPUT_PROVENANCE);
}

#[test]
fn relation_contributors_are_ordered_and_deduplicated() {
    let matrix = relations_matrix();
    let cell = matrix.cell(0, 1).unwrap();
    assert_eq!(
        cell.contributors,
        vec![
            relation("archive", "a-cites-c", "derived"),
            relation("live", "a-cites-c", "curated"),
        ]
    );
    assert_eq!(cell.value, "relation");
    assert_eq!(cell.description, "2 relations from Alpha to Gamma");
    assert_eq!(
        cell.source,
        SourceRef::new("host.relation", "a-cites-c+a-cites-c")
    );
}

#[test]
fn value_cells_carry_both_axis_inputs() {
    let matrix = derive_matrix(
        axis("contacts", "contacts", &[("ada", "Ada")]),
        axis(
            "facets",
            "contact-facets",
            &[("handle", "handle"), ("pgp", "pgp")],
        ),
        |_, column| {
            Ok(match column.source.id.as_str() {
                "handle" => CellReading::Value("@ada@example.net".into()),
                _ => CellReading::NoValue,
            })
        },
    )
    .unwrap();
    let value = matrix.cell(0, 0).unwrap();
    assert_eq!(value.kind, MatrixCellKind::Value);
    assert_eq!(value.value, "@ada@example.net");
    assert_eq!(value.description, "Ada handle: @ada@example.net");
    assert_eq!(
        value
            .contributors
            .iter()
            .map(|contributor| contributor.authority.as_str())
            .collect::<Vec<_>>(),
        ["contacts", "facets"]
    );
    let missing = matrix.cell(0, 1).unwrap();
    assert_eq!(missing.kind, MatrixCellKind::Absence);
    assert_eq!(missing.value, "no value");
    assert_eq!(missing.description, "Ada has no pgp");
}

#[test]
fn refuses_identical_axes() {
    let rows = axis("live", "graph", &[("a", "Alpha")]);
    assert_eq!(
        derive_matrix(rows.clone(), rows.clone(), |_, _| Ok(
            CellReading::NoRelation
        )),
        Err(MatrixError::IdenticalAxes)
    );
    // Another focus is another reading.
    let mut focused = rows.clone();
    focused.focus = Some("a".into());
    assert!(derive_matrix(rows, focused, |_, _| Ok(CellReading::NoRelation)).is_ok());
}

#[test]
fn refuses_empty_unnamed_and_repeated_axes() {
    let rows = axis("live", "graph", &[("a", "Alpha")]);
    let empty = axis("other", "graph", &[]);
    assert_eq!(
        derive_matrix(rows.clone(), empty, |_, _| Ok(CellReading::NoRelation)),
        Err(MatrixError::EmptyAxis(MatrixRole::Columns))
    );
    let mut unnamed = axis("other", "graph", &[("a", "Alpha")]);
    unnamed.record = " ".into();
    assert_eq!(
        derive_matrix(unnamed, rows.clone(), |_, _| Ok(CellReading::NoRelation)),
        Err(MatrixError::UnnamedAxis(MatrixRole::Rows))
    );
    let repeated = axis("other", "graph", &[("a", "Alpha"), ("a", "Again")]);
    assert_eq!(
        derive_matrix(rows, repeated, |_, _| Ok(CellReading::NoRelation)),
        Err(MatrixError::DuplicateSource {
            axis: MatrixRole::Columns,
            source: source("a"),
        })
    );
}

#[test]
fn a_resolver_failure_names_its_cell() {
    let error = derive_matrix(
        axis("contacts", "contacts", &[("ada", "Ada")]),
        axis("facets", "contact-facets", &[("shoe", "shoe size")]),
        |_, column| Err(format!("unsupported facet {}", column.source.id)),
    )
    .unwrap_err();
    assert_eq!(
        error,
        MatrixError::Unresolved {
            row: source("ada"),
            column: source("shoe"),
            message: "unsupported facet shoe".into(),
        }
    );
}

#[test]
fn the_scene_places_headings_then_cells() {
    let matrix = relations_matrix();
    let scene = matrix.scene(7);
    assert_eq!(scene.generation, 7);
    assert_eq!(scene.items.len(), 2 + 2 + 4);
    // Beta is on both axes and backs both headings with one source.
    let beta = scene
        .sources
        .iter()
        .position(|s| *s == source("b"))
        .unwrap();
    assert_eq!(
        scene
            .items
            .iter()
            .filter(|item| item.source.0 as usize == beta)
            .count(),
        2
    );
    let relation = &scene.items[matrix.cell_instance(1).0 as usize];
    assert_eq!(
        relation.representation,
        Representation::Open {
            kind: "matrix.relation-cell".into()
        }
    );
    assert_eq!(relation.transform, Transform2::translation(128.0, 64.0));
    assert_eq!(relation.channels, vec![("matrix.value".into(), 1.0)]);
    let absence = &scene.items[matrix.cell_instance(0).0 as usize];
    assert_eq!(absence.channels, vec![("matrix.value".into(), 0.0)]);
    assert_eq!(
        scene.items[matrix.column_instance(1).0 as usize].transform,
        Transform2::translation(128.0, 0.0)
    );
    assert_eq!(
        scene.items[matrix.row_instance(1).0 as usize].transform,
        Transform2::translation(0.0, 128.0)
    );
    assert_eq!(scene.bounds.size, Size2::new(192.0, 192.0));
}
