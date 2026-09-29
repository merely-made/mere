// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Optional CPU attribution for a single canvas frame, using a host clock.

/// Nonoverlapping CPU intervals within [`super::Canvas::frame_profiled`].
///
/// These exclude GPU rasterization. `stage_ms` follows [`Self::STAGES`]; its sum
/// is the observed total, including the small cost of recording checkpoints.
#[derive(Clone, Debug, Default)]
pub struct CanvasFrameProfile {
    pub stage_ms: [f64; 10],
    pub total_nodes: usize,
    pub visible_nodes: usize,
    /// All layer commands before viewport culling.
    pub paint_commands_before_cull: usize,
    /// All layer commands actually supplied to scene lowering.
    pub paint_commands_after_cull: usize,
}

impl CanvasFrameProfile {
    pub const STAGES: [&'static str; 10] = [
        "physics",
        "prepare",
        "underlay",
        "dom_prepare",
        "dom_mutate",
        "dom_frame",
        "faces",
        "overlays",
        "cull",
        "lower",
    ];

    pub fn total_ms(&self) -> f64 {
        self.stage_ms.iter().sum()
    }
}

pub(super) trait Observer {
    fn mark(&mut self, stage: usize);
    fn counts(&mut self, total: usize, visible: usize, before: usize, after: usize);
}

pub(super) struct Unprofiled;

impl Observer for Unprofiled {
    #[inline(always)]
    fn mark(&mut self, _: usize) {}
    #[inline(always)]
    fn counts(&mut self, _: usize, _: usize, _: usize, _: usize) {}
}

pub(super) struct Profiled<F> {
    clock: F,
    previous: f64,
    pub profile: CanvasFrameProfile,
}

impl<F: FnMut() -> f64> Profiled<F> {
    pub fn new(mut clock: F) -> Self {
        let previous = clock();
        Self {
            clock,
            previous,
            profile: CanvasFrameProfile::default(),
        }
    }
}

impl<F: FnMut() -> f64> Observer for Profiled<F> {
    fn mark(&mut self, stage: usize) {
        let now = (self.clock)();
        self.profile.stage_ms[stage] += (now - self.previous).max(0.0);
        self.previous = now;
    }

    fn counts(&mut self, total: usize, visible: usize, before: usize, after: usize) {
        self.profile.total_nodes = total;
        self.profile.visible_nodes = visible;
        self.profile.paint_commands_before_cull = before;
        self.profile.paint_commands_after_cull = after;
    }
}
