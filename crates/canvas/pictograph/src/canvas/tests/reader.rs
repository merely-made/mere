// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a screen reader is told of the canvas, and the keyboard move
//! (dynamics grammar plan, G9: F67).

use super::*;
use crate::canvas::{ArrangementAction, DRAG_INTENT, PIN_INTENT, PermittedActions};
use seiche::Role;

/// `n` nodes in a row 160 apart with an arrangement placing each, paused.
fn row(n: usize) -> (Canvas, Vec<NodeKey>, Vec<PortablePoint>) {
    let mut graph = Graph::new();
    let keys: Vec<_> = (0..n)
        .map(|i| {
            graph.add_node(
                format!("https://reader-{i}.example"),
                PortablePoint::new(0.0, 0.0),
            )
        })
        .collect();
    for pair in keys.windows(2) {
        graph.assert_relation(pair[0], pair[1], crate::canvas::build::hyperlink());
    }
    let mut canvas = Canvas::with_graph(graph);
    canvas.resize(1400, 900);
    canvas.set_physics_paused(true);
    canvas.set_layout_strategy(Some("test.reader".to_string()));
    let homes: Vec<_> = (0..n)
        .map(|i| PortablePoint::new(i as f32 * 160.0, 0.0))
        .collect();
    let slots: Vec<_> = keys.iter().copied().zip(homes.iter().copied()).collect();
    canvas.apply_strategy_positions(&slots);
    canvas.fit_to_content();
    (canvas, keys, homes)
}

fn member(canvas: &Canvas, key: NodeKey) -> uuid::Uuid {
    canvas.graph().get_node(key).unwrap().id
}

#[test]
fn child_lifetimes_are_authored_before_reads_and_withdrawn_between_reads() {
    let (mut canvas, keys, _) = row(2);
    let id = member(&canvas, keys[0]);
    // Construction has assigned every live member a key, without a reader.
    let original = canvas.described_keys.by_member[&id];
    let untouched = canvas.described_keys.by_member[&member(&canvas, keys[1])];
    let before = canvas.described_keys.by_member.clone();
    let description = canvas.describe_items(DESCRIBED_ITEMS);
    canvas.describe_items(1);
    canvas.describe_items(DESCRIBED_ITEMS);
    assert_eq!(
        canvas.described_keys.by_member, before,
        "reads never author routes"
    );
    assert!(description.items.iter().any(|item| item.key == original));

    assert!(canvas.ingest_graph(|graph| graph.remove_node(keys[0])));
    assert_eq!(canvas.member_of_described(original), None);
    assert!(canvas.ingest_graph(|graph| {
        graph.add_node_with_id(id, "https://restored.example".into(), PortablePoint::zero());
        true
    }));
    // No description occurred during removal/restoration of the SAME UUID.
    let restored = canvas.described_keys.by_member[&id];
    assert_ne!(restored, original);
    assert_eq!(canvas.member_of_described(original), None);
    assert_eq!(canvas.member_of_described(restored), Some(id));
    assert_eq!(
        canvas.described_keys.by_member[&member(&canvas, keys[1])],
        untouched
    );

    let same_graph = canvas.graph().clone();
    assert!(canvas.refresh_same_topology_graph(&same_graph).is_ok());
    assert_eq!(
        canvas.described_keys.by_member[&id], restored,
        "metadata refresh keeps the life"
    );
    canvas.set_graph(same_graph.clone());
    assert_eq!(
        canvas.member_of_described(restored),
        None,
        "graph switch ends the life"
    );
    assert_ne!(canvas.described_keys.by_member[&id], restored);
    let other = Canvas::with_graph(same_graph);
    assert_ne!(
        other.described_keys.by_member[&id],
        canvas.described_keys.by_member[&id]
    );
}

/// Every item on screen is described, named by its caption's label, placed
/// where it is drawn, with drag and pin; the slot says "N of M shown".
/// Panned off, an item leaves the description; the focused one stays.
#[test]
fn the_canvas_describes_its_items_on_screen_with_their_actions() {
    let (mut canvas, keys, _) = row(4);
    let description = canvas.describe_items(DESCRIBED_ITEMS);
    assert_eq!(description.total, 4);
    assert_eq!(description.items.len(), 4);
    assert_eq!(description.name(), "4 of 4 shown");
    for (item, key) in description.items.iter().zip(&keys) {
        assert_eq!(item.member, member(&canvas, *key));
        assert_eq!(item.name, canvas.graph().node_display_label(*key));
        assert_eq!(canvas.member_of_described(item.key), Some(item.member));
        let intents: Vec<_> = item.actions.iter().map(|a| a.intent.0.as_str()).collect();
        assert_eq!(intents, [DRAG_INTENT, PIN_INTENT]);
        let (x, y) = canvas.screen_position_of(*key).unwrap();
        let [rx, ry, rw, rh] = item.rect;
        assert!((rx + rw / 2.0 - x).abs() < 1e-3 && (ry + rh / 2.0 - y).abs() < 1e-3);
        assert!(rw > 0.0 && rh > 0.0);
    }

    // Pan the first item off screen: it leaves the description, unless it
    // is the focused one.
    let (x, _) = canvas.screen_position_of(keys[0]).unwrap();
    canvas.camera.offset.0 -= x + 200.0;
    let panned = canvas.describe_items(DESCRIBED_ITEMS);
    assert_eq!(panned.total, 4, "the total counts every item");
    assert!(
        panned
            .items
            .iter()
            .all(|item| item.member != member(&canvas, keys[0]))
    );
    canvas.select_only(keys[0]);
    let focused = canvas.describe_items(DESCRIBED_ITEMS);
    let first = focused
        .items
        .iter()
        .find(|item| item.member == member(&canvas, keys[0]))
        .expect("the focused item is always described");
    assert!(first.focused);
}

/// The cap: at most `cap` items, in graph order, the focused one always
/// among them, taking the last place when the cap would leave it out.
#[test]
fn the_description_is_capped_and_keeps_the_focused_item() {
    let (mut canvas, keys, _) = row(12);
    let capped = canvas.describe_items(5);
    assert_eq!(capped.items.len(), 5);
    assert_eq!(capped.total, 12);
    assert_eq!(capped.name(), "5 of 12 shown");
    let members: Vec<_> = capped.items.iter().map(|item| item.member).collect();
    let first_five: Vec<_> = keys[..5].iter().map(|k| member(&canvas, *k)).collect();
    assert_eq!(members, first_five, "graph order");
    canvas.select_only(keys[9]);
    let capped = canvas.describe_items(5);
    assert_eq!(capped.items.len(), 5);
    let members: Vec<_> = capped.items.iter().map(|item| item.member).collect();
    let mut expected: Vec<_> = keys[..4].iter().map(|k| member(&canvas, *k)).collect();
    expected.push(member(&canvas, keys[9]));
    assert_eq!(members, expected, "the focused item takes the last place");
}

/// A pin by member holds the item where it is, as the explicit pin does,
/// and is refused with pin withdrawn (the control).
#[test]
fn a_pin_by_member_holds_the_item_and_is_refused_when_withdrawn() {
    let (mut canvas, keys, _) = row(4);
    let item = member(&canvas, keys[1]);
    canvas.set_permitted_actions(PermittedActions::without(ArrangementAction::Pin));
    assert!(!canvas.pin_member(item), "withdrawn, refused");
    assert_eq!(canvas.arrangement_role_of(keys[1]), Role::Seeded);
    canvas.set_permitted_actions(PermittedActions::all());
    assert!(canvas.pin_member(item));
    assert_eq!(canvas.arrangement_role_of(keys[1]), Role::Pinned);
}

/// The keyboard move: started on an item that advertises drag, the arrows
/// nudge it by screen px, Enter drops it by its role (a seeded drop becomes
/// its position; an anchored item jumps back with physics off), and Escape
/// puts it back where the move began. Refused with drag withdrawn (the
/// control), and a withdrawal under way ends it as a release.
#[test]
fn a_keyboard_move_nudges_drops_by_role_and_escapes_back() {
    let (mut canvas, keys, homes) = row(4);
    let zoom = canvas.camera.zoom;
    let seeded = member(&canvas, keys[1]);

    canvas.set_permitted_actions(PermittedActions::without(ArrangementAction::Drag));
    assert!(!canvas.begin_key_move(seeded), "drag withdrawn, refused");
    canvas.set_permitted_actions(PermittedActions::all());

    assert!(canvas.begin_key_move(seeded));
    assert_eq!(canvas.key_moving(), Some(seeded));
    assert!(
        !canvas.begin_key_move(member(&canvas, keys[2])),
        "one at a time"
    );
    for _ in 0..3 {
        assert!(canvas.key_move_by(42.0, 0.0));
        canvas.frame(1400, 900);
    }
    let moved = canvas.view.position_of(keys[1]).unwrap();
    let expected = homes[1].x + 3.0 * 42.0 / zoom;
    assert!(
        (moved.x - expected).abs() < 1e-3,
        "{moved:?} against {expected}"
    );
    assert!(canvas.end_key_move(true), "Enter drops it");
    assert_eq!(canvas.key_moving(), None);
    for _ in 0..10 {
        canvas.frame(1400, 900);
    }
    assert_eq!(
        canvas.view.position_of(keys[1]),
        Some(moved),
        "seeded stays"
    );
    assert_eq!(
        canvas.arrangement_slot(keys[1]),
        Some(moved),
        "its new position"
    );

    // Escape puts it back where this move began.
    assert!(canvas.begin_key_move(seeded));
    assert!(canvas.key_move_by(0.0, 84.0));
    assert!(canvas.end_key_move(false), "Escape");
    for _ in 0..10 {
        canvas.frame(1400, 900);
    }
    assert_eq!(canvas.view.position_of(keys[1]), Some(moved), "back");

    // An anchored item, dropped with physics off, jumps back home.
    let anchored = member(&canvas, keys[2]);
    assert!(canvas.set_member_role(anchored, Some(Role::Anchored)));
    assert!(canvas.begin_key_move(anchored));
    assert!(canvas.key_move_by(0.0, 126.0));
    assert!(canvas.end_key_move(true));
    assert_eq!(
        canvas.view.position_of(keys[2]),
        Some(homes[2]),
        "jumped back"
    );

    // Withdrawn under way, the move ends as a release.
    assert!(canvas.begin_key_move(seeded));
    canvas.set_permitted_actions(PermittedActions::without(ArrangementAction::Drag));
    assert_eq!(canvas.key_moving(), None, "the move ended");
    assert!(!canvas.key_move_by(42.0, 0.0));
}

fn card(id: &str, x: f32, y: f32) -> BoardItem {
    BoardItem {
        id: id.to_string(),
        slot: (x, y),
        site: "fixture".to_string(),
    }
}

/// The board's keyboard move: a drag the arrows steer, refused for a pinned
/// card, Enter dropping it, Escape putting it back.
#[test]
fn the_boards_keyboard_move_nudges_drops_and_escapes_back() {
    let mut board = PhysicsBoard::new();
    board.sync(vec![card("s", 0.0, 0.0), card("p", 300.0, 0.0)]);
    assert!(board.set_item_role("p", Some(Role::Pinned)));
    assert!(!board.begin_key_move("p"), "a pinned card refuses");
    assert!(board.begin_key_move("s"));
    assert_eq!(board.key_moving(), Some("s"));
    assert!(board.key_move_by(40.0, 0.0));
    assert!(board.key_move_by(40.0, 0.0));
    board.tick();
    let held = board.position("s").unwrap();
    assert!((held.0 - 80.0).abs() < 1.0, "steered: {held:?}");
    assert!(board.end_key_move(false), "Escape");
    board.tick();
    let back = board.position("s").unwrap();
    assert!(back.0.abs() < 1.0, "back where it began: {back:?}");
    assert_eq!(board.key_moving(), None);
    assert!(board.begin_key_move("s"));
    assert!(board.key_move_by(0.0, 60.0));
    assert!(board.end_key_move(true), "Enter");
    assert!(!board.key_move_by(0.0, 60.0), "ended");
}

#[test]
fn same_call_recreation_retires_only_that_members_actions_and_ordinary_edits_keep_keys() {
    let (mut canvas, keys, _) = row(3);
    let ids: Vec<_> = keys.iter().map(|key| member(&canvas, *key)).collect();
    let old = canvas.described_keys.by_member.clone();
    assert!(canvas.ingest_graph(|graph| {
        graph.set_node_title(keys[1], "Ordinary title edit".into());
        graph.assert_relation(keys[1], keys[2], crate::canvas::build::hyperlink());
        assert!(graph.remove_node(keys[0]));
        graph.add_node_with_id(
            ids[0],
            "https://same-call.example".into(),
            PortablePoint::zero(),
        );
        true
    }));
    let recreated = canvas.described_keys.by_member[&ids[0]];
    assert_ne!(recreated, old[&ids[0]]);
    assert_eq!(canvas.member_of_described(old[&ids[0]]), None);
    for id in &ids[1..] {
        assert_eq!(canvas.described_keys.by_member[id], old[id]);
    }
    assert!(canvas.ingest_graph(|graph| {
        graph.add_node_with_id(
            ids[0],
            "https://ordinary-upsert.example".into(),
            PortablePoint::zero(),
        );
        true
    }));
    assert_eq!(
        canvas.described_keys.by_member[&ids[0]], recreated,
        "upserts keep live membership"
    );
    let mut owner = canvas.graph().clone();
    owner.set_node_title(keys[1], "Accepted owner metadata".into());
    assert!(canvas.refresh_same_topology_graph(&owner).is_ok());
    assert_eq!(canvas.described_keys.by_member[&ids[0]], recreated);
    for id in &ids[1..] {
        assert_eq!(canvas.described_keys.by_member[id], old[id]);
    }
    let stable = canvas.described_keys.by_member.clone();
    for _ in 0..3 {
        canvas.describe_items(DESCRIBED_ITEMS);
    }
    assert_eq!(
        canvas.described_keys.by_member, stable,
        "reads cannot mint lives"
    );
}

#[test]
fn false_mutation_report_cannot_hide_same_call_recreation() {
    let (mut canvas, keys, _) = row(2);
    let id = member(&canvas, keys[0]);
    let old = canvas.described_keys.by_member[&id];
    assert!(
        canvas.ingest_graph(|graph| {
            assert!(graph.remove_node(keys[0]));
            graph.add_node_with_id(
                id,
                "https://false-report.example".into(),
                PortablePoint::zero(),
            );
            false
        }),
        "kernel mutation wins over the callback's report"
    );
    assert_eq!(canvas.member_of_described(old), None);
    assert_ne!(canvas.described_keys.by_member[&id], old);
    assert!(!canvas.ingest_graph(|_| false));
}

#[test]
fn same_topology_refresh_refuses_recreated_members_and_structural_ingest_receives_their_lives() {
    let (mut canvas, keys, _) = row(2);
    let id = member(&canvas, keys[0]);
    let other = member(&canvas, keys[1]);
    let old = canvas.described_keys.by_member[&id];
    let untouched = canvas.described_keys.by_member[&other];
    let mut owner = canvas.graph().clone();
    assert!(owner.remove_node(keys[0]));
    let reused = owner.add_node_with_id(
        id,
        "https://owner-recreation.example".into(),
        PortablePoint::zero(),
    );
    assert_eq!(reused, keys[0]);
    assert!(
        canvas.refresh_same_topology_graph(&owner).is_err(),
        "same UUID/slot cannot claim metadata-only continuity"
    );
    assert_eq!(
        canvas.described_keys.by_member[&id], old,
        "refusal is atomic"
    );
    assert!(canvas.ingest_graph(|graph| {
        *graph = owner;
        true
    }));
    assert_eq!(canvas.member_of_described(old), None);
    assert_ne!(canvas.described_keys.by_member[&id], old);
    assert_eq!(canvas.described_keys.by_member[&other], untouched);
}
