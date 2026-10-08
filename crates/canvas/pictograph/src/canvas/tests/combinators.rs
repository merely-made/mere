// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! G3's catalog receipts (dynamics grammar plan): every law's currency and
//! what it does with each overlay, the common scale's coverage of the
//! catalog's terms, and the weighted mix of two force laws against each
//! pure law.

use seiche::instruments::{Probe, Vector};
use seiche::{Admission, Currency, Declared, Force, ForceContext, Term, compose, scale};

use super::*;
use crate::canvas::composition::{GroupSource, PhysicsComposition, PhysicsGrouping};
use crate::canvas::physics_catalog::{
    LawInputs, LawSources, PhysicsKindSource, PhysicsLaw, PhysicsMassSource, PhysicsOverlay,
};

/// The ring's keys, edges and sites.
type Ring = (
    Vec<NodeKey>,
    Vec<(NodeKey, NodeKey)>,
    HashMap<NodeKey, String>,
);

/// Twenty-four nodes on four sites: a ring with every fourth node joined
/// across, so the laws have structure to act on.
fn ring() -> Ring {
    let keys: Vec<NodeKey> = (0..24).map(NodeKey::new).collect();
    let mut edges: Vec<(NodeKey, NodeKey)> =
        (0..24).map(|i| (keys[i], keys[(i + 1) % 24])).collect();
    edges.extend((0..24).step_by(4).map(|i| (keys[i], keys[(i + 12) % 24])));
    let sites = keys
        .iter()
        .enumerate()
        .map(|(i, &k)| (k, format!("s{}", i % 4)))
        .collect();
    (keys, edges, sites)
}

/// The sum of a force list, read as one force by the probe.
struct Sum(Vec<Box<dyn Force>>);

impl Force for Sum {
    fn apply(&self, ctx: &mut ForceContext<'_>, dt: f32) {
        for force in &self.0 {
            force.apply(ctx, dt);
        }
    }
}

impl Declared for Sum {
    fn terms(&self) -> Vec<Term> {
        self.0.iter().flat_map(|f| f.terms()).collect()
    }
}

#[test]
fn every_law_reports_its_currency() {
    let mut table = Vec::new();
    for law in PhysicsLaw::ALL {
        let currency = law.currency();
        table.push(format!("{}: {currency:?}", law.id()));
        let expected = match law {
            PhysicsLaw::Anneal | PhysicsLaw::Still | PhysicsLaw::Density => Currency::Kinematic,
            _ => Currency::Force,
        };
        assert_eq!(currency, expected, "{}", law.id());
    }
    println!("{}", table.join("; "));
}

/// Law by overlay: a force law composes every overlay, Still and Anneal
/// take them converted, and Density takes Hub room, Centre and Tide
/// converted and refuses the rest (F73, F81), with the reason both pickers
/// show.
/// Through the canvas, Density with an overlay runs its conversion and
/// Density alone does not.
#[test]
fn the_catalog_composes_converts_or_refuses_each_overlay_by_currency() {
    use crate::canvas::physics_catalog::{DENSITY_ADMITS, DENSITY_REFUSAL};
    for law in PhysicsLaw::ALL {
        for overlay in PhysicsOverlay::ALL {
            let expected = match law.currency() {
                Currency::Force => Admission::Compose,
                _ if law == PhysicsLaw::Density && !DENSITY_ADMITS.contains(&overlay) => {
                    Admission::Refuse(DENSITY_REFUSAL)
                },
                _ => Admission::Convert,
            };
            assert_eq!(
                law.admits(overlay),
                expected,
                "{} with {}",
                law.id(),
                overlay.id()
            );
        }
    }
    assert_eq!(
        DENSITY_ADMITS,
        &[
            PhysicsOverlay::DegreeRepulsion,
            PhysicsOverlay::GravityLocus,
            PhysicsOverlay::Tide
        ]
    );
    // Density with Centre, as the catalog builds it, runs exactly as Density
    // converting with Centre, and apart from Density not converting (the
    // control): its conversion is on once it takes an overlay.
    let (keys, edges, sites) = ring();
    let inputs = LawInputs::from_parts(keys.clone(), edges.clone(), sites);
    let sources = LawSources::bare();
    let run = |forces: Vec<Box<dyn Force>>| {
        let mut sim = seiche::Simulation::new();
        sim.sync_nodes(keys.iter().enumerate().map(|(i, &k)| {
            let a = i as f32 * 2.399_963;
            let r = 20.0 * (i as f32 + 1.0).sqrt();
            (k, euclid::default::Point2D::new(r * a.cos(), r * a.sin()))
        }));
        sim.sync_edges(edges.iter().copied());
        sim.set_forces(forces);
        for _ in 0..120 {
            sim.tick(1.0 / 60.0);
        }
        sim.positions().collect::<HashMap<_, _>>()
    };
    let by_hand = |converts: bool| {
        let mut density = crate::canvas::physics_catalog::density_law(inputs.masses(sources.mass));
        density.converts = converts;
        vec![
            Box::new(density) as Box<dyn Force>,
            inputs.overlay_force(PhysicsOverlay::GravityLocus, sources),
        ]
    };
    let catalog = run(inputs.forces(
        PhysicsLaw::Density,
        &[PhysicsOverlay::GravityLocus],
        sources,
    ));
    let converting = run(by_hand(true));
    let not_converting = run(by_hand(false));
    let apart = |a: &HashMap<NodeKey, euclid::default::Point2D<f32>>,
                 b: &HashMap<NodeKey, euclid::default::Point2D<f32>>| {
        a.iter()
            .map(|(k, p)| (*p - b[k]).length())
            .fold(0.0f32, f32::max)
    };
    println!(
        "density with centre after 120 ticks: catalog against converting {:.4}, against not \
         converting {:.4}",
        apart(&catalog, &converting),
        apart(&catalog, &not_converting)
    );
    assert_eq!(apart(&catalog, &converting), 0.0);
    assert!(apart(&catalog, &not_converting) > 1e-2);
    let mut canvas = Canvas::with_sample_graph();
    canvas
        .set_physics_overlays(vec![PhysicsOverlay::GravityLocus, PhysicsOverlay::Skeleton])
        .unwrap();
    let refusal = canvas.set_physics_law(PhysicsLaw::Density).unwrap_err();
    assert_eq!(refusal.refused, vec![PhysicsOverlay::Skeleton]);
    assert_eq!(refusal.reason, DENSITY_REFUSAL);
    assert_eq!(canvas.physics_overlays(), &[PhysicsOverlay::GravityLocus]);
    canvas.set_physics_law(PhysicsLaw::Still).unwrap();
    canvas
        .set_physics_overlays(vec![PhysicsOverlay::GravityLocus, PhysicsOverlay::Skeleton])
        .unwrap();
    assert_eq!(
        canvas.physics_overlays(),
        &[PhysicsOverlay::GravityLocus, PhysicsOverlay::Skeleton]
    );
}

/// A mix or a grouping holds force laws only, refused with the reason
/// otherwise, and a pick replaces a composition.
#[test]
fn a_composition_takes_force_laws_only_and_a_pick_replaces_it() {
    let mut canvas = Canvas::with_sample_graph();
    let mix = PhysicsComposition::Mix(vec![(PhysicsLaw::Springs, 1.0), (PhysicsLaw::Still, 1.0)]);
    let refusal = canvas.set_physics_composition(Some(mix)).unwrap_err();
    assert_eq!(refusal.law, PhysicsLaw::Still);
    assert_eq!(refusal.reason, compose::UNWEIGHTED);
    let grouped = PhysicsComposition::Grouped(PhysicsGrouping {
        groups: GroupSource::Given(Vec::new()),
        outer: PhysicsLaw::Charge,
        inner: PhysicsLaw::Anneal,
        outer_weight: 1.0,
    });
    assert_eq!(
        canvas
            .set_physics_composition(Some(grouped))
            .unwrap_err()
            .law,
        PhysicsLaw::Anneal
    );
    assert!(canvas.physics_composition().is_none());
    let mix = PhysicsComposition::Mix(vec![(PhysicsLaw::Springs, 0.5), (PhysicsLaw::Charge, 0.5)]);
    canvas.set_physics_composition(Some(mix.clone())).unwrap();
    assert_eq!(canvas.physics_composition(), Some(&mix));
    canvas.set_physics_law(PhysicsLaw::Stress).unwrap();
    assert!(canvas.physics_composition().is_none(), "a pick replaces it");
}

/// Every term the catalog builds is on the common scale, or its kernel
/// depends on no distance (F80); none has a reference and reports no scale.
/// Grid reads at its half cell (F79). The lists are printed.
#[test]
fn every_catalog_term_is_on_the_common_scale_or_named_outside_it() {
    let (keys, edges, sites) = ring();
    let inputs = LawInputs::from_parts(keys.clone(), edges, sites);
    let sources = LawSources {
        kind: PhysicsKindSource::Site,
        mass: PhysicsMassSource::Degree,
        ..LawSources::bare()
    };
    let mut forces: Vec<(String, Box<dyn Force>)> = Vec::new();
    for law in PhysicsLaw::ALL {
        for (k, force) in inputs.law_forces(law, sources).into_iter().enumerate() {
            forces.push((format!("{}[{k}]", law.id()), force));
        }
    }
    for overlay in PhysicsOverlay::ALL {
        forces.push((
            overlay.id().to_string(),
            inputs.overlay_force(overlay, sources),
        ));
    }
    let slots: Vec<_> = keys.iter().map(|&k| (k, (0.0, 0.0))).collect();
    forces.push((
        "slot/anchor".into(),
        Box::new(seiche::AnchorSpring::new(slots)),
    ));
    forces.push((
        "slot/affinity".into(),
        Box::new(seiche::AffinitySpring::new([(keys[0], keys[2], 1.0)])),
    ));
    let (mut scaled, mut unnamed, mut uncovered) = (Vec::new(), Vec::new(), Vec::new());
    for (name, force) in &forces {
        for (t, term) in force.terms().iter().enumerate() {
            let row = format!("{name} {}", term.name);
            match (force.scale(t), scale::family(term)) {
                (Some(s), family) => {
                    assert_eq!(Some(s.reference.family()), family, "{row}");
                    scaled.push(format!("{row} {:.3}", s.weight));
                },
                (None, None) => unnamed.push(format!("{row} ({:?})", term.kernel)),
                (None, Some(_)) => uncovered.push(row),
            }
        }
    }
    println!("on the common scale, by weight: {}", scaled.join("; "));
    println!("kernels F5 names no reference for: {}", unnamed.join("; "));
    println!("a reference, but none reported: {}", uncovered.join("; "));
    assert!(uncovered.is_empty(), "{uncovered:?}");
}

/// The weighted sum, instant by instant: on the ring at a seeded scatter, a
/// mix of Springs and Charge pushes with exactly `w₁·Springs + w₂·Charge`,
/// so at 1/0 and 0/1 it is each pure law.
#[test]
fn a_weighted_mix_pushes_as_each_pure_law_at_one_and_zero() {
    let (keys, edges, sites) = ring();
    let inputs = LawInputs::from_parts(keys.clone(), edges.clone(), sites);
    let sources = LawSources::bare();
    let mut probe = Probe::new(&keys, &edges);
    let at = probe.scatter(3, 400.0);
    let mut read = |forces: Vec<Box<dyn Force>>| -> Vec<Vector> {
        probe.place(&at);
        probe.forces(&Sum(forces))
    };
    let springs = read(inputs.law_forces(PhysicsLaw::Springs, sources));
    let charge = read(inputs.law_forces(PhysicsLaw::Charge, sources));
    let mixed = |w1: f32, w2: f32| -> Vec<Box<dyn Force>> {
        let mut forces: Vec<Box<dyn Force>> = Vec::new();
        for (law, w) in [(PhysicsLaw::Springs, w1), (PhysicsLaw::Charge, w2)] {
            for force in inputs.law_forces(law, sources) {
                forces.push(Box::new(seiche::Weighted::new(force, w).unwrap()));
            }
        }
        forces
    };
    let worst = |a: &[Vector], b: &[Vector]| -> f32 {
        let scale = b.iter().map(|v| v.length()).fold(1e-6f32, f32::max);
        a.iter()
            .zip(b)
            .map(|(x, y)| (*x - *y).length())
            .fold(0.0f32, f32::max)
            / scale
    };
    let at_one_zero = read(mixed(1.0, 0.0));
    let at_zero_one = read(mixed(0.0, 1.0));
    let at_half = read(mixed(0.5, 0.5));
    let halfway: Vec<Vector> = springs
        .iter()
        .zip(&charge)
        .map(|(s, c)| (*s + *c) * 0.5)
        .collect();
    let (e10, e01, e55) = (
        worst(&at_one_zero, &springs),
        worst(&at_zero_one, &charge),
        worst(&at_half, &halfway),
    );
    let apart = worst(&at_half, &springs).min(worst(&at_half, &charge));
    println!(
        "mix against the pure laws, largest difference over the largest push: 1/0 {e10:.2e}, \
         0/1 {e01:.2e}, 0.5/0.5 against the mean {e55:.2e}; 0.5/0.5 against either law {apart:.3}"
    );
    assert!(e10 <= 1e-6 && e01 <= 1e-6 && e55 <= 1e-5);
    // Springs and Charge share their springs and centring, so they differ by
    // the repulsion alone: far above the tolerance, not far apart.
    assert!(apart > 1e-3, "the half mix is neither law");
}

/// The weighted sum over time, through the canvas: from one seeded
/// arrangement, 360 frames of a mix at 1/0 land where Springs alone does,
/// and at 0/1 where Charge does, within a stated tolerance; the half mix
/// lands apart from both (the control).
#[test]
fn a_weighted_mix_runs_as_each_pure_law_at_one_and_zero() {
    let run = |slot: Option<PhysicsComposition>, law: PhysicsLaw| {
        let (keys, edges, _) = ring();
        let mut graph = Graph::new();
        let nodes: Vec<NodeKey> = (0..keys.len())
            .map(|i| {
                graph.add_node(
                    format!("https://s{}.example/{i}", i % 4),
                    PortablePoint::new(0.0, 0.0),
                )
            })
            .collect();
        for (a, b) in edges {
            graph.assert_relation(nodes[a.index()], nodes[b.index()], hyperlink());
        }
        let mut canvas = Canvas::with_graph(graph);
        canvas.resize(1400, 900);
        canvas.set_physics_paused(true);
        canvas.set_layout_strategy(Some("test.scatter".to_string()));
        let seed: Vec<_> = nodes
            .iter()
            .enumerate()
            .map(|(i, &k)| {
                let a = i as f32 * 2.399_963;
                let r = 30.0 * (i as f32 + 1.0).sqrt();
                (k, PortablePoint::new(r * a.cos(), r * a.sin()))
            })
            .collect();
        canvas.apply_strategy_positions(&seed);
        canvas.set_physics_law(law).unwrap();
        canvas.set_physics_composition(slot).unwrap();
        canvas.set_physics_paused(false);
        for _ in 0..360 {
            canvas.step_layout();
        }
        nodes
            .iter()
            .map(|k| canvas.view.position_of(*k).unwrap())
            .collect::<Vec<_>>()
    };
    let apart = |a: &[PortablePoint], b: &[PortablePoint]| {
        a.iter()
            .zip(b)
            .map(|(p, q)| (p.x - q.x).hypot(p.y - q.y))
            .fold(0.0f32, f32::max)
    };
    let mix = |w1, w2| {
        Some(PhysicsComposition::Mix(vec![
            (PhysicsLaw::Springs, w1),
            (PhysicsLaw::Charge, w2),
        ]))
    };
    let springs = run(None, PhysicsLaw::Springs);
    let charge = run(None, PhysicsLaw::Charge);
    let one_zero = run(mix(1.0, 0.0), PhysicsLaw::Springs);
    let zero_one = run(mix(0.0, 1.0), PhysicsLaw::Springs);
    let half = run(mix(0.5, 0.5), PhysicsLaw::Springs);
    let (d10, d01) = (apart(&one_zero, &springs), apart(&zero_one, &charge));
    let (h_s, h_c) = (apart(&half, &springs), apart(&half, &charge));
    println!(
        "after 360 frames, largest distance from the pure law: 1/0 {d10:.4}, 0/1 {d01:.4}; \
         the half mix from Springs {h_s:.1} and from Charge {h_c:.1}"
    );
    assert!(
        d10 <= 0.5 && d01 <= 0.5,
        "within half a unit of the pure law"
    );
    assert!(h_s > 5.0 && h_c > 5.0, "the half mix is neither law");
}
