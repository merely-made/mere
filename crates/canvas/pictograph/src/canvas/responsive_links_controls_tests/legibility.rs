// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::geometry::alias_graph;
use super::*;
use uuid::Uuid;

pub(super) fn source_facts(paths: &[EvaluatedLinkPath]) -> Vec<String> {
    let mut facts: Vec<_> = paths
        .iter()
        .map(|path| {
            format!(
                "{:?}|{:?}|{}|{}|{:?}",
                path.key,
                path.stratum,
                path.displayed_from,
                path.displayed_to,
                path.buckets
                    .iter()
                    .map(|bucket| (
                        bucket.displayed_from.as_str(),
                        bucket.displayed_to.as_str(),
                        bucket.assertion_ids.as_slice(),
                        bucket.legacy_members.as_slice(),
                    ))
                    .collect::<Vec<_>>()
            )
        })
        .collect();
    facts.sort();
    facts
}

fn path_geometry(
    paths: &[EvaluatedLinkPath],
) -> Vec<(LinkKey, Uuid, Uuid, LinkStratum, Vec<(f32, f32)>)> {
    let mut geometry: Vec<_> = paths
        .iter()
        .map(|path| {
            (
                path.key.clone(),
                path.displayed_from,
                path.displayed_to,
                path.stratum,
                path.points.clone(),
            )
        })
        .collect();
    geometry.sort_by_key(|(key, from, to, stratum, _)| (key.clone(), *from, *to, *stratum));
    geometry
}

fn point_segment_distance(point: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let delta = (b.0 - a.0, b.1 - a.1);
    let length_squared = delta.0 * delta.0 + delta.1 * delta.1;
    let t = if length_squared == 0.0 {
        0.0
    } else {
        (((point.0 - a.0) * delta.0 + (point.1 - a.1) * delta.1) / length_squared).clamp(0.0, 1.0)
    };
    (point.0 - (a.0 + t * delta.0)).hypot(point.1 - (a.1 + t * delta.1))
}

pub(super) fn polyline_distance(point: (f32, f32), points: &[(f32, f32)]) -> f32 {
    points
        .windows(2)
        .map(|pair| point_segment_distance(point, pair[0], pair[1]))
        .fold(f32::INFINITY, f32::min)
}

#[test]
fn grouped_alias_bundle_stays_bounded_painted_pickable_and_source_identified() {
    let (graph, ids, assertions) = alias_graph();
    let mut canvas = Canvas::with_graph(graph);
    for (id, (x, y)) in ids.iter().zip([
        (-180.0, -80.0),
        (-180.0, 80.0),
        (180.0, -80.0),
        (180.0, 80.0),
    ]) {
        let key = canvas.graph.get_node_key_by_id(*id).unwrap();
        canvas
            .view
            .set_position(key, euclid::default::Point2D::new(x, y));
    }
    let revision = canvas.graph.revision();
    let baseline = canvas.link_path_response_report();
    let baseline_facts = source_facts(&baseline.paths);
    let baseline_geometry = path_geometry(&baseline.paths);
    let resource_key = baseline
        .paths
        .iter()
        .find(|path| path.stratum == LinkStratum::Resource)
        .expect("resource alias cohort")
        .key
        .clone();
    assert_eq!(
        baseline
            .paths
            .iter()
            .filter(|path| path.stratum == LinkStratum::Resource)
            .count(),
        4
    );

    let mut response = LinkPathResponse::default();
    response.compatible_attraction = true;
    response.compatible_links.push(resource_key.clone());
    response.config.control_points = 17;
    response.config.iterations = 24;
    response.config.compatibility_gain = 1.0;
    response.config.smoothing = 0.08;
    response.config.max_step = 4.0;
    canvas.set_link_path_response(response);

    let report = canvas.link_path_response_report();
    assert_eq!(report.state, LinkPathState::Applied, "{report:?}");
    assert_eq!(
        report.responsive_paths, 4,
        "one path per displayed resource alias"
    );
    assert_ne!(path_geometry(&report.paths), baseline_geometry);
    assert_eq!(source_facts(&report.paths), baseline_facts);
    assert_eq!(canvas.graph.revision(), revision);

    let mut resource_count = 0;
    for path in report
        .paths
        .iter()
        .filter(|path| path.stratum == LinkStratum::Resource)
    {
        resource_count += 1;
        assert!(path.responsive);
        assert!(path.points.len() <= 128);
        assert!(
            path.points
                .iter()
                .all(|(x, y)| x.is_finite() && y.is_finite())
        );
        assert!(
            path.points
                .iter()
                .all(|(x, y)| x.abs() <= 1_000.0 && y.abs() <= 1_000.0)
        );
        assert!(
            path.buckets
                .iter()
                .flat_map(|bucket| &bucket.assertion_ids)
                .all(|id| assertions.iter().any(|(expected, _, _)| expected == id)),
            "every evaluated alias still exposes exact source assertions"
        );

        let from_key = canvas
            .graph
            .get_node_key_by_id(path.displayed_from)
            .unwrap();
        let to_key = canvas.graph.get_node_key_by_id(path.displayed_to).unwrap();
        let from_center = canvas.view.position_of(from_key).unwrap();
        let to_center = canvas.view.position_of(to_key).unwrap();
        // The terminal tangents leave/approach the actual attachment face.
        // A strongly bent path need not be monotone along its unbent chord.
        for (key, center, tip, neighbor, outgoing) in [
            (from_key, from_center, path.points[0], path.points[1], true),
            (
                to_key,
                to_center,
                *path.points.last().unwrap(),
                path.points[path.points.len() - 2],
                false,
            ),
        ] {
            let half = canvas.node_size(key) * 0.5;
            let offset = (tip.0 - center.x, tip.1 - center.y);
            let normal = if offset.0.abs() >= offset.1.abs() {
                (offset.0.signum(), 0.0)
            } else {
                (0.0, offset.1.signum())
            };
            assert!((offset.0.abs().max(offset.1.abs()) - half).abs() < 0.01);
            let step_out = (neighbor.0 - tip.0) * normal.0 + (neighbor.1 - tip.1) * normal.1;
            assert!(
                step_out >= -0.01,
                "attachment tangent enters node; outgoing={outgoing}"
            );
        }

        // The evaluator's samples are clipped to the node square faces, and
        // no later sample may re-enter either endpoint face.
        for (key, center) in [(from_key, from_center), (to_key, to_center)] {
            let half = canvas.node_size(key) * 0.5;
            let outside = |(x, y): &(f32, f32)| {
                (x - center.x).abs() >= half - 0.01 || (y - center.y).abs() >= half - 0.01
            };
            if key == from_key {
                assert!(outside(&path.points[0]));
                assert!(path.points[1..].iter().all(outside));
            } else {
                assert!(outside(path.points.last().unwrap()));
                assert!(path.points[..path.points.len() - 1].iter().all(outside));
            }
        }

        // The report and paint/pick adapter must use the same presented curve.
        let segments = canvas.rendered_link_segments();
        let segment = segments
            .iter()
            .find(|segment| {
                segment.appearance.key == path.key
                    && segment.appearance.display_from == from_key
                    && segment.appearance.display_to == to_key
            })
            .expect("painted displayed alias path");
        assert_eq!(
            segment
                .points
                .iter()
                .map(|point| (point.x, point.y))
                .collect::<Vec<_>>(),
            path.points
        );
        let sample = path
            .points
            .windows(2)
            .map(|pair| ((pair[0].0 + pair[1].0) * 0.5, (pair[0].1 + pair[1].1) * 0.5))
            .max_by(|a, b| {
                let clearance = |point: &(f32, f32)| {
                    segments
                        .iter()
                        .filter(|other| {
                            other.appearance.key != path.key
                                || other.appearance.display_from != from_key
                                || other.appearance.display_to != to_key
                        })
                        .map(|other| {
                            polyline_distance(
                                *point,
                                &other.points.iter().map(|p| (p.x, p.y)).collect::<Vec<_>>(),
                            )
                        })
                        .fold(f32::INFINITY, f32::min)
                };
                clearance(a).total_cmp(&clearance(b))
            })
            .expect("responsive curve has a painted segment");
        let clearance = segments
            .iter()
            .filter(|other| {
                other.appearance.key != path.key
                    || other.appearance.display_from != from_key
                    || other.appearance.display_to != to_key
            })
            .map(|other| {
                polyline_distance(
                    sample,
                    &other.points.iter().map(|p| (p.x, p.y)).collect::<Vec<_>>(),
                )
            })
            .fold(f32::INFINITY, f32::min);
        assert!(
            clearance > 0.25,
            "pick sample must avoid overlapping alias paths"
        );
        let hit = crate::canvas::edge_cells::link_hit_test_segments(
            segments,
            euclid::default::Point2D::new(sample.0, sample.1),
            0.25,
            |_| true,
            |_| true,
        )
        .expect("responsive path remains pickable along its sampled geometry");
        assert_eq!(hit.key, path.key);
        assert_eq!(hit.display_from, from_key);
        assert_eq!(hit.display_to, to_key);
    }
    assert_eq!(resource_count, 4);

    let surface = report
        .paths
        .iter()
        .filter(|path| path.stratum == LinkStratum::Surface)
        .collect::<Vec<_>>();
    assert_eq!(surface.len(), 4);
    assert!(surface.iter().all(|path| !path.responsive));
    for path in surface {
        let resting = baseline
            .paths
            .iter()
            .find(|baseline| {
                baseline.key == path.key
                    && baseline.displayed_from == path.displayed_from
                    && baseline.displayed_to == path.displayed_to
                    && baseline.stratum == path.stratum
            })
            .unwrap();
        assert_eq!(
            path.points, resting.points,
            "ungrouped surface geometry is unchanged"
        );
    }

    let mut combined = canvas.link_path_response().clone();
    combined.field_response = true;
    combined.view_field = Some(VectorField::ConstVec { x: 0.0, y: 500.0 }.into());
    combined.field_strength = 1.0;
    combined.config.field_gain = 1.0;
    combined.config.max_step = 20.0;
    canvas.set_link_path_response(combined);
    let combined_report = canvas.link_path_response_report();
    assert_eq!(
        combined_report.state,
        LinkPathState::Applied,
        "{combined_report:?}"
    );
    assert_eq!(combined_report.responsive_paths, 8);
    assert_eq!(source_facts(&combined_report.paths), baseline_facts);
    assert_eq!(canvas.graph.revision(), revision);

    let changed_position_id = ids[0];
    let changed_position_key = canvas
        .graph
        .get_node_key_by_id(changed_position_id)
        .unwrap();
    canvas.view.set_position(
        changed_position_key,
        euclid::default::Point2D::new(-145.0, -55.0),
    );
    let moved = canvas.link_path_response_report();
    assert_eq!(moved.state, LinkPathState::Applied, "{moved:?}");
    assert_eq!(source_facts(&moved.paths), baseline_facts);
    assert_ne!(
        path_geometry(&moved.paths),
        path_geometry(&combined_report.paths)
    );

    canvas.set_hidden_link_keys([resource_key]);
    let hidden = canvas.link_path_response_report();
    assert_eq!(hidden.state, LinkPathState::Applied, "{hidden:?}");
    assert_eq!(hidden.paths.len(), 4);
    assert!(
        hidden
            .paths
            .iter()
            .all(|path| path.stratum == LinkStratum::Surface)
    );
    assert_eq!(canvas.graph.revision(), revision);
}

#[test]
fn overlapping_endpoint_faces_keep_resting_paths_and_count_presentation_fallbacks() {
    let (graph, ids, _) = alias_graph();
    let mut canvas = Canvas::with_graph(graph);
    for id in ids {
        let key = canvas.graph.get_node_key_by_id(id).unwrap();
        canvas
            .view
            .set_position(key, euclid::default::Point2D::new(0.0, 0.0));
    }
    let revision = canvas.graph.revision();
    let resting = canvas.link_path_response_report();
    let mut response = LinkPathResponse::default();
    response.field_response = true;
    response.view_field = Some(VectorField::ConstVec { x: 0.0, y: 500.0 }.into());
    response.config.field_gain = 1.0;
    response.config.max_step = 20.0;
    canvas.set_link_path_response(response);

    let report = canvas.link_path_response_report();
    assert_eq!(report.state, LinkPathState::Applied, "{report:?}");
    assert!(report.presentation_fallbacks > 0, "{report:?}");
    assert_eq!(report.paths.len(), resting.paths.len());
    assert!(report.paths.iter().any(|path| !path.responsive));
    assert_eq!(canvas.graph.revision(), revision);
}

#[test]
fn compatibility_only_cohort_drops_more_than_sixty_four_unselected_readings() {
    use kernel::geometry::PortablePoint;
    use kernel::graph::apply::add_node;
    use kernel::graph::{Graph, GraphStratum, SemanticStatementSpec, SemanticSubKind};

    const PAIRS: usize = 65;
    let mut graph = Graph::new();
    let predicate = "urn:test:responsive-cohort";
    graph
        .declare_predicate(predicate, GraphStratum::Surface)
        .unwrap();
    let mut pair_ids = Vec::with_capacity(PAIRS);
    for index in 0..PAIRS {
        let from_id = Uuid::from_u128(0x1000 + (index as u128) * 2);
        let to_id = Uuid::from_u128(0x1001 + (index as u128) * 2);
        let y = index as f32 * 160.0;
        let from = add_node(
            &mut graph,
            Some(from_id),
            format!("https://cohort.test/{index}/from"),
            PortablePoint::new(-150.0, y),
        );
        let to = add_node(
            &mut graph,
            Some(to_id),
            format!("https://cohort.test/{index}/to"),
            PortablePoint::new(150.0, y),
        );
        graph
            .try_assert_semantic_statement(
                from,
                to,
                SemanticStatementSpec {
                    predicate: predicate.into(),
                    recognized_sub_kind: Some(SemanticSubKind::Cites),
                    label: Some(format!("cohort pair {index}")),
                    ..Default::default()
                },
            )
            .unwrap();
        pair_ids.push((from_id, to_id, y));
    }

    let mut canvas = Canvas::with_graph(graph);
    for (from_id, to_id, y) in pair_ids {
        for (id, x) in [(from_id, -150.0), (to_id, 150.0)] {
            let key = canvas.graph.get_node_key_by_id(id).unwrap();
            canvas
                .view
                .set_position(key, euclid::default::Point2D::new(x, y));
        }
    }

    let baseline = canvas.visible_link_paths();
    assert_eq!(
        baseline.len(),
        PAIRS,
        "the graph has 65 distinct pair readings"
    );
    let selected = baseline[0].key.clone();
    let mut response = LinkPathResponse::default();
    response.compatible_attraction = true;
    response.compatible_links.push(selected.clone());
    response.config.control_points = 9;
    response.config.iterations = 1;
    canvas.set_link_path_response(response);

    let report = canvas.link_path_response_report();
    assert_eq!(report.state, LinkPathState::Applied, "{report:?}");
    assert_eq!(report.paths.len(), PAIRS);
    assert_eq!(
        report.responsive_paths, 1,
        "only the declared key is evaluated"
    );
    let evaluated = report
        .paths
        .iter()
        .filter(|path| path.responsive)
        .collect::<Vec<_>>();
    assert_eq!(evaluated.len(), 1);
    assert_eq!(evaluated[0].key, selected);
    for path in report.paths.iter().filter(|path| path.key != selected) {
        assert!(!path.responsive);
        let resting = baseline
            .iter()
            .find(|resting| {
                resting.key == path.key
                    && resting.displayed_from == path.displayed_from
                    && resting.displayed_to == path.displayed_to
            })
            .expect("every unselected source pair keeps its resting path");
        assert_eq!(path.points, resting.points);
    }
}
