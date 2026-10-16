// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The graph grammar's attention and seed through the canvas (`attention.rs`).

use super::*;
use scenograph::attention::Attention;

/// Two members, nothing selected (opening a member would select it).
fn two_members() -> (Canvas, uuid::Uuid, uuid::Uuid) {
    let mut graph = Graph::new();
    let a = graph.add_node("https://a.example".into(), PortablePoint::zero());
    let b = graph.add_node("https://b.example".into(), PortablePoint::zero());
    let (a, b) = (graph.get_node(a).unwrap().id, graph.get_node(b).unwrap().id);
    (Canvas::with_graph(graph), a, b)
}

#[test]
fn deselected_members_are_retained_until_dismissed() {
    let (mut canvas, a, b) = two_members();
    canvas.select_member(a);
    canvas.select_member(b);
    assert_eq!(
        canvas.member_attention(a),
        Some(Attention::Retained),
        "read before any frame"
    );
    assert_eq!(canvas.member_attention(b), Some(Attention::Forward));

    assert!(canvas.dismiss_retained_member(a));
    assert_eq!(canvas.member_attention(a), Some(Attention::Ambient));
    assert!(
        !canvas.dismiss_retained_member(b),
        "a forward member is not dismissed"
    );

    canvas.clear_selection();
    canvas.frame(400, 300);
    assert_eq!(canvas.members_with_attention(Attention::Retained), vec![b]);
    assert_eq!(canvas.dismiss_all_retained(), 1);
    assert_eq!(canvas.member_attention(b), Some(Attention::Ambient));
}

#[test]
fn a_foreground_pin_is_attention_not_position() {
    let (mut canvas, a, b) = two_members();
    let key = canvas.graph().get_node_key_by_id(a).unwrap();
    canvas.select_member(a);
    assert!(canvas.set_member_foreground_pin(a, true));
    canvas.select_member(b);
    assert_eq!(canvas.member_attention(a), Some(Attention::Forward));
    assert!(!canvas.pinned_nodes.contains(&key), "no position pin");

    assert!(canvas.set_member_foreground_pin(a, false));
    assert_eq!(canvas.member_attention(a), Some(Attention::Retained));
    assert!(!canvas.set_member_foreground_pin(uuid::Uuid::from_u128(7), true));
}

#[test]
fn suggestions_yield_and_removed_members_are_pruned() {
    let (mut canvas, a, b) = two_members();
    canvas.set_suggestion_settings(scenograph::suggestion::SuggestionSettings {
        on: true,
        ..Default::default()
    });
    canvas.set_suggested_members([a, b, uuid::Uuid::from_u128(9)]);
    assert_eq!(
        canvas.members_with_attention(Attention::Suggestion).len(),
        2
    );
    canvas.select_member(a);
    assert_eq!(canvas.member_attention(a), Some(Attention::Forward));

    canvas.remove_focused().expect("removes the focused member");
    assert_eq!(canvas.member_attention(a), None);
    assert_eq!(
        canvas.members_with_attention(Attention::Suggestion),
        vec![b]
    );
    assert!(
        canvas
            .members_with_attention(Attention::Retained)
            .is_empty()
    );
}

fn paused_canvas() -> (Canvas, Vec<NodeKey>) {
    let mut graph = Graph::new();
    let keys = (0..3)
        .map(|i| graph.add_node(format!("https://seed-{i}.example"), PortablePoint::zero()))
        .collect();
    let mut canvas = Canvas::with_graph(graph);
    canvas.resize(800, 600);
    canvas.set_physics_paused(true);
    (canvas, keys)
}

fn scatter(canvas: &mut Canvas, keys: &[NodeKey], at: PortablePoint) {
    for key in keys {
        canvas.view.set_position(*key, at);
    }
}

#[test]
fn reset_returns_unpinned_nodes_to_their_identity_seeds() {
    let (mut canvas, keys) = paused_canvas();
    let id = canvas.graph().get_node(keys[0]).unwrap().id;
    canvas.select_member(id);
    assert!(canvas.pin_focused());
    let far = PortablePoint::new(900.0, 900.0);
    scatter(&mut canvas, &keys, far);

    let moves = canvas.reset_to_seed();
    assert_eq!(moves.len(), 2, "the position-pinned node stays");
    assert!(moves.iter().all(|m| m.key != id));
    assert_eq!(canvas.view.position_of(keys[0]), Some(far));
    let seeded: Vec<_> = keys[1..]
        .iter()
        .map(|k| canvas.view.position_of(*k))
        .collect();
    for m in &moves {
        assert_eq!(m.from, sceno::Vec2::new(far.x, far.y));
    }

    canvas.frame(800, 600);
    let held: Vec<_> = keys[1..]
        .iter()
        .map(|k| canvas.view.position_of(*k))
        .collect();
    assert_eq!(held, seeded, "a paused frame keeps the seeds");
    assert!(canvas.reset_to_seed().is_empty(), "already home");
}

#[test]
fn set_seed_then_restore_the_recipe_seed() {
    let (mut canvas, keys) = paused_canvas();
    canvas.reset_to_seed();
    let recipe = canvas.view.position_of(keys[1]).unwrap();

    let saved = PortablePoint::new(40.0, -20.0);
    canvas.view.set_position(keys[1], saved);
    canvas.set_seed();
    assert!(canvas.has_saved_seed());
    canvas
        .view
        .set_position(keys[1], PortablePoint::new(500.0, 500.0));
    canvas.reset_to_seed();
    assert_eq!(canvas.view.position_of(keys[1]), Some(saved));

    assert!(canvas.restore_recipe_seed());
    canvas.reset_to_seed();
    assert_eq!(canvas.view.position_of(keys[1]), Some(recipe));
}

#[test]
fn the_active_arrangement_is_the_recipe_seed() {
    let (mut canvas, keys) = paused_canvas();
    canvas.set_layout_strategy(Some("test.slots".to_string()));
    let slots: Vec<_> = keys
        .iter()
        .enumerate()
        .map(|(i, key)| (*key, PortablePoint::new(100.0 * i as f32, 0.0)))
        .collect();
    canvas.apply_strategy_positions(&slots);
    scatter(&mut canvas, &keys, PortablePoint::new(-300.0, 250.0));
    canvas.reset_to_seed();
    for (key, slot) in slots {
        assert_eq!(canvas.view.position_of(key), Some(slot));
    }
}

#[test]
fn a_paused_reset_queues_the_whole_placement_at_the_reset_ease() {
    let (mut canvas, keys) = paused_canvas();
    let id = canvas.graph().get_node(keys[0]).unwrap().id;
    canvas.select_member(id);
    assert!(canvas.pin_focused());
    scatter(&mut canvas, &keys, PortablePoint::new(700.0, 700.0));
    crate::canvas::controls::CanvasCommand::ResetToSeed.apply(&mut canvas, (800, 600));

    let ret = canvas
        .take_stop_return()
        .expect("a paused reset queues its return");
    assert_eq!(ret.duration_ms, Some(scenograph::seed::RESET_EASE_MS));
    assert_eq!(
        ret.from.len(),
        3,
        "the whole placement, so landing keeps the pinned node"
    );
    assert_eq!(ret.to.len(), 3);
    let pinned_to = ret.to.iter().find(|(key, _)| *key == keys[0]).unwrap().1;
    assert_eq!(pinned_to, PortablePoint::new(700.0, 700.0));
    let transition =
        crate::canvas::physics_controls::ArrangementTransition::returning(&canvas, &ret)
            .expect("schedules")
            .expect("something moves");
    let mut transition = Some(transition);
    let (_, done) = transition.as_mut().unwrap().advance(0.0);
    assert!(!done);
}

#[test]
fn the_foreground_pin_command_toggles_the_selection() {
    let (mut canvas, a, b) = two_members();
    use crate::canvas::controls::CanvasCommand;
    canvas.select_member(a);
    canvas.toggle_select_member(b);
    CanvasCommand::ToggleForegroundPin.apply(&mut canvas, (800, 600));
    assert_eq!(canvas.member_foreground_pinned(a), Some(true));
    assert_eq!(canvas.member_foreground_pinned(b), Some(true));
    CanvasCommand::ToggleForegroundPin.apply(&mut canvas, (800, 600));
    assert_eq!(canvas.member_foreground_pinned(a), Some(false));
    canvas.clear_selection();
    assert!(!canvas.toggle_selection_foreground_pin());
    CanvasCommand::DismissRetained.apply(&mut canvas, (800, 600));
    assert!(
        canvas
            .members_with_attention(Attention::Retained)
            .is_empty()
    );
}
