// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The face sits on its body at every zoom: the body read back from Livery
//! by hit test, the face from the face layer's own rect. Without the gnode's
//! top-left `transform-origin` the body drifts `(size/2)(1 - zoom)` off its
//! anchor (physics catalog plan, 2026-10-04).

use super::*;

const ZOOMS: [f32; 6] = [1.0, 0.75, 0.5, 0.25, 0.1, 2.0];

fn one_node() -> (Canvas, NodeKey) {
    let mut graph = Graph::new();
    let key = graph.add_node(
        "https://face.example/".to_string(),
        PortablePoint::new(0.0, 0.0),
    );
    let mut canvas = Canvas::with_graph(graph);
    canvas.set_physics_paused(true);
    (canvas, key)
}

fn at_zoom(canvas: &mut Canvas, zoom: f32) {
    canvas.set_camera(CameraView {
        offset: (400.0, 300.0),
        zoom,
    });
    let _ = canvas.frame(800, 600);
}

fn centre((x0, y0, x1, y1): (f32, f32, f32, f32)) -> (f32, f32) {
    (0.5 * (x0 + x1), 0.5 * (y0 + y1))
}

#[test]
fn the_body_sits_on_its_anchor_and_carries_the_face_at_every_zoom() {
    let (mut canvas, key) = one_node();
    let mut errors = Vec::new();
    for zoom in ZOOMS {
        at_zoom(&mut canvas, zoom);
        let (ax, ay) = canvas.screen_position_of(key).unwrap();
        let body = canvas.node_body_rect(key).expect("the body is drawn");
        let face = canvas.node_face_rect(key).expect("a derived face paints");
        let (bx, by) = centre(body);
        let (fx, fy) = centre(face);
        let ratio = (face.2 - face.0) / (body.2 - body.0);
        println!(
            "zoom {zoom:.2}: body centre - anchor ({:.3}, {:.3}), face centre - anchor ({:.3}, {:.3}), \
             body {:.3} x {:.3}, face/body {ratio:.4}",
            bx - ax,
            by - ay,
            fx - ax,
            fy - ay,
            body.2 - body.0,
            body.3 - body.1,
        );
        if (bx - ax).hypot(by - ay) > 0.01 {
            errors.push(format!(
                "zoom {zoom}: body {:.3} px off its anchor",
                (bx - ax).hypot(by - ay)
            ));
        }
        if (fx - bx).hypot(fy - by) > 0.01 {
            errors.push(format!(
                "zoom {zoom}: face {:.3} px off the body",
                (fx - bx).hypot(fy - by)
            ));
        }
        if (ratio - FACE_INSET).abs() > 0.001 {
            errors.push(format!(
                "zoom {zoom}: face/body {ratio:.4}, not {FACE_INSET}"
            ));
        }
    }
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn a_click_on_the_drawn_body_picks_its_node_below_zoom_1() {
    let (mut canvas, key) = one_node();
    for zoom in [0.75, 0.5, 0.25, 0.1] {
        at_zoom(&mut canvas, zoom);
        let body = canvas.node_body_rect(key).expect("the body is drawn");
        let (cx, cy) = centre(body);
        let half = 0.5 * (body.2 - body.0);
        // The centre and points across the body inside its inscribed circle
        // (the pick is the node's circle, so the square's corners lie outside).
        for (u, v) in [
            (0.0, 0.0),
            (0.9, 0.0),
            (-0.9, 0.0),
            (0.0, 0.9),
            (0.0, -0.9),
            (0.6, 0.6),
            (-0.6, -0.6),
        ] {
            let (x, y) = (cx + u * half, cy + v * half);
            // A bare click far from the node clears the selection first.
            canvas.pointer_down(PointerButton::Left, 5.0, 5.0);
            canvas.pointer_up(PointerButton::Left, 5.0, 5.0);
            assert!(
                canvas.selected.is_empty(),
                "zoom {zoom}: the clearing click left a selection"
            );
            canvas.pointer_down(PointerButton::Left, x, y);
            canvas.pointer_up(PointerButton::Left, x, y);
            assert!(
                canvas.selected.contains(&key),
                "zoom {zoom}: a click at ({x:.2}, {y:.2}) on the drawn body ({u}, {v} of its half-side) missed its node",
            );
        }
    }
}
