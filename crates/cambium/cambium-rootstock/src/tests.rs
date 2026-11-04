// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn cambium_textbox_projection_reads_committed_value_and_preserves_semantics() {
    use genet_scripted_dom::ScriptedDom;
    use layout_dom_api::{LayoutDom as _, LayoutDomMut as _, LocalName, Namespace};

    let mut dom = ScriptedDom::new();
    let root = dom.document();
    dom.set_inner_html(
        root,
        "<div role=\"textbox\" aria-label=\"Draft\" aria-multiline=\"true\" data-cambium-text-value=\"Café 👩🏽‍🚀\nSecond line\" style=\"display:block;width:240px;height:40px\">Café 👩🏽‍🚀<span> ghost</span><span>preedit</span><span>│</span></div><div role=\"textbox\" aria-label=\"Title\" data-cambium-text-value=\"Résumé 🇫🇷\" style=\"display:block;width:240px;height:32px\">Résumé 🇫🇷</div><div role=\"button\" data-cambium-text-value=\"not a field\">Button</div>",
    );
    let node_for = |needle: &str| {
        let mut pending = vec![dom.document()];
        while let Some(node) = pending.pop() {
            if dom.attribute(node, &Namespace::default(), &LocalName::from("aria-label"))
                == Some(needle)
            {
                return node;
            }
            pending.extend(dom.dom_children(node));
        }
        panic!("missing fixture node named {needle}");
    };
    let draft_node = node_for("Draft");
    let focused = dom.opaque_id(draft_node);
    let title = dom.opaque_id(node_for("Title"));
    let drawn_children: Vec<_> = dom.dom_children(draft_node).collect();
    let hidden_ids: std::collections::HashSet<_> = drawn_children
        .iter()
        .flat_map(|child| {
            let mut pending = vec![*child];
            let mut ids = Vec::new();
            while let Some(node) = pending.pop() {
                ids.push(dom.opaque_id(node));
                pending.extend(dom.dom_children(node));
            }
            ids
        })
        .collect();
    let layout = OwnedLayout::new(&dom, &[""], 320.0, 180.0, &[], &Default::default());
    let projection = document_projection(&WindowDom::document(&dom), &layout, Some(focused));
    let field = projection
        .nodes()
        .iter()
        .find(|node| node.id.get() == focused)
        .expect("the focused textbox remains in the projection");
    assert_eq!(field.value.as_deref(), Some("Café 👩🏽‍🚀\nSecond line"));
    assert_eq!(field.name.as_deref(), Some("Draft"));
    assert_eq!(
        field.role,
        document_session_api::DocumentA11yRole::TextField
    );
    assert!(field.state.focused);
    assert!(field.children.is_empty(), "app textboxes are leaves");
    assert!(drawn_children.len() >= 4, "painting DOM remains intact");
    assert!(
        projection
            .nodes()
            .iter()
            .all(|node| !hidden_ids.contains(&node.id.get())),
        "painted descendants are absent from the accessibility projection"
    );
    assert!(field.state.editable);
    assert!(field.state.multiline);
    assert!(field.bounds.is_some());
    assert!(
        field
            .actions
            .contains(&document_session_api::DocumentA11yAction::Click)
    );
    assert!(
        field
            .actions
            .contains(&document_session_api::DocumentA11yAction::Focus)
    );

    let single_line = projection
        .nodes()
        .iter()
        .find(|node| node.id.get() == title)
        .expect("the single-line textbox remains in the projection");
    assert_eq!(single_line.value.as_deref(), Some("Résumé 🇫🇷"));
    assert!(single_line.state.editable);
    assert!(!single_line.state.multiline);

    let button = projection
        .nodes()
        .iter()
        .find(|node| node.role == document_session_api::DocumentA11yRole::Button)
        .expect("the button remains in the projection");
    assert_eq!(button.value, None, "the marker only applies to textboxes");
    assert_eq!(projection.root().get(), dom.opaque_id(dom.document()));
    assert_eq!(projection.revision(), 0);
    assert_eq!(projection.support().limitations().len(), 1);
}

#[test]
fn character_press_carries_its_text() {
    let press = KeyPress::character("h");
    assert_eq!(press.text.as_deref(), Some("h"));
    assert_eq!(press.key, Key::Character("h".into()));
}

#[test]
fn injected_text_only_for_unidentified_keys() {
    // The assistive-input path: unidentified key, real text, no chord.
    let injected = KeyPress {
        key: Key::Unidentified,
        text: Some("é".into()),
        modifiers: Modifiers::NONE,
        repeat: false,
    };
    assert_eq!(injected.injected_text(), Some("é"));

    // A named key never injects, even carrying text.
    let named = KeyPress {
        key: Key::Named(NamedKey::Enter),
        text: Some("\r".into()),
        modifiers: Modifiers::NONE,
        repeat: false,
    };
    assert_eq!(named.injected_text(), None);
}

#[test]
fn a_chord_is_a_command_not_typing() {
    let chord = KeyPress {
        key: Key::Unidentified,
        text: Some("s".into()),
        modifiers: Modifiers {
            ctrl: true,
            ..Modifiers::NONE
        },
        repeat: false,
    };
    assert_eq!(chord.injected_text(), None);
}

#[test]
fn alt_is_not_a_chord_because_altgr_types() {
    let altgr = KeyPress {
        key: Key::Unidentified,
        text: Some("@".into()),
        modifiers: Modifiers {
            alt: true,
            ..Modifiers::NONE
        },
        repeat: false,
    };
    assert_eq!(altgr.injected_text(), Some("@"));
}

#[test]
fn control_characters_are_not_typed() {
    let control = KeyPress {
        key: Key::Unidentified,
        text: Some("\u{7}".into()),
        modifiers: Modifiers::NONE,
        repeat: false,
    };
    assert_eq!(control.injected_text(), None);
}

#[test]
fn a_dead_key_never_types_even_carrying_text() {
    // Preserves the winit path's behaviour: `Key::Dead` produced no runner
    // key there and must not start typing here by falling into the
    // injected-text case.
    let dead = KeyPress {
        key: Key::Dead,
        text: Some("´".into()),
        modifiers: Modifiers::NONE,
        repeat: false,
    };
    assert_eq!(dead.injected_text(), None);
    assert!(dead.to_runner_key().is_none());
}

#[test]
fn injected_text_reaches_the_runner_as_a_character() {
    let injected = KeyPress {
        key: Key::Unidentified,
        text: Some("é".into()),
        modifiers: Modifiers::NONE,
        repeat: false,
    };
    let lowered = injected.to_runner_key().expect("should lower");
    assert!(matches!(lowered.key, cambium::Key::Character(ref c) if c == "é"));
}

#[test]
fn menu_entry_keys_lower_without_losing_identity_or_modifiers() {
    let f10 = KeyPress::named(NamedKey::F10)
        .to_runner_key()
        .expect("F10 reaches the runner");
    assert!(matches!(
        f10.key,
        cambium::Key::Named(cambium::NamedKey::F10)
    ));

    let alt = KeyPress::named(NamedKey::Alt)
        .with_modifiers(Modifiers {
            alt: true,
            ..Modifiers::NONE
        })
        .to_runner_key()
        .expect("Alt reaches the runner");
    assert!(matches!(
        alt.key,
        cambium::Key::Named(cambium::NamedKey::Alt)
    ));
    assert!(alt.mods.alt);
}

#[test]
fn an_unidentified_key_with_no_text_is_dropped() {
    let bare = KeyPress {
        key: Key::Unidentified,
        text: None,
        modifiers: Modifiers::NONE,
        repeat: false,
    };
    assert!(bare.to_runner_key().is_none());
}

#[test]
fn word_granularity_follows_the_caller() {
    let left = KeyPress::named(NamedKey::ArrowLeft);
    assert_eq!(
        left.caret_movement(false),
        Some(VisualMovement::PreviousCluster)
    );
    assert_eq!(
        left.caret_movement(true),
        Some(VisualMovement::PreviousWord)
    );
    assert_eq!(KeyPress::character("h").caret_movement(false), None);
}

#[test]
fn arrows_resolve_to_directions() {
    assert_eq!(
        KeyPress::named(NamedKey::ArrowLeft).direction(),
        Some(Direction::Left)
    );
    assert_eq!(KeyPress::named(NamedKey::Enter).direction(), None);
}
