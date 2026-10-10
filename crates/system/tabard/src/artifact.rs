// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Explicit canonical-mode artifacts. These share the workshop's harmony and
//! contrast derivation and refuse stylesheet overrides rather than presenting
//! derived seed colors as an interpretation of arbitrary authored CSS.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tinct::{ModeProfile, Palette, Seeds, SyntaxPalette};

use crate::library::validate_definition;
use crate::theme::registry::Mode;
use crate::theme::seed::harmonized_seeds;
use crate::{
    DTCG_2025_10_SCHEMA, DtcgColorGroup, DtcgDerivation, DtcgDocument, TABARD_EXTENSION_KEY, Theme,
    custom_properties,
};

/// The exact derived base and syntax palettes for one canonical mode. The
/// effective seeds record harmony after application; the explicit `mode`
/// controls contrast and scheme independently of the authored seed preference.
#[derive(Clone, Debug, PartialEq)]
pub struct ModePalette {
    pub mode: Mode,
    pub effective_seeds: Seeds,
    pub palette: Palette,
    pub syntax: SyntaxPalette,
}

impl ModePalette {
    /// Emit this already-derived mode as base and syntax properties at `:root`.
    /// This avoids deriving and validating again when a retained preview holds
    /// the exact palette for its current seeds, harmony and selected mode.
    pub fn css_custom_properties(&self) -> String {
        format!(
            "/* Tabard derived mode: {}; harmony applied; Tinct explicit-mode base and syntax palettes. */\n{}",
            self.mode.as_key(),
            custom_properties(self.palette, self.syntax),
        )
    }
}

/// Why an exact canonical derived artifact could not be produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModeExportError {
    /// A custom mode needs its registered calculator or stylesheet renderer.
    UnsupportedCustomMode(String),
    /// The selected mode has authored CSS; seed tokens cannot represent it.
    StylesheetOverride(String),
    /// The authored definition is not admissible as a portable theme.
    InvalidTheme(String),
    Serialization(String),
}

impl std::fmt::Display for ModeExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedCustomMode(mode) => write!(
                f,
                "Cannot export derived colors for {mode}: a custom mode needs its calculator or stylesheet renderer."
            ),
            Self::StylesheetOverride(mode) => write!(
                f,
                "Cannot export derived colors for {mode}: this mode has an authored stylesheet. Export the theme definition to preserve it."
            ),
            Self::InvalidTheme(reason) => write!(f, "Cannot export this theme: {reason}"),
            Self::Serialization(reason) => {
                write!(f, "Could not serialize the mode artifact: {reason}")
            },
        }
    }
}

impl std::error::Error for ModeExportError {}

/// DTCG's existing color artifact plus root Tabard metadata sufficient to
/// reproduce the selected-mode derivation. The nested document retains the
/// existing token shape; old readers may ignore the root extension.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DtcgModeDocument {
    #[serde(flatten)]
    pub document: DtcgDocument,
    #[serde(rename = "$extensions")]
    pub extensions: BTreeMap<String, DtcgModeProvenance>,
}

/// Authored definition and effective derivation inputs for one mode artifact.
/// The definition preserves every attached sheet; its presence in provenance
/// does not mean those sheets were evaluated for this derived color artifact.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DtcgModeProvenance {
    pub theme: Theme,
    /// Stable [`Mode::as_key`] string.
    pub mode: String,
    pub effective_seeds: Seeds,
    pub derivation: DtcgDerivation,
}

impl Theme {
    /// Derive the canonical mode's palettes exactly as the workshop does:
    /// harmony first, then explicit scheme and contrast profiles in Tinct.
    ///
    /// Custom modes and nonempty sheets attached to the selected mode are
    /// rejected. A sheet for another mode does not affect this derivation.
    /// The authored definition must pass the same validation as portable JSON
    /// and library persistence before colors or source provenance are exported.
    pub fn palette_for_mode(&self, mode: &Mode) -> Result<ModePalette, ModeExportError> {
        if matches!(mode, Mode::Custom(_)) {
            return Err(ModeExportError::UnsupportedCustomMode(mode.as_key()));
        }
        if self.mode_sheet(mode).is_some() {
            return Err(ModeExportError::StylesheetOverride(mode.as_key()));
        }
        validate_definition(self).map_err(ModeExportError::InvalidTheme)?;
        let effective_seeds = harmonized_seeds(self);
        let profile = ModeProfile {
            dark: mode.dark(),
            high_contrast: mode.high_contrast(),
        };
        Ok(ModePalette {
            mode: mode.clone(),
            effective_seeds,
            palette: tinct::derive_palette_with(&effective_seeds, profile),
            syntax: tinct::derive_syntax_palette_with(&effective_seeds, profile),
        })
    }

    /// Export the exact selected canonical mode as base and syntax properties
    /// at `:root`. This is a derived color artifact, not a complete application
    /// stylesheet; arbitrary mode-sheet overrides cannot be projected into it.
    pub fn css_custom_properties_for_mode(&self, mode: &Mode) -> Result<String, ModeExportError> {
        let derived = self.palette_for_mode(mode)?;
        Ok(derived.css_custom_properties())
    }

    /// Export exact selected-mode color tokens with authored-source provenance.
    /// No arbitrary stylesheet override is silently replaced with seed colors.
    pub fn design_tokens_for_mode(&self, mode: &Mode) -> Result<DtcgModeDocument, ModeExportError> {
        let derived = self.palette_for_mode(mode)?;
        let derivation = DtcgDerivation {
            crate_name: "tinct".to_owned(),
            function: "derive_palette_with".to_owned(),
            profile: if mode.high_contrast() {
                "high-contrast"
            } else {
                "normal-contrast"
            }
            .to_owned(),
        };
        let mut color = DtcgColorGroup::from_palettes(
            self,
            derived.palette,
            derived.syntax,
            derivation.clone(),
        );
        color.syntax.description =
            "Tinct-derived syntax palette (derive_syntax_palette_with).".to_owned();
        let document = DtcgDocument {
            schema: DTCG_2025_10_SCHEMA.to_owned(),
            color,
        };
        let extensions = BTreeMap::from([(
            TABARD_EXTENSION_KEY.to_owned(),
            DtcgModeProvenance {
                theme: self.clone(),
                mode: mode.as_key(),
                effective_seeds: derived.effective_seeds,
                derivation,
            },
        )]);
        Ok(DtcgModeDocument {
            document,
            extensions,
        })
    }

    /// Serialize the exact selected-mode DTCG document as deterministic JSON.
    pub fn design_tokens_json_for_mode(&self, mode: &Mode) -> Result<String, ModeExportError> {
        serde_json::to_string_pretty(&self.design_tokens_for_mode(mode)?)
            .map_err(|error| ModeExportError::Serialization(error.to_string()))
    }
}
