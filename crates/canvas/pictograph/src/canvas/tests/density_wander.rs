// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The wander probe: what changes pass to pass under Density's repeated
//! passes, and which formulations damp it. Per pass: the rank, density CV,
//! contacts, the flow's own shift beside what rapier moved, and shift by
//! mass tercile and rim. Ignored; run optimized with `--ignored --nocapture`.

use std::collections::HashMap;
use std::sync::Arc;

use super::density::{generated, run, seeded};
use super::*;
use crate::canvas::physics_catalog::{
    DENSITY_RESOLUTION, LawSources, PhysicsLaw, PhysicsMassSource,
};
use seiche::{Density, DensityGrid, DensityStop, Force, ForceContext};

struct Shared(Arc<Density>);

impl Force for Shared {
    fn apply(&self, ctx: &mut ForceContext<'_>, dt: f32) {
        self.0.apply(ctx, dt);
    }

    fn wants_tick(&self) -> bool {
        self.0.wants_tick()
    }
}

/// A formulation under test.
#[derive(Clone, Copy, Debug)]
pub(super) struct Form {
    pub name: &'static str,
    pub resolution: usize,
    pub blur: f32,
    pub decay: f32,
    pub quench: bool,
    pub inset: f32,
    pub relax: f32,
    pub renew: Option<f32>,
    /// The walls' centre moved this far along x, world units.
    pub shift: f32,
}

pub(super) const BASE: Form = Form {
    name: "base",
    resolution: DENSITY_RESOLUTION,
    blur: 0.25,
    decay: 1.0,
    quench: false,
    inset: 18.0,
    relax: 1.0,
    renew: None,
    shift: 0.0,
};

pub(super) fn law(canvas: &Canvas, form: Form, stop: DensityStop, passes: u32) -> Arc<Density> {
    let masses = canvas.law_inputs().masses(PhysicsMassSource::Degree);
    let mut d = Density::with_medium(masses, Box::new(DensityGrid::new(form.resolution)));
    d.seconds = 1.0;
    d.initial_blur = form.blur;
    d.decay = form.decay;
    d.quench = form.quench;
    d.wall_inset = form.inset;
    d.relax = form.relax;
    d.renew = form.renew;
    d.centre = (form.shift, 0.0);
    d.stop = stop;
    d.patience = 3;
    d.max_passes = passes;
    Arc::new(d)
}

pub(super) fn install(canvas: &mut Canvas, law: &Arc<Density>) {
    canvas.set_physics_law(PhysicsLaw::Density).unwrap();
    canvas
        .physics
        .set_forces(vec![Box::new(Shared(law.clone()))]);
}

fn positions(canvas: &Canvas) -> Vec<(NodeKey, Point2D<f32>)> {
    let mut at: Vec<_> = canvas.view.positions().collect();
    at.sort_by_key(|(k, _)| k.index());
    at
}

fn near_contacts(at: &[(NodeKey, Point2D<f32>)]) -> usize {
    let diameter = 2.0 * crate::canvas::NODE_HALF;
    let mut n = 0;
    for i in 0..at.len() {
        for j in i + 1..at.len() {
            if (at[i].1 - at[j].1).length() < 1.1 * diameter {
                n += 1;
            }
        }
    }
    n
}

/// One run: the per-pass trace, and where the ruled stop would have landed.
fn trace(name: &str, graph: &Graph, form: Form, passes: u32, every: u32) {
    let mut canvas = seeded(graph.clone(), 40.0);
    let law = law(&canvas, form, DensityStop::Cap, passes + 1);
    install(&mut canvas, &law);
    let masses: HashMap<NodeKey, f32> = canvas
        .law_inputs()
        .masses(PhysicsMassSource::Degree)
        .into_iter()
        .collect();
    let mut order: Vec<NodeKey> = masses.keys().copied().collect();
    order.sort_by(|a, b| {
        masses[a]
            .total_cmp(&masses[b])
            .then(a.index().cmp(&b.index()))
    });
    let tercile: HashMap<NodeKey, usize> = order
        .iter()
        .enumerate()
        .map(|(i, k)| (*k, i * 3 / order.len().max(1)))
        .collect();
    let started = std::time::Instant::now();
    let mut stats_time = std::time::Duration::ZERO;
    run(&mut canvas, 1);
    let mut before = positions(&canvas);
    let mut ranks = Vec::new();
    for pass in 1..=passes {
        run(&mut canvas, 60);
        let t = std::time::Instant::now();
        let at = positions(&canvas);
        let stats = canvas.layout_stats();
        ranks.push(stats.mass_area_rank);
        if pass % every == 0 || pass <= 3 {
            let points: Vec<(f32, f32)> = at.iter().map(|(_, p)| (p.x, p.y)).collect();
            let (_, rim) = crate::canvas::area_share::area_shares_and_rim(&points);
            let mut by_tercile = [(0.0f32, 0usize); 3];
            let mut by_rim = [(0.0f32, 0usize); 2];
            for (i, ((k, p), (_, q))) in at.iter().zip(&before).enumerate() {
                let d = (*p - *q).length();
                let t = &mut by_tercile[tercile[k]];
                t.0 += d;
                t.1 += 1;
                let r = &mut by_rim[usize::from(rim[i])];
                r.0 += d;
                r.1 += 1;
            }
            let mean = |(s, n): (f32, usize)| s / n.max(1) as f32;
            let record = law.pass_history().into_iter().find(|r| r.pass == pass);
            let (shift, flow, residual, fcv) = record.map_or((0.0, 0.0, 0.0, 0.0), |r| {
                (r.shift, r.flow_shift, r.residual, r.field_cv)
            });
            println!(
                "{name} | {} | {pass} | rank {:.3} cv {:.3} ov {} near {} | shift {:.4} flow {:.4} resid {:.4} fcv {:.3} | px by mass lo/mid/hi {:.1}/{:.1}/{:.1} interior/rim {:.1}/{:.1}",
                form.name,
                stats.mass_area_rank,
                stats.density_cv,
                stats.overlaps,
                near_contacts(&at),
                shift,
                flow,
                residual,
                fcv,
                mean(by_tercile[0]),
                mean(by_tercile[1]),
                mean(by_tercile[2]),
                mean(by_rim[0]),
                mean(by_rim[1]),
            );
        }
        stats_time += t.elapsed();
        before = at;
    }
    let ms = (started.elapsed() - stats_time).as_secs_f64() * 1000.0 / f64::from(passes * 60 + 1);
    // Where the ruled stop (shift < 0.05 for three passes) lands on this trace.
    let history = law.pass_history();
    let mut calm = 0;
    let stop = history.iter().find(|r| {
        calm = if r.shift < 0.05 { calm + 1 } else { 0 };
        calm >= 3
    });
    let tail = &ranks[(passes as usize / 3).min(ranks.len())..];
    let (lo, hi) = tail
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), r| (lo.min(*r), hi.max(*r)));
    println!(
        "SUMMARY {name} | {} | stop {} | rank at stop {} | final {:.3} | rank over last two thirds {lo:.3}..{hi:.3} | ms/tick {ms:.2}",
        form.name,
        stop.map_or("never".to_string(), |r| format!("pass {}", r.pass)),
        stop.map_or("-".to_string(), |r| format!(
            "{:.3}",
            ranks[(r.pass - 1) as usize]
        )),
        ranks.last().copied().unwrap_or(0.0),
    );
}

fn forms() -> Vec<Form> {
    let only = std::env::var("WANDER_FORMS").unwrap_or_default();
    let all = vec![
        BASE,
        Form {
            name: "quench",
            quench: true,
            ..BASE
        },
        Form {
            name: "decay0.9",
            decay: 0.9,
            ..BASE
        },
        Form {
            name: "decay0.8",
            decay: 0.8,
            ..BASE
        },
        Form {
            name: "grid128",
            resolution: 128,
            ..BASE
        },
        Form {
            name: "blur0.5",
            blur: 0.5,
            ..BASE
        },
        Form {
            name: "decay0.9+quench",
            decay: 0.9,
            quench: true,
            ..BASE
        },
        Form {
            name: "inset0",
            inset: 0.0,
            ..BASE
        },
        Form {
            name: "inset0+blur0.5",
            inset: 0.0,
            blur: 0.5,
            ..BASE
        },
        Form {
            name: "inset0+decay0.9",
            inset: 0.0,
            decay: 0.9,
            ..BASE
        },
        Form {
            name: "relax1.5",
            relax: 1.5,
            ..BASE
        },
        Form {
            name: "relax1.8",
            relax: 1.8,
            ..BASE
        },
        Form {
            name: "inset0+relax1.8",
            inset: 0.0,
            relax: 1.8,
            ..BASE
        },
        Form {
            name: "grid128+relax1.8",
            resolution: 128,
            relax: 1.8,
            ..BASE
        },
        Form {
            name: "renew-L0.5",
            renew: Some(0.5),
            ..BASE
        },
        Form {
            name: "renew-L1",
            renew: Some(1.0),
            ..BASE
        },
        Form {
            name: "renew-L2",
            renew: Some(2.0),
            ..BASE
        },
    ];
    all.into_iter()
        .filter(|f| only.is_empty() || only.split(',').any(|n| n == f.name))
        .collect()
}

#[test]
#[ignore = "wander probe: prints per-pass traces"]
fn density_wander_probe() {
    let graphs = [
        ("sample-12", crate::canvas::build::sample_graph()),
        ("gen-50", generated(50, 3)),
        ("gen-200", generated(200, 7)),
    ];
    for form in forms() {
        for (name, graph) in &graphs {
            trace(name, graph, form, 90, 10);
        }
    }
}

/// The negative control on the same seeds: Springs' rank.
#[test]
#[ignore = "wander probe: Springs control"]
fn density_wander_springs_control() {
    for (name, graph) in [
        ("sample-12", crate::canvas::build::sample_graph()),
        ("gen-50", generated(50, 3)),
        ("gen-200", generated(200, 7)),
    ] {
        let mut canvas = seeded(graph, 40.0);
        let forces = canvas.law_inputs().law_forces(
            PhysicsLaw::Springs,
            LawSources {
                kind: crate::canvas::PhysicsKindSource::Site,
                mass: PhysicsMassSource::Degree,
                depth: crate::canvas::PhysicsDepthSource::Roots,
                focus: None,
            },
        );
        canvas.physics.set_forces(forces);
        run(&mut canvas, 1800);
        println!(
            "SPRINGS {name} | rank {:.3}",
            canvas.layout_stats().mass_area_rank
        );
    }
}

/// A seed spiral turned by `turn` radians: the same spacing, another start.
fn seeded_turned(graph: Graph, turn: f32) -> Canvas {
    let mut canvas = Canvas::with_graph(graph);
    canvas.set_layout_strategy(None);
    let mut keys: Vec<NodeKey> = canvas.graph().nodes().map(|(k, _)| k).collect();
    keys.sort_by_key(|k| k.index());
    canvas.physics.seed(
        keys.iter()
            .enumerate()
            .map(|(i, &k)| {
                let a = i as f32 * 2.399_963 + turn;
                let r = 40.0 * (i as f32 + 0.5).sqrt();
                (k, Point2D::new(r * a.cos(), r * a.sin()))
            })
            .collect(),
    );
    canvas.physics.refresh(&mut canvas.view);
    canvas
}

/// The rank where the ruled stop lands, over eight starts per graph, for
/// each formulation: how much a small graph's reading depends on its start.
#[test]
#[ignore = "wander probe: rank at the stop over eight starts"]
fn density_start_probe() {
    let graphs = [
        ("sample-12", crate::canvas::build::sample_graph()),
        ("gen-50", generated(50, 3)),
        ("gen-200", generated(200, 7)),
    ];
    for form in forms() {
        for (name, graph) in &graphs {
            let mut ranks = Vec::new();
            for start in 0..8 {
                let mut canvas = seeded_turned(graph.clone(), start as f32 * 0.785);
                let law = law(&canvas, form, DensityStop::Shift(0.05), 120);
                install(&mut canvas, &law);
                super::density::until_settled(&mut canvas, 60 * 122);
                ranks.push(canvas.layout_stats().mass_area_rank);
            }
            let pass = ranks.iter().filter(|r| **r >= 0.8).count();
            let (lo, hi) = ranks
                .iter()
                .fold((f32::MAX, f32::MIN), |(lo, hi), r| (lo.min(*r), hi.max(*r)));
            let mean = ranks.iter().sum::<f32>() / ranks.len() as f32;
            println!(
                "STARTS {name} | {} | mean {mean:.3} range {lo:.3}..{hi:.3} | >= 0.8 in {pass}/8 | {:?}",
                form.name,
                ranks
                    .iter()
                    .map(|r| (r * 100.0).round() / 100.0)
                    .collect::<Vec<_>>()
            );
        }
    }
}

/// A seed spiral turned by `turn` radians with the nodes dealt onto its
/// points in a seeded order (`deal` 0 keeps key order): starts that no
/// symmetry of the square grid maps onto one another.
fn seeded_dealt(graph: Graph, turn: f32, deal: u64) -> Canvas {
    let mut canvas = Canvas::with_graph(graph);
    canvas.set_layout_strategy(None);
    let mut keys: Vec<NodeKey> = canvas.graph().nodes().map(|(k, _)| k).collect();
    keys.sort_by_key(|k| k.index());
    if deal != 0 {
        // Fisher–Yates on xorshift64, seeded per start.
        let mut state = deal.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        for i in (1..keys.len()).rev() {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            keys.swap(i, (state % (i as u64 + 1)) as usize);
        }
    }
    canvas.physics.seed(
        keys.iter()
            .enumerate()
            .map(|(i, &k)| {
                let a = i as f32 * 2.399_963 + turn;
                let r = 40.0 * (i as f32 + 0.5).sqrt();
                (k, Point2D::new(r * a.cos(), r * a.sin()))
            })
            .collect(),
    );
    canvas.physics.refresh(&mut canvas.view);
    canvas
}

fn rank_at_stop(mut canvas: Canvas, form: Form) -> (f32, u32) {
    let law = law(&canvas, form, DensityStop::Shift(0.05), 120);
    install(&mut canvas, &law);
    super::density::until_settled(&mut canvas, 60 * 122);
    let passes = law.pass_history().len() as u32;
    (canvas.layout_stats().mass_area_rank, passes)
}

/// The rank where the ruled stop lands over sixteen inequivalent starts:
/// golden-angle turns (no multiple of a quarter turn) and a seeded deal of
/// the nodes onto the spiral per start.
#[test]
#[ignore = "wander probe: rank at the stop over sixteen inequivalent starts"]
fn density_start_probe_inequivalent() {
    let graphs = [
        ("sample-12", crate::canvas::build::sample_graph()),
        ("gen-50", generated(50, 3)),
        ("gen-200", generated(200, 7)),
    ];
    for form in forms() {
        for (name, graph) in &graphs {
            let ranks: Vec<f32> = (0..16u64)
                .map(|k| {
                    rank_at_stop(
                        seeded_dealt(graph.clone(), k as f32 * 2.399_963, k + 1),
                        form,
                    )
                    .0
                })
                .collect();
            let pass = ranks.iter().filter(|r| **r >= 0.8).count();
            let (lo, hi) = ranks
                .iter()
                .fold((f32::MAX, f32::MIN), |(lo, hi), r| (lo.min(*r), hi.max(*r)));
            let mean = ranks.iter().sum::<f32>() / ranks.len() as f32;
            println!(
                "STARTS16 {name} | {} | mean {mean:.3} range {lo:.3}..{hi:.3} | >= 0.8 in {pass}/16 | {:?}",
                form.name,
                ranks
                    .iter()
                    .map(|r| (r * 100.0).round() / 100.0)
                    .collect::<Vec<_>>()
            );
        }
    }
}

/// The orientation control: one deal, the turn swept across a quarter turn
/// in nine steps. A smooth curve in the turn is the grid's or the walls'
/// anisotropy; a two-valued jump is two basins.
#[test]
#[ignore = "wander probe: rank against the seed's turn over a quarter turn"]
fn density_orientation_probe() {
    let graphs = [
        ("sample-12", crate::canvas::build::sample_graph()),
        ("gen-200", generated(200, 7)),
    ];
    for form in forms() {
        for (name, graph) in &graphs {
            let rows: Vec<String> = (0..9)
                .map(|j| {
                    let turn = j as f32 * std::f32::consts::FRAC_PI_2 / 8.0;
                    let (rank, passes) = rank_at_stop(seeded_dealt(graph.clone(), turn, 0), form);
                    format!("{:.1}°:{rank:.3}(p{passes})", turn.to_degrees())
                })
                .collect();
            println!("TURN {name} | {} | {}", form.name, rows.join(" "));
        }
    }
}

fn rank_under(canvas: Canvas, form: Form, stop: DensityStop) -> (f32, u32) {
    let mut canvas = canvas;
    let law = law(&canvas, form, stop, 120);
    install(&mut canvas, &law);
    super::density::until_settled(&mut canvas, 60 * 122);
    (
        canvas.layout_stats().mass_area_rank,
        law.pass_history().len() as u32,
    )
}

/// One cheap check per anisotropy candidate on the 200-node graph, the
/// aligned start (0°) against a turned one (22.5°): a finer grid, the
/// domain moved half a cell under the same seed, and a wider blur. Then,
/// on four dealt starts, the rank where the ruled stop lands against the
/// rank under the pass cap alone: an early stop, or the law's own rest.
#[test]
#[ignore = "wander probe: anisotropy checks and stop against cap"]
fn density_anisotropy_checks() {
    let graph = generated(200, 7);
    let checks = [
        BASE,
        Form {
            name: "grid128",
            resolution: 128,
            ..BASE
        },
        Form {
            name: "half-cell",
            shift: 11.0,
            ..BASE
        },
        Form {
            name: "blur0.5",
            blur: 0.5,
            ..BASE
        },
    ];
    for form in checks {
        let row: Vec<String> = [0.0f32, 22.5]
            .iter()
            .map(|deg| {
                let canvas = seeded_dealt(graph.clone(), deg.to_radians(), 0);
                let (rank, passes) = rank_under(canvas, form, DensityStop::Shift(0.05));
                format!("{deg}°:{rank:.3}(p{passes})")
            })
            .collect();
        println!("ANISO gen-200 | {} | {}", form.name, row.join(" "));
    }
    for (name, graph) in [("gen-50", generated(50, 3)), ("gen-200", generated(200, 7))] {
        for form in [
            BASE,
            Form {
                name: "renew-L2",
                renew: Some(2.0),
                ..BASE
            },
        ] {
            let row: Vec<String> = (1..=4u64)
                .map(|k| {
                    let start = || seeded_dealt(graph.clone(), k as f32 * 2.399_963, k);
                    let (stop, p) = rank_under(start(), form, DensityStop::Shift(0.05));
                    let (cap, _) = rank_under(start(), form, DensityStop::Cap);
                    format!("start {k}: stop {stop:.3}(p{p}) cap {cap:.3}")
                })
                .collect();
            println!("STOPCAP {name} | {} | {}", form.name, row.join(" | "));
        }
    }
}

/// The 12-node sample over eight dealt starts: the rank where the ruled stop
/// lands against the rank under the pass cap alone.
#[test]
#[ignore = "wander probe: the sample's stop against its cap over dealt starts"]
fn density_sample_stop_against_cap() {
    let graph = crate::canvas::build::sample_graph();
    for form in [
        BASE,
        Form {
            name: "renew-L2",
            renew: Some(2.0),
            ..BASE
        },
    ] {
        let (mut stops, mut caps) = (Vec::new(), Vec::new());
        for k in 1..=8u64 {
            let start = || seeded_dealt(graph.clone(), k as f32 * 2.399_963, k);
            stops.push(rank_under(start(), form, DensityStop::Shift(0.05)).0);
            caps.push(rank_under(start(), form, DensityStop::Cap).0);
        }
        let mean = |v: &[f32]| v.iter().sum::<f32>() / v.len() as f32;
        let r = |v: &[f32]| {
            v.iter()
                .map(|x| (x * 100.0).round() / 100.0)
                .collect::<Vec<_>>()
        };
        println!(
            "SAMPLECAP {} | stop mean {:.3} {:?} | cap mean {:.3} {:?} | cap >= 0.8 in {}/8",
            form.name,
            mean(&stops),
            r(&stops),
            mean(&caps),
            r(&caps),
            caps.iter().filter(|c| **c >= 0.8).count()
        );
    }
}

/// A stop under test: the shift test, its patience, and the passes before it
/// may end them, all under the 120-pass cap.
#[derive(Clone, Copy, Debug)]
struct StopVariant {
    name: &'static str,
    stop: DensityStop,
    min_passes: u32,
}

const STOP_VARIANTS: [StopVariant; 7] = [
    StopVariant {
        name: "ruled",
        stop: DensityStop::Shift(0.05),
        min_passes: 0,
    },
    StopVariant {
        name: "cap",
        stop: DensityStop::Cap,
        min_passes: 0,
    },
    StopVariant {
        name: "shift0.02",
        stop: DensityStop::Shift(0.02),
        min_passes: 0,
    },
    StopVariant {
        name: "shift0.01",
        stop: DensityStop::Shift(0.01),
        min_passes: 0,
    },
    StopVariant {
        name: "min20",
        stop: DensityStop::Shift(0.05),
        min_passes: 20,
    },
    StopVariant {
        name: "min40",
        stop: DensityStop::Shift(0.05),
        min_passes: 40,
    },
    StopVariant {
        name: "min60",
        stop: DensityStop::Shift(0.05),
        min_passes: 60,
    },
];

/// The later stops measured: every variant over sixteen dealt starts, the
/// rank, CV and overlaps where it stops against the seed's, and the passes
/// (seconds of flow) it took. `DENSITY_GRAPH` picks one graph.
#[test]
#[ignore = "wander probe: later stops over sixteen dealt starts"]
fn density_stop_variants() {
    let only = std::env::var("DENSITY_GRAPH").unwrap_or_default();
    let graphs = [
        ("sample-12", crate::canvas::build::sample_graph()),
        ("gen-50", generated(50, 3)),
        ("gen-200", generated(200, 7)),
    ];
    for (name, graph) in graphs.iter().filter(|(n, _)| only.is_empty() || *n == only) {
        for variant in STOP_VARIANTS {
            let mut rows = Vec::new();
            for k in 0..16u64 {
                let mut canvas = seeded_dealt(graph.clone(), k as f32 * 2.399_963, k + 1);
                let seed = canvas.layout_stats();
                let masses = canvas.law_inputs().masses(PhysicsMassSource::Degree);
                let mut law = crate::canvas::physics_catalog::density_law(masses);
                law.stop = variant.stop;
                law.patience = 3;
                law.min_passes = variant.min_passes;
                law.max_passes = 120;
                let law = Arc::new(law);
                install(&mut canvas, &law);
                super::density::until_settled(&mut canvas, 60 * 122);
                let end = canvas.layout_stats();
                let passes = law.pass_history().len() as u32;
                rows.push((seed, end, passes));
            }
            let ranks: Vec<f32> = rows.iter().map(|(_, e, _)| e.mass_area_rank).collect();
            let gain: Vec<f32> = rows
                .iter()
                .map(|(s, e, _)| e.mass_area_rank - s.mass_area_rank)
                .collect();
            let cv_fall = rows
                .iter()
                .filter(|(s, e, _)| e.density_cv < s.density_cv)
                .count();
            let passes: Vec<u32> = rows.iter().map(|(_, _, p)| *p).collect();
            let mean = |v: &[f32]| v.iter().sum::<f32>() / v.len() as f32;
            let min = ranks.iter().cloned().fold(f32::MAX, f32::min);
            let overlaps: usize = rows.iter().map(|(_, e, _)| e.overlaps).sum();
            println!(
                "STOPV {name} | {} | rank min {min:.3} mean {:.3} >=0.8 {}/16 >=0.7 {}/16 | gain over seed min {:.3} mean {:.3} (rose {}/16) | cv fell {}/16 | passes mean {:.1} max {} (seconds of flow) | overlaps {overlaps} | ranks {:?} | passes {:?}",
                variant.name,
                mean(&ranks),
                ranks.iter().filter(|r| **r >= 0.8).count(),
                ranks.iter().filter(|r| **r >= 0.7).count(),
                gain.iter().cloned().fold(f32::MAX, f32::min),
                mean(&gain),
                gain.iter().filter(|g| **g > 0.0).count(),
                cv_fall,
                passes.iter().sum::<u32>() as f32 / 16.0,
                passes.iter().max().unwrap_or(&0),
                ranks
                    .iter()
                    .map(|r| (r * 100.0).round() / 100.0)
                    .collect::<Vec<_>>(),
                passes,
            );
        }
        // The negative control on the same dealt starts: Springs' rank.
        let springs: Vec<f32> = (0..4u64)
            .map(|k| {
                let mut canvas = seeded_dealt(graph.clone(), k as f32 * 2.399_963, k + 1);
                canvas.set_physics_law(PhysicsLaw::Springs).unwrap();
                run(&mut canvas, 900);
                canvas.layout_stats().mass_area_rank
            })
            .collect();
        println!("STOPV {name} | springs control | ranks {springs:?}");
    }
}
