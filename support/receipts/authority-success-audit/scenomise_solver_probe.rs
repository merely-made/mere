//! Public API diagnostics; assertions describe the observed defects, not repairs.
use std::collections::BTreeMap;
use sceno::{Arrangement, AxisValue, Footprint, IterationDepth, LSystem, LSystemGrammar,
    Placement, Representation, Score, ScoreItem, Size2, SourceRef, Timeline};
use scenomise::catalog::{Family, Measure};
use scenotime::{SceneSnapshot, SceneEpoch, Revision};

fn item(ordinal: u32, axis: Option<f64>) -> ScoreItem {
    ScoreItem {
        source: SourceRef::new("audit", ordinal.to_string()), ordinal,
        footprint: Footprint::Rect { size: Size2::new(10.0, 10.0) },
        representation: Representation::Glyph, placement: Placement::Ordinal,
        layer: 0, visible: true, axis: axis.map(AxisValue::Numeric),
        embedding: None, weight: None,
    }
}
fn timeline(values: &[f64]) -> sceno::Scene {
    assert!(values.iter().all(|value| value.is_finite()));
    let mut score = Score::new(Arrangement::Timeline(Timeline::default()));
    score.items = values.iter().enumerate().map(|(i, value)| item(i as u32, Some(*value))).collect();
    let json = serde_json::to_vec(&score).unwrap();
    let restored: Score = serde_json::from_slice(&json).unwrap();
    assert_eq!(restored, score, "the input score reopens exactly");
    scenomise::solve(&restored)
}

#[test]
fn normal_finite_timeline_control() {
    let scene = timeline(&[0.0, 100.0]);
    assert_eq!(scene.items[0].transform.translate.x, Timeline::default().origin.x);
    assert_eq!(scene.items[1].transform.translate.x, Timeline::default().origin.x + Timeline::default().axis_length);
    let snapshot = SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene).unwrap();
    let bytes = serde_json::to_vec(&snapshot).unwrap();
    assert_eq!(serde_json::from_slice::<SceneSnapshot>(&bytes).unwrap(), snapshot);
}

#[test]
fn finite_large_axis_generates_nan_then_a_non_reopenable_snapshot() {
    let scene = timeline(&[0.0, 1e100]);
    assert!(scene.items[1].transform.translate.x.is_nan());
    let snapshot = SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene).unwrap();
    let bytes = serde_json::to_vec(&snapshot).unwrap();
    assert!(serde_json::from_slice::<SceneSnapshot>(&bytes).is_err());
    println!("Finite [0, 1e100] score reopens, solve emits NaN, admitted snapshot JSON cannot reopen");
}

#[test]
fn finite_tiny_axis_collapses_distinct_endpoints() {
    let scene = timeline(&[0.0, 1e-100]);
    assert_eq!(scene.items[0].transform.translate.x, scene.items[1].transform.translate.x);
    assert_ne!(scene.items[1].transform.translate.x, Timeline::default().axis_length);
    println!("Distinct finite [0, 1e-100] axis values both map to origin");
}

#[test]
fn named_grammar_is_silently_hilbert() {
    let solve = |grammar| {
        let mut score = Score::new(Arrangement::LSystem(LSystem {
            grammar, iteration_depth: IterationDepth::Explicit(2), ..LSystem::default()
        }));
        score.items = (0..4).map(|i| item(i, None)).collect();
        scenomise::solve(&score)
    };
    assert_eq!(solve(LSystemGrammar::Named("missing-host-grammar".into())), solve(LSystemGrammar::Hilbert));
    println!("Named missing-host-grammar returns an identical scene to Hilbert without an unavailable-grammar result");
}

#[test]
fn depth_255_is_admitted_without_running_the_expansion() {
    let spec = Family::LSystem.options().into_iter().find(|spec| spec.key == "depth").unwrap();
    assert_eq!(spec.kind.refusal("255"), None);
    let configured = Family::LSystem.arrangement(
        &BTreeMap::from([("depth".into(), "255".into())]),
        &Measure { largest: Size2::new(10.0, 10.0), count: 1, spacing: 16.0, coordinates: None },
    ).unwrap();
    assert!(matches!(configured, Arrangement::LSystem(LSystem { iteration_depth: IterationDepth::Explicit(255), .. })));
    let penrose = Family::Penrose.arrangement(
        &BTreeMap::from([("subdivisions".into(), "255".into())]),
        &Measure { largest: Size2::new(10.0, 10.0), count: 1, spacing: 16.0, coordinates: None },
    ).unwrap();
    assert!(matches!(penrose, Arrangement::Penrose(sceno::Penrose {
        subdivision_count: sceno::SubdivisionCount::Explicit(255), ..
    })));
    // Deliberately do not call solve: the source expansion grows exponentially.
    println!("L-system and Penrose arrangement construction accept depth 255 for one item; expansions not executed");
}

#[test]
fn a_negative_fractional_ring_goes_to_the_focus() {
    let solve = |value| {
        let mut score = Score::new(Arrangement::Radial(sceno::Radial::default()));
        score.items = vec![item(0, Some(2.0)), item(1, Some(value))];
        scenomise::solve(&score)
    };
    let negative_integer = solve(-1.0);
    assert!(negative_integer.items[1].transform.translate.x > negative_integer.items[0].transform.translate.x);
    let negative_fraction = solve(-0.1);
    assert_eq!(negative_fraction.items[1].transform.translate, sceno::Radial::default().center);
    println!("-1 takes the outer-ring fallback; -0.1 rounds into the focus instead");
}

#[test]
fn a_maximum_ring_and_an_undisclosed_item_overflow_the_outer_ring() {
    let mut score = Score::new(Arrangement::Radial(sceno::Radial::default()));
    score.items = vec![item(0, Some(f64::from(u32::MAX))), item(1, None)];
    assert!(std::panic::catch_unwind(|| scenomise::solve(&score)).is_err());
    println!("u32::MAX plus an undisclosed ring panics in the native overflow-checked library");
}

#[test]
fn impossible_set_intersections_silently_get_a_partial_membership_position() {
    let names: Vec<String> = (0..6).map(|i| format!("s{i}")).collect();
    let mut score = Score::new(Arrangement::Setscape(sceno::Setscape {
        sets: names.clone(), ..sceno::Setscape::default()
    }));
    let mut member = item(0, None);
    member.axis = Some(AxisValue::Categorical(names.join("|")));
    score.items.push(member);
    let scene = scenomise::solve(&score);
    let at = scene.items[0].transform.translate;
    let contains = |points: &[sceno::Vec2]| {
        let mut inside = false;
        for (i, a) in points.iter().enumerate() {
            let b = points[(i+1) % points.len()];
            if (a.y > at.y) != (b.y > at.y) && at.x < a.x + (b.x-a.x)*(at.y-a.y)/(b.y-a.y) {
                inside = !inside;
            }
        }
        inside
    };
    let sets: Vec<_> = scene.regions.iter().filter(|region| region.label.as_ref().is_some_and(|label| names.contains(label))).collect();
    assert_eq!(sets.len(), 6);
    assert!(sets.iter().all(|region| region.members == [sceno::InstanceId(0)]));
    let actually_inside = sets.iter().filter(|region| match &region.contour {
        Some(Footprint::Polygon { points }) => contains(points), _ => false
    }).count();
    assert!(actually_inside < 6);
    assert!(scene.unmet_holds.is_empty());
    println!("Six declared memberships remain in region data, but the item is geometrically inside only {actually_inside}; no degraded-placement report");
}
