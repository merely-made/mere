// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use scenograph::{ProjectionInputBinding, RevisionEvidence, SourceBinding};
use scenomise::projection::{
    DisclosedRelationship, ItemSizes, ProjectionCompiler, ProjectionDataset, ProjectionFieldType,
    ProjectionOccurrence, ProjectionValue, RelationshipDataset, RelationshipProvenance,
    RelationshipSnapshot, relationship_recipe,
};
use seiche::Role;

fn fixture() -> (ProjectionDefinition, CompiledRelationshipProjection) {
    let source = SourceBinding {
        authority: "fixture".into(),
        domain: "relationships".into(),
        resource: "copied".into(),
    };
    let revision = scenograph::PublicSourceRevision::new("source:1");
    let occurrences = ["a", "b", "repeat"]
        .into_iter()
        .enumerate()
        .map(|(index, id)| ProjectionOccurrence {
            occurrence_id: id.into(),
            source: sceno::SourceRef::new(
                "knot.document",
                if id == "b" { "other" } else { "same" },
            ),
            values: BTreeMap::from([
                ("occurrence_id".into(), ProjectionValue::Text(id.into())),
                ("label".into(), ProjectionValue::Text(id.into())),
                ("order".into(), ProjectionValue::Number(index as f64)),
            ]),
        })
        .collect();
    let dataset = RelationshipDataset {
        dataset: ProjectionDataset {
            source: source.clone(),
            revision: revision.clone(),
            fields: BTreeMap::from([
                ("occurrence_id".into(), ProjectionFieldType::Text),
                ("label".into(), ProjectionFieldType::Text),
                ("order".into(), ProjectionFieldType::Number),
            ]),
            occurrences,
        },
        facets: [
            "authored_order".into(),
            "occurrence_labels".into(),
            "explained_relationships".into(),
        ]
        .into(),
        relationships: vec![DisclosedRelationship {
            id: "rhyme:a-b".into(),
            from_occurrence: "a".into(),
            to_occurrence: "b".into(),
            kind: "rhyme".into(),
            label: "Rhyme".into(),
            explanation: "Copied fixture disclosure".into(),
            provenance: RelationshipProvenance {
                source: source.clone(),
                source_revision: revision.clone(),
                method: "fixture".into(),
                method_version: 1,
                provider: "fixture".into(),
                evidence: vec![sceno::SourceRef::new("knot.document", "same")],
            },
        }],
    };
    let recipe = relationship_recipe(
        "recipe",
        "Rhyme",
        "author",
        "recipe:1",
        BTreeMap::from([(
            "copied".into(),
            ProjectionInputBinding {
                source,
                expects_generation: Some(revision),
                revision_evidence: RevisionEvidence::PublicGeneration,
            },
        )]),
    );
    let definition = recipe.definition.bind("copied", None).unwrap();
    let snapshot = RelationshipSnapshot {
        recipe,
        source_name: "copied".into(),
        selected_occurrence: None,
        selected_relationship: None,
    };
    let mut compiled = ProjectionCompiler::new(ItemSizes {
        card: sceno::Size2::new(16.0, 10.0),
    })
    .compile_relationship_snapshot(&snapshot, &dataset)
    .unwrap();
    // An explicit fixed physics plane, sufficiently separated for contact
    // exclusion not to mask the spring's contribution.
    for (index, item) in compiled.projection.scene.items.iter_mut().enumerate() {
        item.transform.translate = sceno::Vec2::new(index as f32 * 250.0, index as f32 * 40.0);
    }
    for relation in &mut compiled.projection.scene.relations {
        relation.points = vec![
            compiled.projection.scene.items[relation.from.0 as usize]
                .transform
                .translate,
            compiled.projection.scene.items[relation.to.0 as usize]
                .transform
                .translate,
        ];
    }
    (definition, compiled)
}

#[test]
fn disclosed_edges_drive_springs_without_graph_assertions_and_keep_source_identity() {
    let (definition, compiled) = fixture();
    let slot = super::super::dynamics_recipe::preset_slot("spring.rapier").unwrap();
    let sources = compiled.projection.scene.sources.clone();
    let mut linked =
        ProjectionDynamics::new_relationship(&definition, &compiled, &slot, &BTreeMap::new())
            .unwrap();
    assert_eq!(linked.canvas.graph.projected_relations().count(), 0);
    assert_eq!(linked.canvas.controller_edges().len(), 1);
    assert_eq!(linked.relationship_ids(), ["rhyme:a-b"]);
    let linked_pose = linked.snapshot(8).unwrap();
    let (_, mut bare) = fixture();
    bare.relationships.clear();
    bare.projection.scene.relations.clear();
    let mut control =
        ProjectionDynamics::new_relationship(&definition, &bare, &slot, &BTreeMap::new()).unwrap();
    assert_ne!(
        linked_pose.positions,
        control.snapshot(8).unwrap().positions
    );
    assert_eq!(linked.scene().sources, sources);
    let relation = &linked.scene().relations[0];
    assert_eq!(
        relation.points[0],
        linked.scene().items[relation.from.0 as usize]
            .transform
            .translate
    );
    assert_eq!(relation.kind.as_deref(), Some("rhyme"));
    linked.canvas.reconcile_derived();
    assert_eq!(linked.canvas.controller_edges().len(), 1);
}

#[test]
fn repeated_source_occurrences_take_independent_roles_and_fixed_tick_is_reproducible() {
    let (definition, compiled) = fixture();
    let slot = super::super::dynamics_recipe::preset_slot("spring.rapier").unwrap();
    let roles = BTreeMap::from([("a".into(), Role::Pinned), ("repeat".into(), Role::Seeded)]);
    let mut a =
        ProjectionDynamics::new_relationship(&definition, &compiled, &slot, &roles).unwrap();
    let start = a.scene().items[compiled.projection.instance_by_occurrence["a"].0 as usize]
        .transform
        .translate;
    let report = a.snapshot(12).unwrap();
    assert_eq!(
        a.scene().items[compiled.projection.instance_by_occurrence["a"].0 as usize]
            .transform
            .translate,
        start
    );
    let repeat = compiled.projection.instance_by_occurrence["repeat"].0 as usize;
    assert_ne!(
        a.scene().items[repeat].transform.translate,
        compiled.projection.scene.items[repeat].transform.translate
    );
    let mut b =
        ProjectionDynamics::new_relationship(&definition, &compiled, &slot, &roles).unwrap();
    assert_eq!(report, b.snapshot(12).unwrap());
    assert_eq!(a.scene(), b.scene());
    a.tick().unwrap();
    b.tick().unwrap();
    assert_eq!(a.scene(), b.scene());
    assert_eq!(a.ticks(), 13);
    assert!(a.scene().items.iter().all(
        |item| item.transform.translate.x.is_finite() && item.transform.translate.y.is_finite()
    ));
}

#[test]
fn binding_refuses_unknown_roles_endpoints_channels_and_legacy_relation_inputs() {
    let (definition, mut compiled) = fixture();
    let slot = super::super::dynamics_recipe::preset_slot("spring.rapier").unwrap();
    assert!(ProjectionDynamics::new(&definition, &compiled.projection, &slot).is_err());
    let roles = BTreeMap::from([("missing".into(), Role::Pinned)]);
    let error = ProjectionDynamics::new_relationship(&definition, &compiled, &slot, &roles)
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("occurrence_roles.missing"));
    compiled.relationships[0].disclosure.to_occurrence = "missing".into();
    let error =
        ProjectionDynamics::new_relationship(&definition, &compiled, &slot, &BTreeMap::new())
            .err()
            .unwrap()
            .to_string();
    assert!(error.contains("settle.inputs.relationships"));
    let (_, compiled) = fixture();
    let mut json: serde_json::Value = serde_json::from_str(&slot.spec).unwrap();
    json["channels"] = serde_json::json!({"mass":"mass.pagerank"});
    let named = DynamicsSlot::from_json(1, &json.to_string()).unwrap();
    assert!(
        ProjectionDynamics::new_relationship(&definition, &compiled, &named, &BTreeMap::new())
            .err()
            .unwrap()
            .to_string()
            .contains("channels")
    );
}

#[test]
fn authored_translation_is_used_as_the_pinned_home_without_rewriting_disclosure() {
    let (definition, mut compiled) = fixture();
    let instance = compiled.projection.instance_by_occurrence["a"];
    let at = sceno::Vec2::new(321.0, 123.0);
    compiled.projection.scene.items[instance.0 as usize]
        .transform
        .translate = at;
    let source = compiled.projection.scene.items[instance.0 as usize].source;
    let slot = super::super::dynamics_recipe::preset_slot("spring.rapier").unwrap();
    let mut run = ProjectionDynamics::new_relationship(
        &definition,
        &compiled,
        &slot,
        &BTreeMap::from([("a".into(), Role::Pinned)]),
    )
    .unwrap();
    run.snapshot(8).unwrap();
    assert_eq!(
        run.scene().items[instance.0 as usize].transform.translate,
        at
    );
    assert_eq!(run.scene().items[instance.0 as usize].source, source);
    assert_eq!(run.scene().sources, compiled.projection.scene.sources);
}

#[test]
fn composed_and_scheduled_slots_reopen_without_saving_a_running_stage() {
    use super::super::dynamics_spec::{DynamicsSpec, Node, Stage, Stop};
    let (definition, compiled) = fixture();
    for root in [
        Node::Mix {
            parts: vec![
                Node::preset("spring.rapier"),
                Node::preset("charge.barnes-hut"),
            ],
            weight: 1.0,
            overlays: Vec::new(),
        },
        Node::Schedule {
            stages: vec![
                Stage {
                    node: Node::preset("spring.rapier"),
                    stop: Stop::Frames(3),
                    capture: None,
                },
                Stage {
                    node: Node::preset("spring.rapier"),
                    stop: Stop::Frames(5),
                    capture: None,
                },
            ],
            weight: 1.0,
            overlays: Vec::new(),
        },
    ] {
        let spec = DynamicsSpec::new(root);
        let slot = DynamicsSlot::from_json(1, &serde_json::to_string(&spec).unwrap()).unwrap();
        let bytes = serde_json::to_vec(&slot).unwrap();
        let mut run =
            ProjectionDynamics::new_relationship(&definition, &compiled, &slot, &BTreeMap::new())
                .unwrap();
        let result = run.snapshot(4).unwrap();
        assert_eq!(run.slot, slot);
        let opened: DynamicsSlot = serde_json::from_slice(&bytes).unwrap();
        let mut reopen =
            ProjectionDynamics::new_relationship(&definition, &compiled, &opened, &BTreeMap::new())
                .unwrap();
        assert_eq!(reopen.ticks(), 0);
        assert_eq!(result, reopen.snapshot(4).unwrap());
    }
}
