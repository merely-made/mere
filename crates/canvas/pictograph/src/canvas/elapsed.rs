// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{Canvas, CanvasFrameProfile, ElapsedStepConfig, ElapsedStepReport};
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
