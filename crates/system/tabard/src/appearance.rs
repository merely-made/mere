// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shared appearance resolution. Hosts retain their own settings, product
//! roles and stylesheet cascade; authored CSS is never interpreted as seed
//! colors by this boundary.

use crate::library::validate_definition;
use crate::theme::choice::ThemeChoice;
use crate::theme::registry::{Mode, ThemeRegistry};
use crate::theme::seed::default_mode_for_def;
use crate::{ModeExportError, ModePalette, Theme};

/// The authoritative presentation for a selected definition and mode.
/// An authored stylesheet must be rendered by the host's existing cascade.
/// It carries no claimed palette for those arbitrary rules.
#[derive(Clone, Debug, PartialEq)]
pub enum ThemePresentation {
    Derived(ModePalette),
    AuthoredStylesheet(Vec<String>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThemeChoiceDiagnostic {
    MissingTheme {
        requested_id: String,
        resolved_id: String,
    },
    MissingMode {
        requested_mode: Mode,
        resolved_mode: Mode,
    },
}

impl std::fmt::Display for ThemeChoiceDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingTheme {
                requested_id,
                resolved_id,
            } => write!(
                f,
                "Theme {requested_id} is unavailable; using {resolved_id}."
            ),
            Self::MissingMode {
                requested_mode,
                resolved_mode,
            } => write!(
                f,
                "Mode {} is unavailable; using {}.",
                requested_mode.label(),
                resolved_mode.label()
            ),
        }
    }
}

/// Resolution is read-only. `requested` remains byte-for-byte equivalent to
/// the caller's choice so missing libraries or modes can recover later.
/// `resolved.theme_mode` is always `Some`, including the authored default.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedThemeChoice {
    pub requested: ThemeChoice,
    pub resolved: ThemeChoice,
    pub theme: Theme,
    pub presentation: ThemePresentation,
    pub diagnostics: Vec<ThemeChoiceDiagnostic>,
}

impl Theme {
    /// Resolve this exact mode without falling back or mutating the definition.
    /// Attached CSS takes authority even for canonical modes. Custom modes
    /// without a sheet remain explicit errors; no calculator is invented.
    pub fn presentation_for_mode(&self, mode: &Mode) -> Result<ThemePresentation, ModeExportError> {
        if let Some(rules) = self.mode_sheet(mode) {
            validate_definition(self).map_err(ModeExportError::InvalidTheme)?;
            return Ok(ThemePresentation::AuthoredStylesheet(rules.clone()));
        }
        if matches!(mode, Mode::Custom(_)) {
            validate_definition(self).map_err(ModeExportError::InvalidTheme)?;
        }
        self.palette_for_mode(mode).map(ThemePresentation::Derived)
    }
}

/// Resolve a requested appearance through the existing registry. Missing
/// identities use its existing fallback; missing custom modes use the chosen
/// definition's default, each with a diagnostic. Invalid definitions remain
/// errors instead of silently being replaced. The registry is never activated.
pub fn resolve_theme_choice(
    registry: &ThemeRegistry,
    choice: &ThemeChoice,
) -> Result<ResolvedThemeChoice, ModeExportError> {
    let resolution = registry.resolve_theme(Some(&choice.theme_id));
    let theme = registry
        .theme_def(&resolution.resolved_id)
        .cloned()
        .ok_or_else(|| {
            ModeExportError::InvalidTheme(
                "the registry's fallback definition is unavailable".into(),
            )
        })?;
    let mut diagnostics = Vec::new();
    if !resolution.matched {
        diagnostics.push(ThemeChoiceDiagnostic::MissingTheme {
            requested_id: choice.theme_id.clone(),
            resolved_id: theme.id.clone(),
        });
    }
    let mut mode = choice
        .theme_mode
        .clone()
        .unwrap_or_else(|| default_mode_for_def(&theme));
    if matches!(mode, Mode::Custom(_)) && theme.mode_sheet(&mode).is_none() {
        let default = default_mode_for_def(&theme);
        diagnostics.push(ThemeChoiceDiagnostic::MissingMode {
            requested_mode: mode,
            resolved_mode: default.clone(),
        });
        mode = default;
    }
    let presentation = theme.presentation_for_mode(&mode)?;
    Ok(ResolvedThemeChoice {
        requested: choice.clone(),
        resolved: ThemeChoice::new(theme.id.clone(), Some(mode)),
        theme,
        presentation,
        diagnostics,
    })
}
