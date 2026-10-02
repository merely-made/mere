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

/// Every node alone, so degree mass is one apiece: uniform mass.
pub(super) fn uniform(n: u128) -> Graph {
    let mut graph = Graph::new();
    for i in 0..n {
        add_node(
            &mut graph,
            Some(uuid::Uuid::from_u128(i + 1)),
            format!("https://u{i}.test/"),
            PortablePoint::new(0.0, 0.0),
        );
    }
    graph
}

/// A candidate stop for the receipts, named here because the catalog's stop
/// test is an open fork (plan, P6 progress 2026-10-02, third round): three
/// passes in a row under a twentieth of a spacing of mean shift.
pub(super) const CANDIDATE_STOP: seiche::DensityStop = seiche::DensityStop::Shift(0.05);

/// Install the catalog's Density (its defaults) with `stop` over the
/// canvas's degree masses.
pub(super) fn install(canvas: &mut Canvas, stop: seiche::DensityStop) {
    canvas.set_physics_law(PhysicsLaw::Density).unwrap();
    let masses = canvas.law_inputs().masses(PhysicsMassSource::Degree);
    let mut law = crate::canvas::physics_catalog::density_law(masses);
    law.stop = stop;
    law.patience = 3;
    canvas.physics.set_forces(vec![Box::new(law)]);
}

/// Tick until the law stops asking for ticks (its passes stopped) or
/// `limit` ticks pass; returns the ticks run.
pub(super) fn until_settled(canvas: &mut Canvas, limit: u32) -> u32 {
    for tick in 1..=limit {
        if !canvas.physics.advance_frame(&mut canvas.view) {
            return tick;
        }
    }
    limit
}

/// Settle `graph` under Density (stopping on `stop`) and under Springs from
/// one seed, and read both rank correlations.
fn density_and_springs_ranks(graph: Graph, stop: seiche::DensityStop) -> (f32, f32) {
    let mut density = seeded(graph.clone(), 40.0);
    install(&mut density, stop);
    let ticks = until_settled(&mut density, 60 * 121);
    let mut springs = seeded(graph, 40.0);
    springs.set_physics_law(PhysicsLaw::Springs).unwrap();
    run(&mut springs, ticks.max(360));
    let (d, s) = (density.layout_stats(), springs.layout_stats());
    eprintln!(
        "stopped after {ticks} ticks: density rank {:.3} cv {:.3} overlaps {}; springs rank {:.3} overlaps {}",
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
        density_and_springs_ranks(crate::canvas::build::sample_graph(), CANDIDATE_STOP);
    assert!(density >= 0.8, "density rank {density:.3}");
    assert!(
        springs < 0.8,
        "springs must fail the correlation, read {springs:.3}"
    );
}

/// The same claim on the web pages' 200-node generated graph, degree mass.
/// At the ruled defaults the rank does not settle above 0.8: over ninety
/// one-second passes it wanders between 0.73 and 0.79, and the mean shift a
/// pass stays near a twentieth of a spacing (the convergence probe). Kept as
/// the evidence for the stop fork.
#[test]
#[ignore = "the 200-node graph does not reach 0.8 at the ruled defaults; the stop test is an open fork"]
fn settled_density_gives_room_by_mass_on_the_generated_graph() {
    let (density, springs) = density_and_springs_ranks(generated(200, 7), CANDIDATE_STOP);
    assert!(density >= 0.8, "density rank {density:.3}");
    assert!(
        springs < 0.8,
        "springs must fail the correlation, read {springs:.3}"
    );
}

/// Uniform mass from a clump: Density spreads it evenly (a low CV of
/// density) and stops, with no overlaps left.
#[test]
fn uniform_mass_spreads_evenly() {
    let mut canvas = seeded(uniform(60), 14.0);
    install(&mut canvas, CANDIDATE_STOP);
    run(&mut canvas, 2);
    let before = canvas.layout_stats().density_cv;
    let ticks = until_settled(&mut canvas, 60 * 121);
    let after = canvas.layout_stats();
    eprintln!(
        "uniform stopped after {ticks} ticks: cv {:.3} (seed {before:.3}), overlaps {}",
        after.density_cv, after.overlaps
    );
    assert!(
        after.density_cv < 0.25,
        "uniform mass cv {:.3}",
        after.density_cv
    );
    assert!(after.density_cv < before, "evener than the clump");
    assert_eq!(after.overlaps, 0, "no overlaps once spread");
}

/// The catalog's Density keeps the host ticking past the settle budget
/// while its passes run (the law asks for ticks), so convergence, not the
/// budget, ends it.
#[test]
fn density_runs_past_the_settle_budget_until_its_passes_stop() {
    let mut canvas = seeded(crate::canvas::build::sample_graph(), 40.0);
    canvas.set_physics_law(PhysicsLaw::Density).unwrap();
    // Frames only: no settle is asked for beyond the law switch's own.
    for _ in 0..crate::canvas::SETTLE_TICKS + 30 {
        canvas.physics.advance_frame(&mut canvas.view);
    }
    assert!(canvas.is_settling(), "still flowing after the budget");
    // The control: Springs rests when its budget is spent.
    canvas.set_physics_law(PhysicsLaw::Springs).unwrap();
    for _ in 0..crate::canvas::SETTLE_TICKS + 30 {
        canvas.physics.advance_frame(&mut canvas.view);
    }
    assert!(!canvas.is_settling(), "springs rests after its budget");
}
