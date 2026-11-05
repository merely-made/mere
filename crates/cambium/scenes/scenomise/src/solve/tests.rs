// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use sceno::{HeldPlacement, Representation, Size2, SourceRef};

fn card(id: u32, ordinal: u32) -> ScoreItem {
    ScoreItem {
        source: SourceRef::new("fixture", id.to_string()),
        ordinal,
        footprint: Footprint::Rect {
            size: Size2::new(36.0, 36.0),
        },
        representation: Representation::Card,
        placement: Placement::Ordinal,
        layer: 0,
        visible: true,
        axis: None,
        embedding: None,
        weight: None,
    }
}

#[test]
fn a_hold_outranks_the_arrangement_in_every_family() {
    // The audit that opened A6: Coordinate was honored by Geographic and
    // Hulls and silently dropped by Spiral and Grid. A hold is honored by
    // every family or it is not a hold — so this list is every family, and
    // absorbing the `arrangements` catalog extended it rather than leaving
    // seven new ways to quietly move a pin.
    let at = Vec2::new(140.0, -60.0);
    for arrangement in [
        Arrangement::Spiral(Spiral::default()),
        Arrangement::Grid(Grid::default()),
        Arrangement::Geographic(Geographic::default()),
        Arrangement::Hulls(Hulls::default()),
        Arrangement::Stack(sceno::Stack::default()),
        Arrangement::Penrose(sceno::Penrose::default()),
        Arrangement::LSystem(sceno::LSystem::default()),
        Arrangement::Timeline(sceno::Timeline::default()),
        Arrangement::Kanban(sceno::Kanban::default()),
        Arrangement::Embedded(sceno::Embedded::default()),
        Arrangement::Radial(sceno::Radial::default()),
    ] {
        let mut score = Score::new(arrangement.clone());
        score.items.push(card(0, 0));
        score.items.push(card(1, 1));
        score
            .holds
            .push(HeldPlacement::pinned(SourceRef::new("fixture", "1"), at));

        let scene = solve(&score);
        let held = scene
            .items
            .iter()
            .find(|item| scene.sources[item.source.0 as usize].id == "1")
            .expect("the held item is placed");
        assert_eq!(
            held.transform.translate, at,
            "{arrangement:?} ignored an authored hold"
        );
    }
}

#[test]
fn every_family_places_every_item() {
    // Absorbing seven families is only done if each one actually places.
    // A family that silently returned an empty plan would pass every
    // hold test above and still lose the whole score.
    for arrangement in [
        Arrangement::Stack(sceno::Stack::default()),
        Arrangement::Penrose(sceno::Penrose::default()),
        Arrangement::LSystem(sceno::LSystem::default()),
        Arrangement::Timeline(sceno::Timeline::default()),
        Arrangement::Kanban(sceno::Kanban::default()),
        Arrangement::Embedded(sceno::Embedded::default()),
        Arrangement::Radial(sceno::Radial::default()),
    ] {
        let mut score = Score::new(arrangement.clone());
        for id in 0..6 {
            score.items.push(card(id, id));
        }
        let scene = solve(&score);
        assert_eq!(scene.items.len(), 6, "{arrangement:?} lost items");
        for item in &scene.items {
            let at = item.transform.translate;
            assert!(
                at.x.is_finite() && at.y.is_finite(),
                "{arrangement:?} -> {at:?}"
            );
        }
    }
}

#[test]
fn a_custom_arrangement_places_nothing_here_and_says_which_pins_went_unmet() {
    // Custom dispatches through the scenograph registry. This crate cannot
    // name the solver, so it places nothing rather than inventing positions
    // — and still reports the pins, so "I could not solve this" is
    // distinguishable from "the score was empty".
    let mut score = Score::new(Arrangement::Custom {
        id: "nobody.registered.this".to_string(),
        config: serde_json::json!({}),
    });
    score.items.push(card(0, 0));
    score.holds.push(HeldPlacement::pinned(
        SourceRef::new("fixture", "0"),
        Vec2::new(1.0, 2.0),
    ));

    let scene = solve(&score);
    assert!(scene.items.is_empty(), "an unknown solver places nothing");
    assert_eq!(scene.unmet_holds.len(), 1, "but the pin is still reported");
    assert_eq!(scene.unmet_holds[0].source.id, "0");
}

#[test]
fn an_unheld_score_places_exactly_as_before() {
    // Holds are additive: the empty case must not perturb any receipt.
    let mut score = Score::new(Arrangement::Spiral(Spiral::default()));
    for id in 0..5 {
        score.items.push(card(id, id));
    }
    let before = solve(&score);
    score.holds.clear();
    let after = solve(&score);
    assert_eq!(before.items.len(), after.items.len());
    for (a, b) in before.items.iter().zip(&after.items) {
        assert_eq!(a.transform.translate, b.transform.translate);
    }
}

#[test]
fn pinned_instances_finds_every_instance_of_a_pinned_source() {
    let mut score = Score::new(Arrangement::Spiral(Spiral::default()));
    // One source, twice: legal by design, and both instances are pinned.
    score.items.push(card(0, 0));
    score.items.push(card(0, 1));
    score.items.push(card(1, 2));
    score.holds.push(HeldPlacement::pinned(
        SourceRef::new("fixture", "0"),
        Vec2::new(5.0, 5.0),
    ));
    score.holds.push(HeldPlacement::anchored(
        SourceRef::new("fixture", "1"),
        Vec2::new(9.0, 9.0),
    ));

    let scene = solve(&score);
    let pinned = pinned_instances(&score, &scene);
    assert_eq!(pinned.len(), 2, "both instances of source 0 are pinned");
    // Anchored returns rather than holds, so it is not immovable.
    for instance in &pinned {
        assert_eq!(
            scene.sources[scene.items[instance.0 as usize].source.0 as usize].id,
            "0"
        );
    }
}

#[test]
fn a_satisfied_pin_is_recorded_against_its_instance() {
    let mut score = Score::new(Arrangement::Spiral(Spiral::default()));
    score.items.push(card(0, 0));
    score.items.push(card(1, 1));
    score.holds.push(HeldPlacement::pinned(
        SourceRef::new("fixture", "1"),
        Vec2::new(9.0, -3.0),
    ));
    let scene = solve(&score);

    assert_eq!(scene.honored_holds.len(), 1);
    let honored = &scene.honored_holds[0];
    assert_eq!(honored.placement.source.id, "1");
    // Bound to the instance, so a consumer never re-derives the mapping.
    let placed = scene.items[honored.instance.0 as usize].transform.translate;
    assert_eq!(placed, Vec2::new(9.0, -3.0));
    assert!(scene.unmet_holds.is_empty());
}

#[test]
fn a_seeded_or_anchored_hold_is_not_recorded_as_honored() {
    // Neither is binding. Recording one as honored would invite a later
    // pass to treat it as a pin, which is the opposite of what both mean.
    for held in [HeldPlacement::seeded, HeldPlacement::anchored] {
        let mut score = Score::new(Arrangement::Spiral(Spiral::default()));
        score.items.push(card(0, 0));
        score
            .holds
            .push(held(SourceRef::new("fixture", "0"), Vec2::new(1.0, 1.0)));
        assert!(solve(&score).honored_holds.is_empty());
    }
}

#[test]
fn every_instance_of_a_held_source_is_recorded() {
    let mut score = Score::new(Arrangement::Spiral(Spiral::default()));
    score.items.push(card(0, 0));
    score.items.push(card(0, 1));
    score.holds.push(HeldPlacement::pinned(
        SourceRef::new("fixture", "0"),
        Vec2::new(2.0, 2.0),
    ));
    assert_eq!(solve(&score).honored_holds.len(), 2);
}

#[test]
fn an_unsatisfiable_pin_is_reported_on_the_scene() {
    let mut score = Score::new(Arrangement::Spiral(Spiral::default()));
    score.items.push(card(0, 0));
    score.holds.push(HeldPlacement::pinned(
        SourceRef::new("fixture", "ghost"),
        Vec2::new(7.0, 7.0),
    ));

    let scene = solve(&score);
    assert_eq!(scene.items.len(), 1, "the ghost is not invented as an item");
    assert_eq!(scene.unmet_holds.len(), 1, "the violation is carried");
    assert_eq!(scene.unmet_holds[0].source.id, "ghost");
    // The coordinate travels too, so a viewer without the score can say
    // what was asked for, not merely that something failed.
    assert_eq!(scene.unmet_holds[0].at, Vec2::new(7.0, 7.0));
}

#[test]
fn a_satisfied_pin_reports_nothing() {
    let mut score = Score::new(Arrangement::Spiral(Spiral::default()));
    score.items.push(card(0, 0));
    score.holds.push(HeldPlacement::pinned(
        SourceRef::new("fixture", "0"),
        Vec2::new(4.0, 4.0),
    ));
    let scene = solve(&score);
    assert!(scene.unmet_holds.is_empty());
    assert_eq!(scene.items[0].transform.translate, Vec2::new(4.0, 4.0));
}

#[test]
fn an_unplaced_seed_or_anchor_is_not_a_violation() {
    // Neither must be honored, so its absence is not reported; only a pin
    // earns a violation.
    for held in [HeldPlacement::seeded, HeldPlacement::anchored] {
        let mut score = Score::new(Arrangement::Spiral(Spiral::default()));
        score.items.push(card(0, 0));
        score.holds.push(held(
            SourceRef::new("fixture", "ghost"),
            Vec2::new(7.0, 7.0),
        ));
        assert!(solve(&score).unmet_holds.is_empty());
    }
}

fn overlaps(a: &Rect, b: &Rect) -> bool {
    a.origin.x < b.origin.x + b.size.w
        && b.origin.x < a.origin.x + a.size.w
        && a.origin.y < b.origin.y + b.size.h
        && b.origin.y < a.origin.y + a.size.h
}

fn site(id: u32, x: f32, y: f32) -> ScoreItem {
    ScoreItem {
        source: SourceRef::new("fixture", id.to_string()),
        ordinal: id,
        footprint: Footprint::Point,
        representation: Representation::Glyph,
        placement: Placement::Coordinate(Vec2::new(x, y)),
        layer: 0,
        visible: true,
        axis: None,
        embedding: None,
        weight: None,
    }
}

fn hulls_score(sites: &[(f32, f32)]) -> Score {
    let mut score = Score::new(Arrangement::Hulls(Hulls {
        origin: Vec2::ZERO,
        units_per_coordinate: 1.0,
        invert_y: false,
        bounds: Rect::new(Vec2::new(-100.0, -100.0), Size2::new(200.0, 200.0)),
    }));
    for (index, (x, y)) in sites.iter().enumerate() {
        score.items.push(site(index as u32, *x, *y));
    }
    score
}

fn polygon_area(points: &[Vec2]) -> f32 {
    let mut doubled = 0.0;
    for (index, a) in points.iter().enumerate() {
        let b = points[(index + 1) % points.len()];
        doubled += a.x * b.y - b.x * a.y;
    }
    (doubled / 2.0).abs()
}

fn contains(points: &[Vec2], p: Vec2) -> bool {
    // Even-odd ray cast, good enough for convex cells in a test.
    let mut inside = false;
    for (index, a) in points.iter().enumerate() {
        let b = points[(index + 1) % points.len()];
        if (a.y > p.y) != (b.y > p.y) && p.x < a.x + (b.x - a.x) * (p.y - a.y) / (b.y - a.y) {
            inside = !inside;
        }
    }
    inside
}

#[test]
fn one_site_owns_the_whole_bounds() {
    let scene = solve(&hulls_score(&[(10.0, -5.0)]));
    assert_eq!(scene.regions.len(), 1);
    let Some(Footprint::Polygon { points }) = &scene.regions[0].contour else {
        panic!("a cell is a polygon");
    };
    assert!((polygon_area(points) - 200.0 * 200.0).abs() < 1.0);
}

#[test]
fn cells_tile_the_bounds() {
    // Cover without overlap: the areas of the cells sum to the area of the
    // bounds, which a halo arrangement could never promise.
    let scene = solve(&hulls_score(&[
        (-40.0, -40.0),
        (60.0, 10.0),
        (0.0, 55.0),
        (-20.0, 80.0),
    ]));
    assert_eq!(scene.regions.len(), 4);

    let total: f32 = scene
        .regions
        .iter()
        .map(|region| match &region.contour {
            Some(Footprint::Polygon { points }) => polygon_area(points),
            _ => panic!("every cell is a polygon"),
        })
        .sum();
    assert!((total - 200.0 * 200.0).abs() < 1.0, "cells sum to {total}");
}

#[test]
fn a_cell_is_the_nearest_site_rule_made_visible() {
    // The property that makes hulls over Mesocosm's places exact: any
    // sampled point lies in the cell of the site it is nearest.
    let sites = [(-40.0, -40.0), (60.0, 10.0), (0.0, 55.0)];
    let scene = solve(&hulls_score(&sites));

    for sample in [
        Vec2::new(-70.0, -70.0),
        Vec2::new(80.0, 20.0),
        Vec2::new(5.0, 90.0),
        Vec2::new(-10.0, -3.0),
    ] {
        let nearest = (0..sites.len())
            .min_by(|a, b| {
                let d =
                    |i: usize| (sample.x - sites[i].0).powi(2) + (sample.y - sites[i].1).powi(2);
                d(*a).total_cmp(&d(*b))
            })
            .unwrap();
        let holder = scene
            .regions
            .iter()
            .position(|region| match &region.contour {
                Some(Footprint::Polygon { points }) => contains(points, sample),
                _ => false,
            })
            .expect("every point in bounds is in some cell");
        assert_eq!(
            scene.regions[holder].members,
            vec![sceno::InstanceId(nearest as u32)],
            "the cell holding {sample:?} belongs to the nearest site"
        );
    }
}

#[test]
fn each_cell_contains_its_own_site() {
    let sites = [(-40.0, -40.0), (60.0, 10.0), (0.0, 55.0), (-20.0, 80.0)];
    let scene = solve(&hulls_score(&sites));
    for region in &scene.regions {
        let Some(Footprint::Polygon { points }) = &region.contour else {
            panic!("polygon");
        };
        let site = &scene.items[region.members[0].0 as usize];
        assert!(contains(
            points,
            Vec2::new(site.transform.translate.x, site.transform.translate.y)
        ));
    }
}

#[test]
fn an_invisible_item_is_not_a_site() {
    let mut score = hulls_score(&[(-40.0, 0.0), (40.0, 0.0)]);
    score.items[1].visible = false;
    let scene = solve(&score);
    assert_eq!(scene.regions.len(), 1, "only the visible site got a cell");
}

#[test]
fn a_hulls_score_round_trips() {
    let score = hulls_score(&[(1.0, 2.0)]);
    let json = serde_json::to_string(&score).unwrap();
    assert_eq!(serde_json::from_str::<Score>(&json).unwrap(), score);
}

#[test]
fn spiral_grows_spacing_to_clear_measured_cards() {
    let mut score = Score::new(Arrangement::Spiral(Spiral::default()));
    score.items = (0..15).map(|i| card(i, i)).collect();
    let scene = solve(&score);
    let bounds: Vec<_> = scene
        .items
        .iter()
        .map(|item| placed_bounds(item.transform.translate, &item.footprint).unwrap())
        .collect();
    for (index, a) in bounds.iter().enumerate() {
        for b in &bounds[index + 1..] {
            assert!(!overlaps(a, b), "measured cards must not overlap");
        }
    }
}

#[test]
fn grid_keeps_authored_cells_and_geographic_keeps_coordinates() {
    let mut grid_score = Score::new(Arrangement::Grid(Grid::default()));
    let mut placed = card(1, 0);
    placed.placement = Placement::Cell { column: 3, row: 2 };
    grid_score.items.push(placed);
    let grid = solve(&grid_score);
    assert_eq!(grid.items[0].transform.translate, Vec2::new(204.0, 136.0));

    let mut geo_score = Score::new(Arrangement::Geographic(Geographic::default()));
    let mut point = card(2, 0);
    point.placement = Placement::Coordinate(Vec2::new(12.0, 4.0));
    geo_score.items.push(point);
    let geographic = solve(&geo_score);
    assert_eq!(
        geographic.items[0].transform.translate,
        Vec2::new(12.0, -4.0)
    );
}

#[test]
fn score_order_drives_spiral_not_source_identity() {
    let mut score = Score::new(Arrangement::Spiral(Spiral::default()));
    score.items.push(card(9, 10));
    score.items.push(card(3, 0));
    let scene = solve(&score);
    assert_eq!(scene.sources[scene.items[0].source.0 as usize].id, "3");
    assert_eq!(scene.items[0].transform.translate, Vec2::ZERO);
}

#[test]
fn geographic_map_fixture_keeps_disclosed_coordinates_and_lod() {
    let score: Score = serde_json::from_str(include_str!("../../fixtures/coastal_map.json"))
        .expect("the portable geographic fixture parses");
    let scene = solve(&score);
    assert_eq!(scene.generation, 17);
    let position_for = |id: &str| {
        scene
            .items
            .iter()
            .find(|item| scene.sources[item.source.0 as usize].id == id)
            .map(|item| item.transform.translate)
            .expect("fixture source is realized")
    };
    assert_eq!(position_for("harbor"), Vec2::new(90.0, 194.0));
    assert_eq!(position_for("beacon"), Vec2::new(108.0, 204.0));
    assert_eq!(position_for("ridge"), Vec2::new(118.0, 184.0));
    let ridge = scene
        .items
        .iter()
        .find(|item| scene.sources[item.source.0 as usize].id == "ridge")
        .unwrap();
    assert_eq!(ridge.representation, Representation::Snapshot);
    assert!(
        scene.items.iter().any(|item| item.layer == -10
            && scene.sources[item.source.0 as usize].adapter == "fixture.map-underlay"),
        "the map underlay is an ordinary low-layer scene item"
    );
}
