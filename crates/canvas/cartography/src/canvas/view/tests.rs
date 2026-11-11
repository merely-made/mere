// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use crate::canvas::{Canvas, PointerButton};

/// Swapping between two panes' viewports must move NEITHER camera, at any
/// size. The camera and the size it was framed for install together, so a
/// host never reaches for `resize` (which re-centres) to set the size of a
/// camera it is only borrowing.
///
/// The regression: turnstone's lens installed a camera and then resized to
/// the lens rect, so the re-centring shift landed on the borrowed camera,
/// was read back, and was stored. Both windows' graphs walked off-screen a
/// little per frame.
#[test]
fn swapping_viewports_between_sizes_moves_neither_camera() {
    let mut canvas = Canvas::with_sample_graph();
    canvas.resize(800, 600);
    let primary = canvas.viewport();

    // A second view at a different size, framed its own way.
    canvas.resize(400, 900);
    canvas.wheel(37.0, -11.0);
    let lens = canvas.viewport();
    assert_ne!(primary.view, lens.view, "the two views differ in size");

    // Swap back and forth the way a two-window host does, many times.
    for _ in 0..64 {
        canvas.set_viewport(primary);
        assert_eq!(
            canvas.viewport(),
            primary,
            "the primary is installed exactly"
        );
        canvas.set_viewport(lens);
        assert_eq!(canvas.viewport(), lens, "the lens is installed exactly");
    }

    // Neither drifted, and each still carries its own size.
    canvas.set_viewport(primary);
    assert_eq!(canvas.viewport(), primary);
    assert_eq!(primary.view, (800, 600));
    assert_eq!(lens.view, (400, 900));
}

/// The framing instrument: a fitted graph has every centre on screen, and
/// a camera panned one viewport-width past it (the planted off-screen
/// case) has none; a margin wider than half the view counts every node.
#[test]
fn layout_framing_counts_centres_off_the_viewport() {
    let mut canvas = Canvas::with_sample_graph();
    canvas.resize(800, 600);
    canvas.fit_to_content();
    let fitted = canvas.layout_framing(0.0);
    assert!(fitted.nodes > 0, "the sample graph has nodes");
    assert_eq!(fitted.outside, 0, "fitted: {fitted:?}");
    let [x0, y0, x1, y1] = fitted.extent;
    let [v0, w0, v1, w1] = fitted.view;
    assert!(x0 >= v0 && y0 >= w0 && x1 <= v1 && y1 <= w1, "{fitted:?}");

    let mut camera = canvas.camera();
    camera.offset.0 += 800.0 + (x1 - x0) * camera.zoom;
    canvas.set_camera(camera);
    let planted = canvas.layout_framing(0.0);
    assert_eq!(planted.outside, planted.nodes, "panned away: {planted:?}");
    assert_eq!(planted.extent, fitted.extent, "the layout did not move");

    canvas.fit_to_content();
    assert_eq!(canvas.layout_framing(301.0).outside, fitted.nodes);
}

/// Following the layout ("Follow while playing", 2026-10-03). The control:
/// a camera planted off the graph stays off it while not following. Then
/// following eases it back until every centre is on screen, a wheel pan
/// stops it, a node drag stops it while a click does not, and a paused
/// canvas holds the camera even while following.
#[test]
fn following_eases_the_camera_onto_the_layout_and_a_pan_stops_it() {
    let mut canvas = Canvas::with_sample_graph();
    canvas.resize(800, 600);
    canvas.fit_to_content();
    canvas.set_physics_paused(false);
    let nodes = canvas.layout_framing(0.0).nodes;
    let mut planted = canvas.camera();
    planted.offset.0 += 4_000.0;
    canvas.set_camera(planted);
    assert!(!canvas.view_follows(), "a placement is not followed");
    for _ in 0..60 {
        let _ = canvas.frame(800, 600);
    }
    assert_eq!(canvas.layout_framing(0.0).outside, nodes, "not following");

    canvas.set_view_follow(true);
    for _ in 0..120 {
        let _ = canvas.frame(800, 600);
    }
    let framed = canvas.layout_framing(0.0);
    assert_eq!(framed.outside, 0, "following, two seconds on: {framed:?}");
    assert!(canvas.view_follows());

    canvas.wheel(0.0, 400.0);
    assert!(!canvas.view_follows(), "a pan stops following");
    for _ in 0..120 {
        let _ = canvas.frame(800, 600);
    }
    assert!(
        canvas.layout_framing(0.0).outside > 0,
        "the pan's glide stands; nothing pulls it back"
    );

    // A node drag stops it too; a click on a node does not.
    canvas.fit_to_content();
    canvas.set_view_follow(true);
    let (key, _) = canvas.graph.nodes().next().unwrap();
    let (x, y) = canvas.screen_position_of(key).unwrap();
    canvas.pointer_down(PointerButton::Left, x, y);
    canvas.pointer_up(PointerButton::Left, x, y);
    assert!(canvas.view_follows(), "a click keeps following");
    canvas.pointer_down(PointerButton::Left, x, y);
    canvas.cursor_moved(x + 40.0, y);
    assert!(!canvas.view_follows(), "a node drag stops following");
    canvas.pointer_up(PointerButton::Left, x + 40.0, y);

    canvas.set_physics_paused(true);
    canvas.set_view_follow(true);
    let held = canvas.camera();
    for _ in 0..30 {
        let _ = canvas.frame(800, 600);
    }
    assert_eq!(canvas.camera(), held, "paused, the camera holds");
}

/// World and screen points round-trip through the camera at any zoom, and
/// the focused node's world position maps to its screen position.
#[test]
fn world_and_screen_points_round_trip_through_the_camera() {
    let mut canvas = Canvas::with_sample_graph();
    canvas.resize(800, 600);
    canvas.set_camera(crate::canvas::CameraView {
        offset: (130.0, -40.0),
        zoom: 0.772,
    });
    let world = canvas.world_point_at((410.0, 275.0));
    let back = canvas.screen_point_of(world);
    assert!(
        (back.0 - 410.0).abs() < 1e-3 && (back.1 - 275.0).abs() < 1e-3,
        "{back:?}"
    );
    let (key, _) = canvas.graph.nodes().next().unwrap();
    canvas.selected = [key].into_iter().collect();
    let at = canvas.focused_world_position().unwrap();
    let screen = canvas.focused_screen_position().unwrap();
    let mapped = canvas.screen_point_of(at);
    assert!((mapped.0 - screen.0).abs() < 1e-3 && (mapped.1 - screen.1).abs() < 1e-3);
    // A world step of 220 is 220 x zoom screen px under the top-down camera.
    let step = canvas.screen_point_of((at.0 + 220.0, at.1));
    assert!((step.0 - screen.0 - 220.0 * 0.772).abs() < 1e-2, "{step:?}");
}

/// `resize` still re-centres: a genuine window resize holds whatever sits
/// at the middle. That behaviour is the reason the swap path had to stop
/// borrowing it, so pin it here beside its counterpart.
#[test]
fn resize_still_recentres() {
    let mut canvas = Canvas::with_sample_graph();
    canvas.resize(800, 600);
    let before = canvas.viewport();
    canvas.resize(1000, 600);
    let after = canvas.viewport();
    assert_eq!(
        after.offset.0 - before.offset.0,
        100.0,
        "grew 200px wide, so the centre holds by shifting half that"
    );
    assert_eq!(after.view, (1000, 600));
}
