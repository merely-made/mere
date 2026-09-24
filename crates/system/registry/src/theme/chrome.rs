// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Chrome color tokens — the browser-shell band a host paints around the graph
//! (toolbar / omnibar / suggestions / command palette / settings / context menu
//! / comms pane). These were hardcoded in meerkat's `CHROME_SHEET` CSS; lifting
//! them into a [`ChromeTheme`] makes the shell chrome theme-switchable the same
//! way the graph chrome already is, and keeps one source of truth per theme.
//!
//! The token set is deliberately small (a handful of surfaces + text weights)
//! and reused across the many CSS classes — the toolbar, dropdowns, palette, and
//! comms pane all draw from the same dozen-and-change colors, so a theme reads as
//! one coherent surface rather than a pile of one-off hex values.

pub use tincture::Srgb;

/// Color tokens for the host's chrome band (the shell around the graph canvas).
///
/// Surfaces step up in three tiers — `toolbar_bg` (the band) < `panel_bg`
/// (dropdowns docked to it) < `surface_bg` (floated panels: palette / settings)
/// — with `control_bg` / `field_bg` for the interactive widgets on top. Text
/// comes in three weights (`body_text` / `strong_text` / `muted_text`) plus the
/// `disabled_*` and `error_*` pairs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChromeTheme {
    /// The toolbar band itself (the chrome's base surface).
    pub toolbar_bg: Srgb,
    /// Button background (back / forward / palette buttons, settings + comms btns).
    pub control_bg: Srgb,
    /// Button label text.
    pub control_text: Srgb,
    /// Text-field (omnibar / palette query) background.
    pub field_bg: Srgb,
    /// Text-field text.
    pub field_text: Srgb,
    /// Dropdown / docked-panel surface (suggestions, comms pane) — a step over the
    /// toolbar band.
    pub panel_bg: Srgb,
    /// Floated-panel surface (command palette, settings) — a step over `panel_bg`.
    pub surface_bg: Srgb,
    /// Default row / body text on panels.
    pub body_text: Srgb,
    /// Emphasized text (active rows, titles).
    pub strong_text: Srgb,
    /// De-emphasized text (the sync chip, empty-state hints).
    pub muted_text: Srgb,
    /// Selection / active-row fill (the highlighted suggestion or command).
    pub active_bg: Srgb,
    /// Disabled control text.
    pub disabled_text: Srgb,
    /// Disabled control background.
    pub disabled_bg: Srgb,
    /// Small floating menus (the right-click context menu, incoming comms bubbles).
    pub menu_bg: Srgb,
    /// Failure-banner text (a comms send/load failure).
    pub error_text: Srgb,
    /// Failure-banner background.
    pub error_bg: Srgb,
}

impl ChromeTheme {
    /// Mere's dark chrome — the exact palette `CHROME_SHEET` shipped, lifted into
    /// tokens unchanged. Used by both the default and dark themes (both sit on a
    /// near-black graph backdrop).
    pub fn mere_dark() -> Self {
        Self {
            toolbar_bg: Srgb::rgb(28, 31, 38),
            control_bg: Srgb::rgb(44, 48, 58),
            control_text: Srgb::rgb(222, 226, 234),
            field_bg: Srgb::rgb(36, 39, 48),
            field_text: Srgb::rgb(232, 234, 240),
            panel_bg: Srgb::rgb(30, 33, 41),
            surface_bg: Srgb::rgb(34, 37, 46),
            body_text: Srgb::rgb(206, 210, 220),
            strong_text: Srgb::rgb(234, 238, 246),
            muted_text: Srgb::rgb(150, 156, 168),
            active_bg: Srgb::rgb(48, 58, 82),
            disabled_text: Srgb::rgb(108, 114, 126),
            disabled_bg: Srgb::rgb(34, 37, 45),
            menu_bg: Srgb::rgb(38, 42, 52),
            error_text: Srgb::rgb(240, 184, 184),
            error_bg: Srgb::rgb(50, 32, 32),
        }
    }

    /// A deeper dark chrome for the Dark theme — surfaces pulled toward black so
    /// Dark reads as *dark*, distinct from the Default theme's grayer slate (which
    /// keeps [`mere_dark`](Self::mere_dark)) and a step above High Contrast's pure
    /// black.
    pub fn mere_darker() -> Self {
        Self {
            toolbar_bg: Srgb::rgb(16, 18, 23),
            control_bg: Srgb::rgb(30, 34, 42),
            control_text: Srgb::rgb(222, 226, 234),
            field_bg: Srgb::rgb(22, 25, 31),
            field_text: Srgb::rgb(232, 234, 240),
            panel_bg: Srgb::rgb(15, 17, 22),
            surface_bg: Srgb::rgb(22, 25, 31),
            body_text: Srgb::rgb(202, 207, 217),
            strong_text: Srgb::rgb(234, 238, 246),
            muted_text: Srgb::rgb(140, 146, 158),
            active_bg: Srgb::rgb(40, 52, 76),
            disabled_text: Srgb::rgb(96, 102, 114),
            disabled_bg: Srgb::rgb(22, 25, 31),
            menu_bg: Srgb::rgb(26, 29, 36),
            error_text: Srgb::rgb(240, 184, 184),
            error_bg: Srgb::rgb(46, 28, 28),
        }
    }

    /// A light chrome — paper-toned surfaces with dark text, paired with the light
    /// theme's light graph backdrop.
    pub fn mere_light() -> Self {
        Self {
            toolbar_bg: Srgb::rgb(238, 240, 244),
            control_bg: Srgb::rgb(224, 228, 234),
            control_text: Srgb::rgb(32, 38, 48),
            field_bg: Srgb::rgb(250, 251, 253),
            field_text: Srgb::rgb(24, 28, 36),
            panel_bg: Srgb::rgb(244, 246, 250),
            surface_bg: Srgb::rgb(248, 250, 253),
            body_text: Srgb::rgb(60, 66, 78),
            strong_text: Srgb::rgb(24, 28, 36),
            muted_text: Srgb::rgb(118, 124, 136),
            active_bg: Srgb::rgb(200, 216, 244),
            disabled_text: Srgb::rgb(160, 166, 176),
            disabled_bg: Srgb::rgb(230, 232, 236),
            menu_bg: Srgb::rgb(236, 238, 243),
            error_text: Srgb::rgb(140, 32, 32),
            error_bg: Srgb::rgb(250, 228, 228),
        }
    }

    /// A maximum-contrast chrome — black surfaces, white text, a muted-gray active
    /// fill (a yellow fill would fail contrast against the white row text).
    pub fn high_contrast() -> Self {
        Self {
            toolbar_bg: Srgb::rgb(0, 0, 0),
            control_bg: Srgb::rgb(0, 0, 0),
            control_text: Srgb::rgb(255, 255, 255),
            field_bg: Srgb::rgb(0, 0, 0),
            field_text: Srgb::rgb(255, 255, 255),
            panel_bg: Srgb::rgb(0, 0, 0),
            surface_bg: Srgb::rgb(0, 0, 0),
            body_text: Srgb::rgb(255, 255, 255),
            strong_text: Srgb::rgb(255, 255, 255),
            muted_text: Srgb::rgb(255, 255, 255),
            active_bg: Srgb::rgb(40, 40, 40),
            disabled_text: Srgb::rgb(160, 160, 160),
            disabled_bg: Srgb::rgb(0, 0, 0),
            menu_bg: Srgb::rgb(0, 0, 0),
            error_text: Srgb::rgb(255, 128, 128),
            error_bg: Srgb::rgb(40, 0, 0),
        }
    }
}

impl Default for ChromeTheme {
    fn default() -> Self {
        Self::mere_dark()
    }
}
