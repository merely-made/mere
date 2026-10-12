// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use tabard::theme::registry::{Mode, ThemeRegistry};
use tabard::workshop::{Edit, ThemeDraft};
use tabard::{FontPreferences, FontRole, Theme, ThemePresentation, Typography};

fn theme() -> Theme {
    ThemeRegistry::default()
        .theme_def(tabard::theme::registry::THEME_ID_DEFAULT)
        .unwrap()
        .clone()
}
fn fonts() -> Typography {
    let mut fonts = Typography::default();
    fonts
        .set(
            FontRole::Reading,
            FontPreferences {
                family: Some("A \"named\"; family".into()),
                size_px: Some(21.5),
                line_height: Some(1.6),
            },
        )
        .unwrap();
    fonts
}
#[test]
fn absent_preferences_preserve_old_json_and_css() {
    let original = theme();
    let json = serde_json::to_string(&original).unwrap();
    assert!(!json.contains("typography"));
    let old: Theme = serde_json::from_str(&json).unwrap();
    assert_eq!(old, original);
    assert!(
        !old.css_custom_properties_for_mode(&Mode::Light)
            .unwrap()
            .contains("--tabard-font-")
    );
}
#[test]
fn requested_faces_are_escaped_and_retained_with_explicit_fallback() {
    let fonts = fonts();
    let css = fonts.css_custom_properties();
    assert!(css.contains("\\\"named\\\"; family"));
    assert!(css.contains(", serif;"));
    assert!(css.contains("--tabard-font-reading-size: 21.5px"));
    assert_eq!(fonts.missing_families(&[]).len(), 1);
    let json = serde_json::to_string(&fonts).unwrap();
    assert_eq!(serde_json::from_str::<Typography>(&json).unwrap(), fonts);
    assert!(
        !fonts
            .role_css(FontRole::Reading, ".reader")
            .contains("font-weight")
    );
    assert!(fonts.role_css(FontRole::Ui, ".app").is_empty());
}
#[test]
fn invalid_metrics_are_refused_before_library_or_export() {
    for value in [f32::NAN, f32::INFINITY, 0.0, 129.0] {
        let mut t = theme();
        t.typography.0.insert(
            FontRole::Ui,
            FontPreferences {
                size_px: Some(value),
                ..Default::default()
            },
        );
        assert!(t.palette_for_mode(&Mode::Dark).is_err());
        assert!(tabard::portable::theme_json(&t).is_err());
    }
    assert!(
        FontPreferences {
            family: Some("bad\nfamily".into()),
            ..Default::default()
        }
        .validate()
        .is_err()
    );
}
#[test]
fn typography_is_history_and_css_keeps_final_authority() {
    let mut registry = ThemeRegistry::default();
    let mut draft = ThemeDraft::fork(
        &registry,
        tabard::theme::registry::THEME_ID_DEFAULT,
        "user:fonts",
        "Fonts",
    )
    .unwrap();
    assert!(draft.edit(Edit::Typography(fonts())));
    assert!(draft.undo());
    assert!(draft.theme().typography.is_empty());
    assert!(draft.redo());
    draft.commit(&mut registry).unwrap();
    let mut saved = registry.theme_def("user:fonts").unwrap().clone();
    saved.mode_sheets.insert(
        "dark".into(),
        vec![".reader{font-family:serif;font-size:18px;}".into()],
    );
    let ThemePresentation::AuthoredStylesheet(rules) =
        saved.presentation_for_mode(&Mode::Dark).unwrap()
    else {
        panic!("authored")
    };
    assert!(rules[0].contains("--tabard-font-reading"));
    assert_eq!(rules.last().unwrap(), &saved.mode_sheets["dark"][0]);
    let fork = registry.fork("user:fonts", "user:copy", "Copy").unwrap();
    assert_eq!(fork.typography, saved.typography);
}

#[test]
fn portable_fonts_require_a_versioned_envelope_and_round_trip() {
    let mut t = theme();
    t.typography = fonts();
    let json = tabard::portable::theme_json(&t).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&json).unwrap()["version"],
        2
    );
    assert_eq!(tabard::portable::parse_theme_json(&json).unwrap(), t);
    assert!(tabard::portable::parse_theme_json(&serde_json::to_string(&t).unwrap()).is_err());
    assert!(
        serde_json::from_str::<Theme>(&json).is_err(),
        "old raw-theme hosts refuse the envelope"
    );
    for version in [0, 1, 3] {
        let value = serde_json::json!({"version":version,"theme":t});
        assert!(tabard::portable::parse_theme_json(&value.to_string()).is_err());
    }
}

#[test]
fn missing_family_notices_are_read_only_and_generic_families_need_no_catalogue_entry() {
    let mut fonts = Typography::default();
    fonts
        .set(
            FontRole::Code,
            FontPreferences {
                family: Some("My portable code face".into()),
                ..Default::default()
            },
        )
        .unwrap();
    fonts
        .set(
            FontRole::Ui,
            FontPreferences {
                family: Some("SYSTEM-UI".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let original = fonts.clone();
    let notices = fonts.missing_families(&[]);
    assert_eq!(notices.len(), 1);
    assert!(notices[0].contains("monospace is the fallback"));
    assert_eq!(fonts, original);
    assert!(
        fonts
            .missing_families(&["my PORTABLE code FACE".into()])
            .is_empty()
    );
}

#[test]
fn explicitly_empty_role_data_round_trips_without_changing_css() {
    let mut t = theme();
    t.typography
        .0
        .insert(FontRole::Ui, FontPreferences::default());
    assert!(t.typography.css_custom_properties().is_empty());
    let json = tabard::portable::theme_json(&t).unwrap();
    assert_eq!(tabard::portable::parse_theme_json(&json).unwrap(), t);
    t.typography
        .set(FontRole::Ui, FontPreferences::default())
        .unwrap();
    assert!(t.typography.is_empty());
}
