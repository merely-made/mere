// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A theme's own category colours replace Tinct's derived ones, still gated.

use tabard::{CategoryOverrides, Theme};
use tinct::{Seeds, Srgb, color_to_hex, contrast};

fn seeds(dark: bool) -> Seeds {
    Seeds {
        primary: Srgb::rgb(0x6E, 0x17, 0x12),
        secondary: Srgb::rgb(0x29, 0x48, 0x6B),
        tertiary: Srgb::rgb(0xC7, 0x80, 0x2F),
        neutral: Srgb::rgb(0xF0, 0xEB, 0xDD),
        text_header: None,
        text_body: None,
        success: Srgb::rgb(0x4F, 0xB3, 0x6E),
        danger: Srgb::rgb(0xD5, 0x4E, 0x4E),
        dark,
    }
}

fn value(css: &str, name: &str) -> String {
    let key = format!("--tabard-color-{name}: ");
    let start = css.find(&key).expect("property present") + key.len();
    css[start..start + 7].to_owned()
}

#[test]
fn an_override_replaces_the_derived_category() {
    let mut theme = Theme::new("user:test", "Test", seeds(false));
    let derived = value(&theme.css_custom_properties(), "category-2");
    // A mid teal that already holds 3:1 on the light grounds.
    theme.categories =
        CategoryOverrides([None, Some(Srgb::rgb(0x1F, 0x6F, 0x6A)), None, None, None]);
    let css = theme.css_custom_properties();
    assert_eq!(value(&css, "category-2"), "#1F6F6A");
    assert_ne!(value(&css, "category-2"), derived);
    let tokens = serde_json::to_value(theme.design_tokens().color.gated).unwrap();
    assert_eq!(tokens["category-2"]["$value"]["hex"], "#1F6F6A");
}

#[test]
fn an_override_is_still_gated() {
    let mut theme = Theme::new("user:test", "Test", seeds(false));
    let pale = Srgb::rgb(0xF4, 0xE8, 0xC0);
    theme.categories = CategoryOverrides([Some(pale), None, None, None, None]);
    let palette = theme.palette();
    let shown =
        tinct::color_from_hex(&value(&theme.css_custom_properties(), "category-1")).unwrap();
    assert_ne!(color_to_hex(shown), color_to_hex(pale));
    for ground in [palette.bg, palette.surface] {
        assert!(contrast(shown, ground) >= 3.0);
    }
}

#[test]
fn empty_overrides_are_not_serialized() {
    let theme = Theme::new("user:test", "Test", seeds(true));
    let json = serde_json::to_string(&theme).unwrap();
    assert!(!json.contains("categories"));
}
