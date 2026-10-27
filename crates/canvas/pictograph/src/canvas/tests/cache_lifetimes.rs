// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Runtime caches must not follow recycled graph slots, including the same UUID.
use super::*;

fn fixture() -> (Canvas, NodeKey, NodeKey, uuid::Uuid, uuid::Uuid) {
    let mut graph = Graph::new();
    let a = uuid::Uuid::from_u128(1001);
    let b = uuid::Uuid::from_u128(1002);
    let ak = graph.add_node_with_id(a, "https://a.example".into(), Default::default());
    let bk = graph.add_node_with_id(b, "https://b.example".into(), Default::default());
    (Canvas::with_graph(graph), ak, bk, a, b)
}

fn recreate(canvas: &mut Canvas, key: NodeKey, id: uuid::Uuid) {
    assert!(canvas.ingest_graph(|graph| {
        assert!(graph.remove_node(key));
        assert_eq!(
            graph.add_node_with_id(id, "https://replacement.example".into(), Default::default()),
            key,
            "exercise graph slot reuse"
        );
        false // kernel membership, not this report, governs retirement
    }));
}

#[test]
fn same_uuid_recreation_retires_caches_without_disturbing_other_members() {
    let (mut c, a, b, aid, bid) = fixture();
    for (key, id) in [(a, aid), (b, bid)] {
        c.selected.insert(key);
        c.pinned_nodes.insert(key);
        c.node_states.insert(key, NodeState::Open);
        c.node_shapes.insert(key, NodeShape::Circle);
        c.mosaic.scale.insert(key, 2.0);
        c.roles.table.items.insert(key, seiche::Role::Anchored);
        c.set_node_sprite(id, "data:image/png;base64,AAAA".into());
        c.set_node_material(
            id,
            NodeMaterial {
                restitution: 0.6,
                ..Default::default()
            },
        );
    }
    let positions = vec![
        (a, PortablePoint::new(90., 80.)),
        (b, PortablePoint::new(40., 50.)),
    ];
    c.strategy_positions = Some(positions.clone());
    c.paused_positions = Some(positions);
    c.scope = Some(vec![a, b]);
    let survivor_action = c.described_keys.by_member[&bid];
    recreate(&mut c, a, aid);
    assert!(
        !c.selected.contains(&a),
        "selection followed the replacement"
    );
    assert!(
        !c.pinned_nodes.contains(&a),
        "explicit pin followed the replacement"
    );
    assert!(!c.node_states.contains_key(&a));
    assert!(!c.node_shapes.contains_key(&a));
    assert!(!c.mosaic.scale.contains_key(&a));
    assert_eq!(c.member_role(aid), None);
    assert_eq!(c.node_sprite(a), None);
    assert_eq!(c.node_face(a), Face::Derived);
    assert_eq!(c.node_material(a), NodeMaterial::default());
    assert_eq!(c.scope, Some(vec![b]));
    for positions in [&c.strategy_positions, &c.paused_positions]
        .into_iter()
        .flatten()
    {
        assert_eq!(positions, &vec![(b, PortablePoint::new(40., 50.))]);
    }
    assert!(c.selected.contains(&b));
    assert!(c.pinned_nodes.contains(&b));
    assert_eq!(c.node_states[&b], NodeState::Open);
    assert_eq!(c.node_shapes[&b], NodeShape::Circle);
    assert_eq!(c.mosaic.scale[&b], 2.0);
    assert_eq!(c.member_role(bid), Some(seiche::Role::Anchored));
    assert!(c.node_sprite(b).is_some());
    assert_eq!(c.node_material(b).restitution, 0.6);
    assert_eq!(c.described_keys.by_member[&bid], survivor_action);
    let geometry = c.cartography_geometry();
    assert!(!geometry.sprite_iter().any(|(id, _)| id == aid));
    assert!(!geometry.material_iter().any(|(id, _)| id == aid));
}

#[test]
fn ordinary_edits_keep_cache_identity_and_active_keyboard_moves() {
    let (mut c, a, b, aid, _) = fixture();
    c.selected.insert(a);
    c.scope_to_members([aid]);
    c.node_shapes.insert(a, NodeShape::Circle);
    c.set_node_face(aid, Face::Bare);
    c.mosaic.scale.insert(a, 1.5);
    c.roles.table.items.insert(a, seiche::Role::Anchored);
    assert!(c.begin_key_move(aid));
    let action = c.described_keys.by_member[&aid];
    let mut candidate = c.graph.clone();
    candidate.set_node_title(a, "Edited".into());
    c.refresh_same_topology_graph(&candidate).unwrap();
    assert!(c.selected.contains(&a));
    assert_eq!(c.node_shapes[&a], NodeShape::Circle);
    assert_eq!(c.node_face(a), Face::Bare);
    assert_eq!(c.mosaic.scale[&a], 1.5);
    assert_eq!(c.member_role(aid), Some(seiche::Role::Anchored));
    assert_eq!(c.key_moving(), Some(aid));
    assert_eq!(c.described_keys.by_member[&aid], action);
    assert!(c.key_move_by(10., 5.));
    // A different member's replacement must leave this ongoing move alone.
    let bid = c.graph.get_node(b).unwrap().id;
    recreate(&mut c, b, bid);
    assert_eq!(c.key_moving(), Some(aid));
    assert_eq!(c.described_keys.by_member[&aid], action);
}

#[test]
fn recreated_member_cannot_continue_keyboard_or_pointer_move() {
    for keyboard in [true, false] {
        let (mut c, a, _, aid, _) = fixture();
        if keyboard {
            assert!(c.begin_key_move(aid));
            assert!(c.key_move_by(30., 20.));
        } else {
            c.drag = Some(Drag {
                node: a,
                press: (0., 0.),
                moved: true,
            });
            c.physics.set_dragging(true);
            c.physics.pin(a, Point2D::new(30., 20.));
        }
        recreate(&mut c, a, aid);
        assert_eq!(
            c.key_moving(),
            None,
            "keyboard move reached the replacement"
        );
        assert_eq!(
            c.dragging_node(),
            None,
            "pointer drag reached the replacement"
        );
        assert_eq!(c.key_move_offset(), None);
        assert!(!c.key_move_by(10., 10.));
        assert!(
            !c.end_key_move(false),
            "old Escape rolled back a new member"
        );
    }
}

#[test]
fn retiring_forme_hold_cannot_release_replacement_pin() {
    let (mut c, a, _, aid, _) = fixture();
    c.set_forme_region(Some(FormeRegion {
        id: uuid::Uuid::from_u128(2001),
        bounds: [0., 0., 200., 200.],
        cells: vec![FormeCell {
            member: aid,
            bounds: [0., 0., 1., 1.],
        }],
        locked: true,
        visible: true,
    }))
    .unwrap();
    assert!(c.forme_held.contains(&a));
    // A member with a different identity takes the same slot.
    let replacement = uuid::Uuid::from_u128(3001);
    recreate(&mut c, a, replacement);
    assert!(
        !c.forme_held.contains(&a),
        "departed Forme hold retained its slot"
    );
    assert!(c.pin_member(replacement));
    let at = c.view.position_of(a).unwrap();
    c.set_forme_region(None).unwrap();
    assert!(c.pinned_nodes.contains(&a));
    c.physics.refresh(&mut c.view);
    assert_eq!(c.view.position_of(a), Some(at));
}

#[test]
fn strategy_stamp_changes_on_recreation_but_not_title_edits() {
    let (mut c, a, _, aid, _) = fixture();
    let before = c.strategy_graph_stamp();
    c.set_node_title_for(aid, "Changed caption".into());
    assert_eq!(c.strategy_graph_stamp(), before);
    recreate(&mut c, a, aid);
    assert_ne!(c.strategy_graph_stamp(), before);
}

#[test]
fn explicit_graph_switch_ends_cache_lives_even_for_a_clone() {
    let (mut c, a, b, aid, _) = fixture();
    let graph = c.graph.clone();
    c.scope_to_members([aid]);
    c.roles.table.items.insert(a, seiche::Role::Pinned);
    c.content_affinity = Some(vec![(a, b, 0.9)]);
    c.installed_affinity_revision = Some(c.graph.structural_input_key());
    c.affinity_force_installed = true;
    c.selected.insert(a);
    let old_action = c.described_keys.by_member[&aid];
    c.set_graph(graph);
    assert!(c.roles.table.items.is_empty());
    assert!(c.content_affinity.is_none());
    assert!(c.installed_affinity_revision.is_none());
    assert!(!c.affinity_force_installed);
    assert!(c.selected.is_empty());
    assert_ne!(c.described_keys.by_member[&aid], old_action);
    assert_eq!(c.view.position_of(a), Some(Point2D::zero()));
    assert_eq!(c.scope_members(), Some(vec![aid]));
}

#[test]
fn transient_scope_cannot_alias_a_different_graphs_recycled_slot() {
    let (mut c, _, _, aid, _) = fixture();
    c.scope_to_members([aid]);
    let mut other = Graph::new();
    other.add_node_with_id(
        uuid::Uuid::from_u128(4001),
        "https://other.example".into(),
        Default::default(),
    );
    c.set_graph(other);
    assert_eq!(c.scope_members(), Some(vec![]));
}

#[test]
fn authored_exact_scope_and_forme_roster_reacquire_a_new_live_member() {
    let (mut c, a, _, aid, _) = fixture();
    c.scope_to_exact_members([aid]);
    c.set_forme_region(Some(FormeRegion {
        id: uuid::Uuid::from_u128(2001),
        bounds: [0., 0., 200., 200.],
        cells: vec![FormeCell {
            member: aid,
            bounds: [0., 0., 1., 1.],
        }],
        locked: true,
        visible: true,
    }))
    .unwrap();
    recreate(&mut c, a, aid);
    assert_eq!(
        c.scope,
        Some(vec![a]),
        "authored UUID roster still names this member"
    );
    assert!(!c.forme_held.contains(&a), "old live hold retired");
    assert_eq!(c.forme_region.as_ref().unwrap().cells[0].member, aid);
    c.apply_forme_region();
    assert!(
        c.forme_held.contains(&a),
        "current authored roster reacquires the new life"
    );
    assert_eq!(c.view.position_of(a), Some(Point2D::new(100., 100.)));
}
