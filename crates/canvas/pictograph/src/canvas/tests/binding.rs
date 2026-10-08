// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The binding (dynamics grammar plan, G4b1): every admitted choice to a
//! spec and back (F148), profiles naming themselves after a reopen, the
//! record keeping what it carries (F146), compositions running from a spec
//! as from the API, the seed reaching Kinds (F156), the roles through the
//! target, and what a spec cannot hold refused (F157).

use crate::canvas::tests::ThroughView;
use std::collections::BTreeMap;

use seiche::Role;

use super::*;
use crate::canvas::composition::{GroupSource, PhysicsComposition, PhysicsGrouping};
use crate::canvas::dynamics_spec::{
    self as spec, Bar, BindError, BoundSources, DynamicsSpec, GroupRoles, Node, Observable,
    Realization, Stage, Stop, Target, bind, bind_shape, resolve_channels,
};
use crate::canvas::physics_catalog::{
    CANVAS_PHYSICS_PROFILES, PhysicsDepthSource, PhysicsKindSource, PhysicsLaw, PhysicsMassSource,
    PhysicsOverlay,
};
use crate::canvas::schedule::{PhysicsStage, StageStop};

fn graph() -> (Graph, Vec<NodeKey>) {
    let mut graph = Graph::new();
    let keys: Vec<NodeKey> = (0..24)
        .map(|i| {
            graph.add_node(
                format!("https://s{}.example/{i}", i % 4),
                PortablePoint::new(0.0, 0.0),
            )
        })
        .collect();
    for i in 0..24 {
        graph.assert_relation(keys[i], keys[(i + 1) % 24], hyperlink());
    }
    for i in (0..24).step_by(4) {
        graph.assert_relation(keys[i], keys[(i + 9) % 24], hyperlink());
    }
    (graph, keys)
}

thread_local! {
    /// One graph per test thread, so every canvas a test makes holds the
    /// same node ids and a target's items resolve on each.
    static GRAPH: (Graph, Vec<NodeKey>) = graph();
}

/// Twenty-four nodes on four sites, a ring with chords, seeded on a spiral.
fn canvas() -> (Canvas, Vec<NodeKey>) {
    let (graph, keys) = GRAPH.with(|(graph, keys)| (graph.clone(), keys.clone()));
    let mut canvas = Canvas::with_graph(graph);
    canvas.resize(1400, 900);
    canvas.set_physics_paused(true);
    canvas.set_layout_strategy(Some("test.spiral".to_string()));
    let seed: Vec<_> = keys
        .iter()
        .enumerate()
        .map(|(i, &k)| {
            let (r, a) = (40.0 * (i as f32).sqrt(), i as f32 * 2.399_963);
            (k, PortablePoint::new(r * a.cos(), r * a.sin()))
        })
        .collect();
    canvas.apply_strategy_positions(&seed);
    (canvas, keys)
}

fn all_sources() -> Vec<BoundSources> {
    let mut out = Vec::new();
    for kind in PhysicsKindSource::ALL {
        for groups in PhysicsKindSource::ALL {
            for mass in PhysicsMassSource::ALL {
                for depth in PhysicsDepthSource::ALL {
                    out.push(BoundSources {
                        kind,
                        groups,
                        mass,
                        depth,
                    });
                }
            }
        }
    }
    out
}

/// Every overlay subset in catalog order and reversed, those the law takes.
fn admitted_pairs() -> Vec<(PhysicsLaw, Vec<PhysicsOverlay>)> {
    let mut out = Vec::new();
    for law in PhysicsLaw::ALL {
        for mask in 0u32..256 {
            let subset: Vec<PhysicsOverlay> = PhysicsOverlay::ALL
                .into_iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, o)| o)
                .collect();
            if subset.iter().any(|o| law.refuses(*o).is_some()) {
                continue;
            }
            let reversed: Vec<_> = subset.iter().rev().copied().collect();
            if reversed != subset {
                out.push((law, reversed));
            }
            out.push((law, subset));
        }
    }
    out
}

fn choice(law: PhysicsLaw, overlays: Vec<PhysicsOverlay>, s: BoundSources) -> PhysicsChoice {
    PhysicsChoice {
        law,
        overlays,
        kind: s.kind,
        groups: s.groups,
        mass: s.mass,
        depth: s.depth,
    }
}

/// F148: every admitted (law, overlay subset) pair, in catalog and reversed
/// order, derives and comes back from its spec; across all 216 source
/// combinations each pair's spec comes back to the same choice.
#[test]
fn every_admitted_choice_round_trips_through_a_spec() {
    let pairs = admitted_pairs();
    let sources = all_sources();
    assert_eq!(sources.len(), 216);
    let density = pairs
        .iter()
        .filter(|(law, _)| *law == PhysicsLaw::Density)
        .count();
    println!(
        "{} admitted pairs with order ({density} on Density), {} choices",
        pairs.len(),
        pairs.len() * sources.len()
    );
    for (law, overlays) in &pairs {
        let c = choice(*law, overlays.clone(), BoundSources::default());
        assert_eq!(
            PhysicsChoice::from_spec(&c.into_spec()),
            Ok(c.clone()),
            "{c:?}"
        );
        for s in &sources {
            let c = choice(*law, overlays.clone(), *s);
            let back = PhysicsChoice::flat(bind_shape(&c.into_spec()).unwrap());
            assert_eq!(back, c);
        }
    }
    // A pair the law refuses is not a choice: derive refuses it (F73).
    let refused = choice(
        PhysicsLaw::Density,
        vec![PhysicsOverlay::GridSnap],
        BoundSources::default(),
    );
    assert!(matches!(
        PhysicsChoice::from_spec(&refused.into_spec()),
        Err(BindError::Refused(_))
    ));
}

/// F148: each of the twenty profiles names itself after its spec reopens on
/// a fresh canvas.
#[test]
fn every_profile_names_itself_after_a_reopen() {
    assert_eq!(CANVAS_PHYSICS_PROFILES.len(), 20);
    for profile in CANVAS_PHYSICS_PROFILES {
        let (mut from, _) = canvas();
        assert!(from.pick_profile(profile.id));
        let saved = from.dynamics_spec().unwrap();
        let (mut to, _) = canvas();
        to.set_dynamics_spec(&saved).unwrap();
        assert_eq!(to.view().profile_id(), Some(profile.id));
        assert_eq!(to.dynamics_spec().unwrap(), saved);
    }
}

/// F149: a channel that does not parse, or that its slot cannot read, is
/// refused by slot and id; every channel a slot reads is taken.
#[test]
fn a_slot_takes_only_its_own_family() {
    for s in all_sources() {
        assert_eq!(resolve_channels(&s.channels()), Ok(s));
    }
    let refuse = |slot: &str, id: &str| {
        resolve_channels(&BTreeMap::from([(slot.to_string(), id.to_string())]))
            .unwrap_err()
            .to_string()
    };
    assert_eq!(
        refuse("kind", "kind.colour"),
        "at channels.kind: unknown channel kind.colour"
    );
    assert_eq!(
        refuse("mass", "weight.degree"),
        "at channels.mass: the mass slot cannot read weight.degree"
    );
    assert_eq!(
        refuse("groups", "groups.bridges"),
        "at channels.groups: the groups slot cannot read groups.bridges"
    );
    assert_eq!(
        refuse("depth", "distances.hops"),
        "at channels.depth: the depth slot cannot read distances.hops"
    );
}

/// The spec every root field of which the canvas runs: a schedule of a law,
/// a mix and a grouping, weights off the f32 grid, a seed, bars, a target.
fn recipe(member: uuid::Uuid) -> DynamicsSpec {
    let at = |law: PhysicsLaw, weight: f64| {
        let mut node = Node::preset(law.id());
        if let Node::Preset { weight: w, .. } = &mut node {
            *w = weight;
        }
        node
    };
    let mut first = Node::preset(PhysicsLaw::Springs.id());
    first
        .overlays_mut()
        .push(Node::preset(PhysicsOverlay::GravityLocus.id()));
    let stage = |node, stop, capture| Stage {
        node,
        stop,
        capture,
    };
    let mut spec = DynamicsSpec::new(Node::Schedule {
        stages: vec![
            stage(first, Stop::Frames(30), Some(Role::Anchored)),
            stage(
                Node::Mix {
                    parts: vec![at(PhysicsLaw::Springs, 0.1), at(PhysicsLaw::Energy, 2.0)],
                    weight: 1.0,
                    overlays: Vec::new(),
                },
                Stop::Rest,
                None,
            ),
            stage(
                Node::Grouped {
                    partition: "groups.cluster".into(),
                    outer: Box::new(at(PhysicsLaw::Charge, 16.0)),
                    inner: Box::new(Node::preset(PhysicsLaw::Springs.id())),
                    weight: 1.0,
                    overlays: Vec::new(),
                },
                Stop::LawDone,
                None,
            ),
        ],
        weight: 1.0,
        overlays: Vec::new(),
    });
    spec.seed = 7;
    spec.channels.insert("mass".into(), "mass.pagerank".into());
    spec.realization = Realization::Integrate {
        damping: Some(0.35),
    };
    spec.target = Some(Target {
        arrangement: "grid.default".into(),
        anchored_pull: 0.3,
        default_role: Role::Anchored,
        groups: Some(GroupRoles {
            channel: "groups.site".into(),
            roles: BTreeMap::from([("s1.example".to_string(), Role::Pinned)]),
        }),
        items: BTreeMap::from([
            (member.to_string(), Role::Seeded),
            (uuid::Uuid::from_u128(404).to_string(), Role::Pinned),
        ]),
    });
    spec.bars = vec![Bar {
        observable: Observable::Overlaps,
        at_least: None,
        at_most: Some(4.0),
    }];
    spec
}

/// F146: a spec opened and not edited reads back byte for byte; a picker
/// edit rewrites the root, keeping the seed, bars and target, and its
/// weights come back on the f32 grid the canvas runs.
#[test]
fn the_record_keeps_an_opened_spec_and_what_an_edit_does_not_touch() {
    let (mut canvas, keys) = canvas();
    let member = canvas.graph.get_node(keys[3]).unwrap().id;
    let given = recipe(member);
    let report = canvas.set_dynamics_spec(&given).unwrap();
    assert_eq!(report.absent_items, 1, "an item the graph lacks is counted");
    let read = canvas.dynamics_spec().unwrap();
    assert_eq!(
        serde_json::to_string(&read).unwrap(),
        serde_json::to_string(&given).unwrap(),
        "byte for byte"
    );
    assert_eq!(canvas.physics_schedule_stage(), Some(0), "the recipe runs");
    assert_eq!(canvas.arrangement_roles().default, Role::Anchored);
    assert_eq!(
        canvas.arrangement_roles().items.get(&keys[3]),
        Some(&Role::Seeded)
    );
    assert_eq!(canvas.anchor_stiffness(), 0.3);
    assert_eq!(canvas.view().mass, PhysicsMassSource::PageRank);

    // A source edit through the view: the recipe, its authored weights and
    // the carried parts stay.
    canvas.pick_kind(PhysicsKindSource::Degree);
    let edited = canvas.dynamics_spec().unwrap();
    assert_eq!(edited.channels["kind"], "kind.degree");
    assert_eq!((edited.seed, &edited.bars), (7, &given.bars));
    let Node::Schedule { stages, .. } = &edited.root else {
        panic!("still the recipe")
    };
    let Node::Mix { parts, .. } = &stages[1].node else {
        panic!("the mix stage")
    };
    assert_eq!(parts[0].weight(), 0.1, "the authored weight");
    assert_eq!(edited.root, given.root);
    assert_eq!(
        edited.target.as_ref().unwrap().items.len(),
        2,
        "an item the graph lacks rides on with the record (F105)"
    );

    // A law pick takes over from the recipe, keeping seed, bars, damping.
    canvas.pick_law(PhysicsLaw::Charge).unwrap();
    let picked = canvas.dynamics_spec().unwrap();
    assert!(matches!(&picked.root, Node::Preset { id, .. } if id == PhysicsLaw::Charge.id()));
    assert_eq!((picked.seed, &picked.bars), (7, &given.bars));
    assert_eq!(picked.realization, given.realization);
    // Back from the picked spec, on a fresh canvas: the same record.
    let (mut again, _) = super::binding::canvas();
    again.set_dynamics_spec(&picked).unwrap();
    assert_eq!(again.dynamics_spec().unwrap(), picked);
}

fn run(canvas: &mut Canvas, frames: usize) -> Vec<(NodeKey, PortablePoint)> {
    canvas.set_physics_paused(false);
    for _ in 0..frames {
        canvas.step_layout();
    }
    let mut out: Vec<_> = canvas.view.positions().collect();
    out.sort_by_key(|(k, _)| k.index());
    out
}

fn largest_gap(a: &[(NodeKey, PortablePoint)], b: &[(NodeKey, PortablePoint)]) -> f32 {
    a.iter()
        .zip(b)
        .map(|((_, p), (_, q))| (p.x - q.x).hypot(p.y - q.y))
        .fold(0.0, f32::max)
}

/// Compositions run on reopen: a mix, a grouping on `groups.cluster` and a
/// schedule, each opened from the spec its API-set canvas reads back, move
/// the bodies as the API-set canvas does; a different weight is the
/// control. (Hash order in the exclusion sum moves runs by about 1e-3.)
#[test]
fn a_composition_opened_from_its_spec_runs_as_the_api_set_one() {
    let mix =
        |w: f32| PhysicsComposition::Mix(vec![(PhysicsLaw::Springs, w), (PhysicsLaw::Energy, 1.0)]);
    let grouped = PhysicsComposition::Grouped(PhysicsGrouping::charge_between(
        GroupSource::Channel(PhysicsKindSource::Cluster),
    ));
    let stages = vec![
        PhysicsStage::law(PhysicsLaw::Stress, StageStop::Frames(40)).capturing(Role::Anchored),
        PhysicsStage::law(PhysicsLaw::Springs, StageStop::Rest),
    ];
    type Set = Box<dyn Fn(&mut Canvas)>;
    let cases: Vec<(&str, Set, Set)> = vec![
        (
            "mix",
            Box::new(move |c: &mut Canvas| c.set_physics_composition(Some(mix(0.5))).unwrap()),
            Box::new(move |c: &mut Canvas| c.set_physics_composition(Some(mix(2.0))).unwrap()),
        ),
        (
            "grouped",
            Box::new(move |c: &mut Canvas| {
                c.set_physics_composition(Some(grouped.clone())).unwrap()
            }),
            Box::new(|c: &mut Canvas| c.pick_law(PhysicsLaw::Springs).unwrap()),
        ),
        (
            "schedule",
            Box::new(move |c: &mut Canvas| c.run_physics_schedule(stages.clone())),
            Box::new(|c: &mut Canvas| {
                c.run_physics_schedule(vec![PhysicsStage::law(
                    PhysicsLaw::Springs,
                    StageStop::Rest,
                )])
            }),
        ),
    ];
    for (name, set, control) in cases {
        let (mut api, _) = canvas();
        set(&mut api);
        let saved = api.dynamics_spec().unwrap();
        let (mut opened, _) = canvas();
        opened.set_dynamics_spec(&saved).unwrap();
        assert_eq!(
            opened.physics_composition(),
            api.physics_composition(),
            "{name}"
        );
        let (mut other, _) = canvas();
        control(&mut other);
        let a = run(&mut api, 120);
        let b = run(&mut opened, 120);
        let c = run(&mut other, 120);
        let (same, differs) = (largest_gap(&a, &b), largest_gap(&a, &c));
        println!("{name}: opened from its spec {same:.5}, control {differs:.2}");
        assert!(same < 0.05, "{name}: {same}");
        assert!(differs > 1.0, "{name}: the control moves: {differs}");
    }
}

/// F156: the spec's seed reaches Kinds; the default seed reproduces the API.
#[test]
fn the_seed_reaches_kinds() {
    let (mut api, _) = canvas();
    api.pick_law(PhysicsLaw::Kinds).unwrap();
    let mut spec = api.dynamics_spec().unwrap();
    assert_eq!(spec.seed, spec::DEFAULT_SEED);
    let (mut same, _) = canvas();
    same.set_dynamics_spec(&spec).unwrap();
    spec.seed = 7;
    let (mut other, _) = canvas();
    other.set_dynamics_spec(&spec).unwrap();
    assert_eq!(other.dynamics_spec().unwrap().seed, 7);
    let a = run(&mut api, 90);
    let b = run(&mut same, 90);
    let c = run(&mut other, 90);
    let (kept, moved) = (largest_gap(&a, &b), largest_gap(&a, &c));
    println!("kinds, default seed {kept:.5}, seed 7 {moved:.2}");
    assert!(kept < 0.05 && moved > 1.0, "{kept} {moved}");
}

/// The target's roles and their group source come back from the spec, and
/// a grouping on a partition the host handed in has no spec (F157).
#[test]
fn roles_ride_the_target_and_a_given_partition_has_no_spec() {
    let (mut from, keys) = canvas();
    from.set_role_group_source(PhysicsKindSource::Cluster);
    from.set_arrangement_role(Role::Pinned);
    let member = from.graph.get_node(keys[5]).unwrap().id;
    assert!(from.set_member_role(member, Some(Role::Anchored)));
    from.set_anchor_stiffness(2.5);
    let saved = from.dynamics_spec().unwrap();
    let target = saved.target.clone().unwrap();
    assert_eq!(target.groups.unwrap().channel, "groups.cluster");
    let (mut to, _) = canvas();
    to.set_dynamics_spec(&saved).unwrap();
    assert_eq!(to.role_group_source(), PhysicsKindSource::Cluster);
    assert_eq!(to.arrangement_roles(), from.arrangement_roles());
    assert_eq!(to.anchor_stiffness(), 2.5);

    let given = PhysicsComposition::Grouped(PhysicsGrouping::charge_between(GroupSource::Given(
        keys.iter().map(|k| (*k, (k.index() % 2) as u32)).collect(),
    )));
    from.set_physics_composition(Some(given)).unwrap();
    let refusal = from.dynamics_spec().unwrap_err();
    assert!(refusal.contains("handed in"), "{refusal}");
    // A refused spec changes nothing.
    let mut bad = saved.clone();
    bad.channels.insert("kind".into(), "kind.colour".into());
    let before = to.dynamics_spec().unwrap();
    assert!(to.set_dynamics_spec(&bad).is_err());
    assert_eq!(to.dynamics_spec().unwrap(), before);
    assert!(bind(&saved).is_ok());
}
