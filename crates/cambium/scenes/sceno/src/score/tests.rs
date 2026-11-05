// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::Size2;

#[test]
fn score_round_trips_through_serde() {
    let mut score = Score::new(Arrangement::Spiral(Spiral::default()));
    score.generation = 7;
    score.items.push(ScoreItem {
        source: SourceRef::new("fixture", "north"),
        ordinal: 0,
        footprint: Footprint::Rect {
            size: Size2::new(40.0, 24.0),
        },
        representation: Representation::Card,
        placement: Placement::Ordinal,
        layer: 2,
        visible: true,
        axis: Some(AxisValue::Numeric(1861.0)),
        embedding: Some(Vec2::new(0.25, -0.5)),
        weight: Some(0.75),
    });
    let json = serde_json::to_string(&score).unwrap();
    assert_eq!(serde_json::from_str::<Score>(&json).unwrap(), score);
}

#[test]
fn score_names_regular_cells_grid_on_the_wire() {
    let score = Score::new(Arrangement::Grid(Grid::default()));
    let json = serde_json::to_string(&score).unwrap();
    assert!(json.contains("\"Grid\""));
    assert!(!json.contains("\"Board\""));
}

#[test]
fn score_stamps_v5() {
    assert_eq!(SCORE_VERSION, 5);
    assert_eq!(Score::new(Arrangement::Grid(Grid::default())).version, 5);
}

/// A v3 score predates the three disclosure fields entirely. It must still
/// load — the fields are `#[serde(default)]` precisely so an older score
/// stays readable — and every one of them must arrive absent rather than
/// zeroed, because `Some(0.0)` and `None` mean different things to every
/// arrangement that reads them.
#[test]
fn v3_score_loads_with_disclosure_fields_absent() {
    // Build the v3 wire by serializing a current score and removing exactly
    // what v3 did not have. Deriving the fixture rather than transcribing it
    // keeps this test honest about the real encoding: a rename anywhere in
    // `ScoreItem` changes both sides together, so what is asserted stays
    // "the three new fields are optional" and never decays into "this
    // hand-written blob still parses".
    let mut score = Score::new(Arrangement::Grid(Grid::default()));
    score.items.push(ScoreItem {
        source: SourceRef::new("fixture", "north"),
        ordinal: 0,
        footprint: Footprint::Rect {
            size: Size2::new(40.0, 24.0),
        },
        representation: Representation::Card,
        placement: Placement::Ordinal,
        layer: 2,
        visible: true,
        axis: Some(AxisValue::Numeric(1.0)),
        embedding: Some(Vec2::new(1.0, 1.0)),
        weight: Some(1.0),
    });

    let mut wire = serde_json::to_value(&score).unwrap();
    wire["version"] = serde_json::json!(3);
    let item = wire["items"][0].as_object_mut().unwrap();
    for field in ["axis", "embedding", "weight"] {
        assert!(
            item.remove(field).is_some(),
            "{field} must be on the v4 wire for this test to prove anything"
        );
    }

    let loaded: Score = serde_json::from_value(wire).expect("v3 score must still load");
    assert_eq!(
        loaded.version, SCORE_VERSION,
        "read into the current meaning"
    );
    let item = &loaded.items[0];
    // Absent, not zeroed: `Some(0.0)` and `None` mean different things to
    // every arrangement that reads these.
    assert!(item.axis.is_none());
    assert!(item.embedding.is_none());
    assert!(item.weight.is_none());
    // The fields v3 did have must survive untouched.
    assert_eq!(item.source, SourceRef::new("fixture", "north"));
    assert_eq!(item.layer, 2);
}

/// Every family in the catalog has to survive the wire, including the
/// untyped escape. A variant that serializes but cannot be read back is a
/// score that silently loses its arrangement.
#[test]
fn every_arrangement_round_trips() {
    let arrangements = vec![
        Arrangement::Spiral(Spiral::default()),
        Arrangement::Grid(Grid::default()),
        Arrangement::Geographic(Geographic::default()),
        Arrangement::Hulls(Hulls::default()),
        Arrangement::Stack(Stack::default()),
        Arrangement::Penrose(Penrose::default()),
        Arrangement::LSystem(LSystem::default()),
        Arrangement::Timeline(Timeline::default()),
        Arrangement::Kanban(Kanban::default()),
        Arrangement::Embedded(Embedded::default()),
        Arrangement::Radial(Radial::default()),
        Arrangement::Custom {
            id: "isometry.campaign".to_string(),
            config: serde_json::json!({ "lanes": 3 }),
        },
    ];
    for arrangement in arrangements {
        let score = Score::new(arrangement.clone());
        let json = serde_json::to_string(&score).unwrap();
        let back: Score = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("{arrangement:?} failed to round trip: {e}"));
        assert_eq!(back.arrangement, arrangement);
    }
}

/// `AxisValue` carries two shapes that must stay distinguishable on the
/// wire: a timeline coordinate and a kanban column tag are not the same
/// disclosure, and an arrangement reading the wrong one places silently.
#[test]
fn axis_value_discriminates_numeric_from_categorical() {
    let numeric = AxisValue::Numeric(-3.5);
    let categorical = AxisValue::Categorical("in-review".to_string());
    for value in [numeric, categorical] {
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(serde_json::from_str::<AxisValue>(&json).unwrap(), value);
    }
}

#[test]
fn holds_round_trip_and_resolve_by_source() {
    let mut score = Score::new(Arrangement::Spiral(Spiral::default()));
    let pinned = SourceRef::new("fixture", "north");
    score
        .holds
        .push(HeldPlacement::pinned(pinned.clone(), Vec2::new(12.0, -4.0)));
    score.holds.push(HeldPlacement::anchored(
        SourceRef::new("fixture", "south"),
        Vec2::new(0.0, 9.0),
    ));

    let json = serde_json::to_string(&score).unwrap();
    assert_eq!(serde_json::from_str::<Score>(&json).unwrap(), score);

    let held = score.hold_for(&pinned).expect("north is held");
    assert_eq!(held.hold, Hold::Pinned);
    assert_eq!(held.at, Vec2::new(12.0, -4.0));
    assert!(score.hold_for(&SourceRef::new("fixture", "east")).is_none());
    // Same id, different adapter, is a different source.
    assert!(score.hold_for(&SourceRef::new("other", "north")).is_none());
}

/// F25, F44: a version-5 score round-trips all three holds, byte for
/// byte, and a new anchored hold reads back as anchored.
#[test]
fn three_holds_round_trip() {
    let mut score = Score::new(Arrangement::Spiral(Spiral::default()));
    for (id, hold) in [
        ("seeded", Hold::Seeded),
        ("anchored", Hold::Anchored),
        ("pinned", Hold::Pinned),
    ] {
        score.holds.push(HeldPlacement {
            source: SourceRef::new("fixture", id),
            at: Vec2::new(1.0, 2.0),
            hold,
        });
    }
    let json = serde_json::to_string(&score).unwrap();
    assert!(json.contains(r#""version":5"#));
    let back: Score = serde_json::from_str(&json).unwrap();
    assert_eq!(back, score);
    assert_eq!(serde_json::to_string(&back).unwrap(), json, "byte for byte");
    assert_eq!(
        back.holds[1].hold,
        Hold::Anchored,
        "a new anchored hold returns"
    );
}

/// F44: a version-4 score's "Anchored" was best effort, and reads as
/// `Seeded`; its "Pinned" stays pinned. Written again, it is a version-5
/// score whose seeded hold reads back as seeded.
#[test]
fn an_old_scores_anchored_hold_reads_as_seeded() {
    let json = r#"{
            "version": 4,
            "arrangement": {"Spiral": {"center": {"x": 0.0, "y": 0.0},
                "spacing": 40.0, "angle_radians": 2.399963, "curve": "SquareRoot"}},
            "items": [],
            "holds": [{"source": {"adapter": "fixture", "id": "old"},
                "at": {"x": 1.0, "y": 2.0}, "hold": "Anchored"}],
            "generation": 3
        }"#;
    let score: Score = serde_json::from_str(json).unwrap();
    assert_eq!(
        score.holds[0].hold,
        Hold::Seeded,
        "a v4 anchored hold was best effort, the seeded role"
    );
    let pinned = json.replace(r#""hold": "Anchored""#, r#""hold": "Pinned""#);
    let pinned: Score = serde_json::from_str(&pinned).unwrap();
    assert_eq!(pinned.holds[0].hold, Hold::Pinned, "a v4 pin is a pin");
    let again: Score = serde_json::from_str(&serde_json::to_string(&score).unwrap()).unwrap();
    assert_eq!(again.version, 5);
    assert_eq!(again.holds[0].hold, Hold::Seeded, "and stays seeded");
}

/// F44: a reader refuses a score newer than itself. A version-4 reader
/// meets a version-5 score with an anchored hold; the current reader
/// meets a version-6 score. Each reads its own version (the control).
#[test]
fn a_reader_refuses_a_newer_score() {
    let mut score = Score::new(Arrangement::Spiral(Spiral::default()));
    score.holds.push(HeldPlacement::anchored(
        SourceRef::new("fixture", "new"),
        Vec2::new(1.0, 2.0),
    ));
    let wire = |version: u16| {
        let mut value = serde_json::to_value(&score).unwrap();
        value["version"] = serde_json::json!(version);
        serde_json::from_value::<ScoreWire>(value).unwrap()
    };
    assert_eq!(
        Score::read(wire(5), 4).unwrap_err(),
        ScoreVersionError {
            found: 5,
            reader: 4
        }
    );
    assert!(Score::read(wire(4), 4).is_ok(), "a v4 reader reads v4");
    let mut six = serde_json::to_value(&score).unwrap();
    six["version"] = serde_json::json!(6);
    let refused = serde_json::from_value::<Score>(six).unwrap_err();
    assert_eq!(
        refused.to_string(),
        "score version 6 is newer than this reader's 5"
    );
    assert!(serde_json::from_value::<Score>(serde_json::to_value(&score).unwrap()).is_ok());
}

#[test]
fn a_version_1_score_still_reads_with_no_holds() {
    // The wire before holds existed. It must load, and it must load as
    // "nothing was held", never as "holds unknown".
    let json = r#"{
            "version": 1,
            "arrangement": {"Spiral": {"center": {"x": 0.0, "y": 0.0},
                "spacing": 40.0, "angle_radians": 2.399963, "curve": "SquareRoot"}},
            "items": [],
            "generation": 3
        }"#;
    let score: Score = serde_json::from_str(json).unwrap();
    assert_eq!(
        score.version, SCORE_VERSION,
        "read into the current meaning"
    );
    assert_eq!(score.generation, 3);
    assert!(score.holds.is_empty());
}
