// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::projection::{
    ProjectionFieldType, ProjectionOccurrence, ProjectionValue, RelationshipProvenance,
};
use sceno::SourceRef;
use scenograph::SourceBinding;

fn fixture() -> (ProjectionDataset, Vec<DisclosedRelationship>) {
    let source = SourceBinding {
        authority: "example.test".into(),
        domain: "components".into(),
        resource: "groups".into(),
    };
    let dataset = ProjectionDataset {
        source: source.clone(),
        revision: "r1".into(),
        fields: BTreeMap::from([
            ("occurrence_id".into(), ProjectionFieldType::Text),
            ("label".into(), ProjectionFieldType::Text),
        ]),
        occurrences: [
            "a",
            "a.workspace",
            "a.one",
            "a.two",
            "b",
            "b.one",
            "unrelated",
        ]
        .into_iter()
        .map(|id| ProjectionOccurrence {
            occurrence_id: id.into(),
            source: SourceRef::new("example", id),
            values: BTreeMap::from([
                ("occurrence_id".into(), ProjectionValue::Text(id.into())),
                ("label".into(), ProjectionValue::Text(id.into())),
            ]),
        })
        .collect(),
    };
    let relationships = [
        ("m1", "a", "a.workspace", "contains"),
        ("m2", "a.workspace", "a.one", "contains"),
        ("m3", "a.workspace", "a.two", "contains"),
        ("m4", "b", "b.one", "contains"),
        ("d1", "a.one", "b.one", "depends_on"),
        ("d2", "a.two", "b.one", "depends_on"),
        ("d3", "a.one", "a.two", "depends_on"),
        ("d4", "b.one", "a.two", "depends_on"),
    ]
    .into_iter()
    .map(|(id, from, to, kind)| DisclosedRelationship {
        id: id.into(),
        from_occurrence: from.into(),
        to_occurrence: to.into(),
        kind: kind.into(),
        label: kind.into(),
        explanation: format!("Evidence for {id}"),
        provenance: RelationshipProvenance {
            source: source.clone(),
            source_revision: "r1".into(),
            method: "fixture".into(),
            method_version: 1,
            provider: "tests".into(),
            evidence: vec![SourceRef::new("example", id)],
        },
    })
    .collect();
    (dataset, relationships)
}

#[test]
fn collapsing_preserves_parallel_directional_witnesses_and_internal_edges() {
    let (dataset, relationships) = fixture();
    let original = (dataset.clone(), relationships.clone());
    let hierarchy = GroupHierarchy::new(&dataset, &relationships, "contains").unwrap();
    let projected = hierarchy.project(&GroupViewState::default()).unwrap();
    assert_eq!(
        projected.visible,
        BTreeSet::from(["a".into(), "b".into(), "unrelated".into()])
    );
    assert_eq!(projected.relations[0].relationship_ids, ["d1", "d2"]);
    assert_eq!(
        (
            &projected.relations[1].from[..],
            &projected.relations[1].to[..]
        ),
        ("b", "a")
    );
    assert_eq!(projected.internal_relationships["a"], ["d3"]);
    assert_eq!((dataset, relationships), original);
}

#[test]
fn nested_expansion_and_collapse_restore_the_same_occurrences() {
    let (dataset, relationships) = fixture();
    let hierarchy = GroupHierarchy::new(&dataset, &relationships, "contains").unwrap();
    let closed = hierarchy.project(&GroupViewState::default()).unwrap();
    let mut state = GroupViewState::default();
    state.expanded.insert("a".into());
    let first = hierarchy.project(&state).unwrap();
    assert!(first.visible.contains("a.workspace"));
    assert!(!first.visible.contains("a.one"));
    state.expanded.insert("a.workspace".into());
    let second = hierarchy.project(&state).unwrap();
    assert!(second.visible.contains("a.one"));
    assert!(second.visible.contains("a.two"));
    state.expanded.remove("a");
    assert_eq!(hierarchy.project(&state).unwrap(), closed);
    state.expanded.insert("a".into());
    assert_eq!(hierarchy.project(&state).unwrap(), second);
    assert_eq!(
        serde_json::from_str::<GroupViewState>(&serde_json::to_string(&state).unwrap()).unwrap(),
        state
    );
}

#[test]
fn entering_keeps_breadcrumbs_and_cross_boundary_context_but_drops_unrelated_nodes() {
    let (dataset, relationships) = fixture();
    let hierarchy = GroupHierarchy::new(&dataset, &relationships, "contains").unwrap();
    let state = GroupViewState {
        entered: Some("a.workspace".into()),
        ..Default::default()
    };
    let projection = hierarchy.project(&state).unwrap();
    assert_eq!(projection.breadcrumbs, ["a", "a.workspace"]);
    assert_eq!(projection.boundary, BTreeSet::from(["b".into()]));
    assert!(!projection.visible.contains("unrelated"));
    assert!(projection.visible.contains("a.one"));
    assert!(
        projection
            .relations
            .iter()
            .any(|r| r.from == "b" && r.to == "a.two" && r.relationship_ids == ["d4"])
    );
}

#[test]
fn refuses_ambiguous_ownership_cycles_unknown_endpoints_and_stale_evidence() {
    let (dataset, mut relationships) = fixture();
    let mut extra = relationships[0].clone();
    extra.id = "ambiguous".into();
    extra.from_occurrence = "b".into();
    relationships.push(extra);
    assert!(
        GroupHierarchy::new(&dataset, &relationships, "contains")
            .unwrap_err()
            .contains("multiple")
    );
    relationships.pop();
    let mut cycle = relationships[0].clone();
    cycle.id = "cycle".into();
    cycle.from_occurrence = "a.one".into();
    cycle.to_occurrence = "a".into();
    relationships.push(cycle);
    assert!(
        GroupHierarchy::new(&dataset, &relationships, "contains")
            .unwrap_err()
            .contains("cycle")
    );
    relationships.pop();
    relationships[0].to_occurrence = "missing".into();
    assert!(GroupHierarchy::new(&dataset, &relationships, "contains").is_err());
    relationships[0].to_occurrence = "a.workspace".into();
    relationships[0].provenance.source_revision = "stale".into();
    assert!(GroupHierarchy::new(&dataset, &relationships, "contains").is_err());
}

#[test]
fn membership_is_explicit_and_other_relationship_kinds_stay_separate() {
    let (dataset, mut relationships) = fixture();
    assert!(GroupHierarchy::new(&dataset, &relationships, "not-disclosed").is_err());
    assert!(GroupHierarchy::new(&dataset, &relationships, "").is_err());
    let mut other = relationships[4].clone();
    other.id = "other".into();
    other.kind = "implements".into();
    relationships.push(other);
    let hierarchy = GroupHierarchy::new(&dataset, &relationships, "contains").unwrap();
    let projection = hierarchy.project(&GroupViewState::default()).unwrap();
    assert!(
        projection
            .relations
            .iter()
            .any(|r| r.kind == "implements" && r.relationship_ids == ["other"])
    );
    let state = GroupViewState {
        expanded: BTreeSet::from(["a.one".into()]),
        entered: None,
    };
    assert!(hierarchy.project(&state).is_err());
}

#[test]
fn the_same_projection_works_for_a_document_hierarchy() {
    let (dataset, mut relationships) = fixture();
    for relation in &mut relationships {
        if relation.kind == "contains" {
            relation.kind = "chapter_contains".into();
        }
    }
    let hierarchy = GroupHierarchy::new(&dataset, &relationships, "chapter_contains").unwrap();
    assert_eq!(
        hierarchy
            .project(&GroupViewState::default())
            .unwrap()
            .relations
            .len(),
        2
    );
}
