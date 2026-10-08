// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;

use super::*;
use crate::catalog::FAMILIES;
use crate::registry::{SolveError, Solver, SolverCapability};
use scenograph::options::{OptionDefault, OptionKind, OptionSpec};
use scenograph::{
    Appearance, Arrangement, Encoding, Interaction, Provenance, Reading, RevisionEvidence,
    SelectionMode,
};

fn sizes(w: f32, h: f32) -> ItemSizes {
    ItemSizes {
        card: Size2::new(w, h),
    }
}

/// Three occurrences carrying every kind of value a family might read.
fn dataset() -> ProjectionDataset {
    let occurrence = |id: &str, x: f64, y: f64, lane: &str, layer: f64| ProjectionOccurrence {
        occurrence_id: id.into(),
        source: SourceRef::new("fixture", id),
        values: BTreeMap::from([
            ("occurrence_id".into(), ProjectionValue::Text(id.into())),
            ("label".into(), ProjectionValue::Text(format!("Label {id}"))),
            ("x".into(), ProjectionValue::Number(x)),
            ("y".into(), ProjectionValue::Number(y)),
            ("lane".into(), ProjectionValue::Text(lane.into())),
            ("layer".into(), ProjectionValue::Number(layer)),
        ]),
    };
    ProjectionDataset {
        source: SourceBinding {
            authority: "fixture.local".into(),
            domain: "set".into(),
            resource: "set:catalog".into(),
        },
        revision: "catalog-r1".into(),
        fields: BTreeMap::from([
            ("occurrence_id".into(), ProjectionFieldType::Text),
            ("label".into(), ProjectionFieldType::Text),
            ("x".into(), ProjectionFieldType::Number),
            ("y".into(), ProjectionFieldType::Number),
            ("lane".into(), ProjectionFieldType::Text),
            ("layer".into(), ProjectionFieldType::Number),
        ]),
        occurrences: vec![
            occurrence("c", 0.0, 0.0, "todo", 0.0),
            occurrence("a", 2.0, 0.0, "doing", 1.0),
            occurrence("b", 1.0, 1.0, "todo", 2.0),
        ],
    }
}

fn definition(kind: &str, x: &str) -> ProjectionDefinition {
    let data = dataset();
    ProjectionDefinition {
        version: 1,
        id: "catalog".into(),
        label: "Catalog".into(),
        source: data.source.clone(),
        reading: Reading {
            kind: "nodes".into(),
            key: "occurrence_id".into(),
            value: None,
        },
        encoding: Encoding {
            x: Channel::Field(x.into()),
            y: Channel::Field("y".into()),
            color: None,
            label: Some(Channel::Field("label".into())),
        },
        arrangement: Arrangement {
            kind: kind.into(),
            direction: "coordinates".into(),
            spacing: 16,
            options: BTreeMap::new(),
        },
        interaction: Interaction {
            selection: SelectionMode::Single,
            pan: false,
            zoom: false,
        },
        appearance: Appearance {
            realization: "canvas".into(),
            title: "Catalog".into(),
            theme: "slate".into(),
        },
        provenance: Provenance {
            author: "test".into(),
            source_revision: Some(data.revision.clone()),
            revision_evidence: RevisionEvidence::PublicGeneration,
            note: String::new(),
        },
    }
}

/// The x field a family reads in this fixture.
fn x_for(family: Family) -> &'static str {
    match family.channels() {
        ChannelUse::CategoricalAxis => "lane",
        ChannelUse::IntegerAxis => "layer",
        _ => "x",
    }
}

fn position(compiled: &CompiledProjection, occurrence: &str) -> Vec2 {
    let instance = compiled.instance_by_occurrence[occurrence];
    compiled.scene.items[instance.0 as usize].transform.translate
}

fn fields(issues: &[CompileIssue]) -> Vec<&str> {
    issues.iter().map(|issue| issue.field.as_str()).collect()
}

#[test]
fn every_catalog_family_compiles_from_a_recipe() {
    let compiler = ProjectionCompiler::new(sizes(164.0, 68.0));
    for family in FAMILIES {
        let compiled = compiler
            .compile(&definition(family.id(), x_for(family)), &dataset())
            .unwrap_or_else(|issues| panic!("{} refused: {issues:?}", family.id()));
        assert_eq!(compiled.scene.items.len(), 3, "{}", family.id());
        for item in &compiled.scene.items {
            let at = item.transform.translate;
            assert!(at.x.is_finite() && at.y.is_finite(), "{} placed {at:?}", family.id());
        }
    }
}

#[test]
fn ids_and_aliases_resolve_to_their_families() {
    for family in FAMILIES {
        assert_eq!(Family::resolve(family.id()), Some(family));
        assert_eq!(family.capability().id, family.id());
        assert!(family.capability().is_deterministic);
    }
    assert_eq!(Family::resolve(GRID_ARRANGEMENT_ID), Some(Family::Grid));
    assert_eq!(Family::resolve(SCATTER_ARRANGEMENT_ID), Some(Family::Geographic));
    assert_eq!(Family::resolve("made-up-layout"), None);
}

#[test]
fn grid_pitch_follows_the_host_card() {
    // c and b differ by one column (x ranks 0 and 1) and one row.
    let pitch = |w: f32, h: f32| {
        let compiled = ProjectionCompiler::new(sizes(w, h))
            .compile(&definition("grid", "x"), &dataset())
            .unwrap();
        let (c, b) = (position(&compiled, "c"), position(&compiled, "b"));
        (b.x - c.x, b.y - c.y)
    };
    assert_eq!(pitch(164.0, 68.0), (180.0, 84.0));
    // The control: a different card moves the grid, so the cell is measured.
    assert_eq!(pitch(200.0, 100.0), (216.0, 116.0));
}

#[test]
fn an_option_overrides_a_measured_parameter() {
    let mut recipe = definition("grid", "x");
    recipe
        .arrangement
        .options
        .insert("cell_width".into(), "300".into());
    let compiled = ProjectionCompiler::new(sizes(164.0, 68.0))
        .compile(&recipe, &dataset())
        .unwrap();
    assert_eq!(position(&compiled, "b").x - position(&compiled, "c").x, 316.0);
}

#[test]
fn unknown_ids_and_options_are_typed_issues() {
    let compiler = ProjectionCompiler::new(sizes(164.0, 68.0));
    let issues = compiler
        .compile(&definition("made-up-layout", "x"), &dataset())
        .unwrap_err();
    assert!(fields(&issues).contains(&"arrangement.kind"));

    let mut recipe = definition("grid", "x");
    recipe.arrangement.options.insert("bogus".into(), "1".into());
    recipe.arrangement.options.insert("columns".into(), "zero".into());
    let issues = compiler.compile(&recipe, &dataset()).unwrap_err();
    assert!(fields(&issues).contains(&"arrangement.options.bogus"));
    assert!(fields(&issues).contains(&"arrangement.options.columns"));
}

#[test]
fn a_wrongly_typed_channel_is_a_typed_issue() {
    let compiler = ProjectionCompiler::new(sizes(164.0, 68.0));
    // Kanban buckets by text; a numeric x is refused by name.
    let issues = compiler
        .compile(&definition("kanban", "x"), &dataset())
        .unwrap_err();
    assert!(fields(&issues).contains(&"encoding.x"));
    // Stack layers must be whole.
    let mut data = dataset();
    data.occurrences[0]
        .values
        .insert("layer".into(), ProjectionValue::Number(1.5));
    let issues = compiler
        .compile(&definition("stack", "layer"), &data)
        .unwrap_err();
    assert!(fields(&issues).contains(&"occurrences.c.values.layer"));
}

#[test]
fn order_families_follow_x() {
    let compiled = ProjectionCompiler::new(sizes(164.0, 68.0))
        .compile(&definition("spiral", "x"), &dataset())
        .unwrap();
    // x is c 0, b 1, a 2: the reverse of b and a in id order.
    assert_eq!(compiled.occurrence_by_instance[&InstanceId(0)], "c");
    assert_eq!(compiled.occurrence_by_instance[&InstanceId(1)], "b");
    assert_eq!(compiled.occurrence_by_instance[&InstanceId(2)], "a");
}

/// Places items along a line, one host-chosen step apart.
struct Line;

impl Solver for Line {
    fn capability(&self) -> SolverCapability {
        let mut capability = SolverCapability::new("test.line", "Line");
        capability.options = vec![OptionSpec::new(
            "step",
            "Step",
            OptionKind::Positive,
            OptionDefault::Value("10".into()),
        )];
        capability
    }

    fn place(
        &self,
        config: &serde_json::Value,
        items: &[&ScoreItem],
    ) -> Result<Vec<Vec2>, SolveError> {
        let step: f32 = config
            .get("step")
            .and_then(serde_json::Value::as_str)
            .and_then(|step| step.parse().ok())
            .unwrap_or(10.0);
        Ok((0..items.len())
            .map(|index| Vec2::new(index as f32 * step, 0.0))
            .collect())
    }
}

#[test]
fn a_registered_solver_compiles_through_the_registry() {
    let mut registry = SolverRegistry::new();
    registry.register(Arc::new(Line)).unwrap();
    let compiler = ProjectionCompiler::with_registry(sizes(164.0, 68.0), registry);
    let mut recipe = definition("test.line", "x");
    recipe.arrangement.options.insert("step".into(), "25".into());
    let compiled = compiler.compile(&recipe, &dataset()).unwrap();
    assert_eq!(compiled.scene.items[2].transform.translate, Vec2::new(50.0, 0.0));

    // The control: without the registration the id names nothing.
    let issues = ProjectionCompiler::new(sizes(164.0, 68.0))
        .compile(&recipe, &dataset())
        .unwrap_err();
    assert!(fields(&issues).contains(&"arrangement.kind"));
}

/// A registered solver's options are judged against its declaration when the
/// projection compiles, before it solves (SE8).
#[test]
fn a_solvers_options_are_judged_against_its_declaration() {
    let mut registry = SolverRegistry::new();
    registry.register(Arc::new(Line)).unwrap();
    let compiler = ProjectionCompiler::with_registry(sizes(164.0, 68.0), registry);
    let issues_for = |key: &str, value: &str| {
        let mut recipe = definition("test.line", "x");
        recipe.arrangement.options.insert(key.into(), value.into());
        compiler
            .compile(&recipe, &dataset())
            .err()
            .unwrap_or_default()
            .into_iter()
            .map(|issue| (issue.field, issue.message))
            .collect::<Vec<_>>()
    };
    assert_eq!(issues_for("step", "25"), Vec::new());
    assert_eq!(
        issues_for("step", "-1"),
        [(
            "arrangement.options.step".to_string(),
            "needs a positive finite number".to_string()
        )]
    );
    assert_eq!(
        issues_for("stride", "2"),
        [(
            "arrangement.options.stride".to_string(),
            "test.line does not read this option".to_string()
        )]
    );
}

#[test]
fn two_hosts_compile_one_recipe_to_one_score() {
    let recipe = definition("radial", "x");
    let first = ProjectionCompiler::new(sizes(164.0, 68.0))
        .compile(&recipe, &dataset())
        .unwrap();
    let second = ProjectionCompiler::new(sizes(164.0, 68.0))
        .compile(&recipe, &dataset())
        .unwrap();
    assert_eq!(
        serde_json::to_vec(&first.score).unwrap(),
        serde_json::to_vec(&second.score).unwrap()
    );
    assert_eq!(first.scene, second.scene);
}

/// A degenerate card is a typed issue (S32), and a usable one compiles (the
/// control).
#[test]
fn a_degenerate_card_is_a_typed_issue() {
    for (w, h) in [
        (0.0, 0.0),
        (164.0, 0.0),
        (-1.0, 68.0),
        (f32::NAN, 68.0),
        (164.0, f32::INFINITY),
    ] {
        let issues = ProjectionCompiler::new(sizes(w, h))
            .compile(&definition("grid", "x"), &dataset())
            .unwrap_err();
        assert!(
            issues.iter().any(|issue| issue.field == "items.card"),
            "{w} by {h}: {issues:?}"
        );
    }
    assert!(
        ProjectionCompiler::new(sizes(1.0, 1.0))
            .compile(&definition("grid", "x"), &dataset())
            .is_ok()
    );
}
