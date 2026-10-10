// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use sceno::{Fold, FoldRule, RoutedRelation};
use scenograph::swatch::{Axis, AxisKind, FacetCell, Scope, Swatch, SwatchMode};
use scenotime::{Revision, SceneEpoch, SceneSnapshot};

use super::*;

/// A cell scene of two items `extent` apart, a relation and a fold between them.
fn cell_scene(adapter: &str, extent: f32, generation: u64) -> Scene {
    let mut scene = Scene::new();
    for (index, x) in [0.0, extent].into_iter().enumerate() {
        let source = scene.intern_source(SourceRef::new(adapter, format!("n{index}")));
        scene.items.push(ProjectedItem {
            source,
            space: Scene::WORLD,
            transform: Transform2::translation(x, x),
            footprint: Footprint::Rect {
                size: Size2::new(10.0, 10.0),
            },
            representation: Representation::Glyph,
            layer: 0,
            visible: true,
            hit: None,
            channels: Vec::new(),
        });
    }
    scene.relations.push(RoutedRelation {
        from: InstanceId(0),
        to: InstanceId(1),
        space: Scene::WORLD,
        points: Vec::new(),
        kind: Some("cites".into()),
        weight: None,
    });
    scene.folds.push(Fold {
        members: vec![InstanceId(0), InstanceId(1)],
        stand_in: StandIn::Member(InstanceId(0)),
        rule: Some(FoldRule::Selection),
        boundary: None,
        label: None,
    });
    // Items are centred on their positions, so the bounds start half a
    // footprint before the first.
    scene.bounds = Rect::new(
        Vec2::new(-5.0, -5.0),
        Size2::new(extent + 10.0, extent + 10.0),
    );
    scene.generation = generation;
    scene
}

fn swatch(id: &str) -> Swatch {
    Swatch {
        id: id.into(),
        scope: Scope::Mere("m".into()),
        mode: SwatchMode::Projection {
            definition_id: "recipe".into(),
            variant_id: Some(id.into()),
        },
    }
}

fn facet(columns: AxisScale, rows: Option<AxisScale>) -> Facet {
    let mut cells = vec![
        FacetCell {
            column: 0,
            row: 0,
            swatch: swatch("grid"),
        },
        FacetCell {
            column: 1,
            row: 0,
            swatch: swatch("spiral"),
        },
    ];
    if rows.is_some() {
        cells.push(FacetCell {
            column: 0,
            row: 1,
            swatch: swatch("grid-b"),
        });
        cells.push(FacetCell {
            column: 1,
            row: 1,
            swatch: swatch("spiral-b"),
        });
    }
    Facet {
        id: "compare".into(),
        columns: Axis {
            kind: AxisKind::Arrangement,
            labels: vec!["Grid".into(), "Spiral".into()],
            scale: columns,
        },
        rows: rows.map(|scale| Axis {
            kind: AxisKind::Scope,
            labels: vec!["Mine".into(), "Theirs".into()],
            scale,
        }),
        cells,
    }
}

fn scenes(extents: &[(&str, f32)]) -> Vec<(String, Scene)> {
    extents
        .iter()
        .enumerate()
        .map(|(index, (id, extent))| (id.to_string(), cell_scene(id, *extent, index as u64 + 1)))
        .collect()
}

/// The scale each cell was drawn at, from its frame, by swatch id.
fn scales(scene: &Scene) -> Vec<(String, f32)> {
    scene
        .items
        .iter()
        .filter(|item| {
            item.representation
                == Representation::Open {
                    kind: "facet.cell".into(),
                }
        })
        .map(|item| {
            let id = scene.sources[item.source.0 as usize].id.clone();
            let scale = item
                .channels
                .iter()
                .find(|(k, _)| k == "facet.scale")
                .unwrap()
                .1;
            (id, scale)
        })
        .collect()
}

#[test]
fn a_shared_axis_draws_every_cell_at_one_scale() {
    // 90 and 390 wide in a 320 by 240 box: the larger fits at 240 / 400.
    let composed = compose_facet(
        &facet(AxisScale::Shared, None),
        &scenes(&[("grid", 90.0), ("spiral", 390.0)]),
        &FacetLayout::default(),
    )
    .expect("composes");
    assert_eq!(
        scales(&composed),
        [("grid".into(), 0.6), ("spiral".into(), 0.6)]
    );
}

#[test]
fn an_independent_axis_lets_each_cell_fit_itself() {
    // The control for the test above: the same cells, the scales now differ.
    let composed = compose_facet(
        &facet(AxisScale::Independent, None),
        &scenes(&[("grid", 90.0), ("spiral", 390.0)]),
        &FacetLayout::default(),
    )
    .expect("composes");
    assert_eq!(
        scales(&composed),
        [("grid".into(), 1.0), ("spiral".into(), 0.6)]
    );
}

#[test]
fn shared_columns_and_independent_rows_share_within_a_row() {
    let composed = compose_facet(
        &facet(AxisScale::Shared, Some(AxisScale::Independent)),
        &scenes(&[
            ("grid", 90.0),
            ("spiral", 190.0),
            ("grid-b", 390.0),
            ("spiral-b", 90.0),
        ]),
        &FacetLayout::default(),
    )
    .expect("composes");
    // Row one fits 200 into 240 (capped at 1); row two fits 400 (0.6).
    assert_eq!(
        scales(&composed),
        [
            ("grid".into(), 1.0),
            ("spiral".into(), 1.0),
            ("grid-b".into(), 0.6),
            ("spiral-b".into(), 0.6)
        ]
    );
}

#[test]
fn every_index_is_renumbered_so_each_cell_keeps_its_own_relations_and_folds() {
    let composed = compose_facet(
        &facet(AxisScale::Shared, None),
        &scenes(&[("grid", 90.0), ("spiral", 390.0)]),
        &FacetLayout::default(),
    )
    .expect("composes");
    let adapter = |id: InstanceId| {
        let item = &composed.items[id.0 as usize];
        composed.sources[item.source.0 as usize].adapter.clone()
    };
    for (relation, cell) in composed.relations.iter().zip(["grid", "spiral"]) {
        assert_eq!(adapter(relation.from), cell);
        assert_eq!(adapter(relation.to), cell);
        let space = &composed.spaces[relation.space.0 as usize];
        assert_eq!(
            composed.spaces[space.parent.unwrap().0 as usize]
                .name
                .as_deref(),
            Some(format!("cell: {cell}").as_str())
        );
    }
    for (fold, cell) in composed.folds.iter().zip(["grid", "spiral"]) {
        assert!(fold.members.iter().all(|m| adapter(*m) == cell));
        let StandIn::Member(stand_in) = fold.stand_in else {
            panic!("member stand-in")
        };
        assert_eq!(adapter(stand_in), cell);
    }
    assert_eq!(composed.validate_folds(), Ok(()));
    // Two column headings, two frames, two items per cell.
    assert_eq!(composed.items.len(), 8);
}

#[test]
fn the_composed_scene_round_trips_and_opens_as_a_scenotime_snapshot() {
    let composed = compose_facet(
        &facet(AxisScale::Shared, Some(AxisScale::Shared)),
        &scenes(&[
            ("grid", 90.0),
            ("spiral", 190.0),
            ("grid-b", 390.0),
            ("spiral-b", 90.0),
        ]),
        &FacetLayout::default(),
    )
    .expect("composes");
    let json = serde_json::to_string(&composed).expect("writes");
    let back: Scene = serde_json::from_str(&json).expect("reads");
    assert_eq!(back, composed);
    SceneSnapshot::from_dense(SceneEpoch(1), Revision(0), back).expect("a valid scene");
}

#[test]
fn the_generation_follows_any_cell() {
    let facet = facet(AxisScale::Shared, None);
    let layout = FacetLayout::default();
    let one = compose_facet(
        &facet,
        &scenes(&[("grid", 90.0), ("spiral", 390.0)]),
        &layout,
    )
    .unwrap();
    let mut changed = scenes(&[("grid", 90.0), ("spiral", 390.0)]);
    changed[1].1.generation = 99;
    let two = compose_facet(&facet, &changed, &layout).unwrap();
    assert_ne!(one.generation, two.generation);
}

#[test]
fn a_missing_or_stray_scene_is_refused() {
    let facet = facet(AxisScale::Shared, None);
    let layout = FacetLayout::default();
    assert_eq!(
        compose_facet(&facet, &scenes(&[("grid", 90.0)]), &layout),
        Err(FacetError::MissingScene("spiral".into()))
    );
    assert_eq!(
        compose_facet(
            &facet,
            &scenes(&[("grid", 90.0), ("spiral", 1.0), ("ghost", 1.0)]),
            &layout
        ),
        Err(FacetError::UnknownCell("ghost".into()))
    );
}

#[test]
fn every_item_a_cell_holds_lies_inside_its_frame() {
    let composed = compose_facet(
        &facet(AxisScale::Shared, Some(AxisScale::Independent)),
        &scenes(&[
            ("grid", 90.0),
            ("spiral", 190.0),
            ("grid-b", 390.0),
            ("spiral-b", 90.0),
        ]),
        &FacetLayout::default(),
    )
    .expect("composes");
    let world_rect = |item: &ProjectedItem| {
        let world = composed.to_world(item.space).unwrap().then(&item.transform);
        let local = item.footprint.bounds().unwrap();
        Rect::new(
            Vec2::new(
                world.translate.x + local.origin.x * world.scale,
                world.translate.y + local.origin.y * world.scale,
            ),
            Size2::new(local.size.w * world.scale, local.size.h * world.scale),
        )
    };
    let inside = |inner: Rect, outer: Rect| {
        inner.origin.x >= outer.origin.x - 0.01
            && inner.origin.y >= outer.origin.y - 0.01
            && inner.origin.x + inner.size.w <= outer.origin.x + outer.size.w + 0.01
            && inner.origin.y + inner.size.h <= outer.origin.y + outer.size.h + 0.01
    };
    for frame in composed.items.iter().filter(|item| {
        item.representation
            == Representation::Open {
                kind: "facet.cell".into(),
            }
    }) {
        let cell = &composed.sources[frame.source.0 as usize].id;
        let frame_rect = world_rect(frame);
        let held: Vec<_> = composed
            .items
            .iter()
            .filter(|item| &composed.sources[item.source.0 as usize].adapter == cell)
            .collect();
        assert_eq!(held.len(), 2, "{cell} holds its two items");
        for item in held {
            assert!(
                inside(world_rect(item), frame_rect),
                "{cell}'s item escapes its frame"
            );
        }
    }
    // Headings sit above the cells, inside the scene's bounds.
    for heading in composed
        .items
        .iter()
        .filter(|item| composed.sources[item.source.0 as usize].adapter == FACET_AXIS_ADAPTER)
    {
        assert!(inside(world_rect(heading), composed.bounds));
    }
}
