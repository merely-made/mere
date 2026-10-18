// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::canvas::{NodeShape, Viewport};
use std::collections::HashMap;

fn park_sample_nodes(canvas: &mut Canvas, target: NodeKey, at: Point2D<f32>) {
    for (index, (key, _)) in canvas.graph.nodes().enumerate() {
        let position = if key == target {
            at
        } else {
            Point2D::new(10_000.0 + index as f32 * 1_000.0, 10_000.0)
        };
        canvas.view.set_position(key, position);
    }
}

#[test]
fn top_down_node_pick_uses_visible_square_and_rounded_billboards() {
    let mut canvas = Canvas::with_sample_graph();
    let keys: Vec<_> = canvas.graph.nodes().map(|(key, _)| key).collect();
    let target = keys[0];
    let id = canvas.graph.get_node(target).unwrap().id;
    park_sample_nodes(&mut canvas, target, Point2D::new(200.0, 200.0));

    canvas.set_node_shapes(HashMap::from([(id, NodeShape::Square)]));
    let square = canvas.node_billboard(target).unwrap();
    assert_eq!(
        canvas.pick_at((square.center.x + 17.0, square.center.y + 17.0)),
        Some(target),
        "a visible square corner is selectable even outside the physics view's circle pick"
    );

    canvas.set_node_shapes(HashMap::from([(id, NodeShape::Rounded)]));
    let rounded = canvas.node_billboard(target).unwrap();
    assert_eq!(
        canvas.pick_at((rounded.center.x + 17.0, rounded.center.y + 7.0)),
        Some(target),
        "the rounded-square edge follows its painted silhouette"
    );
    assert_eq!(
        canvas.pick_at((rounded.center.x + 17.0, rounded.center.y + 17.0)),
        None,
        "a transparent rounded corner is not selectable"
    );

    canvas.set_node_shapes(HashMap::from([(id, NodeShape::Square)]));
    canvas
        .set_node_rule_faces(
            canvas.graph_view_epoch(),
            HashMap::from([(id, sceno::Representation::Glyph)]),
        )
        .unwrap();
    let glyph = canvas.node_billboard(target).unwrap();
    assert_eq!(
        canvas.pick_at((glyph.center.x + 17.0, glyph.center.y)),
        Some(target)
    );
    assert_eq!(
        canvas.pick_at((glyph.center.x + 17.0, glyph.center.y + 17.0)),
        None,
        "Glyph's visible circle overrides the square physics body for picking"
    );
}

#[test]
fn yawed_marquee_uses_screen_rectangle_and_visual_billboard_centers() {
    let mut canvas = Canvas::with_sample_graph();
    let keys: Vec<_> = canvas.graph.nodes().map(|(key, _)| key).collect();
    let first = keys[0];
    let second = keys[1];
    park_sample_nodes(&mut canvas, first, Point2D::new(100.0, 100.0));
    canvas.view.set_position(second, Point2D::new(100.0, 110.0));
    canvas.set_viewport(Viewport {
        offset: (300.0, 200.0),
        zoom: 1.0,
        yaw: std::f32::consts::FRAC_PI_4,
        tilt: 1.0,
        pan_velocity: (0.0, 0.0),
        view: (800, 600),
    });
    let first_body = canvas.node_billboard(first).unwrap();
    let second_body = canvas.node_billboard(second).unwrap();
    assert!((second_body.center.x - first_body.center.x).abs() < 8.0);
    assert!((second_body.center.y - first_body.center.y).abs() < 8.0);

    let origin = (first_body.center.x - 10.0, first_body.center.y - 10.0);
    canvas.cursor = (first_body.center.x + 10.0, first_body.center.y + 10.0);
    canvas.rect_select(origin);
    assert!(canvas.selected.contains(&first));
    assert!(
        canvas.selected.contains(&second),
        "the second projected center is inside the actual marquee rectangle"
    );
}

#[test]
fn lifted_billboard_center_remains_marquee_selectable() {
    let mut canvas = Canvas::with_sample_graph();
    let keys: Vec<_> = canvas.graph.nodes().map(|(key, _)| key).collect();
    let key = keys[0];
    canvas
        .graph
        .declare_predicate(
            "urn:test:marquee-height",
            kernel::graph::GraphStratum::Surface,
        )
        .unwrap();
    canvas
        .graph
        .try_assert_semantic_statement(
            key,
            keys[1],
            kernel::graph::SemanticStatementSpec {
                predicate: "urn:test:marquee-height".into(),
                ..Default::default()
            },
        )
        .unwrap();
    park_sample_nodes(&mut canvas, key, Point2D::new(250.0, 180.0));
    canvas.set_height_by_degree(true);
    canvas.set_viewport(Viewport {
        offset: (30.0, 40.0),
        zoom: 1.5,
        yaw: 0.37,
        tilt: 0.62,
        pan_velocity: (0.0, 0.0),
        view: (800, 600),
    });
    let body = canvas.node_billboard(key).unwrap();
    assert!(body.center.y < body.ground_anchor.y);

    let origin = (body.center.x - 2.0, body.center.y - 2.0);
    canvas.cursor = (body.center.x + 2.0, body.center.y + 2.0);
    canvas.rect_select(origin);
    assert!(canvas.selected.contains(&key));
}
