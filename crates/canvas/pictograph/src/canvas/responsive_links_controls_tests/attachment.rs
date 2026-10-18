// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::geometry::alias_graph;
use super::legibility::{polyline_distance, source_facts};
use super::*;
use crate::canvas::{NodeShape, PointerButton, Viewport};
use euclid::default::Point2D;
use scenograph::suggestion::{SuggestionMark, SuggestionSettings};
use std::collections::HashMap;
use uuid::Uuid;

fn attached_canvas() -> (Canvas, [Uuid; 4], u64, Vec<String>) {
    let (graph, ids, assertions) = alias_graph();
    let mut canvas = Canvas::with_graph(graph);
    for (id, (x, y)) in ids.iter().zip([
        (-180.0, -80.0),
        (-180.0, 80.0),
        (180.0, -80.0),
        (180.0, 80.0),
    ]) {
        let key = canvas.graph.get_node_key_by_id(*id).unwrap();
        canvas.view.set_position(key, Point2D::new(x, y));
    }
    canvas.resize(1000, 800);
    canvas.set_physics_paused(true);
    canvas.set_viewport(Viewport {
        offset: (500.0, 400.0),
        zoom: 1.0,
        yaw: 0.0,
        tilt: 1.0,
        pan_velocity: (0.0, 0.0),
        view: (1000, 800),
    });
    let revision = canvas.graph.revision();
    (
        canvas,
        ids,
        revision,
        assertions.into_iter().map(|(id, _, _)| id).collect(),
    )
}

fn configure_bundle(canvas: &mut Canvas, ids: [Uuid; 4]) -> LinkKey {
    let report = canvas.link_path_response_report();
    let key = report
        .paths
        .iter()
        .find(|path| path.stratum == LinkStratum::Resource)
        .expect("resource cohort")
        .key
        .clone();
    let mut response = LinkPathResponse::default();
    response.field_response = true;
    response.compatible_attraction = true;
    response.compatible_links.push(key.clone());
    response.view_field = Some(VectorField::ConstVec { x: 0.0, y: 500.0 }.into());
    response.field_strength = 1.0;
    response.config.control_points = 17;
    response.config.iterations = 24;
    response.config.field_gain = 1.0;
    response.config.compatibility_gain = 1.0;
    response.config.smoothing = 0.08;
    response.config.max_step = 4.0;
    canvas.set_link_path_response(response);
    canvas.set_node_shapes(
        ids.into_iter()
            .map(|id| (id, NodeShape::Rounded))
            .collect::<HashMap<_, _>>(),
    );
    key
}

fn screen(canvas: &Canvas, point: (f32, f32)) -> Point2D<f32> {
    let (x, y) = canvas.screen_point_of(point);
    Point2D::new(x, y)
}

fn endpoint_interval_is_clear(canvas: &Canvas, path: &EvaluatedLinkPath, from: bool) -> bool {
    let id = if from {
        path.displayed_from
    } else {
        path.displayed_to
    };
    let Some(key) = canvas.graph.get_node_key_by_id(id) else {
        return false;
    };
    let Some(billboard) = canvas.node_billboard(key) else {
        return false;
    };
    let endpoint = if from {
        screen(canvas, path.points[0])
    } else {
        screen(canvas, *path.points.last().unwrap())
    };
    if (billboard.center.y - billboard.ground_anchor.y).abs() >= 0.5 {
        return (endpoint.x - billboard.ground_anchor.x)
            .hypot(endpoint.y - billboard.ground_anchor.y)
            < 0.05;
    }
    if billboard
        .signed_distance_screen(endpoint)
        .is_none_or(|distance| distance.abs() > 0.05)
    {
        return false;
    }
    if from {
        let a = screen(canvas, path.points[0]);
        let b = screen(canvas, path.points[1]);
        billboard
            .segment_interior_interval(a, b)
            .is_none_or(|(_, end)| end <= 0.002)
    } else {
        let a = screen(canvas, *path.points.last().unwrap());
        let b = screen(canvas, path.points[path.points.len() - 2]);
        billboard
            .segment_interior_interval(a, b)
            .is_none_or(|(_, end)| end <= 0.002)
    }
}

fn assert_paths_clear_of_billboards(canvas: &Canvas, report: &LinkPathResponseReport) {
    assert_eq!(report.state, LinkPathState::Applied, "{report:?}");
    for path in report.paths.iter().filter(|path| path.responsive) {
        assert!(path.points.len() >= 3);
        assert!(
            path.points
                .iter()
                .all(|(x, y)| x.is_finite() && y.is_finite())
        );
        assert!(
            endpoint_interval_is_clear(canvas, path, true),
            "from body overlap: {path:?}"
        );
        assert!(
            endpoint_interval_is_clear(canvas, path, false),
            "to body overlap: {path:?}"
        );
    }
}

fn most_isolated_resource_sample(
    canvas: &Canvas,
    paths: &[EvaluatedLinkPath],
    key: &LinkKey,
) -> Option<(f32, f32)> {
    let other_screen: Vec<_> = paths
        .iter()
        .filter(|path| &path.key != key)
        .map(|path| {
            path.points
                .iter()
                .map(|&point| canvas.screen_point_of(point))
                .collect::<Vec<_>>()
        })
        .collect();
    let bodies: Vec<_> = canvas
        .graph
        .nodes()
        .filter_map(|(key, _)| canvas.node_billboard(key))
        .collect();
    paths
        .iter()
        .filter(|path| &path.key == key)
        .flat_map(|path| path.points.windows(2))
        .map(|pair| ((pair[0].0 + pair[1].0) * 0.5, (pair[0].1 + pair[1].1) * 0.5))
        .filter_map(|world| {
            let point = canvas.screen_point_of(world);
            let p = Point2D::new(point.0, point.1);
            if bodies.iter().any(|body| {
                body.signed_distance_screen(p)
                    .is_none_or(|distance| distance < 8.0)
            }) {
                return None;
            }
            let clearance = other_screen
                .iter()
                .map(|path| polyline_distance(point, path))
                .fold(f32::INFINITY, f32::min);
            (clearance > 0.25).then_some((world, clearance))
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(point, _)| point)
}

#[test]
fn billboard_attachment_tracks_camera_shape_grammar_and_lift_without_source_changes() {
    let (mut canvas, ids, revision, assertion_ids) = attached_canvas();
    let baseline = canvas.link_path_response_report();
    let facts = source_facts(&baseline.paths);
    let key = configure_bundle(&mut canvas, ids);

    let profiles = [
        (0.0, 1.0, 1.0, false, false),
        (0.42, 0.58, 0.75, false, false),
        (1.1, 0.32, 1.6, false, false),
    ];
    for (yaw, tilt, zoom, lifted, grammar) in profiles {
        canvas.set_viewport(Viewport {
            offset: (500.0, 400.0),
            zoom,
            yaw,
            tilt,
            pan_velocity: (0.0, 0.0),
            view: (1000, 800),
        });
        canvas.set_height_by_degree(lifted);
        canvas.set_graph_grammar(grammar);
        let report = canvas.link_path_response_report();
        assert_paths_clear_of_billboards(&canvas, &report);
        assert!(report.responsive_paths > 0, "{report:?}");
        assert_eq!(source_facts(&report.paths), facts);
        assert_eq!(canvas.graph.revision(), revision);
        assert!(report.paths.iter().all(|path| {
            path.buckets
                .iter()
                .flat_map(|bucket| &bucket.assertion_ids)
                .all(|id| assertion_ids.contains(id))
        }));
    }

    // Every path endpoint remains anchored to the declared graph-plane foot
    // under a lifted billboard; the visual body center is above that foot.
    canvas.set_viewport(Viewport {
        offset: (500.0, 400.0),
        zoom: 1.25,
        yaw: 0.42,
        tilt: 0.58,
        pan_velocity: (0.0, 0.0),
        view: (1000, 800),
    });
    canvas.set_height_by_degree(true);
    let lifted = canvas.link_path_response_report();
    assert!(lifted.responsive_paths > 0, "{lifted:?}");
    for path in lifted.paths.iter().filter(|path| path.responsive) {
        for (id, tip) in [
            (path.displayed_from, path.points[0]),
            (path.displayed_to, *path.points.last().unwrap()),
        ] {
            let key = canvas.graph.get_node_key_by_id(id).unwrap();
            let billboard = canvas.node_billboard(key).unwrap();
            assert!(billboard.center.y < billboard.ground_anchor.y);
            assert!(billboard.ground_anchor.y.is_finite());
            let endpoint = screen(&canvas, tip);
            assert!(
                (endpoint.x - billboard.ground_anchor.x)
                    .hypot(endpoint.y - billboard.ground_anchor.y)
                    < 0.05
            );
        }
    }
    assert_eq!(source_facts(&lifted.paths), facts);
    assert_eq!(canvas.graph.revision(), revision);
    assert_eq!(key, lifted.settings.compatible_links[0]);
}

#[test]
fn rounded_circle_and_suggestion_scaled_billboards_share_attachment_clip_geometry() {
    let (mut canvas, ids, revision, _) = attached_canvas();
    let baseline = canvas.link_path_response_report();
    let facts = source_facts(&baseline.paths);
    configure_bundle(&mut canvas, ids);

    for shape in [NodeShape::Rounded, NodeShape::Circle] {
        canvas.set_node_shapes(ids.into_iter().map(|id| (id, shape)).collect());
        let report = canvas.link_path_response_report();
        assert_paths_clear_of_billboards(&canvas, &report);
        assert!(report.responsive_paths > 0, "{report:?}");
        assert_eq!(source_facts(&report.paths), facts);
        for id in ids {
            let key = canvas.graph.get_node_key_by_id(id).unwrap();
            let billboard = canvas.node_billboard(key).unwrap();
            let half = billboard.local_side * 0.5;
            if shape == NodeShape::Circle {
                assert!((billboard.corner_radius - half).abs() < 0.01);
            } else {
                assert!(billboard.corner_radius > 0.0 && billboard.corner_radius < half);
            }
        }
    }

    canvas.set_node_shapes(HashMap::new());
    canvas.set_graph_grammar(true);
    let offered = ids[0];
    canvas.offer_suggestions([(offered, "attachment-test".to_owned())]);
    canvas.set_suggestion_settings(SuggestionSettings {
        on: true,
        at_once: 2,
        tracked: 20,
        mark: SuggestionMark::Word,
        sources: Vec::new(),
    });
    let report = canvas.link_path_response_report();
    assert_paths_clear_of_billboards(&canvas, &report);
    assert!(report.responsive_paths > 0, "{report:?}");
    assert_eq!(source_facts(&report.paths), facts);
    let suggested = canvas
        .node_billboard(canvas.graph.get_node_key_by_id(offered).unwrap())
        .unwrap();
    let ordinary = canvas
        .node_billboard(canvas.graph.get_node_key_by_id(ids[1]).unwrap())
        .unwrap();
    assert!(suggested.local_side <= ordinary.local_side);
    assert_eq!(canvas.graph.revision(), revision);
}

#[test]
fn custom_physics_hulls_do_not_replace_billboards_or_move_raised_face_stems() {
    let (mut canvas, ids, revision, _) = attached_canvas();
    configure_bundle(&mut canvas, ids);
    let key = canvas.graph.get_node_key_by_id(ids[0]).unwrap();
    canvas.set_node_shapes(ids.into_iter().map(|id| (id, NodeShape::Circle)).collect());
    canvas.set_height_by_degree(true);
    canvas.set_graph_grammar(true);
    canvas.set_viewport(Viewport {
        offset: (500.0, 400.0),
        zoom: 1.4,
        yaw: 0.37,
        tilt: 0.61,
        pan_velocity: (0.0, 0.0),
        view: (1000, 800),
    });
    let before = canvas.node_billboard(key).unwrap();
    let face_before = canvas.node_face_rect(key).unwrap();
    let face_center = (
        (face_before.0 + face_before.2) * 0.5,
        (face_before.1 + face_before.3) * 0.5,
    );
    assert!((face_center.0 - before.center.x).abs() < 0.01);
    assert!((face_center.1 - before.center.y).abs() < 0.01);
    assert!(
        ((face_before.2 - face_before.0) - before.local_side * before.zoom * 0.72).abs() < 0.02
    );
    let before_report = canvas.link_path_response_report();
    assert!(before_report.responsive_paths > 0, "{before_report:?}");
    let before_paths = before_report
        .paths
        .into_iter()
        .map(|path| {
            (
                path.key,
                path.displayed_from,
                path.displayed_to,
                path.stratum,
                path.points,
            )
        })
        .collect::<Vec<_>>();

    canvas.set_node_sprite_hull(ids[0], vec![(-0.45, -0.45), (0.45, -0.45), (0.0, 0.45)]);
    assert!(canvas.node_sprite_hull(key).is_some());
    let after = canvas.node_billboard(key).unwrap();
    assert_eq!(
        after, before,
        "a collider hull is independent of the CSS billboard"
    );
    let face_after = canvas.node_face_rect(key).unwrap();
    assert_eq!(face_after, face_before);
    let after_report = canvas.link_path_response_report();
    assert!(after_report.responsive_paths > 0, "{after_report:?}");
    let after_paths = after_report
        .paths
        .into_iter()
        .map(|path| {
            (
                path.key,
                path.displayed_from,
                path.displayed_to,
                path.stratum,
                path.points,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(after_paths, before_paths);
    assert_eq!(canvas.graph.revision(), revision);
}

#[test]
fn report_paint_pointer_and_rectangle_selection_use_the_same_projected_paths() {
    let (mut canvas, ids, revision, _) = attached_canvas();
    let key = configure_bundle(&mut canvas, ids);
    canvas.set_viewport(Viewport {
        offset: (500.0, 400.0),
        zoom: 0.8,
        yaw: 0.5,
        tilt: 0.62,
        pan_velocity: (0.0, 0.0),
        view: (1000, 800),
    });
    let report = canvas.link_path_response_report();
    assert_paths_clear_of_billboards(&canvas, &report);
    let painted = canvas.rendered_link_segments();
    for path in &report.paths {
        let segment = painted
            .iter()
            .find(|segment| {
                segment.appearance.key == path.key
                    && segment.appearance.display_from
                        == canvas
                            .graph
                            .get_node_key_by_id(path.displayed_from)
                            .unwrap()
                    && segment.appearance.display_to
                        == canvas.graph.get_node_key_by_id(path.displayed_to).unwrap()
            })
            .expect("report path has a paint path");
        assert_eq!(
            segment
                .points
                .iter()
                .map(|p| (p.x, p.y))
                .collect::<Vec<_>>(),
            path.points
        );
    }
    let sample = most_isolated_resource_sample(&canvas, &report.paths, &key)
        .expect("resource alias has a sample away from other canonical links");
    let projected = canvas.screen_point_of(sample);
    canvas.pointer_down(PointerButton::Left, projected.0, projected.1);
    assert!(canvas.pointer_up(PointerButton::Left, projected.0, projected.1));
    assert!(canvas.selected_link_keys().contains(&key));

    canvas.pointer_down(PointerButton::Right, projected.0 - 4.0, projected.1 - 4.0);
    assert!(canvas.pointer_up(PointerButton::Right, projected.0 + 4.0, projected.1 + 4.0));
    assert!(canvas.selected_link_keys().contains(&key));
    assert_eq!(canvas.graph.revision(), revision);
}

#[test]
fn invalid_camera_fails_closed_without_nonfinite_billboard_or_path_geometry() {
    let (mut canvas, ids, revision, _) = attached_canvas();
    configure_bundle(&mut canvas, ids);
    canvas.set_yaw(f32::NAN);
    let key = canvas.graph.get_node_key_by_id(ids[0]).unwrap();
    assert!(canvas.node_billboard(key).is_none());
    let report = canvas.link_path_response_report();
    assert_eq!(report.state, LinkPathState::Refused, "{report:?}");
    assert_eq!(report.failure, Some(LinkPathFailure::InvalidProjection));
    assert_eq!(
        report.presentation_fallbacks,
        report.paths.len(),
        "every non-self appearance keeps its finite resting geometry"
    );
    assert!(report.paths.iter().all(|path| {
        path.points
            .iter()
            .all(|(x, y)| x.is_finite() && y.is_finite())
    }));
    assert_eq!(canvas.graph.revision(), revision);
}
