// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use genet_livery::{InteractionStates, StyleSet, resolve_styles};
use genet_static_dom::StaticDocument;
use layout_dom_api::LayoutDom;
use livery::{media::Device, values::Color};
use tabard::{
    DTCG_2025_10_SCHEMA, DtcgDocument, DtcgTokenType, LAGRANGE_PALETTE_LABELS,
    LAGRANGE_V1_21_1_IGNORED_LABELS, LagrangePaletteDiagnostic, LagrangePaletteMode,
    TABARD_EXTENSION_KEY, Theme,
};
use tinct::{Seeds, Srgb, SyntaxRole, color_to_hex, contrast};

fn theme() -> Theme {
    Theme::new(
        "test:ink",
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

#[test]
fn dtcg_document_preserves_tabard_provenance() {
    let theme = theme();
    let expected = theme.design_tokens();
    let json = theme.design_tokens_json().expect("DTCG JSON");
    let document: DtcgDocument = serde_json::from_str(&json).expect("typed DTCG JSON");

    assert_eq!(document.schema, DTCG_2025_10_SCHEMA);
    assert_eq!(document.color.bg.token_type, DtcgTokenType::Color);
    assert_eq!(document.color.bg.value.color_space, "srgb");
    assert_eq!(
        document.color.bg.value.hex,
        color_to_hex(theme.palette().bg)
    );
    for (actual, expected) in document
        .color
        .bg
        .value
        .components
        .iter()
        .zip(expected.color.bg.value.components)
    {
        assert!((actual - expected).abs() < 1e-12);
    }
    assert_eq!(document.color.extensions, expected.color.extensions);

    let provenance = document
        .color
        .extensions
        .get(TABARD_EXTENSION_KEY)
        .expect("Tabard provenance extension");
    assert_eq!(provenance.theme.name, "Ink");
    assert_eq!(provenance.theme.seeds, theme.seeds);
    assert_eq!(provenance.derivation.crate_name, "tinct");
    assert_eq!(provenance.derivation.function, "derive_palette");
    assert_eq!(provenance.derivation.profile, "normal-contrast");
}

#[test]
fn token_and_css_output_are_deterministic_and_map_every_base_role() {
    let theme = theme();
    assert_eq!(
        theme.design_tokens_json().expect("first JSON"),
        theme.design_tokens_json().expect("second JSON")
    );
    assert_eq!(theme.css_custom_properties(), theme.css_custom_properties());

    let css = theme.css_custom_properties();
    let palette = theme.palette();
    let expected = [
        ("bg", palette.bg),
        ("surface", palette.surface),
        ("surface-2", palette.surface_2),
        ("surface-hover", palette.surface_hover),
        ("text-header", palette.text_header),
        ("text", palette.text),
        ("text-dim", palette.text_dim),
        ("text-disabled", palette.text_disabled),
        ("primary", palette.primary),
        ("on-primary", palette.on_primary),
        ("secondary", palette.secondary),
        ("on-secondary", palette.on_secondary),
        ("tertiary", palette.tertiary),
        ("on-tertiary", palette.on_tertiary),
        ("success", palette.success),
        ("danger", palette.danger),
    ];
    for (name, color) in expected {
        assert_eq!(
            css_value(&css, name),
            color_to_hex(color),
            "CSS custom property for {name}"
        );
    }
}

#[test]
fn exported_palette_keeps_tinct_contrast_roles() {
    let palette = theme().palette();
    assert!(contrast(palette.text, palette.surface) >= 4.5);
    assert!(contrast(palette.on_primary, palette.primary) >= 3.0);
    assert!(contrast(palette.on_secondary, palette.secondary) >= 3.0);
    assert!(contrast(palette.on_tertiary, palette.tertiary) >= 3.0);
}

#[test]
fn livery_resolves_the_emitted_custom_properties() {
    let theme = theme();
    let stylesheet = format!(
        "{}\n.probe {{ color: var(--tabard-color-text); background-color: var(--tabard-color-surface); }}",
        theme.css_custom_properties()
    );
    let styles = StyleSet::cambium(&[&stylesheet]);
    assert!(
        styles.diagnostics().is_empty(),
        "{:?}",
        styles.diagnostics()
    );

    let document = StaticDocument::parse("<html><body><p class=\"probe\">Tabard</p></body></html>");
    let probe = document
        .first_with_class(document.document(), "probe")
        .expect("probe element");
    let plane = resolve_styles(
        &document,
        &styles,
        &Device::screen(320.0, 200.0),
        &InteractionStates::default(),
    );
    let computed = plane.get(probe).expect("computed probe style");
    let palette = theme.palette();

    assert_eq!(
        computed.color,
        color_to_hex(palette.text).parse::<Color>().unwrap()
    );
    assert_eq!(
        computed.background_color,
        color_to_hex(palette.surface).parse::<Color>().unwrap()
    );
}

#[test]
fn tokens_and_stylesheet_carry_every_syntax_role() {
    let theme = theme();
    let syntax = theme.syntax_palette();
    let document = theme.design_tokens();
    let css = theme.css_custom_properties();
    assert_eq!(document.color.syntax.roles.len(), SyntaxRole::ALL.len());
    for role in SyntaxRole::ALL {
        let expected = color_to_hex(syntax.role(role));
        let token = &document.color.syntax.roles[role.name()];
        assert_eq!(token.token_type, DtcgTokenType::Color);
        assert_eq!(token.value.hex, expected, "DTCG token for {}", role.name());
        let declaration = format!("--tabard-syntax-{}: {expected};", role.name());
        assert!(
            css.lines().any(|line| line.trim() == declaration),
            "{declaration}"
        );
    }
    let json = theme.design_tokens_json().expect("DTCG JSON");
    let back: DtcgDocument = serde_json::from_str(&json).expect("typed DTCG JSON");
    let hexes = |document: &DtcgDocument| {
        document
            .color
            .syntax
            .roles
            .iter()
            .map(|(name, token)| (name.clone(), token.value.hex.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(hexes(&back), hexes(&document));

    // Livery resolves a syntax property like any other.
    let stylesheet = format!("{css}\n.probe {{ color: var(--tabard-syntax-keyword); }}");
    let styles = StyleSet::cambium(&[&stylesheet]);
    assert!(
        styles.diagnostics().is_empty(),
        "{:?}",
        styles.diagnostics()
    );
    let page = StaticDocument::parse("<html><body><p class=\"probe\">fn</p></body></html>");
    let probe = page
        .first_with_class(page.document(), "probe")
        .expect("probe element");
    let plane = resolve_styles(
        &page,
        &styles,
        &Device::screen(320.0, 200.0),
        &InteractionStates::default(),
    );
    assert_eq!(
        plane.get(probe).expect("computed probe style").color,
        color_to_hex(syntax.role(SyntaxRole::Keyword))
            .parse::<Color>()
            .unwrap()
    );
}

#[test]
fn lagrange_palette_matches_golden_documented_shape() {
    let export = theme().lagrange_palette_txt();
    assert_eq!(
        export.text,
        include_str!("fixtures/lagrange_palette.txt"),
        "Lagrange palette output is a stable artifact"
    );

    let mut lines = export.text.lines();
    assert_eq!(lines.next(), Some("# Dark"));
    for label in LAGRANGE_PALETTE_LABELS {
        let line = lines.next().expect("dark palette line");
        assert_eq!(
            line.split_once(':').map(|(name, _)| name.trim()),
            Some(label)
        );
        parse_hex_color(line);
    }
    assert_eq!(lines.next(), Some(""));
    assert_eq!(lines.next(), Some("# Light"));
    for label in LAGRANGE_PALETTE_LABELS {
        let line = lines.next().expect("light palette line");
        assert_eq!(
            line.split_once(':').map(|(name, _)| name.trim()),
            Some(label)
        );
    }
    assert!(lines.next().is_none());
}

#[test]
fn lagrange_palette_reports_collapsed_and_unrepresented_roles_per_mode() {
    let diagnostics = theme().lagrange_palette_txt().diagnostics;
    assert!(
        diagnostics.contains(&LagrangePaletteDiagnostic::CollapsedRole {
            source_role: "primary",
            labels: vec!["brown", "orange"],
        })
    );
    assert!(
        diagnostics.contains(&LagrangePaletteDiagnostic::CollapsedRole {
            source_role: "secondary",
            labels: vec!["teal", "cyan"],
        })
    );
    for mode in [LagrangePaletteMode::Dark, LagrangePaletteMode::Light] {
        for label in LAGRANGE_V1_21_1_IGNORED_LABELS {
            assert!(
                diagnostics.contains(&LagrangePaletteDiagnostic::StockVersionIgnored {
                    mode,
                    label,
                    version: "v1.21.1",
                })
            );
        }
        for (label, value) in [
            ("yellow", "#FFFF20"),
            ("magenta", "#FF00FF"),
            ("blue", "#8484FF"),
        ] {
            assert!(
                diagnostics.contains(&LagrangePaletteDiagnostic::ReservedDefault {
                    mode,
                    label,
                    value,
                })
            );
        }
    }
    for role in ["text-header", "text-dim", "text-disabled"] {
        assert!(
            diagnostics.contains(&LagrangePaletteDiagnostic::UnrepresentedRole {
                mode: LagrangePaletteMode::Dark,
                source_role: role,
            })
        );
    }
    for role in ["bg", "text-header", "text-disabled"] {
        assert!(
            diagnostics.contains(&LagrangePaletteDiagnostic::UnrepresentedRole {
                mode: LagrangePaletteMode::Light,
                source_role: role,
            })
        );
    }
}

#[test]
fn lagrange_palette_orders_neutrals_and_accent_variants_for_adversarial_seeds() {
    let mut seeds = theme().seeds;
    seeds.text_header = Some(Srgb::rgb(0xF8, 0x10, 0x10));
    seeds.text_body = Some(Srgb::rgb(0xF0, 0xF0, 0xF0));
    let export = Theme::new("test:adversarial", "Adversarial", seeds).lagrange_palette_txt();

    for section in export.text.split("# ").skip(1) {
        let mut lines = section.lines().skip(1);
        let values = (0..5)
            .map(|_| parse_hex_color(lines.next().expect("neutral line")))
            .collect::<Vec<_>>();
        assert!(
            values
                .windows(2)
                .all(|pair| srgb_luma(pair[0]) <= srgb_luma(pair[1])),
            "neutral ramp is ordered in {section}"
        );

        let first_accent = parse_hex_color(lines.next().expect("brown line"));
        let second_accent = parse_hex_color(lines.next().expect("orange line"));
        assert!(srgb_luma(first_accent) <= srgb_luma(second_accent));
        let first_secondary = parse_hex_color(lines.next().expect("teal line"));
        let second_secondary = parse_hex_color(lines.next().expect("cyan line"));
        assert!(srgb_luma(first_secondary) <= srgb_luma(second_secondary));
    }
}

#[test]
fn lagrange_palette_falls_back_for_non_semantic_status_hues() {
    let mut seeds = theme().seeds;
    seeds.danger = Srgb::rgb(0x20, 0x60, 0xE0);
    seeds.success = Srgb::rgb(0xDC, 0x28, 0xC8);
    let export =
        Theme::new("test:status_fallback", "Status fallback", seeds).lagrange_palette_txt();

    assert!(export.text.contains("red:        #FF4040"));
    assert!(export.text.contains("green:      #00C800"));
    assert!(export.text.contains("green:      #009600"));
    for mode in [LagrangePaletteMode::Dark, LagrangePaletteMode::Light] {
        assert!(
            export
                .diagnostics
                .contains(&LagrangePaletteDiagnostic::ReservedDefault {
                    mode,
                    label: "red",
                    value: "#FF4040",
                })
        );
        let green = if mode == LagrangePaletteMode::Dark {
            "#00C800"
        } else {
            "#009600"
        };
        assert!(
            export
                .diagnostics
                .contains(&LagrangePaletteDiagnostic::ReservedDefault {
                    mode,
                    label: "green",
                    value: green,
                })
        );
    }
}

#[test]
fn lagrange_palette_reports_rgb_alpha_loss() {
    let mut seeds = theme().seeds;
    seeds.danger = Srgb::rgba(0xD5, 0x4E, 0x4E, 0x80);
    let export = Theme::new("test:alpha", "Alpha", seeds).lagrange_palette_txt();
    for mode in [LagrangePaletteMode::Dark, LagrangePaletteMode::Light] {
        assert!(
            export
                .diagnostics
                .contains(&LagrangePaletteDiagnostic::AlphaDiscarded {
                    mode,
                    label: "red",
                    source_role: "danger",
                })
        );
    }
}

fn parse_hex_color(line: &str) -> Srgb {
    let value = line.split_once(':').expect("palette separator").1.trim();
    assert_eq!(value.len(), 7);
    assert_eq!(&value[..1], "#");
    Srgb::rgb(
        u8::from_str_radix(&value[1..3], 16).expect("red channel"),
        u8::from_str_radix(&value[3..5], 16).expect("green channel"),
        u8::from_str_radix(&value[5..7], 16).expect("blue channel"),
    )
}

fn srgb_luma(color: Srgb) -> u32 {
    77 * u32::from(color.r) + 150 * u32::from(color.g) + 29 * u32::from(color.b)
}

fn css_value<'a>(stylesheet: &'a str, name: &str) -> &'a str {
    let declaration = format!("--tabard-color-{name}: ");
    stylesheet
        .lines()
        .find_map(|line| line.trim().strip_prefix(&declaration))
        .and_then(|value| value.strip_suffix(';'))
        .unwrap_or_else(|| panic!("custom property {name}"))
}
