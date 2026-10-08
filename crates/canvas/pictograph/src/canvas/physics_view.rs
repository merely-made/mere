// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The flat view of a dynamics spec (dynamics grammar plan, G4b1, F162).
//!
//! A canvas's public physics surface is its spec: a host sets one with
//! [`Canvas::set_dynamics_spec`] and reads one with
//! [`Canvas::dynamics_spec`]. A picker reads and edits through
//! [`PhysicsChoice`], a law, its overlays and the four sources:
//!
//! ```ignore
//! let mut spec = canvas.dynamics_spec()?;
//! let (choice, refusal) = picked.admitted();
//! choice.write_into(&mut spec);
//! canvas.set_dynamics_spec(&spec)?;
//! ```
//!
//! [`PhysicsChoice::live`] reads what a canvas runs now, the stage under way
//! when a schedule runs ([`Canvas::live_stage_spec`]'s view);
//! [`PhysicsChoice::view`] reads any spec. A composition's view is its first
//! law, which is what a law picker shows (F157).

use super::Canvas;
use super::dynamics_spec::{
    BindError, Bound, BoundRoot, BoundSources, DynamicsSpec, Node, bind, bind_shape,
};
use super::physics_catalog::{
    CANVAS_PHYSICS_PROFILES, OverlayRefusal, PhysicsDepthSource, PhysicsKindSource, PhysicsLaw,
    PhysicsMassSource, PhysicsOverlay, physics_profile,
};

/// A law, its overlays in run order and the four sources: the flat view a
/// picker edits and a reader reads (F162).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicsChoice {
    pub law: PhysicsLaw,
    pub overlays: Vec<PhysicsOverlay>,
    pub kind: PhysicsKindSource,
    /// Group pull's groups channel.
    pub groups: PhysicsKindSource,
    pub mass: PhysicsMassSource,
    pub depth: PhysicsDepthSource,
}

impl Default for PhysicsChoice {
    fn default() -> Self {
        Self {
            law: PhysicsLaw::Springs,
            overlays: Vec::new(),
            kind: PhysicsKindSource::Site,
            groups: PhysicsKindSource::Site,
            mass: PhysicsMassSource::Degree,
            depth: PhysicsDepthSource::Roots,
        }
    }
}

impl PhysicsChoice {
    /// The choice as a spec: its law and overlays at the root, every slot
    /// written, the default seed, no target (F146).
    pub fn into_spec(&self) -> DynamicsSpec {
        let mut spec = DynamicsSpec::new(Node::preset(self.law.id()));
        self.write_into(&mut spec);
        spec
    }

    /// Write this choice into `spec`, keeping its seed, bars, realization and
    /// target: a picker's edit of the record (F146). Every slot takes these
    /// sources; the root becomes this law and its overlays unless the spec's
    /// own view already shows them, so a source edit keeps a composition or
    /// a schedule as a source setter did, and a law or overlay edit replaces
    /// it as a law pick did.
    pub fn write_into(&self, spec: &mut DynamicsSpec) {
        spec.channels = self.sources().channels();
        let shown = Self::view(spec)
            .is_ok_and(|view| view.law == self.law && view.overlays == self.overlays);
        if !shown {
            let mut root = Node::preset(self.law.id());
            root.overlays_mut()
                .extend(self.overlays.iter().map(|o| Node::preset(o.id())));
            spec.root = root;
        }
    }

    /// The flat view of a spec, checked as the canvas checks it: derived,
    /// then bound.
    pub fn from_spec(spec: &DynamicsSpec) -> Result<Self, BindError> {
        Ok(Self::flat(bind(spec)?))
    }

    /// The flat view of a spec's shape, without deriving it: what a reader
    /// reads off a spec a canvas already took.
    pub fn view(spec: &DynamicsSpec) -> Result<Self, BindError> {
        Ok(Self::flat(bind_shape(spec)?))
    }

    /// What `canvas` runs now: its law (a composition's first, a schedule's
    /// stage under way), overlays and sources. The view of
    /// [`Canvas::live_stage_spec`], read without building it.
    pub fn live(canvas: &Canvas) -> Self {
        Self {
            law: canvas.physics_law,
            overlays: canvas.physics_overlays.clone(),
            kind: canvas.physics_kind_source,
            groups: canvas.physics_group_source,
            mass: canvas.physics_mass_source,
            depth: canvas.physics_depth_source,
        }
    }

    /// The flat view of a bound spec.
    pub(crate) fn flat(bound: Bound) -> Self {
        let (law, overlays) = match bound.root {
            BoundRoot::Law { law, overlays } | BoundRoot::Composed { law, overlays, .. } => {
                (law, overlays)
            },
            BoundRoot::Schedule(stages) => (stages[0].law, stages[0].overlays.clone()),
        };
        Self::of(law, overlays, bound.sources)
    }

    fn of(law: PhysicsLaw, overlays: Vec<PhysicsOverlay>, sources: BoundSources) -> Self {
        Self {
            law,
            overlays,
            kind: sources.kind,
            groups: sources.groups,
            mass: sources.mass,
            depth: sources.depth,
        }
    }

    /// The choice's four sources.
    pub fn sources(&self) -> BoundSources {
        BoundSources {
            kind: self.kind,
            groups: self.groups,
            mass: self.mass,
            depth: self.depth,
        }
    }

    /// The choice the law takes: duplicate overlays collapsed and the ones
    /// it refuses dropped, with the law's reason (Density's F73 message), so
    /// a picker applies the law and says what it left out.
    pub fn admitted(mut self) -> (Self, Option<OverlayRefusal>) {
        let mut seen = Vec::new();
        self.overlays.retain(|o| {
            let first = !seen.contains(o);
            seen.push(*o);
            first
        });
        let law = self.law;
        let (refused, kept): (Vec<_>, Vec<_>) = self
            .overlays
            .iter()
            .partition(|o| law.refuses(**o).is_some());
        self.overlays = kept;
        let refusal = refused.first().map(|first| OverlayRefusal {
            law,
            reason: law.refuses(*first).unwrap_or_default(),
            refused: refused.clone(),
        });
        (self, refusal)
    }

    /// This choice with profile `id`'s law and overlays, its sources kept;
    /// `None` for an unknown id.
    pub fn with_profile(self, id: &str) -> Option<Self> {
        let profile = physics_profile(id)?;
        Some(Self {
            law: profile.law,
            overlays: profile.overlays.to_vec(),
            ..self
        })
    }

    /// The profile whose law and overlays match this choice, if any: every
    /// (law, overlays) pair names at most one.
    pub fn profile_id(&self) -> Option<&'static str> {
        CANVAS_PHYSICS_PROFILES
            .iter()
            .find(|profile| profile.law == self.law && profile.overlays == self.overlays.as_slice())
            .map(|profile| profile.id)
    }
}
