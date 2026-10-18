// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn ordinary_canvas_response_is_off_by_default_and_does_not_touch_graph_truth() {
    let canvas = Canvas::with_sample_graph();
    let revision = canvas.graph.revision();
    let report = canvas.link_path_response_report();
    assert_eq!(report.state, LinkPathState::Disabled);
    assert!(!report.paths.is_empty());
    assert!(report.paths.iter().all(|path| !path.responsive));
    assert_eq!(canvas.graph.revision(), revision);
}

#[test]
fn declared_view_field_replaces_live_canvas_rest_path_and_remains_pickable() {
    let mut canvas = Canvas::with_sample_graph();
    let revision = canvas.graph.revision();
    let topology = crate::canvas::seiche_bridge::visible_relation_edges_with_assertion_hides(
        &canvas.graph,
        &canvas.hidden_edges,
        &canvas.hidden_links,
        &canvas.hidden_assertions,
    );
    let mut settings = LinkPathResponse::default();
    settings.field_response = true;
    settings.view_field = Some(numen::VectorField::ConstVec { x: 0.0, y: 500.0 }.into());
    settings.config.control_points = 9;
    settings.config.iterations = 8;
    settings.config.field_gain = 1.0;
    settings.config.max_step = 20.0;
    canvas.set_link_path_response(settings);

    let report = canvas.link_path_response_report();
    assert_eq!(report.state, LinkPathState::Applied, "{report:?}");
    assert!(!report.paths.is_empty());
    let path = report
        .paths
        .iter()
        .find(|path| path.responsive && path.points.len() == 9)
        .unwrap();
    let chord_mid_y = (path.points[0].1 + path.points[path.points.len() - 1].1) * 0.5;
    assert!(
        (path.points[4].1 - chord_mid_y).abs() > 0.1,
        "path should respond to the selected vector field"
    );
    assert_eq!(
        canvas.graph.revision(),
        revision,
        "rendered path evaluation must not mutate graph truth"
    );
    assert_eq!(
        crate::canvas::seiche_bridge::visible_relation_edges_with_assertion_hides(
            &canvas.graph,
            &canvas.hidden_edges,
            &canvas.hidden_links,
            &canvas.hidden_assertions,
        ),
        topology,
        "render-only responses must not alter spring topology"
    );

    let pick = euclid::default::Point2D::new(path.points[4].0, path.points[4].1);
    let hit = crate::canvas::edge_cells::link_hit_test_segments(
        canvas.rendered_link_segments(),
        pick,
        0.25,
        |_| true,
        |_| true,
    )
    .expect("the rendered curve must be the pick geometry");
    assert_eq!(hit.key, path.key);
}

#[test]
fn hidden_member_of_explicit_compatibility_group_does_not_refuse_remaining_paths() {
    let mut canvas = Canvas::with_sample_graph();
    let keys: Vec<_> = canvas
        .visible_link_appearances()
        .into_iter()
        .map(|appearance| appearance.key)
        .collect();
    let hidden = keys[0].clone();
    let mut settings = LinkPathResponse::default();
    settings.field_response = true;
    settings.compatible_attraction = true;
    settings.compatible_links = keys;
    settings.view_field = Some(numen::VectorField::ConstVec { x: 0.0, y: 500.0 }.into());
    settings.config.control_points = 9;
    settings.config.iterations = 8;
    canvas.set_link_path_response(settings);
    canvas.hidden_links.insert(hidden.clone());

    let report = canvas.link_path_response_report();
    assert_eq!(report.state, LinkPathState::Applied, "{report:?}");
    assert!(
        report
            .paths
            .iter()
            .any(|path| path.responsive && path.key != hidden)
    );
    assert!(report.paths.iter().all(|path| path.key != hidden));

    canvas.hidden_links.remove(&hidden);
    let shown = canvas.link_path_response_report();
    assert_eq!(shown.state, LinkPathState::Applied);
    assert!(
        shown
            .paths
            .iter()
            .any(|path| path.key == hidden && path.responsive)
    );
}

#[test]
fn stale_explicit_compatibility_target_refuses_without_hiding_rest_geometry() {
    let mut canvas = Canvas::with_sample_graph();
    let resting = canvas.rendered_link_segments();
    let stale = LinkKey::new(LinkStratum::Surface, "missing:a", "missing:b");
    let mut settings = LinkPathResponse::default();
    settings.compatible_attraction = true;
    settings.compatible_links.push(stale.clone());
    canvas.set_link_path_response(settings);
    let report = canvas.link_path_response_report();
    assert_eq!(report.state, LinkPathState::Refused);
    assert_eq!(
        report.failure,
        Some(LinkPathFailure::StaleCompatibleLink(stale))
    );
    assert!(!resting.is_empty());
    assert_eq!(canvas.rendered_link_segments().len(), resting.len());
}

#[test]
fn graph_swap_clears_graph_field_and_compatibility_targets() {
    let mut canvas = Canvas::with_sample_graph();
    let mut settings = LinkPathResponse::default();
    settings.field_response = true;
    settings.compatible_attraction = true;
    settings.view_field = Some(numen::VectorField::ConstVec { x: 1.0, y: 0.0 }.into());
    settings.compatible_links.push(
        canvas
            .visible_link_appearances()
            .first()
            .unwrap()
            .key
            .clone(),
    );
    canvas.set_link_path_response(settings);

    canvas.set_graph(kernel::graph::Graph::new());
    assert_eq!(
        canvas.link_path_response_report().state,
        LinkPathState::Disabled
    );
    assert!(canvas.link_path_response().compatible_links.is_empty());
    assert!(canvas.link_path_response().selected_fields.is_empty());
    assert!(canvas.link_path_response().view_field.is_none());
}

#[test]
fn compatibility_only_keeps_ungrouped_paths_at_exact_rest_geometry() {
    let mut canvas = Canvas::with_sample_graph();
    let baseline = canvas.visible_link_paths();
    let grouped = baseline[0].key.clone();
    assert!(baseline.iter().any(|path| path.key != grouped));
    let mut settings = LinkPathResponse::default();
    settings.compatible_attraction = true;
    settings.compatible_links.push(grouped.clone());
    canvas.set_link_path_response(settings);
    let report = canvas.link_path_response_report();
    assert_eq!(report.state, LinkPathState::Applied);
    for before in baseline.iter().filter(|path| path.key != grouped) {
        let after = report
            .paths
            .iter()
            .find(|path| {
                path.key == before.key
                    && path.displayed_from == before.displayed_from
                    && path.displayed_to == before.displayed_to
            })
            .unwrap();
        assert_eq!(after.points, before.points);
        assert!(!after.responsive);
    }
    assert!(
        report
            .paths
            .iter()
            .any(|path| path.key == grouped && path.responsive)
    );
}
