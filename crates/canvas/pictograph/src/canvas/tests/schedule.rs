// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Schedules through the canvas (dynamics grammar plan, G3): "Stress, then
//! Springs, remembering it" reproduces Stress's captured layout, by the
//! anchored items' springs while Springs moves and exactly once it rests
//! (F45); the same schedule capturing seeded is the control, Springs
//! relaxing away from the capture; and a stage ends at its frame count or
//! at its law's own stop.

use super::*;
use crate::canvas::physics_catalog::PhysicsLaw;
use crate::canvas::schedule::{PhysicsStage, StageStop};
use crate::canvas::tests::ThroughView;
use seiche::Role;

const N: usize = 40;

/// A random recursive tree of forty plus ten chords, seeded on a spiral.
fn tree_canvas() -> (Canvas, Vec<NodeKey>) {
    let mut state: u64 = 0x5c4ed;
    let mut below = |n: usize| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((state >> 33) % n as u64) as usize
    };
    let mut graph = Graph::new();
    let keys: Vec<NodeKey> = (0..N)
        .map(|i| {
            graph.add_node(
                format!("https://t{}.example/{i}", i % 5),
                PortablePoint::new(0.0, 0.0),
            )
        })
        .collect();
    for i in 1..N {
        graph.assert_relation(keys[below(i)], keys[i], hyperlink());
    }
    for _ in 0..10 {
        let (a, b) = (below(N), below(N));
        if a != b {
            graph.assert_relation(keys[a], keys[b], hyperlink());
        }
    }
    let mut canvas = Canvas::with_graph(graph);
    canvas.resize(1400, 900);
    canvas.set_physics_paused(true);
    canvas.set_layout_strategy(Some("test.spiral".to_string()));
    let seed: Vec<_> = keys
        .iter()
        .enumerate()
        .map(|(i, &k)| {
            let (r, a) = (40.0 * (i as f32).sqrt(), i as f32 * 2.399_963);
            (k, PortablePoint::new(r * a.cos(), r * a.sin()))
        })
        .collect();
    canvas.apply_strategy_positions(&seed);
    (canvas, keys)
}

/// What the run left: the capture, the layout when the last stage ended (at
/// its rest, before any glide home), the layout once anchored items are
/// home, and the frames it took.
struct Run {
    captured: Vec<(NodeKey, PortablePoint)>,
    at_rest: Vec<(NodeKey, PortablePoint)>,
    home: Vec<(NodeKey, PortablePoint)>,
    frames: usize,
}

fn remembering(capture: Role) -> Run {
    let (mut canvas, _) = tree_canvas();
    canvas.pick_schedule(vec![
        PhysicsStage::law(PhysicsLaw::Stress, StageStop::Rest).capturing(capture),
        PhysicsStage::law(PhysicsLaw::Springs, StageStop::Rest),
    ]);
    let mut frames = 0;
    let mut captured = None;
    while canvas.physics_schedule_stage().is_some() && frames < 20_000 {
        canvas.step_layout();
        frames += 1;
        if captured.is_none() {
            captured = canvas.captured_positions().map(<[_]>::to_vec);
        }
    }
    assert!(
        canvas.physics_schedule_stage().is_none(),
        "the schedule ended"
    );
    let at_rest: Vec<_> = canvas.view.positions().collect();
    let anchored = capture == Role::Anchored;
    while anchored && canvas.anchored_home_count() < N && frames < 20_000 {
        canvas.step_layout();
        frames += 1;
    }
    Run {
        captured: captured.expect("Stress's stage captured"),
        at_rest,
        home: canvas.view.positions().collect(),
        frames,
    }
}

/// RMS and largest distance of `layout` from `reference`, item by item.
fn off(layout: &[(NodeKey, PortablePoint)], reference: &[(NodeKey, PortablePoint)]) -> (f32, f32) {
    let at: HashMap<NodeKey, PortablePoint> = reference.iter().copied().collect();
    let d: Vec<f32> = layout
        .iter()
        .map(|(k, p)| {
            let q = at[k];
            (p.x - q.x).hypot(p.y - q.y)
        })
        .collect();
    let rms = (d.iter().map(|v| v * v).sum::<f32>() / d.len() as f32).sqrt();
    (rms, d.iter().copied().fold(0.0, f32::max))
}

/// G3's done-condition: the sequenced blend reproduces the captured anchor
/// layout. Taken anchored, Springs ends within the stated tolerance of
/// Stress's capture by the springs alone (RMS under a quarter of a rest
/// length, 42.5), and exactly at it once the anchored items come home
/// (F45). Taken seeded, the default role, Springs relaxes away from it.
#[test]
fn stress_then_springs_remembering_it_reproduces_the_capture() {
    let anchored = remembering(Role::Anchored);
    let seeded = remembering(Role::Seeded);
    let (rest_rms, rest_max) = off(&anchored.at_rest, &anchored.captured);
    let (home_rms, home_max) = off(&anchored.home, &anchored.captured);
    let (seeded_rms, seeded_max) = off(&seeded.home, &seeded.captured);
    println!(
        "stress, then springs, capture anchored: at springs' rest RMS {rest_rms:.2} (largest \
         {rest_max:.2}); home after {} frames, RMS {home_rms:.4} (largest {home_max:.4}). Capture \
         seeded (the control): RMS {seeded_rms:.2} (largest {seeded_max:.2}) after {} frames",
        anchored.frames, seeded.frames
    );
    assert!(
        rest_rms < 42.5,
        "the springs hold it near the capture: {rest_rms}"
    );
    assert!(
        home_max < 1e-3,
        "anchored items end exactly at the capture: {home_max}"
    );
    assert!(
        seeded_rms > 2.0 * rest_rms && seeded_rms > 20.0,
        "seeded, Springs relaxes away: {seeded_rms}"
    );
}

/// A stage ends at its frame count; a law-done stage ends at a law's own
/// stop, and, for a law with no stop of its own, at rest.
#[test]
fn stages_end_at_their_frames_and_at_their_laws_stop() {
    let (mut canvas, _) = tree_canvas();
    canvas.pick_schedule(vec![
        PhysicsStage::law(PhysicsLaw::Springs, StageStop::Frames(60)),
        PhysicsStage::law(PhysicsLaw::Stress, StageStop::LawDone),
    ]);
    for _ in 0..59 {
        canvas.step_layout();
    }
    assert_eq!(canvas.physics_schedule_stage(), Some(0));
    canvas.step_layout();
    assert_eq!(canvas.physics_schedule_stage(), Some(1));
    assert_eq!(canvas.view().law, PhysicsLaw::Stress);
    let settles = canvas.settle_count();
    let mut frames = 0;
    while canvas.physics_schedule_stage().is_some() && frames < 20_000 {
        canvas.step_layout();
        frames += 1;
    }
    assert!(
        canvas.physics_schedule_stage().is_none(),
        "Stress has no stop of its own: it ends at rest"
    );
    assert!(canvas.settle_count() > settles);
    // A pick replaces a schedule under way.
    canvas.pick_schedule(vec![PhysicsStage::law(PhysicsLaw::Stress, StageStop::Rest)]);
    canvas.pick_law(PhysicsLaw::Springs).unwrap();
    assert!(canvas.physics_schedule_stage().is_none());
}
