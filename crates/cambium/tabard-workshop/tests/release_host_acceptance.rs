// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Production native hooks, retained host input and accessibility on the
//! adopted render stack. These controls create no GPU, window or user library.
//! Windowless routing cannot supply a presented-frame or device-reset receipt.

#![cfg(all(feature = "native-host", not(target_arch = "wasm32")))]

use accesskit::Role;
use cambium_genet_winit_host::{CloseRequest, Harness, HostOptions, Modifiers};
use cambium_rootstock::A11yAction;
use genet_scripted_dom::NodeId;
use tabard_workshop::native_host::{WorkshopLogic, native_hooks, native_init};
use tabard_workshop::{WorkshopState, WorkshopView};
use taproot::Selector;

type WorkshopHarness = Harness<WorkshopState, WorkshopLogic, WorkshopView>;

fn host() -> WorkshopHarness {
    // The command queue is the one the production caption closures receive.
    // The actual native hooks register previews and synchronize controlled text.
    let mut host = Harness::with_command_init(
        |commands| native_init(WorkshopState::in_memory(), commands),
        native_hooks(|_| panic!("acceptance controls must not request an export chooser")),
        HostOptions::default(),
    );
    host.layout_at(1280.0, 1100.0);
    host.prepare_frame();
    host.relayout();
    host
}

fn node(host: &WorkshopHarness, selector: &Selector) -> NodeId {
    host.with_dom(|dom| {
        let nodes = taproot::matching(dom, selector);
        assert_eq!(nodes.len(), 1, "unique production control: {selector:?}");
        nodes[0]
    })
}

fn action(name: &str) -> Selector {
    Selector::role("button").with_attr("data-action", name)
}

fn name() -> Selector {
    Selector::role("textbox").with_attr("data-field", "name")
}

#[test]
fn release_native_text_selection_and_dirty_close_preserve_application_authority() {
    let mut host = host();
    let active = host.state().registry().active_theme();
    let original_id = host.state().draft_theme().id.clone();
    assert!(host.state().library_path().is_none());
    assert!(host.click_on(&name()));
    host.set_modifiers(Modifiers {
        ctrl: true,
        ..Modifiers::NONE
    });
    host.key_char("a");
    host.set_modifiers(Modifiers::NONE);
    host.key_injected("Release Ω draft");
    assert_eq!(host.state().draft_theme().name, "Release Ω draft");
    host.set_modifiers(Modifiers {
        ctrl: true,
        ..Modifiers::NONE
    });
    host.key_char("a");
    host.set_modifiers(Modifiers::NONE);
    let selection = host.state().text_field("name").unwrap().caret_selection();
    assert_eq!(
        host.state().text_field("name").unwrap().selected_text(),
        "Release Ω draft"
    );
    let draft = host.state().draft_theme().clone();
    let focus = host.focus();

    // A renderer-facing frame preparation, zoom and resize must not become
    // editor commands, replace the application draft, or apply a saved theme.
    for (width, height, zoom) in [(960.0, 900.0, 1.25), (1280.0, 1100.0, 1.0)] {
        host.set_ui_zoom(zoom);
        host.layout_at(width, height);
        host.prepare_frame();
        host.relayout();
        assert_eq!(host.state().draft_theme(), &draft);
        assert_eq!(
            host.state().text_field("name").unwrap().caret_selection(),
            selection
        );
        assert_eq!(host.focus(), focus);
        assert_eq!(host.state().registry().active_theme(), active);
        assert_eq!(host.state().draft_theme().id, original_id);
        assert!(host.state().has_changes());
        assert!(host.state().saved_choice().is_err());
    }
    host.request_close(CloseRequest::Native);
    assert!(!host.close_requested());
    assert!(host.state().close_requested());
    host.relayout();
    assert!(host.click_on(&action("cancel-close")));
    assert!(!host.close_requested());
    assert!(!host.state().close_requested());
    assert_eq!(host.state().draft_theme(), &draft);
    assert_eq!(host.state().registry().active_theme(), active);
    assert!(host.state().library_path().is_none());
}

#[test]
fn release_accessibility_focus_click_and_numeric_value_keep_separate_authority() {
    let mut host = host();
    let original = host.state().draft_theme().clone();
    let active = host.state().registry().active_theme();
    let mode = host.state().mode_key().to_owned();
    let dark = Selector::role("button")
        .with_attr("data-mode", "dark")
        .containing("Dark");
    let dark_node = node(&host, &dark);
    host.a11y_request(A11yAction::Focus, dark_node);
    assert_eq!(host.focus(), Some(dark_node));
    assert_eq!(host.state().mode_key(), mode);
    assert_eq!(host.state().draft_theme(), &original);
    host.a11y_request(A11yAction::Click, dark_node);
    assert_eq!(host.state().mode_key(), "dark");
    assert_eq!(host.state().registry().active_theme(), active);

    let hue = Selector::role("slider").containing("Primary hue");
    let hue_node = node(&host, &hue);
    host.a11y_set_value(hue_node, 0.37);
    let edited = host.state().draft_theme().clone();
    assert_ne!(edited.seeds.primary, original.seeds.primary);
    for refused in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        host.a11y_set_value(node(&host, &hue), refused);
        assert_eq!(
            host.state().draft_theme(),
            &edited,
            "nonfinite AT request mutated draft"
        );
    }
    host.a11y_request(A11yAction::Click, node(&host, &action("undo")));
    assert_eq!(host.state().draft_theme(), &original);
    assert_eq!(host.state().registry().active_theme(), active);
    assert_eq!(host.state().mode_key(), "dark");
}

#[test]
fn release_production_preview_semantics_follow_native_bindings_and_geometry() {
    let mut host = host();
    let active = host.state().registry().active_theme();
    for (width, height) in [(1280.0, 1100.0), (960.0, 900.0)] {
        host.layout_at(width, height);
        host.prepare_frame();
        host.relayout();
        let (tree, map) = host.a11y_tree();
        for snippet in ["The leaves hold yesterday's weather", "Every path begins"] {
            let (id, preview) = tree
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::Image
                        && node.label().is_some_and(|label| label.contains(snippet))
                })
                .unwrap_or_else(|| panic!("native producer preview missing: {snippet}"));
            let bounds = preview.bounds().expect("preview accessibility bounds");
            assert!(bounds.x1 > bounds.x0 && bounds.y1 > bounds.y0);
            let dom_node = map
                .get(id)
                .expect("producer semantics routes to its retained owner");
            let painted = host.painted_rect(*dom_node).expect("producer owner paints");
            assert_eq!(
                (
                    bounds.x0 as f32,
                    bounds.y0 as f32,
                    (bounds.x1 - bounds.x0) as f32,
                    (bounds.y1 - bounds.y0) as f32
                ),
                painted
            );
        }
        assert_eq!(host.state().registry().active_theme(), active);
        assert!(!host.state().has_changes());
    }
}
