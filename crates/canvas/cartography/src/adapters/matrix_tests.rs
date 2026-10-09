// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeMap;

use scenomise::projection::{ProjectionOccurrence, ProjectionValue, RelationshipProvenance};
use serde_json::Value;

use super::*;
use crate::reading::default_graph_reading_registry;

/// The site's repository adapter, which both of its authorities use.
pub(crate) const SITE_REPOSITORY_ADAPTER: &str = "mer3ly.repository-graph/v1";
const SITE_RELATION_ADAPTER: &str = "mer3ly.repository-relation/v1";
const SITE_DERIVATION_ADAPTER: &str = "mer3ly.matrix-derivation/v1";

/// A site graph (`mer3ly.repo-graph/v1`'s nodes and edges) disclosed as a host
/// dataset at `revision`: each node an occurrence with its label, class,
/// status and push time, each edge a relationship whose method is the site's
/// edge provenance.
pub(crate) fn site_dataset(
    graph: &Value,
    resource: &str,
    revision: &str,
) -> (ProjectionDataset, Vec<DisclosedRelationship>) {
    let text = |value: &Value, key: &str| value[key].as_str().unwrap().to_owned();
    // Typed through the dataset, since cartography reaches scenograph's
    // binding type only through scenomise's fields.
    let mut dataset: ProjectionDataset = serde_json::from_value(serde_json::json!({
        "source": {
            "authority": "https://merelyllc.com",
            "domain": "public-repositories",
            "resource": resource,
        },
        "revision": revision,
        "fields": {
            "occurrence_id": "text",
            "label": "text",
            "class": "text",
            "status": "text",
            "pushed_at": "text",
        },
        "occurrences": [],
    }))
    .unwrap();
    let source = dataset.source.clone();
    let occurrences = graph["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| {
            let id = text(node, "id");
            let values = BTreeMap::from([
                (
                    "occurrence_id".to_owned(),
                    ProjectionValue::Text(id.clone()),
                ),
                (
                    "label".to_owned(),
                    ProjectionValue::Text(text(node, "name")),
                ),
                (
                    "class".to_owned(),
                    ProjectionValue::Text(text(node, "class")),
                ),
                (
                    "status".to_owned(),
                    ProjectionValue::Text(text(node, "status")),
                ),
                (
                    "pushed_at".to_owned(),
                    ProjectionValue::Text(text(node, "pushed_at")),
                ),
            ]);
            ProjectionOccurrence {
                occurrence_id: id.clone(),
                source: SourceRef::new(SITE_REPOSITORY_ADAPTER, id),
                values,
            }
        })
        .collect();
    let relationships = graph["edges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|edge| DisclosedRelationship {
            id: text(edge, "id"),
            from_occurrence: text(edge, "source"),
            to_occurrence: text(edge, "target"),
            kind: text(edge, "kind"),
            label: format!(
                "{} {} {}",
                text(edge, "source"),
                text(edge, "kind"),
                text(edge, "target")
            ),
            explanation: "Disclosed by the site authority.".into(),
            provenance: RelationshipProvenance {
                source: source.clone(),
                source_revision: revision.into(),
                method: text(edge, "provenance"),
                method_version: 1,
                provider: "mer3ly authority".into(),
                evidence: vec![SourceRef::new(SITE_RELATION_ADAPTER, text(edge, "id"))],
            },
        })
        .collect();
    dataset.occurrences = occurrences;
    (dataset, relationships)
}

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../fixtures/site_matrix.json")).unwrap()
}

/// The site's adapters, read as this derivation's.
fn site_adapter(adapter: &str) -> &str {
    match adapter {
        SITE_RELATION_ADAPTER => DISCLOSED_RELATIONSHIP_ADAPTER,
        SITE_DERIVATION_ADAPTER => MATRIX_DERIVATION_ADAPTER,
        other => other,
    }
}

#[test]
fn the_site_matrix_reproduces_through_the_shared_derivation() {
    let fixture = fixture();
    let registry = default_graph_reading_registry();
    let data = ["rows", "columns"].map(|role| {
        let axis = &fixture[role];
        let (dataset, relationships) = site_dataset(
            &axis["graph"],
            axis["dataset"].as_str().unwrap(),
            axis["record"].as_str().unwrap(),
        );
        (axis, dataset, relationships)
    });
    // The column axis is the site's `changes` reading of an authority with
    // one revision, whose actors are every occurrence.
    let axes = data
        .each_ref()
        .map(|(axis, dataset, relationships)| ReadingAxis {
            authority: axis["dataset"].as_str().unwrap(),
            record: axis["record"].as_str().unwrap(),
            profile: registry.resolve(axis["reading"].as_str().unwrap()).unwrap(),
            focus: axis["focus"].as_str(),
            dataset,
            relationships,
            previous: None,
            label_field: "label",
        });
    let matrix = project_two_reading_matrix(&axes[0], &axes[1]).unwrap();

    for (axis, role) in [(&matrix.rows, "rows"), (&matrix.columns, "columns")] {
        let expected = fixture[role]["expected_sources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| {
                (
                    entry[0].as_str().unwrap(),
                    entry[1].as_str().unwrap(),
                    entry[2].as_str().unwrap(),
                )
            })
            .collect::<Vec<_>>();
        let actual = axis
            .sources
            .iter()
            .map(|entry| {
                (
                    entry.source.adapter.as_str(),
                    entry.source.id.as_str(),
                    entry.label.as_str(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{role}");
    }

    let expected = fixture["expected_cells"].as_array().unwrap();
    assert_eq!(matrix.cells.len(), expected.len());
    assert_eq!(expected.len(), 11 * 12);
    for (cell, expected) in matrix.cells.iter().zip(expected) {
        let at = format!("{} by {}", cell.row.id, cell.column.id);
        let kind = serde_json::to_value(cell.kind).unwrap();
        assert_eq!(cell.row.id, expected[0].as_str().unwrap(), "{at}");
        assert_eq!(cell.column.id, expected[1].as_str().unwrap(), "{at}");
        assert_eq!(kind, expected[2], "{at}");
        assert_eq!(cell.value, expected[3].as_str().unwrap(), "{at}");
        assert_eq!(cell.description, expected[4].as_str().unwrap(), "{at}");
        assert_eq!(
            cell.source.adapter,
            site_adapter(expected[5].as_str().unwrap()),
            "{at}"
        );
        assert_eq!(cell.source.id, expected[6].as_str().unwrap(), "{at}");
        let contributors = expected[7]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| MatrixContributor {
                authority: entry[0].as_str().unwrap().into(),
                source: SourceRef::new(
                    site_adapter(entry[1].as_str().unwrap()),
                    entry[2].as_str().unwrap(),
                ),
                provenance: entry[3].as_str().unwrap().into(),
            })
            .collect::<Vec<_>>();
        assert_eq!(cell.contributors, contributors, "{at}");
    }
    let count = |kind| matrix.cells.iter().filter(|cell| cell.kind == kind).count();
    assert_eq!(count(MatrixCellKind::Relation), 6);
    assert_eq!(count(MatrixCellKind::IdentityMatch), 2);
    assert_eq!(count(MatrixCellKind::Absence), 124);
}

fn small() -> (ProjectionDataset, Vec<DisclosedRelationship>) {
    let graph = serde_json::json!({
        "nodes": [
            {"id":"mere","name":"Mere","class":"platform","status":"active","pushed_at":"1"},
            {"id":"genet","name":"Genet","class":"platform","status":"active","pushed_at":"2"},
            {"id":"turnstone","name":"Turnstone","class":"product","status":"prototype","pushed_at":"3"}
        ],
        "edges": [
            {"id":"mere-depends-on-genet","source":"mere","target":"genet","kind":"depends_on","provenance":"derived"},
            {"id":"turnstone-hosts-mere","source":"turnstone","target":"mere","kind":"host_for","provenance":"curated"}
        ]
    });
    site_dataset(&graph, "live", "r1")
}

#[test]
fn a_two_reading_matrix_crosses_neighbors_with_the_graph() {
    let (dataset, relationships) = small();
    let registry = default_graph_reading_registry();
    let axis = |reading: &str, focus| ReadingAxis {
        authority: "live",
        record: "rev:1",
        profile: registry.resolve(reading).unwrap(),
        focus,
        dataset: &dataset,
        relationships: &relationships,
        previous: None,
        label_field: "label",
    };
    let matrix =
        project_two_reading_matrix(&axis("neighbors", Some("genet")), &axis("graph", None))
            .unwrap();
    assert_eq!(
        matrix
            .rows
            .sources
            .iter()
            .map(|entry| entry.label.as_str())
            .collect::<Vec<_>>(),
        ["Mere", "Genet"]
    );
    assert_eq!(matrix.cells.len(), 6);
    let relation = matrix.cell(0, 1).unwrap();
    assert_eq!(relation.kind, MatrixCellKind::Relation);
    assert_eq!(relation.description, "1 relation from Mere to Genet");
    assert_eq!(relation.contributors[0].provenance, "derived");
    // A relation from outside the focus reading still reaches its cell.
    assert_eq!(
        matrix.cell(0, 0).unwrap().kind,
        MatrixCellKind::IdentityMatch
    );

    assert_eq!(
        project_two_reading_matrix(&axis("graph", None), &axis("graph", None)).unwrap_err(),
        MatrixReadingError::Matrix(MatrixError::IdenticalAxes)
    );
}

#[test]
fn axis_readings_are_actor_readings_with_their_inputs() {
    let (dataset, relationships) = small();
    let registry = default_graph_reading_registry();
    let axis = |reading: &str, focus| ReadingAxis {
        authority: "live",
        record: "rev:1",
        profile: registry.resolve(reading).unwrap(),
        focus,
        dataset: &dataset,
        relationships: &relationships,
        previous: None,
        label_field: "label",
    };
    let graph = axis("graph", None);
    assert_eq!(
        project_two_reading_matrix(&axis("matrix", None), &graph).unwrap_err(),
        MatrixReadingError::NotAnActorReading {
            axis: MatrixRole::Rows,
            reading: "matrix".into()
        }
    );
    assert_eq!(
        project_two_reading_matrix(&graph, &axis("neighbors", Some("absent"))).unwrap_err(),
        MatrixReadingError::MissingFocus {
            axis: MatrixRole::Columns
        }
    );
    // Without a predecessor, the changes reading selects every actor.
    let first =
        project_two_reading_matrix(&axis("neighbors", Some("genet")), &axis("changes", None))
            .unwrap();
    assert_eq!(first.columns.sources.len(), dataset.occurrences.len());
}

#[test]
fn the_two_reading_profile_sits_beside_the_one_authority_matrix() {
    let registry = default_graph_reading_registry();
    let one = registry.resolve("matrix").unwrap();
    assert_eq!(one.actor_scope, ActorScope::All);
    assert_eq!(
        one.description,
        "An exact source-by-target lookup of direct relations."
    );
    let two = registry.resolve(TWO_READING_MATRIX).unwrap();
    assert_eq!(*two, two_reading_matrix_profile());
    assert_eq!(two.surface, ReadingSurface::RelationMatrix);
    assert_eq!(two.emphasis, ReadingEmphasis::Relation);
    assert!(two.default_arrangement.is_none());
    assert_eq!(registry.default_profile().id, "graph");
}
