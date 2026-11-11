// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use tincture::{ModeProfile, Palette, derive_palette_with};

use super::*;
use crate::canvas::palette::{DEFAULT_DERIVED_FACE_SEEDS, SELECTED};

/// Each mode's theme palette from the default seeds, with its name.
fn modes() -> [(&'static str, Palette); 4] {
    let seeds = &DEFAULT_DERIVED_FACE_SEEDS;
    [
        ("light", derive_palette_with(seeds, ModeProfile::LIGHT)),
        ("dark", derive_palette_with(seeds, ModeProfile::DARK)),
        (
            "high-contrast light",
            derive_palette_with(seeds, ModeProfile::HC_LIGHT),
        ),
        (
            "high-contrast dark",
            derive_palette_with(seeds, ModeProfile::HC_DARK),
        ),
    ]
}

/// The grounds a text face sits on in one mode: the page, a panel, a hovered
/// row, and a selected row or tab (the node selection fill).
fn grounds(theme: &Palette) -> [(&'static str, ThemeColor); 4] {
    let [r, g, b] = SELECTED.bg;
    [
        ("bg", theme.bg),
        ("surface", theme.surface),
        ("hover", theme.surface_hover),
        ("selection", ThemeColor::rgb(r, g, b)),
    ]
}

fn rgb([r, g, b, _]: [u8; 4]) -> ThemeColor {
    ThemeColor::rgb(r, g, b)
}

/// The angle between two hues in degrees, in `[0, 180]`.
fn hue_gap(a: f64, b: f64) -> f64 {
    let gap = (a - b).to_degrees().rem_euclid(360.0);
    gap.min(360.0 - gap)
}

#[test]
fn every_entry_holds_three_to_one_on_every_single_ground() {
    for (mode, theme) in modes() {
        let faces = DerivedFacePalette::from_palette(&theme);
        for (name, ground) in grounds(&theme) {
            let text = TextFacePalette::new(faces, &[ground]);
            assert_eq!(text.outlined(), [false; 8], "{mode} {name}");
            for (slot, color) in text.colors().into_iter().enumerate() {
                let ratio = contrast(rgb(color), ground);
                assert!(
                    ratio >= TEXT_FACE_CONTRAST,
                    "{mode} {name} slot {slot}: {ratio:.2}"
                );
            }
        }
    }
}

#[test]
fn clamping_keeps_hue_and_moves_lightness_only_as_far_as_needed() {
    for (mode, theme) in modes() {
        let faces = DerivedFacePalette::from_palette(&theme);
        for (name, ground) in grounds(&theme) {
            let text = TextFacePalette::new(faces, &[ground]);
            for (slot, (before, after)) in faces.colors().into_iter().zip(text.colors()).enumerate()
            {
                if contrast(rgb(before), ground) >= TEXT_FACE_CONTRAST {
                    assert_eq!(before, after, "{mode} {name} slot {slot} already held");
                    continue;
                }
                let (from, to) = (Oklch::from_srgb(rgb(before)), Oklch::from_srgb(rgb(after)));
                // Below this chroma 8-bit rounding alone swings the hue.
                if to.c >= 0.03 {
                    let gap = hue_gap(from.h, to.h);
                    assert!(gap <= 4.0, "{mode} {name} slot {slot}: hue moved {gap:.1}°");
                }
                // One lightness step nearer the original would fail the floor.
                let step = 1.0 / f64::from(STEPS);
                let toward = if to.l > from.l { -step } else { step };
                let nearer = at_lightness(from, to.l + toward);
                assert!(
                    contrast(nearer, ground) < TEXT_FACE_CONTRAST,
                    "{mode} {name} slot {slot} moved further than needed"
                );
            }
        }
    }
}

#[test]
fn a_rows_resting_and_hover_grounds_share_one_palette() {
    for (mode, theme) in modes() {
        let faces = DerivedFacePalette::from_palette(&theme);
        let row = [theme.bg, theme.surface, theme.surface_hover];
        let text = TextFacePalette::new(faces, &row);
        assert_eq!(text.outlined(), [false; 8], "{mode}");
        for (slot, color) in text.colors().into_iter().enumerate() {
            let ratio = worst(rgb(color), &row);
            assert!(
                ratio >= TEXT_FACE_CONTRAST,
                "{mode} slot {slot}: {ratio:.2}"
            );
        }
    }
}

#[test]
fn an_entry_no_lightness_can_save_is_outlined_not_hidden() {
    for (mode, theme) in modes() {
        let faces = DerivedFacePalette::from_palette(&theme);
        let all = grounds(&theme).map(|(_, ground)| ground);
        let text = TextFacePalette::new(faces, &all);
        for (slot, (color, outlined)) in text.colors().into_iter().zip(text.outlined()).enumerate()
        {
            let ratio = worst(rgb(color), &all);
            assert_eq!(
                outlined,
                ratio < TEXT_FACE_CONTRAST,
                "{mode} slot {slot}: {ratio:.2}"
            );
        }
    }
    // Black, mid grey and white leave no lightness at 3:1 against all three.
    let grounds = [0x00, 0x80, 0xFF].map(ThemeColor::gray);
    let text = TextFacePalette::new(DerivedFacePalette::default(), &grounds);
    assert!(text.outlined().iter().any(|&outlined| outlined));
}

#[test]
fn a_selected_row_on_a_dark_theme_takes_its_own_palette() {
    // Amber selection is light; a dark page needs light entries and amber
    // needs dark ones, so one palette cannot serve both. Hosts build one per
    // ground, as the plan says; on a light theme one palette happens to fit.
    for (mode, theme) in modes() {
        let faces = DerivedFacePalette::from_palette(&theme);
        let all = grounds(&theme).map(|(_, ground)| ground);
        let shared = TextFacePalette::new(faces, &all).outlined();
        let dark = mode.contains("dark");
        assert_eq!(shared, [dark; 8], "{mode}");
    }
}

#[test]
fn alpha_is_kept_and_no_grounds_changes_nothing() {
    let mut colors = DerivedFacePalette::default().colors();
    colors[3][3] = 0x80;
    let faces = DerivedFacePalette::new(colors);
    assert_eq!(TextFacePalette::new(faces, &[]).faces(), faces);
    let on_black = TextFacePalette::new(faces, &[ThemeColor::rgb(0, 0, 0)]);
    assert_eq!(on_black.colors()[3][3], 0x80);
}
