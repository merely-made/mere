// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use kernel::graph::apply::{GraphDelta, apply_graph_delta};
use kernel::graph::{Field, FieldDefinition, FieldExtent, FieldId, ScalarField, VectorField};
use uuid::Uuid;

fn geometry(canvas: &Canvas) -> Vec<(LinkKey, NodeKey, NodeKey, Vec<(f32, f32)>)> {
    canvas
        .rendered_link_segments()
        .into_iter()
        .map(|segment| {
            (
                segment.appearance.key,
                segment.appearance.display_from,
                segment.appearance.display_to,
                segment
                    .points
                    .iter()
                    .map(|point| (point.x, point.y))
                    .collect(),
            )
        })
        .collect()
}

fn select_graph_field(canvas: &mut Canvas, id: FieldId) {
    let mut response = LinkPathResponse::default();
    response.field_response = true;
    response.selected_fields.push(id);
    response.config.control_points = 9;
    response.config.iterations = 8;
    response.config.field_gain = 1.0;
    response.config.max_step = 20.0;
    canvas.set_link_path_response(response);
}

fn add_field(canvas: &mut Canvas, field: Field) {
    apply_graph_delta(&mut canvas.graph, GraphDelta::AddField { field });
}

fn field_canvas() -> (Canvas, FieldId) {
    let mut canvas = Canvas::with_sample_graph();
    let id = FieldId::from_uuid(Uuid::from_u128(0xfeed));
    add_field(
        &mut canvas,
        Field::new(
            id,
            FieldDefinition::Vector(VectorField::ConstVec { x: 191.0, y: 503.0 }),
        )
        .with_name("ordinary-test-flow"),
    );
    (canvas, id)
}

fn assert_refusal_preserves_rest(mut canvas: Canvas, id: FieldId, expected: LinkPathFailure) {
    let before_revision = canvas.graph.revision();
    let before = geometry(&canvas);
    select_graph_field(&mut canvas, id);

    let report = canvas.link_path_response_report();
    assert_eq!(report.state, LinkPathState::Refused, "{report:?}");
    assert_eq!(report.failure, Some(expected));
    assert_eq!(canvas.graph.revision(), before_revision);
    assert_eq!(
        geometry(&canvas),
        before,
        "refusal must keep resting paint paths"
    );
    assert!(report.paths.iter().all(|path| !path.responsive));
}

fn assert_sample_refused(
    ast: VectorField,
    targets: Vec<Field>,
    retire: Option<FieldId>,
    expected: LinkPathFailure,
) {
    for graph_selected in [true, false] {
        let mut canvas = Canvas::with_sample_graph();
        for field in targets.iter().cloned() {
            add_field(&mut canvas, field);
        }
        if let Some(id) = retire {
            apply_graph_delta(&mut canvas.graph, GraphDelta::RetireField { id });
        }
        let (outer_id, selected_ast) = if graph_selected {
            let suffix = if retire.is_some() { 1 } else { 0 };
            let id = FieldId::from_uuid(Uuid::from_u128(0xf000 + suffix));
            add_field(
                &mut canvas,
                Field::new(id, FieldDefinition::Vector(ast.clone())),
            );
            (Some(id), None)
        } else {
            (None, Some(ast.clone()))
        };
        let revision = canvas.graph.revision();
        let before = geometry(&canvas);
        let mut response = LinkPathResponse::default();
        response.field_response = true;
        response.selected_fields.extend(outer_id);
        response.view_field = selected_ast.map(Into::into);
        response.config.control_points = 9;
        response.config.iterations = 8;
        canvas.set_link_path_response(response);

        let report = canvas.link_path_response_report();
        assert_eq!(report.state, LinkPathState::Refused, "{report:?}");
        assert_eq!(report.failure, Some(expected.clone()), "{report:?}");
        assert_eq!(geometry(&canvas), before);
        assert_eq!(canvas.graph.revision(), revision);
        assert!(report.paths.iter().all(|path| !path.responsive));
    }
}

#[test]
fn active_global_graph_vector_field_is_evaluated_from_its_graph_identity() {
    let (mut canvas, id) = field_canvas();
    let revision = canvas.graph.revision();
    select_graph_field(&mut canvas, id);

    let report = canvas.link_path_response_report();
    assert_eq!(report.state, LinkPathState::Applied, "{report:?}");
    assert!(report.paths.iter().any(|path| path.responsive));
    assert!(report.paths.iter().any(|path| {
        path.responsive && path.points.len() >= 2 && {
            let middle = path.points[path.points.len() / 2];
            let first = path.points[0];
            let last = *path.points.last().unwrap();
            let dx = last.0 - first.0;
            let dy = last.1 - first.1;
            ((dx * (first.1 - middle.1) - (first.0 - middle.0) * dy).abs())
                / dx.hypot(dy).max(f32::EPSILON)
                > 0.1
        }
    }));
    assert_eq!(canvas.graph.revision(), revision);
    assert!(canvas.link_path_response().view_field.is_none());
    assert_eq!(canvas.link_path_response().selected_fields, vec![id]);
}

#[test]
fn unsupported_graph_fields_refuse_and_retain_exact_rest_geometry() {
    let (canvas, id) = field_canvas();
    let mut scalar = canvas;
    // Replace the active vector with a scalar definition under the same identity.
    add_field(
        &mut scalar,
        Field::new(id, FieldDefinition::Scalar(ScalarField::Const(1.0))),
    );
    assert_refusal_preserves_rest(scalar, id, LinkPathFailure::ScalarField(id));

    let (mut retired, id) = field_canvas();
    apply_graph_delta(&mut retired.graph, GraphDelta::RetireField { id });
    assert_refusal_preserves_rest(retired, id, LinkPathFailure::RetiredField(id));

    let (mut regional, id) = field_canvas();
    add_field(
        &mut regional,
        Field::new(
            id,
            FieldDefinition::Vector(VectorField::ConstVec { x: 0.0, y: 500.0 }),
        )
        .with_extent(FieldExtent::Region {
            min_x: -10.0,
            min_y: -10.0,
            max_x: 10.0,
            max_y: 10.0,
        }),
    );
    assert_refusal_preserves_rest(regional, id, LinkPathFailure::UnsupportedExtent(id));

    let (canvas, _) = field_canvas();
    let missing = FieldId::from_uuid(Uuid::from_u128(0xbeef));
    assert_refusal_preserves_rest(canvas, missing, LinkPathFailure::MissingField(missing));
}

#[test]
fn graph_and_local_field_ast_samples_fail_closed_for_missing_retired_or_wrong_kind_targets() {
    let missing = FieldId::from_uuid(Uuid::from_u128(0xd001));
    assert_sample_refused(
        VectorField::Sample(missing),
        Vec::new(),
        None,
        LinkPathFailure::MissingField(missing),
    );

    let retired = FieldId::from_uuid(Uuid::from_u128(0xd002));
    assert_sample_refused(
        VectorField::Sample(retired),
        vec![Field::new(
            retired,
            FieldDefinition::Vector(VectorField::ConstVec { x: 0.0, y: 500.0 }),
        )],
        Some(retired),
        LinkPathFailure::RetiredField(retired),
    );

    let regional = FieldId::from_uuid(Uuid::from_u128(0xd003));
    assert_sample_refused(
        VectorField::Sample(regional),
        vec![
            Field::new(
                regional,
                FieldDefinition::Vector(VectorField::ConstVec { x: 0.0, y: 500.0 }),
            )
            .with_extent(FieldExtent::Region {
                min_x: -20.0,
                min_y: -20.0,
                max_x: 20.0,
                max_y: 20.0,
            }),
        ],
        None,
        LinkPathFailure::UnsupportedExtent(regional),
    );

    let scalar_target = FieldId::from_uuid(Uuid::from_u128(0xd004));
    assert_sample_refused(
        VectorField::Sample(scalar_target),
        vec![Field::new(
            scalar_target,
            FieldDefinition::Scalar(ScalarField::Const(1.0)),
        )],
        None,
        LinkPathFailure::FieldSampleKindMismatch(scalar_target),
    );

    let vector_target = FieldId::from_uuid(Uuid::from_u128(0xd005));
    assert_sample_refused(
        VectorField::Gradient(Box::new(ScalarField::Sample(vector_target))),
        vec![Field::new(
            vector_target,
            FieldDefinition::Vector(VectorField::ConstVec { x: 1.0, y: 0.0 }),
        )],
        None,
        LinkPathFailure::FieldSampleKindMismatch(vector_target),
    );
}

#[path = "responsive_links_controls_tests/geometry.rs"]
mod geometry;

#[path = "responsive_links_controls_tests/legibility.rs"]
mod legibility;

#[test]
fn deep_local_expression_refuses_before_geometry_work() {
    let mut canvas = Canvas::with_sample_graph();
    let resting = geometry(&canvas);
    let revision = canvas.graph.revision();
    let mut field = VectorField::ConstVec { x: 0.0, y: 10.0 };
    for _ in 0..=numen::MAX_FIELD_AST_DEPTH {
        field = VectorField::Perp(Box::new(field));
    }
    let mut response = LinkPathResponse::default();
    response.field_response = true;
    response.view_field = Some(field.into());
    canvas.set_link_path_response(response);
    let report = canvas.link_path_response_report();
    assert_eq!(report.state, LinkPathState::Refused);
    assert_eq!(
        report.failure,
        Some(LinkPathFailure::FieldDependencyTooLarge)
    );
    assert_eq!(geometry(&canvas), resting);
    assert_eq!(canvas.graph.revision(), revision);
}

#[test]
fn unrelated_invalid_field_does_not_enter_a_local_response_dependency_closure() {
    let mut canvas = Canvas::with_sample_graph();
    let missing = FieldId::from_uuid(Uuid::from_u128(0xdead));
    add_field(
        &mut canvas,
        Field::new(
            FieldId::from_uuid(Uuid::from_u128(0xbeef)),
            FieldDefinition::Vector(VectorField::Sample(missing)),
        ),
    );
    let mut response = LinkPathResponse::default();
    response.field_response = true;
    response.view_field = Some(VectorField::ConstVec { x: 0.0, y: 50.0 }.into());
    canvas.set_link_path_response(response);
    let report = canvas.link_path_response_report();
    assert_eq!(report.state, LinkPathState::Applied, "{report:?}");
    assert!(report.responsive_paths > 0);
}

#[test]
fn cyclic_graph_field_samples_refuse_without_replacing_resting_paths() {
    let mut canvas = Canvas::with_sample_graph();
    let a = FieldId::from_uuid(Uuid::from_u128(0xcafe));
    let b = FieldId::from_uuid(Uuid::from_u128(0xbabe));
    add_field(
        &mut canvas,
        Field::new(a, FieldDefinition::Vector(VectorField::Sample(b))),
    );
    add_field(
        &mut canvas,
        Field::new(b, FieldDefinition::Vector(VectorField::Sample(a))),
    );
    let resting = geometry(&canvas);
    let revision = canvas.graph.revision();
    select_graph_field(&mut canvas, a);
    let report = canvas.link_path_response_report();
    assert_eq!(report.state, LinkPathState::Refused);
    assert_eq!(
        report.failure,
        Some(LinkPathFailure::FieldDependencyTooLarge)
    );
    assert_eq!(geometry(&canvas), resting);
    assert_eq!(canvas.graph.revision(), revision);
}

#[test]
fn sample_chain_depth_is_bounded_across_stored_field_definitions() {
    let mut canvas = Canvas::with_sample_graph();
    let resting = geometry(&canvas);
    let ids: Vec<_> = (0..numen::MAX_FIELD_AST_DEPTH + 3)
        .map(|index| FieldId::from_uuid(uuid::Uuid::from_u128(0xf000 + index as u128)))
        .collect();
    for (index, id) in ids.iter().enumerate() {
        let vector = ids
            .get(index + 1)
            .copied()
            .map(VectorField::Sample)
            .unwrap_or(VectorField::ConstVec { x: 0.0, y: 10.0 });
        add_field(
            &mut canvas,
            Field::new(*id, FieldDefinition::Vector(vector)),
        );
    }
    let revision = canvas.graph.revision();
    select_graph_field(&mut canvas, ids[0]);
    let report = canvas.link_path_response_report();
    assert_eq!(report.state, LinkPathState::Refused);
    assert_eq!(
        report.failure,
        Some(LinkPathFailure::FieldDependencyTooLarge)
    );
    assert_eq!(geometry(&canvas), resting);
    assert_eq!(canvas.graph.revision(), revision);
}

#[path = "responsive_links_controls_tests/attachment.rs"]
mod attachment;
