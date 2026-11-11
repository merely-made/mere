// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::canvas::LinkAppearance;
use crate::canvas::node_billboard::NodeBillboard;
use scenograph::link::{LinkKey, LinkMarks, LinkStratum};

fn billboard(
    ground_anchor: Point2D<f32>,
    center: Point2D<f32>,
    local_side: f32,
    zoom: f32,
    corner_radius: f32,
) -> NodeBillboard {
    NodeBillboard {
        ground_anchor,
        center,
        local_side,
        zoom,
        depth: 0.0,
        corner_radius,
    }
}

fn segment(start: (f32, f32), end: (f32, f32), offset: (f32, f32)) -> LinkAppearanceSegment {
    LinkAppearanceSegment {
        appearance: LinkAppearance {
            key: LinkKey::new(LinkStratum::Resource, "source:a", "source:b"),
            display_from: kernel::graph::NodeKey::new(1),
            display_to: kernel::graph::NodeKey::new(2),
            marks: LinkMarks::default(),
            strength: 1.0,
        },
        from: Point2D::new(start.0 + offset.0, start.1 + offset.1),
        to: Point2D::new(end.0 + offset.0, end.1 + offset.1),
        points: vec![Point2D::new(start.0, start.1), Point2D::new(end.0, end.1)],
        responsive: false,
    }
}

fn screen_points(camera: Camera, points: &[Point2D<f32>]) -> Vec<Point2D<f32>> {
    points
        .iter()
        .map(|point| {
            let (x, y) = camera.to_screen(PortablePoint::new(point.x, point.y));
            Point2D::new(x, y)
        })
        .collect()
}

#[test]
fn screen_space_smoothing_stays_bounded_and_clips_square_bodies() {
    let camera = Camera::default();
    let samples = [
        (0.0, 0.0),
        (10.0, 45.0),
        (28.0, -22.0),
        (64.0, 88.0),
        (100.0, 60.0),
    ];
    let resting = segment(samples[0], samples[samples.len() - 1], (0.0, 0.0));
    let from = billboard(
        Point2D::new(0.0, 0.0),
        Point2D::new(0.0, 0.0),
        8.0,
        1.0,
        0.0,
    );
    let to = billboard(
        Point2D::new(100.0, 60.0),
        Point2D::new(100.0, 60.0),
        8.0,
        1.0,
        0.0,
    );
    let path = responsive_path_geometry(&samples, &resting, camera, from, to).unwrap();
    let screen = screen_points(camera, &path);
    assert!(path.len() <= MAX_PRESENTATION_POINTS);
    assert!(path.len() > samples.len());
    assert!(from.signed_distance_screen(screen[0]).unwrap().abs() < 1.0e-3);
    assert!(
        to.signed_distance_screen(*screen.last().unwrap())
            .unwrap()
            .abs()
            < 1.0e-3
    );
    assert!(screen.iter().all(|point| finite_point(*point)));
}

#[test]
fn yawed_tilted_camera_clips_at_the_rounded_billboard_boundary() {
    let camera = Camera {
        offset: (31.0, -17.0),
        zoom: 1.7,
        yaw: 0.63,
        tilt: 0.55,
    };
    let start = PortablePoint::new(-80.0, 20.0);
    let end = PortablePoint::new(260.0, 220.0);
    let (sx, sy) = camera.to_screen(start);
    let (ex, ey) = camera.to_screen(end);
    let from = billboard(
        Point2D::new(sx, sy),
        Point2D::new(sx, sy),
        40.0,
        camera.zoom,
        9.0,
    );
    let to = billboard(
        Point2D::new(ex, ey),
        Point2D::new(ex, ey),
        40.0,
        camera.zoom,
        9.0,
    );
    let samples = [(start.x, start.y), (0.0, 70.0), (end.x, end.y)];
    let resting = segment((start.x, start.y), (end.x, end.y), (0.0, 0.0));
    let original_screen = screen_point(camera, (samples[0].0, samples[0].1)).unwrap();
    let next_screen = screen_point(camera, (samples[1].0, samples[1].1)).unwrap();
    let (_, expected_exit) = from
        .segment_interior_interval(original_screen, next_screen)
        .expect("first route segment leaves the source body");

    let path = responsive_path_geometry(&samples, &resting, camera, from, to).unwrap();
    let painted = screen_points(camera, &path);
    let expected = interpolate(original_screen, next_screen, expected_exit);
    assert!(point_distance(painted[0], expected) < 1.0e-3);
    assert!(
        from.signed_distance_screen(painted[0]).unwrap().abs() < 1.0e-3,
        "the stroke starts at the face boundary after the camera round trip"
    );
    assert!(
        !from.contains_screen(painted[1]),
        "the stroke leaves the source body's rounded corner silhouette"
    );
}

#[test]
fn circular_body_clips_diagonal_path_to_circle_not_bounding_square() {
    let camera = Camera::default();
    let from = billboard(
        Point2D::new(0.0, 0.0),
        Point2D::new(0.0, 0.0),
        40.0,
        1.0,
        20.0,
    );
    let to = billboard(
        Point2D::new(100.0, 100.0),
        Point2D::new(100.0, 100.0),
        20.0,
        1.0,
        0.0,
    );
    let samples = [(0.0, 0.0), (50.0, 50.0), (100.0, 100.0)];
    let resting = segment(samples[0], samples[2], (0.0, 0.0));
    let path = responsive_path_geometry(&samples, &resting, camera, from, to).unwrap();
    let painted = screen_points(camera, &path);
    let expected = 20.0 / 2.0_f32.sqrt();
    assert!((painted[0].x - expected).abs() < 1.0e-3);
    assert!((painted[0].y - expected).abs() < 1.0e-3);
}

#[test]
fn effective_suggestion_side_controls_the_attachment_boundary() {
    let camera = Camera::default();
    let from = billboard(
        Point2D::new(0.0, 0.0),
        Point2D::new(0.0, 0.0),
        16.2,
        1.0,
        8.1,
    );
    let to = billboard(
        Point2D::new(100.0, 0.0),
        Point2D::new(100.0, 0.0),
        36.0,
        1.0,
        0.0,
    );
    let samples = [(0.0, 0.0), (50.0, 0.0), (100.0, 0.0)];
    let resting = segment(samples[0], samples[2], (0.0, 0.0));
    let path = responsive_path_geometry(&samples, &resting, camera, from, to).unwrap();
    assert!((path[0].x - 8.1).abs() < 1.0e-3);
    assert!(path[0].y.abs() < 1.0e-3);
}

#[test]
fn stratum_offset_tapers_into_flat_silhouette_without_an_endpoint_gap() {
    let camera = Camera::default();
    let from = billboard(
        Point2D::new(0.0, 0.0),
        Point2D::new(0.0, 0.0),
        20.0,
        1.0,
        0.0,
    );
    let to = billboard(
        Point2D::new(100.0, 0.0),
        Point2D::new(100.0, 0.0),
        20.0,
        1.0,
        0.0,
    );
    let samples = [(0.0, 0.0), (50.0, 0.0), (100.0, 0.0)];
    let resting = segment(samples[0], samples[2], (0.0, 5.0));
    let path = responsive_path_geometry(&samples, &resting, camera, from, to).unwrap();
    let painted = screen_points(camera, &path);
    assert!(from.signed_distance_screen(painted[0]).unwrap().abs() < 1.0e-3);
    assert!(
        to.signed_distance_screen(*painted.last().unwrap())
            .unwrap()
            .abs()
            < 1.0e-3
    );
    assert!(
        painted[1].y > painted[0].y,
        "the lane offset grows after leaving the face"
    );
}

#[test]
fn raised_endpoint_stays_at_stem_foot_and_tapers_stratum_offset() {
    let camera = Camera::default();
    let ground = Point2D::new(0.0, 0.0);
    let from = billboard(ground, Point2D::new(0.0, -24.0), 36.0, 1.0, 0.0);
    let to = billboard(
        Point2D::new(240.0, 0.0),
        Point2D::new(240.0, 0.0),
        36.0,
        1.0,
        0.0,
    );
    let samples = [
        (0.0, 0.0),
        (30.0, 0.0),
        (60.0, 0.0),
        (90.0, 0.0),
        (120.0, 0.0),
        (150.0, 0.0),
        (180.0, 0.0),
        (210.0, 0.0),
        (240.0, 0.0),
    ];
    let resting = segment(samples[0], samples[8], (0.0, 3.0));
    let path = responsive_path_geometry(&samples, &resting, camera, from, to).unwrap();
    let painted = screen_points(camera, &path);
    assert_eq!(
        painted[0], ground,
        "a lifted node attaches through its stem foot"
    );
    assert!(painted.iter().skip(1).any(|point| point.y > painted[0].y));
    assert!(
        painted
            .iter()
            .any(|point| { point.x > 80.0 && point.x < 160.0 && (point.y - 3.0).abs() < 1.0e-3 }),
        "offset reaches its full lane after the source taper"
    );
}

#[test]
fn intersecting_bodies_and_strict_path_reentry_fail_closed() {
    let camera = Camera::default();
    let touching_from = billboard(
        Point2D::new(0.0, 0.0),
        Point2D::new(0.0, 0.0),
        20.0,
        1.0,
        0.0,
    );
    let touching_to = billboard(
        Point2D::new(10.0, 0.0),
        Point2D::new(10.0, 0.0),
        20.0,
        1.0,
        0.0,
    );
    let overlapping = segment((0.0, 0.0), (10.0, 0.0), (0.0, 0.0));
    assert!(
        responsive_path_geometry(
            &[(0.0, 0.0), (5.0, 0.0), (10.0, 0.0)],
            &overlapping,
            camera,
            touching_from,
            touching_to,
        )
        .is_none()
    );

    let from = billboard(
        Point2D::new(0.0, 0.0),
        Point2D::new(0.0, 0.0),
        20.0,
        1.0,
        0.0,
    );
    let to = billboard(
        Point2D::new(100.0, 0.0),
        Point2D::new(100.0, 0.0),
        20.0,
        1.0,
        0.0,
    );
    let reentering = [(0.0, 0.0), (30.0, 0.0), (-30.0, 0.0), (100.0, 0.0)];
    let resting = segment(reentering[0], reentering[3], (0.0, 0.0));
    assert!(responsive_path_geometry(&reentering, &resting, camera, from, to).is_none());
}

#[test]
fn boundary_only_contact_is_allowed_but_interior_reentry_is_not() {
    let body = billboard(
        Point2D::new(0.0, 0.0),
        Point2D::new(0.0, 0.0),
        20.0,
        1.0,
        0.0,
    );
    let boundary = [Point2D::new(-5.0, 10.0), Point2D::new(5.0, 10.0)];
    assert!(!has_strict_body_reentry(&boundary, body, 0));
    let interior = [Point2D::new(-5.0, 9.0), Point2D::new(5.0, 9.0)];
    assert!(has_strict_body_reentry(&interior, body, 0));
}

#[test]
fn invalid_or_singular_camera_falls_back_without_nonfinite_path() {
    let invalid_cameras = [
        Camera {
            yaw: f32::NAN,
            ..Camera::default()
        },
        Camera {
            offset: (f32::INFINITY, 0.0),
            ..Camera::default()
        },
        Camera {
            tilt: 0.0,
            ..Camera::default()
        },
        Camera {
            zoom: 0.0,
            ..Camera::default()
        },
    ];
    let samples = [(0.0, 0.0), (50.0, 0.0), (100.0, 0.0)];
    let resting = segment(samples[0], samples[2], (0.0, 0.0));
    let from = billboard(
        Point2D::new(0.0, 0.0),
        Point2D::new(0.0, 0.0),
        20.0,
        1.0,
        0.0,
    );
    let to = billboard(
        Point2D::new(100.0, 0.0),
        Point2D::new(100.0, 0.0),
        20.0,
        1.0,
        0.0,
    );
    for camera in invalid_cameras {
        assert!(responsive_path_geometry(&samples, &resting, camera, from, to).is_none());
    }
}

#[test]
fn inverse_projected_path_remains_pickable_at_its_painted_screen_location() {
    let camera = Camera {
        offset: (400.0, 200.0),
        zoom: 2.0,
        yaw: 0.4,
        tilt: 0.6,
    };
    let start = PortablePoint::new(-80.0, 0.0);
    let end = PortablePoint::new(80.0, 0.0);
    let (sx, sy) = camera.to_screen(start);
    let (ex, ey) = camera.to_screen(end);
    let from = billboard(Point2D::new(sx, sy), Point2D::new(sx, sy), 30.0, 2.0, 0.0);
    let to = billboard(Point2D::new(ex, ey), Point2D::new(ex, ey), 30.0, 2.0, 0.0);
    let samples = [(start.x, start.y), (0.0, 20.0), (end.x, end.y)];
    let resting = segment((start.x, start.y), (end.x, end.y), (0.0, 0.0));
    let world = responsive_path_geometry(&samples, &resting, camera, from, to).unwrap();
    let screen = screen_points(camera, &world);
    let painted_mid = interpolate(screen[screen.len() / 2], screen[screen.len() / 2 + 1], 0.5);
    let picked = camera.to_world((painted_mid.x, painted_mid.y));
    let picked_screen = screen_point(camera, (picked.x, picked.y)).unwrap();
    assert!(point_distance(painted_mid, picked_screen) < 1.0e-3);
    assert!(distance_to_polyline(picked_screen, &screen) < 1.0e-3);
}

#[test]
fn nonfinite_samples_and_presentation_overflow_are_refused() {
    let camera = Camera::default();
    let from = billboard(
        Point2D::new(0.0, 0.0),
        Point2D::new(0.0, 0.0),
        8.0,
        1.0,
        0.0,
    );
    let to = billboard(
        Point2D::new(100.0, 0.0),
        Point2D::new(100.0, 0.0),
        8.0,
        1.0,
        0.0,
    );
    let resting = segment((0.0, 0.0), (100.0, 0.0), (0.0, 0.0));
    let nonfinite = [(0.0, 0.0), (50.0, f32::NAN), (100.0, 0.0)];
    assert!(responsive_path_geometry(&nonfinite, &resting, camera, from, to).is_none());
    let oversized: Vec<_> = (0..=MAX_EVALUATOR_POINTS)
        .map(|index| (index as f32, 0.0))
        .collect();
    assert!(responsive_path_geometry(&oversized, &resting, camera, from, to).is_none());
}

fn distance_to_polyline(point: Point2D<f32>, line: &[Point2D<f32>]) -> f32 {
    line.windows(2)
        .map(|pair| {
            let delta = pair[1] - pair[0];
            let length_sq = delta.dot(delta);
            let amount = if length_sq <= f32::EPSILON {
                0.0
            } else {
                ((point - pair[0]).dot(delta) / length_sq).clamp(0.0, 1.0)
            };
            point_distance(point, pair[0] + delta * amount)
        })
        .fold(f32::INFINITY, f32::min)
}
