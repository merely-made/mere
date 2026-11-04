// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Canvas {
    /// A drag ended on an anchored item with physics paused: it jumps back to
    /// its position (F19).
    pub(crate) fn jump_home(&mut self, key: NodeKey) {
        let Some(home) = self.arrangement_slot(key) else {
            return;
        };
        self.view.set_position(key, home);
        if let Some(positions) = self.paused_positions.as_mut()
            && let Some((_, at)) = positions.iter_mut().find(|(k, _)| *k == key)
        {
            *at = home;
        }
        self.reset_frame_time();
        self.physics.seed(vec![(key, home)]);
    }

    /// A pick while playing pauses for its transition and resumes when the
    /// final placement lands (F24). Called right after the pick's pause.
    pub(crate) fn note_pick(&mut self, was_playing: bool) {
        self.roles.resume_after_pick = was_playing;
    }

    /// Any explicit pause or play cancels a pick's pending resume.
    pub(crate) fn cancel_pick_resume(&mut self) {
        self.roles.resume_after_pick = false;
    }

    /// Called when a final placement lands: resume a pick made while playing.
    pub(crate) fn land_pick(&mut self) {
        if std::mem::take(&mut self.roles.resume_after_pick) {
            self.set_physics_paused(false);
        }
    }

    /// The placement a stop leaves (F24): every item where motion left it,
    /// except anchored items, which go back to their positions.
    pub(crate) fn stop_placement(
        &mut self,
        from: &[(NodeKey, PortablePoint)],
    ) -> Vec<(NodeKey, PortablePoint)> {
        let mut moved = false;
        let roles = self.roles_of(from.iter().map(|(key, _)| *key));
        let to: Vec<_> = from
            .iter()
            .map(|&(key, at)| {
                if roles[&key] == Role::Anchored
                    && let Some(home) = self.arrangement_slot(key)
                {
                    moved |= home != at;
                    (key, home)
                } else {
                    (key, at)
                }
            })
            .collect();
        self.roles.stop_return = moved.then(|| StopReturn {
            source: super::super::host_results::LayoutResultSource::capture(self),
            from: from.to_vec(),
            to: to.clone(),
            duration_ms: None,
        });
        to
    }

    /// Queue a return for the host to animate, replacing any pending one.
    pub(crate) fn queue_return(&mut self, ret: StopReturn) {
        self.roles.stop_return = Some(ret);
    }

    /// Whether a stop's return waits for a host to animate it.
    pub fn has_stop_return(&self) -> bool {
        self.roles.stop_return.is_some()
    }

    /// The last stop's return, for a host that animates it. Taking it clears
    /// it; the canvas has already placed its `to`.
    pub fn take_stop_return(&mut self) -> Option<StopReturn> {
        self.roles.stop_return.take()
    }

    /// Show an in-between paused placement without seeding physics: a
    /// host-clocked stop transition's sampled frames. A no-op while playing.
    pub fn preview_paused_positions(&mut self, positions: &[(NodeKey, PortablePoint)]) {
        if self.physics_paused {
            self.paused_positions = Some(positions.to_vec());
        }
    }

    /// Land a paused placement: shown and seeded, so play starts from it.
    pub fn land_paused_positions(&mut self, positions: &[(NodeKey, PortablePoint)]) {
        if !self.physics_paused {
            return;
        }
        self.paused_positions = Some(positions.to_vec());
        self.apply_strategy_to_view();
        self.reset_frame_time();
        self.physics.seed(positions.to_vec());
    }

    /// The latest settle's positions (F30), or `None` before the first.
    pub fn settled_positions(&self) -> Option<&[(NodeKey, PortablePoint)]> {
        self.roles.settled.as_deref()
    }

    /// How many settles have been recorded. Receipt introspection.
    pub fn settle_count(&self) -> u64 {
        self.roles.settles
    }

    /// The roles' share of a frame: note a settle, then glide anchored items
    /// home. Called right after the physics snapshot lands in the view.
    pub(crate) fn advance_roles(&mut self) {
        self.note_settle();
        self.advance_home();
        self.advance_schedule();
    }

    /// Fold one frame into the settle record: a playing graph whose bodies'
    /// rms speed falls under the floor has settled (F46). Once per rest, the
    /// rest replaces Settled and anchored items start home (F45). A law that
    /// never rests does neither.
    pub(super) fn note_settle(&mut self) {
        let budget_ended = self.roles.rest.budget_ended(self.physics.is_settling());
        if self.physics_paused {
            return;
        }
        if self.drag.is_some_and(|d| d.moved) {
            self.roles.rest.arm();
            return;
        }
        if !self.roles.rest.rested(self.physics.rms_speed()) || self.physics_never_rests() {
            // A budget spent with the bodies still moving is no settle, but
            // anchored items still go home (F63).
            if budget_ended {
                self.start_home();
            }
            return;
        }
        let positions: Vec<_> = self.view.positions().collect();
        self.roles.settles += 1;
        if self.active_strategy.as_deref() == Some(SETTLED_ARRANGEMENT) {
            self.strategy_positions = Some(positions.clone());
            self.sync_arrangement_roles();
        }
        self.roles.settled = Some(positions);
        self.start_home();
    }

    /// At rest, every anchored item away from its position starts home.
    pub(super) fn start_home(&mut self) {
        let roles = self.roles_of(self.view.positions().map(|(key, _)| key));
        let glide: Vec<_> = self
            .view
            .positions()
            .filter(|&(key, _)| {
                !self.pinned_nodes.contains(&key) && roles.get(&key) == Some(&Role::Anchored)
            })
            .filter_map(|(key, at)| Some((key, at, self.arrangement_slot(key)?)))
            .collect();
        self.roles.rest.start(&mut self.physics, glide);
    }

    /// One frame of the glide home; at its end the items hold at home.
    pub(super) fn advance_home(&mut self) {
        self.roles.rest.step(&mut self.physics);
    }

    /// A disturbance (a settle request, a drag, a pause or play) releases
    /// the items held at home and stops a glide, so the spring acts again.
    pub(crate) fn unpark(&mut self) {
        let roles = self.roles_of(self.view.positions().map(|(key, _)| key));
        let keep = |key: NodeKey| roles.get(&key) == Some(&Role::Pinned);
        self.roles.rest.release(&mut self.physics, keep);
    }

    /// How many anchored items are held at home after an at-rest return.
    /// Receipt introspection.
    pub fn anchored_home_count(&self) -> usize {
        self.roles.rest.parked_count()
    }

    pub(crate) fn returning_home(&self) -> bool {
        self.roles.rest.gliding()
    }

    /// A fresh bounded recipe may begin already at rest (Still, empty or
    /// fully pinned). Its first fixed tick must be eligible to record that.
    pub(crate) fn arm_settle_detection(&mut self) {
        self.roles.rest.arm();
    }

    /// The frame's layout stage alone, without composing a scene: the
    /// physics snapshot, the paused placement, and the roles. For receipts
    /// that run thousands of frames.
    pub(crate) fn step_layout(&mut self) {
        self.physics.advance_frame(&mut self.view);
        self.apply_strategy_to_view();
        self.advance_roles();
    }

    /// A bounded preview cannot infer a position-writing kernel's rest from
    /// body velocity. Explicit schedules still advance on every fixed tick.
    pub(crate) fn step_dynamics_layout(&mut self, velocity_rest: bool) {
        if velocity_rest {
            self.step_layout();
        } else {
            self.physics.advance_frame(&mut self.view);
            self.apply_strategy_to_view();
            self.advance_home();
            self.advance_schedule();
        }
    }

    /// A placement applied while Settled is active is Settled's own (a
    /// reopened scene).
    pub(crate) fn adopt_settled(&mut self, positions: &[(NodeKey, PortablePoint)]) {
        if self.active_strategy.as_deref() == Some(SETTLED_ARRANGEMENT) {
            self.roles.settled = Some(positions.to_vec());
        }
    }
}
