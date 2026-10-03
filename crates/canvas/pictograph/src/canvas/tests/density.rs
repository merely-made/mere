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

/// Settle `graph` under the catalog's Density until its flow reports it has
/// stopped, and under Springs from the same seed; read both ranks.
fn density_and_springs_ranks(graph: Graph) -> (f32, f32) {
    let mut density = seeded(graph.clone(), 40.0);
    density.set_physics_law(PhysicsLaw::Density).unwrap();
    let ticks = until_settled(&mut density, 60 * 122);
    assert!(
        !density.physics_tick_demand().0,
        "the flow reports stopped before the rank is read"
    );
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
    let (density, springs) = density_and_springs_ranks(crate::canvas::build::sample_graph());
    assert!(density >= 0.8, "density rank {density:.3}");
    assert!(
        springs < 0.8,
        "springs must fail the correlation, read {springs:.3}"
    );
}

/// The same claim on a 50-node generated graph (seed 3), degree mass. The
/// ruled stop ends it at pass 6 reading 0.786, under the 0.8 the same ruling
/// set for it; reopened with Mark (plan, P6 progress 2026-10-02, fourth
/// round).
#[test]
#[ignore = "reopened: the ruled stop ends gen-50 at 0.786, under its ruled 0.8 bar"]
fn settled_density_gives_room_by_mass_on_fifty_nodes() {
    let (density, springs) = density_and_springs_ranks(generated(50, 3));
    assert!(density >= 0.8, "density rank {density:.3}");
    assert!(
        springs < 0.8,
        "springs must fail the correlation, read {springs:.3}"
    );
}

/// The web pages' 200-node generated graph, degree mass, under the plateau
/// bar ruled 2026-10-02: the rank does not settle above 0.8 at 64² (over
/// ninety one-second passes it wanders 0.73 to 0.79, and the stop test ends
/// it near pass 18 at about 0.76), so the bar here is 0.7, to be revisited
/// with P6b's 512² grid.
#[test]
fn settled_density_holds_the_plateau_bar_on_the_generated_graph() {
    let (density, springs) = density_and_springs_ranks(generated(200, 7));
    assert!(
        density >= 0.7,
        "density rank {density:.3} under the plateau bar 0.7 (measured plateau 0.73 to 0.79)"
    );
    assert!(
        springs < 0.7,
        "springs must fail the correlation, read {springs:.3}"
    );
}

/// Uniform mass from a clump: Density spreads it evenly (a low CV of
/// density) and stops, with no overlaps left.
#[test]
fn uniform_mass_spreads_evenly() {
    let mut canvas = seeded(uniform(60), 14.0);
    canvas.set_physics_law(PhysicsLaw::Density).unwrap();
    run(&mut canvas, 2);
    let before = canvas.layout_stats().density_cv;
    let ticks = until_settled(&mut canvas, 60 * 122);
    assert!(!canvas.physics_tick_demand().0, "the flow reports stopped");
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
/// budget, ends it. The 200-node graph converges well past the budget.
#[test]
fn density_runs_past_the_settle_budget_until_its_passes_stop() {
    let mut canvas = seeded(generated(200, 7), 40.0);
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
