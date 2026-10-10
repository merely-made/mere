// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use tabard::theme::registry::Mode;
use tabard_workshop::{APPLICATION_SOURCE, StylesheetSpecimen, WorkshopState};

#[test]
fn derived_mode_variables_paint_the_actual_application_roles() {
    let state = WorkshopState::in_memory();
    let theme = state.draft_theme();
    for mode in [Mode::Light, Mode::Dark, Mode::HcLight, Mode::HcDark] {
        let appearance = theme.palette_for_mode(&mode).unwrap();
        let sheet = theme.css_custom_properties_for_mode(&mode).unwrap();
        let specimen = StylesheetSpecimen::new(&mode, &[sheet]);
        for (id, property, color) in [
            (
                "application-body",
                "background-color",
                appearance.palette.bg,
            ),
            (
                "preview-toolbar",
                "background-color",
                appearance.palette.surface_2,
            ),
            (
                "preview-button",
                "background-color",
                appearance.palette.primary,
            ),
            ("preview-button", "color", appearance.palette.on_primary),
            ("preview-heading", "color", appearance.palette.text_header),
            ("preview-keyword", "color", appearance.syntax.keyword),
            ("preview-string", "color", appearance.syntax.string),
            ("preview-number", "color", appearance.syntax.number),
        ] {
            assert_eq!(
                specimen.computed_style(id, property),
                Some(format!("rgb({}, {}, {})", color.r, color.g, color.b)),
                "{mode:?} {id} {property}"
            );
        }
        assert!(specimen.diagnostics().is_empty());
    }
}

#[test]
fn exact_author_css_resolves_in_its_own_document() {
    let rules = vec![":root { --paper: rgb(12, 34, 56); } body { background-color: var(--paper); color: rgb(210, 220, 230); } #preview-button { background-color: rgb(70, 80, 90); }".into()];
    let mut specimen = StylesheetSpecimen::new(&Mode::Light, &rules);
    assert_eq!(specimen.rules(), rules);
    assert_eq!(
        specimen
            .computed_style("application-body", "background-color")
            .as_deref(),
        Some("rgb(12, 34, 56)")
    );
    assert_eq!(
        specimen
            .computed_style("preview-paragraph", "color")
            .as_deref(),
        Some("rgb(210, 220, 230)")
    );
    assert_eq!(
        specimen
            .computed_style("preview-button", "background-color")
            .as_deref(),
        Some("rgb(70, 80, 90)")
    );
    // A different document retains the fixture's default cascade. Even
    // unrestricted body/button rules have no editor/neighbor document scope.
    let neighbor = StylesheetSpecimen::default();
    assert_ne!(
        specimen.computed_style("application-body", "background-color"),
        neighbor.computed_style("application-body", "background-color")
    );
    let scene = specimen.frame(420, 340);
    assert!(
        scene
            .ops
            .iter()
            .any(|op| matches!(op, netrender::SceneOp::GlyphRun(_)))
    );
    assert!(specimen.diagnostics().is_empty());
}

#[test]
fn media_prefers_color_scheme_follows_explicit_preview_mode() {
    let rules = vec!["#preview-button { color: rgb(1, 2, 3); } @media (prefers-color-scheme: dark) { #preview-button { color: rgb(200, 210, 220); } }".into()];
    let mut specimen = StylesheetSpecimen::new(&Mode::Light, &rules);
    assert_eq!(
        specimen
            .computed_style("preview-button", "color")
            .as_deref(),
        Some("rgb(1, 2, 3)")
    );
    for mode in [Mode::Dark, Mode::HcDark] {
        specimen.set_appearance(&mode, &rules);
        assert_eq!(
            specimen
                .computed_style("preview-button", "color")
                .as_deref(),
            Some("rgb(200, 210, 220)")
        );
    }
    specimen.set_appearance(&Mode::HcLight, &rules);
    assert_eq!(
        specimen
            .computed_style("preview-button", "color")
            .as_deref(),
        Some("rgb(1, 2, 3)")
    );
}

#[test]
fn invalid_rules_report_real_diagnostics_and_valid_rules_still_render() {
    let rules = vec!["?? { color: red; } #preview-paragraph { color: rgb(4, 5, 6); }".into()];
    let mut specimen = StylesheetSpecimen::new(&Mode::Light, &rules);
    assert!(!specimen.diagnostics().is_empty());
    assert_eq!(specimen.rules(), rules);
    assert_eq!(
        specimen
            .computed_style("preview-paragraph", "color")
            .as_deref(),
        Some("rgb(4, 5, 6)")
    );
    assert!(
        specimen
            .frame(420, 340)
            .ops
            .iter()
            .any(|op| matches!(op, netrender::SceneOp::GlyphRun(_)))
    );
}

#[test]
fn appearance_edits_leave_source_and_accessible_content_unchanged() {
    let mut specimen = StylesheetSpecimen::default();
    let revision = specimen.revision();
    let name = specimen.accessible_name().to_owned();
    assert!(name.contains("Every path begins with a small act of attention."));
    assert!(!specimen.set_appearance(&Mode::Light, &[]));
    assert_eq!(specimen.revision(), revision);
    assert!(specimen.set_appearance(&Mode::Dark, &["body { color: yellow; }".into()]));
    assert_eq!(specimen.source(), APPLICATION_SOURCE);
    assert_eq!(specimen.accessible_name(), name);
    assert_eq!(specimen.revision(), revision + 1);
}
