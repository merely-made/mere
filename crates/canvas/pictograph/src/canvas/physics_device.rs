// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The host's device for the canvas's and the board's physics (feature `gpu`):
//! `NodeExclusion`, the repulsion eight laws carry, staged on the GPU through
//! seiche's lagged seam, with rapier keeping every other role and the CPU law
//! covering every step the device cannot. (Physics catalog plan, P5c, ruled
//! 2026-10-02: "Setters with a shared PhysicsDevice".)
//!
//! A host builds one [`PhysicsDevice`] from its renderer's handles
//! ([`physics_device_for`]) and hands clones to each canvas and board. The
//! device carries the host's threshold and staleness limit, so a host's
//! measured numbers are set once. Setting it before or after `offload` both
//! work: offloaded, the lane rides a physics command onto the actor.

pub use seiche::gpu::PhysicsDevice;
use seiche::{LaggedStats, Physics};

use super::Canvas;

/// One device for a host, from the same wgpu handles its renderer uses.
pub fn physics_device_for(handles: &netrender::WgpuHandles) -> PhysicsDevice {
    PhysicsDevice::from_wgpu(
        handles.instance.clone(),
        handles.adapter.clone(),
        handles.device.clone(),
        handles.queue.clone(),
    )
}

/// Install a lane from `device` on `physics`, or clear it.
pub(crate) fn install(physics: &mut Physics, device: Option<&PhysicsDevice>) {
    match device {
        Some(device) => physics.set_lagged_repulsion(
            Some(Box::new(device.repulsion())),
            device.threshold(),
            device.max_stale_steps(),
        ),
        None => physics.set_lagged_repulsion(None, 0, 1),
    }
}

impl Canvas {
    /// Stage the canvas's repulsion on the host's device, or (`None`) return
    /// it to the CPU. Positions are untouched; the next tick uses the lane.
    pub fn set_physics_device(&mut self, device: Option<PhysicsDevice>) {
        install(&mut self.physics, device.as_ref());
        self.physics_device = device;
    }

    pub fn physics_device(&self) -> Option<&PhysicsDevice> {
        self.physics_device.as_ref()
    }

    /// The lane's counts (inline backend; offloaded, read the device's own
    /// counters).
    pub fn repulsion_stats(&self) -> Option<LaggedStats> {
        self.physics.repulsion_stats()
    }
}
