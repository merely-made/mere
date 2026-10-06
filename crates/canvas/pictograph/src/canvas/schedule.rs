// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Schedules: a sequence of compositions, each run until its stop condition
//! (dynamics grammar plan, G3; "Sequenced blends", P7e).
//!
//! A stage names a law, its overlays and optionally a composition, and how
//! it ends: when the bodies rest (F46's speed floor, the settle every
//! other G7 reading keys to), after a number of frames, or when the stage's
//! law stops by its own test (Density's ruled "Shift < 0.05 for 3 passes"
//! is the precedent). A stage may capture its layout as it ends: the layout
//! becomes the arrangement, taken by the next stage with a role (F27,
//! "Capture takes a role"), seeded unless the stage says otherwise (F23).
//! "Stress, then Springs, remembering it" is Stress's capture taken as
//! anchored, so Springs relaxes locally and its anchored items come home
//! (F45) to the metric map.

use super::composition::PhysicsComposition;
use super::physics_catalog::{PhysicsLaw, PhysicsOverlay};
use super::*;
use seiche::Role;

/// The arrangement id of a schedule's capture. Not a cartography adapter:
/// the canvas supplies its positions, as it does Settled's.
pub const CAPTURED_ARRANGEMENT: &str = "captured";

/// How a stage ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StageStop {
    /// The bodies rest: their rms speed falls under the settle floor (F46).
    Rest,
    /// After this many frames.
    Frames(u32),
    /// The stage's law stops by its own test (Density's passes end), or,
    /// for a law with no stop of its own, the bodies rest. An offloaded host
    /// does not report a law's own demand, so there it is the rest.
    LawDone,
}

/// One stage of a schedule.
#[derive(Clone, Debug, PartialEq)]
pub struct PhysicsStage {
    pub law: PhysicsLaw,
    pub overlays: Vec<PhysicsOverlay>,
    /// Run in the law slot instead of `law`, if set.
    pub composition: Option<PhysicsComposition>,
    pub stop: StageStop,
    /// Capture the layout as the stage ends, taken with this role.
    pub capture: Option<Role>,
}

impl PhysicsStage {
    pub fn law(law: PhysicsLaw, stop: StageStop) -> Self {
        Self {
            law,
            overlays: Vec::new(),
            composition: None,
            stop,
            capture: None,
        }
    }

    pub fn capturing(mut self, role: Role) -> Self {
        self.capture = Some(role);
        self
    }
}

/// A schedule under way.
#[derive(Clone, Debug)]
pub(crate) struct ScheduleRun {
    stages: Vec<PhysicsStage>,
    stage: usize,
    frames: u32,
    settles_at_start: u64,
    /// Whether the world wanted ticks of its own last frame, and at any
    /// frame of the stage, so a law's own stop is noted when it ends.
    wanted: bool,
    ever_wanted: bool,
}

impl Canvas {
    /// Run `stages` in order from the current layout, playing. An empty
    /// schedule clears any schedule under way.
    pub fn run_physics_schedule(&mut self, stages: Vec<PhysicsStage>) {
        if stages.is_empty() {
            self.schedule = None;
            return;
        }
        self.schedule = Some(ScheduleRun {
            stages,
            stage: 0,
            frames: 0,
            settles_at_start: self.settle_count(),
            wanted: false,
            ever_wanted: false,
        });
        self.enter_stage(0);
        self.set_physics_paused(false);
    }

    /// The stage under way, or `None` once the schedule ends or with none.
    pub fn physics_schedule_stage(&self) -> Option<usize> {
        self.schedule.as_ref().map(|run| run.stage)
    }

    /// The last capture's positions, if a schedule captured one.
    pub fn captured_positions(&self) -> Option<&[(NodeKey, PortablePoint)]> {
        (self.active_strategy.as_deref() == Some(CAPTURED_ARRANGEMENT))
            .then(|| self.strategy_positions.as_deref())
            .flatten()
    }

    fn enter_stage(&mut self, index: usize) {
        let Some(stage) = self
            .schedule
            .as_ref()
            .and_then(|run| run.stages.get(index))
            .cloned()
        else {
            return;
        };
        self.physics_law = stage.law;
        self.physics_overlays = stage.overlays;
        self.physics_composition = stage.composition;
        self.rebuild_law_forces();
        self.settle_physics(u32::MAX);
    }

    /// One frame of the schedule: count it, and at the stage's stop capture
    /// if asked and enter the next stage. Called with the roles' share of a
    /// frame, after a settle is noted.
    pub(crate) fn advance_schedule(&mut self) {
        let settles = self.settle_count();
        let wanted = self.physics.tick_demand().0;
        let Some(run) = self.schedule.as_mut() else {
            return;
        };
        if self.physics_paused {
            return;
        }
        run.frames += 1;
        let stage = &run.stages[run.stage];
        let rested = settles > run.settles_at_start;
        let done = match stage.stop {
            StageStop::Rest => rested,
            StageStop::Frames(n) => run.frames >= n,
            StageStop::LawDone => (run.wanted && !wanted) || (!run.ever_wanted && rested),
        };
        run.wanted = wanted;
        run.ever_wanted |= wanted;
        if !done {
            return;
        }
        let capture = stage.capture;
        let next = run.stage + 1;
        if let Some(role) = capture {
            self.capture_layout(role);
        }
        let Some(run) = self.schedule.as_mut() else {
            return;
        };
        if next >= run.stages.len() {
            self.schedule = None;
            return;
        }
        run.stage = next;
        run.frames = 0;
        run.settles_at_start = settles;
        run.wanted = false;
        run.ever_wanted = false;
        self.enter_stage(next);
    }

    /// The layout as it stands becomes the arrangement, its items taking
    /// `role` unless a group or an item says otherwise (F22, F27).
    fn capture_layout(&mut self, role: Role) {
        let positions: Vec<(NodeKey, PortablePoint)> = self.view.positions().collect();
        self.active_strategy = Some(CAPTURED_ARRANGEMENT.to_string());
        self.strategy_positions = Some(positions);
        self.roles.table.default = role;
        self.sync_arrangement_roles();
    }
}
