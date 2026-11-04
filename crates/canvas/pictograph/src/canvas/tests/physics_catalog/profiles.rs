// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn every_profile_applies_and_names_itself_back() {
    let mut canvas = Canvas::with_sample_graph();
    assert!(
        !canvas.pick_profile("plasma"),
        "an unknown profile is refused"
    );
    for profile in CANVAS_PHYSICS_PROFILES {
        assert!(is_plain(profile.label), "{} is plain", profile.label);
        assert!(canvas.pick_profile(profile.id), "{} applies", profile.id);
        assert_eq!(canvas.view().law, profile.law);
        assert_eq!(canvas.view().overlays, profile.overlays);
        assert_eq!(
            canvas.view().profile_id(),
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
    canvas.pick_law(PhysicsLaw::Orbit).unwrap();
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
    canvas.pick_law(PhysicsLaw::Stress).unwrap();
    assert!(!canvas.physics_never_rests());
    let count = canvas.law_force_count();
    canvas.visit("https://a-new-node.example");
    assert_eq!(
        canvas.view().law,
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
    assert_eq!(canvas.view().law, PhysicsLaw::Stress);
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
        canvas.pick_law(law).unwrap();
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
    canvas.pick(&choice).unwrap();
    assert_eq!(
        canvas.law_rebuilds() - before,
        1,
        "one apply is one rebuild, whatever the sources"
    );
    let live = canvas.view();
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
    stepwise.pick_kind(choice.kind);
    stepwise.pick_groups(choice.groups);
    stepwise.pick_mass(choice.mass);
    stepwise.pick_depth(choice.depth);
    stepwise.pick_overlays(choice.overlays.clone()).unwrap();
    let rebuilds = stepwise.law_rebuilds();
    stepwise.pick_law(choice.law).unwrap();
    assert!(stepwise.law_rebuilds() > rebuilds);
    assert_eq!(stepwise.view(), live);
    assert_eq!(stepwise.law_force_count(), canvas.law_force_count());
    // A choice naming a profile's pair names that profile; any other is custom.
    canvas
        .pick(&PhysicsChoice {
            law: PhysicsLaw::Springs,
            overlays: vec![PhysicsOverlay::GravityLocus],
            ..PhysicsChoice::default()
        })
        .unwrap();
    assert_eq!(canvas.view().profile_id(), Some("liquid"));
    canvas
        .pick(&PhysicsChoice {
            law: PhysicsLaw::Still,
            overlays: vec![PhysicsOverlay::Skeleton],
            ..PhysicsChoice::default()
        })
        .unwrap();
    assert_eq!(canvas.view().profile_id(), None);
}
