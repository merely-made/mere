// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The graph grammar's node and link presentation through a frame.

use super::*;
use crate::canvas::grammar_view::{LINK_QUIET, link_tone};
use paint_list_api::PaintCmd;
use scenograph::attention::Attention;

fn class_of(canvas: &Canvas, key: NodeKey) -> String {
    canvas
        .node_document
        .dom()
        .attribute(
            canvas.gnode_of[&key],
            &Namespace::from(""),
            &LocalName::from("class"),
        )
        .unwrap()
        .to_owned()
}

/// A hub linked to two pages on its site, a lone page on the same site, and
/// a page on another site; spread and paused.
fn grammar_canvas() -> (Canvas, [NodeKey; 5]) {
    let mut graph = Graph::new();
    let urls = [
        "https://a.example/hub",
        "https://a.example/one",
        "https://a.example/two",
        "https://a.example/lone",
        "https://b.example/far",
    ];
    let keys = urls.map(|url| graph.add_node(url.to_string(), PortablePoint::zero()));
    graph.assert_relation(keys[0], keys[1], hyperlink());
    graph.assert_relation(keys[0], keys[2], hyperlink());
    let mut canvas = Canvas::with_graph(graph);
    canvas.resize(800, 600);
    let spread = [
        (0.0, 0.0),
        (-120.0, 60.0),
        (120.0, 60.0),
        (-200.0, -150.0),
        (220.0, -160.0),
    ];
    for (key, (x, y)) in keys.iter().zip(spread) {
        canvas.view.set_position(*key, PortablePoint::new(x, y));
    }
    canvas.set_physics_paused(true);
    canvas.cursor_moved(-10_000.0, -10_000.0);
    (canvas, keys)
}

fn id(canvas: &Canvas, key: NodeKey) -> uuid::Uuid {
    canvas.graph().get_node(key).unwrap().id
}

#[test]
fn without_the_grammar_the_activation_colours_stand() {
    let (mut canvas, keys) = grammar_canvas();
    canvas.frame(800, 600);
    assert!(class_of(&canvas, keys[0]).starts_with("gnode-idle"));
}

#[test]
fn forward_bodies_labelled_neighbours_and_quiet_sites() {
    let (mut canvas, keys) = grammar_canvas();
    canvas.set_graph_grammar(true);
    canvas.select_member(id(&canvas, keys[0]));
    canvas.frame(800, 600);
    assert!(class_of(&canvas, keys[0]).starts_with("gnode-body"));
    for neighbour in [keys[1], keys[2]] {
        let class = class_of(&canvas, neighbour);
        assert!(class.starts_with("gnode-site gnode-cat-"), "{class}");
        assert!(
            !class.contains("gnode-quiet"),
            "a neighbour is labelled: {class}"
        );
    }
    assert!(
        class_of(&canvas, keys[3]).contains("gnode-quiet"),
        "ambient is unlabelled"
    );
    let category = |key| class_of(&canvas, key).split(' ').nth(1).unwrap().to_owned();
    assert_eq!(
        category(keys[1]),
        category(keys[3]),
        "one site, one category"
    );
    assert_ne!(
        category(keys[1]),
        category(keys[4]),
        "another site, another category"
    );
}

#[test]
fn retained_and_suggested_nodes_carry_their_marks() {
    let (mut canvas, keys) = grammar_canvas();
    canvas.set_graph_grammar(true);
    canvas.select_member(id(&canvas, keys[0]));
    canvas.select_member(id(&canvas, keys[3]));
    canvas.set_suggestion_settings(scenograph::suggestion::SuggestionSettings {
        on: true,
        ..Default::default()
    });
    canvas.set_suggested_members([id(&canvas, keys[4])]);
    canvas.frame(800, 600);
    let hub = class_of(&canvas, keys[0]);
    assert!(
        hub.contains("gnode-retained") && !hub.contains("gnode-quiet"),
        "{hub}"
    );
    assert!(class_of(&canvas, keys[4]).contains("gnode-suggestion"));
    let style = canvas
        .node_document
        .dom()
        .attribute(
            canvas.gnode_of[&keys[4]],
            &Namespace::from(""),
            &LocalName::from("style"),
        )
        .unwrap()
        .to_owned();
    assert!(style.contains("width: 16.19"), "the smallest mark: {style}");
}

#[test]
fn links_are_quiet_until_they_touch_the_foreground() {
    let (mut canvas, keys) = grammar_canvas();
    assert_eq!(
        link_tone(None, keys[0], keys[1]),
        1.0,
        "no grammar, no quieting"
    );
    canvas.set_graph_grammar(true);
    let frame = canvas.grammar_frame().unwrap();
    assert_eq!(link_tone(Some(&frame), keys[0], keys[1]), LINK_QUIET);
    canvas.select_member(id(&canvas, keys[0]));
    let frame = canvas.grammar_frame().unwrap();
    assert_eq!(link_tone(Some(&frame), keys[0], keys[1]), 1.0);
    assert_eq!(link_tone(Some(&frame), keys[3], keys[4]), LINK_QUIET);
}

#[test]
fn suggestions_are_off_until_the_person_turns_them_on() {
    use scenograph::suggestion::{SuggestionMark, SuggestionSettings};
    let (mut canvas, keys) = grammar_canvas();
    canvas.set_graph_grammar(true);
    let offer: Vec<_> = keys[1..]
        .iter()
        .map(|key| (id(&canvas, *key), "web".to_string()))
        .collect();
    canvas.offer_suggestions(offer);
    assert!(
        canvas
            .members_with_attention(Attention::Suggestion)
            .is_empty(),
        "off by default"
    );

    canvas.set_suggestion_settings(SuggestionSettings {
        on: true,
        at_once: 2,
        mark: SuggestionMark::Ring,
        ..SuggestionSettings::default()
    });
    let shown = canvas.members_with_attention(Attention::Suggestion);
    assert_eq!(
        shown,
        [id(&canvas, keys[1]), id(&canvas, keys[2])],
        "the first two, in order"
    );
    canvas.frame(800, 600);
    assert!(class_of(&canvas, keys[1]).contains("gnode-mark-ring"));

    canvas.set_suggestion_settings(SuggestionSettings {
        on: true,
        sources: vec!["mesh".into()],
        ..SuggestionSettings::default()
    });
    assert!(
        canvas
            .members_with_attention(Attention::Suggestion)
            .is_empty(),
        "no allowed source"
    );
}

#[test]
fn sites_and_territory_take_the_themes_category_colours() {
    use scenograph::suggestion::{SuggestionMark, SuggestionSettings};
    let (mut canvas, keys) = grammar_canvas();
    canvas.set_graph_grammar(true);
    let style_of = |canvas: &Canvas, key| {
        canvas
            .node_document
            .dom()
            .attribute(
                canvas.gnode_of[&key],
                &Namespace::from(""),
                &LocalName::from("style"),
            )
            .unwrap()
            .to_owned()
    };
    canvas.frame(800, 600);
    assert!(style_of(&canvas, keys[1]).contains("background-color: rgb("));

    let theme = [
        [10, 20, 30],
        [40, 50, 60],
        [70, 80, 90],
        [1, 2, 3],
        [4, 5, 6],
    ];
    canvas.set_category_colors(theme);
    assert_eq!(canvas.category_count(), 5);
    canvas.frame(800, 600);
    let site = style_of(&canvas, keys[1]);
    let category: usize = class_of(&canvas, keys[1])
        .split(' ')
        .nth(1)
        .unwrap()
        .trim_start_matches("gnode-cat-")
        .parse()
        .unwrap();
    let [r, g, b] = theme[category];
    assert!(
        site.contains(&format!("background-color: rgb({r}, {g}, {b});")),
        "{site}"
    );

    canvas.select_member(id(&canvas, keys[0]));
    canvas.frame(800, 600);
    assert!(
        !style_of(&canvas, keys[0]).contains("background-color"),
        "a body keeps its own fill"
    );

    canvas.set_suggestion_settings(SuggestionSettings {
        on: true,
        mark: SuggestionMark::Ring,
        ..SuggestionSettings::default()
    });
    canvas.set_suggested_members([id(&canvas, keys[4])]);
    canvas.frame(800, 600);
    assert!(
        !style_of(&canvas, keys[4]).contains("background-color"),
        "a ring is bare"
    );

    canvas.set_territory_field(true);
    let colours: Vec<(u8, u8, u8)> = canvas
        .territory_commands()
        .iter()
        .filter_map(|command| match command {
            PaintCmd::DrawRect(rect) => Some((
                (rect.color.r * 255.0).round() as u8,
                (rect.color.g * 255.0).round() as u8,
                (rect.color.b * 255.0).round() as u8,
            )),
            _ => None,
        })
        .collect();
    assert!(!colours.is_empty());
    assert!(
        colours
            .iter()
            .all(|c| theme.iter().any(|[r, g, b]| *c == (*r, *g, *b))),
        "the territory paints the theme's colours"
    );
    canvas.set_category_colors([]);
    assert_eq!(
        canvas.category_count(),
        8,
        "empty restores the canvas's own"
    );
}
