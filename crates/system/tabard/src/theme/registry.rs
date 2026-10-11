// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Theme registry — [`ThemeRegistry`] keeps the derived token sets and the
//! authored defs behind them, keyed by lowercased `theme:*` id: built-ins
//! first, then user themes, with resolution, forks and modes. Token
//! validation delegates to `crate::theme::edge_style::validate_theme_edge_tokens`
//! (the bundled-in vocabulary, see [`crate::theme`] for the bundling rationale).
//!
//! Cross-crate retargets vs. the original shell-side `theme.rs`:
//! - `crate::theme::graph::edge_style_registry::*` → `crate::theme::edge_style::*`
//! - `crate::theme::registries::atomic::lens::*` → `registry::lens::*`
//! - the dead `#[cfg(feature = "egui-host")] pub use egui::Color32;`
//!   alias was dropped per the "remove egui" half of the bundle decision —
//!   `egui-host` is an empty no-op feature in root `Cargo.toml:96` and egui
//!   is no longer in the dep graph
//! - `pub` → `pub` throughout (items are now this crate's public API)
//!
//! Moved from mere-registry into tabard on 2026-09-24; lens's theme items are
//! now `crate::theme::data`.

use std::collections::HashMap;

pub use tinct::Srgb;

use crate::Theme;
use crate::theme::chrome::ChromeTheme;
use crate::theme::data::ThemeData;
use crate::theme::edge_style::{
    EdgeAccessibilityMode, ThemeAccessibilitySupport, ThemeContract, ThemeEdgeTokens,
    validate_theme_edge_tokens,
};

/// Color tokens for graph-node chrome (badges, pinned fill, rings, default stroke).
///
/// Moved inline here from the retired `graph::egui_adapter` module; themes are
/// the sole consumer.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GraphNodeChromeTheme {
    pub workspace_badge_background: Srgb,
    pub workspace_badge_text: Srgb,
    pub semantic_badge_background: Srgb,
    pub semantic_badge_text: Srgb,
    pub semantic_badge_overflow_background: Srgb,
    pub semantic_badge_orbit_background: Srgb,
    pub pinned_fill: Srgb,
    pub pinned_stroke: Srgb,
    pub clip_ring: Srgb,
    pub default_stroke: Srgb,
}

impl Default for GraphNodeChromeTheme {
    fn default() -> Self {
        Self {
            workspace_badge_background: Srgb::rgba(20, 30, 46, 224),
            workspace_badge_text: Srgb::gray(245),
            semantic_badge_background: Srgb::rgba(34, 44, 64, 224),
            semantic_badge_text: Srgb::gray(245),
            semantic_badge_overflow_background: Srgb::rgba(24, 24, 24, 216),
            semantic_badge_orbit_background: Srgb::rgba(20, 28, 42, 230),
            pinned_fill: Srgb::WHITE,
            pinned_stroke: Srgb::gray(40),
            clip_ring: Srgb::rgb(170, 210, 255),
            default_stroke: Srgb::gray(90),
        }
    }
}

pub const THEME_ID_DEFAULT: &str = "theme:default";
pub const THEME_ID_LIGHT: &str = "theme:light";
pub const THEME_ID_DARK: &str = "theme:dark";
pub const THEME_ID_HIGH_CONTRAST: &str = "theme:high_contrast";

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ThemeTokenSet {
    pub theme_id: String,
    pub display_name: String,
    pub theme_data: ThemeData,
    pub accessibility: ThemeAccessibilitySupport,
    pub theme_contract: ThemeContract,
    pub edge_tokens: ThemeEdgeTokens,
    pub command_notice: Srgb,
    pub radial_disabled_text: Srgb,
    pub radial_hub_fill: Srgb,
    pub radial_hub_stroke: Srgb,
    pub radial_hub_text: Srgb,
    pub radial_domain_active_fill: Srgb,
    pub radial_domain_idle_fill: Srgb,
    pub radial_command_active_fill: Srgb,
    pub radial_command_hover_fill: Srgb,
    pub radial_command_disabled_fill: Srgb,
    pub radial_command_text: Srgb,
    pub radial_chrome_text: Srgb,
    pub radial_warning_text: Srgb,
    pub hover_label_background: Srgb,
    pub hover_label_stroke: Srgb,
    pub hover_label_text: Srgb,
    pub graph_node_search_match: Srgb,
    pub graph_node_search_match_active: Srgb,
    pub graph_node_hover: Srgb,
    pub graph_node_selection: Srgb,
    pub graph_node_focus_ring: Srgb,
    pub graph_node_hover_ring: Srgb,
    pub graph_node_chrome: GraphNodeChromeTheme,
    /// Tokens for the host's shell chrome (toolbar / omnibar / palette / panes).
    pub chrome: ChromeTheme,
    pub status_success: Srgb,
    pub status_warning: Srgb,
    pub status_error: Srgb,
    pub status_neutral: Srgb,
    pub workbench_panel_background: Srgb,
    /// Highlight background for selection chrome over dense panels
    /// (dropdowns, toast strips). Paired with `selection_highlight_text`
    /// + `selection_highlight_stroke` to form a three-token trio that
    /// maintains contrast on any theme.
    pub selection_highlight_background: Srgb,
    pub selection_highlight_text: Srgb,
    pub selection_highlight_stroke: Srgb,
    pub semantic_origin_manual: Srgb,
    pub semantic_origin_semantic: Srgb,
    pub semantic_origin_anchor: Srgb,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeCapability {
    pub requested_id: String,
    pub resolved_id: String,
    pub matched: bool,
    pub fallback_used: bool,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeResolution {
    pub requested_id: String,
    pub resolved_id: String,
    pub matched: bool,
    pub fallback_used: bool,
    pub tokens: ThemeTokenSet,
}

/// A presentation MODE: a derivation profile applied to the active theme's
/// seeds (theme-modes plan, decision record 2026-07-05). The canonical four
/// pick a ladder direction + contrast spread; `Custom` names a registered
/// custom mode (a calculator producing a stylesheet from the seeds — T5, not
/// yet wired). A THEME stays a seed set; light/dark/high-contrast are no
/// longer distinct themes but derivations of the current one.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Mode {
    Light,
    Dark,
    HcLight,
    HcDark,
    Custom(String),
}

impl Mode {
    /// Ladder direction: dark surfaces + light text. `Custom` reports dark so
    /// an unresolved custom mode degrades to the dark canonical derivation.
    pub fn dark(&self) -> bool {
        matches!(self, Mode::Dark | Mode::HcDark | Mode::Custom(_))
    }

    /// Whether this is a high-contrast derivation.
    pub fn high_contrast(&self) -> bool {
        matches!(self, Mode::HcLight | Mode::HcDark)
    }

    /// The canonical mode for a `(dark, high_contrast)` flag pair.
    pub fn from_flags(dark: bool, high_contrast: bool) -> Self {
        match (dark, high_contrast) {
            (false, false) => Mode::Light,
            (true, false) => Mode::Dark,
            (false, true) => Mode::HcLight,
            (true, true) => Mode::HcDark,
        }
    }

    /// Stable settings key (`light` / `dark` / `hc_light` / `hc_dark` /
    /// `custom:<id>`).
    pub fn as_key(&self) -> String {
        match self {
            Mode::Light => "light".to_string(),
            Mode::Dark => "dark".to_string(),
            Mode::HcLight => "hc_light".to_string(),
            Mode::HcDark => "hc_dark".to_string(),
            Mode::Custom(id) => format!("custom:{id}"),
        }
    }

    /// Parse a settings key back to a mode. `None` for an unknown key.
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "light" => Some(Mode::Light),
            "dark" => Some(Mode::Dark),
            "hc_light" => Some(Mode::HcLight),
            "hc_dark" => Some(Mode::HcDark),
            _ => key
                .strip_prefix("custom:")
                .map(|id| Mode::Custom(id.to_string())),
        }
    }

    /// The user-facing label.
    pub fn label(&self) -> String {
        match self {
            Mode::Light => "Light".to_string(),
            Mode::Dark => "Dark".to_string(),
            Mode::HcLight => "High contrast light".to_string(),
            Mode::HcDark => "High contrast dark".to_string(),
            Mode::Custom(id) => format!("Custom ({id})"),
        }
    }
}

/// Where a theme came from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThemeSource {
    /// A code-defined built-in. Immutable in place; editing forks a user copy.
    BuiltIn,
    /// A user-authored theme (created in-app, or loaded from a theme file).
    #[default]
    User,
}

/// How the secondary + tertiary accents relate to the primary. `Custom` keeps
/// them independent (each its own seed); `Locked` ties their **hue** to the
/// primary by a fixed offset in degrees (keeping each accent's own saturation +
/// lightness), so editing the base rotates the whole triad and the derived
/// activity accents stay coordinated. Presets (triadic / analogous / …) are
/// just `Locked` with a known offset pair; "lock current" captures the triad's
/// present hue gaps. (Seed-palette harmony.)
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Harmony {
    #[default]
    Custom,
    Locked {
        secondary_deg: f32,
        tertiary_deg: f32,
    },
}

pub struct ThemeRegistry {
    /// Derived, resolvable token sets (built-ins + user themes), keyed by
    /// lowercased id.
    themes: HashMap<String, ThemeTokenSet>,
    /// The authored def behind each theme — enables listing, forking, and
    /// re-deriving after an edit.
    defs: HashMap<String, Theme>,
    /// Listing order: built-ins first (registration order), then user themes.
    order: Vec<String>,
    active: String,
    fallback_id: String,
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        let mut registry = Self {
            themes: HashMap::new(),
            defs: HashMap::new(),
            order: Vec::new(),
            active: THEME_ID_DEFAULT.to_string(),
            fallback_id: THEME_ID_DEFAULT.to_string(),
        };
        for def in crate::theme::seed::builtin_defs() {
            let id = def.id.clone();
            registry
                .insert_def(def)
                .unwrap_or_else(|e| panic!("built-in theme {id} must be valid: {e}"));
        }
        registry
    }
}

impl ThemeRegistry {
    /// Derive + validate a def, then register it (or replace an existing entry
    /// of the same id, preserving its order slot).
    fn insert_def(&mut self, def: Theme) -> Result<(), String> {
        let tokens = crate::theme::seed::derive_from_def(&def);
        validate_theme_tokens(&tokens)?;
        let key = def.id.trim().to_ascii_lowercase();
        if !self.defs.contains_key(&key) {
            self.order.push(key.clone());
        }
        self.themes.insert(key.clone(), tokens);
        self.defs.insert(key, def);
        Ok(())
    }

    /// Add (or replace) a user theme from its def. Forces `source = User` and
    /// validates the derived tokens (a malformed seed set is rejected, not
    /// registered).
    pub fn add_user_theme(&mut self, mut def: Theme) -> Result<(), String> {
        def.source = ThemeSource::User;
        self.insert_def(def)
    }

    /// Remove a user theme by id. Built-ins + the fallback can't be removed.
    /// Returns whether anything was removed.
    pub fn remove_user_theme(&mut self, theme_id: &str) -> bool {
        let key = theme_id.trim().to_ascii_lowercase();
        if key == self.fallback_id {
            return false;
        }
        if !matches!(
            self.defs.get(&key).map(|d| d.source),
            Some(ThemeSource::User)
        ) {
            return false;
        }
        self.defs.remove(&key);
        self.themes.remove(&key);
        self.order.retain(|k| k != &key);
        true
    }

    /// Rename a user theme in place. Built-ins can't be renamed (fork instead).
    pub fn rename_user_theme(&mut self, theme_id: &str, name: &str) -> bool {
        let key = theme_id.trim().to_ascii_lowercase();
        match self.defs.get_mut(&key) {
            Some(d) if d.source == ThemeSource::User => {
                d.name = name.to_string();
                if let Some(tokens) = self.themes.get_mut(&key) {
                    tokens.display_name = name.to_string();
                }
                true
            },
            _ => false,
        }
    }

    /// Fork any theme (built-in or user) into a new **user** theme seeded from
    /// the source's seeds. The non-destructive "edit a built-in" path. Returns
    /// the new def, or `None` if `source_id` is unknown / the new id collides.
    pub fn fork(&mut self, source_id: &str, new_id: &str, new_name: &str) -> Option<Theme> {
        let src = self
            .defs
            .get(&source_id.trim().to_ascii_lowercase())?
            .clone();
        let new_key = new_id.trim().to_ascii_lowercase();
        if self.defs.contains_key(&new_key) {
            return None;
        }
        let def = Theme {
            id: new_id.to_string(),
            name: new_name.to_string(),
            source: ThemeSource::User,
            seeds: src.seeds,
            high_contrast: src.high_contrast,
            harmony: src.harmony,
            // A fork carries the source's per-mode custom sheets (they are
            // part of the theme's look; remove them by editing the fork).
            mode_sheets: src.mode_sheets,
        };
        self.add_user_theme(def.clone()).ok()?;
        Some(def)
    }

    /// All themes in listing order (built-ins first, then user).
    pub fn list(&self) -> Vec<&Theme> {
        self.order.iter().filter_map(|k| self.defs.get(k)).collect()
    }

    /// The authored def for a theme id (for editing / export).
    pub fn theme_def(&self, theme_id: &str) -> Option<&Theme> {
        self.defs.get(&theme_id.trim().to_ascii_lowercase())
    }

    pub fn resolve_theme(&self, theme_id: Option<&str>) -> ThemeResolution {
        let requested = theme_id
            .unwrap_or(self.active.as_str())
            .trim()
            .to_ascii_lowercase();
        let fallback = self
            .themes
            .get(&self.fallback_id)
            .cloned()
            .unwrap_or_else(crate::theme::seed::default_token_set);

        if requested.is_empty() {
            return ThemeResolution {
                requested_id: requested,
                resolved_id: self.fallback_id.clone(),
                matched: false,
                fallback_used: true,
                tokens: fallback,
            };
        }

        if let Some(tokens) = self.themes.get(&requested).cloned() {
            return ThemeResolution {
                requested_id: requested.clone(),
                resolved_id: requested,
                matched: true,
                fallback_used: false,
                tokens,
            };
        }

        ThemeResolution {
            requested_id: requested,
            resolved_id: self.fallback_id.clone(),
            matched: false,
            fallback_used: true,
            tokens: fallback,
        }
    }

    pub fn describe_theme(&self, theme_id: Option<&str>) -> ThemeCapability {
        let resolution = self.resolve_theme(theme_id);
        ThemeCapability {
            requested_id: resolution.requested_id,
            resolved_id: resolution.resolved_id,
            matched: resolution.matched,
            fallback_used: resolution.fallback_used,
            display_name: resolution.tokens.display_name,
        }
    }

    /// The active-or-named theme's tokens derived under an explicit [`Mode`]
    /// (theme-modes plan): the def's seeds re-derived with the mode's ladder
    /// direction + contrast spread. `None` when the id resolves to no def.
    pub fn mode_tokens(&self, theme_id: &str, mode: &Mode) -> Option<ThemeTokenSet> {
        self.theme_def(theme_id)
            .map(|def| crate::theme::seed::derive_from_def_for_mode(def, mode))
    }

    pub fn set_active_theme(&mut self, theme_id: &str) -> ThemeResolution {
        let resolution = self.resolve_theme(Some(theme_id));
        self.active = resolution.resolved_id.clone();
        resolution
    }

    pub fn active_theme(&self) -> ThemeResolution {
        self.resolve_theme(None)
    }
}

/// Toggle a user theme's light/dark mode in place. Returns whether the edit applied.
pub fn toggle_user_theme_mode(def: &mut Theme) -> bool {
    if def.source != ThemeSource::User {
        return false;
    }
    def.seeds.dark = !def.seeds.dark;
    true
}

/// Set one HSL channel of one seed of a user theme to `fraction` of its range.
/// Returns whether the edit applied.
pub fn set_user_theme_seed_channel(
    def: &mut Theme,
    seed: &str,
    channel: char,
    fraction: f64,
) -> bool {
    if def.source != ThemeSource::User {
        return false;
    }
    let target = match seed {
        "primary" => &mut def.seeds.primary,
        "secondary" => &mut def.seeds.secondary,
        "tertiary" => &mut def.seeds.tertiary,
        "neutral" => &mut def.seeds.neutral,
        _ => return false,
    };
    let (mut h, mut s, mut l) = tinct::color_to_hsl(*target);
    let f = fraction.clamp(0.0, 1.0);
    match channel {
        'h' => h = (f * 360.0).rem_euclid(360.0),
        's' => s = f,
        'l' => l = f,
        _ => return false,
    }
    *target = tinct::color_from_hsl(h, s, l);
    true
}

/// Set a user theme's accent harmony by key. Returns whether the edit applied.
pub fn set_user_theme_harmony(def: &mut Theme, key: &str) -> bool {
    use tinct::oklch::Oklch;

    if def.source != ThemeSource::User {
        return false;
    }
    let base_h = Oklch::from_srgb(def.seeds.primary).h;
    let gap = |c| {
        (Oklch::from_srgb(c).h - base_h)
            .to_degrees()
            .rem_euclid(360.0) as f32
    };
    def.harmony = match key {
        "custom" => Harmony::Custom,
        "lock" => Harmony::Locked {
            secondary_deg: gap(def.seeds.secondary),
            tertiary_deg: gap(def.seeds.tertiary),
        },
        "triadic" => Harmony::Locked {
            secondary_deg: 120.0,
            tertiary_deg: 240.0,
        },
        "analogous" => Harmony::Locked {
            secondary_deg: 30.0,
            tertiary_deg: -30.0,
        },
        "complementary" => Harmony::Locked {
            secondary_deg: 180.0,
            tertiary_deg: 150.0,
        },
        "mono" => Harmony::Locked {
            secondary_deg: 0.0,
            tertiary_deg: 0.0,
        },
        _ => return false,
    };
    true
}

fn validate_theme_tokens(tokens: &ThemeTokenSet) -> Result<(), String> {
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

fn contrast_ratio(foreground: Srgb, background: Srgb) -> f32 {
    let mut l1 = relative_luminance(foreground);
    let mut l2 = relative_luminance(background);
    if l2 > l1 {
        std::mem::swap(&mut l1, &mut l2);
    }
    (l1 + 0.05) / (l2 + 0.05)
}

fn relative_luminance(color: Srgb) -> f32 {
    0.2126 * to_linear_component(color.r)
        + 0.7152 * to_linear_component(color.g)
        + 0.0722 * to_linear_component(color.b)
}

fn to_linear_component(component: u8) -> f32 {
    let value = component as f32 / 255.0;
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests;
