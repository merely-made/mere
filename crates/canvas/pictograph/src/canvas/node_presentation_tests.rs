// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
use super::*;
#[test]
fn rule_faces_preserve_source_pose_selection_pins_and_footprint() {
    let mut canvas = Canvas::with_sample_graph();
    let id = canvas.graph.nodes().next().unwrap().1.id;
    let key = canvas.graph.get_node_key_by_id(id).unwrap();
    canvas.select_member(id);
    canvas.pin_focused();
    let source = serde_json::to_value(canvas.graph.to_snapshot()).unwrap();
    let selected = canvas.selected_members();
    let pose: Vec<_> = canvas.view.positions().collect();
    let size = canvas.node_presentation_screen_size(id);
    let paused = canvas.physics_paused();
    let epoch = canvas.graph_view_epoch();
    canvas
        .set_node_rule_faces(epoch, HashMap::from([(id, sceno::Representation::Glyph)]))
        .unwrap();
    assert_eq!(
        canvas.node_representation_class(key),
        " gnode-representation-glyph"
    );
    assert_eq!(canvas.node_presentation_screen_size(id), size);
    assert_eq!(canvas.selected_members(), selected);
    assert_eq!(canvas.view.positions().collect::<Vec<_>>(), pose);
    assert_eq!(canvas.physics_paused(), paused);
    assert_eq!(canvas.arrangement_role_of(key), Role::Pinned);
    assert_eq!(
        serde_json::to_value(canvas.graph.to_snapshot()).unwrap(),
        source
    );
    canvas.set_node_rule_faces(epoch, HashMap::new()).unwrap();
    assert_eq!(
        canvas.node_representation_class(key),
        " gnode-representation-card"
    );
}
#[test]
fn unsupported_face_and_missing_member_refuse_without_partial_replacement() {
    let mut canvas = Canvas::with_sample_graph();
    let id = canvas.graph.nodes().next().unwrap().1.id;
    let epoch = canvas.graph_view_epoch();
    canvas
        .set_node_rule_faces(epoch, HashMap::from([(id, sceno::Representation::Glyph)]))
        .unwrap();
    assert!(
        canvas
            .set_node_rule_faces(
                epoch,
                HashMap::from([(id, sceno::Representation::Snapshot)])
            )
            .is_err()
    );
    assert_eq!(
        canvas.node_rule_representation(id),
        Some(&sceno::Representation::Glyph)
    );
    assert!(
        canvas
            .set_node_rule_faces(
                epoch,
                HashMap::from([(uuid::Uuid::from_u128(999), sceno::Representation::Card)])
            )
            .is_err()
    );
}
#[test]
fn graph_replacement_clears_faces_and_refuses_old_epoch() {
    let mut canvas = Canvas::with_sample_graph();
    let id = canvas.graph.nodes().next().unwrap().1.id;
    let epoch = canvas.graph_view_epoch();
    canvas
        .set_node_rule_faces(epoch, HashMap::from([(id, sceno::Representation::Glyph)]))
        .unwrap();
    canvas.set_graph(Graph::new());
    assert!(canvas.node_rule_representation(id).is_none());
    assert_ne!(canvas.graph_view_epoch(), epoch);
    assert!(canvas.set_node_rule_faces(epoch, HashMap::new()).is_err());
}
