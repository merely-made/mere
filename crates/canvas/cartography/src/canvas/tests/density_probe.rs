// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The Density convergence probe: repeated passes at the catalog's defaults
//! with no stop test, so every candidate test can be read off one trace.
//! Each pass prints the law's own numbers (mean shift in spacings, the
//! fresh field's CV) beside the canvas's (Spearman of mass against area,
//! density CV, overlaps). Ignored; run optimized with `--ignored --nocapture`.

use crate::canvas::tests::ThroughView;
use std::sync::Arc;

use super::density::{generated, run, seeded, uniform};
use crate::canvas::physics_catalog::{PhysicsLaw, PhysicsMassSource};
use seiche::{Density, DensityGrid, DensityStop, Force, ForceContext};

/// Shares a law with the canvas so the probe can read its pass history.
struct Shared(Arc<Density>);

impl seiche::Declared for Shared {
    fn terms(&self) -> Vec<seiche::Term> {
        self.0.terms()
    }

    fn metric(&self, term: usize, layout: &seiche::Layout<'_>) -> Option<Vec<f64>> {
        self.0.metric(term, layout)
    }
}

impl Force for Shared {
    fn apply(&self, ctx: &mut ForceContext<'_>, dt: f32) {
        self.0.apply(ctx, dt);
    }

    fn wants_tick(&self) -> bool {
        self.0.wants_tick()
    }
}

const PASSES: u32 = 90;

#[test]
#[ignore = "convergence probe: prints a per-pass trace"]
fn density_convergence_probe() {
    let graphs = [
        ("sample-12", crate::canvas::build::sample_graph(), 40.0),
        ("gen-50", generated(50, 3), 40.0),
        ("gen-200", generated(200, 7), 40.0),
        ("uniform-200", uniform(200), 14.0),
    ];
    println!("graph | mass | pass | shift | field cv | rank | density cv | overlaps | ms/tick");
    for (name, graph, spacing) in &graphs {
        for mass in [PhysicsMassSource::Degree, PhysicsMassSource::PageRank] {
            if name.starts_with("uniform") && mass == PhysicsMassSource::PageRank {
                continue;
            }
            let mut canvas = seeded(graph.clone(), *spacing);
            canvas.pick_mass(mass);
            canvas.pick_law(PhysicsLaw::Density).unwrap();
            let mut law = Density::with_medium(
                canvas.law_inputs().masses(mass),
                Box::new(DensityGrid::new(
                    crate::canvas::physics_catalog::DENSITY_RESOLUTION,
                )),
            );
            law.stop = DensityStop::Cap;
            law.max_passes = PASSES + 1;
            let law = Arc::new(law);
            canvas
                .physics
                .set_forces(vec![Box::new(Shared(law.clone()))]);
            let mut stats_time = std::time::Duration::ZERO;
            let started = std::time::Instant::now();
            // Pass k closes on tick 60k + 1: one tick to begin, then a pass
            // of sixty, so the canvas reads where the law took its numbers.
            run(&mut canvas, 1);
            for pass in 1..=PASSES {
                run(&mut canvas, 60);
                let t = std::time::Instant::now();
                let stats = canvas.layout_stats();
                stats_time += t.elapsed();
                let history = law.pass_history();
                let Some(record) = history.iter().find(|r| r.pass == pass) else {
                    println!("{name} | {} | {pass} | no record", mass.id());
                    continue;
                };
                let ms = (started.elapsed() - stats_time).as_secs_f64() * 1000.0
                    / f64::from(pass * 60 + 1);
                println!(
                    "{name} | {} | {pass} | {:.4} | {:.4} | {:.3} | {:.3} | {} | {ms:.2}",
                    mass.id(),
                    record.shift,
                    record.field_cv,
                    stats.mass_area_rank,
                    stats.density_cv,
                    stats.overlaps,
                );
            }
        }
    }
}
