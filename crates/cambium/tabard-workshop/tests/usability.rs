// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Authoring acceptance at the mounted retained surface boundary.

use cambium_genet_winit_host::{Harness, KeyPress, Modifiers};
use genet_livery::{Device, InteractionStates, StyleSet, resolve_styles};
use tabard::{portable::theme_json, theme::registry::Mode};
use tabard_workshop::{WorkshopState, WorkshopView, workshop_stylesheet, workshop_view};
use taproot::Selector;

type Host = Harness<WorkshopState, fn(&WorkshopState) -> WorkshopView, WorkshopView>;

fn mount(state: WorkshopState) -> Host {
    let mut host = Harness::new(
        workshop_stylesheet(),
        state,
        workshop_view as fn(&WorkshopState) -> WorkshopView,
    );
    host.layout_at(1280.0, 960.0);
    host
}

fn action(name: &str) -> Selector {
    Selector::role("button").with_attr("data-action", name)
}

#[track_caller]
fn click(host: &mut Host, selector: &Selector) {
    assert!(
        host.click_on(selector),
        "mounted control must paint: {selector:?}"
    );
}

fn replace_text(host: &mut Host, field: &str, value: &str) {
    click(
        host,
        &Selector::role("textbox").with_attr("data-field", field),
    );
    host.press_key(&KeyPress::character("a").with_modifiers(Modifiers {
        ctrl: true,
        ..Modifiers::NONE
    }));
    host.press_key(&KeyPress::character(value));
    assert_eq!(host.state().text_field(field).unwrap().text(), value);
}

fn editor_colors(host: &Host) -> Vec<(Option<String>, Option<String>)> {
    // Use the retained host sheet and real DOM through the shared style
    // authority. An isolated specimen must never add CSS to this sheet.
    host.with_surfaces(|surfaces| {
        let surface = &surfaces[0];
        let styles = StyleSet::cambium(&[surface.sheet]);
        let plane = resolve_styles(
            surface.dom,
            &styles,
            &Device::screen(surface.rect[2], surface.rect[3]),
            &InteractionStates::default(),
        );
        [Selector::class("theme-editor"), action("save")]
            .into_iter()
            .map(|selector| {
                let nodes = taproot::matching(surface.dom, &selector);
                assert_eq!(nodes.len(), 1);
                (
                    plane.computed_style(nodes[0], "background-color"),
                    plane.computed_style(nodes[0], "color"),
                )
            })
            .collect()
    })
}

#[test]
fn incomplete_hex_blocks_save_and_navigation_and_undo_keeps_authored_alpha() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("themes.json");
    let mut state = WorkshopState::load(&path).unwrap();
    let mut fixture = state.draft_theme().clone();
    fixture.seeds.primary.a = 73;
    state.import_theme_json(&theme_json(&fixture).unwrap());
    let mut host = mount(state);
    let original = host.state().draft_theme().clone();
    let original_mode = host.state().mode().clone();

    replace_text(&mut host, "seed-hex", "#12");
    for selector in [
        action("save"),
        action("new-copy"),
        Selector::role("button").with_attr("data-seed", "secondary"),
        Selector::role("button").with_attr("data-mode", "hc_light"),
    ] {
        click(&mut host, &selector);
        assert_eq!(host.state().draft_theme(), &original);
        assert_eq!(host.state().text_field("seed-hex").unwrap().text(), "#12");
        assert_eq!(host.state().mode(), &original_mode);
        assert!(!path.exists());
        assert!(host.state().status().contains("Fix the seed color"));
    }
    assert!(host.with_dom(|dom| {
        !taproot::matching(
            dom,
            &Selector::role("textbox")
                .with_attr("data-field", "seed-hex")
                .with_attr("aria-invalid", "true"),
        )
        .is_empty()
    }));
    click(&mut host, &action("undo"));
    assert_eq!(host.state().draft_theme(), &original);
    assert!(!host.state().has_pending_fields());

    replace_text(&mut host, "seed-hex", "#123456");
    click(&mut host, &action("apply-hex"));
    let edited = host.state().draft_theme().clone();
    assert_eq!(edited.seeds.primary.to_array(), [0x12, 0x34, 0x56, 73]);
    click(&mut host, &action("undo"));
    assert_eq!(host.state().draft_theme(), &original);
    click(&mut host, &action("redo"));
    assert_eq!(host.state().draft_theme(), &edited);
}

#[test]
fn applied_css_is_isolated_and_save_reopen_keeps_exact_css_and_default_mode() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("themes.json");
    let mut host = mount(WorkshopState::load(&path).unwrap());
    click(
        &mut host,
        &Selector::role("button").with_attr("data-mode", "hc_dark"),
    );
    let before = editor_colors(&host);
    let css = "/* authored spacing retained */\nbody { background-color: rgb(12, 34, 56); color: rgb(210, 220, 230); }\nbutton { background-color: rgb(70, 80, 90); }";
    click(&mut host, &action("toggle-stylesheet"));
    replace_text(&mut host, "mode-sheet", css);
    assert!(
        host.state()
            .draft_theme()
            .mode_sheet(&Mode::HcDark)
            .is_none()
    );
    click(&mut host, &action("apply-stylesheet"));
    let preview = host.state().stylesheet_preview();
    assert_eq!(preview.borrow().rules(), &[css.to_owned()]);
    assert_eq!(
        preview
            .borrow()
            .computed_style("application-body", "background-color")
            .as_deref(),
        Some("rgb(12, 34, 56)")
    );
    assert_eq!(
        preview
            .borrow()
            .computed_style("preview-button", "background-color")
            .as_deref(),
        Some("rgb(70, 80, 90)")
    );
    assert_eq!(editor_colors(&host), before);
    click(&mut host, &action("default-mode"));
    assert!(host.state().draft_theme().seeds.dark);
    assert!(host.state().draft_theme().high_contrast);
    let authored = host.state().draft_theme().clone();
    click(&mut host, &action("save"));
    assert!(!host.state().is_dirty());
    assert!(path.is_file());

    let reopened = WorkshopState::load(&path).unwrap();
    assert_eq!(reopened.draft_theme(), &authored);
    assert_eq!(reopened.mode(), &Mode::HcDark);
    assert_eq!(
        reopened.draft_theme().mode_sheet(&Mode::HcDark),
        Some(&vec![css.to_owned()])
    );
    assert_eq!(
        reopened.stylesheet_preview().borrow().rules(),
        &[css.to_owned()]
    );

    click(&mut host, &action("clear-stylesheet"));
    assert!(
        host.state()
            .draft_theme()
            .mode_sheet(&Mode::HcDark)
            .is_none()
    );
    assert_ne!(
        host.state()
            .stylesheet_preview()
            .borrow()
            .computed_style("application-body", "background-color")
            .as_deref(),
        Some("rgb(12, 34, 56)")
    );
    click(&mut host, &action("undo"));
    assert_eq!(host.state().draft_theme(), &authored);
    assert_eq!(host.state().text_field("mode-sheet").unwrap().text(), css);
}
