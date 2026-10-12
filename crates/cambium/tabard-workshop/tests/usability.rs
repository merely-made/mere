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

/// Cambium fields are `div[role=textbox]`, not native `<input>`/`<textarea>`,
/// so the sheet must reach every field by class. An empty stylesheet field
/// with no matching rule lays out at zero height.
#[test]
fn every_text_field_lays_out_at_its_authored_height() {
    let directory = tempfile::tempdir().unwrap();
    let mut host = mount(WorkshopState::load(&directory.path().join("themes.json")).unwrap());
    click(&mut host, &action("toggle-stylesheet"));
    let mut collapsed = Vec::new();
    for (field, min_height) in [
        ("name", 34.0),
        ("seed-hex", 34.0),
        ("mode-sheet", 200.0),
        ("font-family", 34.0),
        ("font-size", 34.0),
        ("font-line-height", 34.0),
    ] {
        let selector = Selector::role("textbox").with_attr("data-field", field);
        let node = host.with_dom(|dom| {
            let nodes = taproot::matching(dom, &selector);
            assert_eq!(nodes.len(), 1, "{field} must be one textbox");
            nodes[0]
        });
        let (_, _, width, height) = host.painted_rect(node).expect("field paints");
        if !(width > 0.0 && height >= min_height - 0.5) {
            collapsed.push(format!(
                "{field} at {width}x{height}, wants height {min_height}"
            ));
        }
    }
    assert!(
        collapsed.is_empty(),
        "fields lose their geometry: {collapsed:?}"
    );
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

#[test]
fn font_controls_are_retained_guard_invalid_input_and_save_reopen() {
    use tabard::FontRole;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("themes.json");
    let mut state = WorkshopState::load(&path).unwrap();
    state.set_installed_font_families(vec!["Georgia".into(), "Menlo".into(), "georgia".into()]);
    let mut host = mount(state);
    click(
        &mut host,
        &Selector::role("button").with_attr("data-font-role", "reading"),
    );
    let before = host.state().draft_theme().clone();
    let editor = editor_colors(&host);
    replace_text(&mut host, "font-size", "NaN");
    for selector in [
        action("save"),
        Selector::role("button").with_attr("data-font-role", "code"),
        Selector::role("button").with_attr("data-mode", "dark"),
    ] {
        click(&mut host, &selector);
        assert_eq!(host.state().draft_theme(), &before);
        assert_eq!(host.state().font_role(), FontRole::Reading);
        assert_eq!(host.state().text_field("font-size").unwrap().text(), "NaN");
        assert!(!path.exists());
    }
    assert!(host.with_dom(|dom| {
        !taproot::matching(
            dom,
            &Selector::role("textbox")
                .with_attr("data-field", "font-size")
                .with_attr("aria-invalid", "true"),
        )
        .is_empty()
    }));
    click(&mut host, &action("undo"));
    assert!(!host.state().has_pending_fields());
    replace_text(&mut host, "font-family", "Georgia");
    click(
        &mut host,
        &Selector::role("button").with_attr("data-installed-font", "Georgia"),
    );
    replace_text(&mut host, "font-size", "24");
    replace_text(&mut host, "font-line-height", "1.7");
    click(&mut host, &action("apply-fonts"));
    let authored = host.state().draft_theme().clone();
    let reading = &authored.typography.0[&FontRole::Reading];
    assert_eq!(reading.family.as_deref(), Some("Georgia"));
    assert_eq!(reading.size_px, Some(24.0));
    assert_eq!(reading.line_height, Some(1.7));
    assert_eq!(editor_colors(&host), editor);
    assert!(!path.exists());
    assert_eq!(
        host.state()
            .stylesheet_preview()
            .borrow()
            .computed_style("preview-paragraph", "font-size")
            .as_deref(),
        Some("24px")
    );
    click(&mut host, &action("undo"));
    assert_eq!(
        host.state().draft_theme().typography.0[&FontRole::Reading].size_px,
        None
    );
    click(&mut host, &action("redo"));
    assert_eq!(host.state().draft_theme(), &authored);
    click(&mut host, &action("save"));
    assert!(!host.state().is_dirty());
    let reopened = WorkshopState::load(&path).unwrap();
    assert_eq!(reopened.draft_theme(), &authored);
    click(&mut host, &action("reset-font-role"));
    assert!(host.state().draft_theme().typography.is_empty());
    click(&mut host, &action("discard"));
    assert_eq!(host.state().draft_theme(), &authored);
}

#[test]
fn applying_a_color_keeps_staged_font_input_until_its_own_action() {
    let mut host = mount(WorkshopState::in_memory());
    replace_text(&mut host, "font-size", "26");
    replace_text(&mut host, "seed-hex", "#123456");
    click(&mut host, &action("apply-hex"));
    assert_eq!(host.state().text_field("font-size").unwrap().text(), "26");
    assert!(host.state().draft_theme().typography.is_empty());
    click(&mut host, &action("apply-fonts"));
    assert_eq!(
        host.state().draft_theme().typography.0[&tabard::FontRole::Interface].size_px,
        Some(26.0)
    );
}

#[test]
fn generic_font_buttons_select_the_exact_family_despite_shared_name_substrings() {
    let mut host = mount(WorkshopState::in_memory());
    click(&mut host, &Selector::class("font-family-serif"));
    assert_eq!(
        host.state().text_field("font-family").unwrap().text(),
        "serif"
    );
    assert_eq!(
        host.state().draft_theme().typography.0[&tabard::FontRole::Interface]
            .family
            .as_deref(),
        Some("serif")
    );
    click(&mut host, &Selector::class("font-family-sans-serif"));
    assert_eq!(
        host.state().draft_theme().typography.0[&tabard::FontRole::Interface]
            .family
            .as_deref(),
        Some("sans-serif")
    );
}

#[test]
fn visible_chrome_and_syntax_font_roles_change_without_styling_the_editor() {
    use tabard::FontRole;
    let mut state = WorkshopState::in_memory();
    for (role, family, size) in [
        (FontRole::Interface, "serif", "18"),
        (FontRole::Heading, "serif", "26"),
        (FontRole::Label, "monospace", "13"),
        (FontRole::Code, "sans-serif", "19"),
    ] {
        state.set_font_role(role);
        *state.text_field_mut("font-family").unwrap() = cambium::TextInput::new(family);
        *state.text_field_mut("font-size").unwrap() = cambium::TextInput::new(size);
        assert!(state.apply_fonts());
    }
    let syntax_color = format!(
        "rgb({}, {}, {})",
        state.preview_syntax().surface.r,
        state.preview_syntax().surface.g,
        state.preview_syntax().surface.b
    );
    let host = mount(state);
    host.with_surfaces(|surfaces| {
        let surface = &surfaces[0];
        let plane = resolve_styles(
            surface.dom,
            &StyleSet::cambium(&[surface.sheet]),
            &Device::screen(surface.rect[2], surface.rect[3]),
            &InteractionStates::default(),
        );
        for (class, family, size) in [
            ("chrome-bar", "serif", "18px"),
            ("specimen-action", "serif", "18px"),
            ("specimen-title", "serif", "26px"),
            ("address-field", "monospace", "13px"),
            ("code-filename", "monospace", "13px"),
            ("syntax-code", "sans-serif", "19px"),
        ] {
            let nodes = taproot::matching(surface.dom, &Selector::class(class));
            assert_eq!(nodes.len(), 1, "{class}");
            let actual = plane.computed_style(nodes[0], "font-family").unwrap();
            assert_eq!(actual.split(',').next().unwrap().trim(), family, "{class}");
            assert_eq!(
                plane.computed_style(nodes[0], "font-size").as_deref(),
                Some(size),
                "{class}"
            );
        }
        let syntax = taproot::matching(surface.dom, &Selector::class("syntax-code"))[0];
        assert_eq!(
            plane.computed_style(syntax, "background-color").as_deref(),
            Some(syntax_color.as_str()),
            "font styles must preserve Illume's local palette"
        );
        let save = taproot::matching(surface.dom, &action("save"))[0];
        assert_eq!(
            plane.computed_style(save, "font-family").as_deref(),
            Some("sans-serif"),
            "authoring UI stays neutral"
        );
    });
}
