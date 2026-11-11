// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::canvas::Canvas;
use kernel::geometry::PortablePoint;
use kernel::graph::apply::{GraphDelta, apply_graph_delta};
use kernel::graph::fixtures::GraphFixtures;
use kernel::graph::{
    Coupling, CouplingId, CouplingResponse, Field, FieldDefinition, FieldExtent, FieldId, Graph,
    NodeSelector, ScalarField,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use uuid::Uuid;

fn canvas(
    selector: NodeSelector,
    extent: FieldExtent,
) -> (Canvas, kernel::graph::NodeKey, FieldId) {
    let mut graph = Graph::new();
    let node = graph.add_node_with_id(
        Uuid::from_u128(42),
        "mere://field-runtime".into(),
        PortablePoint::zero(),
    );
    let field = FieldId::from_uuid(Uuid::from_u128(100));
    apply_graph_delta(
        &mut graph,
        GraphDelta::AddField {
            field: Field::new(field, FieldDefinition::Scalar(ScalarField::Const(1.0)))
                .with_extent(extent),
        },
    );
    apply_graph_delta(
        &mut graph,
        GraphDelta::AddCoupling {
            coupling: Coupling::new(
                CouplingId::from_uuid(Uuid::from_u128(101)),
                field,
                selector,
                CouplingResponse::open("mere://test/entry"),
                1.0,
            ),
        },
    );
    let mut canvas = Canvas::with_graph(graph);
    canvas.set_coupling_trace_limit(NonZeroUsize::new(8));
    (canvas, node, field)
}
fn step(canvas: &mut Canvas) {
    canvas.physics.settle(1);
    canvas.physics.advance_frame(&mut canvas.view);
}
fn counter(canvas: &mut Canvas) -> Arc<AtomicUsize> {
    let counter = Arc::new(AtomicUsize::new(0));
    let actual = Arc::clone(&counter);
    canvas.set_coupling_entry_handler(Some(Arc::new(move |report| {
        assert_eq!(report.identity.unwrap().member, 42);
        assert_eq!(
            report.binding.source.unwrap().coupling.as_uuid(),
            Uuid::from_u128(101)
        );
        actual.fetch_add(1, Ordering::SeqCst);
        Ok(())
    })));
    counter
}

#[test]
fn field_runtime_canvas_camera_changes_membership_neither_by_zoom_nor_pan() {
    let (mut canvas, _, _) = canvas(
        NodeSelector::All,
        FieldExtent::Region {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 100.0,
            max_y: 100.0,
        },
    );
    let counter = counter(&mut canvas);
    step(&mut canvas);
    let old = canvas.coupling_membership().unwrap().clone();
    assert_eq!(
        old.reports[0].eligibility,
        seiche::CouplingEligibility::Inside,
        "origin is an inclusive boundary point"
    );
    canvas.wheel(10.0, -20.0);
    canvas.frame(400, 300);
    canvas.set_ctrl(true);
    let zoom = canvas.camera.zoom;
    canvas.wheel(0.0, 40.0);
    assert_ne!(canvas.camera.zoom, zoom);
    canvas.frame(400, 300);
    canvas.set_ctrl(false);
    step(&mut canvas);
    let current = canvas.coupling_membership().unwrap();
    assert_eq!(current.reports[0].eligibility, old.reports[0].eligibility);
    assert_eq!(
        current.reports[0].reference_point,
        old.reports[0].reference_point
    );
    assert_eq!(
        current.reports[0].change,
        seiche::CouplingMembershipChange::Unchanged
    );
    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

#[test]
fn field_runtime_canvas_pause_and_reseed_wire_the_ruled_clock_reset() {
    let (mut canvas, _, _) = canvas(NodeSelector::All, FieldExtent::Global);
    let counter = counter(&mut canvas);
    step(&mut canvas);
    let clock = canvas.field_clock();
    assert!(clock.elapsed > 0.0);
    canvas.set_physics_paused(true);
    for _ in 0..3 {
        canvas.frame(400, 300);
    }
    assert_eq!(canvas.field_clock().elapsed, clock.elapsed);
    canvas.reseed();
    assert_eq!(canvas.field_clock().elapsed, 0.0);
    assert_eq!(canvas.field_clock().epoch, clock.epoch + 1);
    assert!(canvas.field_clock().paused);
    canvas.frame(400, 300);
    assert_eq!(canvas.field_clock().elapsed, 0.0);
    canvas.set_physics_paused(false);
    canvas.frame(400, 300);
    let report = &canvas.coupling_membership().unwrap().reports[0];
    assert_eq!(report.change, seiche::CouplingMembershipChange::Initial);
    assert!(report.cause.reset);
    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

#[test]
fn field_runtime_canvas_query_rebuilds_keep_stable_identity_and_later_entry_actions() {
    let (mut canvas, node, _) = canvas(NodeSelector::Tagged("lane".into()), FieldExtent::Global);
    let counter = counter(&mut canvas);
    step(&mut canvas);
    assert!(canvas.coupling_membership().unwrap().reports.is_empty());
    apply_graph_delta(
        &mut canvas.graph,
        GraphDelta::InsertNodeTag {
            key: node,
            tag: "lane".into(),
        },
    );
    canvas.rebuild_coupling_forces();
    step(&mut canvas);
    assert_eq!(
        canvas.coupling_membership().unwrap().reports[0].change,
        seiche::CouplingMembershipChange::Initial
    );
    assert_eq!(counter.load(Ordering::SeqCst), 0);
    apply_graph_delta(
        &mut canvas.graph,
        GraphDelta::RemoveNodeTag {
            key: node,
            tag: "lane".into(),
        },
    );
    canvas.rebuild_coupling_forces();
    step(&mut canvas);
    assert_eq!(
        canvas.coupling_membership().unwrap().reports[0].eligibility,
        seiche::CouplingEligibility::NotCandidate
    );
    apply_graph_delta(
        &mut canvas.graph,
        GraphDelta::InsertNodeTag {
            key: node,
            tag: "lane".into(),
        },
    );
    canvas.rebuild_coupling_forces();
    step(&mut canvas);
    let report = &canvas.coupling_membership().unwrap().reports[0];
    assert_eq!(report.change, seiche::CouplingMembershipChange::Entered);
    assert!(report.cause.query_change);
    assert_eq!(counter.load(Ordering::SeqCst), 1);
    assert_eq!(
        report.binding.selector,
        Some(NodeSelector::Tagged("lane".into()))
    );
}

#[test]
fn field_runtime_canvas_graph_swap_reinitializes_membership_and_retired_fields_do_not_bind() {
    let (mut canvas, _, field) = canvas(NodeSelector::All, FieldExtent::Global);
    let counter = counter(&mut canvas);
    step(&mut canvas);
    let epoch = canvas.field_clock().epoch;
    canvas.set_graph(canvas.graph.clone());
    step(&mut canvas);
    assert_eq!(canvas.field_clock().epoch, epoch + 1);
    assert_eq!(
        canvas.coupling_membership().unwrap().reports[0].change,
        seiche::CouplingMembershipChange::Initial
    );
    assert_eq!(counter.load(Ordering::SeqCst), 0);
    let mut graph = canvas.graph.clone();
    apply_graph_delta(&mut graph, GraphDelta::RetireField { id: field });
    let mut retired = Canvas::with_graph(graph);
    retired.set_coupling_trace_limit(NonZeroUsize::new(8));
    step(&mut retired);
    assert!(retired.coupling_trace().unwrap().records.is_empty());
    assert!(retired.coupling_membership().unwrap().reports.is_empty());
}
