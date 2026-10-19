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
    assert_eq!(report.topology_mode, DynamicsTopologyMode::RelationRows);
    assert_eq!(report.controller_edges, canvas.controller_topology().len());
    assert_eq!(
        report.controller_edge_runs,
        canvas.controller_topology().run_count()
    );
    assert!(report.members[0].explicit_position_pin);
    assert_eq!(report.members[0].role, Role::Pinned);
    assert_eq!(
        report.pair_controller_edges,
        report.pair_inputs.as_ref().map(|pair| pair.visible_rows)
    );
    assert_eq!(
        report.pair_topology_rows.as_ref().map(Vec::len),
        report.pair_controller_edges
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
fn occurrence_controller_mode_is_explicit_and_inspection_uses_its_ordered_runs() {
    let mut canvas = Canvas::with_sample_graph();
    let keys: Vec<_> = canvas.graph.nodes().take(2).map(|(key, _)| key).collect();
    let ids = [
        canvas.graph.get_node(keys[0]).unwrap().id,
        canvas.graph.get_node(keys[1]).unwrap().id,
    ];
    canvas.occurrence_edges = Some(vec![
        (keys[0], keys[1]),
        (keys[0], keys[1]),
        (keys[1], keys[0]),
    ]);
    canvas
        .physics
        .sync_edge_topology(canvas.controller_topology());

    let report = canvas
        .inspect_dynamics(DynamicsTarget::Pair {
            first: ids[0],
            second: ids[1],
        })
        .unwrap();
    assert_eq!(
        report.topology_mode,
        DynamicsTopologyMode::OccurrenceController
    );
    assert_eq!(report.controller_edges, 3);
    assert_eq!(report.controller_edge_runs, 2);
    assert_eq!(report.pair_controller_edges, Some(3));
    assert_eq!(report.pair_controller_edge_runs, Some(2));
    assert!(report.pair_inputs.is_none());
    assert!(report.pair_topology_rows.is_none());

    canvas.occurrence_edges = Some(Vec::new());
    let empty = canvas
        .inspect_dynamics(DynamicsTarget::Pair {
            first: ids[0],
            second: ids[1],
        })
        .unwrap();
    assert_eq!(
        empty.topology_mode,
        DynamicsTopologyMode::OccurrenceController
    );
    assert_eq!(empty.controller_edges, 0);
    assert_eq!(empty.controller_edge_runs, 0);
    assert!(empty.pair_inputs.is_none());
    assert!(empty.pair_topology_rows.is_none());
}

#[test]
fn inspector_pair_rows_follow_exact_assertion_hides_and_restore() {
    let mut canvas = Canvas::new();
    let first = canvas.open_member_as_new_node(None, "https://topology-a.test/");
    let second = canvas.open_member_as_new_node(None, "https://topology-b.test/");
    let a = canvas.graph.get_node_by_id(first).unwrap().0;
    let b = canvas.graph.get_node_by_id(second).unwrap().0;
    assert!(canvas.assert_relation_between_members(first, second, SemanticSubKind::Cites));
    assert!(canvas.assert_relation_between_members(first, second, SemanticSubKind::Quotes));
    let assertion_ids: Vec<_> = canvas
        .graph
        .projected_assertions()
        .filter_map(|(_, from, to, statement)| {
            ((from == a && to == b) || (from == b && to == a))
                .then_some(statement.statement_id.clone())
        })
        .collect();
    assert_eq!(assertion_ids.len(), 2);
    let target = DynamicsTarget::Pair { first, second };
    let all = canvas.inspect_dynamics(target).unwrap();
    assert_eq!(all.pair_controller_edges, Some(2));
    assert_eq!(all.pair_controller_edge_runs, Some(1));
    assert_eq!(all.pair_topology_rows.as_ref().unwrap().len(), 2);
    let all_ids: Vec<_> = all
        .pair_topology_rows
        .as_ref()
        .unwrap()
        .iter()
        .map(|row| match &row.origin {
            DynamicsTopologyOrigin::Assertion { statement_id } => statement_id.clone(),
            DynamicsTopologyOrigin::Traversal => panic!("assertion was reported as traversal"),
        })
        .collect();
    assert_eq!(all_ids, assertion_ids);

    assert!(canvas.set_hidden_assertion_ids([assertion_ids[0].clone()]));
    let hidden = canvas.inspect_dynamics(target).unwrap();
    assert_eq!(hidden.pair_controller_edges, Some(1));
    assert_eq!(hidden.pair_controller_edge_runs, Some(1));
    assert_eq!(hidden.pair_topology_rows.as_ref().unwrap().len(), 1);
    assert!(matches!(
        &hidden.pair_topology_rows.as_ref().unwrap()[0].origin,
        DynamicsTopologyOrigin::Assertion { statement_id } if statement_id == &assertion_ids[1]
    ));
    assert_eq!(hidden.pair_inputs.as_ref().unwrap().visible_rows, 1);
    assert_eq!(hidden.controller_edges, canvas.controller_topology().len());

    assert_eq!(canvas.show_all_assertions(), 1);
    let restored = canvas.inspect_dynamics(target).unwrap();
    assert_eq!(restored.pair_controller_edges, Some(2));
    assert_eq!(restored.pair_controller_edge_runs, Some(1));
    assert_eq!(restored.pair_topology_rows.as_ref().unwrap().len(), 2);
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
