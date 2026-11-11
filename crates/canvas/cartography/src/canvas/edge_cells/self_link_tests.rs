// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::canvas::LinkAppearance;
use scenograph::link::{LinkKey, LinkMarks, LinkStratum};

fn segment(key: NodeKey, stratum: LinkStratum) -> LinkAppearanceSegment {
    LinkAppearanceSegment {
        appearance: LinkAppearance {
            key: LinkKey::new(stratum, "self", "self"),
            display_from: key,
            display_to: key,
            marks: LinkMarks::default(),
            strength: 1.0,
        },
        from: Point2D::new(0.0, 0.0),
        to: Point2D::new(0.0, 0.0),
        points: vec![Point2D::new(0.0, 0.0); 2],
        responsive: true,
    }
}

fn body(center: Point2D<f32>, ground_anchor: Point2D<f32>, radius: f32) -> NodeBillboard {
    NodeBillboard {
        ground_anchor,
        center,
        local_side: 40.0,
        zoom: 1.4,
        depth: 0.0,
        corner_radius: radius.min(20.0),
    }
}

#[test]
fn flat_self_links_attach_to_tilted_circle_silhouette_and_keep_strata() {
    let canvas = crate::canvas::Canvas::with_sample_graph();
    let key = canvas.graph.nodes().next().unwrap().0;
    let camera = Camera {
        offset: (80.0, 40.0),
        zoom: 1.7,
        yaw: 0.43,
        tilt: 0.63,
    };
    let center = Point2D::new(340.0, 220.0);
    let body = body(center, center, 20.0);
    let mut segments = vec![
        segment(key, LinkStratum::Surface),
        segment(key, LinkStratum::Resource),
    ];
    let fallbacks = attach_self_link_geometry(&mut segments, &camera, |_| Some(body));
    assert_eq!(fallbacks, 0);
    assert!(segments.iter().all(|item| !item.responsive));
    assert_eq!(segments[0].appearance.key.stratum, LinkStratum::Surface);
    assert_eq!(segments[1].appearance.key.stratum, LinkStratum::Resource);
    assert_ne!(segments[0].points[4], segments[1].points[4]);

    for segment in &segments {
        assert!(segment.points.len() >= 4);
        assert!(
            segment
                .points
                .iter()
                .all(|point| point.x.is_finite() && point.y.is_finite())
        );
        for endpoint in [segment.from, segment.to] {
            let screen =
                camera.to_screen(kernel::geometry::PortablePoint::new(endpoint.x, endpoint.y));
            let distance = body
                .signed_distance_screen(Point2D::new(screen.0, screen.1))
                .unwrap();
            assert!(
                distance.abs() < 0.03,
                "endpoint distance from body boundary: {distance}"
            );
        }
    }
}

#[test]
fn lifted_self_links_meet_ground_stem_foot_and_do_not_reenter_body() {
    let canvas = crate::canvas::Canvas::with_sample_graph();
    let key = canvas.graph.nodes().next().unwrap().0;
    let camera = Camera {
        offset: (0.0, 0.0),
        zoom: 1.3,
        yaw: -0.27,
        tilt: 0.71,
    };
    let body = body(Point2D::new(260.0, 140.0), Point2D::new(260.0, 220.0), 20.0);
    let mut segments = vec![
        segment(key, LinkStratum::Surface),
        segment(key, LinkStratum::Resource),
    ];
    let fallbacks = attach_self_link_geometry(&mut segments, &camera, |_| Some(body));
    assert_eq!(fallbacks, 0);
    let anchor_world = camera.to_world((body.ground_anchor.x, body.ground_anchor.y));
    for segment in &segments {
        assert!(!segment.responsive);
        assert!(
            segment
                .points
                .iter()
                .all(|point| point.x.is_finite() && point.y.is_finite())
        );
        assert_eq!(segment.from, Point2D::new(anchor_world.x, anchor_world.y));
        assert_eq!(segment.to, Point2D::new(anchor_world.x, anchor_world.y));

        let screen: Vec<_> = segment
            .points
            .iter()
            .map(|point| {
                let projected =
                    camera.to_screen(kernel::geometry::PortablePoint::new(point.x, point.y));
                Point2D::new(projected.0, projected.1)
            })
            .collect();
        let outside: Vec<_> = screen
            .iter()
            .enumerate()
            .filter_map(|(index, point)| {
                (body.signed_distance_screen(*point).unwrap() >= -0.05).then_some(index)
            })
            .collect();
        let first = *outside.first().unwrap();
        let last = *outside.last().unwrap();
        for endpoint in [screen[0], screen[screen.len() - 1]] {
            assert!(
                (endpoint.x - body.ground_anchor.x).hypot(endpoint.y - body.ground_anchor.y)
                    < 0.001
            );
        }
        let twice_area: f32 = screen
            .windows(2)
            .map(|pair| pair[0].x * pair[1].y - pair[1].x * pair[0].y)
            .sum();
        assert!(twice_area.abs() > 0.1, "raised self-link must enclose area");
        let expected_direction = if segment.appearance.key.stratum == LinkStratum::Surface {
            -1.0
        } else {
            1.0
        };
        assert!(
            screen[first..=last]
                .iter()
                .skip(1)
                .take(last - first - 1)
                .all(|point| (point.x - body.ground_anchor.x) * expected_direction > 0.0)
        );
        for pair in screen[first..=last].windows(2) {
            if let Some((enter, exit)) = body.segment_interior_interval(pair[0], pair[1]) {
                if exit - enter > f32::EPSILON {
                    let midpoint = Point2D::new(
                        pair[0].x + (pair[1].x - pair[0].x) * (enter + exit) * 0.5,
                        pair[0].y + (pair[1].y - pair[0].y) * (enter + exit) * 0.5,
                    );
                    assert!(body.signed_distance_screen(midpoint).unwrap() >= -0.05);
                }
            }
        }
    }
}
