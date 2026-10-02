// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Density's receipts on the canvas: settled, room follows mass (Spearman of
//! mass against area share), uniform mass spreads evenly (a low density CV),
//! and Springs does not show the correlation (the negative control).

use super::*;
use crate::canvas::physics_catalog::{PhysicsLaw, PhysicsMassSource};
use kernel::graph::apply::{add_node, assert_relation};

/// The web pages' generated graph (`web_graphs::generated`): each node links
/// to one earlier node, and half as many extra links join random pairs.
pub(super) fn generated(nodes: usize, seed: u64) -> Graph {
    let mut graph = Graph::new();
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let keys: Vec<_> = (0..nodes)
        .map(|index| {
            let id = uuid::Uuid::from_u128((u128::from(seed) << 64) | index as u128);
            add_node(
                &mut graph,
                Some(id),
                format!("https://node-{index}.generated.test/"),
                PortablePoint::new(0.0, 0.0),
            )
        })
        .collect();
    for index in 1..nodes {
        let earlier = (next() % index as u64) as usize;
        assert_relation(&mut graph, keys[index], keys[earlier], hyperlink());
    }
    for _ in 0..nodes / 2 {
        let from = (next() % nodes as u64) as usize;
        let to = (next() % nodes as u64) as usize;
        if from != to {
            assert_relation(&mut graph, keys[from], keys[to], hyperlink());
        }
    }
    graph
}

/// A canvas over `graph`, free (no arrangement), seeded on a golden-angle
/// spiral `spacing` units apart: the boot phyllotaxis, without its anchors.
pub(super) fn seeded(graph: Graph, spacing: f32) -> Canvas {
    let mut canvas = Canvas::with_graph(graph);
    canvas.set_layout_strategy(None);
    let mut keys: Vec<NodeKey> = canvas.graph().nodes().map(|(k, _)| k).collect();
    keys.sort_by_key(|k| k.index());
    canvas.physics.seed(
        keys.iter()
            .enumerate()
            .map(|(i, &k)| {
                let a = i as f32 * 2.399_963;
                let r = spacing * (i as f32 + 0.5).sqrt();
                (k, Point2D::new(r * a.cos(), r * a.sin()))
            })
            .collect(),
    );
    canvas.physics.refresh(&mut canvas.view);
    canvas
}

/// Run `ticks` physics steps regardless of the settle budget.
pub(super) fn run(canvas: &mut Canvas, ticks: u32) {
    canvas.physics.settle(ticks);
    for _ in 0..ticks {
        canvas.physics.advance_frame(&mut canvas.view);
    }
}

/// The setting a receipt runs the law at, while the catalog's defaults are
/// an open fork (plan, P6 progress 2026-10-02): passes, seconds per pass,
/// grid resolution.
#[derive(Clone, Copy, Debug)]
pub(super) struct Run {
    pub passes: u32,
    pub seconds: f32,
    pub resolution: usize,
}

/// Several four-second passes on the full grid: the sample graph's setting.
pub(super) const SAMPLE_RUN: Run = Run {
    passes: 8,
    seconds: 4.0,
    resolution: 128,
};

/// Install Density at `setting` over the canvas's degree masses.
pub(super) fn install(canvas: &mut Canvas, setting: Run) {
    canvas.set_physics_law(PhysicsLaw::Density);
    let masses = canvas.law_inputs().masses(PhysicsMassSource::Degree);
    let mut law = seiche::Density::new(masses, setting.resolution);
    law.passes = setting.passes;
    law.seconds = setting.seconds;
    canvas.physics.set_forces(vec![Box::new(law)]);
}

/// Settle `graph` under Density (at `setting`) and under Springs from one
/// seed, and read both rank correlations.
fn density_and_springs_ranks(graph: Graph, setting: Run, ticks: u32) -> (f32, f32) {
    let mut density = seeded(graph.clone(), 40.0);
    install(&mut density, setting);
    run(&mut density, ticks);
    let mut springs = seeded(graph, 40.0);
    springs.set_physics_law(PhysicsLaw::Springs);
    run(&mut springs, ticks);
    let (d, s) = (density.layout_stats(), springs.layout_stats());
    eprintln!(
        "density rank {:.3} cv {:.3} overlaps {}; springs rank {:.3} overlaps {}",
        d.mass_area_rank, d.density_cv, d.overlaps, s.mass_area_rank, s.overlaps
    );
    (d.mass_area_rank, s.mass_area_rank)
}

/// The law's claim on the canvas sample graph: settled Density gives the
/// heavy nodes the room (Spearman >= 0.8), and Springs on the same seed
/// does not (the negative control).
#[test]
fn settled_density_gives_room_by_mass_and_springs_does_not() {
    let (density, springs) =
        density_and_springs_ranks(crate::canvas::build::sample_graph(), SAMPLE_RUN, 900);
    assert!(density >= 0.8, "density rank {density:.3}");
    assert!(
        springs < 0.8,
        "springs must fail the correlation, read {springs:.3}"
    );
}

/// The same claim on the web pages' 200-node generated graph, degree mass.
/// One pass reads about 0.60; one-second passes, repeated, cross 0.8 near
/// 3 600 ticks on a 64² grid (the probe's tables). The setting is named
/// here because the settle and the defaults are open forks.
#[test]
fn settled_density_gives_room_by_mass_on_the_generated_graph() {
    let setting = Run {
        passes: 120,
        seconds: 1.0,
        resolution: 64,
    };
    let (density, springs) = density_and_springs_ranks(generated(200, 7), setting, 3_600);
    assert!(density >= 0.8, "density rank {density:.3}");
    assert!(
        springs < 0.8,
        "springs must fail the correlation, read {springs:.3}"
    );
}

/// Uniform mass (every node alone, so degree mass is one apiece): Density
/// spreads a clumped seed evenly, a low CV of density.
#[test]
fn uniform_mass_spreads_evenly() {
    let mut graph = Graph::new();
    for i in 0..60 {
        add_node(
            &mut graph,
            Some(uuid::Uuid::from_u128(i + 1)),
            format!("https://u{i}.test/"),
            PortablePoint::new(0.0, 0.0),
        );
    }
    let mut canvas = seeded(graph, 14.0);
    install(&mut canvas, SAMPLE_RUN);
    run(&mut canvas, 2);
    let before = canvas.layout_stats().density_cv;
    run(&mut canvas, 900);
    let after = canvas.layout_stats();
    assert!(
        after.density_cv < 0.25,
        "uniform mass cv {:.3} (seed {before:.3})",
        after.density_cv
    );
    assert!(
        after.density_cv < before,
        "evener than the clump: {:.3} vs {before:.3}",
        after.density_cv
    );
    assert_eq!(after.overlaps, 0, "no overlaps once spread");
}
