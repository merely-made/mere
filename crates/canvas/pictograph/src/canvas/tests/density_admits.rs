// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Which overlays Density could take converted (dynamics grammar plan, G3;
//! §3's sixth question). Density is kinematic, so a force composed onto it
//! enters converted, `v = F/γ`, the rule Hold follows; the catalog still
//! refuses every overlay on Density by the interim ruling, so this probe
//! builds the force list itself, with Density's conversion on (it is off
//! while Density takes nothing; Density alone is read both ways, since on, a
//! crowded start's contacts no longer carry over). Each overlay, and
//! `EdgeSpring` as the
//! Bonds candidate ("accepted only if rank stays at least 0.8"), runs under
//! Density from the release receipts' first four dealt starts of their
//! graphs, until Density's own stop. Printed only; the figures go to Mark.
//!
//! `cargo test --release -p pictograph --features canvas --lib tests::density_admits:: -- --ignored --nocapture`

use super::density::{dealt, generated, until_settled};
use super::*;
use crate::canvas::physics_catalog::{LawSources, PhysicsLaw, PhysicsOverlay};

/// What joins Density: nothing (as the catalog runs it, or converting), an
/// overlay, or the edges as springs.
#[derive(Clone, Copy)]
enum Joining {
    AsBuilt,
    Alone,
    Overlay(PhysicsOverlay),
    Bonds,
}

impl Joining {
    fn name(self) -> String {
        match self {
            Joining::AsBuilt => "density alone, as the catalog runs it".into(),
            Joining::Alone => "density alone, converting".into(),
            Joining::Overlay(o) => format!("+ {}", o.label()),
            Joining::Bonds => "+ bonds (edge springs)".into(),
        }
    }
}

#[test]
#[ignore = "release probe for G3's Density question; prints its table"]
fn probe_density_with_each_overlay_converted() {
    let mut candidates = vec![Joining::AsBuilt, Joining::Alone, Joining::Bonds];
    candidates.extend(PhysicsOverlay::ALL.map(Joining::Overlay));
    for (name, graph) in [("gen-50", generated(50, 3)), ("gen-200", generated(200, 7))] {
        for joining in &candidates {
            let mut rows = Vec::new();
            for k in 0..4 {
                let mut canvas = dealt(&graph, k);
                canvas.set_physics_law(PhysicsLaw::Density).unwrap();
                let forces = {
                    let inputs = canvas.law_inputs();
                    let mut law = crate::canvas::physics_catalog::density_law(
                        inputs.masses(crate::canvas::physics_catalog::PhysicsMassSource::Degree),
                    );
                    law.converts = !matches!(joining, Joining::AsBuilt);
                    let mut forces: Vec<Box<dyn seiche::Force>> = vec![Box::new(law)];
                    match joining {
                        Joining::AsBuilt | Joining::Alone => {},
                        Joining::Overlay(o) => {
                            forces.push(inputs.overlay_force(*o, LawSources::bare()))
                        },
                        Joining::Bonds => forces.push(Box::new(seiche::EdgeSpring::default())),
                    }
                    forces
                };
                canvas.physics.set_forces(forces);
                let ticks = until_settled(&mut canvas, 60 * 122);
                let stats = canvas.layout_stats();
                rows.push((
                    stats.mass_area_rank,
                    stats.density_cv,
                    stats.overlaps,
                    ticks,
                ));
            }
            let mean = rows.iter().map(|r| r.0).sum::<f32>() / rows.len() as f32;
            let lowest = rows.iter().map(|r| r.0).fold(f32::INFINITY, f32::min);
            println!(
                "{name} {}: rank mean {mean:.3}, lowest {lowest:.3}; per start {}",
                joining.name(),
                rows.iter()
                    .map(|(r, cv, o, t)| format!("{r:.3} (cv {cv:.3}, {o} overlaps, {t} ticks)"))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
}
