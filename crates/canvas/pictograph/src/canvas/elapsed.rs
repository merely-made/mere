// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::{
    Canvas, CanvasFrameProfile, ElapsedStepConfig, ElapsedStepReport, PaceStats, Speed, StepBudget,
};
use netrender::Scene;
use std::time::Duration;

impl Canvas {
    /// Forget host time and fractional physics debt after suspension or reseeding.
    /// Positions and the simulation's playing state are preserved.
    pub fn reset_frame_time(&mut self) {
        self.frame_timestamp = None;
        self.elapsed_step = None;
        self.physics.reset_elapsed();
    }

    /// The last timed call's work, or None after reset/deterministic advancement.
    pub fn elapsed_step_report(&self) -> Option<ElapsedStepReport> {
        self.elapsed_step
    }

    /// Set the simulation speed, 0.2x to 50x: ticks per frame at the fixed
    /// step, so the layout's trajectory does not depend on it.
    pub fn set_physics_speed(&mut self, speed: Speed) {
        self.physics.set_speed(speed);
    }

    pub fn physics_speed(&self) -> Speed {
        self.physics.speed()
    }

    /// Bound a frame's ticks above real time on the host's clock; `None`
    /// leaves fast-forward bounded only by the step and elapsed caps.
    pub fn set_physics_step_budget(&mut self, budget: Option<StepBudget>) {
        self.physics.set_step_budget(budget);
    }

    /// Budget fast-forward to half the frame of a display refreshing at
    /// `millihertz`, as winit's `MonitorHandle::refresh_rate_millihertz`
    /// reports it, or 60 Hz's when the rate is unknown, on the native
    /// monotonic clock (ruled 2026-10-04, "Mere entry point, then
    /// turnstone"). A browser host infers its period instead.
    #[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
    pub fn set_physics_display_rate(&mut self, millihertz: Option<u32>) {
        self.physics
            .set_step_budget(Some(StepBudget::for_display(millihertz)));
    }

    /// The step budget last set, as given.
    pub fn physics_step_budget(&self) -> Option<StepBudget> {
        self.physics.step_budget()
    }

    /// Ticks run, the effective speed reached, and whether the budget bound.
    pub fn physics_pace(&self) -> PaceStats {
        self.physics.pace()
    }

    fn elapsed_since(&mut self, timestamp: Duration) -> Duration {
        let previous = if self.elapsed_step.is_some_and(|report| !report.settling) {
            // No frames may have been requested while idle. A new nudge/drag
            // starts now; the quiet interval does not belong to that motion.
            self.physics.reset_elapsed();
            self.frame_timestamp.unwrap_or(timestamp).max(timestamp)
        } else {
            self.frame_timestamp.unwrap_or(timestamp)
        };
        // Immediate input draws may be newer than the next animation callback's
        // timestamp. Never move the baseline backwards or count that time twice.
        self.frame_timestamp = Some(previous.max(timestamp));
        timestamp.saturating_sub(previous)
    }

    /// Advance bounded physics from host monotonic time, then compose once.
    /// The first frame after a reset establishes a baseline without ticking.
    pub fn frame_at(
        &mut self,
        w: u32,
        h: u32,
        timestamp: Duration,
        config: ElapsedStepConfig,
    ) -> (Scene, bool) {
        let elapsed = self.elapsed_since(timestamp);
        self.frame_observed(
            w,
            h,
            Some((elapsed, config)),
            &mut super::frame_profile::Unprofiled,
        )
    }

    /// Timed advancement with the same optional CPU attribution as frame_profiled.
    pub fn frame_profiled_at(
        &mut self,
        w: u32,
        h: u32,
        timestamp: Duration,
        config: ElapsedStepConfig,
        now_ms: impl FnMut() -> f64,
    ) -> (Scene, bool, CanvasFrameProfile) {
        let elapsed = self.elapsed_since(timestamp);
        let mut observer = super::frame_profile::Profiled::new(now_ms);
        let (scene, moving) = self.frame_observed(w, h, Some((elapsed, config)), &mut observer);
        (scene, moving, observer.profile)
    }
}
