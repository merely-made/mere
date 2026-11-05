// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Tabard owns portable theme artifacts: Tinct seeds in, typed design tokens
//! and a Livery stylesheet out.
//!
//! [`Theme`] derives Tinct's base palette and emits DTCG tokens, a Livery
//! stylesheet and a Lagrange palette; Pelt previews it. The [`theme`] module
//! holds the host theme model: `ThemeTokenSet`, its registry, seed derivation,
//! custom modes, and the user's theme choice with its stores. The [`smolweb`]
//! module holds the smolweb document palettes. The DTCG tokens and the
//! stylesheet carry Tinct's syntax palette beside the base palette. Icon
//! policy is not here yet.

#![doc(html_no_source)]
#![forbid(unsafe_code)]

pub mod appearance;
pub mod artifact;
pub mod library;
pub mod portable;
pub mod smolweb;
pub mod theme;
pub mod theme_styles;
pub mod typography;
pub mod workshop;
pub use theme_styles::{ThemeFontStyle, Typography};
mod reading;
pub use reading::{ReadingAnchor, ReadingPreferences, ReadingProjections, ReadingTrail};
pub use typography::{FontFamily, FontPreferences, FontRole, ResolvedFontRole};

pub use appearance::{
    ResolvedThemeChoice, ThemeChoiceDiagnostic, ThemePresentation, presentation_palette,
    presentation_stylesheet, resolve_theme_choice,
};
pub use artifact::{DtcgModeDocument, DtcgModeProvenance, ModeExportError, ModePalette};

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::theme::registry::{Harmony, Mode, ThemeSource};
use tinct::{
    GatedRoles, Palette, Seeds, Srgb, SyntaxPalette, SyntaxRole, color_to_hex, derive_gated_roles,
    derive_palette, derive_syntax_palette,
};

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

/// An authored theme: an id, a human-facing name and the small Tinct seed
/// set, plus how the registry derives from them. The full
/// [`ThemeTokenSet`](theme::registry::ThemeTokenSet) is derived from this
/// (see [`theme::seed::derive_from_def`]). User themes persist as this (a
/// theme file or settings entry); built-ins carry it too, so editing one can
/// fork a user copy from its seeds.
///
/// The legacy no-argument exports derive raw authored seeds at normal
/// contrast, selected by `Seeds::dark`. Explicit-mode exports apply harmony
/// and the selected canonical contrast profile; see [`Theme::palette_for_mode`].
/// Hosts map semantic roles onto controls and document styles. CSS carries
/// optional font properties; typed DTCG tokens remain color-only.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Theme {
    pub id: String,
    pub name: String,
    /// Defaults to `User` so a loaded theme file is a user theme without
    /// needing the field.
    #[serde(default)]
    pub source: ThemeSource,
    pub seeds: Seeds,
    /// High-contrast derivation mode (forced extremes + max-contrast text).
    #[serde(default)]
    pub high_contrast: bool,
    /// How the accents relate to the primary (default `Custom` = independent).
    #[serde(default)]
    pub harmony: Harmony,
    /// Per-mode CUSTOM STYLESHEET overrides (theme-modes T4): CSS rule lists
    /// keyed by [`Mode::as_key`] (`"dark"`, `"hc_light"`, …). When a mode has
    /// an entry, the host renders that sheet for (theme, mode) instead of the
    /// palette-derived one; modes without an entry keep deriving. Authored by
    /// hand in the theme file today (the mod-distribution path); empty = fully
    /// derived. The host's scheme-pair baking only applies when BOTH scheme
    /// counterparts are derived — an override on either side of the pair
    /// routes that theme through the sheet-swap path (correctness first).
    #[serde(default)]
    pub mode_sheets: BTreeMap<String, Vec<String>>,
    /// Portable role preferences; absent fields preserve host defaults.
    #[serde(default, skip_serializing_if = "Typography::is_empty")]
    pub typography: Typography,
    /// The person's own category colours, replacing Tinct's derived ones;
    /// each is still gated to 3:1 on bg and surface. Empty = fully derived.
    #[serde(default, skip_serializing_if = "CategoryOverrides::is_empty")]
    pub categories: CategoryOverrides,
}

/// Per-theme overrides for the five category colours (index 0 is
/// `category-1`). A `None` keeps Tinct's derived colour. An override is gated
/// like the colour it replaces (3:1 on `bg` and `surface`), so a theme can
/// choose a hue but not an illegible mark.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryOverrides(pub [Option<Srgb>; 5]);

impl CategoryOverrides {
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(Option::is_none)
    }
}

/// Tinct's gated roles for `palette`, with the theme's category overrides
/// applied and gated.
pub fn gated_roles(palette: &Palette, overrides: &CategoryOverrides) -> GatedRoles {
    let mut roles = derive_gated_roles(palette);
    let grounds = [palette.bg, palette.surface];
    for (slot, chosen) in roles.category.iter_mut().zip(overrides.0) {
        if let Some(color) = chosen {
            *slot = tinct::gated::gate(color, &grounds, tinct::gated::MARK_FLOOR);
        }
    }
    roles
}

impl Theme {
    /// A user theme from its id, name and authored Tinct seed set, with the
    /// derivation settings at their defaults.
    pub fn new(id: impl Into<String>, name: impl Into<String>, seeds: Seeds) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            source: ThemeSource::default(),
            seeds,
            high_contrast: false,
            harmony: Harmony::default(),
            mode_sheets: BTreeMap::new(),
            typography: Typography::default(),
            categories: CategoryOverrides::default(),
        }
    }

    /// This theme's custom stylesheet for `mode`, if one is attached. Empty
    /// rule lists count as absent (a stray empty entry can't blank the shell).
    pub fn mode_sheet(&self, mode: &Mode) -> Option<&Vec<String>> {
        self.mode_sheets
            .get(&mode.as_key())
            .filter(|rules| !rules.is_empty())
    }

    /// Derive the base palette owned by the current Tabard artifact.
    pub fn palette(&self) -> Palette {
        derive_palette(&self.seeds)
    }

    /// Derive Tinct's syntax-highlight palette, contrast-gated against the
    /// base palette's surface.
    pub fn syntax_palette(&self) -> SyntaxPalette {
        derive_syntax_palette(&self.seeds)
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
    /// --tabard-color-bg, --tabard-color-surface-2, and so on, with success
    /// and danger gated to 4.5:1; then Tinct's
    /// contrast-gated roles (--tabard-color-accent, -focus, -border,
    /// -border-strong, -category-1 … -category-5); then the syntax roles as
    /// --tabard-syntax-keyword and the like. A host may append
    /// ordinary author rules which use them through var().
    pub fn css_custom_properties(&self) -> String {
        format!(
            "{}{}",
            custom_properties(self.palette(), self.syntax_palette(), &self.categories),
            self.typography.css_custom_properties()
        )
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

/// The palette as published: the base roles, with `success` and `danger`
/// replaced by their contrast-gated values (4.5:1 on `bg` and `surface`), so
/// status text holds its floor in every mode. The raw seeds stay in the
/// theme's provenance and in the Lagrange export.
fn published(palette: Palette) -> Palette {
    let gated = derive_gated_roles(&palette);
    Palette {
        success: gated.success,
        danger: gated.danger,
        ..palette
    }
}

/// A generated Lagrange `palette.txt` plus explicit diagnostics about the
/// projection from Tabard's richer role set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LagrangePaletteExport {
    pub text: String,
    pub diagnostics: Vec<LagrangePaletteDiagnostic>,
}

/// The two palettes emitted for Lagrange's `palette.txt` file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LagrangePaletteMode {
    Dark,
    Light,
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
    /// Tinct's contrast-gated roles (accent, focus, border, border-strong,
    /// category-1 … category-5). Absent in documents written before
    /// 2026-10-16.
    #[serde(default)]
    pub gated: DtcgGatedGroup,
    pub syntax: DtcgSyntaxGroup,
}

/// Tinct's contrast-gated roles as a DTCG group, keyed by the CSS suffix
/// (`accent`, `border-strong`, `category-1`, …).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DtcgGatedGroup {
    #[serde(rename = "$description", default)]
    pub description: String,
    #[serde(flatten)]
    pub roles: BTreeMap<String, DtcgColorToken>,
}

impl DtcgGatedGroup {
    fn new(palette: Palette, categories: &CategoryOverrides) -> Self {
        Self {
            description: "Tinct-derived roles gated against bg and surface (derive_gated_roles): 4.5:1 for accent, 3:1 for focus, border-strong and category marks; border is decorative.".to_owned(),
            roles: gated_roles(&palette, categories)
                .named()
                .into_iter()
                .map(|(name, value)| (name.to_owned(), value.into()))
                .collect(),
        }
    }
}

/// Tinct's syntax-highlight palette as a DTCG group: one color token per
/// [`SyntaxRole`], keyed by [`SyntaxRole::name`] and contrast-gated against
/// the palette surface.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DtcgSyntaxGroup {
    #[serde(rename = "$description")]
    pub description: String,
    #[serde(flatten)]
    pub roles: BTreeMap<String, DtcgColorToken>,
}

impl From<SyntaxPalette> for DtcgSyntaxGroup {
    fn from(palette: SyntaxPalette) -> Self {
        Self {
            description: "Tinct-derived syntax palette (derive_syntax_palette).".to_owned(),
            roles: SyntaxRole::ALL
                .into_iter()
                .map(|role| (role.name().to_owned(), palette.role(role).into()))
                .collect(),
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

#[path = "lagrange.rs"]
mod lagrange;

use lagrange::srgb_luma;

#[path = "css.rs"]
mod css;

use css::custom_properties;

#[path = "dtcg.rs"]
mod dtcg;
