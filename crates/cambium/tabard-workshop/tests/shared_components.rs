// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The shipped workshop consumes the shared lexer, reader and graph surfaces.
//! These receipts verify retained DOM and real native input routing; they do
//! not stand in for headed pixels or the reader's scene/glyph tests.

use std::{rc::Rc, sync::Arc};

use cambium_genet_winit_host::Harness;
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::LayoutDom;
use tabard_workshop::{
    READER_LEAF_KEY, ReaderSpecimen, WORKSHOP_CODE_SAMPLE, WorkshopState, WorkshopView,
    workshop_stylesheet, workshop_view,
};
use taproot::Selector;
use tinct::{Srgb, SyntaxRole};
use winit::keyboard::NamedKey;

type Host = Harness<WorkshopState, fn(&WorkshopState) -> WorkshopView, WorkshopView>;

fn host(state: WorkshopState) -> Host {
    let mut host = Harness::new(
        workshop_stylesheet(),
        state,
        workshop_view as fn(&WorkshopState) -> WorkshopView,
    );
    host.layout_at(1280.0, 1000.0);
    host
}

fn attr<'a>(dom: &'a ScriptedDom, node: NodeId, name: &str) -> Option<&'a str> {
    dom.attributes(node)
        .find(|attr| attr.name.local.as_ref() == name)
        .map(|attr| attr.value)
}

fn descendants(dom: &ScriptedDom, root: NodeId) -> Vec<NodeId> {
    let mut nodes = vec![root];
    for child in dom.dom_children(root) {
        nodes.extend(descendants(dom, child));
    }
    nodes
}

fn text(dom: &ScriptedDom, root: NodeId) -> String {
    descendants(dom, root)
        .into_iter()
        .filter_map(|node| dom.text(node))
        .collect()
}

fn by_id(host: &Host, id: &str) -> NodeId {
    host.with_dom(|dom| {
        descendants(dom, dom.document())
            .into_iter()
            .find(|node| attr(dom, *node, "id") == Some(id))
            .unwrap_or_else(|| panic!("missing mounted {id}"))
    })
}

fn selector_node(host: &Host, selector: &Selector) -> NodeId {
    host.with_dom(|dom| {
        let nodes = taproot::matching(dom, selector);
        assert_eq!(nodes.len(), 1, "ambiguous target {selector:?}");
        nodes[0]
    })
}

fn click(host: &mut Host, selector: &Selector) {
    assert!(host.click_on(selector), "target must paint: {selector:?}");
}

fn mode(key: &str, label: &str) -> Selector {
    Selector::role("button")
        .with_attr("data-mode", key)
        .containing(label)
}

fn css_color(color: Srgb) -> String {
    format!("rgb({}, {}, {})", color.r, color.g, color.b)
}

fn declaration(style: &str, property: &str) -> Option<String> {
    style
        .split(';')
        .filter_map(|declaration| declaration.split_once(':'))
        .find(|(name, _)| name.trim() == property)
        .map(|(_, value)| value.trim().to_owned())
}

#[test]
fn workshop_code_is_actual_readonly_rust_with_illume_runs() {
    let mut host = host(WorkshopState::in_memory());
    let code = by_id(&host, "syntax-code");
    let subtree = host.with_dom(|dom| {
        assert_eq!(dom.element_name(code).unwrap().local.as_ref(), "pre");
        assert_eq!(attr(dom, code, "data-language"), Some("rust"));
        assert_eq!(text(dom, code), WORKSHOP_CODE_SAMPLE);
        for (class, lexeme) in [
            ("syntax-keyword", "fn"),
            ("syntax-keyword", "let"),
            ("syntax-comment", "// A little colour, everywhere"),
            ("syntax-string", "\"spring\""),
            ("syntax-number", "24"),
        ] {
            assert!(
                descendants(dom, code).into_iter().any(|node| {
                    attr(dom, node, "class")
                        .is_some_and(|classes| classes.split_whitespace().any(|item| item == class))
                        && text(dom, node) == lexeme
                }),
                "the shared Rust lexer must emit {class} for {lexeme:?}"
            );
        }
        let nodes = descendants(dom, code);
        for node in &nodes {
            assert_ne!(attr(dom, *node, "contenteditable"), Some("true"));
            assert_ne!(attr(dom, *node, "role"), Some("textbox"));
            assert!(attr(dom, *node, "tabindex").is_none_or(|index| index == "-1"));
            if let Some(name) = dom.element_name(*node) {
                assert!(!matches!(name.local.as_ref(), "input" | "textarea"));
            }
        }
        nodes
    });
    // Native traversal must never turn the visual code sample into an editor.
    for _ in 0..90 {
        host.tab(true);
        assert!(!host.focus().is_some_and(|node| subtree.contains(&node)));
    }
    assert_eq!(host.state().draft_theme().name, "Untitled theme");
    assert!(!host.state().has_changes());
}

#[test]
fn canonical_modes_retheme_shared_runs_without_rewriting_the_source() {
    let mut host = host(WorkshopState::in_memory());
    let theme = host.state().draft_theme().clone();
    let active = host.state().registry().active_theme();
    for (key, label) in [
        ("light", "Light"),
        ("dark", "Dark"),
        ("hc_light", "High contrast light"),
        ("hc_dark", "High contrast dark"),
    ] {
        click(&mut host, &mode(key, label));
        let palette = host.state().preview_syntax();
        let code = by_id(&host, "syntax-code");
        host.with_dom(|dom| {
            let style = attr(dom, code, "style").expect("local shared syntax palette");
            assert_eq!(
                declaration(style, "background-color"),
                Some(css_color(palette.surface))
            );
            assert_eq!(
                declaration(style, "color"),
                Some(css_color(palette.emphasis))
            );
            for role in SyntaxRole::ALL {
                assert_eq!(
                    declaration(style, &format!("--{}", cambium::role_class(role))),
                    Some(css_color(palette.role(role))),
                    "{key} {role:?} comes from the selected Tinct profile"
                );
            }
            assert_eq!(text(dom, code), WORKSHOP_CODE_SAMPLE);
        });
        let (_, _, width, height) = host.painted_rect(code).expect("code is laid out");
        assert!(width > 0.0 && height > 0.0);
        assert_eq!(host.state().draft_theme(), &theme);
        assert_eq!(host.state().registry().active_theme(), active);
        assert!(!host.state().has_changes());
    }
}

#[test]
fn graph_native_selection_is_preview_state_and_never_saves_a_theme() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("themes.json");
    let mut host = host(WorkshopState::load(path.clone()).unwrap());
    let theme = host.state().draft_theme().clone();
    let active = host.state().registry().active_theme();
    let dirty = host.state().is_dirty();
    assert_eq!(host.state().selected_graph_node(), Some(2));
    let notes = Selector::role("button").with_attr("data-key", "tabard-notes");
    let garden = Selector::role("button").with_attr("data-key", "tabard-garden");
    let paths = Selector::role("button").with_attr("data-key", "tabard-paths");
    for selector in [&notes, &garden, &paths] {
        let (_, _, width, height) = host
            .painted_rect(selector_node(&host, selector))
            .expect("shared canvas publishes native targets");
        assert!(width >= 44.0 && height >= 44.0);
    }
    click(&mut host, &notes);
    assert_eq!(host.state().selected_graph_node(), Some(1));
    host.tab(true);
    assert_eq!(host.focus(), Some(selector_node(&host, &garden)));
    host.key_named(NamedKey::Enter);
    assert_eq!(host.state().selected_graph_node(), Some(2));
    host.tab(true);
    assert_eq!(host.focus(), Some(selector_node(&host, &paths)));
    host.key_named(NamedKey::Space);
    assert_eq!(host.state().selected_graph_node(), Some(3));
    assert_eq!(host.state().draft_theme(), &theme);
    assert_eq!(host.state().registry().active_theme(), active);
    assert_eq!(host.state().is_dirty(), dirty);
    assert!(!host.state().has_changes());
    assert!(
        !path.exists(),
        "preview selection must not create a library"
    );
}

#[test]
fn reader_mount_keeps_the_extracted_source_across_modes_and_seed_edits() {
    let mut host = host(WorkshopState::in_memory());
    let reader = host.state().reader_preview();
    let source = reader.borrow().source_document();
    let name = reader.borrow().accessible_name().to_owned();
    assert!(name.contains("The leaves hold yesterday's weather"));
    assert!(!name.contains("Garden journal"));
    for (key, label) in [
        ("dark", "Dark"),
        ("hc_light", "High contrast light"),
        ("hc_dark", "High contrast dark"),
        ("light", "Light"),
    ] {
        click(&mut host, &mode(key, label));
        let current = host.state().reader_preview();
        assert!(Rc::ptr_eq(&reader, &current));
        assert!(Arc::ptr_eq(&source, &current.borrow().source_document()));
        assert_eq!(
            current.borrow().palette(),
            &ReaderSpecimen::palette_for_state(host.state())
        );
        let node = selector_node(
            &host,
            &Selector::role("img").with_attr("data-view-kind", "shared-reader"),
        );
        host.with_dom(|dom| {
            assert_eq!(
                dom.element_name(node).unwrap().local.as_ref(),
                "custom-leaf"
            );
            assert_eq!(
                attr(dom, node, "key"),
                Some(READER_LEAF_KEY.to_string().as_str())
            );
            assert_eq!(attr(dom, node, "aria-label"), Some(name.as_str()));
            assert!(attr(dom, node, "tabindex").is_none_or(|index| index == "-1"));
        });
        let (_, _, width, height) = host.painted_rect(node).expect("reader canvas is laid out");
        assert!(width > 0.0 && height > 0.0);
    }
    let before = host.state().draft_theme().seeds.primary;
    click(
        &mut host,
        &Selector::role("slider").containing("Primary hue"),
    );
    host.key_named(NamedKey::ArrowRight);
    assert_ne!(host.state().draft_theme().seeds.primary, before);
    let current = host.state().reader_preview();
    assert!(Rc::ptr_eq(&reader, &current));
    assert!(Arc::ptr_eq(&source, &current.borrow().source_document()));
    assert_eq!(current.borrow().accessible_name(), name);
    assert_eq!(
        current.borrow().palette(),
        &ReaderSpecimen::palette_for_state(host.state())
    );
}
