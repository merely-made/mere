// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The suite's physics edits and reads, made as a host makes them: through
//! the canvas's spec and its flat view (dynamics grammar plan, F162). A pick
//! reads the record, rewrites its root or slots through [`PhysicsChoice`],
//! and sets it back; a read is the live view.

use crate::canvas::composition::{CompositionRefusal, GroupSource, PhysicsComposition};
use crate::canvas::dynamics_spec::{DynamicsSpec, running_node, schedule_node};
use crate::canvas::physics_catalog::{
    OverlayRefusal, PhysicsDepthSource, PhysicsKindSource, PhysicsLaw, PhysicsMassSource,
    PhysicsOverlay,
};
use crate::canvas::schedule::PhysicsStage;
use crate::canvas::{Canvas, PhysicsChoice};

pub(crate) trait ThroughView {
    /// What the canvas runs now, as the flat view.
    fn view(&self) -> PhysicsChoice;
    /// The record to edit.
    fn record(&self) -> DynamicsSpec;
    /// Apply `choice` as a picker does: the overlays the law refuses dropped
    /// and returned with the law's reason, the rest set through the spec.
    fn pick(&mut self, choice: &PhysicsChoice) -> Result<(), OverlayRefusal>;
    fn pick_law(&mut self, law: PhysicsLaw) -> Result<(), OverlayRefusal> {
        let choice = PhysicsChoice { law, ..self.view() };
        self.pick(&choice)
    }
    fn pick_overlays(&mut self, overlays: Vec<PhysicsOverlay>) -> Result<(), OverlayRefusal> {
        let choice = PhysicsChoice {
            overlays,
            ..self.view()
        };
        self.pick(&choice)
    }
    /// Toggle one overlay; whether it is now on (off for one the law refuses).
    fn toggle_overlay(&mut self, overlay: PhysicsOverlay) -> bool {
        let mut overlays = self.view().overlays;
        let on = match overlays.iter().position(|o| *o == overlay) {
            Some(i) => {
                overlays.remove(i);
                false
            },
            None => {
                overlays.push(overlay);
                true
            },
        };
        self.pick_overlays(overlays).is_ok() && on
    }
    fn pick_kind(&mut self, kind: PhysicsKindSource) {
        let choice = PhysicsChoice {
            kind,
            ..self.view()
        };
        let _ = self.pick(&choice);
    }
    fn pick_groups(&mut self, groups: PhysicsKindSource) {
        let choice = PhysicsChoice {
            groups,
            ..self.view()
        };
        let _ = self.pick(&choice);
    }
    fn pick_mass(&mut self, mass: PhysicsMassSource) {
        let choice = PhysicsChoice {
            mass,
            ..self.view()
        };
        let _ = self.pick(&choice);
    }
    fn pick_depth(&mut self, depth: PhysicsDepthSource) {
        let choice = PhysicsChoice {
            depth,
            ..self.view()
        };
        let _ = self.pick(&choice);
    }
    /// A profile's law and overlays, the sources kept; `false` for an
    /// unknown id.
    fn pick_profile(&mut self, id: &str) -> bool;
    /// A composition in the law slot (`None` returns to the law). A grouping
    /// on a partition the host hands in has no spec (F157) and goes in
    /// directly.
    fn pick_composition(
        &mut self,
        composition: Option<PhysicsComposition>,
    ) -> Result<(), CompositionRefusal>;
    /// A schedule, run from its first stage.
    fn pick_schedule(&mut self, stages: Vec<PhysicsStage>);
}

impl ThroughView for Canvas {
    fn view(&self) -> PhysicsChoice {
        PhysicsChoice::live(self)
    }

    fn record(&self) -> DynamicsSpec {
        // A grouping on a partition handed in has no spec (F157): a pick
        // after one starts from the stage as a law.
        self.dynamics_spec()
            .unwrap_or_else(|_| PhysicsChoice::live(self).into_spec())
    }

    fn pick(&mut self, choice: &PhysicsChoice) -> Result<(), OverlayRefusal> {
        let (choice, refusal) = choice.clone().admitted();
        let mut spec = self.record();
        choice.write_into(&mut spec, &self.view());
        self.set_dynamics_spec(&spec)
            .expect("an admitted choice binds");
        refusal.map_or(Ok(()), Err)
    }

    fn pick_profile(&mut self, id: &str) -> bool {
        match self.view().with_profile(id) {
            Some(choice) => self.pick(&choice).is_ok(),
            None => false,
        }
    }

    fn pick_composition(
        &mut self,
        composition: Option<PhysicsComposition>,
    ) -> Result<(), CompositionRefusal> {
        if let Some(refusal) = composition.as_ref().and_then(PhysicsComposition::refusal) {
            return Err(refusal);
        }
        if let Some(PhysicsComposition::Grouped(grouping)) = &composition
            && matches!(grouping.groups, GroupSource::Given(_))
        {
            return self.set_physics_composition(composition);
        }
        let view = self.view();
        let mut spec = self.record();
        spec.root = running_node(view.law, composition.as_ref(), &view.overlays)
            .expect("a composition on a channel has a node");
        self.set_dynamics_spec(&spec)
            .expect("the composition binds");
        Ok(())
    }

    fn pick_schedule(&mut self, stages: Vec<PhysicsStage>) {
        if stages.is_empty() {
            let view = self.view();
            let _ = self.pick(&view);
            return;
        }
        let mut spec = self.record();
        spec.root = schedule_node(&stages).expect("a schedule on channels has a node");
        self.set_dynamics_spec(&spec).expect("the schedule binds");
    }
}
