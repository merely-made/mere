// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use scenograph::{ProjectionInputBinding, RevisionEvidence};
use std::collections::BTreeSet;

/// The 164 by 68 card the compiler wrote in before hosts supplied sizes.
fn compiler() -> ProjectionCompiler {
    ProjectionCompiler::new(ItemSizes {
        card: Size2::new(164.0, 68.0),
    })
}

// Adapter-shaped fixtures, not domain computations. Actual Mora/Working Set
// calculations belong to their consumer integration tests, not this compiler.
fn disclosed(owner: &str, kind: &str) -> RelationshipDataset {
    let source = SourceBinding {
        authority: owner.into(),
        domain: "selected-material".into(),
        resource: "selection:1".into(),
    };
    let revision: PublicSourceRevision = format!("{owner}-r1").into();
    let occurrence = |id: &str, order: f64, original: &str| ProjectionOccurrence {
        occurrence_id: id.into(),
        source: SourceRef::new(owner, original),
        values: BTreeMap::from([
            ("occurrence_id".into(), ProjectionValue::Text(id.into())),
            ("label".into(), ProjectionValue::Text(format!("Label {id}"))),
            ("order".into(), ProjectionValue::Number(order)),
        ]),
    };
    RelationshipDataset {
        dataset: ProjectionDataset { source: source.clone(), revision: revision.clone(),
            fields: BTreeMap::from([("occurrence_id".into(), ProjectionFieldType::Text), ("label".into(), ProjectionFieldType::Text), ("order".into(), ProjectionFieldType::Number)]),
            occurrences: vec![occurrence("a", 2.0, "same-original"), occurrence("b", 0.0, "other-original"), occurrence("repeat", 1.0, "same-original")] },
        facets: BTreeSet::from([AUTHORED_ORDER_FACET.into(), OCCURRENCE_LABELS_FACET.into(), EXPLAINED_RELATIONSHIPS_FACET.into()]),
        relationships: vec![DisclosedRelationship { id: format!("{owner}:comparison:1"), from_occurrence: "repeat".into(), to_occurrence: "b".into(), kind: kind.into(), label: "Disclosed comparison".into(), explanation: "The owner explains the result for these exact occurrences; compilation does not calculate a domain result.".into(),
            provenance: RelationshipProvenance { source, source_revision: revision, method: format!("{owner}.compare"), method_version: 1, provider: format!("{owner} fixture adapter"), evidence: vec![SourceRef::new(owner, "same-original"), SourceRef::new(owner, "other-original")] } }],
    }
}
fn binding(data: &RelationshipDataset) -> ProjectionInputBinding {
    ProjectionInputBinding {
        source: data.dataset.source.clone(),
        expects_generation: Some(data.dataset.revision.clone()),
        revision_evidence: RevisionEvidence::PublicGeneration,
    }
}
fn snapshot(data: &RelationshipDataset) -> RelationshipSnapshot {
    RelationshipSnapshot {
        recipe: relationship_recipe(
            "material-comparison",
            "Material relationships",
            "author",
            "recipe-r1",
            BTreeMap::from([("selected".into(), binding(data))]),
        ),
        source_name: "selected".into(),
        selected_occurrence: Some("repeat".into()),
        selected_relationship: Some(data.relationships[0].id.clone()),
    }
}
fn refuses(saved: &RelationshipSnapshot, data: &RelationshipDataset, field: &str) {
    let issues = compiler().compile_relationship_snapshot(saved, data).unwrap_err();
    assert!(
        issues.iter().any(|issue| issue.field.contains(field)),
        "expected {field}, got {issues:?}"
    );
}

#[test]
fn scene_preserves_repeated_source_occurrences_and_exact_explained_edge_endpoints() {
    let data = disclosed("sound-adapter", "sound.rhyme");
    let saved = snapshot(&data);
    let compiled = compiler().compile_relationship_snapshot(&saved, &data).unwrap();
    assert_eq!(compiled.projection.scene.items.len(), 3);
    assert_eq!(compiled.projection.scene.sources.len(), 2);
    let original = compiled.projection.instance_by_occurrence["a"];
    let repeat = compiled.projection.instance_by_occurrence["repeat"];
    let b = compiled.projection.instance_by_occurrence["b"];
    assert_ne!(original, repeat);
    assert_eq!(
        compiled.projection.scene.items[original.0 as usize].source,
        compiled.projection.scene.items[repeat.0 as usize].source
    );
    assert_eq!(compiled.projection.selected, Some(repeat));
    assert_eq!(compiled.relationships[0].from, repeat);
    assert_eq!(compiled.relationships[0].to, b);
    assert_eq!(compiled.relationships[0].disclosure, data.relationships[0]);
    assert_eq!(compiled.projection.scene.relations[0].from, repeat);
    assert_eq!(
        compiled.projection.scene.relations[0].kind.as_deref(),
        Some("sound.rhyme")
    );
    assert_eq!(compiled.projection.scene.relations[0].points.len(), 2);
    assert_eq!(compiled.selected_relationship, saved.selected_relationship);
    assert!(
        compiled.projection.scene.items[b.0 as usize]
            .transform
            .translate
            .x
            < compiled.projection.scene.items[repeat.0 as usize]
                .transform
                .translate
                .x
    );
}

#[test]
fn same_edited_recipe_saves_reopens_and_rebinds_between_two_host_disclosures() {
    let sound = disclosed("sound-adapter", "sound.rhyme");
    let music = disclosed("working-set-adapter", "music.shared_pitch_classes");
    let mut draft = RelationshipRecipeDraft::new(snapshot(&sound).recipe);
    draft.apply(RecipeEdit::SetLabel(
        "My selected-material comparison".into(),
    ));
    let mut arrangement = draft.recipe().definition.arrangement.clone();
    arrangement.spacing = 48;
    draft.apply(RecipeEdit::SetArrangement(arrangement));
    draft.apply(RecipeEdit::BindInput {
        name: "music".into(),
        binding: binding(&music),
    });
    let mut saved = RelationshipSnapshot {
        recipe: draft.to_recipe().unwrap(),
        source_name: "selected".into(),
        selected_occurrence: Some("repeat".into()),
        selected_relationship: Some(sound.relationships[0].id.clone()),
    };
    let reopened: RelationshipSnapshot =
        serde_json::from_slice(&serde_json::to_vec(&saved).unwrap()).unwrap();
    assert_eq!(saved, reopened);
    assert_eq!(
        compiler().compile_relationship_snapshot(&reopened, &sound)
            .unwrap()
            .projection
            .selected,
        Some(InstanceId(2))
    );
    saved.source_name = "music".into();
    saved.selected_relationship = Some(music.relationships[0].id.clone());
    let rebound = compiler().compile_relationship_snapshot(&saved, &music).unwrap();
    assert_eq!(saved.recipe.definition.id, "material-comparison");
    assert_eq!(
        saved
            .recipe
            .definition
            .provenance
            .source_revision
            .as_ref()
            .unwrap()
            .as_str(),
        "recipe-r1"
    );
    assert_eq!(
        saved.recipe.definition.sources["selected"]
            .expects_generation
            .as_ref()
            .unwrap(),
        &sound.dataset.revision
    );
    assert_eq!(
        saved.recipe.definition.sources["music"]
            .expects_generation
            .as_ref()
            .unwrap(),
        &music.dataset.revision
    );
    assert_eq!(
        rebound.relationships[0].disclosure.provenance,
        music.relationships[0].provenance
    );
    assert_eq!(
        rebound.projection.score.arrangement,
        compiler().compile_relationship_snapshot(&reopened, &sound)
            .unwrap()
            .projection
            .score
            .arrangement
    );
}

#[test]
fn numeric_field_matching_without_named_semantic_facets_is_refused() {
    for facet in [
        AUTHORED_ORDER_FACET,
        OCCURRENCE_LABELS_FACET,
        EXPLAINED_RELATIONSHIPS_FACET,
    ] {
        let mut data = disclosed("sound-adapter", "sound.rhyme");
        let saved = snapshot(&data);
        data.facets.remove(facet);
        refuses(&saved, &data, facet);
    }
}

#[test]
fn ordinal_authored_order_is_not_silently_interpreted_as_scatter_distance() {
    let data = disclosed("sound-adapter", "sound.rhyme");
    let mut saved = snapshot(&data);
    saved.recipe.definition.arrangement.kind = SCATTER_ARRANGEMENT_ID.into();
    refuses(&saved, &data, "arrangement.kind");
}

#[test]
fn unknown_kind_and_stale_exact_selections_refuse_instead_of_empty_fallback() {
    let data = disclosed("sound-adapter", "sound.rhyme");
    let mut saved = snapshot(&data);
    saved.recipe.relationship_kind = Some("music.shared_pitch_classes".into());
    refuses(&saved, &data, "relationship_kind");
    saved = snapshot(&data);
    saved.selected_occurrence = Some("removed-occurrence".into());
    refuses(&saved, &data, "selected_occurrence");
    saved = snapshot(&data);
    saved.selected_relationship = Some("removed-relationship".into());
    refuses(&saved, &data, "selected_relationship");
}

#[test]
fn provenance_revision_missing_explanation_and_bad_endpoints_are_refused() {
    let original = disclosed("sound-adapter", "sound.rhyme");
    let saved = snapshot(&original);
    let mut changed = original.clone();
    changed.relationships[0].provenance.source_revision = "different-revision".into();
    refuses(&saved, &changed, "provenance.source_revision");
    changed = original.clone();
    changed.relationships[0].explanation.clear();
    refuses(&saved, &changed, "explanation");
    changed = original.clone();
    changed.relationships[0].to_occurrence = "source-id-not-occurrence".into();
    refuses(&saved, &changed, "endpoints");
    changed = original.clone();
    changed.relationships[0].provenance.method_version = 0;
    refuses(&saved, &changed, "method_version");
    changed = original.clone();
    changed.relationships[0].provenance.evidence.clear();
    refuses(&saved, &changed, "evidence");
    changed = original;
    changed.dataset.revision = "r2".into();
    changed.relationships[0].provenance.source_revision = "r2".into();
    refuses(&saved, &changed, "provenance.source_revision");
}

#[test]
fn duplicate_occurrence_order_and_relationship_ids_cannot_alias_repeated_material() {
    let original = disclosed("sound-adapter", "sound.rhyme");
    let saved = snapshot(&original);
    let mut changed = original.clone();
    changed.dataset.occurrences[0]
        .values
        .insert("order".into(), ProjectionValue::Number(0.0));
    refuses(&saved, &changed, "order");
    changed = original.clone();
    changed.relationships.push(changed.relationships[0].clone());
    refuses(&saved, &changed, ".id");
    changed = original;
    changed.dataset.occurrences[0].occurrence_id = "repeat".into();
    refuses(&saved, &changed, "occurrence_id");
}

#[test]
fn incomplete_common_edit_state_retains_keystrokes_without_promotion() {
    let data = disclosed("sound-adapter", "sound.rhyme");
    let mut draft = RelationshipRecipeDraft::new(snapshot(&data).recipe);
    draft.apply(RecipeEdit::SetLabel(String::new()));
    assert_eq!(draft.recipe().definition.label, "");
    assert!(draft.to_recipe().is_err());
    draft.apply(RecipeEdit::SetLabel("Repaired".into()));
    assert!(draft.to_recipe().is_ok());
}

#[test]
fn bounds_precede_binding_cloning_and_solving_and_all_numbers_are_finite() {
    let data = disclosed("sound-adapter", "sound.rhyme");
    let saved = snapshot(&data);
    for limits in [
        RelationshipCompileLimits {
            max_occurrences: 1,
            ..RelationshipCompileLimits::default()
        },
        RelationshipCompileLimits {
            max_relationships: 0,
            ..RelationshipCompileLimits::default()
        },
        RelationshipCompileLimits {
            max_fields: 1,
            ..RelationshipCompileLimits::default()
        },
        RelationshipCompileLimits {
            max_inputs: 0,
            ..RelationshipCompileLimits::default()
        },
        RelationshipCompileLimits {
            max_total_text_bytes: 16,
            ..RelationshipCompileLimits::default()
        },
    ] {
        assert!(
            compiler().compile_relationship_snapshot_with_limits(&saved, &data, &limits)
                .unwrap_err()
                .iter()
                .any(|issue| issue.field.starts_with("limits."))
        );
    }
    let mut invalid = data;
    invalid.dataset.occurrences[0]
        .values
        .insert("order".into(), ProjectionValue::Number(f64::NAN));
    refuses(&saved, &invalid, "dataset.values");
}

/// The relationship compile refuses a degenerate card the same way (S32).
#[test]
fn a_relationship_compile_refuses_a_degenerate_card() {
    let data = disclosed("sound-adapter", "sound.rhyme");
    let saved = snapshot(&data);
    for card in [Size2::new(0.0, 0.0), Size2::new(f32::NAN, 68.0)] {
        let issues = ProjectionCompiler::new(ItemSizes { card })
            .compile_relationship_snapshot(&saved, &data)
            .unwrap_err();
        assert!(
            issues.iter().any(|issue| issue.field == "items.card"),
            "{issues:?}"
        );
    }
    assert!(
        compiler()
            .compile_relationship_snapshot(&saved, &data)
            .is_ok()
    );
}
