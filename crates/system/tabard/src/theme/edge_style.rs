// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::{BTreeMap, BTreeSet};

use tinct::Srgb;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum EdgeStyleFamily {
    Semantic,
    Traversal,
    Containment,
    Arrangement,
    Imported,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum EdgeStrokePattern {
    Solid,
    Dotted,
    DashedShort,
    DashedLong,
    DashDot,
    DoubleStroke,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum EdgeEndpointMarker {
    None,
    Arrow,
    Bracket,
    Square,
    Diamond,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum EdgeAccessibilityMode {
    ColorAndPattern,
    Monochrome,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ThemeEdgeFamilyToken {
    pub color: Srgb,
    pub pattern: EdgeStrokePattern,
    pub end_marker: EdgeEndpointMarker,
    pub width: f32,
    pub opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ThemeEdgeEmphasisToken {
    pub foreground_color: Srgb,
    pub halo_color: Srgb,
    pub halo_width: f32,
    pub width_delta: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ThemeAccessibilitySupport {
    pub supports_monochrome: bool,
    pub supports_high_contrast: bool,
    pub default_edge_mode: EdgeAccessibilityMode,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ThemeContract {
    pub min_family_luminance_delta: f32,
    pub require_non_color_family_distinction: bool,
    pub require_monochrome_preservation: bool,
}

/// Generic family styling shared across all relation kinds. Relation-specific
/// styling is resolved by the owning renderer from source assertion data.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ThemeEdgeTokens {
    pub family_tokens: BTreeMap<EdgeStyleFamily, ThemeEdgeFamilyToken>,
    pub hover: ThemeEdgeEmphasisToken,
    pub selection: ThemeEdgeEmphasisToken,
}

impl Default for ThemeEdgeTokens {
    fn default() -> Self {
        Self::default_theme()
    }
}

impl ThemeEdgeTokens {
    pub fn default_theme() -> Self {
        Self::with_palette(EdgeThemePalette {
            semantic: Srgb::rgb(150, 150, 155),
            selection: Srgb::rgb(236, 171, 64),
            traversal: Srgb::rgb(120, 180, 210),
            containment: Srgb::rgb(42, 168, 132),
            arrangement: Srgb::rgb(130, 110, 220),
            imported: Srgb::rgb(126, 112, 100),
            hover_halo: Srgb::rgba(236, 244, 255, 84),
            selection_halo: Srgb::rgba(255, 230, 164, 128),
        })
    }

    pub fn light_theme() -> Self {
        Self::with_palette(EdgeThemePalette {
            semantic: Srgb::rgb(108, 112, 122),
            selection: Srgb::rgb(196, 136, 36),
            traversal: Srgb::rgb(66, 128, 170),
            containment: Srgb::rgb(10, 130, 92),
            arrangement: Srgb::rgb(94, 88, 182),
            imported: Srgb::rgb(80, 72, 68),
            hover_halo: Srgb::rgba(40, 78, 114, 54),
            selection_halo: Srgb::rgba(214, 160, 56, 108),
        })
    }

    pub fn dark_theme() -> Self {
        Self::with_palette(EdgeThemePalette {
            semantic: Srgb::rgb(164, 168, 178),
            selection: Srgb::rgb(236, 171, 64),
            traversal: Srgb::rgb(124, 186, 220),
            containment: Srgb::rgb(52, 176, 140),
            arrangement: Srgb::rgb(142, 124, 236),
            imported: Srgb::rgb(124, 112, 104),
            hover_halo: Srgb::rgba(214, 230, 255, 72),
            selection_halo: Srgb::rgba(255, 220, 144, 132),
        })
    }

    pub fn high_contrast_theme() -> Self {
        Self::with_palette(EdgeThemePalette {
            semantic: Srgb::rgb(255, 255, 255),
            selection: Srgb::rgb(255, 230, 0),
            traversal: Srgb::rgb(0, 255, 255),
            containment: Srgb::rgb(0, 255, 170),
            arrangement: Srgb::rgb(255, 128, 0),
            imported: Srgb::rgb(188, 188, 188),
            hover_halo: Srgb::rgba(255, 255, 255, 120),
            selection_halo: Srgb::rgba(255, 230, 0, 156),
        })
    }

    fn with_palette(palette: EdgeThemePalette) -> Self {
        let family_tokens = BTreeMap::from([
            (
                EdgeStyleFamily::Semantic,
                ThemeEdgeFamilyToken {
                    color: palette.semantic,
                    pattern: EdgeStrokePattern::Solid,
                    end_marker: EdgeEndpointMarker::Arrow,
                    width: 1.4,
                    opacity: 0.85,
                },
            ),
            (
                EdgeStyleFamily::Traversal,
                ThemeEdgeFamilyToken {
                    color: palette.traversal,
                    pattern: EdgeStrokePattern::DashedLong,
                    end_marker: EdgeEndpointMarker::Arrow,
                    width: 1.8,
                    opacity: 0.7,
                },
            ),
            (
                EdgeStyleFamily::Containment,
                ThemeEdgeFamilyToken {
                    color: palette.containment,
                    pattern: EdgeStrokePattern::Dotted,
                    end_marker: EdgeEndpointMarker::Bracket,
                    width: 1.0,
                    opacity: 0.6,
                },
            ),
            (
                EdgeStyleFamily::Arrangement,
                ThemeEdgeFamilyToken {
                    color: palette.arrangement,
                    pattern: EdgeStrokePattern::DoubleStroke,
                    end_marker: EdgeEndpointMarker::Square,
                    width: 2.0,
                    opacity: 0.5,
                },
            ),
            (
                EdgeStyleFamily::Imported,
                ThemeEdgeFamilyToken {
                    color: palette.imported,
                    pattern: EdgeStrokePattern::DashDot,
                    end_marker: EdgeEndpointMarker::Diamond,
                    width: 0.8,
                    opacity: 0.35,
                },
            ),
        ]);
        Self {
            family_tokens,
            hover: ThemeEdgeEmphasisToken {
                foreground_color: Srgb::rgb(210, 226, 242),
                halo_color: palette.hover_halo,
                halo_width: 1.0,
                width_delta: 0.5,
            },
            selection: ThemeEdgeEmphasisToken {
                foreground_color: palette.selection,
                halo_color: palette.selection_halo,
                halo_width: 1.2,
                width_delta: 1.0,
            },
        }
    }
}

pub fn validate_theme_edge_tokens(
    tokens: &ThemeEdgeTokens,
    contract: &ThemeContract,
) -> Result<(), String> {
    let families = [
        EdgeStyleFamily::Semantic,
        EdgeStyleFamily::Traversal,
        EdgeStyleFamily::Containment,
        EdgeStyleFamily::Arrangement,
        EdgeStyleFamily::Imported,
    ];
    for family in families {
        if !tokens.family_tokens.contains_key(&family) {
            return Err(format!("missing family edge token for {family:?}"));
        }
    }

    if contract.require_non_color_family_distinction || contract.require_monochrome_preservation {
        let mut signatures = BTreeSet::new();
        for family in families {
            let token = tokens.family_tokens[&family];
            if !signatures.insert((token.pattern, token.end_marker)) {
                return Err(format!(
                    "family edge signature collision for {family:?}; pattern+marker must remain unique"
                ));
            }
        }
    }

    for (index, family) in families.iter().enumerate() {
        let a = edge_family_luminance(tokens.family_tokens[family]);
        for other in families.iter().skip(index + 1) {
            let b = edge_family_luminance(tokens.family_tokens[other]);
            if (a - b).abs() < contract.min_family_luminance_delta {
                return Err(format!(
                    "family luminance delta for {family:?} and {other:?} below {:.1}",
                    contract.min_family_luminance_delta
                ));
            }
        }
    }

    Ok(())
}

fn edge_family_luminance(token: ThemeEdgeFamilyToken) -> f32 {
    let [r, g, b, _] = token.color.to_array();
    0.2126 * f32::from(r) + 0.7152 * f32::from(g) + 0.0722 * f32::from(b)
}

#[derive(Debug, Clone, Copy)]
struct EdgeThemePalette {
    semantic: Srgb,
    selection: Srgb,
    traversal: Srgb,
    containment: Srgb,
    arrangement: Srgb,
    imported: Srgb,
    hover_halo: Srgb,
    selection_halo: Srgb,
}

#[cfg(test)]
mod tests;
