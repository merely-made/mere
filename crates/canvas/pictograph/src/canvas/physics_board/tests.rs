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

fn card_items() -> Vec<BoardItem> {
    (0..14)
        .map(|i| BoardItem {
            id: format!("card{i}"),
            slot: ((i % 5) as f32 * 40.0, (i / 5) as f32 * 40.0),
            site: format!("site{}", i % 3),
        })
        .collect()
}

fn card_links() -> Vec<(String, String)> {
    (1..14)
        .map(|i| (format!("card{i}"), format!("card{}", i * 7 % i.max(1))))
        .filter(|(a, b)| a != b)
        .collect()
}

/// The bits of every card's position, in item order.
fn card_bits(board: &PhysicsBoard) -> Vec<(u32, u32)> {
    card_items()
        .iter()
        .map(|item| {
            let (x, y) = board.position(&item.id).expect("a card the board holds");
            (x.to_bits(), y.to_bits())
        })
        .collect()
}

/// A board of 14 cards in three sites, linked, under `law`, run for `ticks`
/// ticks: in process, or on the far side of a transport (a Worker in
/// process, moving the bytes `postMessage` would).
fn board_run(law: PhysicsLaw, remote: bool, ticks: u64) -> Vec<(u32, u32)> {
    let hub = super::super::tests::loopback::Loopback::new();
    let mut board = if remote {
        PhysicsBoard::with_transport(hub.transport())
    } else {
        PhysicsBoard::new()
    };
    board.sync(card_items());
    board.set_links(card_links());
    board
        .set_stage(
            &PhysicsChoice {
                law,
                ..PhysicsChoice::default()
            }
            .into_spec(),
        )
        .unwrap();
    if remote {
        let mut turns = 0;
        while board.pace().ticks < ticks {
            hub.pump();
            board.tick();
            turns += 1;
            assert!(turns < 100 * ticks, "{} never ran", law.id());
        }
        assert_eq!(board.pace().ticks, ticks, "{}", law.id());
    } else {
        for _ in 0..ticks {
            board.tick();
        }
        assert_eq!(board.pace().ticks, ticks, "{}", law.id());
    }
    card_bits(&board)
}

/// The board's physics on the far side of a transport settles to the bits the
/// in-process board reaches, over every law in the catalog: the board sends
/// the bodies, the topology, the force set (as data) and the settle as
/// commands, and the far side builds the same forces.
#[test]
fn a_remote_board_settles_where_the_inline_one_does() {
    let slots: Vec<(u32, u32)> = card_items()
        .iter()
        .map(|item| (item.slot.0.to_bits(), item.slot.1.to_bits()))
        .collect();
    let mut moved = 0;
    for law in PhysicsLaw::ALL {
        let inline = board_run(law, false, 40);
        assert_eq!(board_run(law, true, 40), inline, "{}", law.id());
        moved += usize::from(inline != slots);
    }
    // Not a vacuous comparison: most laws move the cards in 40 ticks.
    assert!(moved >= PhysicsLaw::ALL.len() / 2, "{moved} laws moved the cards");
}

/// Before the far side has answered, a card is drawn at its slot, where the
/// inline backend would have put it; a drag pins and moves it through the same
/// commands, and the far side then holds it there.
#[test]
fn a_remote_board_draws_at_the_slots_and_drags_through_commands() {
    let hub = super::super::tests::loopback::Loopback::new();
    let mut board = PhysicsBoard::with_transport(hub.transport());
    assert_eq!(board.sync(card_items()), 14);
    assert_eq!(board.position("card3"), Some((120.0, 0.0)), "its slot, unanswered");
    for _ in 0..10 {
        hub.pump();
        board.tick();
    }
    assert!(board.drag_start("card3"));
    assert!(board.drag_move(500.0, 300.0));
    for _ in 0..4 {
        hub.pump();
        board.tick();
    }
    let held = board.position("card3").unwrap();
    assert!(
        (held.0 - 500.0).abs() < 1.0 && (held.1 - 300.0).abs() < 1.0,
        "held under the pointer: {held:?}"
    );
    assert!(board.drag_end());
    board.sync(Vec::new());
    assert!(board.position("card3").is_none());
}

/// A board built without an engine and with no Worker holds its cards at their
/// slots and never moves them: the transport that fails closed.
#[test]
fn a_board_with_an_unavailable_transport_keeps_the_slots() {
    let mut board = PhysicsBoard::with_transport(Box::new(
        crate::canvas::UnavailableTransport::default(),
    ));
    board.sync(card_items());
    for _ in 0..30 {
        board.tick();
    }
    for item in card_items() {
        assert_eq!(board.position(&item.id), Some(item.slot), "{}", item.id);
    }
}
