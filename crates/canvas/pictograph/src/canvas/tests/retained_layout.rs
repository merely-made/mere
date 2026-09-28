// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A stationary canvas must leave retained DOM layout reusable, while actual
//! camera, size and selection changes still update the rendered document.

use super::*;

fn paused_canvas() -> (Canvas, NodeKey) {
    let mut graph = Graph::new();
    let key = graph.add_node(
        "https://retained.example/".to_string(),
        PortablePoint::new(0.0, 0.0),
    );
    let mut canvas = Canvas::with_graph(graph);
    canvas.set_physics_paused(true);
    canvas.set_camera(CameraView {
        offset: (400.0, 300.0),
        zoom: 1.0,
    });
    (canvas, key)
}

fn node_attr(canvas: &Canvas, key: NodeKey, attribute: &str) -> String {
    canvas
        .node_document
        .dom()
        .attribute(
            canvas.gnode_of[&key],
            &Namespace::from(""),
            &LocalName::from(attribute),
        )
        .unwrap()
        .to_owned()
}

#[test]
fn unchanged_paused_frame_reuses_node_layout_and_paint() {
    let (mut canvas, _) = paused_canvas();
    let (_, moving) = canvas.frame(800, 600);
    assert!(!moving);
    let layout = canvas.node_document.layout_generation();
    let paint = canvas.node_document.generation();
    assert!(layout > 0, "the first frame must actually lay out its node");

    for _ in 0..3 {
        let (_, moving) = canvas.frame(800, 600);
        assert!(!moving);
        assert_eq!(
            canvas.node_document.layout_generation(),
            layout,
            "equal canvas attributes must not trigger another geometry pass"
        );
        assert_eq!(
            canvas.node_document.generation(),
            paint,
            "an unchanged node document should reuse its paint list"
        );
    }
}

#[test]
fn real_camera_size_and_class_changes_invalidate_retained_paint() {
    let (mut canvas, key) = paused_canvas();
    let _ = canvas.frame(800, 600);
    let mut style = node_attr(&canvas, key, "style");
    let mut paint = canvas.node_document.generation();

    // Camera changes affect paint even if a future layout backend can retain
    // geometry for them. Do not require a full geometry pass for transforms.
    for camera in [
        CameraView {
            offset: (425.0, 310.0),
            zoom: 1.0,
        },
        CameraView {
            offset: (425.0, 310.0),
            zoom: 1.5,
        },
    ] {
        canvas.set_camera(camera);
        let _ = canvas.frame(800, 600);
        let next_style = node_attr(&canvas, key, "style");
        assert_ne!(
            next_style, style,
            "a real camera change reaches the node DOM"
        );
        assert!(canvas.node_document.generation() > paint);
        style = next_style;
        paint = canvas.node_document.generation();
    }

    let layout = canvas.node_document.layout_generation();
    let id = canvas.graph().get_node(key).unwrap().id;
    canvas.set_node_size(id, 72.0);
    let _ = canvas.frame(800, 600);
    assert!(node_attr(&canvas, key, "style").contains("width: 72px; height: 72px;"));
    assert!(
        canvas.node_document.layout_generation() > layout,
        "changing the node footprint must update geometry"
    );
    assert!(canvas.node_document.generation() > paint);

    paint = canvas.node_document.generation();
    canvas.select_only(key);
    let _ = canvas.frame(800, 600);
    assert!(node_attr(&canvas, key, "class").contains("gnode-selected"));
    assert!(
        canvas.node_document.generation() > paint,
        "selection must repaint even when node geometry is unchanged"
    );
}
