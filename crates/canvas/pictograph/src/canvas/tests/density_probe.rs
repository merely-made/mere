// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The Density fork probe: the evolving field's open defaults (seconds to
//! even, initial blur, grid, CFL fraction) on the sample graphs, with rank
//! over time, density CV, overlaps against Springs, step cost and the
//! substep diagnostics. Ignored; run optimized with `--ignored --nocapture`.

use std::sync::Arc;

use super::density::{generated, run, seeded};
use super::*;
use crate::canvas::physics_catalog::{LawSources, PhysicsLaw, PhysicsMassSource};
use seiche::{Density, DensityGrid, Force, ForceContext};

/// Shares a law with the canvas so the probe can read its flow state.
struct Shared(Arc<Density>);

impl Force for Shared {
    fn apply(&self, ctx: &mut ForceContext<'_>, dt: f32) {
        self.0.apply(ctx, dt);
    }
}

#[derive(Clone, Copy, Debug)]
struct Setting {
    passes: u32,
    seconds: f32,
    blur: f32,
    resolution: usize,
    cfl: f32,
}

const DEFAULT: Setting = Setting {
    passes: 1,
    seconds: 4.0,
    blur: 0.25,
    resolution: 128,
    cfl: 0.5,
};

fn settings() -> Vec<Setting> {
    if std::env::var("DENSITY_PROBE").as_deref() == Ok("blur") {
        let base = Setting {
            passes: 120,
            seconds: 1.0,
            resolution: 64,
            ..DEFAULT
        };
        return vec![
            base,
            Setting { blur: 0.5, ..base },
            Setting { blur: 1.0, ..base },
            Setting {
                blur: 0.5,
                resolution: 128,
                ..base
            },
        ];
    }
    if std::env::var("DENSITY_PROBE").as_deref() == Ok("passes") {
        return vec![
            Setting {
                passes: 3,
                ..DEFAULT
            },
            Setting {
                passes: 8,
                ..DEFAULT
            },
            Setting {
                passes: 8,
                seconds: 2.0,
                ..DEFAULT
            },
            Setting {
                passes: 30,
                seconds: 1.0,
                ..DEFAULT
            },
            Setting {
                passes: 30,
                seconds: 1.0,
                resolution: 64,
                ..DEFAULT
            },
        ];
    }
    vec![
        DEFAULT,
        Setting {
            seconds: 2.0,
            ..DEFAULT
        },
        Setting {
            seconds: 6.0,
            ..DEFAULT
        },
        Setting {
            blur: 0.1,
            ..DEFAULT
        },
        Setting {
            blur: 0.5,
            ..DEFAULT
        },
        Setting {
            resolution: 64,
            ..DEFAULT
        },
        Setting {
            cfl: 0.25,
            ..DEFAULT
        },
    ]
}

fn sources(mass: PhysicsMassSource) -> LawSources {
    LawSources {
        kind: crate::canvas::PhysicsKindSource::Site,
        mass,
        depth: crate::canvas::PhysicsDepthSource::Roots,
        focus: None,
    }
}

#[test]
#[ignore = "fork probe: prints a table"]
fn density_fork_probe() {
    let graphs = [
        ("sample-12", crate::canvas::build::sample_graph()),
        ("gen-50", generated(50, 3)),
        ("gen-200", generated(200, 7)),
    ];
    let blur_only = std::env::var("DENSITY_PROBE").as_deref() == Ok("blur");
    let checkpoints: Vec<u32> = if blur_only {
        vec![240, 900, 1800, 3600]
    } else if std::env::var("DENSITY_PROBE").as_deref() == Ok("passes") {
        vec![240, 480, 900, 1800]
    } else {
        vec![60, 240, 360, 900]
    };
    println!(
        "graph | mass | setting | rank@{checkpoints:?} | cv | overlaps | spread | ms/tick | substeps max | capped"
    );
    for (name, graph) in &graphs {
        for mass in [PhysicsMassSource::Degree, PhysicsMassSource::PageRank] {
            if blur_only && (*name != "gen-200" || mass != PhysicsMassSource::Degree) {
                continue;
            }
            let mut rows: Vec<Option<Setting>> = settings().into_iter().map(Some).collect();
            rows.push(None);
            for setting in rows {
                let mut canvas = seeded(graph.clone(), 40.0);
                canvas.set_physics_mass_source(mass);
                let masses = canvas.law_inputs().masses(mass);
                let law = setting.map(|s| {
                    let mut d =
                        Density::with_medium(masses, Box::new(DensityGrid::new(s.resolution)));
                    d.seconds = s.seconds;
                    d.initial_blur = s.blur;
                    d.cfl = s.cfl;
                    d.passes = s.passes;
                    Arc::new(d)
                });
                let forces: Vec<Box<dyn Force>> = match &law {
                    Some(law) => vec![Box::new(Shared(law.clone()))],
                    None => canvas
                        .law_inputs()
                        .law_forces(PhysicsLaw::Springs, sources(mass)),
                };
                canvas.physics.set_forces(forces);
                let mut ranks = Vec::new();
                let mut done = 0;
                let started = std::time::Instant::now();
                let mut stats_time = std::time::Duration::ZERO;
                for &at in &checkpoints {
                    run(&mut canvas, at - done);
                    done = at;
                    let t = std::time::Instant::now();
                    ranks.push(canvas.layout_stats().mass_area_rank);
                    stats_time += t.elapsed();
                }
                let ms = (started.elapsed() - stats_time).as_secs_f64() * 1000.0 / f64::from(done);
                let stats = canvas.layout_stats();
                let flow = law.as_ref().and_then(|l| l.flow_state());
                println!(
                    "{name} | {} | {} | {:.2} {:.2} {:.2} {:.2} | {:.3} | {} | {:.0} | {ms:.2} | {} | {}",
                    mass.id(),
                    setting.map_or("Springs".to_string(), |s| format!("{s:?}")),
                    ranks[0],
                    ranks[1],
                    ranks[2],
                    ranks[3],
                    stats.density_cv,
                    stats.overlaps,
                    stats.spread,
                    flow.map_or(0, |f| f.max_substeps_seen),
                    flow.map_or(0, |f| f.capped_ticks),
                );
            }
        }
    }
}

/// Uniform mass from a clump: the density CV and overlaps the law reaches
/// at each setting, beside Springs on the same seed.
#[test]
#[ignore = "fork probe: prints uniform-mass evenness"]
fn density_uniform_probe() {
    use super::density::{Run, install};
    let settings = [
        Some(Run {
            passes: 1,
            seconds: 4.0,
            resolution: 128,
        }),
        Some(super::density::SAMPLE_RUN),
        Some(Run {
            passes: 120,
            seconds: 1.0,
            resolution: 64,
        }),
        None,
    ];
    for n in [60u128, 200] {
        for setting in settings {
            let mut graph = Graph::new();
            for i in 0..n {
                kernel::graph::apply::add_node(
                    &mut graph,
                    Some(uuid::Uuid::from_u128(i + 1)),
                    format!("https://u{i}.test/"),
                    PortablePoint::new(0.0, 0.0),
                );
            }
            let mut canvas = seeded(graph, 14.0);
            match setting {
                Some(setting) => install(&mut canvas, setting),
                None => canvas.set_physics_law(PhysicsLaw::Springs),
            }
            let mut trail = Vec::new();
            let mut done = 0;
            for at in [2u32, 60, 360, 900, 1800] {
                run(&mut canvas, at - done);
                done = at;
                let stats = canvas.layout_stats();
                trail.push(format!("{at}:{:.3}/{}", stats.density_cv, stats.overlaps));
            }
            println!(
                "uniform n={n} {} cv/overlaps {}",
                setting.map_or("Springs".to_string(), |s| format!("{s:?}")),
                trail.join(" ")
            );
        }
    }
}

/// Each overlay composed onto Density through the catalog (the catalog's
/// defaults): what the force-currency overlays do to the reading.
#[test]
#[ignore = "fork probe: prints Density under each overlay"]
fn density_overlay_probe() {
    use crate::canvas::physics_catalog::PhysicsOverlay;
    let mut rows: Vec<Option<PhysicsOverlay>> = vec![None];
    rows.extend(PhysicsOverlay::ALL.into_iter().map(Some));
    for (name, graph) in [("gen-50", generated(50, 3)), ("gen-200", generated(200, 7))] {
        for overlay in &rows {
            let mut canvas = seeded(graph.clone(), 40.0);
            canvas.set_physics_law(PhysicsLaw::Density);
            if let Some(overlay) = overlay {
                canvas.set_physics_overlays(vec![*overlay]);
            }
            run(&mut canvas, 900);
            let stats = canvas.layout_stats();
            println!(
                "overlay {name} | {} | rank {:.2} | cv {:.3} | overlaps {} | spread {:.0} | forces {}",
                overlay.map_or("none", |o| o.id()),
                stats.mass_area_rank,
                stats.density_cv,
                stats.overlaps,
                stats.spread,
                canvas.law_force_count()
            );
        }
    }
}
