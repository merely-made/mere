// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use tabard::theme::registry::{Harmony, Mode};
use tabard::theme::seed::harmonized_seeds;
use tabard::{DtcgDocument, DtcgModeDocument, ModeExportError, TABARD_EXTENSION_KEY, Theme};
use tinct::{ModeProfile, Palette, Seeds, Srgb, SyntaxRole, color_to_hex, contrast};

fn theme() -> Theme {
    Theme::new(
        "user:ink",
        "Ink",
        Seeds {
            primary: Srgb::rgb(0x33, 0x66, 0xC8),
            secondary: Srgb::rgb(0x2E, 0x9D, 0xA6),
            tertiary: Srgb::rgb(0xE0, 0xA8, 0x46),
            neutral: Srgb::rgb(0x10, 0x14, 0x22),
            text_header: None,
            text_body: None,
            success: Srgb::rgb(0x4F, 0xB3, 0x6E),
            danger: Srgb::rgb(0xD5, 0x4E, 0x4E),
            dark: true,
        },
    )
}

fn base_roles(p: Palette) -> [(&'static str, Srgb); 16] {
    [
        ("bg", p.bg),
        ("surface", p.surface),
        ("surface-2", p.surface_2),
        ("surface-hover", p.surface_hover),
        ("text-header", p.text_header),
        ("text", p.text),
        ("text-dim", p.text_dim),
        ("text-disabled", p.text_disabled),
        ("primary", p.primary),
        ("on-primary", p.on_primary),
        ("secondary", p.secondary),
        ("on-secondary", p.on_secondary),
        ("tertiary", p.tertiary),
        ("on-tertiary", p.on_tertiary),
        ("success", p.success),
        ("danger", p.danger),
    ]
}

fn assert_documents_equivalent(actual: &DtcgDocument, expected: &DtcgDocument) {
    assert_eq!(actual.schema, expected.schema);
    assert_eq!(actual.color.description, expected.color.description);
    assert_eq!(actual.color.extensions, expected.color.extensions);
    assert_eq!(
        actual.color.syntax.description,
        expected.color.syntax.description
    );
    fn base_tokens(document: &DtcgDocument) -> [&tabard::DtcgColorToken; 16] {
        let color = &document.color;
        [
            &color.bg,
            &color.surface,
            &color.surface_2,
            &color.surface_hover,
            &color.text_header,
            &color.text,
            &color.text_dim,
            &color.text_disabled,
            &color.primary,
            &color.on_primary,
            &color.secondary,
            &color.on_secondary,
            &color.tertiary,
            &color.on_tertiary,
            &color.success,
            &color.danger,
        ]
    }
    for (actual, expected) in base_tokens(actual).into_iter().zip(base_tokens(expected)) {
        assert_tokens_equivalent(actual, expected);
    }
    assert_eq!(
        actual.color.syntax.roles.keys().collect::<Vec<_>>(),
        expected.color.syntax.roles.keys().collect::<Vec<_>>()
    );
    for (name, actual) in &actual.color.syntax.roles {
        assert_tokens_equivalent(actual, &expected.color.syntax.roles[name]);
    }
}

fn assert_tokens_equivalent(actual: &tabard::DtcgColorToken, expected: &tabard::DtcgColorToken) {
    assert_eq!(actual.token_type, expected.token_type);
    assert_eq!(actual.value.color_space, expected.value.color_space);
    assert_eq!(actual.value.hex, expected.value.hex);
    assert_eq!(actual.value.alpha, expected.value.alpha);
    // Normalized sRGB components can round-trip through this serde_json parser
    // one ULP apart. Their canonical 8-bit hex value remains exact above.
    for (actual, expected) in actual
        .value
        .components
        .iter()
        .zip(expected.value.components)
    {
        assert!(
            (actual - expected).abs() < 1e-12,
            "component {actual} differs from {expected}"
        );
    }
}

#[test]
fn canonical_artifacts_export_the_exact_harmonized_mode_and_all_roles() {
    let mut theme = theme();
    theme.harmony = Harmony::Locked {
        secondary_deg: 120.0,
        tertiary_deg: 240.0,
    };
    // Both authored preference flags differ from at least one tested profile.
    theme.high_contrast = true;
    let effective = harmonized_seeds(&theme);
    assert_ne!(effective.secondary, theme.seeds.secondary);
    assert_ne!(effective.tertiary, theme.seeds.tertiary);
    let original = theme.clone();
    let mut surfaces = Vec::new();
    for mode in [Mode::Light, Mode::Dark, Mode::HcLight, Mode::HcDark] {
        let profile = ModeProfile {
            dark: mode.dark(),
            high_contrast: mode.high_contrast(),
        };
        let expected = tinct::derive_palette_with(&effective, profile);
        let syntax = tinct::derive_syntax_palette_with(&effective, profile);
        let actual = theme
            .palette_for_mode(&mode)
            .expect("derived canonical mode");
        assert_eq!(actual.mode, mode);
        assert_eq!(actual.effective_seeds, effective);
        assert_eq!(actual.palette, expected);
        assert_eq!(actual.syntax, syntax);
        assert_eq!(actual.syntax.surface, actual.palette.surface);
        surfaces.push(actual.palette.surface);

        let css = theme.css_custom_properties_for_mode(&mode).unwrap();
        let tokens = theme.design_tokens_for_mode(&mode).unwrap();
        let colors = serde_json::to_value(&tokens.document.color).unwrap();
        for (name, color) in base_roles(expected) {
            let hex = color_to_hex(color);
            assert_eq!(colors[name]["$value"]["hex"], hex, "{mode:?} {name}");
            assert!(css.contains(&format!("--tabard-color-{name}: {hex};")));
        }
        let minimum = if mode.high_contrast() { 7.0 } else { 4.5 };
        if mode.high_contrast() {
            assert!(contrast(expected.text, expected.surface) >= minimum);
        }
        for role in SyntaxRole::ALL {
            let color = syntax.role(role);
            let hex = color_to_hex(color);
            assert_eq!(
                tokens.document.color.syntax.roles[role.name()].value.hex,
                hex
            );
            assert!(css.contains(&format!("--tabard-syntax-{}: {hex};", role.name())));
            assert!(
                contrast(color, syntax.surface) >= minimum,
                "{mode:?} {role:?}"
            );
        }
    }
    let distinct: std::collections::HashSet<_> = surfaces.into_iter().collect();
    assert_eq!(
        distinct.len(),
        4,
        "each explicit profile has its own surface"
    );
    assert_eq!(
        theme, original,
        "exports cannot mutate authored preferences"
    );
}

#[test]
fn selected_mode_provenance_roundtrips_the_authored_definition_and_effective_inputs() {
    let mut theme = theme();
    theme.harmony = Harmony::Locked {
        secondary_deg: 150.0,
        tertiary_deg: 210.0,
    };
    theme
        .mode_sheets
        .insert("dark".into(), vec!["body { color: red; }".into()]);
    let tokens = theme.design_tokens_for_mode(&Mode::HcLight).unwrap();
    let metadata = &tokens.extensions[TABARD_EXTENSION_KEY];
    assert_eq!(metadata.theme, theme);
    assert_eq!(metadata.mode, "hc_light");
    assert_eq!(metadata.effective_seeds, harmonized_seeds(&theme));
    assert_eq!(metadata.derivation.function, "derive_palette_with");
    assert_eq!(metadata.derivation.profile, "high-contrast");
    assert_eq!(
        tokens.document.color.extensions[TABARD_EXTENSION_KEY].derivation,
        metadata.derivation
    );
    assert!(
        tokens
            .document
            .color
            .syntax
            .description
            .contains("derive_syntax_palette_with")
    );
    let json = theme.design_tokens_json_for_mode(&Mode::HcLight).unwrap();
    let roundtrip: DtcgModeDocument = serde_json::from_str(&json).unwrap();
    assert_eq!(roundtrip.extensions, tokens.extensions);
    assert_documents_equivalent(&roundtrip.document, &tokens.document);
    let raw: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(raw["$extensions"][TABARD_EXTENSION_KEY]["mode"], "hc_light");
    assert!(
        raw.get("document").is_none(),
        "legacy color shape stays flattened"
    );
    let old_reader: DtcgDocument = serde_json::from_str(&json).unwrap();
    assert_documents_equivalent(&old_reader, &tokens.document);
    assert_eq!(
        json,
        theme.design_tokens_json_for_mode(&Mode::HcLight).unwrap()
    );
}

#[test]
fn authored_body_overrides_are_preserved_but_syntax_is_contrast_gated_for_selected_mode() {
    let mut theme = theme();
    theme.seeds.text_body = Some(Srgb::rgb(127, 127, 127));
    theme.seeds.text_header = Some(Srgb::rgb(120, 120, 120));
    let derived = theme.palette_for_mode(&Mode::HcLight).unwrap();
    assert_eq!(derived.palette.text, theme.seeds.text_body.unwrap());
    assert_eq!(
        derived.palette.text_header,
        theme.seeds.text_header.unwrap()
    );
    for role in SyntaxRole::ALL {
        assert!(
            contrast(derived.syntax.role(role), derived.syntax.surface) >= 7.0,
            "{role:?}"
        );
    }
    let metadata = &theme
        .design_tokens_for_mode(&Mode::HcLight)
        .unwrap()
        .extensions[TABARD_EXTENSION_KEY];
    assert_eq!(metadata.theme.seeds.text_body, theme.seeds.text_body);
}

#[test]
fn strict_exports_refuse_selected_overrides_and_custom_modes() {
    let mut theme = theme();
    theme
        .mode_sheets
        .insert("dark".into(), vec!["body { color: red; }".into()]);
    let expected = ModeExportError::StylesheetOverride("dark".into());
    assert_eq!(theme.palette_for_mode(&Mode::Dark).unwrap_err(), expected);
    assert_eq!(
        theme
            .css_custom_properties_for_mode(&Mode::Dark)
            .unwrap_err(),
        expected
    );
    assert_eq!(
        theme.design_tokens_for_mode(&Mode::Dark).unwrap_err(),
        expected
    );
    assert_eq!(
        theme.design_tokens_json_for_mode(&Mode::Dark).unwrap_err(),
        expected
    );
    assert!(theme.palette_for_mode(&Mode::Light).is_ok());
    theme.mode_sheets.insert("hc_dark".into(), Vec::new());
    assert!(theme.palette_for_mode(&Mode::HcDark).is_ok());
    for with_sheet in [false, true] {
        let custom = Mode::Custom("solar".into());
        if with_sheet {
            theme
                .mode_sheets
                .insert(custom.as_key(), vec!["body { color: blue; }".into()]);
        }
        let expected = ModeExportError::UnsupportedCustomMode("custom:solar".into());
        assert_eq!(theme.palette_for_mode(&custom).unwrap_err(), expected);
        assert_eq!(
            theme.css_custom_properties_for_mode(&custom).unwrap_err(),
            expected
        );
        assert_eq!(theme.design_tokens_for_mode(&custom).unwrap_err(), expected);
        assert_eq!(
            theme.design_tokens_json_for_mode(&custom).unwrap_err(),
            expected
        );
    }
}

#[test]
fn explicit_artifacts_reject_invalid_authored_definitions_before_exporting() {
    let mut blank_name = theme();
    blank_name.name = " \t".into();
    let mut blank_id = theme();
    blank_id.id = " \n".into();
    let mut nonfinite = theme();
    nonfinite.harmony = Harmony::Locked {
        secondary_deg: f32::NAN,
        tertiary_deg: 120.0,
    };
    let mut infinite = theme();
    infinite.harmony = Harmony::Locked {
        secondary_deg: 120.0,
        tertiary_deg: f32::INFINITY,
    };
    for theme in [blank_name, blank_id, nonfinite, infinite] {
        let error = theme.palette_for_mode(&Mode::Light).unwrap_err();
        assert!(
            matches!(error, ModeExportError::InvalidTheme(_)),
            "{error:?}"
        );
        assert_eq!(
            theme
                .css_custom_properties_for_mode(&Mode::Light)
                .unwrap_err(),
            error
        );
        assert_eq!(
            theme.design_tokens_for_mode(&Mode::Light).unwrap_err(),
            error
        );
        assert_eq!(
            theme.design_tokens_json_for_mode(&Mode::Light).unwrap_err(),
            error
        );
        // Legacy callers retain their existing permissive artifact behavior.
        assert!(theme.css_custom_properties().starts_with(":root {"));
    }
}

#[test]
fn no_argument_exports_keep_legacy_raw_seed_behavior_and_document_shape() {
    let mut theme = theme();
    let legacy_css = theme.css_custom_properties();
    let legacy_json = theme.design_tokens_json().unwrap();
    let legacy_lagrange = theme.lagrange_palette_txt();
    let legacy_palette = theme.palette();
    let legacy_syntax = theme.syntax_palette();
    theme.harmony = Harmony::Locked {
        secondary_deg: 120.0,
        tertiary_deg: 240.0,
    };
    theme.high_contrast = true;
    theme
        .mode_sheets
        .insert("dark".into(), vec!["body { color: red; }".into()]);
    assert_eq!(theme.css_custom_properties(), legacy_css);
    assert_eq!(theme.design_tokens_json().unwrap(), legacy_json);
    assert_eq!(theme.lagrange_palette_txt(), legacy_lagrange);
    assert_eq!(theme.palette(), legacy_palette);
    assert_eq!(theme.syntax_palette(), legacy_syntax);
    let raw: serde_json::Value = serde_json::from_str(&legacy_json).unwrap();
    assert!(raw.get("$extensions").is_none());
    assert_eq!(
        raw["color"]["$extensions"][TABARD_EXTENSION_KEY]["derivation"]["function"],
        "derive_palette"
    );
    assert_eq!(
        raw["color"]["syntax"]["$description"],
        "Tinct-derived syntax palette (derive_syntax_palette)."
    );
    assert_ne!(
        theme
            .css_custom_properties_for_mode(&Mode::HcLight)
            .unwrap(),
        legacy_css
    );
}
