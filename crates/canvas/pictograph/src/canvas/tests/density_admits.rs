// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Which overlays Density takes converted (dynamics grammar plan, G3; F73).
//! Density is kinematic, so a force composed onto it enters converted,
//! `v = F/γ`, the rule Hold follows, its conversion on once it takes an
//! overlay. The receipt: each overlay the catalog admits, set on Density by
//! the canvas, holds Density's own bar ("Min 60, bar: all >= 0.7") at all
//! sixteen dealt starts of both bar graphs. The probes build the force list
//! themselves: the four-start table every overlay and `EdgeSpring` (the
//! Bonds candidate, "accepted only if rank stays at least 0.8") were read
//! from, and F73's three candidates over all sixteen starts.
//!
//! `cargo test --release -p pictograph --features canvas --lib tests::density_admits:: -- --ignored --nocapture`

use super::density::{dealt, generated, until_settled};
use crate::canvas::physics_catalog::{LawSources, PhysicsLaw, PhysicsOverlay};
use kernel::graph::Graph;

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

/// One start: `joining` under Density from dealt start `k` of `graph`, run
/// until Density's own stop: rank, density CV, overlaps, ticks.
fn one_start(graph: &Graph, k: u64, joining: Joining) -> (f32, f32, usize, u32) {
    let mut canvas = dealt(graph, k);
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
            Joining::Overlay(o) => forces.push(inputs.overlay_force(o, LawSources::bare())),
            Joining::Bonds => forces.push(Box::new(seiche::EdgeSpring::default())),
        }
        forces
    };
    canvas.physics.set_forces(forces);
    let ticks = until_settled(&mut canvas, 60 * 122);
    let stats = canvas.layout_stats();
    (
        stats.mass_area_rank,
        stats.density_cv,
        stats.overlaps,
        ticks,
    )
}

fn print_rows(name: &str, joining: Joining, rows: &[(f32, f32, usize, u32)]) {
    let mean = rows.iter().map(|r| r.0).sum::<f32>() / rows.len() as f32;
    let lowest = rows.iter().map(|r| r.0).fold(f32::INFINITY, f32::min);
    let at_eight = rows.iter().filter(|r| r.0 >= 0.8).count();
    println!(
        "{name} {}: rank mean {mean:.3}, lowest {lowest:.3}, {at_eight} of {} at 0.8 or more; \
         per start {}",
        joining.name(),
        rows.len(),
        rows.iter()
            .map(|(r, cv, o, t)| format!("{r:.3} (cv {cv:.3}, {o} overlaps, {t} ticks)"))
            .collect::<Vec<_>>()
            .join(", ")
    );
}

#[test]
#[ignore = "release probe for G3's Density question; prints its table"]
fn probe_density_with_each_overlay_converted() {
    let mut candidates = vec![Joining::AsBuilt, Joining::Alone, Joining::Bonds];
    candidates.extend(PhysicsOverlay::ALL.map(Joining::Overlay));
    for (name, graph) in [("gen-50", generated(50, 3)), ("gen-200", generated(200, 7))] {
        for joining in &candidates {
            let rows: Vec<_> = (0..4).map(|k| one_start(&graph, k, *joining)).collect();
            print_rows(name, *joining, &rows);
        }
    }
}

/// F73's test (ruled 2026-10-06, "Hub room, Centre, Tide if they hold"):
/// each of the three, converted into Density, against Density's own bar
/// ("Min 60, bar: all >= 0.7") over all sixteen dealt starts of both bar
/// graphs. Printed; it decided the catalog's two (Tide's 0.699 at gen-50's
/// seventh start misses).
#[test]
#[ignore = "release probe for F73; prints its table"]
fn probe_f73_candidates_over_all_sixteen_starts() {
    let candidates = [
        PhysicsOverlay::DegreeRepulsion,
        PhysicsOverlay::GravityLocus,
        PhysicsOverlay::Tide,
    ];
    for (name, graph) in [("gen-50", generated(50, 3)), ("gen-200", generated(200, 7))] {
        for overlay in candidates {
            let rows: Vec<_> = (0..16)
                .map(|k| one_start(&graph, k, Joining::Overlay(overlay)))
                .collect();
            print_rows(name, Joining::Overlay(overlay), &rows);
            let holds = rows.iter().all(|r| r.0 >= 0.7);
            println!(
                "{name} {}: {}",
                overlay.label(),
                if holds {
                    "holds the bar"
                } else {
                    "misses the bar"
                }
            );
        }
    }
}

/// F73's receipt: every overlay the catalog admits on Density, set by the
/// canvas (so the catalog builds Density converting), holds Density's own
/// bar at every one of the sixteen dealt starts of both bar graphs.
#[test]
#[ignore = "release receipt: the module doc's one line runs it"]
fn every_overlay_density_admits_holds_its_bar_at_all_sixteen_starts() {
    for (name, graph) in [("gen-50", generated(50, 3)), ("gen-200", generated(200, 7))] {
        for &overlay in crate::canvas::physics_catalog::DENSITY_ADMITS {
            let rows: Vec<_> = (0..16)
                .map(|k| {
                    let mut canvas = dealt(&graph, k);
                    canvas.set_physics_law(PhysicsLaw::Density).unwrap();
                    canvas.set_physics_overlays(vec![overlay]).unwrap();
                    let ticks = until_settled(&mut canvas, 60 * 122);
                    let stats = canvas.layout_stats();
                    (stats.mass_area_rank, stats.density_cv, stats.overlaps, ticks)
                })
                .collect();
            print_rows(name, Joining::Overlay(overlay), &rows);
            for (k, row) in rows.iter().enumerate() {
                assert!(
                    row.0 >= 0.7,
                    "{name} + {}: start {k} reads {:.3}, under the bar",
                    overlay.label(),
                    row.0
                );
            }
        }
    }
}
