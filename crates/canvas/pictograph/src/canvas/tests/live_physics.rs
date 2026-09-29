// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Exercise the inline physics backend through the same frame and pointer
//! methods the browser uses. These are behavior checks, not timing receipts.

use super::*;

fn pair() -> (Canvas, Vec<NodeKey>) {
    let mut graph = Graph::new();
    for url in ["https://live-a.example", "https://live-b.example"] {
        graph.add_node(url.to_string(), PortablePoint::new(0.0, 0.0));
    }
    let mut canvas = Canvas::with_graph(graph);
    assert!(matches!(canvas.physics, Physics::Inline(_)));
    canvas.resize(800, 600);
    let keys: Vec<_> = canvas.graph().nodes().map(|(key, _)| key).collect();
    canvas.set_layout_strategy(Some("test.live".to_string()));
    canvas.apply_strategy_positions(&[
        (keys[0], PortablePoint::new(-80.0, 0.0)),
        (keys[1], PortablePoint::new(80.0, 0.0)),
    ]);
    canvas.frame(800, 600);
    (canvas, keys)
}

fn positions(canvas: &Canvas, keys: &[NodeKey]) -> Vec<PortablePoint> {
    keys.iter()
        .map(|&key| {
            let point = canvas.node_position(key).expect("live node position");
            assert!(point.x.is_finite() && point.y.is_finite(), "{point:?}");
            point
        })
        .collect()
}

fn changed(before: &[PortablePoint], after: &[PortablePoint]) -> bool {
    assert_eq!(before.len(), after.len());
    before
        .iter()
        .zip(after)
        .any(|(a, b)| (a.x - b.x).hypot(a.y - b.y) > 0.001)
}

fn frames(canvas: &mut Canvas, keys: &[NodeKey], count: usize) {
    for _ in 0..count {
        canvas.frame(800, 600);
        positions(canvas, keys);
        assert!(canvas.physics_energy().is_finite());
    }
}

#[test]
fn inline_play_pause_resume_changes_then_holds_then_changes_positions() {
    let (mut canvas, keys) = pair();
    // Exercise the no-arrangement half of the global pause control too.
    canvas.set_layout_strategy(None);
    let before = positions(&canvas, &keys);
    frames(&mut canvas, &keys, 24);
    let moving = positions(&canvas, &keys);
    assert!(changed(&before, &moving), "play must advance the bodies");

    canvas.set_physics_paused(true);
    frames(&mut canvas, &keys, 12);
    assert_eq!(
        positions(&canvas, &keys),
        moving,
        "pause must hold positions"
    );
    assert!(!canvas.is_settling());

    canvas.set_physics_paused(false);
    frames(&mut canvas, &keys, 24);
    assert!(
        changed(&moving, &positions(&canvas, &keys)),
        "resume must advance from the frozen state"
    );
    assert!(canvas.is_settling());
}

#[test]
fn arrangement_pause_freezes_motion_and_restore_is_explicit() {
    let (mut canvas, keys) = pair();
    let original = positions(&canvas, &keys);
    let slots = canvas.strategy_positions.clone();
    canvas.set_physics_paused(false);
    frames(&mut canvas, &keys, 24);
    let relaxed = positions(&canvas, &keys);
    assert!(changed(&original, &relaxed));

    canvas.set_physics_paused(true);
    frames(&mut canvas, &keys, 12);
    assert_eq!(positions(&canvas, &keys), relaxed, "pause must not restore");
    assert_eq!(
        canvas.strategy_positions, slots,
        "pause preserves the slots"
    );

    canvas.set_physics_paused(false);
    assert_eq!(positions(&canvas, &keys), relaxed, "resume must not snap");
    frames(&mut canvas, &keys, 1);
    let resumed = positions(&canvas, &keys);
    for ((start, frozen), current) in original.iter().zip(&relaxed).zip(&resumed) {
        let from_frozen = (current.x - frozen.x).hypot(current.y - frozen.y);
        let from_original = (current.x - start.x).hypot(current.y - start.y);
        assert!(
            from_frozen < from_original,
            "first resumed step starts at the frozen view"
        );
    }
    frames(&mut canvas, &keys, 24);
    assert!(changed(&relaxed, &positions(&canvas, &keys)));

    assert!(canvas.restore_arrangement());
    assert!(canvas.physics_paused());
    assert_eq!(positions(&canvas, &keys), original);
    frames(&mut canvas, &keys, 6);
    assert_eq!(
        positions(&canvas, &keys),
        original,
        "restore holds its placement"
    );
    assert_eq!(canvas.strategy_positions, slots);

    // Restoration must seed the solver too, not just cover stale bodies with
    // an overlay: the first step after play begins beside the restored slots.
    canvas.set_physics_paused(false);
    frames(&mut canvas, &keys, 1);
    for ((start, frozen), current) in original.iter().zip(&relaxed).zip(positions(&canvas, &keys)) {
        let from_original = (current.x - start.x).hypot(current.y - start.y);
        let from_relaxed = (current.x - frozen.x).hypot(current.y - frozen.y);
        assert!(
            from_original < from_relaxed,
            "restore must seed the simulation"
        );
    }
    canvas.set_layout_strategy(None);
    assert!(!canvas.restore_arrangement());
}

#[test]
fn inline_playing_arrangement_holds_a_drag_then_releases_the_body() {
    let (mut canvas, keys) = pair();
    canvas.set_physics_paused(false);
    let start = canvas.screen_position_of(keys[0]).expect("node on screen");
    let end = (start.0 - 64.0, start.1 - 48.0);
    let target = canvas.screen_to_world(end);
    canvas.pointer_down(PointerButton::Left, start.0, start.1);
    assert!(canvas.cursor_moved(end.0, end.1));
    assert!(
        canvas
            .drag
            .is_some_and(|drag| drag.node == keys[0] && drag.moved)
    );
    let neighbor_before = positions(&canvas, &keys)[1];

    for _ in 0..12 {
        frames(&mut canvas, &keys, 1);
        let held = positions(&canvas, &keys)[0];
        assert!(
            (held.x - target.x).hypot(held.y - target.y) < 0.001,
            "the grabbed body must remain under the pointer: {held:?}"
        );
    }
    assert!(changed(&[neighbor_before], &[positions(&canvas, &keys)[1]],));

    assert!(canvas.pointer_up(PointerButton::Left, end.0, end.1));
    assert!(canvas.drag.is_none());
    let dropped = positions(&canvas, &keys)[0];
    frames(&mut canvas, &keys, 24);
    assert!(
        changed(&[dropped], &[positions(&canvas, &keys)[0]]),
        "release must return an ordinary grabbed node to the simulation"
    );
    assert_eq!(canvas.layout_strategy(), Some("test.live"));
    assert!(!canvas.physics_paused());
}

#[test]
fn paused_free_layout_accepts_nudge_and_explicit_reseed() {
    let (mut canvas, keys) = pair();
    canvas.set_layout_strategy(None);
    canvas.set_physics_paused(true);
    canvas.select_only(keys[0]);
    let before = positions(&canvas, &keys);
    assert!(canvas.nudge_focused(50.0, -30.0));
    let nudged = positions(&canvas, &keys);
    assert!(changed(&before, &nudged));
    frames(&mut canvas, &keys, 3);
    assert_eq!(positions(&canvas, &keys), nudged);

    canvas.reseed();
    let reseeded = positions(&canvas, &keys);
    assert!(changed(&nudged, &reseeded));
    frames(&mut canvas, &keys, 3);
    assert_eq!(positions(&canvas, &keys), reseeded);
}

#[test]
fn paused_placements_follow_graph_membership_and_replacement() {
    let (mut canvas, keys) = pair();
    assert!(canvas.ingest_graph(|graph| graph.remove_node(keys[0])));
    frames(&mut canvas, &keys[1..], 3);
    assert!(canvas.view.position_of(keys[0]).is_none());
    assert!(canvas.restore_arrangement());
    frames(&mut canvas, &keys[1..], 3);
    assert!(canvas.view.position_of(keys[0]).is_none());

    let added = canvas.open_member_as_new_node(None, "https://new-paused.example");
    let added = canvas.graph().get_node_by_id(added).unwrap().0;
    let placed = canvas.node_position(added).unwrap();
    frames(&mut canvas, &[keys[1], added], 3);
    assert_eq!(canvas.node_position(added), Some(placed));

    let mut replacement = Graph::new();
    let replacement_key = replacement.add_node(
        "https://replacement.example".to_string(),
        PortablePoint::new(0.0, 0.0),
    );
    canvas.set_graph(replacement);
    assert!(
        !canvas.restore_arrangement(),
        "old slots belong to the old graph"
    );
    let placed = canvas.node_position(replacement_key).unwrap();
    frames(&mut canvas, &[replacement_key], 3);
    assert_eq!(canvas.node_position(replacement_key), Some(placed));
}
