// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The canvas's dynamics record (dynamics grammar plan, G4b1; F146, F157):
//! the spec a host sets and reads. What the canvas runs (the law, overlays,
//! composition and sources) is the record's root and channels, a schedule
//! as authored; the seed, bars, damping and the target's arrangement are
//! carried from the spec it was given; the target's roles are read live. A
//! spec opened and not edited since reads back as it was given.

use std::collections::HashMap;

use kernel::graph::NodeKey;
use seiche::{Role, RoleTable};

use super::Canvas;
use super::channels::Channel;
use super::dynamics_spec::{
    Bar, BindError, BoundRoot, DEFAULT_SEED, DYNAMICS_SPEC_VERSION, DynamicsSpec, GroupRoles,
    Realization, Target, bind, running_node, schedule_node,
};
use super::physics_catalog::PhysicsKindSource;
use super::schedule::PhysicsStage;

/// The parts of the record the canvas does not run: carried from the spec
/// it was given (F146).
#[derive(Clone, Debug)]
pub(crate) struct DynamicsRecord {
    pub(crate) seed: u64,
    bars: Vec<Bar>,
    arrangement: Option<String>,
    damping: Option<f64>,
    /// The schedule as authored, kept after its run ends (F157).
    pub(crate) schedule: Option<Vec<PhysicsStage>>,
    /// The spec as given and as the canvas read it back right after, so an
    /// unedited record reads back as it was given.
    opened: Option<(DynamicsSpec, DynamicsSpec)>,
}

impl Default for DynamicsRecord {
    fn default() -> Self {
        Self {
            seed: DEFAULT_SEED,
            bars: Vec::new(),
            arrangement: None,
            damping: None,
            schedule: None,
            opened: None,
        }
    }
}

/// What applying a spec found: target items the graph does not hold, which
/// fall to their group or the default (F105's reading).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DynamicsReport {
    pub absent_items: usize,
}

impl Canvas {
    /// Run `spec` and hold it as the record: its sources, seed, root, roles
    /// and damping. Nothing changes when it is refused, and the refusal names
    /// where (F97, F149).
    pub fn set_dynamics_spec(&mut self, spec: &DynamicsSpec) -> Result<DynamicsReport, BindError> {
        let bound = bind(spec)?;
        let sources = bound.sources;
        self.physics_kind_source = sources.kind;
        self.physics_group_source = sources.groups;
        self.physics_mass_source = sources.mass;
        self.physics_depth_source = sources.depth;
        self.dynamics.seed = bound.seed;
        self.dynamics.bars = spec.bars.clone();
        self.dynamics.damping = bound.damping;
        self.dynamics.arrangement = bound.target.as_ref().map(|t| t.arrangement.clone());
        let mut report = DynamicsReport::default();
        match &bound.target {
            Some(target) => {
                let items: HashMap<NodeKey, Role> = target
                    .items
                    .iter()
                    .filter_map(|(id, role)| {
                        let key = self.graph.get_node_key_by_id(*id);
                        report.absent_items += usize::from(key.is_none());
                        Some((key?, *role))
                    })
                    .collect();
                self.set_role_group_source(target.group_source);
                self.set_anchor_stiffness(target.anchored_pull);
                self.set_arrangement_roles(RoleTable {
                    default: target.default_role,
                    groups: target.groups.clone(),
                    items,
                });
            },
            None => {
                self.set_role_group_source(PhysicsKindSource::Site);
                self.set_arrangement_roles(RoleTable::uniform(Role::Seeded));
            },
        }
        if let Some(damping) = bound.damping {
            self.set_physics_damping(damping as f32);
        }
        match bound.root {
            BoundRoot::Law { law, overlays } => {
                self.dynamics.schedule = None;
                self.schedule = None;
                self.physics_composition = None;
                self.physics_overlays = overlays;
                self.physics_law = law;
                self.rebuild_law_forces();
                self.settle_for_law();
            },
            BoundRoot::Composed {
                law,
                composition,
                overlays,
            } => {
                self.dynamics.schedule = None;
                self.schedule = None;
                self.physics_overlays = overlays;
                self.physics_law = law;
                self.physics_composition = Some(composition);
                self.rebuild_law_forces();
                self.settle_for_law();
            },
            BoundRoot::Schedule(stages) => self.run_physics_schedule(stages),
        }
        let mut given = spec.clone();
        given.version = DYNAMICS_SPEC_VERSION;
        self.dynamics.opened = self.running_spec().ok().map(|read| (given, read));
        Ok(report)
    }

    /// The record: what the canvas runs, as a spec (F146), the target's
    /// roles read live and its arrangement the one the spec named (a host
    /// writes its own before saving). Refused for a grouping on a partition
    /// the host handed in, which has no channel id (F157).
    pub fn dynamics_spec(&self) -> Result<DynamicsSpec, String> {
        let live = self.running_spec()?;
        match &self.dynamics.opened {
            Some((given, read)) if *read == live => Ok(given.clone()),
            _ => Ok(live),
        }
    }

    /// The bars the record carries (F102), evaluated by nothing here.
    pub fn dynamics_bars(&self) -> &[Bar] {
        &self.dynamics.bars
    }

    fn running_spec(&self) -> Result<DynamicsSpec, String> {
        let root = match &self.dynamics.schedule {
            Some(stages) => schedule_node(stages)?,
            None => running_node(
                self.physics_law,
                self.physics_composition.as_ref(),
                &self.physics_overlays,
            )?,
        };
        let mut spec = DynamicsSpec::new(root);
        spec.seed = self.dynamics.seed;
        spec.bars = self.dynamics.bars.clone();
        spec.channels = self.physics_choice().sources().channels();
        spec.realization = Realization::Integrate {
            damping: self.dynamics.damping,
        };
        spec.target = Some(self.running_target());
        Ok(spec)
    }

    fn running_target(&self) -> Target {
        let table = self.arrangement_roles();
        let source = self.role_group_source();
        Target {
            arrangement: self.dynamics.arrangement.clone().unwrap_or_default(),
            anchored_pull: f64::from(self.anchor_stiffness()),
            default_role: table.default,
            groups: (source != PhysicsKindSource::Site || !table.groups.is_empty()).then(|| {
                GroupRoles {
                    channel: Channel::Groups(source).id(),
                    roles: table.groups.clone(),
                }
            }),
            items: table
                .items
                .iter()
                .filter_map(|(key, role)| Some((self.graph.get_node(*key)?.id.to_string(), *role)))
                .collect(),
        }
    }

    /// Note the damping the host set, so the record carries it.
    /// A damping the record already holds, at the f32 the bodies take, keeps
    /// the record's own number, so a spec's damping reads back as given.
    pub(crate) fn note_physics_damping(&mut self, damping: f32) {
        if self.dynamics.damping.map(|held| held as f32) != Some(damping) {
            self.dynamics.damping = Some(f64::from(damping));
        }
    }
}
