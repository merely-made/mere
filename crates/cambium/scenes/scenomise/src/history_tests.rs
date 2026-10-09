// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use serde_json::{Value, json};

use super::*;
use crate::host_dataset::{HOST_DATASET_SCHEMA_V2, parse_host_history};

fn source() -> Value {
    json!({ "authority": "host", "domain": "example.graph", "resource": "graph:1" })
}

fn occurrence(id: &str, label: &str, note: &str) -> Value {
    json!({
        "occurrence_id": id,
        "source": { "adapter": "host.graph", "id": id },
        "values": {
            "label": { "kind": "text", "value": label },
            "note": { "kind": "text", "value": note },
        },
    })
}

fn relationship(id: &str, from: &str, to: &str, kind: &str, revision: &str) -> Value {
    json!({
        "id": id,
        "from_occurrence": from,
        "to_occurrence": to,
        "kind": kind,
        "label": format!("{from} {kind} {to}"),
        "explanation": "The host's source records it.",
        "provenance": {
            "source": source(),
            "source_revision": revision,
            "method": "host.links",
            "method_version": 1,
            "provider": "host exporter",
            "evidence": [{ "adapter": "host.graph", "id": from }],
        },
    })
}

fn revision(
    sequence: u64,
    revision: &str,
    occurrences: Vec<Value>,
    relationships: Vec<Value>,
) -> Value {
    json!({
        "sequence": sequence,
        "revision": revision,
        "dataset": {
            "source": source(),
            "revision": revision,
            "fields": { "label": "text", "note": "text" },
            "occurrences": occurrences,
        },
        "relationships": relationships,
    })
}

fn envelope(revisions: Vec<Value>, compared: Option<&[&str]>) -> HostDatasetV2 {
    envelope_comparing(revisions, compared, None)
}

fn envelope_comparing(
    revisions: Vec<Value>,
    compared: Option<&[&str]>,
    relationship_parts: Option<&[&str]>,
) -> HostDatasetV2 {
    let mut value = json!({ "schema": HOST_DATASET_SCHEMA_V2, "revisions": revisions });
    if let Some(fields) = compared {
        value["compared_fields"] = json!(fields);
    }
    if let Some(parts) = relationship_parts {
        value["compared_relationship_fields"] = json!(parts);
    }
    parse_host_history(&value.to_string()).unwrap()
}

/// a's link to c moves to d and its link to b goes; b is renamed; c is
/// removed; d is added; e is untouched; f's note changes and nothing else
/// does.
fn history(compared: Option<&[&str]>) -> HostDatasetV2 {
    envelope(
        vec![
            revision(
                1,
                "r1",
                vec![
                    occurrence("a", "Alpha", ""),
                    occurrence("b", "Beta", ""),
                    occurrence("c", "Gamma", ""),
                    occurrence("e", "Epsilon", ""),
                    occurrence("f", "Phi", "draft"),
                ],
                vec![
                    relationship("a-b", "a", "b", "links", "r1"),
                    relationship("a-c", "a", "c", "links", "r1"),
                    relationship("e-f", "e", "f", "links", "r1"),
                ],
            ),
            revision(
                2,
                "r2",
                vec![
                    occurrence("a", "Alpha", ""),
                    occurrence("b", "Beta two", ""),
                    occurrence("d", "Delta", ""),
                    occurrence("e", "Epsilon", ""),
                    occurrence("f", "Phi", "final"),
                ],
                vec![
                    relationship("a-d", "a", "d", "links", "r2"),
                    relationship("e-f", "e", "f", "links", "r2"),
                ],
            ),
        ],
        compared,
    )
}

fn classes(changes: &RevisionChanges) -> Vec<(&str, Change)> {
    changes
        .occurrences
        .iter()
        .map(|entry| (entry.occurrence_id.as_str(), entry.change))
        .collect()
}

#[test]
fn every_occurrence_is_added_updated_stable_or_removed() {
    let changes = history(None).changes();
    assert_eq!(changes.previous.as_ref().map(|r| r.as_str()), Some("r1"));
    assert_eq!(changes.current.as_str(), "r2");
    assert_eq!(
        classes(&changes),
        [
            ("a", Change::Updated),
            ("b", Change::Updated),
            ("d", Change::Added),
            ("e", Change::Stable),
            ("f", Change::Updated),
            ("c", Change::Removed),
        ]
    );
    let a = &changes.occurrences[0];
    assert!(a.relationships_differ);
    assert!(a.fields.is_empty());
    let b = &changes.occurrences[1];
    assert_eq!(b.fields, ["label"]);
    // a-b went, so b's incident relationships differ too.
    assert!(b.relationships_differ);
    let f = &changes.occurrences[4];
    assert_eq!(f.fields, ["note"]);
    assert!(!f.relationships_differ);
    assert_eq!(
        changes.counts(),
        BTreeMap::from([
            (Change::Added, 1),
            (Change::Updated, 3),
            (Change::Stable, 1),
            (Change::Removed, 1),
        ])
    );
}

#[test]
fn a_removed_relationship_is_reported_only_between_current_endpoints() {
    let changes = history(None).changes();
    let relationships = changes
        .relationships
        .iter()
        .map(|entry| (entry.id.as_str(), entry.change))
        .collect::<Vec<_>>();
    // e-f was disclosed again at r2; its new provenance revision is not a
    // change. a-b went while both ends remain, so it is removed; a-c went
    // with c, whose removal already tells it (Ruling 147b).
    assert_eq!(
        relationships,
        [
            ("a-d", Change::Added),
            ("e-f", Change::Stable),
            ("a-b", Change::Removed),
        ]
    );
}

#[test]
fn a_host_may_narrow_the_compared_fields() {
    let narrowed = history(Some(&["label"])).changes();
    assert_eq!(narrowed.occurrence("f"), Some(Change::Stable));
    assert_eq!(narrowed.occurrence("b"), Some(Change::Updated));
    // A relationship change still updates its endpoints.
    assert_eq!(narrowed.occurrence("a"), Some(Change::Updated));
    assert_eq!(
        history(None).changes().occurrence("f"),
        Some(Change::Updated)
    );
}

#[test]
fn a_changed_relationship_updates_both_endpoints() {
    let history = envelope(
        vec![
            revision(
                1,
                "r1",
                vec![occurrence("a", "Alpha", ""), occurrence("b", "Beta", "")],
                vec![relationship("a-b", "a", "b", "links", "r1")],
            ),
            revision(
                2,
                "r2",
                vec![occurrence("a", "Alpha", ""), occurrence("b", "Beta", "")],
                vec![relationship("a-b", "a", "b", "cites", "r2")],
            ),
        ],
        Some(&["label"]),
    );
    let changes = history.changes();
    assert_eq!(
        classes(&changes),
        [("a", Change::Updated), ("b", Change::Updated)]
    );
    assert_eq!(changes.relationships[0].change, Change::Updated);
}

#[test]
fn the_first_revision_adds_everything() {
    let history = history(None);
    let first = history.changes_at(0).unwrap();
    assert!(first.previous.is_none());
    assert!(
        first
            .occurrences
            .iter()
            .all(|entry| entry.change == Change::Added)
    );
    assert!(
        first
            .relationships
            .iter()
            .all(|entry| entry.change == Change::Added)
    );
    assert!(history.changes_at(2).is_none());
}

/// A site graph at one checkpoint as a history revision. Besides the fields
/// the site's checkpoint slider compares, each occurrence discloses its
/// position in the site's canonical id order, as the site exporter does, so
/// that adding or removing a repository moves its neighbours.
fn site_revision(sequence: u64, graph: &Value, revision: &str) -> Value {
    let text = |value: &Value, key: &str| value[key].as_str().unwrap().to_owned();
    let mut ids = graph["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| text(node, "id"))
        .collect::<Vec<_>>();
    ids.sort();
    let occurrences = graph["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| {
            let id = text(node, "id");
            json!({
                "occurrence_id": id,
                "source": { "adapter": "mer3ly.repository-graph/v1", "id": id },
                "values": {
                    "occurrence_id": { "kind": "text", "value": id },
                    "label": { "kind": "text", "value": text(node, "name") },
                    "class": { "kind": "text", "value": text(node, "class") },
                    "status": { "kind": "text", "value": text(node, "status") },
                    "pushed_at": { "kind": "text", "value": text(node, "pushed_at") },
                    "order": {
                        "kind": "number",
                        "value": ids.iter().position(|other| *other == id).unwrap(),
                    },
                },
            })
        })
        .collect::<Vec<_>>();
    let binding = json!({
        "authority": "https://merelyllc.com",
        "domain": "public-repositories",
        "resource": "repository-graph",
    });
    let relationships = graph["edges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|edge| {
            json!({
                "id": text(edge, "id"),
                "from_occurrence": text(edge, "source"),
                "to_occurrence": text(edge, "target"),
                "kind": text(edge, "kind"),
                "label": format!("{} {} {}", text(edge, "source"), text(edge, "kind"), text(edge, "target")),
                "explanation": "Disclosed by the site authority.",
                "provenance": {
                    "source": binding,
                    "source_revision": revision,
                    "method": text(edge, "provenance"),
                    "method_version": 1,
                    "provider": "mer3ly authority",
                    "evidence": [{ "adapter": "mer3ly.repository-relation/v1", "id": text(edge, "id") }],
                },
            })
        })
        .collect::<Vec<_>>();
    json!({
        "sequence": sequence,
        "revision": revision,
        "dataset": {
            "source": binding,
            "revision": revision,
            "fields": {
                "occurrence_id": "text",
                "label": "text",
                "class": "text",
                "status": "text",
                "pushed_at": "text",
                "order": "number",
            },
            "occurrences": occurrences,
        },
        "relationships": relationships,
    })
}

fn site_history(compared: Option<&[&str]>) -> (Value, HostDatasetV2) {
    let fixture: Value =
        serde_json::from_str(include_str!("../fixtures/site_checkpoints.json")).unwrap();
    let revisions = ["previous", "current"]
        .iter()
        .enumerate()
        .map(|(index, side)| {
            site_revision(
                index as u64 + 1,
                &fixture[side]["graph"],
                fixture[side]["cursor"]["commit"].as_str().unwrap(),
            )
        })
        .collect();
    (fixture, envelope(revisions, compared))
}

#[test]
fn the_sites_narrowed_fields_reproduce_its_checkpoint_classes_exactly() {
    let (fixture, history) = site_history(Some(&["label", "class", "status", "pushed_at"]));
    let changes = history.changes();
    let expected_nodes = fixture["expected"]["nodes"]
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
    let actual_nodes = changes
        .occurrences
        .iter()
        .map(|entry| (entry.occurrence_id.clone(), entry.change))
        .collect::<Vec<_>>();
    assert_eq!(actual_nodes, expected_nodes);
    assert_eq!(
        changes.counts(),
        BTreeMap::from([
            (Change::Added, 1),
            (Change::Updated, 15),
            (Change::Stable, 5),
            (Change::Removed, 5),
        ])
    );

    // The site lists current edges unmarked and appends removed ones. This
    // deliberately diverges from the site's diff_graphs as written, which
    // checked endpoints against current-plus-removed nodes and so kept every
    // removed edge (8 for this pair, all of them touching a removed
    // repository). Ruling 147b reports a removed relationship only between
    // endpoints that still exist, as the site's comment intended; the
    // fixture's expectations follow the ruling, so none remain here.
    let expected_edges = fixture["expected"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            (
                entry[0].as_str().unwrap(),
                entry[1].as_str() == Some("removed"),
            )
        })
        .collect::<Vec<_>>();
    let actual_edges = changes
        .relationships
        .iter()
        .map(|entry| (entry.id.as_str(), entry.change == Change::Removed))
        .collect::<Vec<_>>();
    assert_eq!(actual_edges, expected_edges);
    assert_eq!(
        actual_edges.iter().filter(|(_, removed)| *removed).count(),
        0
    );
}

#[test]
fn comparing_every_field_reads_the_same_checkpoints_more_broadly() {
    let (_, narrowed) = site_history(Some(&["label", "class", "status", "pushed_at"]));
    let (_, everything) = site_history(None);
    let narrowed = narrowed.changes();
    let everything = everything.changes();
    // The canonical order moved when repositories came and went, so every
    // field finds updates the narrowed set does not, and never fewer.
    assert!(everything.counts()[&Change::Updated] > narrowed.counts()[&Change::Updated]);
    for (wide, narrow) in everything.occurrences.iter().zip(&narrowed.occurrences) {
        assert_eq!(wide.occurrence_id, narrow.occurrence_id);
        if narrow.change == Change::Updated {
            assert_eq!(wide.change, Change::Updated);
        }
    }
    assert!(
        everything
            .occurrences
            .iter()
            .any(|entry| entry.fields == ["order"])
    );
}

/// a and b at two revisions with one relationship between them, whose
/// label names b and whose explanation dates its verification.
fn renamed_and_reverified(parts: Option<&[&str]>) -> RevisionChanges {
    let link = |revision: &str, b_label: &str, verified: &str| {
        let mut value = relationship("a-b", "a", "b", "links", revision);
        value["label"] = json!(format!("Alpha links {b_label}"));
        value["explanation"] = json!(format!("Verified on {verified}."));
        value
    };
    envelope_comparing(
        vec![
            revision(
                1,
                "r1",
                vec![occurrence("a", "Alpha", ""), occurrence("b", "Beta", "")],
                vec![link("r1", "Beta", "2026-09-01")],
            ),
            revision(
                2,
                "r2",
                vec![
                    occurrence("a", "Alpha", ""),
                    occurrence("b", "Beta two", ""),
                ],
                vec![link("r2", "Beta two", "2026-10-01")],
            ),
        ],
        Some(&["label"]),
        parts,
    )
    .changes()
}

#[test]
fn by_default_a_rename_or_reverification_updates_the_relationship() {
    let changes = renamed_and_reverified(None);
    assert_eq!(
        classes(&changes),
        [("a", Change::Updated), ("b", Change::Updated)]
    );
    assert!(changes.occurrences[0].relationships_differ);
    assert_eq!(changes.relationships[0].change, Change::Updated);
}

#[test]
fn a_host_may_narrow_what_counts_in_a_relationship() {
    let changes = renamed_and_reverified(Some(&["endpoints", "kind"]));
    // b was renamed, so b alone is updated; its neighbour a is stable, and
    // the relationship's new label and verification date do not count.
    assert_eq!(
        classes(&changes),
        [("a", Change::Stable), ("b", Change::Updated)]
    );
    assert!(!changes.occurrences[1].relationships_differ);
    assert_eq!(changes.relationships[0].change, Change::Stable);
}
