// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cartography::projection::{PositionedEdge, PositionedNode};

fn node(key: NodeKey, x: f32, y: f32, radius: f32) -> PositionedNode {
    PositionedNode {
        node: key,
        position: PortablePoint::new(x, y),
        radius,
    }
}

#[test]
fn paints_camera_then_edges_then_nodes() {
    let a = NodeKey::new(0);
    let b = NodeKey::new(1);
    let projection = Projection {
        nodes: vec![node(a, 0.0, 0.0, 10.0), node(b, 100.0, 0.0, 0.0)],
        edges: vec![PositionedEdge {
            edge: None,
            from: a,
            to: b,
            path: Vec::new(),
            weight: 1.0,
        }],
        ..Projection::empty()
    };

    let list = paint_projection(
        &projection,
        DeviceIntSize::new(800, 600),
        Camera::default(),
        &ScenePaintStyle::default(),
        7,
    );
    let cmds = list.commands();

    // PushTransform + 1 edge stroke + 2 node rects + PopTransform.
    assert_eq!(cmds.len(), 5);
    assert!(matches!(cmds.first(), Some(PaintCmd::PushTransform(_))));
    assert!(matches!(cmds.last(), Some(PaintCmd::PopTransform)));
    assert_eq!(
        cmds.iter()
            .filter(|c| matches!(c, PaintCmd::DrawStroke(_)))
            .count(),
        1
    );
    assert_eq!(
        cmds.iter()
            .filter(|c| matches!(c, PaintCmd::DrawRect(_)))
            .count(),
        2
    );
    assert_eq!(list.engine_id(), EngineId::UNASSIGNED);
    assert_eq!(list.viewport(), DeviceIntSize::new(800, 600));
    assert_eq!(list.generation_id(), 7);
}

#[test]
fn edge_weight_scales_the_stroke_width() {
    // a-b at unit weight, c-d at the multigraph multiplicity of 2: the heavier pair paints a
    // proportionally thicker stroke. This is the paint half of the multiplicity encoding; since
    // the canvas's universal underlay path (project_keys -> paint_projection_filtered) feeds
    // weighted edges in EVERY layout mode, edge thickness shows under analytic layouts too, not
    // only force-directed. (Graph signals — edge-weight encoding.)
    let (a, b, c, d) = (
        NodeKey::new(0),
        NodeKey::new(1),
        NodeKey::new(2),
        NodeKey::new(3),
    );
    let projection = Projection {
        nodes: vec![
            node(a, 0.0, 0.0, 0.0),
            node(b, 100.0, 0.0, 0.0),
            node(c, 0.0, 100.0, 0.0),
            node(d, 100.0, 100.0, 0.0),
        ],
        edges: vec![
            PositionedEdge {
                edge: None,
                from: a,
                to: b,
                path: Vec::new(),
                weight: 1.0,
            },
            PositionedEdge {
                edge: None,
                from: c,
                to: d,
                path: Vec::new(),
                weight: 2.0,
            },
        ],
        ..Projection::empty()
    };
    let style = ScenePaintStyle::default();
    let list = paint_projection(
        &projection,
        DeviceIntSize::new(800, 600),
        Camera::default(),
        &style,
        0,
    );
    let mut widths: Vec<f32> = list
        .commands()
        .iter()
        .filter_map(|c| match c {
            PaintCmd::DrawStroke(s) => Some(s.width),
            _ => None,
        })
        .collect();
    widths.sort_by(|x, y| x.partial_cmp(y).unwrap());
    assert_eq!(widths.len(), 2, "one stroke per edge");
    assert!(
        (widths[0] - style.edge_width).abs() < 1e-4,
        "the unit pair paints the default width"
    );
    assert!(
        (widths[1] - style.edge_width * 2.0).abs() < 1e-4,
        "the weight-2 pair paints twice as thick: {}",
        widths[1]
    );
}

#[test]
fn edge_with_missing_endpoint_is_skipped() {
    let a = NodeKey::new(0);
    let ghost = NodeKey::new(99);
    let projection = Projection {
        nodes: vec![node(a, 0.0, 0.0, 10.0)],
        edges: vec![PositionedEdge {
            edge: None,
            from: a,
            to: ghost,
            path: Vec::new(),
            weight: 1.0,
        }],
        ..Projection::empty()
    };
    let list = paint_projection(
        &projection,
        DeviceIntSize::new(10, 10),
        Camera::default(),
        &ScenePaintStyle::default(),
        0,
    );
    // PushTransform + 1 node rect + PopTransform; the dangling edge is skipped.
    assert_eq!(list.commands().len(), 3);
}

#[test]
fn empty_projection_is_just_the_camera_frame() {
    let list = paint_projection(
        &Projection::empty(),
        DeviceIntSize::new(10, 10),
        Camera::default(),
        &ScenePaintStyle::default(),
        0,
    );
    assert_eq!(list.commands().len(), 2);
}

#[test]
fn trim_to_faces_pulls_endpoints_to_the_radii() {
    let a = PortablePoint::new(0.0, 0.0);
    let b = PortablePoint::new(100.0, 0.0);
    // a's face at r=10, b's at r=18 → the stroke runs 10..82 along the x-axis.
    let seg = trim_to_faces(a, b, 10.0, 18.0);
    assert_eq!(seg.len(), 2);
    assert!(
        (seg[0].x - 10.0).abs() < 1e-4 && seg[0].y.abs() < 1e-4,
        "near end at a's face"
    );
    assert!(
        (seg[1].x - 82.0).abs() < 1e-4 && seg[1].y.abs() < 1e-4,
        "far end at b's face"
    );

    // Faces that overlap (radii exceed the gap) fall back to center-to-center so
    // the stroke never reverses.
    let seg = trim_to_faces(a, b, 60.0, 60.0);
    assert!((seg[0].x - 0.0).abs() < 1e-4, "fallback keeps a's center");
    assert!((seg[1].x - 100.0).abs() < 1e-4, "fallback keeps b's center");
}

#[test]
fn camera_top_down_is_plain_scale_translate() {
    let cam = Camera {
        offset: (10.0, 20.0),
        zoom: 2.0,
        yaw: 0.0,
        tilt: 1.0,
    };
    assert_eq!(
        cam.to_screen(PortablePoint::new(5.0, 7.0)),
        (5.0 * 2.0 + 10.0, 7.0 * 2.0 + 20.0)
    );
    let w = cam.to_world((30.0, 40.0));
    assert!((w.x - (30.0 - 10.0) / 2.0).abs() < 1e-4);
    assert!((w.y - (40.0 - 20.0) / 2.0).abs() < 1e-4);
}

#[test]
fn camera_isometric_foreshortens_y_and_round_trips() {
    let cam = Camera {
        offset: (0.0, 0.0),
        zoom: 1.0,
        yaw: 0.0,
        tilt: 0.5,
    };
    let (sx, sy) = cam.to_screen(PortablePoint::new(10.0, 10.0));
    assert!((sx - 10.0).abs() < 1e-4, "x is unaffected by tilt");
    assert!((sy - 5.0).abs() < 1e-4, "y is foreshortened by tilt");
    let w = cam.to_world((sx, sy));
    assert!(
        (w.x - 10.0).abs() < 1e-4 && (w.y - 10.0).abs() < 1e-4,
        "to_world inverts to_screen under foreshorten"
    );
}

#[test]
fn ground_transform_matches_to_screen_under_yaw() {
    // The ground affine must agree with `to_screen` (which positions the upright
    // billboards) so edges meet the cards under an orbit. Verified at yaw != 0.
    let cam = Camera {
        offset: (3.0, 5.0),
        zoom: 1.5,
        yaw: 0.6,
        tilt: 0.55,
    };
    let t = cam.ground_transform();
    for (x, y) in [(0.0_f32, 0.0_f32), (40.0, -20.0), (-15.0, 33.0)] {
        let (sx, sy) = cam.to_screen(PortablePoint::new(x, y));
        let p = t
            .transform_point2d(LayoutPoint::new(x, y))
            .expect("affine maps");
        assert!(
            (p.x - sx).abs() < 1e-3 && (p.y - sy).abs() < 1e-3,
            "ground transform must match to_screen at ({x},{y}): ({},{}) vs ({sx},{sy})",
            p.x,
            p.y
        );
    }
}
