// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Permitted actions (dynamics grammar plan, G9): drag and pin are
//! chirograph `AdvertisedAction`s with the Curation effect on the canvas and
//! the board, each explanation says what the gesture then does and it does
//! it, and a withdrawn action refuses its gesture (the positive control),
//! ending a drag already under way.

use super::*;
use crate::canvas::{
    AdvertisedAction, ArrangementAction, DRAG_INTENT, IntentEffect, PIN_INTENT, PermittedActions,
};
use seiche::{Axes, Role};

fn intents(actions: &[AdvertisedAction]) -> Vec<&str> {
    actions
        .iter()
        .map(|action| action.intent.0.as_str())
        .collect()
}

fn explanation(actions: &[AdvertisedAction], action: ArrangementAction) -> &str {
    actions
        .iter()
        .find(|advertised| advertised.intent.0 == action.intent())
        .map(|advertised| advertised.explanation.as_str())
        .unwrap_or_else(|| panic!("{action:?} is advertised"))
}

fn distance(a: PortablePoint, b: PortablePoint) -> f32 {
    (a.x - b.x).hypot(a.y - b.y)
}

/// Four nodes in a row, an arrangement placing each, paused.
fn row() -> (Canvas, Vec<NodeKey>, Vec<PortablePoint>) {
    let mut graph = Graph::new();
    let keys: Vec<_> = (0..4)
        .map(|i| {
            graph.add_node(
                format!("https://row-{i}.example"),
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
    canvas.set_layout_strategy(Some("test.row".to_string()));
    let homes: Vec<_> = (0..4)
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

/// Press `key`, move the pointer by screen `(dx, dy)` in ten steps, a frame
/// each, and release. Returns where the item was at the last move.
fn gesture(canvas: &mut Canvas, key: NodeKey, dx: f32, dy: f32) -> PortablePoint {
    let start = canvas.screen_position_of(key).expect("on screen");
    canvas.pointer_down(PointerButton::Left, start.0, start.1);
    for step in 1..=10 {
        let t = step as f32 / 10.0;
        canvas.cursor_moved(start.0 + dx * t, start.1 + dy * t);
        canvas.frame(1400, 900);
    }
    let at = canvas.view.position_of(key).unwrap();
    canvas.pointer_up(PointerButton::Left, start.0 + dx, start.1 + dy);
    at
}

fn card(id: &str, x: f32, y: f32) -> BoardItem {
    BoardItem {
        id: id.to_string(),
        slot: (x, y),
        site: "fixture".to_string(),
    }
}

fn tick(board: &mut PhysicsBoard, frames: u32) {
    for _ in 0..frames {
        board.tick();
    }
}

/// S3's done-condition: every item's drag and pin are advertised with the
/// Curation effect, the label, schema and explanation filled. The board's
/// pinned card leaves drag out, its refusal (F47) being what it advertises.
#[test]
fn drag_and_pin_advertise_as_curation_on_both_surfaces() {
    let (canvas, keys, _) = row();
    for key in &keys {
        let actions = canvas.advertised_actions(member(&canvas, *key));
        assert_eq!(intents(&actions), [DRAG_INTENT, PIN_INTENT]);
        for (advertised, action) in actions.iter().zip(ArrangementAction::ALL) {
            assert_eq!(advertised.effect, IntentEffect::Curation);
            assert_eq!(advertised.label, action.label());
            assert_eq!(advertised.payload_schema, action.payload_schema());
            assert!(advertised.input_form.is_none());
            assert!(!advertised.explanation.is_empty());
        }
    }
    assert!(
        canvas.advertised_actions(uuid::Uuid::nil()).is_empty(),
        "an unknown member advertises nothing"
    );

    let mut board = PhysicsBoard::new();
    board.sync(vec![card("s", 0.0, 0.0), card("p", 120.0, 0.0)]);
    assert!(board.set_item_role("p", Some(Role::Pinned)));
    assert_eq!(
        intents(&board.advertised_actions("s")),
        [DRAG_INTENT, PIN_INTENT]
    );
    assert_eq!(
        intents(&board.advertised_actions("p")),
        [PIN_INTENT],
        "a pinned card advertises no drag"
    );
    for advertised in board.advertised_actions("s") {
        assert_eq!(advertised.effect, IntentEffect::Curation);
    }
    assert!(board.advertised_actions("missing").is_empty());
}

/// On the canvas, each role's drag says what a release does, and it does
/// it (F19, F47): a seeded drop becomes the item's position, an anchored
/// item jumps back with physics off, and a pinned item's pin moves to the
/// drop. An explicit pin advertises the pinned drag.
#[test]
fn the_canvas_advertises_what_each_roles_drag_does() {
    for (role, says) in [
        (Role::Seeded, "nothing returns it"),
        (Role::Anchored, "returns to its arrangement position"),
        (Role::Pinned, "pin"),
    ] {
        let (mut canvas, keys, homes) = row();
        let key = keys[1];
        assert!(canvas.set_member_role(member(&canvas, key), Some(role)));
        let actions = canvas.advertised_actions(member(&canvas, key));
        let text = explanation(&actions, ArrangementAction::Drag);
        assert!(text.contains(says), "{role:?}: {text}");
        let drop = gesture(&mut canvas, key, 0.0, 160.0);
        assert!(
            distance(drop, homes[1]) > 50.0,
            "{role:?}: the drag moved it"
        );
        for _ in 0..30 {
            canvas.frame(1400, 900);
        }
        let end = canvas.view.position_of(key).unwrap();
        let slot = canvas.arrangement_slot(key).unwrap();
        println!("{role:?}: dropped {drop:?}, ends {end:?}, position {slot:?}; \"{text}\"");
        match role {
            Role::Seeded => {
                assert_eq!(end, drop, "stays where dropped");
                assert_eq!(slot, drop, "the drop became its position");
            },
            Role::Anchored => {
                assert_eq!(end, homes[1], "jumped back");
                assert_eq!(slot, homes[1], "its position is unmoved");
            },
            Role::Pinned => {
                assert_eq!(end, drop, "held at the drop");
                assert_eq!(slot, drop, "the pin moved there");
                assert_eq!(canvas.arrangement_role_of(key), Role::Pinned);
            },
        }
    }

    let (mut canvas, keys, _) = row();
    canvas.select_only(keys[2]);
    assert!(canvas.pin_focused());
    let actions = canvas.advertised_actions(member(&canvas, keys[2]));
    assert!(explanation(&actions, ArrangementAction::Drag).contains("pin"));
}

/// The positive control on the canvas: with drag withdrawn the item
/// advertises pin alone, a pointer drag leaves it exactly where it was (the
/// press stays a click and selects it), and a nudge is refused; restored,
/// the same gesture moves it. With pin withdrawn, neither the explicit pin
/// nor the item's pinned role is taken, while other roles still are.
#[test]
fn a_withdrawn_action_refuses_its_gesture_on_the_canvas() {
    let (mut canvas, keys, homes) = row();
    let key = keys[1];
    canvas.set_permitted_actions(PermittedActions::without(ArrangementAction::Drag));
    assert_eq!(
        intents(&canvas.advertised_actions(member(&canvas, key))),
        [PIN_INTENT]
    );
    let at = gesture(&mut canvas, key, 0.0, 160.0);
    assert_eq!(at, homes[1], "a withdrawn drag does not move it");
    for _ in 0..30 {
        canvas.frame(1400, 900);
    }
    assert_eq!(canvas.view.position_of(key), Some(homes[1]));
    assert_eq!(canvas.focused_key(), Some(key), "the press was a click");
    assert!(!canvas.nudge_focused(30.0, 0.0), "the keyboard's drag too");
    assert_eq!(canvas.view.position_of(key), Some(homes[1]));

    canvas.set_permitted_actions(PermittedActions::all());
    let moved = gesture(&mut canvas, key, 0.0, 160.0);
    assert!(
        distance(moved, homes[1]) > 50.0,
        "restored, the same gesture moves it: {moved:?}"
    );

    let (mut canvas, keys, _) = row();
    let item = member(&canvas, keys[2]);
    canvas.set_permitted_actions(PermittedActions::without(ArrangementAction::Pin));
    assert_eq!(intents(&canvas.advertised_actions(item)), [DRAG_INTENT]);
    canvas.select_only(keys[2]);
    assert!(!canvas.pin_focused(), "the explicit pin is refused");
    assert!(
        !canvas.set_member_role(item, Some(Role::Pinned)),
        "and the pinned role"
    );
    assert_eq!(canvas.arrangement_role_of(keys[2]), Role::Seeded);
    assert!(
        canvas.set_member_role(item, Some(Role::Anchored)),
        "not a pin"
    );
    canvas.set_permitted_actions(PermittedActions::all());
    assert!(canvas.set_member_role(item, Some(Role::Pinned)));
    assert!(canvas.pin_focused());
    assert_eq!(canvas.arrangement_role_of(keys[2]), Role::Pinned);
}

/// A drag withdrawn while under way ends at once, as a release by its
/// role: on the canvas an anchored item jumps back with physics off and the
/// pointer no longer moves it; on the board a card pinned mid-drag returns
/// to its position, and a withdrawn drag ends until it is restored.
#[test]
fn a_drag_withdrawn_mid_drag_ends_as_a_release() {
    let (mut canvas, keys, homes) = row();
    let key = keys[1];
    assert!(canvas.set_member_role(member(&canvas, key), Some(Role::Anchored)));
    let start = canvas.screen_position_of(key).unwrap();
    canvas.pointer_down(PointerButton::Left, start.0, start.1);
    for step in 1..=5 {
        canvas.cursor_moved(start.0, start.1 + 30.0 * step as f32);
        canvas.frame(1400, 900);
    }
    assert!(
        distance(canvas.view.position_of(key).unwrap(), homes[1]) > 50.0,
        "the drag is under way"
    );
    canvas.set_permitted_actions(PermittedActions::without(ArrangementAction::Drag));
    assert_eq!(canvas.dragging_node(), None, "the drag ended");
    assert_eq!(canvas.view.position_of(key), Some(homes[1]), "jumped back");
    canvas.cursor_moved(start.0, start.1 + 300.0);
    canvas.frame(1400, 900);
    assert_eq!(
        canvas.view.position_of(key),
        Some(homes[1]),
        "the pointer no longer moves it"
    );
    canvas.pointer_up(PointerButton::Left, start.0, start.1 + 300.0);

    let mut board = PhysicsBoard::new();
    board.sync(vec![card("s", 0.0, 0.0), card("t", 300.0, 0.0)]);
    assert!(board.drag_start("s"));
    assert!(board.drag_move(-200.0, 90.0));
    board.tick();
    assert!(board.set_item_role("s", Some(Role::Pinned)));
    assert!(
        !board.drag_move(-260.0, 120.0),
        "pinned mid-drag, the drag ended"
    );
    tick(&mut board, 10);
    assert_eq!(
        board.position("s"),
        Some((0.0, 0.0)),
        "pinned at its position"
    );

    assert!(board.drag_start("t"));
    assert!(board.drag_move(300.0, 200.0));
    board.set_permitted_actions(PermittedActions::without(ArrangementAction::Drag));
    assert!(!board.drag_move(300.0, 260.0), "withdrawn, the drag ended");
    assert!(!board.drag_start("t"), "and none starts");
    board.set_permitted_actions(PermittedActions::all());
    assert!(board.drag_start("t"), "restored");
    assert!(board.drag_end());
}

/// On the board, each drag says what a release does, and it does it: a
/// pinned card refuses (F47), a seeded card is not returned, an anchored
/// card comes home, and on the practice board's encoded axes the drag names
/// the axes that return to their values, which they do exactly (F28). The
/// positive control: a seeded card whose drag is withdrawn refuses it, and
/// a withdrawn pin refuses the pinned role.
#[test]
fn the_board_advertises_what_its_drags_do() {
    let drag_text = |board: &PhysicsBoard, id: &str| {
        explanation(&board.advertised_actions(id), ArrangementAction::Drag).to_string()
    };
    // At the board's own gentle stiffness and at the canvas's. At the gentle
    // one the settle budget can end while the cards still move, so the
    // at-rest glide never starts and the card stops short of home, at the
    // canvas's too while another card moves
    // (`Code/testing/mere/grammar-g9/board-return-probe.log`); it still
    // returns most of the way.
    for stiffness in [None, Some(seiche::DEFAULT_ANCHOR_STIFFNESS)] {
        let mut board = PhysicsBoard::new();
        if let Some(stiffness) = stiffness {
            board.set_anchor_stiffness(stiffness);
        }
        board.sync(vec![
            card("s", 0.0, 0.0),
            card("a", 300.0, 0.0),
            card("p", 0.0, 300.0),
        ]);
        assert!(board.set_item_role("a", Some(Role::Anchored)));
        assert!(board.set_item_role("p", Some(Role::Pinned)));
        assert!(drag_text(&board, "s").contains("nothing returns it"));
        assert!(drag_text(&board, "a").contains("returns to its arrangement position"));
        assert!(!board.drag_start("p"), "pinned refuses");

        assert!(board.drag_start("s"));
        assert!(board.drag_move(-200.0, 90.0));
        board.tick();
        assert!(board.drag_end());
        assert!(board.drag_start("a"));
        assert!(board.drag_move(300.0, 220.0));
        board.tick();
        assert!(board.drag_end());
        let mut frames = 0;
        while board.anchored_home_count() == 0 && frames < SETTLE_TICKS * 6 {
            board.tick();
            frames += 1;
        }
        let (s, a) = (board.position("s").unwrap(), board.position("a").unwrap());
        let off = (a.0 - 300.0).hypot(a.1);
        println!(
            "board at stiffness {}: seeded ends {s:?} after a drop at (-200, 90); anchored ends \
             {a:?}, {off:.1} from its position after a drop 220 off, {} home after {frames} frames",
            board.anchor_stiffness(),
            board.anchored_home_count(),
        );
        assert!(
            (s.0 + 200.0).hypot(s.1 - 90.0) < 60.0,
            "nothing returned the seeded card: {s:?}"
        );
        assert!(
            off < 110.0,
            "the anchored card returned most of the way: {a:?}"
        );
        if stiffness.is_some() {
            assert!(off < 5.0, "stiffer, nearly home: {a:?}");
        }
        assert_eq!(board.position("p"), Some((0.0, 300.0)), "pinned held");
    }

    for (axes, says) in [
        (Axes { x: true, y: false }, "x returns to its value"),
        (Axes::BOTH, "x and y return to their values"),
    ] {
        let mut practice = PhysicsBoard::new();
        practice.set_encoded_axes(axes);
        practice.sync(vec![card("c", 100.0, 40.0), card("d", 400.0, 300.0)]);
        tick(&mut practice, SETTLE_TICKS);
        let text = drag_text(&practice, "c");
        assert!(text.contains(says), "{axes:?}: {text}");
        assert!(practice.drag_start("c"));
        assert!(practice.drag_move(260.0, 200.0));
        practice.tick();
        assert!(practice.drag_end());
        practice.tick();
        let at = practice.position("c").unwrap();
        assert_eq!(at.0, 100.0, "{axes:?}: x back to its value");
        if axes.y {
            assert_eq!(at.1, 40.0, "y back to its value");
        } else {
            assert!((at.1 - 200.0).abs() < 1.0, "y left at the drop: {at:?}");
        }
    }

    let mut control = PhysicsBoard::new();
    control.sync(vec![card("s", 0.0, 0.0)]);
    control.set_permitted_actions(PermittedActions::without(ArrangementAction::Drag));
    assert_eq!(intents(&control.advertised_actions("s")), [PIN_INTENT]);
    assert!(
        !control.drag_start("s"),
        "a seeded card whose drag is withdrawn"
    );
    control.set_permitted_actions(PermittedActions::without(ArrangementAction::Pin));
    assert!(control.drag_start("s"), "drag restored");
    assert!(control.drag_end());
    assert!(
        !control.set_item_role("s", Some(Role::Pinned)),
        "pin withdrawn"
    );
    assert_eq!(control.role_of("s"), Some(Role::Seeded));
    assert!(
        control.set_item_role("s", Some(Role::Anchored)),
        "not a pin"
    );
}
