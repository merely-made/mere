// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use sceno::{
    Backdrop, InstanceId, ProjectedItem, Representation, RoutedRelation, Size2, SourceRef,
    Transform2,
};

fn scene_with(points: &[(f32, f32)]) -> Scene {
    let mut scene = Scene::new();
    for (i, (x, y)) in points.iter().enumerate() {
        let source = scene.intern_source(SourceRef::new("test", i.to_string()));
        scene.items.push(ProjectedItem {
            source,
            space: Scene::WORLD,
            transform: Transform2::translation(*x, *y),
            footprint: Footprint::Rect {
                size: Size2::new(20.0, 20.0),
            },
            representation: Representation::Glyph,
            layer: 0,
            visible: true,
            hit: None,
            channels: Vec::new(),
        });
    }
    scene
}

fn separation(scene: &Scene, a: usize, b: usize) -> f32 {
    let p = scene.items[a].transform.translate;
    let q = scene.items[b].transform.translate;
    ((p.x - q.x).powi(2) + (p.y - q.y).powi(2)).sqrt()
}

#[test]
fn zero_steps_leaves_the_scene_untouched() {
    let mut scene = scene_with(&[(0.0, 0.0), (1.0, 0.0)]);
    let before = scene.clone();
    relax(
        &mut scene,
        &Relaxation {
            steps: 0,
            ..Default::default()
        },
    );
    assert_eq!(scene, before);
}

#[test]
fn plain_relax_no_longer_drags_a_recorded_pin() {
    // The footgun this closes: a caller who reaches for `relax` rather than
    // `relax_holding` used to move a pinned placement in silence, and
    // nothing in the types objected.
    let mut scene = scene_with(&[(0.0, 0.0), (1.0, 0.0), (-1.0, 0.5)]);
    scene.honored_holds.push(sceno::HonoredHold {
        instance: InstanceId(0),
        placement: sceno::HeldPlacement::pinned(SourceRef::new("test", "0"), Vec2::new(0.0, 0.0)),
    });
    let pinned_at = scene.items[0].transform.translate;

    relax(&mut scene, &Relaxation::default());

    assert_eq!(
        scene.items[0].transform.translate, pinned_at,
        "a recorded pin moved under plain relax"
    );
    assert_ne!(
        scene.items[1].transform.translate,
        Vec2::new(1.0, 0.0),
        "and its neighbours still relaxed around it"
    );
}

#[test]
fn seeded_items_relax_and_anchored_items_end_home() {
    // A seeded item is not in honored_holds, so it keeps moving; a scene
    // where seeding silently pinned would be the silent-soft failure
    // wearing the other face. An anchored item is drawn back while the
    // scene loosens and ends exactly home (F45).
    let mut seeded = scene_with(&[(0.0, 0.0), (1.0, 0.0)]);
    let before = seeded.items[0].transform.translate;
    relax(&mut seeded, &Relaxation::default());
    assert_ne!(seeded.items[0].transform.translate, before);

    let mut anchored = scene_with(&[(0.0, 0.0), (1.0, 0.0)]);
    relax_roles(
        &mut anchored,
        &Relaxation::default(),
        &[(InstanceId(0), Hold::Anchored)],
    );
    assert_eq!(anchored.items[0].transform.translate, before, "home");
    assert_ne!(
        anchored.items[1].transform.translate,
        Vec2::new(1.0, 0.0),
        "its seeded neighbour relaxed"
    );
}

/// F22 at swatch scale: an item's role overrides the recipe's. The pinned
/// item holds exactly and the anchored one ends home (F45), while the
/// seeded one beside it stays where it relaxed.
#[test]
fn per_item_roles_override_the_recipe_role() {
    let points = [(0.0, 0.0), (10.0, 0.0), (20.0, 0.0), (30.0, 0.0)];
    let mut scene = scene_with(&points);
    let settings = Relaxation {
        anchor_pull: 4.0,
        ..Relaxation::default().anchored()
    };
    relax_roles(
        &mut scene,
        &settings,
        &[(InstanceId(0), Hold::Pinned), (InstanceId(3), Hold::Seeded)],
    );
    let drift = |i: usize| {
        let at = scene.items[i].transform.translate;
        (at.x - points[i].0).hypot(at.y - points[i].1)
    };
    assert_eq!(drift(0), 0.0, "pinned holds");
    assert_eq!(drift(2), 0.0, "anchored ends home");
    assert!(drift(3) > 0.0, "seeded stays where it relaxed");
}

#[test]
fn a_held_item_does_not_move_however_crowded() {
    // Three items on top of each other: maximum pressure to displace.
    let mut scene = scene_with(&[(0.0, 0.0), (1.0, 0.0), (-1.0, 0.5)]);
    let anchored = scene.items[0].transform.translate;
    relax_holding(&mut scene, &Relaxation::default(), &[InstanceId(0)]);
    assert_eq!(
        scene.items[0].transform.translate, anchored,
        "a hold was relaxed away, which is the silent-soft failure"
    );
}

#[test]
fn a_held_item_still_pushes_its_neighbours() {
    // The asymmetry that makes a hold useful: the scene accommodates the
    // pin rather than the pin being averaged into the crowd.
    let mut scene = scene_with(&[(0.0, 0.0), (1.0, 0.0)]);
    let before = separation(&scene, 0, 1);
    relax_holding(&mut scene, &Relaxation::default(), &[InstanceId(0)]);
    assert!(
        separation(&scene, 0, 1) > before,
        "the free neighbour should have been pushed clear"
    );
}

#[test]
fn relax_is_relax_holding_with_nothing_held() {
    let mut plain = scene_with(&[(0.0, 0.0), (1.0, 0.0), (-1.0, 0.5)]);
    let mut empty_holds = plain.clone();
    relax(&mut plain, &Relaxation::default());
    relax_holding(&mut empty_holds, &Relaxation::default(), &[]);
    assert_eq!(plain, empty_holds);
}

#[test]
fn crowded_items_push_apart() {
    let mut scene = scene_with(&[(0.0, 0.0), (2.0, 0.0), (-2.0, 1.0)]);
    let before = separation(&scene, 0, 1);
    relax(&mut scene, &Relaxation::default().untethered());
    assert!(
        separation(&scene, 0, 1) > before,
        "a crowded pair should loosen, got {} from {before}",
        separation(&scene, 0, 1)
    );
}

#[test]
fn a_collidable_backdrop_excludes_item_placement_even_when_invisible() {
    let mut scene = scene_with(&[(0.0, 0.0)]);
    let source = scene.intern_source(SourceRef::new("test", "wall"));
    scene.backdrops.push(Backdrop {
        source,
        space: Scene::WORLD,
        transform: Transform2::IDENTITY,
        footprint: Footprint::Rect {
            size: Size2::new(80.0, 80.0),
        },
        kind: "test:wall".into(),
        visible: false,
        collidable: true,
    });

    relax(&mut scene, &Relaxation::default().untethered());

    let item = placed_bounds(&scene.items[0].footprint, scene.items[0].transform).unwrap();
    let obstacle =
        placed_bounds(&scene.backdrops[0].footprint, scene.backdrops[0].transform).unwrap();
    assert_eq!(separating_shift(item, obstacle), None);
}

#[test]
fn a_noncollidable_backdrop_does_not_move_an_item() {
    let mut scene = scene_with(&[(0.0, 0.0)]);
    let source = scene.intern_source(SourceRef::new("test", "floor"));
    scene.backdrops.push(Backdrop {
        source,
        space: Scene::WORLD,
        transform: Transform2::IDENTITY,
        footprint: Footprint::Rect {
            size: Size2::new(80.0, 80.0),
        },
        kind: "test:floor".into(),
        visible: true,
        collidable: false,
    });
    let before = scene.items[0].transform;
    relax(&mut scene, &Relaxation::default().untethered());
    assert_eq!(scene.items[0].transform, before);
}

#[test]
fn coincident_items_separate_reproducibly() {
    let mut a = scene_with(&[(5.0, 5.0), (5.0, 5.0)]);
    let mut b = a.clone();
    let settings = Relaxation::default().untethered();
    relax(&mut a, &settings);
    relax(&mut b, &settings);
    assert!(separation(&a, 0, 1) > 1.0, "coincident items must separate");
    assert_eq!(a, b, "relaxation is deterministic");
}

#[test]
fn an_anchored_arrangement_holds_its_shape() {
    let points = [(0.0, 0.0), (10.0, 0.0), (20.0, 0.0)];
    let mut tethered = scene_with(&points);
    let mut free = scene_with(&points);
    relax(
        &mut tethered,
        &Relaxation {
            anchor_pull: 4.0,
            ..Relaxation::default().anchored()
        },
    );
    relax(&mut free, &Relaxation::default().untethered());

    let drift = |scene: &Scene| -> f32 {
        scene
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let (ax, ay) = points[i];
                ((item.transform.translate.x - ax).powi(2)
                    + (item.transform.translate.y - ay).powi(2))
                .sqrt()
            })
            .sum()
    };
    assert_eq!(drift(&tethered), 0.0, "an anchored arrangement ends home");
    assert!(
        drift(&free) > 0.0,
        "a seeded one does not: {}",
        drift(&free)
    );
}

#[test]
fn related_items_are_drawn_toward_the_rest_length() {
    let mut scene = scene_with(&[(0.0, 0.0), (400.0, 0.0)]);
    scene.relations.push(RoutedRelation {
        from: InstanceId(0),
        to: InstanceId(1),
        space: Scene::WORLD,
        points: Vec::new(),
        kind: None,
        weight: Some(1.0),
        link_key: None,
        marks: sceno::LinkMarks::default(),
        strength: None,
    });
    let before = separation(&scene, 0, 1);
    relax(&mut scene, &Relaxation::default().untethered());
    assert!(
        separation(&scene, 0, 1) < before,
        "a stretched relation should pull in"
    );
}
