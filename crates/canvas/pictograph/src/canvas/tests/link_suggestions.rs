// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A page's unvisited links as derived members.

use super::*;
use crate::canvas::link_suggestions::{KeptLink, LINK_SOURCE};
use scenograph::attention::Attention;
use scenograph::suggestion::SuggestionSettings;

fn page_canvas() -> (Canvas, uuid::Uuid) {
    let mut graph = Graph::new();
    graph.add_node("https://a.example/page".into(), PortablePoint::zero());
    graph.add_node(
        "https://a.example/held".into(),
        PortablePoint::new(80.0, 0.0),
    );
    let mut canvas = Canvas::with_graph(graph);
    canvas.resize(800, 600);
    canvas.set_physics_paused(true);
    let page = canvas
        .graph()
        .nodes()
        .find(|(_, node)| node.url().ends_with("/page"))
        .unwrap()
        .1
        .id;
    (canvas, page)
}

fn links() -> Vec<String> {
    ["held", "one", "two", "three"]
        .map(|p| format!("https://a.example/{p}"))
        .to_vec()
}

fn on(at_once: u32) -> SuggestionSettings {
    SuggestionSettings {
        on: true,
        at_once,
        ..SuggestionSettings::default()
    }
}

#[test]
fn links_become_derived_members_only_while_suggestions_admit_them() {
    let (mut canvas, page) = page_canvas();
    canvas.offer_page_links(page, links());
    assert!(canvas.derived_members().is_empty(), "off by default");

    canvas.set_suggestion_settings(on(2));
    let derived = canvas.derived_members();
    assert_eq!(derived.len(), 2);
    let urls: Vec<String> = derived
        .iter()
        .map(|id| {
            canvas
                .graph()
                .get_node_by_id(*id)
                .unwrap()
                .1
                .url()
                .to_string()
        })
        .collect();
    assert!(urls.contains(&"https://a.example/one".to_string()));
    assert!(
        !urls.iter().any(|u| u.ends_with("/held")),
        "held links are skipped"
    );
    let mut suggested = canvas.members_with_attention(Attention::Suggestion);
    suggested.sort();
    let mut expected = derived.clone();
    expected.sort();
    assert_eq!(suggested, expected);
    let origin = canvas.graph().get_node_key_by_id(page).unwrap();
    let linked = canvas
        .graph()
        .projected_relations()
        .filter(|(_, r)| r.from == origin)
        .count();
    assert_eq!(linked, 2, "each derived member is linked from its page");
    assert!(
        canvas
            .cartography_geometry()
            .members()
            .iter()
            .all(|m| !derived.contains(m)),
        "derived members are not saved"
    );

    canvas.set_suggestion_settings(SuggestionSettings::default());
    assert!(canvas.derived_members().is_empty(), "off removes them");
    assert_eq!(canvas.graph().nodes().count(), 2);
}

#[test]
fn selecting_a_derived_member_keeps_it_and_a_reinstall_mints_again() {
    let (mut canvas, page) = page_canvas();
    canvas.set_suggestion_settings(on(2));
    canvas.offer_page_links(page, links());
    let first = canvas.derived_members()[0];
    canvas.select_member(first);
    canvas.frame(800, 600);
    let kept = canvas.take_kept_links();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].member, first);
    assert_eq!(kept[0].origin, Some(page));
    assert!(canvas.take_kept_links().is_empty(), "taken once");
    assert!(
        !canvas
            .members_with_attention(Attention::Suggestion)
            .contains(&first),
        "a kept link is no longer offered"
    );

    // A host reinstalls its own graph (without derived members): the next
    // frame mints the offered links again under the same ids.
    let others: Vec<uuid::Uuid> = canvas
        .derived_members()
        .into_iter()
        .filter(|id| *id != first)
        .collect();
    let mut owner = Graph::new();
    owner.add_node_with_id(page, "https://a.example/page".into(), PortablePoint::zero());
    canvas.set_graph(owner);
    canvas.frame(800, 600);
    for id in others {
        assert!(canvas.is_derived_member(id) && canvas.graph().get_node_by_id(id).is_some());
    }
}

fn sorted(mut ids: Vec<uuid::Uuid>) -> Vec<uuid::Uuid> {
    ids.sort();
    ids
}

/// A page and three scene members, 800 by 600.
fn page_and_scene() -> (Canvas, uuid::Uuid, Vec<uuid::Uuid>) {
    let mut graph = Graph::new();
    let page = graph.add_node("https://a.example/page".into(), PortablePoint::zero());
    let scene: Vec<_> = (0..3)
        .map(|i| graph.add_node(format!("https://a.example/scene{i}"), PortablePoint::zero()))
        .collect();
    let id = |key| graph.get_node(key).unwrap().id;
    let (page, scene) = (id(page), scene.into_iter().map(id).collect());
    let mut canvas = Canvas::with_graph(graph);
    canvas.resize(800, 600);
    canvas.set_physics_paused(true);
    (canvas, page, scene)
}

/// Minted links are exactly the admitted link members: attention is the scene
/// members `scene` plus the derived ones. Returns how many are derived.
fn admitted_agree(canvas: &Canvas, scene: &[uuid::Uuid]) -> usize {
    let derived = canvas.derived_members();
    let mut expected = scene.to_vec();
    expected.extend(&derived);
    let suggested = canvas.members_with_attention(Attention::Suggestion);
    assert_eq!(sorted(suggested), sorted(expected));
    derived.len()
}

#[test]
fn mixed_scene_and_link_offers_with_an_excluded_source_admit_one_set() {
    let (mut canvas, page, scene) = page_and_scene();
    let settings = |sources: &[&str]| SuggestionSettings {
        sources: sources.iter().map(|s| s.to_string()).collect(),
        ..on(3)
    };
    let offer = |canvas: &mut Canvas, n: usize| {
        canvas.offer_suggestions(scene[..n].iter().map(|id| (*id, "scene".to_string())));
    };
    canvas.offer_page_links(
        page,
        ["one", "two", "three"].map(|p| format!("https://a.example/{p}")),
    );
    offer(&mut canvas, 2);

    canvas.set_suggestion_settings(settings(&["scene"]));
    assert_eq!(
        admitted_agree(&canvas, &scene[..2]),
        0,
        "the links source is excluded"
    );

    canvas.set_suggestion_settings(settings(&["scene", LINK_SOURCE]));
    assert_eq!(
        admitted_agree(&canvas, &scene[..2]),
        1,
        "two offers leave one slot"
    );
    let member = canvas.derived_members()[0];
    let url = canvas
        .graph()
        .get_node_by_id(member)
        .unwrap()
        .1
        .url()
        .to_string();
    assert_eq!(url, "https://a.example/one", "the first link offered");

    offer(&mut canvas, 3);
    assert_eq!(
        admitted_agree(&canvas, &scene),
        0,
        "scene offers displace the link"
    );
    offer(&mut canvas, 1);
    assert_eq!(
        admitted_agree(&canvas, &scene[..1]),
        2,
        "fewer offers free slots"
    );

    canvas.set_suggestion_settings(settings(&[LINK_SOURCE]));
    assert_eq!(
        admitted_agree(&canvas, &[]),
        3,
        "offers from an excluded source take no slot"
    );
}

#[test]
fn a_minted_link_stays_admitted_on_a_small_plate() {
    let mut graph = Graph::new();
    let key = graph.add_node("https://a.example/page".into(), PortablePoint::zero());
    let page = graph.get_node(key).unwrap().id;
    let mut canvas = Canvas::with_graph(graph);
    canvas.resize(60, 60);
    canvas.set_physics_paused(true);
    canvas.set_suggestion_settings(on(1));
    canvas.offer_page_links(page, ["https://a.example/one".to_string()]);
    assert_eq!(admitted_agree(&canvas, &[]), 1);
}

/// Two pages that offer one URL, and the member they share.
fn shared_link() -> (Canvas, [uuid::Uuid; 2], uuid::Uuid) {
    let mut graph = Graph::new();
    let keys = ["first", "second"]
        .map(|p| graph.add_node(format!("https://a.example/{p}"), PortablePoint::zero()));
    let pages = keys.map(|key| graph.get_node(key).unwrap().id);
    let mut canvas = Canvas::with_graph(graph);
    canvas.resize(800, 600);
    canvas.set_physics_paused(true);
    canvas.set_suggestion_settings(on(2));
    for page in pages {
        canvas.offer_page_links(page, ["https://a.example/shared".to_string()]);
    }
    let [member] = canvas.derived_members()[..] else {
        panic!("one member per URL");
    };
    (canvas, pages, member)
}

fn holders(canvas: &Canvas, member: uuid::Uuid) -> Vec<uuid::Uuid> {
    let to = canvas.graph().get_node_key_by_id(member).unwrap();
    let from = canvas
        .graph()
        .projected_relations()
        .filter(|(_, relation)| relation.to == to)
        .map(|(_, relation)| canvas.graph().get_node(relation.from).unwrap().id);
    sorted(from.collect())
}

fn keep(canvas: &mut Canvas, member: uuid::Uuid) -> KeptLink {
    canvas.select_member(member);
    canvas.frame(800, 600);
    let kept = canvas.take_kept_links();
    assert_eq!(kept.len(), 1);
    kept[0].clone()
}

#[test]
fn a_shared_url_keeps_its_origin_from_the_page_that_still_holds_the_relation() {
    let (mut canvas, [first, second], member) = shared_link();
    assert_eq!(holders(&canvas, member), sorted(vec![first, second]));
    canvas.select_member(first);
    assert_eq!(canvas.remove_focused(), Some(first));
    assert_eq!(keep(&mut canvas, member).origin, Some(second));
}

#[test]
fn several_holding_pages_name_the_least_id_as_origin() {
    let (mut canvas, pages, member) = shared_link();
    assert_eq!(
        keep(&mut canvas, member).origin,
        Some(pages[0].min(pages[1]))
    );
}

#[test]
fn a_page_that_withdraws_its_offer_stops_being_the_origin() {
    let (mut canvas, pages, member) = shared_link();
    let (low, high) = (pages[0].min(pages[1]), pages[0].max(pages[1]));
    canvas.offer_page_links(low, Vec::<String>::new());
    assert_eq!(
        holders(&canvas, member),
        [high],
        "the relation went with the offer"
    );
    assert_eq!(keep(&mut canvas, member).origin, Some(high));
}

#[test]
fn a_keep_with_no_page_left_names_no_origin() {
    let (mut canvas, page) = page_canvas();
    canvas.set_suggestion_settings(on(1));
    canvas.offer_page_links(page, ["https://a.example/one".to_string()]);
    let member = canvas.derived_members()[0];
    canvas.select_member(page);
    canvas.remove_focused();
    let kept = keep(&mut canvas, member);
    assert_eq!((kept.member, kept.origin), (member, None));
}
