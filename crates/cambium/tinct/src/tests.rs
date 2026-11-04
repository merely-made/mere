// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn oklch_roundtrips_within_a_couple_steps() {
    for c in [
        Srgb::rgb(0x33, 0x66, 0xC8),
        Srgb::rgb(0x12, 0x12, 0x16),
        Srgb::rgb(0xEC, 0xEC, 0xF0),
        Srgb::rgb(0xDA, 0x5E, 0x3A),
    ] {
        let back = oklch::Oklch::from_srgb(c).to_srgb();
        assert!(
            (c.r as i32 - back.r as i32).abs() <= 2,
            "r {c:?} -> {back:?}"
        );
        assert!(
            (c.g as i32 - back.g as i32).abs() <= 2,
            "g {c:?} -> {back:?}"
        );
        assert!(
            (c.b as i32 - back.b as i32).abs() <= 2,
            "b {c:?} -> {back:?}"
        );
    }
}

#[test]
fn best_on_picks_readable() {
    let dark = Srgb::rgb(0x2A, 0x55, 0xB4);
    let light = Srgb::rgb(0xE0, 0xA8, 0x46);
    assert!(contrast(best_on(dark), dark) >= 3.0);
    assert!(contrast(best_on(light), light) >= 3.0);
}

fn slate_dark() -> Seeds {
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
    }
}

#[test]
fn derived_dark_has_ordered_ladder_and_readable_text() {
    let p = derive_palette(&slate_dark());
    assert!(relative_luminance(p.bg) < relative_luminance(p.surface));
    assert!(relative_luminance(p.surface) < relative_luminance(p.surface_2));
    assert!(contrast(p.text, p.surface) >= 4.5, "text on surface");
    assert!(contrast(p.on_primary, p.primary) >= 3.0, "on_primary");
}

#[test]
fn derived_light_inverts_and_stays_readable() {
    let p = derive_palette(&Seeds {
        neutral: Srgb::rgb(0xDF, 0xE3, 0xEE),
        dark: false,
        ..slate_dark()
    });
    assert!(relative_luminance(p.surface) > 0.6);
    assert!(relative_luminance(p.text) < 0.2);
    assert!(contrast(p.text, p.surface) >= 4.5, "text on surface");
}

#[test]
fn four_canonical_modes_derive_distinct_wider_hc_palettes() {
    // One seed set, four mode profiles (theme-modes plan T1): each derives
    // a distinct palette; both hc modes clear the WCAG 7:1 floor on body
    // text and are measurably wider than their normal-contrast counterpart.
    let seeds = slate_dark();
    let light = derive_palette_with(&seeds, ModeProfile::LIGHT);
    let dark = derive_palette_with(&seeds, ModeProfile::DARK);
    let hc_light = derive_palette_with(&seeds, ModeProfile::HC_LIGHT);
    let hc_dark = derive_palette_with(&seeds, ModeProfile::HC_DARK);

    // Distinct surfaces across all four.
    let surfaces = [
        light.surface,
        dark.surface,
        hc_light.surface,
        hc_dark.surface,
    ];
    for i in 0..surfaces.len() {
        for j in (i + 1)..surfaces.len() {
            assert_ne!(
                surfaces[i], surfaces[j],
                "modes {i} and {j} share a surface"
            );
        }
    }

    // Ladder direction: dark modes darker than light modes.
    assert!(relative_luminance(dark.surface) < relative_luminance(light.surface));
    assert!(relative_luminance(hc_dark.surface) < relative_luminance(hc_light.surface));

    // All modes readable; hc clears 7:1 and widens on its counterpart.
    for (label, p) in [("light", &light), ("dark", &dark)] {
        assert!(contrast(p.text, p.surface) >= 4.5, "{label} text/surface");
    }
    for (label, hc, normal) in [
        ("hc_light", &hc_light, &light),
        ("hc_dark", &hc_dark, &dark),
    ] {
        let hc_ratio = contrast(hc.text, hc.surface);
        let normal_ratio = contrast(normal.text, normal.surface);
        assert!(hc_ratio >= 7.0, "{label} text/surface {hc_ratio:.2} < 7.0");
        assert!(
            hc_ratio > normal_ratio,
            "{label} {hc_ratio:.2} not wider than normal {normal_ratio:.2}"
        );
        // Dim text stays proportionally closer to the body tier in hc.
        assert!(
            contrast(hc.text_dim, hc.surface) > contrast(normal.text_dim, normal.surface),
            "{label} dim tier"
        );
    }

    // The degenerate two-mode path still matches its profile equivalent.
    assert_eq!(derive_palette(&seeds), dark);
}

#[test]
fn text_body_override_cascades_to_dim_and_disabled() {
    let mut seeds = slate_dark();
    seeds.text_body = Some(Srgb::rgb(0xCC, 0xDD, 0xEE));
    let p = derive_palette(&seeds);
    assert_eq!(p.text, Srgb::rgb(0xCC, 0xDD, 0xEE));
    // dim / disabled blend the override toward the surface → between the two.
    let surf = relative_luminance(p.surface);
    let body = relative_luminance(p.text);
    let dim = relative_luminance(p.text_dim);
    assert!(
        dim > surf.min(body) && dim < surf.max(body),
        "dim between body+surface"
    );
}

#[test]
fn hex_roundtrips() {
    let c = Srgb::rgb(0x33, 0x66, 0xC8);
    assert_eq!(color_to_hex(c), "#3366C8");
    assert_eq!(color_from_hex("#3366c8"), Some(c));
    assert_eq!(color_from_hex("3366C8"), Some(c));
    assert_eq!(color_from_hex("nope"), None);
}

#[test]
fn hsl_roundtrips_within_a_step() {
    for c in [Srgb::rgb(0x33, 0x66, 0xC8), Srgb::rgb(0xE0, 0xA8, 0x46)] {
        let (h, s, l) = color_to_hsl(c);
        let back = color_from_hsl(h, s, l);
        assert!((c.r as i32 - back.r as i32).abs() <= 1);
        assert!((c.g as i32 - back.g as i32).abs() <= 1);
        assert!((c.b as i32 - back.b as i32).abs() <= 1);
    }
}

#[test]
fn oklch_lightness_step_is_monotonic() {
    let base = oklch::Oklch::from_srgb(Srgb::rgb(0x33, 0x66, 0xC8));
    let dark = base.with_l(0.20).to_srgb();
    let light = base.with_l(0.80).to_srgb();
    assert!(relative_luminance(dark) < relative_luminance(light));
}
