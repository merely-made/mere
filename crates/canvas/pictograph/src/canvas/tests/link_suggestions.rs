// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A page's unvisited links as derived members.

use super::*;
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
    assert_eq!(kept[0].origin, page);
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
