// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The Density fork probe: every contested option on the sample graphs, its
//! rank correlation over time, density CV, spread and step cost. Ignored;
//! run with `--ignored --nocapture` (optimized) for the fork report's table.

use super::density::{generated, run, seeded};
use super::*;
use crate::canvas::physics_catalog::{PhysicsLaw, PhysicsMassSource};
use seiche::{Density, DensityBounds, DensityGrid, EdgeSpring, Force};

#[derive(Clone, Copy, Debug)]
enum Variant {
    Walls(f32),
    /// Walls at (spacings, diffusivity, Jacobi iterations).
    Tuned(f32, f32, usize),
    /// As Tuned, always at the full 128² (no coarser working level).
    Full(f32, f32, usize),
    /// Gastner–Newman's evolving field at full 128²: (initial blur in
    /// spacings, seconds to even, Jacobi iterations).
    Gastner(f32, f32, usize),
    Sea(f32),
    WallsEdges(f32),
    Springs,
}

fn forces(canvas: &Canvas, variant: Variant, mass: PhysicsMassSource) -> Vec<Box<dyn Force>> {
    let masses = canvas.law_inputs().masses(mass);
    let law = |spacings: f32, bounds: DensityBounds| {
        let mut d = Density::new(masses.clone(), 128);
        d.diffusion_spacings = spacings;
        d.bounds = bounds;
        d
    };
    let walls = DensityBounds::Walls { centre: (0.0, 0.0) };
    match variant {
        Variant::Walls(s) => vec![Box::new(law(s, walls))],
        Variant::Gastner(s, seconds, iters) => {
            let mut grid = DensityGrid::new(128, iters);
            grid.max_diffusion_cells = None;
            let mut density = Density::with_medium(masses.clone(), Box::new(grid));
            density.diffusion_spacings = s;
            density.flow = seiche::DensityFlow::Gastner { seconds };
            density.max_speed = 1_200.0;
            vec![Box::new(density)]
        },
        Variant::Tuned(s, d, iters) | Variant::Full(s, d, iters) => {
            let mut grid = DensityGrid::new(128, iters);
            if matches!(variant, Variant::Full(..)) {
                grid.max_diffusion_cells = None;
            }
            let mut density = Density::with_medium(masses.clone(), Box::new(grid));
            density.diffusion_spacings = s;
            density.diffusivity = d;
            vec![Box::new(density)]
        },
        Variant::Sea(s) => vec![Box::new(law(s, DensityBounds::Sea { margin: 2.0 }))],
        Variant::WallsEdges(s) => vec![Box::new(law(s, walls)), Box::new(EdgeSpring::default())],
        Variant::Springs => canvas.law_inputs().law_forces(
            PhysicsLaw::Springs,
            crate::canvas::physics_catalog::LawSources {
                kind: crate::canvas::PhysicsKindSource::Site,
                mass,
                depth: crate::canvas::PhysicsDepthSource::Roots,
                focus: None,
            },
        ),
    }
}

#[test]
#[ignore = "fork probe: prints a table"]
fn density_fork_probe() {
    let only = std::env::var("DENSITY_PROBE").unwrap_or_default();
    let graphs = [
        ("sample-12", crate::canvas::build::sample_graph()),
        ("gen-50", generated(50, 3)),
        ("gen-200", generated(200, 7)),
    ];
    let variants: Vec<Variant> = if only == "gastner" {
        vec![
            Variant::Gastner(0.5, 4.0, 24),
            Variant::Gastner(1.0, 4.0, 24),
            Variant::Gastner(0.5, 4.0, 64),
            Variant::Gastner(0.5, 2.0, 24),
            Variant::Tuned(1.0, 100_000.0, 24),
            Variant::Springs,
        ]
    } else if only == "level" {
        vec![
            Variant::Tuned(1.0, 12_000.0, 24),
            Variant::Tuned(1.0, 50_000.0, 24),
            Variant::Tuned(1.0, 100_000.0, 24),
            Variant::Full(1.0, 12_000.0, 24),
            Variant::Full(1.0, 50_000.0, 24),
            Variant::Springs,
        ]
    } else if only == "tune" {
        vec![
            Variant::Tuned(1.0, 12_000.0, 24),
            Variant::Tuned(1.0, 50_000.0, 24),
            Variant::Tuned(1.0, 100_000.0, 24),
            Variant::Tuned(1.0, 50_000.0, 64),
            Variant::Tuned(0.5, 50_000.0, 24),
            Variant::Tuned(2.0, 50_000.0, 64),
        ]
    } else {
        vec![
            Variant::Walls(0.5),
            Variant::Walls(1.0),
            Variant::Walls(2.0),
            Variant::Sea(1.0),
            Variant::WallsEdges(1.0),
            Variant::Springs,
        ]
    };
    let checkpoints: Vec<u32> = if only == "gastner" {
        vec![60, 240, 360, 900]
    } else if only == "tune" {
        vec![60, 360, 900, 3600]
    } else {
        vec![60, 360, 900, 1800]
    };
    println!(
        "graph | mass | variant | rank@{checkpoints:?} | inner rank | cv | spread | overlaps | ms/tick"
    );
    for (name, graph) in &graphs {
        for mass in [PhysicsMassSource::Degree, PhysicsMassSource::PageRank] {
            for &variant in &variants {
                let mut canvas = seeded(graph.clone(), 40.0);
                canvas.set_physics_mass_source(mass);
                let f = forces(&canvas, variant, mass);
                canvas.physics.set_forces(f);
                let mut ranks = Vec::new();
                let mut done = 0;
                let started = std::time::Instant::now();
                for &at in &checkpoints {
                    run(&mut canvas, at - done);
                    done = at;
                    ranks.push(canvas.layout_stats().mass_area_rank);
                }
                let ms = started.elapsed().as_secs_f64() * 1000.0 / f64::from(done);
                let stats = canvas.layout_stats();
                let inner = interior_rank(&canvas, mass);
                println!(
                    "{name} | {} | {variant:?} | {:.2} {:.2} {:.2} {:.2} | {inner:.2} | {:.3} | {:.0} | {} | {ms:.2}",
                    mass.id(),
                    ranks[0],
                    ranks[1],
                    ranks[2],
                    ranks[3],
                    stats.density_cv,
                    stats.spread,
                    stats.overlaps
                );
            }
        }
    }
}

/// Spearman over the nodes whose cells do not reach the measured rim.
fn interior_rank(canvas: &Canvas, mass: PhysicsMassSource) -> f32 {
    let masses: HashMap<NodeKey, f32> = canvas.law_inputs().masses(mass).into_iter().collect();
    let at: Vec<(NodeKey, Point2D<f32>)> = canvas.view.positions().collect();
    let points: Vec<(f32, f32)> = at.iter().map(|(_, p)| (p.x, p.y)).collect();
    let (areas, rim) = crate::canvas::area_share::area_shares_and_rim(&points);
    let (mut m, mut a) = (Vec::new(), Vec::new());
    for (i, (k, _)) in at.iter().enumerate() {
        if !rim[i] {
            m.push(masses[k]);
            a.push(areas[i]);
        }
    }
    crate::canvas::area_share::spearman(&m, &a)
}

/// Uniform mass from a clump: the density CV the law reaches, beside
/// Springs on the same seed, with the time to fall under 0.25.
#[test]
#[ignore = "fork probe: prints uniform-mass evenness"]
fn density_uniform_probe() {
    for n in [60u128, 200] {
        for law in [PhysicsLaw::Density, PhysicsLaw::Springs] {
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
            canvas.set_physics_law(law);
            let mut trail = Vec::new();
            let mut done = 0;
            for at in [2u32, 60, 180, 360, 900, 1800] {
                run(&mut canvas, at - done);
                done = at;
                trail.push(format!("{at}:{:.3}", canvas.layout_stats().density_cv));
            }
            println!("uniform n={n} {:?} cv {}", law, trail.join(" "));
        }
    }
}
