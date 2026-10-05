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
