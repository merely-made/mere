// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! # tinct
//!
//! Perceptual seed-to-palette derivation for the Merely family (Woodshed,
//! Strophe, Mere). A theme is authored as a handful of **seed** colours
//! (a primary/secondary/tertiary brand triad + a neutral surface hue + a
//! light/dark mode); the full set of UI roles is *derived* — surface ladder,
//! text hierarchy, and contrast-picked `on_*` colours — so adding a theme is
//! cheap and consistent. Derivation runs in **OKLCH** for perceptually-even
//! lightness steps (unlike HSL), and gates text colours on WCAG contrast.
//!
//! The crate depends on nothing but `serde` and owns its colour type
//! ([`Srgb`]) so hosts on any toolkit can use it: convert from the host
//! colour type to [`Srgb`] for the seeds, and from the derived [`Palette`]
//! back. The [`oklch`] module is public so a host can build a richer token set
//! (extra surface steps, accent variants, hue rotations) on the same maths the
//! base [`derive_palette`] uses.
//!
//! ```
//! use tinct::{Seeds, Srgb, derive_palette};
//! let seeds = Seeds {
//!     primary: Srgb::rgb(0x33, 0x66, 0xC8),
//!     secondary: Srgb::rgb(0x2E, 0x9D, 0xA6),
//!     tertiary: Srgb::rgb(0xE0, 0xA8, 0x46),
//!     neutral: Srgb::rgb(0x10, 0x14, 0x22),
//!     text_header: None,
//!     text_body: None,
//!     success: Srgb::rgb(0x4F, 0xB3, 0x6E),
//!     danger: Srgb::rgb(0xD5, 0x4E, 0x4E),
//!     dark: true,
//! };
//! let palette = derive_palette(&seeds);
//! assert!(tinct::contrast(palette.text, palette.surface) >= 4.5);
//! ```

use serde::{Deserialize, Serialize};

pub mod gated;
pub mod syntax;
pub use gated::{GatedRoles, derive_gated_roles};
pub use syntax::{SyntaxPalette, SyntaxRole, derive_syntax_palette, derive_syntax_palette_with};

// =============================================================================
// Colour type
// =============================================================================

/// An sRGB colour, 8 bits per channel, with straight (non-premultiplied) alpha.
/// The crate's own type so it depends on no host toolkit; convert at the edges
/// (`Srgb::rgba(c.r(), c.g(), c.b(), c.a())` and back).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Srgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Srgb {
    /// An opaque colour.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 0xFF }
    }

    /// A colour with explicit straight alpha.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// `[r, g, b, a]`.
    pub const fn to_array(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Same colour, replaced alpha.
    pub const fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    /// An opaque grey: the same value on all three channels.
    pub const fn gray(value: u8) -> Self {
        Self::rgb(value, value, value)
    }

    /// Opaque black.
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    /// Opaque white.
    pub const WHITE: Self = Self::rgb(0xFF, 0xFF, 0xFF);
    /// Opaque mid grey, `#808080`.
    pub const GRAY: Self = Self::rgb(0x80, 0x80, 0x80);
    /// Fully transparent black.
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
}

// =============================================================================
// Seeds + derived palette
// =============================================================================

/// The minimal authored input for a derived [`Palette`].
///
/// `primary` / `secondary` / `tertiary` are the brand triad (used as-is for
/// those roles); surfaces and text derive from `neutral`. `success` / `danger`
/// are functional hues kept distinct from the brand. `dark` runs the ladders
/// dark (low surface L, high text L) or light.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Seeds {
    pub primary: Srgb,
    pub secondary: Srgb,
    pub tertiary: Srgb,
    /// Surface/text hue. Usually a tinted near-grey — the tint carries the
    /// theme into the surface ladder rather than only the accents.
    pub neutral: Srgb,
    /// Optional explicit heading / body text colours. `None` derives them from
    /// `neutral` (legible against the surface); `Some` is used as-is. `text_body`
    /// also drives the derived dim / disabled tiers.
    pub text_header: Option<Srgb>,
    pub text_body: Option<Srgb>,
    pub success: Srgb,
    pub danger: Srgb,
    pub dark: bool,
}

/// The derived base palette — the universally-shared semantic roles. A host
/// reads these and layers product-specific colours (and any extra token tiers)
/// on top, deriving those from the same seeds via [`oklch`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Palette {
    /// The window itself (lowest surface).
    pub bg: Srgb,
    /// One elevation up (cards / panels).
    pub surface: Srgb,
    /// Two up (controls on cards).
    pub surface_2: Srgb,
    /// Hover state for a surface.
    pub surface_hover: Srgb,

    /// Titles / headings (the big type tiers).
    pub text_header: Srgb,
    /// Body + control text.
    pub text: Srgb,
    /// Secondary metadata.
    pub text_dim: Srgb,
    /// Inactive / ghost text.
    pub text_disabled: Srgb,

    pub primary: Srgb,
    pub on_primary: Srgb,
    pub secondary: Srgb,
    pub on_secondary: Srgb,
    pub tertiary: Srgb,
    pub on_tertiary: Srgb,

    pub success: Srgb,
    pub danger: Srgb,
}

/// Perceptual lightness targets (OKLCH L, 0..1) for the surface ladder + text
/// tiers, per mode. Dark: dark surfaces, light text. Light: light surfaces,
/// dark text, with `surface_2` a touch darker than `surface` for an inset look.
/// The high-contrast pairs push the surface ladder toward the extremes and the
/// text past it, so text/surface separation is measurably wider than the
/// normal-contrast counterpart (see `ModeProfile`).
struct LadderTargets {
    bg: f64,
    surface: f64,
    surface_2: f64,
    surface_hover: f64,
    text: f64,
}

impl LadderTargets {
    const DARK: Self = Self {
        bg: 0.16,
        surface: 0.21,
        surface_2: 0.26,
        surface_hover: 0.32,
        text: 0.95,
    };
    const LIGHT: Self = Self {
        bg: 0.93,
        surface: 0.97,
        surface_2: 0.86,
        surface_hover: 0.90,
        text: 0.24,
    };
    const HC_DARK: Self = Self {
        bg: 0.05,
        surface: 0.10,
        surface_2: 0.17,
        surface_hover: 0.26,
        text: 0.99,
    };
    const HC_LIGHT: Self = Self {
        bg: 0.97,
        surface: 0.995,
        surface_2: 0.90,
        surface_hover: 0.94,
        text: 0.03,
    };
}

/// A mode's derivation profile: the lightness-ladder direction plus the
/// contrast spread. This generalises `Seeds.dark` (the degenerate two-mode
/// version) to the four canonical modes — light, dark, high-contrast light,
/// high-contrast dark. High-contrast modes derive a wider text/surface
/// lightness separation and keep the dim/disabled text tiers closer to the
/// body text, so secondary text stays legible.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeProfile {
    /// Ladder direction: dark surfaces + light text, or the inverse.
    pub dark: bool,
    /// Wider lightness separation (surfaces near the extreme, text past it)
    /// and a tighter dim/disabled blend.
    pub high_contrast: bool,
}

impl ModeProfile {
    pub const LIGHT: Self = Self {
        dark: false,
        high_contrast: false,
    };
    pub const DARK: Self = Self {
        dark: true,
        high_contrast: false,
    };
    pub const HC_LIGHT: Self = Self {
        dark: false,
        high_contrast: true,
    };
    pub const HC_DARK: Self = Self {
        dark: true,
        high_contrast: true,
    };

    /// The profile a seed set's own `dark` flag encodes (normal contrast).
    pub fn from_seeds(s: &Seeds) -> Self {
        Self {
            dark: s.dark,
            high_contrast: false,
        }
    }

    fn targets(&self) -> LadderTargets {
        match (self.dark, self.high_contrast) {
            (true, false) => LadderTargets::DARK,
            (false, false) => LadderTargets::LIGHT,
            (true, true) => LadderTargets::HC_DARK,
            (false, true) => LadderTargets::HC_LIGHT,
        }
    }

    /// The dim / disabled text blend factors toward the surface. High contrast
    /// blends less, keeping the secondary tiers further from the surface.
    fn text_blend(&self) -> (f64, f64) {
        if self.high_contrast {
            (0.16, 0.38)
        } else {
            (0.32, 0.60)
        }
    }
}

/// Derive the full base [`Palette`] from [`Seeds`], honouring the seed set's
/// own `dark` flag at normal contrast. Equivalent to
/// [`derive_palette_with`]`(s, ModeProfile::from_seeds(s))`.
pub fn derive_palette(s: &Seeds) -> Palette {
    derive_palette_with(s, ModeProfile::from_seeds(s))
}

/// Derive the full base [`Palette`] from [`Seeds`] under an explicit
/// [`ModeProfile`] — the per-mode derivation entry point. The profile, not
/// `Seeds.dark`, decides the ladder direction and contrast spread, so one seed
/// set derives all four canonical modes. Surfaces + text are perceptual
/// lightness steps off `neutral`; `on_*` are contrast-picked against each
/// brand fill; dim / disabled blend the body text toward the surface so a
/// `text_body` override cascades.
pub fn derive_palette_with(s: &Seeds, mode: ModeProfile) -> Palette {
    let neutral = oklch::Oklch::from_srgb(s.neutral);
    let t = mode.targets();
    let (dim_blend, disabled_blend) = mode.text_blend();
    let step = |l: f64| neutral.with_l(l).to_srgb();
    let surface = step(t.surface);
    let body = s.text_body.unwrap_or_else(|| step(t.text));
    let header = s.text_header.unwrap_or(body);
    Palette {
        bg: step(t.bg),
        surface,
        surface_2: step(t.surface_2),
        surface_hover: step(t.surface_hover),
        text_header: header,
        text: body,
        text_dim: mix(body, surface, dim_blend),
        text_disabled: mix(body, surface, disabled_blend),
        primary: s.primary,
        on_primary: best_on(s.primary),
        secondary: s.secondary,
        on_secondary: best_on(s.secondary),
        tertiary: s.tertiary,
        on_tertiary: best_on(s.tertiary),
        success: s.success,
        danger: s.danger,
    }
}

// =============================================================================
// Contrast + blending helpers
// =============================================================================

/// Near-white or near-black, whichever has the higher WCAG contrast against
/// `bg`. Used for `on_*` text / icon colours over brand fills.
pub fn best_on(bg: Srgb) -> Srgb {
    let white = Srgb::rgb(0xF4, 0xF4, 0xF8);
    let black = Srgb::rgb(0x14, 0x14, 0x1A);
    if contrast(white, bg) >= contrast(black, bg) {
        white
    } else {
        black
    }
}

/// Straight-RGB lerp `a` to `b` by `t` (0..1). Crude but fine for blending text
/// toward a surface (dim / disabled tiers) or tinting a surface toward an
/// accent. Alpha is taken from `a`.
pub fn mix(a: Srgb, b: Srgb, t: f64) -> Srgb {
    let t = t.clamp(0.0, 1.0);
    let m = |x: u8, y: u8| ((x as f64) * (1.0 - t) + (y as f64) * t).round() as u8;
    Srgb::rgba(m(a.r, b.r), m(a.g, b.g), m(a.b, b.b), a.a)
}

/// WCAG contrast ratio between two colours (1.0 ..= 21.0). Alpha is ignored.
pub fn contrast(a: Srgb, b: Srgb) -> f64 {
    let la = relative_luminance(a);
    let lb = relative_luminance(b);
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// WCAG relative luminance of a colour (0..1). Alpha is ignored.
pub fn relative_luminance(c: Srgb) -> f64 {
    0.2126 * srgb_to_linear(c.r) + 0.7152 * srgb_to_linear(c.g) + 0.0722 * srgb_to_linear(c.b)
}

fn srgb_to_linear(c: u8) -> f64 {
    let c = c as f64 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

// =============================================================================
// Hex + HSL helpers (for theme files + colour-picker UIs)
// =============================================================================

/// Format a colour as `#RRGGBB` (alpha dropped — seeds are opaque).
pub fn color_to_hex(c: Srgb) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r, c.g, c.b)
}

/// Parse `#RRGGBB` / `RRGGBB` (case-insensitive) to an opaque colour. `None`
/// for anything that isn't six hex digits.
pub fn color_from_hex(s: &str) -> Option<Srgb> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Srgb::rgb(r, g, b))
}

/// Decompose an opaque colour into HSL — hue `0..360`, saturation + lightness
/// `0..1`. For colour-picker sliders (more intuitive than raw RGB).
pub fn color_to_hsl(c: Srgb) -> (f64, f64, f64) {
    let (r, g, b) = (c.r as f64 / 255.0, c.g as f64 / 255.0, c.b as f64 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let d = max - min;
    if d.abs() < 1e-9 {
        return (0.0, 0.0, l);
    }
    let s = d / (1.0 - (2.0 * l - 1.0).abs());
    let h = if max == r {
        ((g - b) / d).rem_euclid(6.0)
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    ((h * 60.0).rem_euclid(360.0), s, l)
}

/// Build an opaque colour from HSL (`h` in degrees, `s` / `l` in `0..1`).
pub fn color_from_hsl(h: f64, s: f64, l: f64) -> Srgb {
    let s = s.clamp(0.0, 1.0);
    let l = l.clamp(0.0, 1.0);
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = h.rem_euclid(360.0) / 60.0;
    let x = c * (1.0 - (hp.rem_euclid(2.0) - 1.0).abs());
    let (r1, g1, b1) = match hp as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    let to8 = |v: f64| ((v + m).clamp(0.0, 1.0) * 255.0).round() as u8;
    Srgb::rgb(to8(r1), to8(g1), to8(b1))
}

// =============================================================================
// OKLCH transform (Björn Ottosson's oklab), hand-rolled to avoid a colour-crate
// dependency. Public so hosts can derive extra token tiers (surface steps,
// accent variants, hue rotations) on the same maths.
// =============================================================================

pub mod oklch {
    use super::Srgb;
    use std::f64::consts::PI;

    /// A colour in OKLCH: perceptual lightness `l` (0..1), chroma `c` (0..~0.4),
    /// hue `h` in **radians**.
    #[derive(Clone, Copy, Debug)]
    pub struct Oklch {
        pub l: f64,
        pub c: f64,
        pub h: f64,
    }

    fn srgb_to_linear(c: f64) -> f64 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    fn linear_to_srgb(c: f64) -> f64 {
        if c <= 0.0031308 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        }
    }

    impl Oklch {
        pub fn from_srgb(col: Srgb) -> Self {
            let r = srgb_to_linear(col.r as f64 / 255.0);
            let g = srgb_to_linear(col.g as f64 / 255.0);
            let b = srgb_to_linear(col.b as f64 / 255.0);
            let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
            let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
            let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;
            let l_ = l.cbrt();
            let m_ = m.cbrt();
            let s_ = s.cbrt();
            let ll = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
            let aa = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
            let bb = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;
            Self {
                l: ll,
                c: (aa * aa + bb * bb).sqrt(),
                h: bb.atan2(aa),
            }
        }

        /// Same hue + chroma, new lightness (the surface-ladder primitive).
        pub fn with_l(&self, l: f64) -> Self {
            Self { l, ..*self }
        }

        /// Same hue + lightness, new chroma (mute / saturate a tone).
        pub fn with_c(&self, c: f64) -> Self {
            Self { c, ..*self }
        }

        /// Same lightness + chroma, new hue (radians).
        pub fn with_h(&self, h: f64) -> Self {
            Self { h, ..*self }
        }

        /// Lightness shifted by `dl` (clamped to 0..1).
        pub fn lighten(&self, dl: f64) -> Self {
            self.with_l((self.l + dl).clamp(0.0, 1.0))
        }

        /// Lightness shifted down by `dl`.
        pub fn darken(&self, dl: f64) -> Self {
            self.lighten(-dl)
        }

        /// Hue rotated by `degrees` (analogous / complementary accents).
        pub fn rotate_hue(&self, degrees: f64) -> Self {
            self.with_h(self.h + degrees * PI / 180.0)
        }

        pub fn to_srgb(&self) -> Srgb {
            let a = self.c * self.h.cos();
            let b = self.c * self.h.sin();
            let l_ = self.l + 0.3963377774 * a + 0.2158037573 * b;
            let m_ = self.l - 0.1055613458 * a - 0.0638541728 * b;
            let s_ = self.l - 0.0894841775 * a - 1.2914855480 * b;
            let l = l_ * l_ * l_;
            let m = m_ * m_ * m_;
            let s = s_ * s_ * s_;
            let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
            let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
            let bb = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;
            let to8 = |x: f64| (linear_to_srgb(x).clamp(0.0, 1.0) * 255.0).round() as u8;
            Srgb::rgb(to8(r), to8(g), to8(bb))
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
