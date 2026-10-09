// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The speed dial on the canvas: every catalog law keeps one trajectory at
//! every speed (bit for bit since "Sum in key order", 2026-10-04), the canvas's
//! own frame loop adds nothing between frames that depends on how many there
//! are, slow motion draws every frame, and fast-forward under a real clock's
//! budget reports the speed it reached.

use crate::canvas::tests::ThroughView;
use std::time::Duration;

use super::*;
use crate::canvas::physics_catalog::PhysicsLaw;
use crate::canvas::{ElapsedStepConfig, Speed, StepBudget};
use kernel::graph::apply::{add_node, assert_relation};

const W: u32 = 800;
const H: u32 = 600;

/// `nodes` on a golden-angle spiral, each linked to the one before and to a
/// chord seven along: a connected graph whose every body interacts.
fn chorded(nodes: usize) -> Graph {
    let mut graph = Graph::new();
    let keys: Vec<_> = (0..nodes)
        .map(|i| {
            add_node(
                &mut graph,
                Some(uuid::Uuid::from_u128(0x5eed_0000 + i as u128)),
                format!("https://speed-{i}.test/"),
                PortablePoint::new(0.0, 0.0),
            )
        })
        .collect();
    for i in 1..nodes {
        assert_relation(&mut graph, keys[i], keys[i - 1], hyperlink());
        assert_relation(&mut graph, keys[i], keys[(i * 7) % i], hyperlink());
    }
    graph
}

/// A free canvas (no arrangement) seeded in key order, under `law`.
fn canvas(nodes: usize, law: PhysicsLaw) -> Canvas {
    let mut canvas = Canvas::with_graph(chorded(nodes));
    canvas.set_layout_strategy(None);
    canvas.resize(W, H);
    let mut keys: Vec<NodeKey> = canvas.graph().nodes().map(|(k, _)| k).collect();
    keys.sort_by_key(|k| k.index());
    canvas.physics.seed(
        keys.iter()
            .enumerate()
            .map(|(i, &k)| {
                let a = i as f32 * 2.399_963;
                let r = 40.0 * (i as f32 + 0.5).sqrt();
                (k, Point2D::new(r * a.cos(), r * a.sin()))
            })
            .collect(),
    );
    canvas.pick_law(law).unwrap();
    canvas
}

fn bits(canvas: &Canvas) -> Vec<(usize, u32, u32)> {
    let mut out: Vec<_> = canvas
        .view
        .positions()
        .map(|(k, p)| (k.index(), p.x.to_bits(), p.y.to_bits()))
        .collect();
    out.sort_unstable();
    out
}

/// The settle the frame-loop receipt runs: short, since every frame composes
/// the whole canvas at the dev profile's opt-level 0.
const FRAME_TICKS: u32 = 60;

/// Frame the canvas at `speed` through `Canvas::frame` until a settle of
/// `ticks` ends; the frames, the ticks, and where it came to rest.
fn settle(speed: f32, ticks: u32) -> (u32, u64, Vec<(usize, u32, u32)>) {
    let mut canvas = canvas(30, PhysicsLaw::Springs);
    canvas.set_physics_speed(Speed::from_factor(speed));
    canvas.physics.halt();
    canvas.physics.settle(ticks);
    let mut frames = 0;
    loop {
        frames += 1;
        let (_, moving) = canvas.frame(W, H);
        if !moving {
            break;
        }
        assert!(frames < 20 * ticks, "the settle never ended");
    }
    (frames, canvas.physics_pace().ticks, bits(&canvas))
}

/// Through the canvas's own frame loop, which reprojects, drains and rebuilds
/// between frames: Springs settles to the same bits at 1x, 0.2x and 50x.
#[test]
fn the_canvas_keeps_one_trajectory_at_every_speed() {
    let (frames_1x, ticks_1x, at_1x) = settle(1.0, FRAME_TICKS);
    assert_eq!(settle(1.0, FRAME_TICKS).2, at_1x, "1x twice");
    let (frames_slow, ticks_slow, slow) = settle(0.2, FRAME_TICKS);
    let (frames_fast, ticks_fast, fast) = settle(50.0, FRAME_TICKS);
    let budget = u64::from(FRAME_TICKS);
    assert_eq!((ticks_1x, ticks_slow, ticks_fast), (budget, budget, budget));
    assert_eq!(frames_1x, FRAME_TICKS);
    assert_eq!(frames_slow, 5 * (FRAME_TICKS - 1));
    assert_eq!(frames_fast, FRAME_TICKS.div_ceil(50));
    assert_eq!(slow, at_1x, "0.2x left the 1x trajectory");
    assert_eq!(fast, at_1x, "50x left the 1x trajectory");
    // Positive control: one tick more lands elsewhere, so the comparison
    // resolves a single tick.
    let (_, ticks_more, more) = settle(1.0, FRAME_TICKS + 1);
    assert_eq!(ticks_more, budget + 1);
    assert_ne!(more, at_1x, "one more tick matched");
}

/// The ticks the every-law receipt runs.
const LAW_TICKS: u64 = 150;

/// `law`'s force set on the canvas, stepped by its own backend at `speed`
/// until `ticks` have run, then read at real time (so slow motion's drawing
/// between ticks is not what is compared).
fn law_at(law: PhysicsLaw, speed: f32, ticks: u64) -> Vec<(usize, u32, u32)> {
    let mut canvas = canvas(30, law);
    canvas.physics.settle(u32::MAX);
    canvas.set_physics_speed(Speed::from_factor(speed));
    while canvas.physics_pace().ticks < ticks {
        canvas.physics.advance_frame(&mut canvas.view);
    }
    assert_eq!(canvas.physics_pace().ticks, ticks, "{} overshot", law.id());
    canvas.set_physics_speed(Speed::REAL_TIME);
    canvas.physics.refresh(&mut canvas.view);
    bits(&canvas)
}

/// Every catalog law, its real force set: two 1x runs agree bit for bit, and
/// 0.2x and 50x land on the same bits at the same tick. One tick more lands
/// elsewhere (the control), except under Still, whose bodies hold.
#[test]
fn every_law_keeps_one_trajectory_at_every_speed() {
    let mut held = Vec::new();
    for law in PhysicsLaw::ALL {
        let at_1x = law_at(law, 1.0, LAW_TICKS);
        assert_eq!(law_at(law, 1.0, LAW_TICKS), at_1x, "{}: 1x twice", law.id());
        assert_eq!(law_at(law, 0.2, LAW_TICKS), at_1x, "{}: 0.2x", law.id());
        assert_eq!(law_at(law, 50.0, LAW_TICKS), at_1x, "{}: 50x", law.id());
        if law_at(law, 1.0, LAW_TICKS + 1) == at_1x {
            held.push(law.id());
        }
    }
    assert_eq!(
        held,
        vec![PhysicsLaw::Still.id()],
        "laws whose next tick moved nothing"
    );
}

#[test]
fn slow_motion_on_the_canvas_draws_every_frame_and_steps_every_fifth() {
    let mut canvas = canvas(30, PhysicsLaw::Springs);
    canvas.set_physics_speed(Speed::from_factor(0.2));
    let mut last = bits(&canvas);
    let mut stepped = 0;
    for frame in 1..=60 {
        let before = canvas.physics_pace().ticks;
        let (_, moving) = canvas.frame(W, H);
        assert!(moving);
        stepped += u32::from(canvas.physics_pace().ticks > before);
        let drawn = bits(&canvas);
        assert_ne!(drawn, last, "frame {frame} drew nothing new");
        last = drawn;
    }
    assert_eq!(stepped, 13, "the lead tick, then one every fifth frame");
    assert_eq!(canvas.physics_pace().ticks, 13);
}

/// The canvas's timed path at `speed` under a `budget` on the real clock:
/// per frame the steps run, the compute spent, and whether the budget bound.
fn timed(
    nodes: usize,
    speed: f32,
    budget: Duration,
    frames: u32,
) -> (Canvas, Vec<(u32, Duration, bool)>) {
    let mut canvas = canvas(nodes, PhysicsLaw::Springs);
    canvas.set_physics_speed(Speed::from_factor(speed));
    canvas.set_physics_step_budget(Some(StepBudget {
        per_frame: budget,
        clock: seiche::monotonic_clock,
        margin: Duration::ZERO,
    }));
    let mut out = Vec::new();
    for frame in 0..=frames {
        let at = seiche::TICK_DURATION * frame;
        canvas.frame_at(W, H, at, ElapsedStepConfig::default());
        let report = canvas.elapsed_step_report().unwrap();
        if frame > 0 {
            out.push((report.steps, report.compute.unwrap(), report.budget_bound));
        }
    }
    (canvas, out)
}

#[test]
fn fast_forward_on_the_canvas_stops_at_the_budget_and_reports_the_speed() {
    // Measure a tick on this build and machine, then give 50x a budget of
    // about four: the graph is too large for fifty ticks a frame.
    let (_, at_1x) = timed(240, 1.0, Duration::from_secs(1), 20);
    let mut costs: Vec<Duration> = at_1x.iter().map(|f| f.1).collect();
    costs.sort();
    let tick = costs[costs.len() / 2];
    let budget = (tick * 4).max(Duration::from_millis(2));
    let (canvas, fast) = timed(240, 50.0, budget, 30);
    let worst_tick = costs[costs.len() - 1];
    for (frame, &(steps, compute, bound)) in fast.iter().enumerate() {
        assert!(bound, "frame {frame}: {steps} steps did not hit the budget");
        assert!((1..50).contains(&steps), "frame {frame}: {steps} steps");
        // The forecast can be wrong by at most one tick's variation.
        assert!(
            compute <= budget + worst_tick,
            "frame {frame}: {compute:?} against {budget:?} (+{worst_tick:?})"
        );
    }
    let reached = canvas.physics_pace().effective_speed.unwrap();
    assert!((1.0..50.0).contains(&reached), "reached {reached}");
    let inside = fast.iter().filter(|f| f.1 <= budget).count();
    println!(
        "tick {tick:?} (worst {worst_tick:?}), budget {budget:?}: reached {reached:.1}x, \
         {inside} of {} frames inside the budget, worst {:?}",
        fast.len(),
        fast.iter().map(|f| f.1).max().unwrap()
    );

    // Control: six nodes, with a budget twice what fifty of their slowest
    // ticks cost, run every tick owed and reach the speed asked while they
    // settle.
    let (_, small_1x) = timed(6, 1.0, Duration::from_secs(1), 20);
    let small_worst = small_1x.iter().map(|f| f.1).max().unwrap();
    let roomy = small_worst * 100;
    let (small, cheap) = timed(6, 50.0, roomy, 6);
    let settling: Vec<_> = cheap.iter().filter(|f| f.0 > 0).collect();
    assert!(settling.len() >= 5, "{cheap:?}");
    assert!(
        settling.iter().all(|f| !f.2 && f.0 == 50),
        "{settling:?} under {roomy:?}"
    );
    let reached = small.physics_pace().effective_speed.unwrap();
    assert!(
        (reached - 50.0).abs() < 0.5,
        "the small graph reached {reached}"
    );
}

#[test]
fn the_board_runs_at_its_speed() {
    let mut board = PhysicsBoard::new();
    let item = |id: &str, slot| BoardItem {
        id: id.to_string(),
        slot,
        site: "fixture".to_string(),
    };
    board.sync(vec![
        item("a", (0.0, 0.0)),
        item("b", (10.0, 0.0)),
        item("c", (0.0, 10.0)),
    ]);
    board.set_speed(Speed::from_factor(50.0));
    let before = board.pace().ticks;
    for _ in 0..3 {
        board.tick();
    }
    assert_eq!(board.pace().ticks - before, 150);
}

/// The native entry point (ruled 2026-10-04, "Mere entry point, then
/// turnstone"): a display's refresh rate in millihertz becomes half its
/// frame on the monotonic clock, 60 Hz's when unknown, and 50x then binds on
/// that budget.
#[test]
fn the_display_rate_sets_half_its_frame_as_the_budget() {
    let mut canvas = canvas(24, PhysicsLaw::Springs);
    assert!(canvas.physics_step_budget().is_none());
    let us = |canvas: &Canvas| canvas.physics_step_budget().unwrap().per_frame.as_micros();
    canvas.set_physics_display_rate(Some(165_000));
    assert_eq!(us(&canvas), 3_030);
    canvas.set_physics_display_rate(Some(144_000));
    assert_eq!(us(&canvas), 3_472);
    canvas.set_physics_display_rate(Some(60_000));
    assert_eq!(us(&canvas), 8_333);
    canvas.set_physics_display_rate(None);
    assert_eq!(us(&canvas), 8_333);
    assert_eq!(canvas.physics_step_budget().unwrap().margin, Duration::ZERO);
}
