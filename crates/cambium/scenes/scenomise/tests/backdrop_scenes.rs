// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use sceno::{Footprint, Vec2};
use scenomise::backdrops::*;
use scenotime::{Revision, SceneEpoch, SceneSnapshot};

const METRE: &str = include_str!("../../../../../support/design-studies/backdrops/metre.json");
const FRET: &str = include_str!("../../../../../support/design-studies/backdrops/fretboard.json");
const TERRAIN: &str = include_str!("../../../../../support/design-studies/backdrops/terrain.json");

#[test]
fn each_recipe_exports_a_valid_scene_without_product_authorities() {
    for json in [METRE, FRET, TERRAIN] {
        let input = SceneInput::parse(json).unwrap();
        let scene = input.compile().unwrap();
        SceneSnapshot::from_dense(SceneEpoch(1), Revision(0), scene.scene.clone()).unwrap();
        let bytes = serde_json::to_string(&scene).unwrap();
        let reopened: BackdropScene = serde_json::from_str(&bytes).unwrap();
        assert_eq!(scene, reopened);
        assert_eq!(
            scene,
            SceneInput::parse(&serde_json::to_string(&input).unwrap())
                .unwrap()
                .compile()
                .unwrap()
        );
        assert!(scene.scene.backdrops.iter().all(|b| !b.collidable));
        assert_eq!(scene.scene.items.len(), scene.marks.len());
    }
}
#[test]
fn metre_uses_supplied_stress_and_keeps_mismatches_visible() {
    let mut input = SceneInput::parse(METRE).unwrap();
    let SceneFacts::MetreGrid(grid) = &mut input.facts else {
        unreachable!()
    };
    grid.lines[2].rhyme_tail = None;
    grid.lines[0].syllables[0].stressed = true;
    grid.lines[0].syllables.push(Syllable {
        text: "extra".into(),
        stressed: false,
    });
    let out = input.compile().unwrap();
    assert_eq!(out.scene.relations.len(), 2);
    assert_eq!(
        out.scene.relations[0].kind.as_deref(),
        Some("expected.rhyme")
    );
    assert_eq!(
        out.scene.relations[1].kind.as_deref(),
        Some("derived.rhyme")
    );
    assert_eq!(out.diagnostics.len(), 2);
    let stress = out
        .marks
        .iter()
        .find(|m| m.key == "line-1/syllable/0")
        .unwrap();
    assert_eq!(stress.role, MarkRole::StrongStress);
    assert!(
        matches!(out.scene.items[stress.instance.0 as usize].footprint, Footprint::Rect { size } if size.h == 24.0)
    );
    assert!(!stress.selectable);
    assert_eq!(
        out.scene.items[stress.instance.0 as usize].hit,
        Some(Footprint::Point)
    );
}
#[test]
fn fretboard_shares_pitch_identity_but_keeps_occurrences_distinct() {
    let out = SceneInput::parse(FRET).unwrap().compile().unwrap();
    let cs: Vec<_> = out.marks.iter().filter(|m| m.label == "C").collect();
    assert_eq!(cs.len(), 2);
    let a = &out.scene.items[cs[0].instance.0 as usize];
    let b = &out.scene.items[cs[1].instance.0 as usize];
    assert_eq!(a.source, b.source);
    assert_ne!(a.transform.translate, b.transform.translate);
    assert_ne!(cs[0].key, cs[1].key);
    let mut input = SceneInput::parse(FRET).unwrap();
    let SceneFacts::Fretboard(board) = &mut input.facts else {
        unreachable!()
    };
    board.positions[0].pitch_class = 3;
    assert!(input.compile().unwrap_err().reason.contains("tuning"));
}
#[test]
fn unknown_radio_positions_never_become_origin_nodes_or_fields() {
    let input = SceneInput::parse(TERRAIN).unwrap();
    let out = input.compile().unwrap();
    assert_eq!(out.unplaced.len(), 1);
    assert_eq!(out.unplaced[0].key, "pocket");
    assert!(!out.marks.iter().any(|m| m.key == "pocket"));
    assert!(
        out.fields
            .iter()
            .all(|f| f.device.as_deref() != Some("pocket"))
    );
    assert!(
        out.fields
            .iter()
            .any(|f| f.tier.is_none() && f.device.is_none())
    );
    let mut bad = input.clone();
    let SceneFacts::TerrainCoverage(map) = &mut bad.facts else {
        unreachable!()
    };
    map.coverage[0].device = "pocket".into();
    assert!(bad.compile().unwrap_err().reason.contains("placed"));
}
#[test]
fn invalid_coordinates_and_unknown_input_are_refused() {
    let mut input = SceneInput::parse(TERRAIN).unwrap();
    let SceneFacts::TerrainCoverage(map) = &mut input.facts else {
        unreachable!()
    };
    map.devices[0].position = Some(Vec2::new(f32::NAN, 0.0));
    assert!(input.compile().is_err());
    for contour in [
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(2.0, 2.0),
        ],
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100_000.0, 0.0),
            Vec2::new(0.0, 1.0),
        ],
    ] {
        let mut input = SceneInput::parse(TERRAIN).unwrap();
        let SceneFacts::TerrainCoverage(map) = &mut input.facts else {
            unreachable!()
        };
        map.coverage[0].contour = contour;
        assert!(input.compile().is_err());
    }
    for path in ["top", "source", "extent", "position"] {
        let mut json: serde_json::Value = serde_json::from_str(TERRAIN).unwrap();
        let target = match path {
            "top" => &mut json,
            "source" => &mut json["source"],
            "extent" => &mut json["facts"]["data"]["extent"],
            _ => &mut json["facts"]["data"]["devices"][0]["position"],
        };
        target["surprise"] = true.into();
        assert!(SceneInput::parse(&json.to_string()).is_err(), "{path}");
    }
    let mut json: serde_json::Value = serde_json::from_str(METRE).unwrap();
    json["version"] = 2.into();
    assert_eq!(
        SceneInput::parse(&json.to_string()).unwrap_err().path,
        "version"
    );
}
#[test]
fn ambiguous_occurrence_keys_are_refused_and_generation_tracks_facts() {
    let mut input = SceneInput::parse(METRE).unwrap();
    let original = input.compile().unwrap().scene.generation;
    input.revision.push_str("-next");
    assert_ne!(original, input.compile().unwrap().scene.generation);
    let SceneFacts::MetreGrid(grid) = &mut input.facts else {
        unreachable!()
    };
    grid.lines[1].key = "line-1/syllable/0".into();
    assert_eq!(input.compile().unwrap_err().path, "marks");
}

#[test]
fn ternary_template_keeps_actual_syllables_and_discloses_expected_extent() {
    let mut input = SceneInput::parse(METRE).unwrap();
    let SceneFacts::MetreGrid(grid) = &mut input.facts else {
        unreachable!()
    };
    grid.positions_per_foot = 3;
    let reading = input.compile().unwrap();
    assert!(reading.marks[0].detail.contains("15 expected"));
    assert!(reading.diagnostics.iter().any(|d| d.contains("expects 15")));
    let SceneFacts::MetreGrid(grid) = &mut input.facts else {
        unreachable!()
    };
    grid.positions_per_foot = 0;
    assert!(input.compile().is_err());
}
