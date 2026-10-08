// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The settle and the at-rest return, shared by the canvas and the board
//! (dynamics grammar plan, G7).
//!
//! A settle is the bodies' rms speed falling under [`SETTLE_SPEED_FLOOR`]
//! (F46). An anchored item returns by its spring while things move; once
//! they rest, a short glide takes it exactly to its position, as a kinematic
//! body so its neighbours respond, and it holds there until the next
//! disturbance releases it to the spring again (F45, "Spring, then home").
//! A settle budget that ends while the bodies still move starts the same
//! glide, so an anchored item always finishes at its position (F63, "Home
//! at budget end too"); only the speed floor counts as a settle.

use std::collections::HashSet;

use euclid::default::Point2D;
use kernel::graph::NodeKey;
use seiche::Physics;

/// The bodies' rms speed, in world units (pixels at zoom 1) a second, below
/// which moving bodies have settled (F46). F10's energy floor of 1 remains
/// the measure of whether a law never rests.
pub const SETTLE_SPEED_FLOOR: f32 = 1.0;

/// Frames the at-rest return takes (F45): a short eased glide home.
pub const HOME_FRAMES: u32 = 18;

/// One settle record and return in progress. See the module docs.
#[derive(Clone, Debug, Default)]
pub(crate) struct AtRest {
    /// Set while the bodies move, so a rest is noted once.
    armed: bool,
    /// Each gliding item: from where it rested, to its position.
    glide: Vec<(NodeKey, Point2D<f32>, Point2D<f32>)>,
    frame: u32,
    /// Items held at home after a return, until a disturbance.
    parked: HashSet<NodeKey>,
    /// Whether the backend was stepping last frame, so a budget's end is
    /// noted once (F63).
    stepping: bool,
}

impl AtRest {
    /// Fold one frame's stepping state in: true on the frame a settle
    /// budget runs out (F63).
    pub(crate) fn budget_ended(&mut self, stepping: bool) -> bool {
        let ended = self.stepping && !stepping;
        self.stepping = stepping;
        ended
    }

    /// Fold one frame's rms speed in: true on the frame the bodies come to
    /// rest after moving, once per rest.
    pub(crate) fn rested(&mut self, speed: f32) -> bool {
        if speed >= SETTLE_SPEED_FLOOR {
            self.armed = true;
            return false;
        }
        std::mem::take(&mut self.armed)
    }

    /// Note motion without reading a speed (a drag), so the next rest counts.
    pub(crate) fn arm(&mut self) {
        self.armed = true;
    }

    /// How many items are held at home.
    pub(crate) fn parked_count(&self) -> usize {
        self.parked.len()
    }

    /// Whether a glide is under way.
    pub(crate) fn gliding(&self) -> bool {
        !self.glide.is_empty()
    }

    /// Start the glide home for `glide` (`(item, at, home)`; items already
    /// home are dropped), keeping the backend ticking through it.
    pub(crate) fn start(
        &mut self,
        physics: &mut Physics,
        glide: impl IntoIterator<Item = (NodeKey, Point2D<f32>, Point2D<f32>)>,
    ) {
        self.glide = glide
            .into_iter()
            .filter(|(key, at, home)| at != home && !self.parked.contains(key))
            .collect();
        self.frame = 0;
        if self.gliding() && !physics.is_settling() {
            physics.settle(HOME_FRAMES + 2);
        }
    }

    /// One frame of the glide. Its last pin lands at the next physics step,
    /// so the items count as home one frame after it.
    pub(crate) fn step(&mut self, physics: &mut Physics) {
        if !self.gliding() {
            return;
        }
        self.frame += 1;
        if self.frame > HOME_FRAMES {
            self.parked
                .extend(self.glide.drain(..).map(|(key, _, _)| key));
            return;
        }
        let t = self.frame as f32 / HOME_FRAMES as f32;
        let eased = t * t * (3.0 - 2.0 * t);
        for &(key, from, to) in &self.glide {
            physics.pin(key, from.lerp(to, eased));
        }
    }

    /// A disturbance: stop any glide and release the held items to the
    /// spring, except those `keep` says stay kinematic.
    pub(crate) fn release(&mut self, physics: &mut Physics, keep: impl Fn(NodeKey) -> bool) {
        let gliding = self.glide.drain(..).map(|(key, _, _)| key);
        for key in gliding.chain(self.parked.drain()) {
            if !keep(key) {
                physics.unpin(key);
            }
        }
    }
}
