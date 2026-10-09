// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A composed facet's frozen form names every cell and every heading
//! (Scenograph editor plan, S1): the static realization a reader gets lists
//! the grid rather than flattening it away.

use std::collections::HashMap;

use graphshell_client::frozen::FrozenScene;
use sceno::{
    Footprint, ProjectedItem, Rect, Representation, Scene, Size2, SourceRef, Transform2, Vec2,
};
use scenograph::swatch::{Axis, AxisKind, AxisScale, Facet, FacetCell, Scope, Swatch, SwatchMode};
use scenomise::facet::{FACET_AXIS_ADAPTER, FACET_CELL_ADAPTER, FacetLayout, compose_facet};

fn cell_scene(adapter: &str) -> Scene {
    let mut scene = Scene::new();
    let source = scene.intern_source(SourceRef::new(adapter, "n0"));
    scene.items.push(ProjectedItem {
        source,
        space: Scene::WORLD,
        transform: Transform2::IDENTITY,
        footprint: Footprint::Rect {
            size: Size2::new(10.0, 10.0),
        },
        representation: Representation::Glyph,
        layer: 0,
        visible: true,
        hit: None,
        channels: Vec::new(),
    });
    scene.bounds = Rect::new(Vec2::ZERO, Size2::new(10.0, 10.0));
    scene
}

#[test]
fn freezing_a_facet_lists_every_cell_and_heading() {
    let cell = |column, id: &str| FacetCell {
        column,
        row: 0,
        swatch: Swatch {
            id: id.into(),
            scope: Scope::Mere("m".into()),
            mode: SwatchMode::Projection {
                definition_id: "recipe".into(),
                variant_id: Some(id.into()),
            },
        },
    };
    let facet = Facet {
        id: "compare".into(),
        columns: Axis {
            kind: AxisKind::Arrangement,
            labels: vec!["Grid".into(), "Spiral".into()],
            scale: AxisScale::Shared,
        },
        rows: None,
        cells: vec![cell(0, "grid"), cell(1, "spiral")],
    };
    let scenes = vec![
        ("grid".to_string(), cell_scene("grid")),
        ("spiral".to_string(), cell_scene("spiral")),
    ];
    let composed = compose_facet(&facet, &scenes, &FacetLayout::default()).expect("composes");

    // The host names headings from the facet's labels and frames by cell.
    let mut names = HashMap::new();
    for (index, label) in facet.columns.labels.iter().enumerate() {
        names.insert(
            SourceRef::new(FACET_AXIS_ADAPTER, format!("columns:{index}")),
            label.clone(),
        );
    }
    for (cell, label) in facet.cells.iter().zip(["Grid cell", "Spiral cell"]) {
        names.insert(
            SourceRef::new(FACET_CELL_ADAPTER, &cell.swatch.id),
            label.to_string(),
        );
    }
    let frozen = FrozenScene::freeze(&composed, "Compare arrangements", &names);
    let listed: Vec<&str> = frozen.instances.iter().map(|i| i.name.as_str()).collect();
    for expected in ["Grid", "Spiral", "Grid cell", "Spiral cell"] {
        assert!(
            listed.contains(&expected),
            "{expected} missing from {listed:?}"
        );
    }
    // Each cell's own item is listed too, not dropped.
    assert_eq!(frozen.instances.len(), 6);
}
