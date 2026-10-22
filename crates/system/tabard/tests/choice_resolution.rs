// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use tabard::theme::choice::ThemeChoice;
use tabard::theme::registry::{Harmony, Mode, THEME_ID_DEFAULT, ThemeRegistry};
use tabard::{
    ModeExportError, Theme, ThemeChoiceDiagnostic, ThemePresentation, resolve_theme_choice,
};

fn authored() -> Theme {
    let registry = ThemeRegistry::default();
    let mut theme = Theme::new(
        "theme:authored",
        "Authored",
        registry.theme_def(THEME_ID_DEFAULT).unwrap().seeds,
    );
    theme.harmony = Harmony::Locked {
        secondary_deg: 60.0,
        tertiary_deg: -60.0,
    };
    theme
}

#[test]
fn canonical_choice_uses_exact_shared_mode_math_and_does_not_activate_registry() {
    let mut registry = ThemeRegistry::default();
    let theme = authored();
    registry.add_user_theme(theme.clone()).unwrap();
    let active = registry.active_theme();
    for mode in [Mode::Light, Mode::Dark, Mode::HcLight, Mode::HcDark] {
        let requested = ThemeChoice::new("  THEME:AUTHORED  ", Some(mode.clone()));
        let resolution = resolve_theme_choice(&registry, &requested).unwrap();
        assert_eq!(resolution.requested, requested);
        assert_eq!(
            resolution.resolved,
            ThemeChoice::new(&theme.id, Some(mode.clone()))
        );
        assert_eq!(resolution.theme, theme);
        assert_eq!(
            resolution.presentation,
            ThemePresentation::Derived(theme.palette_for_mode(&mode).unwrap())
        );
        assert!(resolution.diagnostics.is_empty());
        assert_eq!(registry.active_theme(), active);
    }
}

#[test]
fn authored_sheets_are_authority_for_canonical_and_custom_modes_without_claimed_palette() {
    let mut registry = ThemeRegistry::default();
    let mut theme = authored();
    let rules = vec![
        "/* opaque author CSS */\n:root { color: #010203; }".into(),
        "@media (min-width: 500px) { .panel { background: #fedcba; } }".into(),
    ];
    theme.mode_sheets.insert("hc_light".into(), rules.clone());
    theme
        .mode_sheets
        .insert("custom:concert".into(), rules.clone());
    registry.add_user_theme(theme.clone()).unwrap();
    for mode in [Mode::HcLight, Mode::Custom("concert".into())] {
        let choice = ThemeChoice::new(&theme.id, Some(mode.clone()));
        let resolved = resolve_theme_choice(&registry, &choice).unwrap();
        assert_eq!(
            resolved.presentation,
            ThemePresentation::AuthoredStylesheet(rules.clone())
        );
        assert_eq!(resolved.resolved, choice);
        assert_eq!(resolved.theme.mode_sheets, theme.mode_sheets);
        assert!(resolved.diagnostics.is_empty());
        assert_eq!(
            theme.presentation_for_mode(&mode).unwrap(),
            resolved.presentation
        );
        assert!(theme.palette_for_mode(&mode).is_err());
    }
}

#[test]
fn fallback_preserves_request_reports_both_missing_identity_and_mode_and_recovers() {
    let mut registry = ThemeRegistry::default();
    let requested = ThemeChoice::new("theme:later", Some(Mode::Custom("later".into())));
    let resolution = resolve_theme_choice(&registry, &requested).unwrap();
    assert_eq!(resolution.requested, requested);
    assert_eq!(resolution.diagnostics.len(), 2);
    assert!(
        matches!(&resolution.diagnostics[0], ThemeChoiceDiagnostic::MissingTheme { requested_id, resolved_id } if requested_id == "theme:later" && resolved_id == THEME_ID_DEFAULT)
    );
    assert!(
        matches!(&resolution.diagnostics[1], ThemeChoiceDiagnostic::MissingMode { requested_mode: Mode::Custom(key), .. } if key == "later")
    );
    let mut theme = authored();
    theme.id = "theme:later".into();
    theme.mode_sheets.insert(
        "custom:later".into(),
        vec![".app { color: #234567; }".into()],
    );
    registry.add_user_theme(theme).unwrap();
    let recovered = resolve_theme_choice(&registry, &requested).unwrap();
    assert_eq!(recovered.requested, requested);
    assert_eq!(recovered.resolved, requested);
    assert!(recovered.diagnostics.is_empty());
    assert!(matches!(
        recovered.presentation,
        ThemePresentation::AuthoredStylesheet(_)
    ));
}

#[test]
fn empty_sheets_derive_and_default_mode_respects_authored_contrast_flags() {
    let mut registry = ThemeRegistry::default();
    let mut theme = authored();
    theme.seeds.dark = false;
    theme.high_contrast = true;
    theme.mode_sheets.insert("hc_light".into(), vec![]);
    theme.mode_sheets.insert("custom:empty".into(), vec![]);
    registry.add_user_theme(theme.clone()).unwrap();
    let resolution = resolve_theme_choice(&registry, &ThemeChoice::new(&theme.id, None)).unwrap();
    assert_eq!(resolution.resolved.theme_mode, Some(Mode::HcLight));
    assert!(matches!(
        resolution.presentation,
        ThemePresentation::Derived(_)
    ));
    let missing = resolve_theme_choice(
        &registry,
        &ThemeChoice::new(&theme.id, Some(Mode::Custom("empty".into()))),
    )
    .unwrap();
    assert_eq!(missing.resolved.theme_mode, Some(Mode::HcLight));
    assert_eq!(missing.diagnostics.len(), 1);
    assert!(
        theme
            .presentation_for_mode(&Mode::Custom("empty".into()))
            .is_err()
    );
}

#[test]
fn invalid_definitions_do_not_gain_authority_via_css_and_inert_builtins_still_validate() {
    let mut theme = authored();
    theme.name = " ".into();
    theme
        .mode_sheets
        .insert("dark".into(), vec![".app { color: #ffffff; }".into()]);
    assert!(matches!(
        theme.presentation_for_mode(&Mode::Dark),
        Err(ModeExportError::InvalidTheme(_))
    ));
    let registry = ThemeRegistry::default();
    let builtin = registry.theme_def(THEME_ID_DEFAULT).unwrap();
    let json = tabard::portable::theme_json(builtin).unwrap();
    assert_eq!(tabard::portable::parse_theme_json(&json).unwrap(), *builtin);
    assert!(builtin.presentation_for_mode(&Mode::Dark).is_ok());
}

#[test]
fn font_requests_survive_theme_fallback_without_entering_the_definition() {
    let registry = ThemeRegistry::default();
    let mut choice = ThemeChoice::new("unavailable", Some(Mode::Dark));
    choice.fonts = serde_json::from_str(r#"{"reading":["Missing face","serif"]}"#).unwrap();
    let resolved = resolve_theme_choice(&registry, &choice).unwrap();
    assert_eq!(resolved.requested, choice);
    assert_eq!(resolved.resolved.fonts, choice.fonts);
    assert!(
        !serde_json::to_string(&resolved.theme)
            .unwrap()
            .contains("Missing face")
    );
}

#[test]
fn presentation_stylesheet_is_one_function_for_derived_and_authored() {
    let registry = ThemeRegistry::default();
    let derived = resolve_theme_choice(&registry, &ThemeChoice::new(THEME_ID_DEFAULT, Some(Mode::Light))).unwrap();
    let sheet = tabard::presentation_stylesheet(&derived.presentation, &Mode::Light, THEME_ID_DEFAULT, ".role{}");
    let ThemePresentation::Derived(palette) = &derived.presentation else { panic!() };
    assert_eq!(sheet, format!("{}\n.role{{}}\n", palette.css_custom_properties()));

    let authored = ThemePresentation::AuthoredStylesheet(vec![":root{--tabard-color-bg:#123456}".into()]);
    let light = tabard::presentation_palette(&authored, &Mode::Light, THEME_ID_DEFAULT);
    let dark = tabard::presentation_palette(&authored, &Mode::Dark, THEME_ID_DEFAULT);
    assert_ne!(light.palette.bg, dark.palette.bg, "the fallback follows the resolved mode");
    assert_eq!(
        tabard::presentation_palette(&authored, &Mode::Custom("garden".into()), THEME_ID_DEFAULT),
        dark,
        "a custom mode borrows the dark fallback"
    );
    let sheet = tabard::presentation_stylesheet(&authored, &Mode::Light, THEME_ID_DEFAULT, ".role{}");
    assert!(sheet.starts_with(&light.css_custom_properties()));
    assert!(sheet.ends_with(".role{}\n:root{--tabard-color-bg:#123456}"));
}
