// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
use super::*;
use kernel::geometry::PortablePoint;
use kernel::graph::apply::{GraphDelta, apply_graph_delta};
use kernel::graph::fixtures::GraphFixtures;
use kernel::graph::{
    Coupling, CouplingId, CouplingResponse, Field, FieldDefinition, FieldId, NodeSelector,
    VectorField,
};
use std::num::NonZeroUsize;

fn fixture() -> (Canvas, Uuid, Uuid, FieldId) {
    let mut graph = Graph::new();
    let first = Uuid::from_u128(11);
    let second = Uuid::from_u128(22);
    for member in [first, second] {
        graph.add_node_with_id(member, format!("mere://{member}"), PortablePoint::zero());
    }
    let field = FieldId::from_uuid(Uuid::from_u128(100));
    apply_graph_delta(
        &mut graph,
        GraphDelta::AddField {
            field: Field::new(
                field,
                FieldDefinition::Vector(VectorField::const_vec(8.0, 0.0)),
            ),
        },
    );
    apply_graph_delta(
        &mut graph,
        GraphDelta::AddCoupling {
            coupling: Coupling::new(
                CouplingId::from_uuid(Uuid::from_u128(101)),
                field,
                NodeSelector::All,
                CouplingResponse::FlowAdvect,
                1.0,
            ),
        },
    );
    let mut canvas = Canvas::with_graph(graph);
    canvas.resize(800, 600);
    let positions: Vec<_> = [first, second]
        .into_iter()
        .zip([-180.0, 180.0])
        .map(|(id, x)| {
            let key = canvas.graph.get_node_key_by_id(id).unwrap();
            let point = euclid::default::Point2D::new(x, 0.0);
            canvas.view.set_position(key, point);
            (key, point)
        })
        .collect();
    canvas.physics.seed(positions);
    (canvas, first, second, field)
}

fn tick(canvas: &mut Canvas) {
    canvas.physics.settle(1);
    canvas.physics.advance_frame(&mut canvas.view);
}

#[test]
fn bound_inspection_retains_edits_but_refuses_same_uuid_slot_recreation() {
    let (mut canvas, first, _, _) = fixture();
    let target = DynamicsTarget::Access { member: first };
    let subject = canvas.bind_dynamics_target(target).unwrap();
    let key = canvas.graph.get_node_key_by_id(first).unwrap();
    canvas.set_coupling_trace_limit(NonZeroUsize::new(8));
    tick(&mut canvas);
    assert!(canvas.coupling_trace().is_some());
    canvas.ingest_graph(|graph| {
        apply_graph_delta(
            graph,
            GraphDelta::SetNodeTitle {
                key,
                title: "Edited".into(),
            },
        );
        true
    });
    assert!(canvas.inspect_bound_dynamics(&subject).is_ok());
    tick(&mut canvas);
    assert!(canvas.coupling_trace().is_some());
    canvas.ingest_graph(|graph| {
        assert!(graph.remove_node(key));
        let replacement =
            graph.add_node_with_id(first, "mere://replacement".into(), PortablePoint::zero());
        assert_eq!(replacement, key);
        false // Kernel-observed membership changes must win over this hint.
    });
    assert!(
        canvas
            .inspect_bound_dynamics(&subject)
            .unwrap_err()
            .contains("ended")
    );
    assert!(canvas.coupling_trace().is_none());
    let fresh = canvas.bind_dynamics_target(target).unwrap();
    assert!(canvas.inspect_bound_dynamics(&fresh).is_ok());
}

#[test]
fn observed_endpoint_calls_are_filtered_and_read_without_ticking() {
    let (mut canvas, first, second, field) = fixture();
    canvas.set_coupling_trace_limit(NonZeroUsize::new(8));
    let target = DynamicsTarget::Access { member: first };
    assert!(
        canvas
            .inspect_dynamics(target)
            .unwrap()
            .coupling_observation["trace"]
            .is_null()
    );
    tick(&mut canvas);
    let before = canvas.coupling_trace().unwrap().clone();
    let pose: Vec<_> = canvas.view.positions().collect();
    let source = serde_json::to_value(canvas.graph.to_snapshot()).unwrap();
    let report = canvas.inspect_dynamics(target).unwrap();
    let rows = report.coupling_observation["trace"]["rows"]
        .as_array()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["member"], first.to_string());
    assert_eq!(
        report.coupling_observation["trace"]["bindings"]["0"]["source"]["field"],
        serde_json::to_value(field).unwrap()
    );
    assert!(rows[0]["evaluation"].as_str().unwrap().contains("Displace"));
    assert!(rows[0]["applied"].as_str().unwrap().contains("Write"));
    let pair = canvas
        .inspect_dynamics(DynamicsTarget::Pair { first, second })
        .unwrap();
    assert_eq!(
        pair.coupling_observation["trace"]["rows"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(canvas.coupling_trace(), Some(&before));
    assert_eq!(canvas.view.positions().collect::<Vec<_>>(), pose);
    assert_eq!(
        serde_json::to_value(canvas.graph.to_snapshot()).unwrap(),
        source
    );
}

#[test]
fn trace_discloses_cohort_truncation_and_is_cleared_by_binding_retirement() {
    let (mut canvas, first, second, field) = fixture();
    canvas.set_coupling_trace_limit(NonZeroUsize::new(1));
    tick(&mut canvas);
    let target = DynamicsTarget::Pair { first, second };
    let report = canvas.inspect_dynamics(target).unwrap();
    assert_eq!(report.coupling_observation["trace"]["cohort_omitted"], 1);
    assert_eq!(
        report.coupling_observation["trace"]["rows"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    canvas.delete_field(field);
    assert!(
        canvas
            .inspect_dynamics(target)
            .unwrap()
            .coupling_observation["trace"]
            .is_null()
    );
    tick(&mut canvas);
    assert_eq!(
        canvas
            .inspect_dynamics(target)
            .unwrap()
            .coupling_observation["trace"]["rows"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn pair_path_readback_preserves_exact_appearances_and_view_only_hides() {
    let (mut canvas, first, second, _) = fixture();
    assert!(canvas.assert_relation_between_members(first, second, SemanticSubKind::Cites));
    let target = DynamicsTarget::Pair { first, second };
    let reading = canvas.inspect_dynamics(target).unwrap();
    let path = &reading.path_response.as_ref().unwrap()["paths"][0];
    assert_eq!(path["responsive"], false);
    assert!(
        path["buckets"][0]["assertion_ids"]
            .as_array()
            .unwrap()
            .len()
            > 0
    );
    let mut response = canvas.link_path_response().clone();
    response.field_response = true;
    response.view_field = Some(std::sync::Arc::new(VectorField::const_vec(0.0, -1.0)));
    response.field_strength = 20.0;
    canvas.set_link_path_response(response);
    let source = serde_json::to_value(canvas.graph.to_snapshot()).unwrap();
    let responding = canvas.inspect_dynamics(target).unwrap();
    assert_eq!(
        responding.path_response.as_ref().unwrap()["state"],
        "applied"
    );
    assert_eq!(
        responding.path_response.as_ref().unwrap()["paths"][0]["responsive"],
        true
    );
    let ids: Vec<_> = canvas
        .graph
        .projected_assertions()
        .map(|(_, _, _, statement)| statement.statement_id.clone())
        .collect();
    canvas.set_hidden_assertion_ids(ids);
    let hidden = canvas.inspect_dynamics(target).unwrap();
    assert_eq!(hidden.pair_inputs.as_ref().unwrap().held_rows, 1);
    assert_eq!(hidden.pair_controller_edges, Some(0));
    assert!(
        hidden.path_response.as_ref().unwrap()["paths"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        serde_json::to_value(canvas.graph.to_snapshot()).unwrap(),
        source
    );
}

#[test]
fn bound_subject_cannot_be_used_by_another_canvas_with_the_same_graph() {
    let (canvas, first, _, _) = fixture();
    let subject = canvas
        .bind_dynamics_target(DynamicsTarget::Access { member: first })
        .unwrap();
    let other = Canvas::with_graph(canvas.graph.clone());
    assert!(other.inspect_bound_dynamics(&subject).is_err());
    assert!(canvas.inspect_bound_dynamics(&subject).is_ok());
}
