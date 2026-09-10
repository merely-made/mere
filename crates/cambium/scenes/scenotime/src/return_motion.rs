// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A small host-clocked spring for a transient visual offset.
//!
//! This deliberately does not move a scene item, its footprint, or any source
//! coordinates. A consuming projection applies [`ReturnMotion::offset`] to a
//! label, callout, or other local visual and asks [`ReturnMotion::tick`] whether
//! it needs another frame. It is the lightweight counterpart to seiche's
//! arrangement springs when one visual needs to return without a physics world.

/// Tuning for one [`ReturnMotion`]. Values are in the caller's coordinate unit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReturnMotionSettings {
    /// Pull back toward the zero (home) offset, in acceleration per offset.
    pub strength: f32,
    /// Velocity retained by one nominal 60 Hz frame. `0` is critically abrupt;
    /// values near one preserve more of the drag's release velocity.
    pub damping: f32,
    /// Distance below which home is exact, avoiding forever-active tiny frames.
    pub settle_distance: f32,
    /// Speed below which home is exact.
    pub settle_speed: f32,
}

impl Default for ReturnMotionSettings {
    fn default() -> Self {
        Self {
            strength: 18.0,
            damping: 0.78,
            settle_distance: 0.0005,
            settle_speed: 0.0005,
        }
    }
}

impl ReturnMotionSettings {
    fn sanitized(self) -> Self {
        Self {
            strength: finite_or(self.strength, Self::default().strength).max(0.0),
            damping: finite_or(self.damping, Self::default().damping).clamp(0.0, 1.0),
            settle_distance: finite_or(self.settle_distance, Self::default().settle_distance)
                .max(0.0),
            settle_speed: finite_or(self.settle_speed, Self::default().settle_speed).max(0.0),
        }
    }
}

fn finite_or(value: f32, fallback: f32) -> f32 {
    value.is_finite().then_some(value).unwrap_or(fallback)
}

/// The projection reading of a local visual offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnMotionMode {
    /// At the immutable source position with no local offset.
    Home,
    /// A temporary free offset, returning to home after the pointer releases.
    Free,
    /// A deliberate local override which holds until explicitly unpinned.
    Pinned,
}

/// Frame result returned by [`ReturnMotion::tick`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReturnMotionTick {
    /// The displayed offset changed this frame.
    pub changed: bool,
    /// The host should schedule another frame (a drag or an unsettled return).
    pub active: bool,
}

/// One local offset that can be dragged, pinned, and returned home.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReturnMotion {
    settings: ReturnMotionSettings,
    offset: (f32, f32),
    velocity: (f32, f32),
    mode: ReturnMotionMode,
    dragging: bool,
}

impl Default for ReturnMotion {
    fn default() -> Self {
        Self::new(ReturnMotionSettings::default())
    }
}

impl ReturnMotion {
    pub fn new(settings: ReturnMotionSettings) -> Self {
        Self {
            settings: settings.sanitized(),
            offset: (0.0, 0.0),
            velocity: (0.0, 0.0),
            mode: ReturnMotionMode::Home,
            dragging: false,
        }
    }

    pub const fn settings(&self) -> ReturnMotionSettings {
        self.settings
    }

    pub fn set_settings(&mut self, settings: ReturnMotionSettings) {
        self.settings = settings.sanitized();
    }

    pub const fn offset(&self) -> (f32, f32) {
        self.offset
    }

    pub const fn mode(&self) -> ReturnMotionMode {
        self.mode
    }

    pub const fn is_dragging(&self) -> bool {
        self.dragging
    }

    /// Start a transient drag. A pinned local placement is made free first.
    pub fn begin(&mut self) -> bool {
        let changed = self.mode != ReturnMotionMode::Free || !self.dragging;
        self.mode = ReturnMotionMode::Free;
        self.dragging = true;
        self.velocity = (0.0, 0.0);
        changed
    }

    /// Place the visual at an explicit local offset while dragging.
    pub fn move_to(&mut self, offset: (f32, f32)) -> bool {
        if !self.dragging || !offset.0.is_finite() || !offset.1.is_finite() {
            return false;
        }
        let changed = self.offset != offset;
        self.offset = offset;
        self.velocity = (0.0, 0.0);
        changed
    }

    /// Release the free visual. Its next ticks return it to home.
    pub fn end(&mut self) -> bool {
        if !self.dragging {
            return false;
        }
        self.dragging = false;
        true
    }

    /// Hold the current local offset until [`Self::unpin`] or another drag.
    pub fn pin(&mut self) -> bool {
        let changed = self.mode != ReturnMotionMode::Pinned || self.dragging;
        self.mode = ReturnMotionMode::Pinned;
        self.dragging = false;
        self.velocity = (0.0, 0.0);
        changed
    }

    /// Make a pinned offset free again so it returns home.
    pub fn unpin(&mut self) -> bool {
        if self.mode != ReturnMotionMode::Pinned {
            return false;
        }
        self.mode = ReturnMotionMode::Free;
        true
    }

    /// Advance by host-supplied seconds. Reduced motion reaches home in one
    /// call, so the host can go idle immediately.
    pub fn tick(&mut self, dt: f32, reduced_motion: bool) -> ReturnMotionTick {
        if self.dragging {
            return ReturnMotionTick {
                changed: false,
                active: true,
            };
        }
        if self.mode != ReturnMotionMode::Free {
            return ReturnMotionTick {
                changed: false,
                active: false,
            };
        }
        if reduced_motion {
            return self.settle();
        }
        if self.settings.strength == 0.0 || !dt.is_finite() {
            // A zero pull is an intentional free placement. It must not spin
            // the host forever, and an invalid clock sample must not put NaNs
            // into a projection which has no authority to repair them.
            return ReturnMotionTick {
                changed: false,
                active: false,
            };
        }
        let dt = dt.clamp(0.0, 0.1);
        if dt == 0.0 {
            return ReturnMotionTick {
                changed: false,
                active: true,
            };
        }
        let frames = dt * 60.0;
        let damping = self.settings.damping.powf(frames);
        // Damping attenuates the velocity that already exists; the spring's
        // current pull remains effective even at damping zero.
        self.velocity.0 = self.velocity.0 * damping - self.offset.0 * self.settings.strength * dt;
        self.velocity.1 = self.velocity.1 * damping - self.offset.1 * self.settings.strength * dt;
        let before = self.offset;
        self.offset.0 += self.velocity.0 * dt;
        self.offset.1 += self.velocity.1 * dt;
        if self.offset.0.hypot(self.offset.1) <= self.settings.settle_distance
            && self.velocity.0.hypot(self.velocity.1) <= self.settings.settle_speed
        {
            return self.settle();
        }
        ReturnMotionTick {
            changed: self.offset != before,
            active: true,
        }
    }

    fn settle(&mut self) -> ReturnMotionTick {
        let changed = self.offset != (0.0, 0.0)
            || self.velocity != (0.0, 0.0)
            || self.mode != ReturnMotionMode::Home;
        self.offset = (0.0, 0.0);
        self.velocity = (0.0, 0.0);
        self.mode = ReturnMotionMode::Home;
        ReturnMotionTick {
            changed,
            active: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn released_motion() -> ReturnMotion {
        let mut motion = ReturnMotion::default();
        assert!(motion.begin());
        assert!(motion.move_to((0.5, -0.25)));
        assert!(motion.end());
        motion
    }

    #[test]
    fn identical_clocks_are_deterministic() {
        let mut left = released_motion();
        let mut right = released_motion();
        for _ in 0..120 {
            assert_eq!(left.tick(1.0 / 60.0, false), right.tick(1.0 / 60.0, false));
            assert_eq!(left, right);
        }
    }

    #[test]
    fn released_offset_settles_and_goes_idle() {
        let mut motion = released_motion();
        let mut tick = ReturnMotionTick {
            changed: false,
            active: true,
        };
        for _ in 0..2_000 {
            tick = motion.tick(1.0 / 60.0, false);
            if !tick.active {
                break;
            }
        }
        assert!(!tick.active);
        assert_eq!(motion.mode(), ReturnMotionMode::Home);
        assert_eq!(motion.offset(), (0.0, 0.0));
    }

    #[test]
    fn pin_holds_until_unpinned() {
        let mut motion = released_motion();
        assert!(motion.pin());
        let pinned = motion.offset();
        assert_eq!(
            motion.tick(1.0, false),
            ReturnMotionTick {
                changed: false,
                active: false
            }
        );
        assert_eq!(motion.offset(), pinned);
        assert!(motion.unpin());
        assert!(motion.tick(1.0 / 60.0, false).active);
    }

    #[test]
    fn reduced_motion_returns_home_in_one_tick() {
        let mut motion = released_motion();
        assert_eq!(
            motion.tick(1.0 / 60.0, true),
            ReturnMotionTick {
                changed: true,
                active: false
            }
        );
        assert_eq!(motion.mode(), ReturnMotionMode::Home);
        assert_eq!(motion.offset(), (0.0, 0.0));
    }

    #[test]
    fn nonfinite_tuning_and_clock_cannot_create_a_busy_loop() {
        let mut motion = ReturnMotion::new(ReturnMotionSettings {
            strength: f32::NAN,
            damping: f32::INFINITY,
            settle_distance: f32::NEG_INFINITY,
            settle_speed: f32::NAN,
        });
        assert_eq!(motion.settings(), ReturnMotionSettings::default());
        motion.begin();
        motion.move_to((0.25, 0.0));
        motion.end();
        assert_eq!(
            motion.tick(f32::NAN, false),
            ReturnMotionTick {
                changed: false,
                active: false
            }
        );
        let mut disabled = ReturnMotion::new(ReturnMotionSettings {
            strength: 0.0,
            ..ReturnMotionSettings::default()
        });
        disabled.begin();
        disabled.move_to((0.25, 0.0));
        disabled.end();
        assert!(!disabled.tick(1.0 / 60.0, false).active);
    }

    #[test]
    fn variable_finite_steps_still_settle() {
        let mut motion = released_motion();
        let mut active = true;
        for dt in [0.004, 0.020, 0.013, 0.050, 0.008]
            .into_iter()
            .cycle()
            .take(2_000)
        {
            active = motion.tick(dt, false).active;
            if !active {
                break;
            }
        }
        assert!(!active);
        assert_eq!(motion.offset(), (0.0, 0.0));
    }
}
