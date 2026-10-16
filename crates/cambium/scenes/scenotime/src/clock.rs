// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The host's elapsed clock for one transition.
//!
//! A schedule is sampled at an elapsed time the host supplies (see
//! [`crate::TransitionSchedule::sample_at`]); the host also owns whether that
//! clock advances. This is the small piece every host of a transition
//! otherwise rewrites: it turns frame timestamps into elapsed time that
//! pausing does not charge.

/// The host-owned elapsed clock for one projection transition.
///
/// Browser frame timestamps enter through [`observe`](Self::observe). Pausing
/// keeps observing the host so resuming cannot charge the paused interval to
/// the transition. Scenotime sees only the resulting elapsed value.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectionClock {
    elapsed_ms: f32,
    last_host_ms: Option<f64>,
    paused: bool,
}

impl ProjectionClock {
    pub fn observe(&mut self, host_ms: f64) -> f32 {
        if !host_ms.is_finite() {
            return self.elapsed_ms;
        }
        let previous = self.last_host_ms.replace(host_ms);
        if !self.paused
            && let Some(previous) = previous
        {
            self.elapsed_ms += (host_ms - previous).max(0.0) as f32;
        }
        self.elapsed_ms
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn elapsed_ms(&self) -> f32 {
        self.elapsed_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elapsed_time_follows_the_host_and_skips_a_pause() {
        let mut clock = ProjectionClock::default();
        assert_eq!(
            clock.observe(1000.0),
            0.0,
            "the first frame starts the clock"
        );
        assert_eq!(clock.observe(1016.0), 16.0);
        clock.set_paused(true);
        assert!(clock.is_paused());
        assert_eq!(
            clock.observe(5000.0),
            16.0,
            "a paused interval is not charged"
        );
        clock.set_paused(false);
        assert_eq!(
            clock.observe(5016.0),
            32.0,
            "resuming charges only what follows"
        );
        assert_eq!(clock.elapsed_ms(), 32.0);
    }

    #[test]
    fn a_backward_or_non_finite_timestamp_adds_nothing() {
        let mut clock = ProjectionClock::default();
        clock.observe(100.0);
        clock.observe(110.0);
        assert_eq!(clock.observe(105.0), 10.0, "time never runs backward");
        assert_eq!(clock.observe(f64::NAN), 10.0);
        assert_eq!(clock.observe(f64::INFINITY), 10.0);
    }
}
