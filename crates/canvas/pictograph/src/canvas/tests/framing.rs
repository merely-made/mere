// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Framing must change the camera alone, including scoped and folded views.
use super::*;

fn placed(points: &[(f32, f32)]) -> (Canvas, Vec<NodeKey>) {
    let mut graph = Graph::new();
    let keys = points
        .iter()
        .enumerate()
        .map(|(i, _)| graph.add_node(format!("https://framing.test/{i}"), PortablePoint::zero()))
        .collect::<Vec<_>>();
    let mut canvas = Canvas::with_graph(graph);
    canvas.resize(800, 600);
    for (&key, &(x, y)) in keys.iter().zip(points) {
        canvas.view.set_position(key, Point2D::new(x, y));
    }
    (canvas, keys)
}

fn inside(canvas: &Canvas, key: NodeKey) {
    let (x, y) = canvas.screen_position_of(key).unwrap();
    assert!(
        (0.0..=800.0).contains(&x) && (0.0..=600.0).contains(&y),
        "{x}, {y}"
    );
}

#[test]
fn visible_fit_ignores_scoped_outlier_and_preserves_positions_and_focus() {
    let (mut canvas, keys) = placed(&[(-250.0, -70.0), (250.0, 70.0), (9000.0, 9000.0)]);
    let ids = keys[..2]
        .iter()
        .map(|&k| canvas.graph.get_node(k).unwrap().id)
        .collect::<Vec<_>>();
    canvas.scope_to_members(ids);
    canvas.select_only(keys[0]);
    canvas.camera.offset = (-9000.0, 2300.0);
    canvas.pan_velocity = (500.0, -600.0);
    canvas.set_view_follow(true);
    let before = keys
        .iter()
        .map(|&k| canvas.world_position_of(k))
        .collect::<Vec<_>>();
    assert!(canvas.fit_visible());
    inside(&canvas, keys[0]);
    inside(&canvas, keys[1]);
    assert!(
        canvas.camera.zoom > 0.8,
        "outlier must not shrink the scoped view"
    );
    assert_eq!(canvas.focused_key(), Some(keys[0]));
    assert_eq!(
        before,
        keys.iter()
            .map(|&k| canvas.world_position_of(k))
            .collect::<Vec<_>>()
    );
    assert!(!canvas.view_follows());
    assert_eq!(canvas.pan_velocity, (0.0, 0.0));
    assert!(
        !canvas.follow_step(1.0),
        "fit must stay put while physics runs"
    );
}

#[test]
fn selection_fit_frames_all_visible_selected_nodes_without_changing_scope() {
    let (mut canvas, keys) = placed(&[(-400.0, -100.0), (400.0, 100.0), (7000.0, 7000.0)]);
    canvas.selected.extend(keys[..2].iter().copied());
    assert!(canvas.can_fit_selection());
    assert!(canvas.fit_selection());
    inside(&canvas, keys[0]);
    inside(&canvas, keys[1]);
    assert!(!canvas.is_scoped());
    assert_eq!(canvas.selected.len(), 2);
    assert!(canvas.camera.zoom > 0.7);
}

#[test]
fn visible_fit_uses_rotated_and_foreshortened_positions() {
    let (mut canvas, keys) = placed(&[(0.0, -1200.0), (0.0, 1200.0)]);
    canvas.set_yaw(std::f32::consts::FRAC_PI_2);
    canvas.set_tilt(0.55);
    assert!(canvas.fit_visible());
    for key in keys {
        inside(&canvas, key);
    }
    assert_eq!(canvas.yaw(), std::f32::consts::FRAC_PI_2);
    assert_eq!(canvas.tilt(), 0.55);
}

#[test]
fn visible_fit_includes_fold_summary_instead_of_hidden_member_extents() {
    let (mut canvas, keys) = placed(&[(-5000.0, 0.0), (5000.0, 0.0), (100.0, 0.0)]);
    canvas.selected.extend(keys[..2].iter().copied());
    canvas.fold_selected("framing-test").unwrap();
    assert!(canvas.fit_visible());
    assert_eq!(
        canvas.camera.zoom, 1.0,
        "hidden extents do not shrink the summary"
    );
    let summary = canvas.screen_point_of((0.0, 0.0));
    assert!((0.0..800.0).contains(&summary.0));
    inside(&canvas, keys[2]);
    assert_eq!(canvas.graph.node_count(), 3, "framing is view-only");
}

#[test]
fn empty_nonfinite_and_hidden_selection_leave_viewport_unchanged() {
    let (mut canvas, keys) = placed(&[(f32::NAN, 0.0), (7000.0, 0.0)]);
    let id = canvas.graph.get_node(keys[0]).unwrap().id;
    canvas.scope_to_members([id]);
    canvas.selected.extend(keys.iter().copied());
    canvas.pan_velocity = (11.0, 13.0);
    canvas.set_view_follow(true);
    let before = canvas.viewport();
    assert!(!canvas.can_fit_selection());
    assert!(!canvas.fit_selection());
    assert!(!canvas.fit_visible());
    assert_eq!(canvas.viewport(), before);
    let mut empty = Canvas::new();
    let before = empty.viewport();
    assert!(!empty.fit_visible());
    assert_eq!(empty.viewport(), before);
}

#[test]
fn single_selection_centers_at_natural_size_and_respects_narrow_viewport() {
    let (mut canvas, keys) = placed(&[(8000.0, -6000.0)]);
    canvas.select_only(keys[0]);
    canvas.resize(220, 170);
    assert!(canvas.fit_selection());
    let (x, y) = canvas.screen_position_of(keys[0]).unwrap();
    assert!((x - 110.0).abs() < 0.01 && (y - 85.0).abs() < 0.01);
    assert!(canvas.camera.zoom > 0.1 && canvas.camera.zoom < 1.0);
}
