// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The dynamics grammar's G1 receipts over the physics catalog: every law,
//! overlay and slot declares its terms, and seiche's instruments agree with
//! each declared class on the P2 fixture and on a generated graph. Plan:
//! `design_docs/mere_docs/implementation_strategy/2026-10-02_dynamics_grammar_plan.md`.

use std::rc::Rc;

use seiche::instruments::{self, Candidate, Probe, Reading};
use seiche::{
    AffinitySpring, AnchorSpring, Class, CouplingForce, Currency, Declared, Force, Layout, Metric,
    SceneField, State, Term,
};

use super::*;
use crate::canvas::physics_catalog::{
    CHARGE_STRENGTH, LAW_SEED, LawInputs, LawSources, PhysicsDepthSource, PhysicsKindSource,
    PhysicsLaw, PhysicsMassSource, PhysicsOverlay,
};

/// Charge's row on its exact rung, beside the catalog's Barnes–Hut one.
const EXACT_CHARGE: &str = "charge.barnes-hut[0]@exact";
/// The catalog's own Charge row: a Barnes–Hut rung at θ 0.5.
const APPROXIMATE_CHARGE: &str = "charge.barnes-hut[0]/charge";

/// The P2 fixture: Graphshell's reference host graph
/// (`ports/graphshell/src/mere_host_fixture.rs`), the eleven nodes in
/// creation order and its ten relations, the graph every P2 and P4 receipt
/// ran on.
const P2_URLS: [&str; 11] = [
    "https://example.test/i2p-port",
    "i2p://reference/service",
    "file:///Graphshell/reference-notes.md",
    "mere://scene/reference-host",
    "graphshell://projection/loopback-g1",
    "personae://persona/alice",
    "personae://device/laptop",
    "personae://device/phone",
    "personae://key/ssh-ed25519/test",
    "personae://grant/open-addresses",
    "personae://receipt/signing/test",
];
const P2_EDGES: [(usize, usize); 10] = [
    (0, 1),
    (0, 2),
    (3, 0),
    (3, 2),
    (4, 3),
    (5, 6),
    (5, 7),
    (6, 8),
    (9, 5),
    (10, 9),
];

/// The catalog's default sources, the web host's choice.
const SOURCES: LawSources = LawSources {
    kind: PhysicsKindSource::Site,
    groups: PhysicsKindSource::Site,
    mass: PhysicsMassSource::Degree,
    depth: PhysicsDepthSource::Roots,
    focus: None,
};

struct Fixture {
    name: &'static str,
    nodes: Vec<NodeKey>,
    edges: Vec<(NodeKey, NodeKey)>,
    sites: HashMap<NodeKey, String>,
    radius: f32,
}

impl Fixture {
    fn from_urls(
        name: &'static str,
        urls: &[String],
        edges: &[(usize, usize)],
        radius: f32,
    ) -> Self {
        let nodes: Vec<NodeKey> = (0..urls.len()).map(NodeKey::new).collect();
        Self {
            name,
            edges: edges.iter().map(|&(a, b)| (nodes[a], nodes[b])).collect(),
            sites: nodes
                .iter()
                .zip(urls)
                .map(|(&k, url)| (k, Graph::url_grouping_key(url).to_string()))
                .collect(),
            nodes,
            radius,
        }
    }

    fn p2() -> Self {
        let urls: Vec<String> = P2_URLS.iter().map(|u| u.to_string()).collect();
        Self::from_urls("p2", &urls, &P2_EDGES, 150.0)
    }

    /// Forty nodes over five sites: a seeded random tree plus twenty chords.
    fn generated() -> Self {
        let mut state = 7u64;
        let mut next = move || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 33) as usize
        };
        let n = 40;
        let urls: Vec<String> = (0..n)
            .map(|i| format!("https://s{}.test/{i}", i % 5))
            .collect();
        let mut edges: Vec<(usize, usize)> = (1..n).map(|i| (i, next() % i)).collect();
        for _ in 0..n / 2 {
            let (a, b) = (next() % n, next() % n);
            if a != b {
                edges.push((a, b));
            }
        }
        Self::from_urls("generated", &urls, &edges, 300.0)
    }

    fn inputs(&self) -> Rc<LawInputs<'static>> {
        Rc::new(LawInputs::from_parts(
            self.nodes.clone(),
            self.edges.clone(),
            self.sites.clone(),
        ))
    }

    fn probe(&self) -> Probe {
        Probe::new(&self.nodes, &self.edges)
    }

    fn starts(&self, probe: &Probe) -> Vec<Vec<seiche::instruments::Vector>> {
        (1..=3)
            .map(|seed| probe.scatter(seed, self.radius))
            .collect()
    }
}

type Make = Rc<dyn Fn() -> Box<dyn Force>>;

/// One row of the brief's tables (§5.1–5.3) and the force that realizes it.
struct Row {
    name: String,
    make: Make,
}

/// The numen fields the coupling rows sample: a paraboloid for the scalar
/// responses, a uniform flow for the vector ones.
fn paraboloid() -> numen::FieldDefinition {
    use numen::ScalarField as S;
    numen::FieldDefinition::Scalar(S::Scale(
        Box::new(S::Add(
            Box::new(S::Mul(Box::new(S::CoordX), Box::new(S::CoordX))),
            Box::new(S::Mul(Box::new(S::CoordY), Box::new(S::CoordY))),
        )),
        0.5,
    ))
}

fn flow() -> numen::FieldDefinition {
    numen::FieldDefinition::Vector(numen::VectorField::ConstVec { x: 1.0, y: 0.0 })
}

/// Every force the catalog builds on `fixture`, by row: each law's forces,
/// each overlay (the hub overlays under both mass sources), and the slots.
fn rows(fixture: &Fixture) -> Vec<Row> {
    let inputs = fixture.inputs();
    let mut rows = Vec::new();
    for law in PhysicsLaw::ALL {
        let count = inputs.law_forces(law, SOURCES).len();
        for k in 0..count {
            let inputs = inputs.clone();
            rows.push(Row {
                name: format!("{}[{k}]", law.id()),
                make: Rc::new(move || inputs.law_forces(law, SOURCES).remove(k)),
            });
        }
    }
    // Charge's term on its exact rung: the quadtree at θ 0 opens every cell.
    // The catalog's θ 0.5 is a Barnes–Hut rung of this term, read beside it.
    rows.push(Row {
        name: EXACT_CHARGE.into(),
        make: Rc::new(|| {
            Box::new(seiche::BarnesHutRepulsion {
                strength: CHARGE_STRENGTH,
                config: seiche::BarnesHutConfig {
                    theta: 0.0,
                    ..seiche::BarnesHutConfig::default()
                },
                ..seiche::BarnesHutRepulsion::default()
            })
        }),
    });
    for overlay in PhysicsOverlay::ALL {
        let masses: &[PhysicsMassSource] = if overlay.weighted() {
            &PhysicsMassSource::ALL
        } else {
            &[PhysicsMassSource::Degree]
        };
        for &mass in masses {
            let inputs = inputs.clone();
            let sources = LawSources { mass, ..SOURCES };
            rows.push(Row {
                name: format!("{}/{}", overlay.id(), mass.id()),
                make: Rc::new(move || inputs.overlay_force(overlay, sources)),
            });
        }
    }
    let nodes = fixture.nodes.clone();
    let slots: Vec<(NodeKey, (f32, f32))> = nodes
        .iter()
        .enumerate()
        .map(|(i, &k)| {
            (
                k,
                ((i % 6) as f32 * 90.0 - 225.0, (i / 6) as f32 * 90.0 - 150.0),
            )
        })
        .collect();
    rows.push(Row {
        name: "slot/anchor".into(),
        make: Rc::new(move || Box::new(AnchorSpring::new(slots.clone()))),
    });
    let pairs: Vec<(NodeKey, NodeKey, f32)> = nodes.windows(3).map(|w| (w[0], w[2], 0.8)).collect();
    rows.push(Row {
        name: "slot/affinity".into(),
        make: Rc::new(move || Box::new(AffinitySpring::new(pairs.clone()))),
    });
    use numen::CouplingResponse as R;
    for (label, response, field) in [
        ("attract", R::AttractToMin, paraboloid()),
        ("repel", R::RepelFromMax, paraboloid()),
        ("wall", R::ContainmentWall, paraboloid()),
        ("dampen", R::DampenInside { factor: 0.5 }, paraboloid()),
        ("align", R::AlignVelocity, flow()),
        ("advect", R::FlowAdvect, flow()),
    ] {
        let targets = nodes.clone();
        rows.push(Row {
            name: format!("slot/coupling-{label}"),
            make: Rc::new(move || {
                Box::new(CouplingForce::new(
                    response.clone(),
                    0.002,
                    targets.clone(),
                    field.clone(),
                ))
            }),
        });
    }
    rows
}

/// What the brief's tables declare, row by row (§5.1–5.3), with the plan's
/// F8: Hub room and Hub pull Em, Flow's needle N.
fn expected_classes(row: &str) -> Option<&'static [Class]> {
    use Class::*;
    Some(match row {
        "spring.rapier[0]" | "spring.rapier[1]" | "spring.rapier[2]" => &[E],
        "charge.barnes-hut[0]" | "charge.barnes-hut[1]" | "charge.barnes-hut[2]" => &[E],
        EXACT_CHARGE => &[E],
        "stress.kamada-kawai[0]" | "stress.kamada-kawai[1]" | "stress.kamada-kawai[2]" => &[E],
        "energy.linlog[0]" => &[E],
        "energy.linlog[1]" => &[E, E, E],
        "orbit.gravity[1]" => &[H, N, K],
        "kinds.particle-life[1]" => &[N, E],
        "flock.boids[1]" => &[E, N, Em, N, E],
        "sync.kuramoto[1]" => &[H, H],
        "flow.magnetic[1]" => &[E, E, N],
        "flow.magnetic[2]" => &[E],
        "anneal.davidson-harel[0]" => &[K],
        "still.default[0]" => &[K],
        // Density writes positions from its medium (the plan's sketch).
        "density.gastner-newman[0]" => &[K],
        "degree-repulsion/degree" | "degree-repulsion/pagerank" => &[Em],
        "hub-gravity/degree" | "hub-gravity/pagerank" => &[Em],
        "domain-cluster/degree"
        | "depth-gravity/degree"
        | "grid-snap/degree"
        | "gravity-locus/degree"
        | "skeleton/degree" => &[E],
        "tide/degree" => &[H],
        "slot/anchor" | "slot/affinity" => &[E],
        "slot/coupling-attract" | "slot/coupling-repel" | "slot/coupling-wall" => &[E],
        "slot/coupling-dampen" | "slot/coupling-align" | "slot/coupling-advect" => &[K],
        // Every law's leading `NodeExclusion`.
        name if name.ends_with("[0]") => &[E],
        _ => return None,
    })
}

/// Every row of the brief's §5.1–5.3 declares its terms: the twelve laws
/// (Density the twelfth, declared as the plan's Findings sketched it), the
/// eight overlays, the slots, and the always-on
/// terms rapier realizes. The repulsion solver seam is a rung of
/// `NodeExclusion`, not a term, so it has no row.
#[test]
fn every_catalog_term_declares_itself() {
    assert_eq!(
        PhysicsLaw::ALL.len(),
        12,
        "every law declares its terms"
    );
    let fixture = Fixture::p2();
    let probe = fixture.probe();
    let positions = probe.scatter(1, fixture.radius);
    let nodes: Vec<(NodeKey, seiche::instruments::Vector)> =
        fixture.nodes.iter().copied().zip(positions).collect();
    let layout = Layout {
        nodes: &nodes,
        edges: &fixture.edges,
    };
    for row in rows(&fixture) {
        let force = (row.make)();
        let terms = force.terms();
        assert!(!terms.is_empty(), "{} declares no term", row.name);
        let classes: Vec<Class> = terms.iter().map(|t| t.class).collect();
        let expected = expected_classes(&row.name)
            .unwrap_or_else(|| panic!("{} has no row in the brief's tables", row.name));
        assert_eq!(classes, expected, "{} declares other classes", row.name);
        for (t, term) in terms.iter().enumerate() {
            if terms.len() > 1 {
                assert!(
                    force.isolate(t).is_some(),
                    "{} cannot isolate {}",
                    row.name,
                    term.name
                );
            }
            // E and Em expose their energy; N has none; a K realization may
            // expose the energy it realizes (Anneal).
            let energy = force.energy(t, &layout);
            match term.class {
                Class::E | Class::Em => {
                    assert!(
                        energy.is_some(),
                        "{}: {} has no energy",
                        row.name,
                        term.name
                    )
                },
                Class::N => assert!(
                    energy.is_none(),
                    "{}: {} has an energy",
                    row.name,
                    term.name
                ),
                Class::H | Class::K => {},
            }
            if let Some(weights) = force.metric(t, &layout) {
                assert_eq!(weights.len(), nodes.len(), "{}: {}", row.name, term.name);
                assert!(
                    term.metric.is_some(),
                    "{}: {} has weights, no metric",
                    row.name,
                    term.name
                );
            } else {
                assert!(
                    term.metric.is_none(),
                    "{}: {} has a metric, no weights",
                    row.name,
                    term.name
                );
            }
            assert_eq!(
                term.currency == Currency::Kinematic,
                term.class == Class::K,
                "{}: {} writes state exactly when it is K",
                row.name,
                term.name
            );
        }
        assert!(force.isolate(terms.len()).is_none());
    }
    // The catalog's own statement of which overlays read the mass source is
    // the declarations': exactly those with the mass metric.
    let inputs = fixture.inputs();
    for overlay in PhysicsOverlay::ALL {
        let terms = inputs.overlay_force(overlay, SOURCES).terms();
        assert_eq!(
            overlay.weighted(),
            terms.iter().any(|t| t.metric == Some(Metric::Mass)),
            "{}",
            overlay.id()
        );
    }
    // The open coupling tail applies nothing and declares nothing.
    let open = CouplingForce::new(
        numen::CouplingResponse::Open {
            predicate: "https://example.test/open".into(),
        },
        1.0,
        fixture.nodes.clone(),
        paraboloid(),
    );
    assert!(open.terms().is_empty());
    // What rapier realizes outside the force list, and the scene's field.
    for term in [seiche::terms::CONTACTS, seiche::terms::DAMPING] {
        assert_eq!(
            (term.class, term.currency),
            (Class::K, Currency::Integrator)
        );
    }
    assert_eq!(
        (seiche::terms::PIN.class, seiche::terms::PIN.state),
        (Class::K, State::Position)
    );
    let vortex = SceneField::Vortex {
        center: (0.0, 0.0),
        strength: 1.0,
        inward: 1.0,
    };
    let classes: Vec<Class> = vortex.terms().iter().map(|t| t.class).collect();
    assert_eq!(classes, [Class::N, Class::E]);
}

/// Every term the force rows declare, read on one fixture. K terms write
/// state and phase terms move no body, so neither is read by force here
/// (Anneal's energy is checked below, Sync's phase in seiche).
fn readings(fixture: &Fixture) -> Vec<(String, Reading)> {
    let mut probe = fixture.probe();
    let starts = fixture.starts(&probe);
    let mut out = Vec::new();
    for row in rows(fixture) {
        let terms: Vec<Term> = (row.make)().terms();
        for (t, term) in terms.iter().enumerate() {
            if term.class == Class::K || term.state == State::Phase {
                continue;
            }
            let make = row.make.clone();
            let reading = instruments::read(&mut probe, &move || make(), t, &starts);
            out.push((format!("{}/{}", row.name, term.name), reading));
        }
    }
    out
}

fn print(fixture: &str, name: &str, r: &Reading) {
    let opt = |v: Option<f64>| v.map_or("-".to_string(), |v| format!("{v:.2e}"));
    let each = |v: &[f64]| {
        v.iter()
            .map(|x| format!("{x:.1e}"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    println!(
        "{fixture:9} {name:44} {:2?} rise {:>8} grad {:>8} bal {:>8} id-bal {:>8} \
         loop {:.2e} [{}] jac {:.2e} [{}] motion {:.2e} vel {:.2e}",
        r.term.class,
        opt(r.rise),
        opt(r.gradient),
        opt(r.balance),
        opt(r.identity_balance),
        instruments::median(&r.loop_work),
        each(&r.loop_work),
        instruments::median(&r.jacobian),
        each(&r.jacobian),
        r.motion,
        r.velocity,
    );
}

/// The instruments agree with every declared class on both fixtures:
/// energy, gradient, balance, Jacobian symmetry as the descent test for a
/// term with no energy (F14) and the velocity check beside it (F15). Loop
/// work's and persistent motion's verdicts are printed as diagnostics, and so
/// is Charge's Barnes–Hut θ 0.5 beside its law at θ 0 (F16). The full table
/// goes to the test's output.
#[test]
fn the_instruments_agree_with_every_declared_class() {
    let mut disagreements: HashMap<Candidate, Vec<String>> = HashMap::new();
    for fixture in [Fixture::p2(), Fixture::generated()] {
        for (name, reading) in readings(&fixture) {
            print(fixture.name, &name, &reading);
            for candidate in Candidate::ALL {
                if let Err(why) = reading.agrees_by(candidate) {
                    disagreements
                        .entry(candidate)
                        .or_default()
                        .push(format!("{} {name}: {why}", fixture.name));
                }
            }
            // F8: the hub overlays balance only in their own metric.
            if reading.term.class == Class::Em && reading.term.topology.is_internal() {
                assert!(
                    reading.identity_balance.unwrap() > instruments::tolerance::BALANCE,
                    "{name} balances at unit weight too: {reading:?}"
                );
            }
        }
    }
    for candidate in Candidate::ALL {
        let list = disagreements.get(&candidate).cloned().unwrap_or_default();
        println!("{candidate:?}: {} disagreement(s)", list.len());
        for line in &list {
            println!("  {}", line.split(": Reading").next().unwrap_or(line));
        }
    }
    let wrong: Vec<&String> = disagreements
        .get(&Candidate::Jacobian)
        .into_iter()
        .flatten()
        .filter(|line| !line.contains(APPROXIMATE_CHARGE))
        .collect();
    assert!(wrong.is_empty(), "the instruments disagree: {wrong:#?}");
}

/// The positive control, in one run on both fixtures: Kinds with the
/// catalog's seeded matrix fails descent (it has no energy, and Jacobian
/// symmetry calls it non-conservative) and reciprocity; the same law with its
/// matrix symmetrized passes both. Both readings, diagnostics included, are
/// printed.
#[test]
fn kinds_seeded_fails_both_instruments_and_symmetrized_passes_both() {
    for fixture in [Fixture::p2(), Fixture::generated()] {
        let inputs = fixture.inputs();
        let (kinds, count) = inputs.kinds(SOURCES.kind);
        let seeded = seiche::ParticleLife::seeded(kinds, count, LAW_SEED);
        let symmetrized = seeded.symmetrized();
        let mut probe = fixture.probe();
        let starts = fixture.starts(&probe);
        let read = |probe: &mut Probe, law: &seiche::ParticleLife| {
            let law = law.clone();
            instruments::read(probe, &move || Box::new(law.clone()), 0, &starts)
        };
        let fails = read(&mut probe, &seeded);
        let passes = read(&mut probe, &symmetrized);
        print(fixture.name, "kinds seeded", &fails);
        print(fixture.name, "kinds symmetrized", &passes);
        assert_eq!((fails.term.class, passes.term.class), (Class::N, Class::E));
        // Descent: no energy to descend, and not symmetric.
        assert!(fails.rise.is_none() && fails.gradient.is_none());
        assert!(!fails.conservative(Candidate::Jacobian), "{fails:?}");
        assert!(passes.conservative(Candidate::Jacobian), "{passes:?}");
        // Reciprocity.
        assert!(
            fails.balance.unwrap() > instruments::tolerance::BALANCE,
            "{fails:?}"
        );
        // Both agree with their declared classes.
        fails.agrees().unwrap();
        passes.agrees().unwrap();
    }
}

/// Anneal is K: it writes positions, so no force instrument reads it, but the
/// energy it realizes must end lower than it starts over its schedule.
#[test]
fn anneal_lowers_the_energy_it_realizes() {
    for fixture in [Fixture::p2(), Fixture::generated()] {
        let inputs = fixture.inputs();
        let measure = inputs.law_forces(PhysicsLaw::Anneal, SOURCES).remove(0);
        let probe = fixture.probe();
        let start = probe.scatter(1, fixture.radius);
        let mut sim = seiche::Simulation::new();
        sim.sync_nodes(
            fixture
                .nodes
                .iter()
                .zip(&start)
                .map(|(&k, p)| (k, euclid::default::Point2D::new(p.x, p.y))),
        );
        sim.sync_edges(fixture.edges.clone());
        let energy = |sim: &seiche::Simulation| {
            let mut nodes: Vec<(NodeKey, seiche::instruments::Vector)> = sim
                .positions()
                .map(|(k, p)| (k, seiche::instruments::Vector::new(p.x, p.y)))
                .collect();
            nodes.sort_by_key(|(k, _)| k.index());
            measure
                .energy(
                    0,
                    &Layout {
                        nodes: &nodes,
                        edges: &fixture.edges,
                    },
                )
                .unwrap()
        };
        let before = energy(&sim);
        sim.set_forces(inputs.law_forces(PhysicsLaw::Anneal, SOURCES));
        for _ in 0..1800 {
            sim.tick(1.0 / 60.0);
        }
        let after = energy(&sim);
        println!("{} anneal energy {before:.4e} -> {after:.4e}", fixture.name);
        assert!(after < before, "{}: {before} -> {after}", fixture.name);
    }
}
