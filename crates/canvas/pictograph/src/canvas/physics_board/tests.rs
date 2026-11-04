// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::canvas::physics_catalog::PhysicsLaw;

fn item(id: &str, x: f32, y: f32) -> BoardItem {
    BoardItem {
        id: id.to_string(),
        slot: (x, y),
        site: "fixture".to_string(),
    }
}

fn settle(board: &mut PhysicsBoard) {
    for _ in 0..SETTLE_TICKS {
        board.tick();
    }
}

/// Anchored at the canvas's stiffness, items sit at their slots; a later
/// item spawns at its slot and the earlier ones are not re-seeded. Anchored
/// at the board's gentle stiffness the boundary force bends the shape a
/// little (a body 300 units out drifts about thirty toward the origin), and
/// seeded, the default, it bends further (the control).
#[test]
fn items_hold_their_slots_and_a_new_one_joins_without_reseeding() {
    let pair = || vec![item("a", 0.0, 0.0), item("b", 300.0, 0.0)];
    let mut gentle = PhysicsBoard::new();
    gentle.set_roles(RoleTable::uniform(Role::Anchored));
    gentle.sync(pair());
    settle(&mut gentle);
    let b = gentle.position("b").unwrap();
    assert!(
        (b.0 - 300.0).abs() < 60.0 && b.0 > 200.0,
        "the gentle anchor keeps the shape within a fifth: {b:?}"
    );
    let mut seeded = PhysicsBoard::new();
    seeded.sync(pair());
    settle(&mut seeded);
    let free = seeded.position("b").unwrap();
    assert!(free.0 < b.0 - 1.0, "seeded drifts further: {free:?} {b:?}");

    let mut board = PhysicsBoard::new();
    board.set_roles(RoleTable::uniform(Role::Anchored));
    board.set_anchor_stiffness(DEFAULT_ANCHOR_STIFFNESS);
    assert_eq!(
        board.sync(vec![item("a", 0.0, 0.0), item("b", 300.0, 0.0)]),
        2
    );
    for _ in 0..SETTLE_TICKS {
        board.tick();
    }
    let a = board.position("a").unwrap();
    let b = board.position("b").unwrap();
    assert!(
        a.0.abs() < 20.0 && (b.0 - 300.0).abs() < 20.0,
        "at the slots: {a:?} {b:?}"
    );
    // Nudge a off its slot by moving its slot, then add c: a keeps its
    // simulated position (not teleported to the new slot), c spawns at its own.
    assert_eq!(
        board.sync(vec![
            item("a", 60.0, 0.0),
            item("b", 300.0, 0.0),
            item("c", 150.0, 200.0)
        ]),
        1
    );
    let a_after = board.position("a").unwrap();
    assert!(
        (a_after.0 - a.0).abs() < 1.0,
        "a was not re-seeded: {a_after:?}"
    );
    let c = board.position("c").unwrap();
    assert!(
        (c.0 - 150.0).abs() < 1.0 && (c.1 - 200.0).abs() < 1.0,
        "c at its slot: {c:?}"
    );
    assert_eq!(board.len(), 3);
    // A departed item drops.
    board.sync(vec![item("b", 300.0, 0.0)]);
    assert!(board.position("a").is_none());
    assert_eq!(board.len(), 1);
}

#[test]
fn drag_pins_one_item_and_release_returns_toward_its_slot() {
    let mut board = PhysicsBoard::new();
    board.set_roles(RoleTable::uniform(Role::Anchored));
    board.set_anchor_stiffness(DEFAULT_ANCHOR_STIFFNESS);
    board.sync(vec![item("a", 0.0, 0.0), item("b", 220.0, 0.0)]);
    for _ in 0..SETTLE_TICKS {
        board.tick();
    }

    let start = board.position("a").unwrap();
    assert!(board.drag_start("a"));
    assert!(!board.drag_start("b"));
    assert!(board.drag_move(140.0, 70.0));
    assert!(board.tick());
    let held = board.position("a").unwrap();
    assert!((held.0 - 140.0).abs() < 1.0 && (held.1 - 70.0).abs() < 1.0);
    assert!(board.is_settling());

    assert!(board.drag_end());
    assert!(!board.drag_move(20.0, 20.0));
    for _ in 0..SETTLE_TICKS {
        board.tick();
    }
    let released = board.position("a").unwrap();
    assert!(
        (released.0 - 0.0).abs() < (held.0 - 0.0).abs(),
        "release should let the item return toward its slot: start={start:?}, held={held:?}, released={released:?}"
    );
}

#[test]
fn halt_cancels_drag_and_sync_removal_clears_it() {
    let mut board = PhysicsBoard::new();
    board.sync(vec![item("a", 0.0, 0.0), item("b", 120.0, 0.0)]);
    assert!(board.drag_start("a"));
    assert!(!board.drag_move(f32::NAN, 4.0));
    board.halt();
    assert!(!board.drag_move(20.0, 20.0));
    assert!(!board.is_settling());

    assert!(board.drag_start("b"));
    board.sync(vec![item("a", 0.0, 0.0)]);
    assert!(!board.drag_move(20.0, 20.0));
    assert!(!board.drag_end());
}

/// Under Charge, two seeded items whose slots overlap settle apart; two
/// anchored ones are pushed apart while moving and, at rest, glide back
/// to their slots, overlapping as their arrangement does (F45). Under
/// Orbit the board keeps moving.
#[test]
fn charge_separates_overlapping_slots_and_orbit_never_rests() {
    let pair = || vec![item("a", 0.0, 0.0), item("b", 4.0, 2.0)];
    let charge = PhysicsChoice {
        law: PhysicsLaw::Charge,
        ..PhysicsChoice::default()
    }
    .into_spec();
    let mut anchored = PhysicsBoard::new();
    anchored.set_roles(RoleTable::uniform(Role::Anchored));
    anchored.sync(pair());
    anchored.set_stage(&charge).unwrap();
    let mut widest: f32 = 0.0;
    for _ in 0..SETTLE_TICKS * 4 {
        anchored.tick();
        widest = widest.max(anchored.gap("a", "b").unwrap());
    }
    assert!(
        widest > 30.0,
        "charge pushed the anchored pair apart: {widest:.0}"
    );
    assert_eq!(anchored.anchored_home_count(), 2, "both came home at rest");
    assert_eq!(anchored.position("b"), Some((4.0, 2.0)), "exactly home");

    let mut board = PhysicsBoard::new();
    board.sync(pair());
    board.set_stage(&charge).unwrap();
    for _ in 0..SETTLE_TICKS {
        board.tick();
    }
    let gap = board.gap("a", "b").unwrap();
    assert!(gap > 60.0, "charge pushed the seeded pair apart: {gap:.0}");
    assert!(!board.tick(), "charge comes to rest");

    board
        .set_stage(
            &PhysicsChoice {
                law: PhysicsLaw::Orbit,
                ..PhysicsChoice::default()
            }
            .into_spec(),
        )
        .unwrap();
    for _ in 0..120 {
        assert!(board.tick(), "orbit keeps ticking");
    }
    assert!(
        board.energy() > 0.0,
        "orbit carries energy: {}",
        board.energy()
    );
}

/// F150: the board takes the canvas's live stage, a composition
/// included (a mix runs graph-free, Charge's repulsion between the site
/// groups), and refuses a schedule, whose stages the canvas drives; a
/// refused stage leaves the board as it was.
#[test]
fn the_board_takes_a_stage_and_refuses_a_schedule() {
    use crate::canvas::composition::{GroupSource, PhysicsGrouping};
    use crate::canvas::dynamics_spec::{running_node, schedule_node};
    use crate::canvas::physics_catalog::PhysicsKindSource;
    use crate::canvas::schedule::{PhysicsStage, StageStop};
    let mut board = PhysicsBoard::new();
    board.sync(vec![item("a", 0.0, 0.0), item("b", 4.0, 2.0)]);
    let mut stage = PhysicsChoice::default().into_spec();
    for composition in [
        PhysicsComposition::Mix(vec![(PhysicsLaw::Charge, 2.0), (PhysicsLaw::Springs, 1.0)]),
        PhysicsComposition::Grouped(PhysicsGrouping::charge_between(GroupSource::Channel(
            PhysicsKindSource::Cluster,
        ))),
    ] {
        stage.root = running_node(PhysicsLaw::Charge, Some(&composition), &[]).unwrap();
        board.set_stage(&stage).unwrap();
        assert_eq!(board.stage(), &stage);
        assert_eq!(board.composition.as_ref(), Some(&composition));
        for _ in 0..SETTLE_TICKS {
            board.tick();
        }
        assert!(
            board.gap("a", "b").unwrap() > 10.0,
            "{composition:?} moved them"
        );
    }
    let held = board.stage().clone();
    let mut schedule = held.clone();
    schedule.root =
        schedule_node(&[PhysicsStage::law(PhysicsLaw::Springs, StageStop::Rest)]).unwrap();
    assert!(board.set_stage(&schedule).is_err());
    assert_eq!(board.stage(), &held, "a refused stage changes nothing");
}
