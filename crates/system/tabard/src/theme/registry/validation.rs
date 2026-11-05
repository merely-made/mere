// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) fn validate_theme_tokens(tokens: &ThemeTokenSet) -> Result<(), String> {
    let minimum_ratio = if tokens.theme_id == THEME_ID_HIGH_CONTRAST {
        7.0
    } else {
        4.5
    };

    for (label, foreground, background) in [
        (
            "radial disabled text",
            tokens.radial_disabled_text,
            tokens.radial_command_disabled_fill,
        ),
        (
            "radial hub text",
            tokens.radial_hub_text,
            tokens.radial_hub_fill,
        ),
        (
            "hover label text",
            tokens.hover_label_text,
            tokens.hover_label_background,
        ),
        (
            "command notice",
            tokens.command_notice,
            tokens.hover_label_background,
        ),
    ] {
        let ratio = contrast_ratio(foreground, background);
        if ratio < minimum_ratio {
            return Err(format!(
                "{label} contrast {ratio:.2} below minimum {minimum_ratio:.2}"
            ));
        }
    }

    validate_theme_edge_tokens(&tokens.edge_tokens, &tokens.theme_contract)?;

    if matches!(
        tokens.accessibility.default_edge_mode,
        EdgeAccessibilityMode::Monochrome
    ) && !tokens.accessibility.supports_monochrome
    {
        return Err(
            "default edge mode cannot be monochrome when monochrome support is false".into(),
        );
    }

    Ok(())
}

pub(super) fn contrast_ratio(foreground: Srgb, background: Srgb) -> f32 {
    let mut l1 = relative_luminance(foreground);
    let mut l2 = relative_luminance(background);
    if l2 > l1 {
        std::mem::swap(&mut l1, &mut l2);
    }
    (l1 + 0.05) / (l2 + 0.05)
}

pub(super) fn relative_luminance(color: Srgb) -> f32 {
    0.2126 * to_linear_component(color.r)
        + 0.7152 * to_linear_component(color.g)
        + 0.0722 * to_linear_component(color.b)
}

pub(super) fn to_linear_component(component: u8) -> f32 {
    let value = component as f32 / 255.0;
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}
