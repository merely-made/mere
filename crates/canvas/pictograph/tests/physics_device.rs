// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The canvas's and the board's device setters (physics catalog plan, P5c),
//! on a device booted the way a host's renderer boots it: the lane runs at
//! threshold 0, survives a law switch, stays off below the default
//! threshold, and clears. Skips where there is no adapter.

#![cfg(feature = "gpu")]

use std::time::Duration;

use pictograph::canvas::{
    BoardItem, Canvas, PhysicsBoard, PhysicsChoice, PhysicsLaw, physics_device_for,
};

fn device() -> Option<pictograph::canvas::PhysicsDevice> {
    let handles = netrender::boot().ok()?;
    println!("adapter: {}", handles.adapter.get_info().name);
    Some(physics_device_for(&handles))
}

/// Frames at 60 Hz, as a host draws them, so a lagged answer has its frame.
fn frames(canvas: &mut Canvas, count: usize) {
    for _ in 0..count {
        canvas.frame(1024, 600);
        std::thread::sleep(Duration::from_millis(16));
    }
}

#[test]
fn the_canvas_stages_its_repulsion_on_the_device_and_keeps_it_across_a_law_switch() {
    let Some(device) = device() else {
        eprintln!("no wgpu adapter: skipping the canvas device receipt");
        return;
    };
    let mut canvas = Canvas::with_sample_graph();
    assert_eq!(canvas.repulsion_stats(), None, "a lane before any device");

    // The default threshold (1,000 nodes) leaves the sample graph on the CPU.
    canvas.set_physics_device(Some(device.clone()));
    frames(&mut canvas, 20);
    let stats = canvas
        .repulsion_stats()
        .expect("a lane once a device is set");
    assert_eq!(stats.device_steps + stats.submissions, 0, "{stats:?}");

    let device = device.with_threshold(0);
    canvas.set_physics_device(Some(device.clone()));
    frames(&mut canvas, 60);
    let springs = canvas.repulsion_stats().unwrap();
    println!("springs: {springs:?}");
    assert!(springs.device_steps > 0, "{springs:?}");

    // A law switch replaces the force set, not the lane.
    let mut spec = canvas.dynamics_spec().expect("the record reads");
    PhysicsChoice {
        law: PhysicsLaw::Stress,
        ..PhysicsChoice::live(&canvas)
    }
    .write_into(&mut spec, &PhysicsChoice::live(&canvas));
    canvas.set_dynamics_spec(&spec).expect("Stress binds");
    frames(&mut canvas, 60);
    let stress = canvas.repulsion_stats().unwrap();
    println!("after the switch to Stress: {stress:?}");
    assert!(stress.device_steps > springs.device_steps, "{stress:?}");
    assert!(device.answers() > 0);

    canvas.set_physics_device(None);
    assert_eq!(
        canvas.repulsion_stats(),
        None,
        "the lane outlived its clearing"
    );
    assert!(canvas.physics_device().is_none());
}

#[test]
fn the_board_stages_its_repulsion_on_the_device() {
    let Some(device) = device() else {
        eprintln!("no wgpu adapter: skipping the board device receipt");
        return;
    };
    let mut board = PhysicsBoard::new();
    board.set_physics_device(Some(device.with_threshold(0)));
    let items: Vec<BoardItem> = (0..12)
        .map(|i| BoardItem {
            id: format!("card:{i}"),
            slot: ((i % 4) as f32 * 30.0, (i / 4) as f32 * 30.0),
            site: "fixture".into(),
        })
        .collect();
    board.sync(items);
    for _ in 0..60 {
        board.tick();
        std::thread::sleep(Duration::from_millis(16));
    }
    let stats = board.repulsion_stats().unwrap();
    println!("board: {stats:?}");
    assert!(stats.device_steps > 0, "{stats:?}");
}

/// Offloaded physics: a device set before `offload_physics` rides the
/// simulation onto the actor, and one set after arrives by command. The
/// actor's lane is read through the device's shared counters.
#[test]
fn an_offloaded_canvas_uses_the_device_set_before_or_after_offload() {
    let Some(device) = device() else {
        eprintln!("no wgpu adapter: skipping the offloaded receipt");
        return;
    };
    for before in [true, false] {
        // Fresh counters on the same client, so each run counts only itself.
        let device =
            pictograph::canvas::PhysicsDevice::new(device.client().clone()).with_threshold(0);
        let mut canvas = Canvas::with_sample_graph();
        if before {
            canvas.set_physics_device(Some(device.clone()));
        }
        canvas.offload_physics(std::sync::Arc::new(|| {}));
        if !before {
            canvas.set_physics_device(Some(device.clone()));
        }
        frames(&mut canvas, 60);
        let answered = device.answers();
        println!(
            "offloaded, device set {}: {answered} answers",
            if before { "before" } else { "after" }
        );
        assert!(answered > 0, "the actor never used the device");
        assert_eq!(
            canvas.repulsion_stats(),
            None,
            "offloaded stats are the device's"
        );
    }
}
