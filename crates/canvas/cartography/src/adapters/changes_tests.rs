// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeMap;

use serde_json::Value;

use super::*;
use crate::adapters::matrix::tests::{SITE_REPOSITORY_ADAPTER, site_dataset};
use crate::adapters::matrix::{MatrixCellKind, project_two_reading_matrix};
use crate::reading::default_graph_reading_registry;

/// Two real checkpoints of the site's repository authority, with the site's
/// own reading of them; see the fixture's note.
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../../cambium/scenes/scenomise/fixtures/site_checkpoints.json"
    ))
    .unwrap()
}

/// The two checkpoints as one v2 history, parsed at the boundary.
fn site_history(compared: Option<&[&str]>) -> HostDatasetV2 {
    let fixture = fixture();
    let revisions = ["previous", "current"]
        .iter()
        .enumerate()
        .map(|(index, side)| {
            let revision = fixture[side]["cursor"]["commit"].as_str().unwrap();
            let (dataset, relationships) =
                site_dataset(&fixture[side]["graph"], "repository-graph", revision);
            HostDatasetRevisionV2 {
                sequence: index as u64 + 1,
                revision: revision.into(),
                dataset,
                relationships,
            }
        })
        .collect();
    let history = HostDatasetV2 {
        schema: HOST_DATASET_SCHEMA_V2.into(),
        revisions,
        compared_fields: compared.map(|fields| fields.iter().map(|f| f.to_string()).collect()),
    };
    parse_host_history(&serde_json::to_string(&history).unwrap()).unwrap()
}

const SITE_FIELDS: &[&str] = &["label", "class", "status", "pushed_at"];

#[test]
fn the_changes_reading_reproduces_the_sites_checkpoint_classes() {
    let registry = default_graph_reading_registry();
    let changes = read_changes(
        registry.resolve("changes").unwrap(),
        &site_history(Some(SITE_FIELDS)),
    )
    .unwrap();
    let expected = fixture()["expected"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            (
                entry[0].as_str().unwrap().to_owned(),
                serde_json::from_value::<Change>(entry[1].clone()).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        changes
            .occurrences
            .iter()
            .map(|entry| (entry.occurrence_id.clone(), entry.change))
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        changes.counts(),
        BTreeMap::from([
            (Change::Added, 1),
            (Change::Updated, 15),
            (Change::Stable, 5),
            (Change::Removed, 5),
        ])
    );
    assert_eq!(
        changes
            .relationships
            .iter()
            .filter(|entry| entry.change == Change::Removed)
            .count(),
        8
    );
}

#[test]
fn only_an_adjacent_revision_reading_reads_changes() {
    let registry = default_graph_reading_registry();
    let history = site_history(None);
    assert_eq!(
        read_changes(registry.resolve("graph").unwrap(), &history).unwrap_err(),
        ChangesReadingError::NotAChangesReading {
            reading: "graph".into()
        }
    );
    let changes = registry.resolve("changes").unwrap();
    assert_eq!(
        read_changes_at(changes, &history, 2).unwrap_err(),
        ChangesReadingError::NoRevision { index: 2 }
    );
    let first = read_changes_at(changes, &history, 0).unwrap();
    assert!(first.previous.is_none());
    assert!(
        first
            .occurrences
            .iter()
            .all(|entry| entry.change == Change::Added)
    );
}

#[test]
fn a_changes_axis_lists_removed_actors_after_current_ones() {
    let registry = default_graph_reading_registry();
    let history = site_history(Some(SITE_FIELDS));
    let rows = ReadingAxis::of_history(
        "live",
        "checkpoint",
        registry.resolve("changes").unwrap(),
        None,
        &history,
        "label",
    );
    let columns = ReadingAxis {
        profile: registry.resolve("graph").unwrap(),
        previous: None,
        ..rows
    };
    let matrix = project_two_reading_matrix(&rows, &columns).unwrap();
    let changes = history.changes();
    assert_eq!(matrix.rows.sources.len(), changes.occurrences.len());
    assert_eq!(
        matrix.columns.sources.len(),
        history.current().dataset.occurrences.len()
    );
    for (source, entry) in matrix.rows.sources.iter().zip(&changes.occurrences) {
        assert_eq!(
            source.source,
            sceno::SourceRef::new(SITE_REPOSITORY_ADAPTER, &entry.occurrence_id)
        );
    }
    // A removed repository keeps its last label, and meets the current
    // graph's columns as absences.
    let removed = changes
        .occurrences
        .iter()
        .position(|entry| entry.change == Change::Removed)
        .unwrap();
    let previous = &history.revisions[0].dataset;
    let label = previous
        .occurrences
        .iter()
        .find(|occurrence| occurrence.occurrence_id == changes.occurrences[removed].occurrence_id)
        .and_then(|occurrence| occurrence.values["label"].text())
        .unwrap();
    assert_eq!(matrix.rows.sources[removed].label, label);
    assert!(
        (0..matrix.columns.sources.len())
            .all(|column| matrix.cell(removed, column).unwrap().kind == MatrixCellKind::Absence)
    );
}
