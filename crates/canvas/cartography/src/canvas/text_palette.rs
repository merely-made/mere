// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The text palette: a face's eight entries clamped to read against the
//! surface a line of text sits on (iconography T2).
//!
//! On the canvas a face sits on its own tile; in a tab, a row or a sentence it
//! sits on the page, and an entry near the page's lightness vanishes. So a
//! text palette is built per background. Each entry keeps its OKLCH hue and
//! moves its lightness only, to the nearest value holding at least
//! [`TEXT_FACE_CONTRAST`] against every given ground; chroma drops only where
//! the new lightness is outside the sRGB gamut at the old chroma.
//!
//! One ground always admits such a lightness: against a ground of luminance
//! `Y`, white reaches `1.05 / (Y + 0.05)` and black `(Y + 0.05) / 0.05`, and
//! the larger is never under 4.58. Several grounds at once (a row's surface,
//! its hover and its selection) may admit none; that entry takes the lightness
//! with the best worst-case contrast and is marked [`outlined`], and its host
//! draws a one-pixel outline in the text colour around the face.
//!
//! [`outlined`]: TextFacePalette::outlined

use tincture::{contrast, oklch::Oklch};

use super::palette::{DERIVED_FACE_COLOR_COUNT, DerivedFacePalette, ThemeColor};

/// The floor every entry holds against its grounds: WCAG 1.4.11's 3:1 for
/// graphical objects, the same as tinct's `MARK_FLOOR`.
pub const TEXT_FACE_CONTRAST: f64 = 3.0;

/// Lightness steps searched between 0 and 1.
const STEPS: u32 = 400;
/// Chroma bisection rounds when a lightness is out of gamut at full chroma.
const CHROMA_ROUNDS: u32 = 24;

/// A [`DerivedFacePalette`] clamped for one set of grounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextFacePalette {
    colors: [[u8; 4]; DERIVED_FACE_COLOR_COUNT],
    outlined: [bool; DERIVED_FACE_COLOR_COUNT],
}

impl TextFacePalette {
    /// Clamp `faces` to hold [`TEXT_FACE_CONTRAST`] against every ground.
    /// Alpha is kept. With no grounds the palette is unchanged.
    pub fn new(faces: DerivedFacePalette, grounds: &[ThemeColor]) -> Self {
        let mut colors = faces.colors();
        let mut outlined = [false; DERIVED_FACE_COLOR_COUNT];
        for (color, outline) in colors.iter_mut().zip(&mut outlined) {
            let [r, g, b, a] = *color;
            let (clamped, reached) = clamp(ThemeColor::rgb(r, g, b), grounds);
            *color = clamped.with_alpha(a).to_array();
            *outline = !reached;
        }
        Self { colors, outlined }
    }

    /// The straight-alpha sRGB entries, in pictograph slot order.
    pub const fn colors(self) -> [[u8; 4]; DERIVED_FACE_COLOR_COUNT] {
        self.colors
    }

    /// Which entries could not reach the floor against every ground. A face
    /// using one of them is drawn with a one-pixel outline in the text colour.
    pub const fn outlined(self) -> [bool; DERIVED_FACE_COLOR_COUNT] {
        self.outlined
    }

    /// The same as [`DerivedFacePalette`], for a renderer that takes one.
    pub const fn faces(self) -> DerivedFacePalette {
        DerivedFacePalette::new(self.colors)
    }
}

/// The worst contrast of `color` against `grounds`.
fn worst(color: ThemeColor, grounds: &[ThemeColor]) -> f64 {
    grounds
        .iter()
        .map(|&ground| contrast(color, ground))
        .fold(f64::INFINITY, f64::min)
}

/// `color` at the nearest lightness holding the floor, and whether one did.
fn clamp(color: ThemeColor, grounds: &[ThemeColor]) -> (ThemeColor, bool) {
    if worst(color, grounds) >= TEXT_FACE_CONTRAST {
        return (color, true);
    }
    let start = Oklch::from_srgb(color);
    let mut nearest: Option<(f64, ThemeColor)> = None;
    let mut best = (f64::NEG_INFINITY, color);
    for step in 0..=STEPS {
        let lightness = f64::from(step) / f64::from(STEPS);
        let candidate = at_lightness(start, lightness);
        let score = worst(candidate, grounds);
        let distance = (lightness - start.l).abs();
        if score >= TEXT_FACE_CONTRAST && nearest.is_none_or(|(d, _)| distance < d) {
            nearest = Some((distance, candidate));
        }
        if score > best.0 {
            best = (score, candidate);
        }
    }
    match nearest {
        Some((_, candidate)) => (candidate, true),
        None => (best.1, false),
    }
}

/// `start`'s hue at `lightness`, with the most chroma up to its own that the
/// sRGB gamut holds there.
fn at_lightness(start: Oklch, lightness: f64) -> ThemeColor {
    let at = start.with_l(lightness);
    if in_gamut(at) {
        return at.to_srgb();
    }
    let (mut inside, mut outside) = (0.0, start.c);
    for _ in 0..CHROMA_ROUNDS {
        let middle = (inside + outside) / 2.0;
        if in_gamut(at.with_c(middle)) {
            inside = middle;
        } else {
            outside = middle;
        }
    }
    at.with_c(inside).to_srgb()
}

/// Whether `color` is inside the sRGB gamut, before tinct's conversion clamps
/// it. The matrices are OKLab's (Ottosson 2020), as in tinct.
fn in_gamut(color: Oklch) -> bool {
    const SLACK: f64 = 1e-4;
    let (a, b) = (color.c * color.h.cos(), color.c * color.h.sin());
    let l = (color.l + 0.396_337_777_4 * a + 0.215_803_757_3 * b).powi(3);
    let m = (color.l - 0.105_561_345_8 * a - 0.063_854_172_8 * b).powi(3);
    let s = (color.l - 0.089_484_177_5 * a - 1.291_485_548_0 * b).powi(3);
    [
        4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s,
        -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s,
        -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701_0 * s,
    ]
    .into_iter()
    .all(|channel| (-SLACK..=1.0 + SLACK).contains(&channel))
}

#[cfg(test)]
#[path = "text_palette/tests.rs"]
mod tests;
