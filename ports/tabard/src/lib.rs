// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Tabard owns portable theme artifacts: Tinct seeds in, typed design tokens
//! and a Livery stylesheet out.
//!
//! The first slice intentionally exposes only Tinct's base palette. It has no
//! host theme struct, icon policy, syntax palette, persistence, or Pelt
//! preview. Those consumers can share the artifact once it is real instead of
//! each inventing their own theme format.

#![doc(html_no_source)]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tinct::{Palette, Seeds, Srgb, color_to_hex, derive_palette};

/// DTCG's stable Design Tokens Format Module schema for this output.
pub const DTCG_2025_10_SCHEMA: &str = "https://www.designtokens.org/schemas/2025.10/format.json";

/// Reverse-DNS extension namespace for Tabard's source provenance.
pub const TABARD_EXTENSION_KEY: &str = "org.merely.tabard";

/// The labels and order documented by Lagrange's `palette.txt` help.
///
/// The first five entries are the documented neutral intensity ramp, followed
/// by two accent pairs and the five reserved status/link colors.
///
/// This is the documented vocabulary emitted by Tabard. It is not a promise
/// that every stock Lagrange loader accepts every documented label; see
/// [`LAGRANGE_V1_21_1_IGNORED_LABELS`].
pub const LAGRANGE_PALETTE_LABELS: [&str; 14] = [
    "black", "gray25", "gray50", "gray75", "white", "brown", "orange", "teal", "cyan", "yellow",
    "red", "magenta", "blue", "green",
];

/// Labels documented by Lagrange's v1.21.1 help but absent from that
/// version's `loadPalette_Color` label table. They remain in the emitted
/// compatibility artifact so the output preserves the documented shape.
pub const LAGRANGE_V1_21_1_IGNORED_LABELS: [&str; 2] = ["yellow", "magenta"];

/// An authored theme: the small Tinct seed set plus a human-facing name.
///
/// Tabard derives the normal-contrast palette selected by Seeds::dark.
/// High-contrast profiles, syntax palettes, and product-specific roles remain
/// separate follow-on work.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub seeds: Seeds,
}

impl Theme {
    /// Create a theme from its authored Tinct seed set.
    pub fn new(name: impl Into<String>, seeds: Seeds) -> Self {
        Self {
            name: name.into(),
            seeds,
        }
    }

    /// Derive the base palette owned by the current Tabard artifact.
    pub fn palette(&self) -> Palette {
        derive_palette(&self.seeds)
    }

    /// Emit the typed DTCG 2025.10 document for this theme.
    ///
    /// Every color token carries an explicit color type. The source seed set
    /// and the narrow derivation choice live under Tabard's reverse-DNS
    /// extension so a consumer can retain the provenance without mistaking it
    /// for an interchange requirement.
    pub fn design_tokens(&self) -> DtcgDocument {
        DtcgDocument {
            schema: DTCG_2025_10_SCHEMA.to_owned(),
            color: DtcgColorGroup::from_theme(self, self.palette()),
        }
    }

    /// Serialize the DTCG document as deterministic, pretty JSON.
    pub fn design_tokens_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.design_tokens())
    }

    /// Emit a Livery author stylesheet with the derived palette at :root.
    ///
    /// The property names intentionally mirror the owned Tinct base roles:
    /// --tabard-color-bg, --tabard-color-surface-2, and so on. A host may
    /// append ordinary author rules which use them through var().
    pub fn css_custom_properties(&self) -> String {
        let mut stylesheet = String::from(":root {\n");
        for role in color_roles(self.palette()) {
            stylesheet.push_str("  --tabard-color-");
            stylesheet.push_str(role.name);
            stylesheet.push_str(": ");
            stylesheet.push_str(&css_color(role.value));
            stylesheet.push_str(";\n");
        }
        stylesheet.push_str("}\n");
        stylesheet
    }

    /// Emit Lagrange's documented UI `palette.txt` artifact for both modes.
    ///
    /// Lagrange uses this file for application chrome and link icons. It does
    /// not control page color themes. The returned diagnostics make the
    /// intentional semantic loss visible: Tabard has more roles than this
    /// format and Lagrange's accent/status labels do not have one-to-one
    /// equivalents in the Tabard palette.
    pub fn lagrange_palette_txt(&self) -> LagrangePaletteExport {
        LagrangePaletteExport::from_theme(self)
    }
}

/// A generated Lagrange `palette.txt` plus explicit diagnostics about the
/// projection from Tabard's richer role set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LagrangePaletteExport {
    pub text: String,
    pub diagnostics: Vec<LagrangePaletteDiagnostic>,
}

impl LagrangePaletteExport {
    fn from_theme(theme: &Theme) -> Self {
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

/// The two palettes emitted for Lagrange's `palette.txt` file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LagrangePaletteMode {
    Dark,
    Light,
}

impl LagrangePaletteMode {
    fn name(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
        }
    }

    fn palette(self, theme: &Theme) -> Palette {
        let mut seeds = theme.seeds;
        seeds.dark = matches!(self, Self::Dark);
        derive_palette(&seeds)
    }

    fn mapping(self, palette: Palette) -> Vec<LagrangePaletteEntry> {
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

    fn diagnostics(self, mapping: &[LagrangePaletteEntry]) -> Vec<LagrangePaletteDiagnostic> {
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

/// A diagnostic describing where Lagrange's smaller role vocabulary loses
/// Tabard information.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LagrangePaletteDiagnostic {
    /// One Tabard role supplies several Lagrange labels.
    CollapsedRole {
        source_role: &'static str,
        labels: Vec<&'static str>,
    },
    /// A Tabard role has no Lagrange label in this mode's neutral mapping.
    UnrepresentedRole {
        mode: LagrangePaletteMode,
        source_role: &'static str,
    },
    /// A Lagrange reserved label uses its documented built-in value because
    /// Tabard has no corresponding status or protocol role.
    ReservedDefault {
        mode: LagrangePaletteMode,
        label: &'static str,
        value: &'static str,
    },
    /// Lagrange's documented RGB syntax cannot carry Tabard alpha.
    AlphaDiscarded {
        mode: LagrangePaletteMode,
        label: &'static str,
        source_role: &'static str,
    },
    /// The pinned stock loader omits a label that its help documents.
    StockVersionIgnored {
        mode: LagrangePaletteMode,
        label: &'static str,
        version: &'static str,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LagrangePaletteEntry {
    label: &'static str,
    role: Option<LagrangeSourceRole>,
    value: Srgb,
    default: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LagrangeSourceRole {
    Bg,
    Surface,
    Surface2,
    SurfaceHover,
    TextHeader,
    Text,
    TextDim,
    TextDisabled,
    Primary,
    OnPrimary,
    Secondary,
    OnSecondary,
    Tertiary,
    OnTertiary,
    Success,
    Danger,
}

impl LagrangeSourceRole {
    fn name(self) -> &'static str {
        match self {
            Self::Bg => "bg",
            Self::Surface => "surface",
            Self::Surface2 => "surface-2",
            Self::SurfaceHover => "surface-hover",
            Self::TextHeader => "text-header",
            Self::Text => "text",
            Self::TextDim => "text-dim",
            Self::TextDisabled => "text-disabled",
            Self::Primary => "primary",
            Self::OnPrimary => "on-primary",
            Self::Secondary => "secondary",
            Self::OnSecondary => "on-secondary",
            Self::Tertiary => "tertiary",
            Self::OnTertiary => "on-tertiary",
            Self::Success => "success",
            Self::Danger => "danger",
        }
    }
}

const ALL_PALETTE_ROLES: [LagrangeSourceRole; 16] = [
    LagrangeSourceRole::Bg,
    LagrangeSourceRole::Surface,
    LagrangeSourceRole::Surface2,
    LagrangeSourceRole::SurfaceHover,
    LagrangeSourceRole::TextHeader,
    LagrangeSourceRole::Text,
    LagrangeSourceRole::TextDim,
    LagrangeSourceRole::TextDisabled,
    LagrangeSourceRole::Primary,
    LagrangeSourceRole::OnPrimary,
    LagrangeSourceRole::Secondary,
    LagrangeSourceRole::OnSecondary,
    LagrangeSourceRole::Tertiary,
    LagrangeSourceRole::OnTertiary,
    LagrangeSourceRole::Success,
    LagrangeSourceRole::Danger,
];

fn srgb_luma(color: Srgb) -> u32 {
    77 * u32::from(color.r) + 150 * u32::from(color.g) + 29 * u32::from(color.b)
}

fn blend(color: Srgb, toward: Srgb, percent: u16) -> Srgb {
    let channel = |from: u8, to: u8| {
        ((u16::from(from) * (100 - percent) + u16::from(to) * percent + 50) / 100) as u8
    };
    Srgb::rgb(
        channel(color.r, toward.r),
        channel(color.g, toward.g),
        channel(color.b, toward.b),
    )
}

fn accent_pair(color: Srgb, palette: Palette, mode: LagrangePaletteMode) -> (Srgb, Srgb) {
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

fn is_redish(color: Srgb) -> bool {
    u16::from(color.r) >= u16::from(color.g) + 24 && u16::from(color.r) >= u16::from(color.b) + 24
}

fn is_greenish(color: Srgb) -> bool {
    u16::from(color.g) >= u16::from(color.r) + 24 && u16::from(color.g) >= u16::from(color.b) + 12
}

fn reserved_default(label: &'static str, mode: LagrangePaletteMode) -> LagrangePaletteEntry {
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

fn reserved_status_entry(
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

/// A typed DTCG document containing Tabard's current color group.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DtcgDocument {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub color: DtcgColorGroup,
}

/// The DTCG color group emitted by Tabard.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DtcgColorGroup {
    #[serde(rename = "$description")]
    pub description: String,
    #[serde(rename = "$extensions")]
    pub extensions: BTreeMap<String, DtcgProvenance>,
    pub bg: DtcgColorToken,
    pub surface: DtcgColorToken,
    #[serde(rename = "surface-2")]
    pub surface_2: DtcgColorToken,
    #[serde(rename = "surface-hover")]
    pub surface_hover: DtcgColorToken,
    #[serde(rename = "text-header")]
    pub text_header: DtcgColorToken,
    pub text: DtcgColorToken,
    #[serde(rename = "text-dim")]
    pub text_dim: DtcgColorToken,
    #[serde(rename = "text-disabled")]
    pub text_disabled: DtcgColorToken,
    pub primary: DtcgColorToken,
    #[serde(rename = "on-primary")]
    pub on_primary: DtcgColorToken,
    pub secondary: DtcgColorToken,
    #[serde(rename = "on-secondary")]
    pub on_secondary: DtcgColorToken,
    pub tertiary: DtcgColorToken,
    #[serde(rename = "on-tertiary")]
    pub on_tertiary: DtcgColorToken,
    pub success: DtcgColorToken,
    pub danger: DtcgColorToken,
}

impl DtcgColorGroup {
    fn from_theme(theme: &Theme, palette: Palette) -> Self {
        let mut extensions = BTreeMap::new();
        extensions.insert(
            TABARD_EXTENSION_KEY.to_owned(),
            DtcgProvenance {
                theme: DtcgThemeSource {
                    name: theme.name.clone(),
                    seeds: theme.seeds,
                },
                derivation: DtcgDerivation {
                    crate_name: "tinct".to_owned(),
                    function: "derive_palette".to_owned(),
                    profile: "normal-contrast".to_owned(),
                },
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
            success: palette.success.into(),
            danger: palette.danger.into(),
        }
    }
}

/// A DTCG color token with an explicit type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DtcgColorToken {
    #[serde(rename = "$type")]
    pub token_type: DtcgTokenType,
    #[serde(rename = "$value")]
    pub value: DtcgColorValue,
}

impl From<Srgb> for DtcgColorToken {
    fn from(color: Srgb) -> Self {
        Self {
            token_type: DtcgTokenType::Color,
            value: color.into(),
        }
    }
}

/// The only DTCG token type emitted by the first Tabard slice.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DtcgTokenType {
    #[serde(rename = "color")]
    Color,
}

/// A DTCG sRGB color value with a CSS hexadecimal fallback.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DtcgColorValue {
    #[serde(rename = "colorSpace")]
    pub color_space: String,
    pub components: [f64; 3],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpha: Option<f64>,
    pub hex: String,
}

impl From<Srgb> for DtcgColorValue {
    fn from(color: Srgb) -> Self {
        Self {
            color_space: "srgb".to_owned(),
            components: [
                f64::from(color.r) / f64::from(u8::MAX),
                f64::from(color.g) / f64::from(u8::MAX),
                f64::from(color.b) / f64::from(u8::MAX),
            ],
            alpha: (color.a != u8::MAX).then(|| f64::from(color.a) / f64::from(u8::MAX)),
            hex: color_to_hex(color),
        }
    }
}

/// Tabard-specific metadata which records how a DTCG color group was derived.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DtcgProvenance {
    pub theme: DtcgThemeSource,
    pub derivation: DtcgDerivation,
}

/// The authored source carried in the Tabard extension.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DtcgThemeSource {
    pub name: String,
    pub seeds: Seeds,
}

/// The fixed derivation choice behind this first palette artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DtcgDerivation {
    #[serde(rename = "crate")]
    pub crate_name: String,
    pub function: String,
    pub profile: String,
}

#[derive(Clone, Copy)]
struct ColorRole {
    name: &'static str,
    value: Srgb,
}

fn color_roles(palette: Palette) -> [ColorRole; 16] {
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

fn css_color(color: Srgb) -> String {
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

fn css_alpha(alpha: u8) -> String {
    let mut value = format!("{:.6}", f64::from(alpha) / f64::from(u8::MAX));
    while value.ends_with('0') {
        value.pop();
    }
    if value.ends_with('.') {
        value.pop();
    }
    value
}
