// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Physics-host binding of Scenograph's opaque recipe carrier (F192–F195).
//! Arrangement validation occurs here, before a controller is changed.

use scenograph::DynamicsSlot;
use seiche::{DynamicsRunner, SettleError, Speed};
pub use seiche::{SettleEnd, SettleReport};

use super::Canvas;
use super::dynamics_spec::{self, BindError, DynamicsSpec, Node};
use super::physics_catalog::PhysicsLaw;

fn refused(place: &str, reason: impl ToString) -> BindError {
    BindError::Unbound {
        place: place.into(),
        reason: reason.to_string(),
    }
}

/// Decode and validate the runnable spec, including the recipe's arrangement.
/// An absent target keeps the existing seeded role default. Catalog aliases
/// may identify the same arrangement; two unresolved names must match exactly.
pub fn bind_slot(slot: &DynamicsSlot, arrangement: &str) -> Result<DynamicsSpec, BindError> {
    slot.validate()
        .map_err(|error| refused("dynamics", error))?;
    let json: serde_json::Value =
        serde_json::from_str(&slot.spec).map_err(|error| refused("dynamics.spec", error))?;
    let spec = dynamics_spec::read(&json).map_err(|error| refused("dynamics.spec", error))?;
    dynamics_spec::bind(&spec)?;
    if let Some(target) = &spec.target {
        let equal = target.arrangement == arrangement
            || matches!(
                (scenomise::catalog::Family::resolve(&target.arrangement),
                 scenomise::catalog::Family::resolve(arrangement)),
                (Some(a), Some(b)) if a == b
            );
        if !equal {
            return Err(refused(
                "dynamics.spec.target.arrangement",
                format!(
                    "{:?} does not match recipe arrangement.kind {arrangement:?}",
                    target.arrangement
                ),
            ));
        }
    }
    Ok(spec)
}

/// Resolve a variant's law preset through the same catalog used by the canvas.
pub fn preset_slot(id: &str) -> Result<DynamicsSlot, String> {
    let law = PhysicsLaw::parse(id)
        .ok_or_else(|| format!("variant.dynamics.id: unknown law preset {id:?}"))?;
    let spec = DynamicsSpec::new(Node::preset(law.id()));
    let json = serde_json::to_string(&spec).map_err(|error| error.to_string())?;
    DynamicsSlot::from_json(scenograph::DYNAMICS_SLOT_VERSION, &json)
}

#[derive(Default)]
pub(crate) struct Progress {
    settles: u64,
    schedule: bool,
    wanted: bool,
    velocity_rest: bool,
}

impl Progress {
    pub(crate) fn start(
        &mut self,
        canvas: &mut Canvas,
        spec: &DynamicsSpec,
    ) -> Result<(), BindError> {
        if !matches!(canvas.physics, seiche::Physics::Inline(_)) {
            return Err(refused(
                "settle.inputs",
                "fixed-step execution requires an inline controller",
            ));
        }
        // Position-writing kernels can move with zero reported body speed.
        // Until they expose completion, only their explicit schedule or the
        // caller's bound can end this preview. Keep ordinary Canvas budgets
        // and rest detection unchanged.
        self.velocity_rest = canvas.roles.encoded == seiche::Axes::BOTH
            || !dynamics_spec::derive(spec)
                .map_err(BindError::Refused)?
                .leaves
                .iter()
                .any(|leaf| {
                    matches!(
                        leaf.currency,
                        seiche::Currency::Kinematic | seiche::Currency::Resident
                    )
                });
        canvas.set_dynamics_spec(spec)?;
        canvas.physics.set_speed(Speed::REAL_TIME);
        canvas.physics.set_step_budget(None);
        canvas.set_physics_paused(false);
        canvas.settle_physics(u32::MAX);
        if self.velocity_rest && !canvas.physics.tick_demand().0 {
            canvas.arm_settle_detection();
        }
        self.settles = canvas.settle_count();
        self.schedule = canvas.physics_schedule_stage().is_some();
        self.wanted = canvas.physics.tick_demand().0;
        Ok(())
    }
    pub(crate) fn step(&mut self, canvas: &mut Canvas) -> Option<SettleEnd> {
        canvas.step_dynamics_layout(self.velocity_rest);
        // A complete recipe includes the anchored return. A cap, in contrast,
        // is a snapshot cutoff, not the user's Pause/return-home operation.
        if canvas.returning_home() {
            return None;
        }
        if self.schedule {
            return canvas
                .physics_schedule_stage()
                .is_none()
                .then_some(SettleEnd::LawFinished);
        }
        let wanted = canvas.physics.tick_demand().0;
        let law_finished = self.wanted && !wanted;
        self.wanted |= wanted;
        if law_finished {
            Some(SettleEnd::LawFinished)
        } else if wanted {
            // A converging kernel can ask for another pass at zero body
            // speed. Its own completion remains authoritative for this run.
            None
        } else if self.velocity_rest && canvas.settle_count() > self.settles {
            Some(SettleEnd::Rested)
        } else {
            None
        }
    }
}

struct CanvasRunner<'a> {
    canvas: &'a mut Canvas,
    progress: Progress,
}
impl DynamicsRunner for CanvasRunner<'_> {
    type Error = BindError;
    fn prepare(&mut self, spec: &DynamicsSpec) -> Result<(), BindError> {
        self.progress.start(self.canvas, spec)
    }
    fn step_fixed(&mut self) -> Result<Option<SettleEnd>, BindError> {
        Ok(self.progress.step(self.canvas))
    }

    fn positions(&self) -> Vec<(kernel::graph::NodeKey, euclid::default::Point2D<f32>)> {
        self.canvas.view.positions().collect()
    }
}

impl Canvas {
    /// Run an explicitly bounded preview through the live layout controller.
    /// Use a separate preview canvas: this advances its bodies and recipe.
    /// The user's speed and per-frame budget are restored even on refusal.
    /// The bound stays explicit here; F202's editable default belongs to the host.
    pub fn settle_dynamics(
        &mut self,
        spec: &DynamicsSpec,
        bound: u32,
    ) -> Result<SettleReport, SettleError<BindError>> {
        let speed = self.physics.speed();
        let budget = self.physics.step_budget();
        let mut runner = CanvasRunner {
            canvas: self,
            progress: Progress::default(),
        };
        let result = seiche::settle(spec, &mut runner, bound);
        runner.canvas.physics.set_speed(speed);
        runner.canvas.physics.set_step_budget(budget);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::dynamics_spec::{Stage, Stop, Target};
    use super::*;
    use euclid::default::Point2D;
    use kernel::graph::Graph;
    use kernel::graph::fixtures::GraphFixtures;
    use std::collections::BTreeMap;

    fn canvas() -> Canvas {
        let mut graph = Graph::new();
        let positions: Vec<_> = (0..12)
            .map(|i| {
                let p = Point2D::new((i % 4) as f32 * 50.0, (i / 4) as f32 * 50.0);
                (graph.add_node(format!("https://s{}.test/{i}", i % 3), p), p)
            })
            .collect();
        let mut canvas = Canvas::with_graph(graph);
        canvas.set_physics_paused(true);
        canvas.set_layout_strategy(Some("grid".into()));
        canvas.apply_strategy_positions(&positions);
        canvas.apply_strategy_to_view();
        canvas
    }

    #[test]
    fn host_refuses_unknown_presets_and_conflicting_targets_before_execution() {
        assert!(
            preset_slot("unknown.law")
                .unwrap_err()
                .contains("unknown law")
        );
        let slot = preset_slot(PhysicsLaw::Springs.id()).unwrap();
        assert!(bind_slot(&slot, "grid").unwrap().target.is_none());
        let mut spec = bind_slot(&slot, "grid").unwrap();
        spec.target = Some(Target {
            arrangement: "grid.default".into(),
            anchored_pull: 1.0,
            default_role: seiche::Role::Pinned,
            groups: None,
            items: BTreeMap::new(),
        });
        let slot = DynamicsSlot::from_json(1, &serde_json::to_string(&spec).unwrap()).unwrap();
        assert!(bind_slot(&slot, "grid").is_ok());
        let error = bind_slot(&slot, "scatter").unwrap_err().to_string();
        assert!(error.contains("target.arrangement") && error.contains("arrangement.kind"));
    }

    #[test]
    fn a_living_law_caps_and_a_schedule_counts_fixed_ticks() {
        let mut live = canvas();
        let orbit = DynamicsSpec::new(Node::preset(PhysicsLaw::Orbit.id()));
        let report = live.settle_dynamics(&orbit, 40).unwrap();
        assert_eq!(report.end, SettleEnd::StepLimit);
        assert_eq!(report.steps, 40);
        let schedule = DynamicsSpec::new(Node::Schedule {
            stages: vec![
                Stage {
                    node: Node::preset(PhysicsLaw::Orbit.id()),
                    stop: Stop::Frames(2),
                    capture: None,
                },
                Stage {
                    node: Node::preset(PhysicsLaw::Orbit.id()),
                    stop: Stop::Frames(3),
                    capture: None,
                },
            ],
            weight: 1.0,
            overlays: Vec::new(),
        });
        let mut scheduled = canvas();
        let report = scheduled.settle_dynamics(&schedule, 20).unwrap();
        assert_eq!(report.end, SettleEnd::LawFinished);
        assert_eq!(report.steps, 5);
    }

    #[test]
    fn a_position_writing_law_is_not_reported_at_rest_from_zero_velocity() {
        let mut preview = canvas();
        let anneal = DynamicsSpec::new(Node::preset(PhysicsLaw::Anneal.id()));
        let report = preview.settle_dynamics(&anneal, 8).unwrap();
        assert_eq!(report.end, SettleEnd::StepLimit);
        assert_eq!(report.steps, 8);
        assert_eq!(preview.settle_count(), 0);
    }

    #[test]
    fn a_fresh_run_repeats_and_preserves_pinned_positions() {
        let mut a = canvas();
        let mut b = canvas();
        let mut spec = DynamicsSpec::new(Node::preset(PhysicsLaw::Springs.id()));
        let a_report = a.settle_dynamics(&spec, 30).unwrap();
        let b_report = b.settle_dynamics(&spec, 30).unwrap();
        assert_eq!(a_report, b_report);
        let mut pinned = canvas();
        let initial: Vec<_> = pinned.view.positions().collect();
        spec.target = Some(Target {
            arrangement: "grid".into(),
            anchored_pull: 1.0,
            default_role: seiche::Role::Pinned,
            groups: None,
            items: BTreeMap::new(),
        });
        let report = pinned.settle_dynamics(&spec, 30).unwrap();
        for (key, position) in initial {
            assert_eq!(
                report.positions.iter().find(|(k, _)| *k == key).unwrap().1,
                position
            );
        }
    }

    #[test]
    fn measure_the_catalog_bound_on_a_fixed_arrangement() {
        for law in PhysicsLaw::ALL {
            let mut canvas = canvas();
            let spec = DynamicsSpec::new(Node::preset(law.id()));
            let started = std::time::Instant::now();
            let report = canvas.settle_dynamics(&spec, 4000).unwrap();
            println!(
                "{}: {:?}, {} steps, {} us",
                law.id(),
                report.end,
                report.steps,
                started.elapsed().as_micros()
            );
        }
    }
}
