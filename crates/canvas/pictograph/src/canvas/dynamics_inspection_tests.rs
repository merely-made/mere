// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
use super::*;
#[test]
fn reading_dynamics_preserves_graph_selection_pose_and_pause() {
    let mut canvas = Canvas::with_sample_graph();
    let ids: Vec<_> = canvas
        .graph
        .nodes()
        .take(2)
        .map(|(_, node)| node.id)
        .collect();
    canvas.select_member(ids[0]);
    canvas.pin_focused();
    let graph = serde_json::to_value(canvas.graph.to_snapshot()).unwrap();
    let selected = canvas.selected_members();
    let pose: Vec<_> = canvas.view.positions().collect();
    let paused = canvas.physics_paused();
    let report = canvas
        .inspect_dynamics(DynamicsTarget::Pair {
            first: ids[0],
            second: ids[1],
        })
        .unwrap();
    assert!(report.members[0].explicit_position_pin);
    assert_eq!(report.members[0].role, Role::Pinned);
    assert_eq!(
        report.pair_controller_edges,
        report.pair_inputs.as_ref().map(|pair| pair.visible_rows)
    );
    assert_eq!(
        serde_json::to_value(canvas.graph.to_snapshot()).unwrap(),
        graph
    );
    assert_eq!(canvas.selected_members(), selected);
    assert_eq!(canvas.view.positions().collect::<Vec<_>>(), pose);
    assert_eq!(canvas.physics_paused(), paused);
}
#[test]
fn forme_hold_is_separate_from_explicit_pin_and_hidden_boundary() {
    let mut canvas = Canvas::with_sample_graph();
    let id = canvas.graph.nodes().next().unwrap().1.id;
    canvas
        .set_forme_region(Some(FormeRegion {
            id: Uuid::from_u128(44),
            bounds: [0., 0., 240., 180.],
            cells: vec![FormeCell {
                member: id,
                bounds: [0., 0., 1., 1.],
            }],
            locked: true,
            visible: false,
        }))
        .unwrap();
    let target = DynamicsTarget::Access { member: id };
    let report = canvas.inspect_dynamics(target).unwrap();
    assert!(report.members[0].forme_hold);
    assert!(!report.members[0].explicit_position_pin);
    canvas.select_member(id);
    canvas.pin_focused();
    let report = canvas.inspect_dynamics(target).unwrap();
    assert!(report.members[0].explicit_position_pin);
    assert!(!report.members[0].forme_hold);
}
#[test]
fn missing_exact_member_cannot_bind_to_new_graph() {
    let mut canvas = Canvas::with_sample_graph();
    let id = canvas.graph.nodes().next().unwrap().1.id;
    canvas.set_graph(Graph::new());
    assert!(
        canvas
            .inspect_dynamics(DynamicsTarget::Access { member: id })
            .is_err()
    );
}
