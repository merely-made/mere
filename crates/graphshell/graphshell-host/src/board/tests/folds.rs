// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Relations become edges and folds hide their members (stack-lift D2).

use super::*;
use graphshell_endpoint::{ProjectionCatalog, ProjectionSource};
use sceno::{Fold, RoutedRelation, Scene};

/// A mounted live scene of `cards` cards.
fn mounted(cards: usize) -> MountedScene {
    let mut endpoint = LiveEndpoint::new();
    for _ in 1..cards {
        endpoint.append();
    }
    let snapshot = endpoint.snapshot(endpoint.describe().projections[0].request.clone()).unwrap();
    let mut client = ClientState::default();
    client.apply_snapshot(snapshot.clone()).unwrap();
    client.mounted(&snapshot.session).unwrap().clone()
}

fn relate(scene: &mut MountedScene, from: u32, to: u32, kind: &str) {
    let points = vec![Default::default(), Default::default()];
    scene.scene.tables.relations.push(Some(RoutedRelation {
        from: InstanceId(from),
        to: InstanceId(to),
        space: Scene::WORLD,
        points,
        kind: Some(kind.into()),
        weight: None,
        link_key: None,
        marks: Default::default(),
        strength: None,
    }));
}

fn ids(scene: &BoardScene) -> Vec<&str> {
    scene.cards.iter().map(|card| card.id.as_str()).collect()
}

#[test]
fn relations_become_edges_between_drawn_cards() {
    let mut scene = mounted(3);
    relate(&mut scene, 0, 1, "route");
    relate(&mut scene, 1, 2, "kin");
    let board = board_scene(&scene);
    assert_eq!(board.relations.len(), 2);
    assert_eq!(board.drawn_relations(), 2);
    assert_eq!(board.relations[0].kind.as_deref(), Some("route"));
}

#[test]
fn a_fold_hides_its_members_and_its_stand_in_carries_the_count() {
    let mut scene = mounted(5);
    relate(&mut scene, 2, 3, "resides");
    relate(&mut scene, 1, 2, "kin");
    scene.scene.tables.folds.push(Some(Fold {
        members: vec![InstanceId(0), InstanceId(1), InstanceId(2)],
        stand_in: StandIn::Member(InstanceId(0)),
        rule: None,
        boundary: None,
        label: Some("site 0's residents".into()),
    }));
    let board = board_scene(&scene);
    assert_eq!(ids(&board), vec!["0", "3", "4"], "hidden members are left out");
    assert_eq!(board.cards[0].folded, 2);
    assert!(board.cards[0].labelled().ends_with(" +2"));
    // A hidden end is lifted to its stand-in; an edge inside the fold is not drawn.
    assert_eq!(board.relations.len(), 1);
    assert_eq!((board.relations[0].from.as_str(), board.relations[0].to.as_str()), ("0", "3"));
    // Control: the same scene unfolded draws every card and both edges.
    scene.scene.tables.folds.clear();
    let unfolded = board_scene(&scene);
    assert_eq!(unfolded.cards.len(), 5);
    assert_eq!(unfolded.relations.len(), 2);
    assert!(unfolded.cards.iter().all(|card| card.folded == 0));
}

#[test]
fn a_summary_fold_draws_one_body_for_all_its_members() {
    let mut scene = mounted(4);
    scene.scene.tables.folds.push(Some(Fold {
        members: vec![InstanceId(1), InstanceId(2), InstanceId(3)],
        stand_in: StandIn::Summary { label: Some("three".into()) },
        rule: None,
        boundary: None,
        label: None,
    }));
    let board = board_scene(&scene);
    assert_eq!(ids(&board), vec!["0", "fold:0"]);
    assert_eq!(board.cards[1].labelled(), "three +3");
}
