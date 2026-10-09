// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! G2c's done-conditions on the canvas (dynamics grammar plan, G2c; F174,
//! F178 to F181): every physics channel is the registry's, computed once per
//! physics view with all its readers live and once more when its key moves;
//! the view revision moves with every hide and show that changes the view;
//! `edges.spanning` resolves; a raw term's input slot resolves against the
//! registry while running a raw term stays refused.

use std::collections::BTreeSet;

use kernel::graph::{EdgeFamily, NodeKey, RelationSelector, SemanticSubKind};

use super::arrangement_goldens::topic_fixture;
use crate::canvas::Canvas;
use crate::canvas::channels::{Channel, ChannelValues, EdgeSubset};
use crate::canvas::dynamics_spec::{
    BindError, DynamicsSpec, InputSlot, Node, RAW_SLOT_DEPTHS, RAW_SLOT_GROUPS, RAW_SLOT_KINDS,
    RAW_SLOT_MASSES, RAW_SLOT_PAIRS, RAW_SLOTS, RawTerm,
};
use crate::canvas::physics_catalog::{
    LawInputs, PhysicsDepthSource, PhysicsKindSource, PhysicsLaw, PhysicsMassSource, PhysicsOverlay,
};
use crate::canvas::tests::ThroughView;
use crate::signals::PhysicsRuns;

/// Every channel the physics view produces, the kinds folded from its groups.
fn physics_channels() -> Vec<Channel> {
    let mut all = vec![
        Channel::Mass(PhysicsMassSource::Degree),
        Channel::Mass(PhysicsMassSource::PageRank),
    ];
    for source in [
        PhysicsKindSource::Coloring,
        PhysicsKindSource::Component,
        PhysicsKindSource::Degree,
    ] {
        all.extend([Channel::Groups(source), Channel::Kind(source)]);
    }
    all.extend(PhysicsDepthSource::ALL.map(Channel::Depth));
    all.extend([Channel::Distances, Channel::Edges(EdgeSubset::Spanning)]);
    all
}

fn runs(n: u64) -> PhysicsRuns {
    PhysicsRuns {
        mass_degree: n,
        pagerank: n,
        coloring: n,
        components: n,
        degree_bands: n,
        depth_roots: n,
        depth_layers: n,
        depth_focus: n,
        distances: n,
        spanning: n,
    }
}

/// Every reader of every physics fact, three times over: a law build reading
/// each (Orbit's masses, PageRank's hub weights, Kinds by each topology
/// source, the Depth overlay by each depth, Stress, the skeleton), every
/// physics channel read, the layout's stats, and frames.
fn read_everything(canvas: &mut Canvas) {
    for _ in 0..3 {
        for mass in PhysicsMassSource::ALL {
            canvas.pick_mass(mass);
            let _ = canvas.pick_law(PhysicsLaw::Orbit);
            let _ = canvas.pick_law(PhysicsLaw::Springs);
            let _ = canvas.pick_overlays(vec![PhysicsOverlay::HubGravity]);
            let _ = canvas.pick_overlays(Vec::new());
        }
        for kind in [
            PhysicsKindSource::Coloring,
            PhysicsKindSource::Component,
            PhysicsKindSource::Degree,
        ] {
            canvas.pick_kind(kind);
            let _ = canvas.pick_law(PhysicsLaw::Kinds);
            let _ = canvas.pick_law(PhysicsLaw::Springs);
            canvas.pick_groups(kind);
            let _ = canvas.pick_overlays(vec![PhysicsOverlay::DomainCluster]);
            let _ = canvas.pick_overlays(Vec::new());
        }
        for depth in PhysicsDepthSource::ALL {
            canvas.pick_depth(depth);
            let _ = canvas.pick_overlays(vec![PhysicsOverlay::DepthGravity]);
            let _ = canvas.pick_overlays(Vec::new());
        }
        let _ = canvas.pick_law(PhysicsLaw::Stress);
        let _ = canvas.pick_law(PhysicsLaw::Springs);
        let _ = canvas.pick_overlays(vec![PhysicsOverlay::Skeleton]);
        for channel in physics_channels() {
            canvas.channel_values(channel);
        }
        canvas.layout_stats();
        for _ in 0..3 {
            canvas.frame(1024, 600);
        }
        let _ = canvas.pick_overlays(Vec::new());
    }
}

fn member(canvas: &Canvas, key: NodeKey) -> uuid::Uuid {
    canvas.graph().get_node(key).expect("a fixture node").id
}

/// The fixture's linked pairs, each once, in relation order.
fn linked_pairs(canvas: &Canvas) -> Vec<(uuid::Uuid, uuid::Uuid)> {
    let mut seen = BTreeSet::new();
    let mut pairs = Vec::new();
    for (_, relation) in canvas.graph().projected_relations() {
        let pair = (
            relation.from.min(relation.to),
            relation.from.max(relation.to),
        );
        if relation.from != relation.to && seen.insert(pair) {
            pairs.push((member(canvas, relation.from), member(canvas, relation.to)));
        }
    }
    pairs
}

/// The topic fixture's links are open predicates, one semantic-family cell.
const LINKS: RelationSelector = RelationSelector::Family(EdgeFamily::Semantic);

#[test]
fn every_physics_channel_runs_once_with_its_readers_live_and_once_more_when_its_key_moves() {
    let (graph, keys) = topic_fixture();
    let mut canvas = Canvas::with_graph(graph);
    canvas.select_member(member(&canvas, keys[3]));

    read_everything(&mut canvas);
    assert_eq!(
        canvas.channels.physics_runs(),
        runs(1),
        "every reader live, three times over: each fact computed once"
    );

    // The uncached path (F179: a subgraph's, the board's) over the same
    // view gives the same values and leaves the registry's counts alone.
    let before = canvas.channels.physics_runs();
    {
        let focus = canvas.focused_key();
        let served = canvas.law_inputs();
        let bare = LawInputs::new(canvas.graph(), &canvas.hidden_edges, None, None);
        for mass in PhysicsMassSource::ALL {
            assert_eq!(served.mass_values(mass), bare.mass_values(mass));
            assert_eq!(served.masses(mass), bare.masses(mass));
        }
        for source in PhysicsKindSource::ALL {
            assert_eq!(served.groups(source), bare.groups(source));
            assert_eq!(served.kinds(source), bare.kinds(source));
        }
        for depth in PhysicsDepthSource::ALL {
            assert_eq!(
                served.depth_values(depth, focus),
                bare.depth_values(depth, focus)
            );
        }
        assert_eq!(served.skeleton_edges(), bare.skeleton_edges());
        assert_eq!(served.weighted_distances(), bare.weighted_distances());
    }
    assert_eq!(canvas.channels.physics_runs(), before);

    // A structural change: every fact once more.
    let linked: BTreeSet<_> = linked_pairs(&canvas).into_iter().collect();
    let (a, b) = keys
        .iter()
        .flat_map(|&x| keys.iter().map(move |&y| (x, y)))
        .map(|(x, y)| (member(&canvas, x), member(&canvas, y)))
        .find(|&(x, y)| x != y && !linked.contains(&(x, y)) && !linked.contains(&(y, x)))
        .expect("an unlinked pair");
    assert!(canvas.assert_relation_between_members(a, b, SemanticSubKind::Cites));
    read_everything(&mut canvas);
    assert_eq!(canvas.channels.physics_runs(), runs(2), "structure moved");

    // A view change, the graph's structure unmoved: every fact once more.
    let structure = canvas.physics_view_key().structure;
    let (c, d) = linked_pairs(&canvas)[0];
    assert!(canvas.hide_relation_between_members(c, d, LINKS));
    assert_eq!(canvas.physics_view_key().structure, structure);
    read_everything(&mut canvas);
    assert_eq!(canvas.channels.physics_runs(), runs(3), "the view moved");

    // A focus change: only the focus's depth.
    canvas.select_member(member(&canvas, keys[9]));
    read_everything(&mut canvas);
    assert_eq!(
        canvas.channels.physics_runs(),
        PhysicsRuns {
            depth_focus: 4,
            ..runs(3)
        },
        "the focus moved"
    );

    // A hide that changes nothing moves no key.
    assert!(!canvas.hide_relation_between_members(c, d, LINKS));
    read_everything(&mut canvas);
    assert_eq!(
        canvas.channels.physics_runs(),
        PhysicsRuns {
            depth_focus: 4,
            ..runs(3)
        },
        "nothing moved"
    );

    // edges.spanning resolves (F174, F180), to what the skeleton reads.
    let id = Channel::Edges(EdgeSubset::Spanning).id();
    assert_eq!(id, "edges.spanning");
    assert_eq!(
        Channel::parse(&id),
        Some(Channel::Edges(EdgeSubset::Spanning))
    );
    let ChannelValues::Edges(edges) = canvas.channel_values(Channel::Edges(EdgeSubset::Spanning))
    else {
        panic!("edges.spanning is an edge set");
    };
    assert_eq!(edges, canvas.law_inputs().skeleton_edges());
    assert!(!edges.is_empty());
    assert_eq!(canvas.channels.physics_runs().spanning, 3);
}

/// A hide or show that `did` change the view moved its revision by one.
fn changed(canvas: &Canvas, at: &mut u64, did: bool, what: &str) {
    assert!(did, "{what} changed the view");
    assert_eq!(
        canvas.physics_view_key().view,
        *at + 1,
        "{what} bumps the revision"
    );
    *at += 1;
}

#[test]
fn every_hide_and_show_that_changes_the_view_bumps_its_revision() {
    let (graph, _) = topic_fixture();
    let mut canvas = Canvas::with_graph(graph);
    let view = |canvas: &Canvas| canvas.physics_view_key().view;
    let structure = canvas.physics_view_key().structure;
    let pairs = linked_pairs(&canvas);
    let (a, b) = pairs[0];
    let (c, d) = pairs[1];

    // Controls: nothing hidden, so no show changes the view.
    let mut at = view(&canvas);
    assert!(!canvas.show_relation_between_members(a, b, LINKS));
    assert!(!canvas.show_edge_between_members(a, b));
    assert_eq!(canvas.show_all_edges(), 0);
    assert_eq!(canvas.hide_selected_edges(), 0);
    assert_eq!(view(&canvas), at, "a show or hide that changes nothing");

    let did = canvas.hide_relation_between_members(a, b, LINKS);
    changed(&canvas, &mut at, did, "hide_relation_between_members");
    assert!(!canvas.hide_relation_between_members(a, b, LINKS));
    assert_eq!(view(&canvas), at, "a no-op hide leaves it");
    let did = canvas.show_relation_between_members(a, b, LINKS);
    changed(&canvas, &mut at, did, "show_relation_between_members");

    let did = canvas.hide_edge_between_members(c, d);
    changed(&canvas, &mut at, did, "hide_edge_between_members");
    assert!(!canvas.hide_edge_between_members(c, d));
    assert_eq!(view(&canvas), at, "a no-op hide leaves it");
    let did = canvas.show_edge_between_members(c, d);
    changed(&canvas, &mut at, did, "show_edge_between_members");

    let (_, relation) = canvas
        .graph()
        .projected_relations()
        .next()
        .expect("a relation");
    let cell = crate::canvas::edge_cells::edge_cell_for_relation(
        relation.from,
        relation.to,
        relation.kind,
    );
    canvas.selected_edges.insert(cell);
    let did = canvas.hide_selected_edges() == 1;
    changed(&canvas, &mut at, did, "hide_selected_edges");
    let did = canvas.show_all_edges() > 0;
    changed(&canvas, &mut at, did, "show_all_edges");

    assert_eq!(
        canvas.physics_view_key().structure,
        structure,
        "no hide or show moves the graph's structure: the view revision is what keys them"
    );
}

#[test]
fn a_raw_terms_input_slot_resolves_against_the_registry_and_running_it_stays_refused() {
    let (graph, _) = topic_fixture();
    let mut canvas = Canvas::with_graph(graph);

    // The slots seiche's raw terms name are the ones the resolver knows.
    let mut slots: Vec<(&'static str, bool)> = Vec::new();
    for term in RawTerm::defaults() {
        for input in term.inputs() {
            if !slots.contains(&(input.slot, input.required)) {
                slots.push((input.slot, input.required));
            }
        }
    }
    let named: BTreeSet<&str> = slots.iter().map(|(slot, _)| *slot).collect();
    assert_eq!(named, RAW_SLOTS.into_iter().collect::<BTreeSet<_>>());

    let mut readable: Vec<(&str, String)> = Vec::new();
    for &(slot, required) in &slots {
        let place = format!("root.{slot}");
        for channel in Channel::all() {
            let id = channel.id();
            let reads = match slot {
                RAW_SLOT_MASSES => matches!(channel, Channel::Mass(_)),
                RAW_SLOT_KINDS => matches!(channel, Channel::Kind(_)),
                RAW_SLOT_GROUPS => matches!(channel, Channel::Groups(_)),
                RAW_SLOT_DEPTHS => matches!(channel, Channel::Depth(_)),
                RAW_SLOT_PAIRS => matches!(channel, Channel::Distances | Channel::Edges(_)),
                _ => false,
            };
            let input = InputSlot {
                slot,
                channel: Some(&id),
                required,
            };
            match canvas.resolve_raw_input("root", &input) {
                Ok(Some(values)) => {
                    assert!(reads, "{slot} reads {id}");
                    assert_eq!(values, canvas.channel_values(channel), "{slot} {id}");
                    if !readable.contains(&(slot, id.clone())) {
                        readable.push((slot, id));
                    }
                },
                Err(BindError::Unbound { place: at, reason }) => {
                    assert!(!reads, "{slot} refuses {id}: {reason}");
                    assert_eq!(at, place);
                    assert!(reason.contains(&id), "{reason}");
                },
                other => panic!("{slot} {id}: {other:?}"),
            }
        }
        let empty = InputSlot {
            slot,
            channel: None,
            required,
        };
        match canvas.resolve_raw_input("root", &empty) {
            Ok(None) => assert!(!required, "{slot} left empty"),
            Err(BindError::Unbound { place: at, .. }) => {
                assert!(required, "{slot} left empty");
                assert_eq!(at, place);
            },
            other => panic!("{slot} empty: {other:?}"),
        }
        let unknown = InputSlot {
            slot,
            channel: Some("mass.planet"),
            required,
        };
        assert!(matches!(
            canvas.resolve_raw_input("root", &unknown),
            Err(BindError::Unbound { reason, .. }) if reason.contains("unknown channel mass.planet")
        ));
    }
    let count = |slot: &str| readable.iter().filter(|(s, _)| *s == slot).count();
    assert_eq!(count(RAW_SLOT_MASSES), 2);
    assert_eq!(count(RAW_SLOT_KINDS), 6);
    assert_eq!(count(RAW_SLOT_GROUPS), 6, "groups.bridges is refused");
    assert_eq!(count(RAW_SLOT_DEPTHS), 3);
    assert_eq!(
        count(RAW_SLOT_PAIRS),
        2,
        "distances.hops and edges.spanning"
    );
    assert_eq!(count("radii"), 0, "no family carries radii");

    // Running a raw term stays refused (F155), whichever of derive and bind
    // refuses it, and the record is untouched.
    let before = canvas.dynamics_spec();
    let spec = DynamicsSpec::new(Node::raw(RawTerm::Hold(Default::default())));
    let refused = canvas.set_dynamics_spec(&spec);
    assert!(refused.is_err(), "{refused:?}");
    assert_eq!(canvas.dynamics_spec(), before);
}
