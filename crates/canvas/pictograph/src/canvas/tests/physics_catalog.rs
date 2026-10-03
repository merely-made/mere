// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The physics catalog: every id builds, every label is plain, a switch does
//! not move a body, and the choice survives a graph reconcile.

use super::*;
use crate::canvas::physics_catalog::{
    CANVAS_PHYSICS_DEPTH_SOURCES, CANVAS_PHYSICS_KIND_SOURCES, CANVAS_PHYSICS_LAWS,
    CANVAS_PHYSICS_MASS_SOURCES, CANVAS_PHYSICS_OVERLAYS, CANVAS_PHYSICS_PROFILES,
    PhysicsDepthSource, PhysicsKindSource, PhysicsLaw, PhysicsMassSource, PhysicsOverlay,
};

/// A canvas with named nodes joined by directed semantic relations, for the
/// petgraph-backed sources. A pair listed more than once gets a different
/// relation kind each time (asserting one kind twice is idempotent), so it
/// carries that many relation cells.
fn wired(urls: &[&str], edges: &[(usize, usize)]) -> (Canvas, Vec<NodeKey>) {
    const KINDS: [SemanticSubKind; 4] = [
        SemanticSubKind::Cites,
        SemanticSubKind::Quotes,
        SemanticSubKind::Summarizes,
        SemanticSubKind::Elaborates,
    ];
    let mut canvas = Canvas::new();
    let ids: Vec<uuid::Uuid> = urls
        .iter()
        .map(|url| canvas.open_member_as_new_node(None, url))
        .collect();
    let mut seen: HashMap<(usize, usize), usize> = HashMap::new();
    for &(a, b) in edges {
        let nth = seen.entry((a, b)).or_default();
        assert!(canvas.assert_relation_between_members(ids[a], ids[b], KINDS[*nth]));
        *nth += 1;
    }
    let keys = ids
        .iter()
        .map(|id| canvas.graph().get_node_by_id(*id).unwrap().0)
        .collect();
    (canvas, keys)
}

fn lookup<T: Copy>(table: &[(NodeKey, T)], key: NodeKey) -> T {
    table
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
        .unwrap()
}

/// A label in the plain register: a short capitalised phrase, no ids or
/// technical punctuation.
fn is_plain(label: &str) -> bool {
    !label.is_empty()
        && label.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        && label.len() <= 16
        && !label.contains(['.', '_', '-', '(', ')'])
}

fn positions(canvas: &Canvas) -> Vec<(NodeKey, Point2D<f32>)> {
    let mut all: Vec<_> = canvas
        .graph()
        .nodes()
        .map(|(key, _)| (key, canvas.view.position_of(key).unwrap_or_default()))
        .collect();
    all.sort_by_key(|(key, _)| key.index());
    all
}

#[test]
fn every_law_id_round_trips_and_its_label_is_plain() {
    assert_eq!(CANVAS_PHYSICS_LAWS.len(), PhysicsLaw::ALL.len());
    for (id, label) in CANVAS_PHYSICS_LAWS {
        let law = PhysicsLaw::parse(id).unwrap_or_else(|| panic!("{id} parses"));
        assert_eq!(law.id(), *id);
        assert_eq!(law.label(), *label);
        assert!(is_plain(label), "{label} is plain");
    }
    assert_eq!(CANVAS_PHYSICS_OVERLAYS.len(), PhysicsOverlay::ALL.len());
    for (id, label) in CANVAS_PHYSICS_OVERLAYS {
        let overlay = PhysicsOverlay::parse(id).unwrap_or_else(|| panic!("{id} parses"));
        assert_eq!(overlay.id(), *id);
        assert_eq!(overlay.label(), *label);
        assert!(is_plain(label), "{label} is plain");
    }
    assert_eq!(
        CANVAS_PHYSICS_KIND_SOURCES.len(),
        PhysicsKindSource::ALL.len()
    );
    for (id, label) in CANVAS_PHYSICS_KIND_SOURCES {
        let source = PhysicsKindSource::parse(id).unwrap_or_else(|| panic!("{id} parses"));
        assert_eq!(source.id(), *id);
        assert_eq!(source.label(), *label);
        assert!(is_plain(label), "{label} is plain");
    }
    assert_eq!(
        CANVAS_PHYSICS_MASS_SOURCES.len(),
        PhysicsMassSource::ALL.len()
    );
    for (id, label) in CANVAS_PHYSICS_MASS_SOURCES {
        let source = PhysicsMassSource::parse(id).unwrap_or_else(|| panic!("{id} parses"));
        assert_eq!(source.id(), *id);
        assert_eq!(source.label(), *label);
        assert!(is_plain(label), "{label} is plain");
    }
    assert_eq!(
        CANVAS_PHYSICS_DEPTH_SOURCES.len(),
        PhysicsDepthSource::ALL.len()
    );
    for (id, label) in CANVAS_PHYSICS_DEPTH_SOURCES {
        let source = PhysicsDepthSource::parse(id).unwrap_or_else(|| panic!("{id} parses"));
        assert_eq!(source.id(), *id);
        assert_eq!(source.label(), *label);
        assert!(is_plain(label), "{label} is plain");
    }
    assert!(PhysicsLaw::parse("spring").is_none());
    assert!(PhysicsOverlay::parse("").is_none());
}

/// The signature numbers a receipt asserts on: a laid-out path has no
/// overlaps and stretches to about its hop count under Stress; a paused,
/// halted canvas has no energy.
#[test]
fn layout_stats_carry_the_laws_signatures() {
    let (mut canvas, keys) = wired(
        &[
            "https://a.test",
            "https://b.test",
            "https://c.test",
            "https://d.test",
        ],
        &[(0, 1), (1, 2), (2, 3)],
    );
    canvas.set_physics_paused(true);
    for (i, &key) in keys.iter().enumerate() {
        canvas
            .view
            .set_position(key, Point2D::new(i as f32 * 170.0, 0.0));
    }
    let stats = canvas.layout_stats();
    assert_eq!(stats.overlaps, 0);
    assert!(
        (stats.stretch - 3.0).abs() < 1e-3,
        "a straight path stretches to its hops: {}",
        stats.stretch
    );
    assert!(stats.spread > 100.0);
    assert_eq!(stats.energy, 0.0, "halted bodies carry no energy");
    // Fold the path onto one point: every pair overlaps, nothing stretches.
    for &key in &keys {
        canvas.view.set_position(key, Point2D::new(0.0, 0.0));
    }
    let folded = canvas.layout_stats();
    assert_eq!(folded.overlaps, 6);
    assert_eq!(folded.stretch, 0.0);
    assert_eq!(folded.spread, 0.0);
    // The linear form agrees on spread and overlaps, both ways folded.
    let linear = canvas.layout_stats_without_stretch();
    assert_eq!((linear.overlaps, linear.spread), (6, 0.0));
    for (i, &key) in keys.iter().enumerate() {
        // Two touching pairs: 0-1 at 20 apart, 2-3 at 35, the pairs far apart.
        let x = [0.0, 20.0, 900.0, 935.0][i];
        canvas.view.set_position(key, Point2D::new(x, 0.0));
    }
    let (full, linear) = (canvas.layout_stats(), canvas.layout_stats_without_stretch());
    assert_eq!(full.overlaps, 2);
    assert_eq!(linear.overlaps, full.overlaps);
    assert!((linear.spread - full.spread).abs() < 1e-3);
    assert_eq!(linear.stretch, 0.0);
}

/// A proper colouring never puts a kind beside itself; islands each get one.
#[test]
fn coloring_and_island_kinds_read_the_topology() {
    // A path a-b-c, and an island d.
    let (canvas, keys) = wired(
        &[
            "https://a.test",
            "https://b.test",
            "https://c.test",
            "https://d.test",
        ],
        &[(0, 1), (1, 2)],
    );
    let inputs = canvas.law_inputs();
    let coloring = inputs.coloring_groups();
    assert_ne!(lookup(&coloring, keys[0]), lookup(&coloring, keys[1]));
    assert_ne!(lookup(&coloring, keys[1]), lookup(&coloring, keys[2]));
    assert_eq!(
        lookup(&coloring, keys[0]),
        lookup(&coloring, keys[2]),
        "a path is two-colourable"
    );
    let islands = inputs.component_groups();
    assert_eq!(lookup(&islands, keys[0]), lookup(&islands, keys[2]));
    assert_ne!(lookup(&islands, keys[0]), lookup(&islands, keys[3]));
    let (kinds, count) = inputs.kinds(PhysicsKindSource::Coloring);
    assert_eq!(count, 2);
    assert_ne!(lookup(&kinds, keys[0]), lookup(&kinds, keys[1]));
}

/// The most linked-to node outranks its linkers, and the weights average one.
#[test]
fn page_rank_weights_favour_the_linked_to() {
    let (canvas, keys) = wired(
        &[
            "https://a.test",
            "https://b.test",
            "https://c.test",
            "https://hub.test",
        ],
        &[(0, 3), (1, 3), (2, 3)],
    );
    let inputs = canvas.law_inputs();
    let weights = inputs.page_rank_weights();
    let hub = lookup(&weights, keys[3]);
    assert!(
        hub > lookup(&weights, keys[0]) * 2.0,
        "the hub outranks a linker: {hub}"
    );
    let mean = weights.iter().map(|(_, w)| w).sum::<f32>() / weights.len() as f32;
    assert!((mean - 1.0).abs() < 1e-3, "weights average one, got {mean}");
    // And it is the mass source Orbit and the hub overlays read.
    let mut canvas = canvas;
    canvas.set_physics_overlays(vec![PhysicsOverlay::HubGravity]);
    canvas.set_physics_mass_source(PhysicsMassSource::PageRank);
    assert!(
        canvas.physics_forces_are_graph_bound(),
        "ranked hub weights follow the topology"
    );
    canvas.set_physics_law(PhysicsLaw::Orbit);
    assert!(canvas.law_force_count() >= 3);
}

/// Layers survive a cycle and order a chain; dominators count from the focus.
#[test]
fn layer_and_focus_depths_order_the_graph() {
    // a -> b -> c -> a (a cycle), c -> d, and e unreachable.
    let (mut canvas, keys) = wired(
        &[
            "https://a.test",
            "https://b.test",
            "https://c.test",
            "https://d.test",
            "https://e.test",
        ],
        &[(0, 1), (1, 2), (2, 0), (2, 3)],
    );
    let inputs = canvas.law_inputs();
    let layers = inputs.layer_depths();
    let (a, b, c, d) = (
        lookup(&layers, keys[0]),
        lookup(&layers, keys[1]),
        lookup(&layers, keys[2]),
        lookup(&layers, keys[3]),
    );
    assert!(d > c, "d sits below c: {d} > {c}");
    // One arc of the cycle is cut; the other two still order their ends.
    let ordered = [(a, b), (b, c), (c, a)]
        .iter()
        .filter(|(x, y)| y > x)
        .count();
    assert_eq!(
        ordered, 2,
        "two of the three cycle arcs keep their order: {a} {b} {c}"
    );
    // Depth from the focus counts dominators; the unreachable node sits below all.
    let focus = inputs.focus_depths(Some(keys[0]));
    assert_eq!(lookup(&focus, keys[0]), 0);
    assert_eq!(lookup(&focus, keys[1]), 1);
    assert_eq!(lookup(&focus, keys[2]), 2);
    assert_eq!(lookup(&focus, keys[3]), 3);
    assert_eq!(
        lookup(&focus, keys[4]),
        4,
        "the unreachable node is one below the deepest"
    );
    // Without a focus the Focus source is the Roots source.
    assert_eq!(inputs.focus_depths(None), inputs.root_depths());
    drop(inputs);
    canvas.set_physics_overlays(vec![PhysicsOverlay::DepthGravity]);
    canvas.set_physics_depth_source(PhysicsDepthSource::Layers);
    assert_eq!(canvas.law_force_count(), 4, "springs and the depth overlay");
}

/// The skeleton is a spanning tree that prefers the pairs with more relations,
/// and Stress reads those pairs as shorter.
#[test]
fn skeleton_and_weighted_stress_read_multiplicity() {
    // A triangle a-b-c, with b-c joined three times.
    let (mut canvas, keys) = wired(
        &["https://a.test", "https://b.test", "https://c.test"],
        &[(0, 1), (0, 2), (1, 2), (1, 2), (1, 2)],
    );
    let inputs = canvas.law_inputs();
    let tree = inputs.skeleton_edges();
    assert_eq!(
        tree.len(),
        2,
        "a spanning tree of three nodes has two edges"
    );
    let joins = |x: NodeKey, y: NodeKey| {
        tree.iter()
            .any(|(p, q)| (*p == x && *q == y) || (*p == y && *q == x))
    };
    assert!(
        joins(keys[1], keys[2]),
        "the thrice-joined pair is in the tree"
    );
    let distances = inputs.weighted_distances();
    let dist = |x: NodeKey, y: NodeKey| {
        distances
            .iter()
            .find(|(p, q, _)| (*p == x && *q == y) || (*p == y && *q == x))
            .map(|(_, _, d)| *d)
            .unwrap()
    };
    assert!(
        (dist(keys[1], keys[2]) - 1.0 / 3.0).abs() < 1e-5,
        "three relations are a third of a hop"
    );
    assert!((dist(keys[0], keys[1]) - 1.0).abs() < 1e-5);
    drop(inputs);
    canvas.set_physics_law(PhysicsLaw::Still);
    canvas.set_physics_overlays(vec![PhysicsOverlay::Skeleton]);
    assert_eq!(canvas.law_force_count(), 2, "the hold and the tree");
    assert!(
        canvas.physics_forces_are_graph_bound(),
        "the tree follows the topology"
    );
    assert!(canvas.apply_physics_profile("skeleton"));
    assert_eq!(canvas.physics_profile_id(), Some("skeleton"));
}

#[test]
fn every_law_and_overlay_builds_on_the_sample_graph_without_moving_a_body() {
    let mut canvas = Canvas::with_sample_graph();
    canvas.set_physics_paused(true);
    let before = positions(&canvas);
    for law in PhysicsLaw::ALL {
        canvas.set_physics_law(law);
        assert_eq!(canvas.physics_law(), law);
        let expected_min = if law == PhysicsLaw::Still { 0 } else { 1 };
        assert!(
            canvas.law_force_count() >= expected_min,
            "{} builds its forces",
            law.id()
        );
        assert_eq!(
            positions(&canvas),
            before,
            "{} switch moved a body",
            law.id()
        );
    }
    for overlay in PhysicsOverlay::ALL {
        assert!(
            canvas.toggle_physics_overlay(overlay),
            "{} toggles on",
            overlay.id()
        );
        assert_eq!(
            positions(&canvas),
            before,
            "{} switch moved a body",
            overlay.id()
        );
    }
    assert_eq!(canvas.physics_overlays().len(), PhysicsOverlay::ALL.len());
    // Still + every overlay: the hold, then exactly one force per overlay.
    canvas.set_physics_law(PhysicsLaw::Still);
    assert_eq!(canvas.law_force_count(), PhysicsOverlay::ALL.len() + 1);
    for overlay in PhysicsOverlay::ALL {
        assert!(
            !canvas.toggle_physics_overlay(overlay),
            "{} toggles off",
            overlay.id()
        );
    }
    assert_eq!(canvas.law_force_count(), 1, "still is the hold alone");
    for source in PhysicsKindSource::ALL {
        canvas.set_physics_law(PhysicsLaw::Kinds);
        canvas.set_physics_kind_source(source);
        assert_eq!(canvas.physics_kind_source(), source);
        assert!(
            canvas.law_force_count() >= 2,
            "kinds by {} builds",
            source.id()
        );
    }
}

#[test]
fn every_profile_applies_and_names_itself_back() {
    let mut canvas = Canvas::with_sample_graph();
    assert!(
        !canvas.apply_physics_profile("plasma"),
        "an unknown profile is refused"
    );
    for profile in CANVAS_PHYSICS_PROFILES {
        assert!(is_plain(profile.label), "{} is plain", profile.label);
        assert!(
            canvas.apply_physics_profile(profile.id),
            "{} applies",
            profile.id
        );
        assert_eq!(canvas.physics_law(), profile.law);
        assert_eq!(canvas.physics_overlays(), profile.overlays);
        assert_eq!(
            canvas.physics_profile_id(),
            Some(profile.id),
            "{} names itself back",
            profile.id
        );
    }
    // The donor's ten come first, under their own names.
    let donor: Vec<&str> = CANVAS_PHYSICS_PROFILES
        .iter()
        .take(10)
        .map(|p| p.id)
        .collect();
    assert_eq!(
        donor,
        [
            "liquid",
            "gas",
            "solid",
            "archipelago",
            "constellation",
            "crystal",
            "tide",
            "sediment",
            "magnet",
            "void"
        ]
    );
    // And no two profiles share a (law, overlays) pair, or the picker could not name the live one.
    for (i, a) in CANVAS_PHYSICS_PROFILES.iter().enumerate() {
        for b in &CANVAS_PHYSICS_PROFILES[i + 1..] {
            assert!(
                !(a.law == b.law && a.overlays == b.overlays),
                "{} and {} coincide",
                a.id,
                b.id
            );
        }
    }
    // Every law is one pick away: bare, under its own name or a donor's.
    for law in PhysicsLaw::ALL {
        assert!(
            CANVAS_PHYSICS_PROFILES
                .iter()
                .any(|p| p.law == law && p.overlays.is_empty()),
            "{} has a bare profile",
            law.id()
        );
    }
}

#[test]
fn a_living_law_runs_until_paused_and_a_graph_bound_law_survives_a_reconcile() {
    let mut canvas = Canvas::with_sample_graph();
    canvas.set_physics_law(PhysicsLaw::Orbit);
    assert!(canvas.physics_never_rests());
    assert!(canvas.is_settling(), "orbit keeps ticking");
    // Kinds is living too (F10's figures on the P2 fixture).
    let living: Vec<PhysicsLaw> = PhysicsLaw::ALL
        .into_iter()
        .filter(|law| law.never_rests())
        .collect();
    assert_eq!(
        living,
        [
            PhysicsLaw::Orbit,
            PhysicsLaw::Kinds,
            PhysicsLaw::Flock,
            PhysicsLaw::Sync
        ]
    );
    canvas.set_physics_law(PhysicsLaw::Stress);
    assert!(!canvas.physics_never_rests());
    let count = canvas.law_force_count();
    canvas.visit("https://a-new-node.example");
    assert_eq!(
        canvas.physics_law(),
        PhysicsLaw::Stress,
        "the law survives a topology change"
    );
    assert_eq!(
        canvas.law_force_count(),
        count,
        "stress rebuilt against the new topology"
    );
    // A graph swap keeps the choice too: the scene restore re-applies it afterwards anyway.
    canvas.set_graph(Graph::new());
    assert_eq!(canvas.physics_law(), PhysicsLaw::Stress);
}

/// What `never_rests` changes: switched to from rest, a living law (Kinds,
/// since F10) ticks on past the settle budget, where a resting law (Stress)
/// stops at it.
#[test]
fn from_rest_a_living_law_ticks_on_and_a_resting_one_stops() {
    use std::time::Duration;
    for (law, living) in [(PhysicsLaw::Kinds, true), (PhysicsLaw::Stress, false)] {
        let mut canvas = Canvas::with_sample_graph();
        canvas.resize(800, 600);
        canvas.park_physics();
        assert!(!canvas.is_settling(), "at rest before the switch");
        canvas.set_physics_law(law);
        for frame in 0..=u64::from(SETTLE_TICKS) + 60 {
            canvas.frame_at(
                800,
                600,
                Duration::from_micros(frame * 1_000_000 / 60),
                Default::default(),
            );
        }
        assert_eq!(
            canvas.is_settling(),
            living,
            "{} after the budget",
            law.id()
        );
    }
}

#[test]
fn a_whole_choice_applies_with_one_rebuild_and_reads_back() {
    use crate::canvas::PhysicsChoice;
    let mut canvas = Canvas::with_sample_graph();
    let choice = PhysicsChoice {
        law: PhysicsLaw::Kinds,
        overlays: vec![
            PhysicsOverlay::DepthGravity,
            PhysicsOverlay::HubGravity,
            PhysicsOverlay::DepthGravity,
        ],
        kind: PhysicsKindSource::Cluster,
        groups: PhysicsKindSource::Component,
        mass: PhysicsMassSource::PageRank,
        depth: PhysicsDepthSource::Layers,
    };
    let before = canvas.law_rebuilds();
    canvas.set_physics_choice(&choice);
    assert_eq!(
        canvas.law_rebuilds() - before,
        1,
        "one apply is one rebuild, whatever the sources"
    );
    let live = canvas.physics_choice();
    assert_eq!(live.law, PhysicsLaw::Kinds);
    assert_eq!(live.kind, PhysicsKindSource::Cluster);
    assert_eq!(live.groups, PhysicsKindSource::Component);
    assert_eq!(live.mass, PhysicsMassSource::PageRank);
    assert_eq!(live.depth, PhysicsDepthSource::Layers);
    assert_eq!(
        live.overlays,
        [PhysicsOverlay::DepthGravity, PhysicsOverlay::HubGravity],
        "duplicates collapse, order is kept"
    );
    assert!(canvas.is_settling(), "the apply earns a settle");
    // The same choice, built through the separate setters law-last, gives the
    // same force set: the one rebuild read the sources it was handed.
    let mut stepwise = Canvas::with_sample_graph();
    stepwise.set_physics_kind_source(choice.kind);
    stepwise.set_physics_group_source(choice.groups);
    stepwise.set_physics_mass_source(choice.mass);
    stepwise.set_physics_depth_source(choice.depth);
    stepwise.set_physics_overlays(choice.overlays.clone());
    let rebuilds = stepwise.law_rebuilds();
    stepwise.set_physics_law(choice.law);
    assert!(stepwise.law_rebuilds() > rebuilds);
    assert_eq!(stepwise.physics_choice(), live);
    assert_eq!(stepwise.law_force_count(), canvas.law_force_count());
    // A choice naming a profile's pair names that profile; any other is custom.
    canvas.set_physics_choice(&PhysicsChoice {
        law: PhysicsLaw::Springs,
        overlays: vec![PhysicsOverlay::GravityLocus],
        ..PhysicsChoice::default()
    });
    assert_eq!(canvas.physics_profile_id(), Some("liquid"));
    canvas.set_physics_choice(&PhysicsChoice {
        law: PhysicsLaw::Still,
        overlays: vec![PhysicsOverlay::Skeleton],
        ..PhysicsChoice::default()
    });
    assert_eq!(canvas.physics_profile_id(), None);
}
