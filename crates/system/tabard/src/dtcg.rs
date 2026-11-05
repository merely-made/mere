// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl DtcgColorGroup {
    pub(super) fn from_theme(theme: &Theme, palette: Palette) -> Self {
        Self::from_palettes(
            theme,
            palette,
            theme.syntax_palette(),
            DtcgDerivation {
                crate_name: "tinct".to_owned(),
                function: "derive_palette".to_owned(),
                profile: "normal-contrast".to_owned(),
            },
        )
    }

    pub(super) fn from_palettes(
        theme: &Theme,
        palette: Palette,
        syntax: SyntaxPalette,
        derivation: DtcgDerivation,
    ) -> Self {
        let mut extensions = BTreeMap::new();
        extensions.insert(
            TABARD_EXTENSION_KEY.to_owned(),
            DtcgProvenance {
                theme: DtcgThemeSource {
                    name: theme.name.clone(),
                    seeds: theme.seeds,
                },
                derivation,
            },
        );

        Self {
            description: "Tinct-derived base palette authored by Tabard.".to_owned(),
            extensions,
            bg: palette.bg.into(),
            surface: palette.surface.into(),
            surface_2: palette.surface_2.into(),
            surface_hover: palette.surface_hover.into(),
            text_header: palette.text_header.into(),
            text: palette.text.into(),
            text_dim: palette.text_dim.into(),
            text_disabled: palette.text_disabled.into(),
            primary: palette.primary.into(),
            on_primary: palette.on_primary.into(),
            secondary: palette.secondary.into(),
            on_secondary: palette.on_secondary.into(),
            tertiary: palette.tertiary.into(),
            on_tertiary: palette.on_tertiary.into(),
            success: published(palette).success.into(),
            danger: published(palette).danger.into(),
            gated: DtcgGatedGroup::new(palette, &theme.categories),
            syntax: syntax.into(),
        }
    }
}
