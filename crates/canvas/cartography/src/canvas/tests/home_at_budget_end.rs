// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! F63, "Home at budget end too": when a settle budget ends with the bodies
//! still moving, anchored items still run the home step, so a dragged
//! anchored item finishes exactly at its position. G9's probe is the
//! receipt: before, the board's dragged anchored card stopped 58.191 units
//! short at its own stiffness and 2.326 short at the canvas's while another
//! card moved (`Code/testing/mere/grammar-g9/f63-probe-before.log`).

use super::*;
use seiche::Role;

fn card(id: &str, x: f32, y: f32) -> BoardItem {
    BoardItem {
        id: id.to_string(),
        slot: (x, y),
        site: "fixture".to_string(),
    }
}

/// The board: a seeded card dragged, then an anchored one, a pinned card
/// beside them. Returns how far the anchored card ends from its position
/// and how many frames its settle budget ran.
fn board_probe(stiffness: Option<f32>) -> (f32, u32) {
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
    for _ in 0..SETTLE_TICKS {
        board.tick();
    }
    for (id, to) in [("s", (-200.0, 90.0)), ("a", (300.0, 220.0))] {
        assert!(board.drag_start(id));
        assert!(board.drag_move(to.0, to.1));
        board.tick();
        assert!(board.drag_end());
    }
    let mut frames = 0;
    while board.is_settling() && frames < SETTLE_TICKS * 6 {
        board.tick();
        frames += 1;
    }
    for _ in 0..SETTLE_TICKS {
        board.tick();
    }
    assert!(!board.is_settling(), "the budget is spent");
    assert!(board.energy() > 1.0, "with the cards still moving");
    let a = board.position("a").unwrap();
    ((a.0 - 300.0).hypot(a.1), frames)
}

/// The canvas: four anchored items in a row, playing; one is dragged and
/// released while a second is dragged beside it. With `budget` the playing
/// canvas is then given a finite settle budget, which no public path does
/// today (a pick pauses and play runs unbounded), to reach the budget's end
/// with the bodies moving. Returns how far the dragged item ends from its
/// position and the frames run.
fn canvas_probe(budget: Option<u32>) -> (f32, u32) {
    let mut graph = Graph::new();
    let keys: Vec<_> = (0..4)
        .map(|i| {
            graph.add_node(
                format!("https://budget-{i}.example"),
                PortablePoint::new(0.0, 0.0),
            )
        })
        .collect();
    for pair in keys.windows(2) {
        graph.assert_relation(pair[0], pair[1], crate::canvas::build::hyperlink());
    }
    let mut canvas = Canvas::with_graph(graph);
    canvas.resize(1400, 900);
    canvas.set_layout_strategy(Some("test.budget".to_string()));
    let homes: Vec<_> = (0..4)
        .map(|i| PortablePoint::new(i as f32 * 160.0, 0.0))
        .collect();
    let slots: Vec<_> = keys.iter().copied().zip(homes.iter().copied()).collect();
    canvas.apply_strategy_positions(&slots);
    canvas.set_arrangement_role(Role::Anchored);
    canvas.fit_to_content();
    canvas.set_physics_paused(false);
    for _ in 0..30 {
        canvas.step_layout();
    }
    for (key, dy) in [(keys[2], 200.0), (keys[1], 160.0)] {
        let start = canvas.screen_position_of(key).unwrap();
        canvas.pointer_down(PointerButton::Left, start.0, start.1);
        for step in 1..=10 {
            canvas.cursor_moved(start.0, start.1 + dy * step as f32 / 10.0);
            canvas.step_layout();
        }
        canvas.pointer_up(PointerButton::Left, start.0, start.1 + dy);
    }
    if let Some(ticks) = budget {
        canvas.physics.halt();
        canvas.physics.settle(ticks);
    }
    let home = homes[1];
    let off = |canvas: &Canvas| {
        let at = canvas.view.position_of(keys[1]).unwrap();
        (at.x - home.x).hypot(at.y - home.y)
    };
    let mut frames = 0;
    let mut away_at_budget_end = None;
    while off(&canvas) > 0.0 && frames < SETTLE_TICKS * 20 {
        canvas.step_layout();
        frames += 1;
        if budget == Some(frames) {
            away_at_budget_end = Some(off(&canvas));
        }
    }
    if budget.is_some() {
        let away = away_at_budget_end.expect("the budget ran out before the item came home");
        assert!(
            away > 1.0,
            "the item was away when the budget ran out: {away}"
        );
    }
    (off(&canvas), frames)
}

/// The board, at its own anchored stiffness and at the canvas's, ends the
/// dragged anchored card exactly home although its budget runs out with
/// the cards still moving.
#[test]
fn the_board_finishes_home_at_the_budgets_end() {
    let offs: Vec<f32> = [None, Some(seiche::DEFAULT_ANCHOR_STIFFNESS)]
        .into_iter()
        .map(|stiffness| {
            let (off, frames) = board_probe(stiffness);
            println!(
                "F63 board at stiffness {stiffness:?}: the dragged anchored card ends {off:.3} \
                 from its position; the budget ran {frames} frames"
            );
            off
        })
        .collect();
    assert_eq!(offs, [0.0, 0.0], "exactly home at both stiffnesses");
}

/// The canvas: played, the item dragged while another moves comes exactly
/// home by the speed floor; on a finite budget that ends while things
/// move, it comes exactly home by F63's step.
#[test]
fn the_canvas_finishes_home_played_and_at_a_budgets_end() {
    let offs: Vec<f32> = [None, Some(30)]
        .into_iter()
        .map(|budget| {
            let (off, frames) = canvas_probe(budget);
            println!(
                "F63 canvas, budget {budget:?}: the dragged anchored item ends {off:.3} from its \
                 position after {frames} frames"
            );
            off
        })
        .collect();
    assert_eq!(
        offs,
        [0.0, 0.0],
        "exactly home, played and at a budget's end"
    );
}
