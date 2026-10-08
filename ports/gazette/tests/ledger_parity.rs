// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Gazette's three Ledger tests, reproduced through the shared matrix
//! derivation (`scenomise::matrix`) and the frozen grid, without the Ledger's
//! own projection (mer3ly site canvas plan, Ruling 143). `ledger.rs` retires
//! onto these at the dramatis split; until then each test also checks that
//! the shared result agrees with `project_ledger`'s.

use std::collections::{BTreeMap, BTreeSet};

use chirograph::{
    CoordinatedSelection, Selection, SelectionResolution, SelectionRole, SelectionTarget,
};
use gazette::ledger::{
    CONTACT_ADAPTER, Contact, ContactFacet, FACET_ADAPTER, LedgerCitationInputs,
    LedgerInstanceAddress, LedgerInstanceDelta, project_ledger, resolve_ledger_shelfmark,
};
use graphshell_client::frozen::{FrozenGrid, FrozenGridCell, FrozenGridHeading, FrozenGridRow};
use incipit::{ShelfmarkAuthorityV1, ShelfmarkInputV1, ShelfmarkV1};
use sceno::SourceRef;
use scenomise::matrix::{
    CellReading, Matrix, MatrixAxis, MatrixAxisSource, MatrixCellKind, derive_matrix,
};

fn contacts() -> Vec<Contact> {
    vec![
        Contact {
            id: "ada".into(),
            name: "Ada".into(),
            handle: "@ada@example.net".into(),
            trust: "vouched".into(),
            freshness: "today".into(),
        },
        Contact {
            id: "mina".into(),
            name: "Mina".into(),
            handle: "@mina@example.org".into(),
            trust: "known".into(),
            freshness: "yesterday".into(),
        },
    ]
}

fn facets() -> Vec<ContactFacet> {
    ["handle", "trust", "freshness"]
        .into_iter()
        .map(|id| ContactFacet {
            id: id.into(),
            label: id.into(),
        })
        .collect()
}

/// The Ledger as a two-reading matrix: contacts by facet, each cell the
/// contact's disclosed value under that facet.
fn shared_ledger(generations: (&str, &str)) -> Matrix {
    let contacts = contacts();
    let rows = MatrixAxis {
        authority: "contacts".into(),
        record: "resident:contacts".into(),
        reading: "contacts".into(),
        focus: None,
        generation: generations.0.into(),
        sources: contacts
            .iter()
            .map(|contact| MatrixAxisSource {
                source: SourceRef::new(CONTACT_ADAPTER, &contact.id),
                label: contact.name.clone(),
            })
            .collect(),
    };
    let columns = MatrixAxis {
        authority: "facets".into(),
        record: "gazette:facet-catalog".into(),
        reading: "contact-facets".into(),
        focus: None,
        generation: generations.1.into(),
        sources: facets()
            .iter()
            .map(|facet| MatrixAxisSource {
                source: SourceRef::new(FACET_ADAPTER, &facet.id),
                label: facet.label.clone(),
            })
            .collect(),
    };
    derive_matrix(rows, columns, |row, column| {
        let contact = contacts
            .iter()
            .find(|contact| contact.id == row.source.id)
            .ok_or("unknown contact")?;
        Ok(CellReading::Value(match column.source.id.as_str() {
            "handle" => contact.handle.clone(),
            "trust" => contact.trust.clone(),
            "freshness" => contact.freshness.clone(),
            other => return Err(format!("unsupported Ledger facet {other}")),
        }))
    })
    .expect("the shared Ledger derives")
}

fn grid(matrix: &Matrix) -> FrozenGrid {
    let heading = |entry: &MatrixAxisSource| FrozenGridHeading {
        instance: None,
        source: entry.source.clone(),
        name: entry.label.clone(),
    };
    FrozenGrid::new(
        "Contacts by facet",
        "Contact",
        matrix.columns.sources.iter().map(heading).collect(),
        matrix
            .rows
            .sources
            .iter()
            .enumerate()
            .map(|(row, entry)| FrozenGridRow {
                heading: heading(entry),
                cells: (0..matrix.columns.sources.len())
                    .map(|column| {
                        let cell = matrix.cell(row, column).unwrap();
                        FrozenGridCell {
                            instance: None,
                            source: cell.source.clone(),
                            text: cell.value.clone(),
                            description: cell.description.clone(),
                        }
                    })
                    .collect(),
            })
            .collect(),
    )
    .expect("the Ledger grid")
}

/// The views a contact appears in, as Gazette addresses them.
fn appearances(matrix: &Matrix) -> Vec<LedgerInstanceAddress> {
    matrix
        .rows
        .sources
        .iter()
        .flat_map(|row| {
            [
                ("recipient-picker", "row"),
                ("ledger", "row-heading"),
                ("contact-detail", "card"),
            ]
            .map(|(view, facet)| LedgerInstanceAddress {
                view: view.into(),
                source: row.source.clone(),
                facet: facet.into(),
            })
        })
        .collect()
}

/// The rows a consumer sees under a coordinated selection.
fn visible_contacts<'a>(
    matrix: &'a Matrix,
    selection: &CoordinatedSelection,
    consumer: &str,
) -> Vec<&'a MatrixAxisSource> {
    let targets = selection.targets_for(consumer);
    matrix
        .rows
        .sources
        .iter()
        .filter(|row| {
            targets.as_ref().is_none_or(|targets| {
                targets.contains(&SelectionTarget {
                    kind: "contact".into(),
                    id: row.source.id.clone(),
                })
            })
        })
        .collect()
}

/// A citation of the shared Ledger: each axis's authority, reading and
/// generation, the selection, and the instance deltas.
fn shelfmark(
    matrix: &Matrix,
    selection: &CoordinatedSelection,
    deltas: &[LedgerInstanceDelta],
) -> ShelfmarkV1 {
    let mut shelfmark = ShelfmarkV1::new("ledger");
    for (role, axis) in [("rows", &matrix.rows), ("columns", &matrix.columns)] {
        shelfmark.inputs.insert(
            role.into(),
            ShelfmarkInputV1 {
                authority: ShelfmarkAuthorityV1 {
                    adapter: format!("gazette.{}/v1", axis.authority),
                    record: axis.record.clone(),
                },
                reading: axis.reading.clone(),
                reading_parameters: None,
                arrangement: None,
                expects_generation: axis.generation.clone(),
            },
        );
    }
    shelfmark.delta.insert(
        "selection".into(),
        serde_json::to_string(selection).unwrap(),
    );
    shelfmark.delta.insert(
        "gazette.instances".into(),
        serde_json::to_string(deltas).unwrap(),
    );
    shelfmark.validate().expect("valid shelfmark");
    shelfmark
}

/// Gazette's `ledger_replays_matrix_and_repeated_instance_receipts`.
#[test]
fn ledger_replays_matrix_and_repeated_instance_receipts() {
    let matrix = shared_ledger(("contacts-7", "facets-2"));
    assert_eq!(matrix.cells.len(), 6);
    assert!(matrix.cells.iter().all(|cell| cell.contributors.len() == 2));
    assert!(
        matrix
            .cells
            .iter()
            .all(|cell| cell.kind == MatrixCellKind::Value)
    );
    let html = grid(&matrix).to_html("ledger");
    assert!(html.contains("<caption"));
    assert!(html.contains("scope=\"row\""));
    assert!(html.contains("scope=\"col\""));

    let ada = SourceRef::new(CONTACT_ADAPTER, "ada");
    let appearances = appearances(&matrix);
    let ada_appearances = appearances
        .iter()
        .filter(|appearance| appearance.source == ada)
        .collect::<Vec<_>>();
    assert_eq!(ada_appearances.len(), 3);
    assert_eq!(
        ada_appearances
            .iter()
            .map(|appearance| appearance.view.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        3
    );

    // The shared cells are the Ledger's: the same crossings, values, and
    // contributing authorities and sources, in the same order.
    let ledger = project_ledger(
        &contacts(),
        &facets(),
        CoordinatedSelection::new(SelectionResolution::Crossfilter),
    )
    .unwrap();
    assert_eq!(ledger.cells.len(), matrix.cells.len());
    for (theirs, ours) in ledger.cells.iter().zip(&matrix.cells) {
        assert_eq!(theirs.contact, ours.row);
        assert_eq!(theirs.facet, ours.column);
        assert_eq!(theirs.value, ours.value);
        assert_eq!(
            theirs
                .contributors
                .iter()
                .map(|contributor| (&contributor.authority, &contributor.source))
                .collect::<Vec<_>>(),
            ours.contributors
                .iter()
                .map(|contributor| (&contributor.authority, &contributor.source))
                .collect::<Vec<_>>()
        );
    }
    assert_eq!(ledger.appearances, appearances);
}

/// Gazette's `ledger_and_recipient_picker_crossfilter_without_private_coordination`.
#[test]
fn ledger_and_recipient_picker_crossfilter_without_private_coordination() {
    let matrix = shared_ledger(("contacts-7", "facets-2"));
    let mut selection = CoordinatedSelection::new(SelectionResolution::Crossfilter);
    selection.set(
        SelectionRole::Brush,
        Selection::one("ledger", "contact", "ada"),
    );
    selection.set(
        SelectionRole::Filter,
        Selection::one("recipient-picker", "contact", "mina"),
    );
    assert_eq!(
        visible_contacts(&matrix, &selection, "ledger")[0].source.id,
        "mina"
    );
    assert_eq!(
        visible_contacts(&matrix, &selection, "recipient-picker")[0]
            .source
            .id,
        "ada"
    );
    assert!(selection.remove("recipient-picker"));
    assert_eq!(
        visible_contacts(&matrix, &selection, "ledger").len(),
        contacts().len()
    );
}

/// Gazette's `ledger_shelfmark_checks_both_authorities_and_instance_delta`.
#[test]
fn ledger_shelfmark_checks_both_authorities_and_instance_delta() {
    let matrix = shared_ledger(("contacts-7", "facets-2"));
    let selection = CoordinatedSelection::new(SelectionResolution::Crossfilter);
    let deltas = [LedgerInstanceDelta {
        instance: LedgerInstanceAddress {
            view: "contact-detail".into(),
            source: SourceRef::new(CONTACT_ADAPTER, "ada"),
            facet: "card".into(),
        },
        visible: false,
    }];
    let shelfmark = shelfmark(&matrix, &selection, &deltas);
    let receipt = resolve_ledger_shelfmark(
        &shelfmark,
        &BTreeMap::from([
            ("columns".into(), "facets-2".into()),
            ("rows".into(), "contacts-7".into()),
        ]),
    )
    .expect("the shared citation resolves");
    assert_eq!(receipt.input_generations.len(), 2);
    assert_eq!(receipt.honored_instance_deltas, 1);
    let wire = serde_json::to_string(&shelfmark).expect("stable wire");
    assert_eq!(
        serde_json::from_str::<ShelfmarkV1>(&wire).expect("decode"),
        shelfmark
    );

    // The citation is the one Gazette's Ledger writes for the same inputs.
    let ledger = project_ledger(&contacts(), &facets(), selection).unwrap();
    let theirs = ledger
        .shelfmark(
            LedgerCitationInputs {
                contacts_record: "resident:contacts".into(),
                contacts_generation: "contacts-7".into(),
                facets_record: "gazette:facet-catalog".into(),
                facets_generation: "facets-2".into(),
            },
            &deltas,
        )
        .unwrap();
    assert_eq!(theirs, shelfmark);
}
