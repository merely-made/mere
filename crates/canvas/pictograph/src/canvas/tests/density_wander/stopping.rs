// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

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
                super::super::density::until_settled(&mut canvas, 60 * 122);
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
                canvas.pick_law(PhysicsLaw::Springs).unwrap();
                run(&mut canvas, 900);
                canvas.layout_stats().mass_area_rank
            })
            .collect();
        println!("STOPV {name} | springs control | ranks {springs:?}");
    }
}
