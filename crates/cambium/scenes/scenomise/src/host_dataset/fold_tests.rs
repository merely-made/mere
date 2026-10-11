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

fn folds(
    dataset: &ProjectionDataset,
    relationships: &[DisclosedRelationship],
    kind: &str,
) -> Result<HostDatasetFolds, String> {
    let mut scene = Scene::new();
    let mut instances = BTreeMap::new();
    // Deliberately reverse dataset order to check that membership follows the
    // compiled mapping, not an independent occurrence sort.
    for item in dataset.occurrences.iter().rev() {
        let source = scene.intern_source(item.source.clone());
        instances.insert(
            item.occurrence_id.clone(),
            InstanceId(scene.items.len() as u32),
        );
        scene.items.push(sceno::ProjectedItem {
            source,
            space: Scene::WORLD,
            transform: sceno::Transform2::IDENTITY,
            footprint: sceno::Footprint::Point,
            representation: sceno::Representation::Card,
            layer: 0,
            visible: true,
            hit: None,
            channels: Vec::new(),
        });
    }
    scene.relations = relationships
        .iter()
        .filter(|relation| {
            instances.contains_key(&relation.from_occurrence)
                && instances.contains_key(&relation.to_occurrence)
        })
        .map(|relation| sceno::RoutedRelation {
            from: instances[&relation.from_occurrence],
            to: instances[&relation.to_occurrence],
            space: Scene::WORLD,
            points: Vec::new(),
            kind: Some(relation.kind.clone()),
            weight: None,
        })
        .collect();
    HostDatasetFolds::new(dataset, relationships, kind, scene, instances)
}

#[test]
fn collapsing_preserves_parallel_directional_witnesses_and_internal_edges() {
    let (dataset, relationships) = fixture();
    let original = (dataset.clone(), relationships.clone());
    let hierarchy = folds(&dataset, &relationships, "contains").unwrap();
    let projected = hierarchy.project(&FoldViewState::default()).unwrap();
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
    let hierarchy = folds(&dataset, &relationships, "contains").unwrap();
    let closed = hierarchy.project(&FoldViewState::default()).unwrap();
    let mut state = FoldViewState::default();
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
        serde_json::from_str::<FoldViewState>(&serde_json::to_string(&state).unwrap()).unwrap(),
        state
    );
}

#[test]
fn entering_keeps_breadcrumbs_and_cross_boundary_context_but_drops_unrelated_nodes() {
    let (dataset, relationships) = fixture();
    let hierarchy = folds(&dataset, &relationships, "contains").unwrap();
    let state = FoldViewState {
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
        folds(&dataset, &relationships, "contains")
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
        folds(&dataset, &relationships, "contains")
            .unwrap_err()
            .contains("cycle")
    );
    relationships.pop();
    relationships[0].to_occurrence = "missing".into();
    assert!(folds(&dataset, &relationships, "contains").is_err());
    relationships[0].to_occurrence = "a.workspace".into();
    relationships[0].provenance.source_revision = "stale".into();
    assert!(folds(&dataset, &relationships, "contains").is_err());
}

#[test]
fn membership_is_explicit_and_other_relationship_kinds_stay_separate() {
    let (dataset, mut relationships) = fixture();
    assert!(folds(&dataset, &relationships, "not-disclosed").is_err());
    assert!(folds(&dataset, &relationships, "").is_err());
    let mut other = relationships[4].clone();
    other.id = "other".into();
    other.kind = "implements".into();
    relationships.push(other);
    let hierarchy = folds(&dataset, &relationships, "contains").unwrap();
    let projection = hierarchy.project(&FoldViewState::default()).unwrap();
    assert!(
        projection
            .relations
            .iter()
            .any(|r| r.kind == "implements" && r.relationship_ids == ["other"])
    );
    let state = FoldViewState {
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
    let hierarchy = folds(&dataset, &relationships, "chapter_contains").unwrap();
    assert_eq!(
        hierarchy
            .project(&FoldViewState::default())
            .unwrap()
            .relations
            .len(),
        2
    );
}

#[test]
fn portable_facts_are_the_only_hiding_authority_and_survive_roundtrip() {
    let (dataset, relationships) = fixture();
    let host = folds(&dataset, &relationships, "contains").unwrap();
    let projection = host.project(&FoldViewState::default()).unwrap();
    let scene = projection.scene;
    assert_eq!(scene.validate_folds(), Ok(()));
    assert_eq!(scene.folds.len(), 2);
    assert_eq!(scene.items, host.scene.items);
    assert_eq!(scene.sources, host.scene.sources);
    assert_eq!(scene.relations, host.scene.relations);
    let effect = scene.fold_effect();
    for (id, instance) in host.instances() {
        assert_eq!(
            projection.visible.contains(id),
            effect.is_shown(*instance, scene.items[instance.0 as usize].visible)
        );
    }
    let a = scene
        .folds
        .iter()
        .find(|fold| fold.stand_in_member() == Some(host.instances["a"]))
        .unwrap();
    assert_eq!(a.members.len(), 4);
    assert_eq!(a.label.as_deref(), Some("a"));
    assert_eq!(
        a.rule,
        Some(FoldRule::Descendants {
            root: host.instances["a"],
            family: "contains".into(),
            direction: FoldDirection::Outgoing
        })
    );
    let boundary = a.boundary.as_ref().unwrap();
    assert_eq!(boundary.internal_relations, 4); // three membership assertions and d3
    assert!(
        boundary
            .bundles
            .iter()
            .any(|bundle| bundle.outside == host.instances["b.one"]
                && bundle.direction == FoldDirection::Outgoing
                && bundle.count == 2)
    );
    assert!(
        boundary
            .bundles
            .iter()
            .any(|bundle| bundle.direction == FoldDirection::Incoming && bundle.count == 1)
    );
    let mut reopened: Scene =
        serde_json::from_str(&serde_json::to_string(&scene).unwrap()).unwrap();
    assert_eq!(reopened, scene);
    reopened.folds.clear();
    assert_eq!(reopened, host.scene); // unfolding restores the complete original scene
}

#[test]
fn an_entered_child_unfolds_its_ancestors_without_overlapping_active_facts() {
    let (dataset, relationships) = fixture();
    let host = folds(&dataset, &relationships, "contains").unwrap();
    let state = FoldViewState {
        entered: Some("a.workspace".into()),
        ..Default::default()
    };
    let entered = host.project(&state).unwrap();
    assert_eq!(entered.scene.validate_folds(), Ok(()));
    assert_eq!(entered.scene.folds.len(), 1); // only b remains folded
    assert!(entered.visible.contains("a.one"));
    assert!(!entered.visible.contains("a"));
    assert!(entered.boundary.contains("b"));
    assert_eq!(state.expanded, BTreeSet::new()); // entering did not overwrite remembered choices
    let closed = host.scene(&FoldViewState::default()).unwrap();
    assert_eq!(closed.folds.len(), 2);
}

#[test]
fn appearance_visibility_survives_fold_and_unfold() {
    let (dataset, relationships) = fixture();
    let mut host = folds(&dataset, &relationships, "contains").unwrap();
    host.scene.items[host.instances["a.one"].0 as usize].visible = false;
    let closed = host.project(&FoldViewState::default()).unwrap();
    let open = host
        .project(&FoldViewState {
            expanded: host.groups().map(str::to_owned).collect(),
            entered: None,
        })
        .unwrap();
    assert!(open.scene.folds.is_empty());
    assert_eq!(open.scene.items, closed.scene.items);
    assert!(!open.visible.contains("a.one"));
    assert!(
        open.relations
            .iter()
            .all(|relation| relation.from != "a.one" && relation.to != "a.one")
    );
}

#[test]
fn a_partial_or_aliased_instance_mapping_cannot_create_fold_facts() {
    let (dataset, relationships) = fixture();
    let host = folds(&dataset, &relationships, "contains").unwrap();
    let mut instances = host.instances.clone();
    instances.remove("a.one");
    assert!(
        HostDatasetFolds::new(
            &dataset,
            &relationships,
            "contains",
            host.scene.clone(),
            instances
        )
        .is_err()
    );
    let mut instances = host.instances.clone();
    instances.insert("a.one".into(), instances["a.two"]);
    assert!(
        HostDatasetFolds::new(
            &dataset,
            &relationships,
            "contains",
            host.scene.clone(),
            instances
        )
        .is_err()
    );
    let folded = host.scene(&FoldViewState::default()).unwrap();
    assert!(
        HostDatasetFolds::new(
            &dataset,
            &relationships,
            "contains",
            folded,
            host.instances.clone()
        )
        .unwrap_err()
        .contains("existing")
    );
}
