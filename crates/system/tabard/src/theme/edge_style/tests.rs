// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn theme_tokens_satisfy_generic_family_contract() {
    let contract = ThemeContract {
        min_family_luminance_delta: 4.0,
        require_non_color_family_distinction: true,
        require_monochrome_preservation: true,
    };

    validate_theme_edge_tokens(&ThemeEdgeTokens::dark_theme(), &contract)
        .expect("dark theme family tokens should satisfy contract");
    validate_theme_edge_tokens(&ThemeEdgeTokens::high_contrast_theme(), &contract)
        .expect("high contrast family tokens should satisfy contract");
}

#[test]
fn generic_family_signatures_do_not_depend_on_colors() {
    let mut tokens = ThemeEdgeTokens::default_theme();
    let before = tokens.family_tokens[&EdgeStyleFamily::Semantic];
    tokens
        .family_tokens
        .get_mut(&EdgeStyleFamily::Semantic)
        .unwrap()
        .color = Srgb::gray(128);
    let after = tokens.family_tokens[&EdgeStyleFamily::Semantic];

    assert_eq!(before.pattern, after.pattern);
    assert_eq!(before.end_marker, after.end_marker);
}
