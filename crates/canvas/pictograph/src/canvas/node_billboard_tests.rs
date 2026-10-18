// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::canvas::scene_paint::Camera;
use crate::canvas::{NodeShape, Viewport};
use std::collections::HashMap;

#[test]
fn shape_contains_and_segment_intervals_share_the_rounded_square() {
    let billboard = NodeBillboard {
        ground_anchor: Point2D::new(0.0, 0.0),
        center: Point2D::new(0.0, 0.0),
        local_side: 20.0,
        zoom: 1.0,
        depth: 0.0,
        corner_radius: 5.0,
    };
    assert!(billboard.contains_screen(Point2D::new(10.0, 0.0)));
    assert!(!billboard.contains_screen(Point2D::new(9.0, 9.0)));
    assert!(billboard.contains_screen(Point2D::new(7.0, 7.0)));

    let horizontal = billboard
        .segment_interior_interval(Point2D::new(-20.0, 0.0), Point2D::new(20.0, 0.0))
        .unwrap();
    assert!((horizontal.0 - 0.25).abs() < 1.0e-5);
    assert!((horizontal.1 - 0.75).abs() < 1.0e-5);
    let mut circle = billboard;
    circle.corner_radius = 10.0;
    let tangent =
        circle.segment_interior_interval(Point2D::new(-20.0, 10.0), Point2D::new(20.0, 10.0));
    assert!(tangent.is_some());
    assert!((tangent.unwrap().1 - tangent.unwrap().0) < 1.0e-5);
}

#[test]
fn square_and_circle_are_the_radius_extremes() {
    let mut billboard = NodeBillboard {
        ground_anchor: Point2D::new(0.0, 0.0),
        center: Point2D::new(0.0, 0.0),
        local_side: 20.0,
        zoom: 1.0,
        depth: 0.0,
        corner_radius: 0.0,
    };
    assert!(billboard.contains_screen(Point2D::new(10.0, 10.0)));
    billboard.corner_radius = 10.0;
    assert!(!billboard.contains_screen(Point2D::new(9.0, 9.0)));
    assert!(billboard.contains_screen(Point2D::new(7.0, 7.0)));
}

#[test]
fn segment_interval_is_reversal_symmetric_and_zoom_aware() {
    let billboard = NodeBillboard {
        ground_anchor: Point2D::new(0.0, 0.0),
        center: Point2D::new(10.0, -4.0),
        local_side: 20.0,
        zoom: 2.0,
        depth: 0.0,
        corner_radius: 5.0,
    };
    let a = Point2D::new(-20.0, -4.0);
    let b = Point2D::new(40.0, -4.0);
    let forward = billboard.segment_interior_interval(a, b).unwrap();
    let reverse = billboard.segment_interior_interval(b, a).unwrap();
    assert!((forward.0 - 1.0 / 6.0).abs() < 1.0e-5);
    assert!((forward.1 - 5.0 / 6.0).abs() < 1.0e-5);
    assert!((forward.0 - (1.0 - reverse.1)).abs() < 1.0e-5);
    assert!((forward.1 - (1.0 - reverse.0)).abs() < 1.0e-5);
}

#[test]
fn boundary_ray_uses_shared_shape_interval_for_circle_and_rounded_body() {
    let mut billboard = NodeBillboard {
        ground_anchor: Point2D::new(10.0, 20.0),
        center: Point2D::new(10.0, 20.0),
        local_side: 24.0,
        zoom: 1.5,
        depth: 0.0,
        corner_radius: 12.0,
    };
    let point = billboard
        .boundary_point_toward(Point2D::new(1.0, 1.0))
        .unwrap();
    assert!(((point.x - 10.0).abs() - (point.y - 20.0).abs()).abs() < 1.0e-4);
    assert!(billboard.signed_distance_screen(point).unwrap().abs() < 1.0e-4);
    assert!(
        billboard
            .boundary_point_toward(Point2D::new(0.0, 0.0))
            .is_none()
    );

    billboard.corner_radius = 4.0;
    let point = billboard
        .boundary_point_toward(Point2D::new(1.0, 1.0))
        .unwrap();
    assert!(billboard.signed_distance_screen(point).unwrap().abs() < 1.0e-4);
}

#[test]
fn canvas_billboard_resolves_grammar_shape_and_projection_once() {
    let mut canvas = Canvas::with_sample_graph();
    let key = canvas.graph.nodes().next().unwrap().0;
    let id = canvas.graph.get_node(key).unwrap().id;
    canvas.view.set_position(key, PortablePoint::new(5.0, 7.0));
    canvas.set_node_shapes(HashMap::from([(id, NodeShape::Rounded)]));
    canvas.set_viewport(Viewport {
        offset: (100.0, 200.0),
        zoom: 2.0,
        yaw: 0.3,
        tilt: 0.6,
        pan_velocity: (0.0, 0.0),
        view: (800, 600),
    });
    let billboard = canvas.node_billboard(key).unwrap();
    let (x, y) = canvas.camera.to_screen(PortablePoint::new(5.0, 7.0));
    assert_eq!(billboard.ground_anchor, Point2D::new(x, y));
    assert_eq!(billboard.center, Point2D::new(x, y));
    assert_eq!(billboard.zoom, 2.0);
    assert_eq!(billboard.corner_radius, 9.0);
    let inset = billboard.inset_screen_rect(0.4);
    assert!((inset.width() - billboard.local_side * billboard.zoom * 0.4).abs() < 1.0e-4);

    canvas.set_height_by_degree(true);
    let lifted = canvas.node_billboard(key).unwrap();
    assert_eq!(
        lifted.center.y,
        lifted.ground_anchor.y - canvas.node_height(key) * lifted.zoom
    );
    canvas.set_height_by_degree(false);

    canvas.set_node_shapes(HashMap::from([(id, NodeShape::Circle)]));
    assert_eq!(canvas.node_billboard(key).unwrap().corner_radius, 18.0);
    canvas.set_node_shapes(HashMap::from([(id, NodeShape::Square)]));
    assert_eq!(canvas.node_billboard(key).unwrap().corner_radius, 0.0);

    canvas.set_node_shapes(HashMap::from([(id, NodeShape::Rounded)]));
    let epoch = canvas.graph_view_epoch();
    canvas
        .set_node_rule_faces(epoch, HashMap::from([(id, sceno::Representation::Glyph)]))
        .unwrap();
    assert_eq!(canvas.node_billboard(key).unwrap().corner_radius, 18.0);

    canvas.set_graph_grammar(true);
    let grammar_billboard = canvas.node_billboard(key).unwrap();
    // An unselected graph node is a site: grammar CSS follows and overrides the
    // content's rounded shape and representation treatment with radius 2px.
    assert_eq!(grammar_billboard.corner_radius, 2.0);
    assert_eq!(grammar_billboard.local_side, canvas.node_size(key));
    canvas.offer_suggestions([(id, "test".to_string())]);
    canvas.set_suggestion_settings(scenograph::suggestion::SuggestionSettings {
        on: true,
        at_once: 1,
        ..Default::default()
    });
    let suggestion = canvas.node_billboard(key).unwrap();
    assert_eq!(suggestion.local_side, canvas.node_size(key) * 0.45);
    assert_eq!(suggestion.corner_radius, suggestion.local_side * 0.5);
    canvas.select_member(id);
    assert_eq!(
        canvas.node_billboard(key).unwrap().corner_radius,
        canvas.node_size(key) * 0.5
    );
}

#[test]
fn invalid_camera_and_nonfinite_segment_inputs_refuse_geometry() {
    let mut billboard = NodeBillboard {
        ground_anchor: Point2D::new(0.0, 0.0),
        center: Point2D::new(0.0, 0.0),
        local_side: 20.0,
        zoom: 1.0,
        depth: 0.0,
        corner_radius: 10.0,
    };
    assert!(
        billboard
            .segment_interior_interval(Point2D::new(f32::NAN, 0.0), Point2D::new(0.0, 0.0))
            .is_none()
    );
    let mut canvas = Canvas::with_sample_graph();
    let key = canvas.graph.nodes().next().unwrap().0;
    billboard.local_side = 0.0;
    assert!(
        billboard
            .segment_interior_interval(Point2D::new(-1.0, 0.0), Point2D::new(1.0, 0.0))
            .is_none()
    );
    canvas.set_viewport(Viewport {
        offset: (0.0, 0.0),
        zoom: 0.0,
        tilt: 1.0,
        yaw: 0.0,
        pan_velocity: (0.0, 0.0),
        view: (800, 600),
    });
    canvas.camera.zoom = 0.0;
    assert!(canvas.node_billboard(key).is_none());
}

#[test]
fn malformed_public_billboards_fail_shared_validity_before_geometry_queries() {
    let valid = NodeBillboard {
        ground_anchor: Point2D::new(0.0, 0.0),
        center: Point2D::new(0.0, 0.0),
        local_side: 20.0,
        zoom: 1.0,
        depth: 0.0,
        corner_radius: 5.0,
    };
    let query_a = Point2D::new(-30.0, 0.0);
    let query_b = Point2D::new(30.0, 0.0);
    let assert_refused = |billboard: NodeBillboard| {
        assert!(!billboard.is_valid());
        assert!(!billboard.contains_screen(Point2D::new(0.0, 0.0)));
        assert!(
            billboard
                .signed_distance_screen(Point2D::new(0.0, 0.0))
                .is_none()
        );
        assert!(
            billboard
                .segment_interior_interval(query_a, query_b)
                .is_none()
        );
    };

    let mut malformed = valid;
    malformed.local_side = f32::NAN;
    assert_refused(malformed);
    let mut malformed = valid;
    malformed.corner_radius = f32::NAN;
    assert_refused(malformed);
    let mut malformed = valid;
    malformed.center.x = f32::NAN;
    assert_refused(malformed);
    let mut malformed = valid;
    malformed.zoom = -1.0;
    assert_refused(malformed);
    let mut malformed = valid;
    malformed.local_side = f32::MAX;
    malformed.zoom = 2.0;
    malformed.corner_radius = f32::MAX * 0.5;
    assert_refused(malformed);
    let mut malformed = valid;
    malformed.local_side = f32::MAX;
    malformed.center.x = f32::MAX;
    assert_refused(malformed);
}

#[test]
fn huge_finite_rounded_billboard_rejects_corner_without_squared_overflow() {
    let billboard = NodeBillboard {
        ground_anchor: Point2D::new(0.0, 0.0),
        center: Point2D::new(0.0, 0.0),
        local_side: f32::MAX,
        zoom: 1.0,
        depth: 0.0,
        corner_radius: f32::MAX * 0.5,
    };
    assert!(billboard.is_valid());
    let half = f32::MAX * 0.5;
    assert!(!billboard.contains_screen(Point2D::new(half, half)));
}

#[test]
fn camera_round_trip_at_circle_boundary_is_not_a_strict_crossing() {
    let billboard = NodeBillboard {
        ground_anchor: Point2D::new(270.0, 279.0),
        center: Point2D::new(270.0, 279.0),
        local_side: 36.0,
        zoom: 1.0,
        depth: 0.0,
        corner_radius: 18.0,
    };
    let camera = Camera {
        offset: (450.0, 279.0),
        zoom: 1.0,
        yaw: 0.37,
        tilt: 0.73,
    };
    let mut observed_inward_rounding = false;
    for index in 0..64 {
        let angle = std::f32::consts::TAU * index as f32 / 64.0;
        let direction = Point2D::new(angle.cos(), angle.sin());
        let boundary = billboard.boundary_point_toward(direction).unwrap();
        let world = camera.to_world((boundary.x, boundary.y));
        let round_trip = camera.to_screen(world);
        let projected = Point2D::new(round_trip.0, round_trip.1);
        let distance = billboard.signed_distance_screen(projected).unwrap();
        observed_inward_rounding |= distance < 0.0;

        let outside = Point2D::new(
            billboard.center.x + direction.x * 64.0,
            billboard.center.y + direction.y * 64.0,
        );
        assert!(
            !billboard.segment_crosses_interior(projected, outside),
            "boundary round-trip at angle {angle} produced signed distance {distance}"
        );
    }
    assert!(
        observed_inward_rounding,
        "fixture should exercise at least one inward f32 round-off"
    );
}

#[test]
fn subpixel_but_real_circle_intrusion_still_counts_as_crossing() {
    let billboard = NodeBillboard {
        ground_anchor: Point2D::new(270.0, 279.0),
        center: Point2D::new(270.0, 279.0),
        local_side: 36.0,
        zoom: 1.0,
        depth: 0.0,
        corner_radius: 18.0,
    };
    let penetration = 0.1;
    let y = billboard.center.y - 18.0 + penetration;
    let from = Point2D::new(220.0, y);
    let to = Point2D::new(320.0, y);
    assert!(billboard.segment_crosses_interior(from, to));
}
