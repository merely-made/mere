// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Density's receipts on the canvas: settled, room follows mass (Spearman of
//! mass against area share), uniform mass spreads evenly (a low density CV),
//! and Springs does not show the correlation (the negative control).
//!
//! The bars run in release as ignored receipts ("Release bars, quick
//! default", ruled 2026-10-03), which Density-touching lanes and merges run:
//!
//! `cargo test --release -p pictograph --features canvas --lib tests::density:: -- --ignored --nocapture`
//!
//! The default suite keeps one quick check on the sample.

use super::*;
use crate::canvas::physics_catalog::PhysicsLaw;
use crate::canvas::tests::ThroughView;
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

/// A seed spiral turned by `turn` radians with the nodes dealt onto its
/// points in a seeded order (`deal` 0 keeps key order): starts no symmetry
/// of the square grid maps onto one another, and none that puts the hubs at
/// the centre (the round-six correction).
pub(super) fn seeded_dealt(graph: Graph, turn: f32, deal: u64) -> Canvas {
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

/// Dealt start `k` of the sixteen the bars were measured on.
pub(super) fn dealt(graph: &Graph, k: u64) -> Canvas {
    seeded_dealt(graph.clone(), k as f32 * 2.399_963, k + 1)
}

/// The sixteen dealt starts the bar was measured on (`density_stop_variants`).
/// A start costs about two minutes in a debug build and eight seconds in
/// release, so their receipts are release-only.
const ALL_STARTS: std::ops::Range<u64> = 0..16;

/// One start under the catalog's Density, run until its flow reports
/// stopped: the seed's stats, the stop's, and the ticks it took.
struct Settled {
    seed: crate::canvas::physics_catalog::LayoutStats,
    end: crate::canvas::physics_catalog::LayoutStats,
    ticks: u32,
}

fn settle_from(mut canvas: Canvas) -> Settled {
    let seed = canvas.layout_stats();
    canvas.pick_law(PhysicsLaw::Density).unwrap();
    let ticks = until_settled(&mut canvas, 60 * 122);
    assert!(
        !canvas.physics_tick_demand().0,
        "the flow reports stopped before the rank is read"
    );
    Settled {
        seed,
        end: canvas.layout_stats(),
        ticks,
    }
}

/// Springs from the same start: the negative control's rank.
fn springs_from(mut canvas: Canvas) -> f32 {
    canvas.pick_law(PhysicsLaw::Springs).unwrap();
    run(&mut canvas, 900);
    canvas.layout_stats().mass_area_rank
}

/// The bar ruled 2026-10-03 ("Min 60, bar: all >= 0.7") on a generated graph:
/// every dealt start at 0.7 or more where the stop lands, with the mean and
/// the count at 0.8 recorded; Springs reads negative on the same starts.
fn holds_the_bar(name: &str, graph: Graph, starts: std::ops::Range<u64>) {
    let mut ranks = Vec::new();
    for k in starts.clone() {
        let run = settle_from(dealt(&graph, k));
        let springs = springs_from(dealt(&graph, k));
        eprintln!(
            "{name} start {k}: rank {:.3} (seed {:.3}) cv {:.3} (seed {:.3}) overlaps {} after {} ticks; springs {springs:.3}",
            run.end.mass_area_rank,
            run.seed.mass_area_rank,
            run.end.density_cv,
            run.seed.density_cv,
            run.end.overlaps,
            run.ticks
        );
        assert!(
            springs < 0.0,
            "{name} start {k}: springs reads {springs:.3}"
        );
        ranks.push(run.end.mass_area_rank);
    }
    let mean = ranks.iter().sum::<f32>() / ranks.len() as f32;
    let at_eight = ranks.iter().filter(|r| **r >= 0.8).count();
    eprintln!(
        "{name}: mean {mean:.3}, {at_eight} of {} at 0.8 or more",
        ranks.len()
    );
    for (k, rank) in starts.zip(&ranks) {
        assert!(*rank >= 0.7, "{name} start {k}: rank {rank:.3} under 0.7");
    }
}

#[test]
#[ignore = "release receipt: the module doc's one line runs it"]
fn density_holds_the_bar_on_fifty_nodes_from_all_sixteen_starts() {
    holds_the_bar("gen-50", generated(50, 3), ALL_STARTS);
}

#[test]
#[ignore = "release receipt: the module doc's one line runs it"]
fn density_holds_the_bar_on_the_generated_graph_from_all_sixteen_starts() {
    holds_the_bar("gen-200", generated(200, 7), ALL_STARTS);
}

/// The sample, qualitatively (ruled 2026-10-03, "0.8 from 50 nodes up"): from
/// every dealt start the rank rises above the seed's and the density CV
/// falls; the per-start values go to the output.
fn rises_and_evens(starts: std::ops::Range<u64>) {
    let graph = crate::canvas::build::sample_graph();
    for k in starts {
        let run = settle_from(dealt(&graph, k));
        eprintln!(
            "sample start {k}: rank {:.3} (seed {:.3}) cv {:.3} (seed {:.3}) after {} ticks",
            run.end.mass_area_rank,
            run.seed.mass_area_rank,
            run.end.density_cv,
            run.seed.density_cv,
            run.ticks
        );
        assert!(
            run.end.mass_area_rank > run.seed.mass_area_rank,
            "sample start {k}: rank {:.3} not above the seed's {:.3}",
            run.end.mass_area_rank,
            run.seed.mass_area_rank
        );
        assert!(
            run.end.density_cv < run.seed.density_cv,
            "sample start {k}: cv {:.3} not under the seed's {:.3}",
            run.end.density_cv,
            run.seed.density_cv
        );
    }
}

/// The default suite's one Density check: the sample from dealt start 0,
/// run past the settle budget by frames alone. The law still asks for ticks
/// there (its passes are not done), the rank has risen above the seed's and
/// the CV fallen; Springs from the same start rests once its budget is spent
/// (the control).
#[test]
fn density_on_the_sample_rises_and_evens_past_the_settle_budget() {
    let graph = crate::canvas::build::sample_graph();
    let mut canvas = dealt(&graph, 0);
    let seed = canvas.layout_stats();
    canvas.pick_law(PhysicsLaw::Density).unwrap();
    // Frames only: no settle is asked for beyond the law switch's own.
    for _ in 0..crate::canvas::SETTLE_TICKS + 30 {
        canvas.physics.advance_frame(&mut canvas.view);
    }
    assert!(canvas.is_settling(), "still flowing after the budget");
    let now = canvas.layout_stats();
    eprintln!(
        "sample start 0 after {} ticks: rank {:.3} (seed {:.3}) cv {:.3} (seed {:.3})",
        crate::canvas::SETTLE_TICKS + 30,
        now.mass_area_rank,
        seed.mass_area_rank,
        now.density_cv,
        seed.density_cv
    );
    assert!(
        now.mass_area_rank > seed.mass_area_rank,
        "rank {:.3} not above the seed's {:.3}",
        now.mass_area_rank,
        seed.mass_area_rank
    );
    assert!(
        now.density_cv < seed.density_cv,
        "cv {:.3} not under the seed's {:.3}",
        now.density_cv,
        seed.density_cv
    );
    let mut springs = dealt(&graph, 0);
    springs.pick_law(PhysicsLaw::Springs).unwrap();
    for _ in 0..crate::canvas::SETTLE_TICKS + 30 {
        springs.physics.advance_frame(&mut springs.view);
    }
    assert!(!springs.is_settling(), "springs rests after its budget");
}

#[test]
#[ignore = "release receipt: the module doc's one line runs it"]
fn density_on_the_sample_rises_and_evens_from_all_sixteen_starts() {
    rises_and_evens(ALL_STARTS);
}

/// Uniform mass from a clump: Density spreads it evenly (a low CV of
/// density) and stops, with no overlaps left.
#[test]
#[ignore = "release receipt: the module doc's one line runs it"]
fn uniform_mass_spreads_evenly() {
    let mut canvas = seeded(uniform(60), 14.0);
    canvas.pick_law(PhysicsLaw::Density).unwrap();
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
