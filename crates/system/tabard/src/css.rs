// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) fn custom_properties(
    palette: Palette,
    syntax: SyntaxPalette,
    categories: &CategoryOverrides,
) -> String {
    let mut stylesheet = String::from(":root {\n");
    for role in color_roles(published(palette)) {
        stylesheet.push_str("  --tabard-color-");
        stylesheet.push_str(role.name);
        stylesheet.push_str(": ");
        stylesheet.push_str(&css_color(role.value));
        stylesheet.push_str(";\n");
    }
    for (name, value) in gated_roles(&palette, categories).named() {
        stylesheet.push_str("  --tabard-color-");
        stylesheet.push_str(name);
        stylesheet.push_str(": ");
        stylesheet.push_str(&css_color(value));
        stylesheet.push_str(";\n");
    }
    for role in SyntaxRole::ALL {
        stylesheet.push_str("  --tabard-syntax-");
        stylesheet.push_str(role.name());
        stylesheet.push_str(": ");
        stylesheet.push_str(&css_color(syntax.role(role)));
        stylesheet.push_str(";\n");
    }
    stylesheet.push_str("}\n");
    stylesheet
}

pub(super) fn color_roles(palette: Palette) -> [ColorRole; 16] {
    [
        ColorRole {
            name: "bg",
            value: palette.bg,
        },
        ColorRole {
            name: "surface",
            value: palette.surface,
        },
        ColorRole {
            name: "surface-2",
            value: palette.surface_2,
        },
        ColorRole {
            name: "surface-hover",
            value: palette.surface_hover,
        },
        ColorRole {
            name: "text-header",
            value: palette.text_header,
        },
        ColorRole {
            name: "text",
            value: palette.text,
        },
        ColorRole {
            name: "text-dim",
            value: palette.text_dim,
        },
        ColorRole {
            name: "text-disabled",
            value: palette.text_disabled,
        },
        ColorRole {
            name: "primary",
            value: palette.primary,
        },
        ColorRole {
            name: "on-primary",
            value: palette.on_primary,
        },
        ColorRole {
            name: "secondary",
            value: palette.secondary,
        },
        ColorRole {
            name: "on-secondary",
            value: palette.on_secondary,
        },
        ColorRole {
            name: "tertiary",
            value: palette.tertiary,
        },
        ColorRole {
            name: "on-tertiary",
            value: palette.on_tertiary,
        },
        ColorRole {
            name: "success",
            value: palette.success,
        },
        ColorRole {
            name: "danger",
            value: palette.danger,
        },
    ]
}

pub(super) fn css_color(color: Srgb) -> String {
    if color.a == u8::MAX {
        return color_to_hex(color);
    }

    format!(
        "rgba({}, {}, {}, {})",
        color.r,
        color.g,
        color.b,
        css_alpha(color.a)
    )
}

pub(super) fn css_alpha(alpha: u8) -> String {
    let mut value = format!("{:.6}", f64::from(alpha) / f64::from(u8::MAX));
    while value.ends_with('0') {
        value.pop();
    }
    if value.ends_with('.') {
        value.pop();
    }
    value
}
