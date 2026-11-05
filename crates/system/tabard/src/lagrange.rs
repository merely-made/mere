// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl LagrangePaletteMode {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
        }
    }

    pub(super) fn palette(self, theme: &Theme) -> Palette {
        let mut seeds = theme.seeds;
        seeds.dark = matches!(self, Self::Dark);
        derive_palette(&seeds)
    }

    pub(super) fn mapping(self, palette: Palette) -> Vec<LagrangePaletteEntry> {
        let mut neutral = match self {
            Self::Dark => vec![
                (LagrangeSourceRole::Bg, palette.bg),
                (LagrangeSourceRole::Surface, palette.surface),
                (LagrangeSourceRole::Surface2, palette.surface_2),
                (LagrangeSourceRole::SurfaceHover, palette.surface_hover),
                (LagrangeSourceRole::Text, palette.text),
            ],
            Self::Light => vec![
                (LagrangeSourceRole::Text, palette.text),
                (LagrangeSourceRole::TextDim, palette.text_dim),
                (LagrangeSourceRole::Surface2, palette.surface_2),
                (LagrangeSourceRole::SurfaceHover, palette.surface_hover),
                (LagrangeSourceRole::Surface, palette.surface),
            ],
        };
        neutral.sort_by_key(|(_, color)| srgb_luma(*color));

        let mut entries = neutral
            .into_iter()
            .zip(["black", "gray25", "gray50", "gray75", "white"])
            .map(|((role, value), label)| LagrangePaletteEntry {
                label,
                role: Some(role),
                value,
                default: None,
            })
            .collect::<Vec<_>>();

        let (brown, orange) = accent_pair(palette.primary, palette, self);
        let (teal, cyan) = accent_pair(palette.secondary, palette, self);
        entries.extend([
            LagrangePaletteEntry {
                label: "brown",
                role: Some(LagrangeSourceRole::Primary),
                value: brown,
                default: None,
            },
            LagrangePaletteEntry {
                label: "orange",
                role: Some(LagrangeSourceRole::Primary),
                value: orange,
                default: None,
            },
            LagrangePaletteEntry {
                label: "teal",
                role: Some(LagrangeSourceRole::Secondary),
                value: teal,
                default: None,
            },
            LagrangePaletteEntry {
                label: "cyan",
                role: Some(LagrangeSourceRole::Secondary),
                value: cyan,
                default: None,
            },
        ]);

        entries.extend([
            reserved_default("yellow", self),
            reserved_status_entry(
                "red",
                self,
                LagrangeSourceRole::Danger,
                palette.danger,
                is_redish,
            ),
            reserved_default("magenta", self),
            reserved_default("blue", self),
            reserved_status_entry(
                "green",
                self,
                LagrangeSourceRole::Success,
                palette.success,
                is_greenish,
            ),
        ]);

        entries
    }

    pub(super) fn diagnostics(
        self,
        mapping: &[LagrangePaletteEntry],
    ) -> Vec<LagrangePaletteDiagnostic> {
        let mut labels_by_role: BTreeMap<&'static str, Vec<&'static str>> = BTreeMap::new();
        for entry in mapping {
            if let Some(role) = entry.role {
                labels_by_role
                    .entry(role.name())
                    .or_default()
                    .push(entry.label);
            }
        }

        let mut diagnostics = labels_by_role
            .iter()
            .filter_map(|(role, labels)| {
                (labels.len() > 1).then(|| LagrangePaletteDiagnostic::CollapsedRole {
                    source_role: role,
                    labels: labels.clone(),
                })
            })
            .collect::<Vec<_>>();

        diagnostics.extend(mapping.iter().filter_map(|entry| {
            entry
                .default
                .map(|value| LagrangePaletteDiagnostic::ReservedDefault {
                    mode: self,
                    label: entry.label,
                    value,
                })
        }));

        diagnostics.extend(LAGRANGE_V1_21_1_IGNORED_LABELS.into_iter().map(|label| {
            LagrangePaletteDiagnostic::StockVersionIgnored {
                mode: self,
                label,
                version: "v1.21.1",
            }
        }));

        diagnostics.extend(mapping.iter().filter_map(|entry| {
            entry.role.filter(|_| entry.value.a != u8::MAX).map(|role| {
                LagrangePaletteDiagnostic::AlphaDiscarded {
                    mode: self,
                    label: entry.label,
                    source_role: role.name(),
                }
            })
        }));

        for role in ALL_PALETTE_ROLES {
            if !labels_by_role.contains_key(role.name()) {
                diagnostics.push(LagrangePaletteDiagnostic::UnrepresentedRole {
                    mode: self,
                    source_role: role.name(),
                });
            }
        }
        diagnostics
    }
}

impl LagrangePaletteExport {
    pub(super) fn from_theme(theme: &Theme) -> Self {
        let mut text = String::new();
        let mut diagnostics = Vec::new();

        for (index, mode) in [LagrangePaletteMode::Dark, LagrangePaletteMode::Light]
            .into_iter()
            .enumerate()
        {
            if index != 0 {
                text.push('\n');
            }
            text.push_str("# ");
            text.push_str(mode.name());
            text.push('\n');

            let palette = mode.palette(theme);
            let mapping = mode.mapping(palette);
            for entry in &mapping {
                text.push_str(&format!(
                    "{:<12}{}\n",
                    format!("{}:", entry.label),
                    color_to_hex(entry.value)
                ));
            }
            diagnostics.extend(mode.diagnostics(&mapping));
        }

        Self { text, diagnostics }
    }
}

pub(super) fn srgb_luma(color: Srgb) -> u32 {
    77 * u32::from(color.r) + 150 * u32::from(color.g) + 29 * u32::from(color.b)
}

pub(super) fn blend(color: Srgb, toward: Srgb, percent: u16) -> Srgb {
    let channel = |from: u8, to: u8| {
        ((u16::from(from) * (100 - percent) + u16::from(to) * percent + 50) / 100) as u8
    };
    Srgb::rgb(
        channel(color.r, toward.r),
        channel(color.g, toward.g),
        channel(color.b, toward.b),
    )
}

pub(super) fn accent_pair(
    color: Srgb,
    palette: Palette,
    mode: LagrangePaletteMode,
) -> (Srgb, Srgb) {
    let dim_base = match mode {
        LagrangePaletteMode::Dark => palette.bg,
        LagrangePaletteMode::Light => palette.text,
    };
    let bright_base = match mode {
        LagrangePaletteMode::Dark => palette.text,
        LagrangePaletteMode::Light => palette.surface,
    };
    let first = blend(color, dim_base, 40);
    let second = blend(color, bright_base, 40);
    if srgb_luma(first) <= srgb_luma(second) {
        (first, second)
    } else {
        (second, first)
    }
}

pub(super) fn is_redish(color: Srgb) -> bool {
    u16::from(color.r) >= u16::from(color.g) + 24 && u16::from(color.r) >= u16::from(color.b) + 24
}

pub(super) fn is_greenish(color: Srgb) -> bool {
    u16::from(color.g) >= u16::from(color.r) + 24 && u16::from(color.g) >= u16::from(color.b) + 12
}

pub(super) fn reserved_default(
    label: &'static str,
    mode: LagrangePaletteMode,
) -> LagrangePaletteEntry {
    let value = match (mode, label) {
        (_, "yellow") => Srgb::rgb(255, 255, 32),
        (_, "magenta") => Srgb::rgb(255, 0, 255),
        (_, "blue") => Srgb::rgb(132, 132, 255),
        (LagrangePaletteMode::Dark, "red") => Srgb::rgb(255, 64, 64),
        (LagrangePaletteMode::Light, "red") => Srgb::rgb(255, 64, 64),
        (LagrangePaletteMode::Dark, "green") => Srgb::rgb(0, 200, 0),
        (LagrangePaletteMode::Light, "green") => Srgb::rgb(0, 150, 0),
        _ => unreachable!("unknown Lagrange reserved label"),
    };
    let hex = match (mode, label) {
        (_, "yellow") => "#FFFF20",
        (_, "magenta") => "#FF00FF",
        (_, "blue") => "#8484FF",
        (_, "red") => "#FF4040",
        (LagrangePaletteMode::Dark, "green") => "#00C800",
        (LagrangePaletteMode::Light, "green") => "#009600",
        _ => unreachable!("unknown Lagrange reserved label"),
    };
    LagrangePaletteEntry {
        label,
        role: None,
        value,
        default: Some(hex),
    }
}

pub(super) fn reserved_status_entry(
    label: &'static str,
    mode: LagrangePaletteMode,
    role: LagrangeSourceRole,
    value: Srgb,
    accepts: fn(Srgb) -> bool,
) -> LagrangePaletteEntry {
    if accepts(value) {
        LagrangePaletteEntry {
            label,
            role: Some(role),
            value,
            default: None,
        }
    } else {
        reserved_default(label, mode)
    }
}
