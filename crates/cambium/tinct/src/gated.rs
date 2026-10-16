// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Contrast-gated roles layered on a derived [`Palette`].
//!
//! The base palette uses the brand triad as-is, which is right for fills
//! (`primary` under `on_primary`) and wrong for the same hue as text, a focus
//! ring or a mark on the canvas: oxblood on a dark surface is under 2:1. These
//! roles take a palette colour and, only where it misses its floor, mix it
//! toward white (dark grounds) or black (light grounds) until it holds against
//! both `bg` and `surface`. The floors follow the Merely design system's
//! ruling of 2026-10-16: 4.5:1 for anything read as text, 3:1 for marks and
//! focus rings (WCAG 1.4.3 and 1.4.11).
//!
//! This mirrors `tokens/tinct.js` in the Merely design system, so a theme
//! derives the same hex values in the browser specimens and in the hosts.

use serde::{Deserialize, Serialize};

use crate::oklch::Oklch;
use crate::{Palette, Srgb, contrast, mix, relative_luminance};

/// Contrast floor for text at body size (WCAG 1.4.3).
pub const TEXT_FLOOR: f64 = 4.5;
/// Contrast floor for marks, outlines and focus rings (WCAG 1.4.11).
pub const MARK_FLOOR: f64 = 3.0;

/// How far `border` mixes the body text toward the surface (decorative).
const BORDER_BLEND: f64 = 0.78;
/// How far `border_strong` mixes the body text toward the surface before
/// gating.
const BORDER_STRONG_BLEND: f64 = 0.55;
/// Steps the gate takes toward white or black before giving up.
const GATE_STEPS: u32 = 50;

/// Roles derived from a [`Palette`] by contrast gating. Each is measured
/// against both `palette.bg` and `palette.surface`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatedRoles {
    /// Primary as text or a mark (4.5:1). `primary` stays the fill.
    pub accent: Srgb,
    /// The focus ring, from tertiary (3:1). `tertiary` stays the
    /// current-state fill.
    pub focus: Srgb,
    /// Status as text (4.5:1), from `palette.success`. Always shown with a
    /// word or icon.
    pub success: Srgb,
    /// Status as text (4.5:1), from `palette.danger`. Always shown with a
    /// word or icon.
    pub danger: Srgb,
    /// Hairlines between panes, rows and cards: the body text mixed 78%
    /// toward the surface. Decorative, so not gated.
    pub border: Srgb,
    /// The outline that identifies a control (a field, a toggle, an
    /// unselected chip): the body text mixed 55% toward the surface, gated
    /// to 3:1.
    pub border_strong: Srgb,
    /// Five category colours for marks on a canvas (3:1): primary, secondary,
    /// tertiary, an open hue (see [`open_hue`]), then the dim text as a
    /// neutral fifth. No category takes a status hue.
    pub category: [Srgb; 5],
}

impl GatedRoles {
    /// The roles that have no base-palette counterpart, in emission order, as
    /// CSS custom-property suffixes (`accent`, `focus`, `border`,
    /// `border-strong`, `category-1` … `category-5`). `success` and `danger`
    /// replace the base palette's raw seeds instead.
    pub fn named(&self) -> [(&'static str, Srgb); 9] {
        [
            ("accent", self.accent),
            ("focus", self.focus),
            ("border", self.border),
            ("border-strong", self.border_strong),
            ("category-1", self.category[0]),
            ("category-2", self.category[1]),
            ("category-3", self.category[2]),
            ("category-4", self.category[3]),
            ("category-5", self.category[4]),
        ]
    }
}

/// Derive the gated roles for a palette. Needs only the palette, so a host
/// holding a stored palette (not its seeds) derives the same values.
pub fn derive_gated_roles(p: &Palette) -> GatedRoles {
    let grounds = [p.bg, p.surface];
    GatedRoles {
        accent: gate(p.primary, &grounds, TEXT_FLOOR),
        focus: gate(p.tertiary, &grounds, MARK_FLOOR),
        success: gate(p.success, &grounds, TEXT_FLOOR),
        danger: gate(p.danger, &grounds, TEXT_FLOOR),
        border: mix(p.text, p.surface, BORDER_BLEND),
        border_strong: gate(
            mix(p.text, p.surface, BORDER_STRONG_BLEND),
            &grounds,
            MARK_FLOOR,
        ),
        category: [
            gate(p.primary, &grounds, MARK_FLOOR),
            gate(p.secondary, &grounds, MARK_FLOOR),
            gate(p.tertiary, &grounds, MARK_FLOOR),
            gate(open_hue(p), &grounds, MARK_FLOOR),
            p.text_dim,
        ],
    }
}

/// Minimum chroma for the open hue, so it reads as a colour, not a grey.
const OPEN_HUE_MIN_CHROMA: f64 = 0.10;
/// Lightness candidates for the open hue; it takes the one furthest from the
/// triad's, so it differs from its neighbours in lightness as well as hue.
const OPEN_HUE_LIGHTNESS: [f64; 3] = [0.40, 0.55, 0.70];

/// The fourth category colour: the hue at the middle of the widest gap
/// between the triad and the two status hues on the OKLCH wheel, so it is
/// neither a brand colour nor a status colour. Lightness is whichever of 0.40,
/// 0.55 and 0.70 lies furthest from the triad's (the first on a tie); chroma
/// the triad's mean, at least 0.10. Not yet gated.
pub fn open_hue(p: &Palette) -> Srgb {
    use std::f64::consts::TAU;
    let triad = [p.primary, p.secondary, p.tertiary].map(Oklch::from_srgb);
    let mut hues: Vec<f64> = [p.primary, p.secondary, p.tertiary, p.success, p.danger]
        .into_iter()
        .map(|c| Oklch::from_srgb(c).h.rem_euclid(TAU))
        .collect();
    hues.sort_by(f64::total_cmp);
    let (mut best_gap, mut best_mid) = (-1.0, 0.0);
    for (i, &h) in hues.iter().enumerate() {
        let next = if i + 1 < hues.len() {
            hues[i + 1]
        } else {
            hues[0] + TAU
        };
        let gap = next - h;
        if gap > best_gap {
            best_gap = gap;
            best_mid = (h + gap / 2.0).rem_euclid(TAU);
        }
    }
    let distance = |l: f64| {
        triad
            .iter()
            .map(|o| (o.l - l).abs())
            .fold(f64::INFINITY, f64::min)
    };
    let mut l = OPEN_HUE_LIGHTNESS[0];
    for candidate in OPEN_HUE_LIGHTNESS {
        if distance(candidate) > distance(l) {
            l = candidate;
        }
    }
    let c = (triad.iter().map(|o| o.c).sum::<f64>() / 3.0).max(OPEN_HUE_MIN_CHROMA);
    Oklch { l, c, h: best_mid }.to_srgb()
}

/// `color` itself if it holds `min` contrast on every ground; otherwise
/// `color` mixed toward white (when the first ground is dark) or black (when
/// it is light) in 2% steps until it does. Falls back to pure white or black.
/// Alpha is taken from `color`.
pub fn gate(color: Srgb, grounds: &[Srgb], min: f64) -> Srgb {
    let holds = |c: Srgb| grounds.iter().all(|&g| contrast(c, g) >= min);
    if holds(color) {
        return color;
    }
    let dark_ground = grounds
        .first()
        .is_some_and(|&g| relative_luminance(g) < 0.4);
    let toward = if dark_ground {
        Srgb::rgb(0xFF, 0xFF, 0xFF)
    } else {
        Srgb::rgb(0x00, 0x00, 0x00)
    };
    for k in 1..=GATE_STEPS {
        let candidate = mix(color, toward, f64::from(k) / f64::from(GATE_STEPS));
        if holds(candidate) {
            return candidate;
        }
    }
    Srgb::rgba(toward.r, toward.g, toward.b, color.a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ModeProfile, Seeds, derive_palette_with};

    fn brand() -> Seeds {
        Seeds {
            primary: Srgb::rgb(0x6E, 0x17, 0x12),
            secondary: Srgb::rgb(0x29, 0x48, 0x6B),
            tertiary: Srgb::rgb(0xC7, 0x80, 0x2F),
            neutral: Srgb::rgb(0xF0, 0xEB, 0xDD),
            text_header: None,
            text_body: None,
            success: Srgb::rgb(0x4F, 0xB3, 0x6E),
            danger: Srgb::rgb(0xD5, 0x4E, 0x4E),
            dark: false,
        }
    }

    fn graphshell_night() -> Seeds {
        Seeds {
            primary: Srgb::rgb(0xD3, 0x9B, 0x4A),
            secondary: Srgb::rgb(0x8D, 0xB8, 0xA4),
            tertiary: Srgb::rgb(0xF0, 0xC6, 0x74),
            neutral: Srgb::rgb(0x17, 0x24, 0x2C),
            text_header: None,
            text_body: None,
            success: Srgb::rgb(0x4F, 0xB3, 0x6E),
            danger: Srgb::rgb(0xE9, 0x93, 0x87),
            dark: true,
        }
    }

    const MODES: [ModeProfile; 4] = [
        ModeProfile::LIGHT,
        ModeProfile::DARK,
        ModeProfile::HC_LIGHT,
        ModeProfile::HC_DARK,
    ];

    #[test]
    fn every_gated_role_holds_its_floor_in_every_mode() {
        for seeds in [brand(), graphshell_night()] {
            for mode in MODES {
                let p = derive_palette_with(&seeds, mode);
                let r = derive_gated_roles(&p);
                for ground in [p.bg, p.surface] {
                    assert!(contrast(r.accent, ground) >= TEXT_FLOOR, "{mode:?} accent");
                    assert!(contrast(r.focus, ground) >= MARK_FLOOR, "{mode:?} focus");
                    assert!(
                        contrast(r.success, ground) >= TEXT_FLOOR,
                        "{mode:?} success"
                    );
                    assert!(contrast(r.danger, ground) >= TEXT_FLOOR, "{mode:?} danger");
                    assert!(
                        contrast(r.border_strong, ground) >= MARK_FLOOR,
                        "{mode:?} border-strong"
                    );
                    for (i, c) in r.category[..4].iter().enumerate() {
                        assert!(
                            contrast(*c, ground) >= MARK_FLOOR,
                            "{mode:?} category-{}",
                            i + 1
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn a_colour_that_already_holds_is_unchanged() {
        let p = derive_palette_with(&brand(), ModeProfile::LIGHT);
        // Oxblood on parchment is far above 4.5:1.
        assert_eq!(derive_gated_roles(&p).accent, p.primary);
    }

    #[test]
    fn dark_grounds_gate_toward_white() {
        let p = derive_palette_with(&brand(), ModeProfile::DARK);
        let accent = derive_gated_roles(&p).accent;
        assert_ne!(accent, p.primary);
        assert!(relative_luminance(accent) > relative_luminance(p.primary));
    }

    #[test]
    fn matches_the_design_system_specimen() {
        // tokens/tinct.js in the Merely design system, light brand theme.
        let p = derive_palette_with(&brand(), ModeProfile::LIGHT);
        let r = derive_gated_roles(&p);
        assert_eq!(crate::color_to_hex(r.border), "#CBC6B9");
        assert_eq!(crate::color_to_hex(r.border_strong), "#878379");
        assert_eq!(crate::color_to_hex(r.success), "#337346");
        assert_eq!(crate::color_to_hex(r.danger), "#B34242");
        assert_eq!(crate::color_to_hex(r.category[3]), "#8B5C9A");
        let night =
            derive_gated_roles(&derive_palette_with(&graphshell_night(), ModeProfile::DARK));
        assert_eq!(crate::color_to_hex(night.category[3]), "#5E6394");
    }

    #[test]
    fn no_category_takes_a_status_hue() {
        use std::f64::consts::PI;
        for seeds in [brand(), graphshell_night()] {
            let p = derive_palette_with(&seeds, ModeProfile::LIGHT);
            let open = Oklch::from_srgb(open_hue(&p)).h;
            for status in [p.success, p.danger] {
                let d = (open - Oklch::from_srgb(status).h).rem_euclid(2.0 * PI);
                assert!(
                    d.min(2.0 * PI - d) > PI / 6.0,
                    "open hue within 30° of a status hue"
                );
            }
        }
    }

    #[test]
    fn border_is_quieter_than_border_strong() {
        for mode in MODES {
            let p = derive_palette_with(&graphshell_night(), mode);
            let r = derive_gated_roles(&p);
            assert!(contrast(r.border, p.surface) < contrast(r.border_strong, p.surface));
        }
    }
}
