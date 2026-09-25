// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Smolweb document palettes: how a gemtext, gopher or feed page is coloured.
//!
//! The default is a **per-site palette**: the document's host is hashed into a
//! hue and the whole palette derives from it, so each capsule has its own
//! consistent colour identity (the Lagrange approach), as a restrained tint that
//! keeps legibility stable. Presets override it: a neutral `Plain`, fixed
//! `Light` and `Dark`, the host application's palette (`App`), or the OS scheme
//! (`System`).
//!
//! Hosts render the palette their own way: cambium-nematic as a stylesheet for
//! its view classes, document-lanes as a document style sheet. Until
//! 2026-09-24 each defined these types and palettes itself.

/// How a smolweb document is coloured.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SmolwebTheme {
    /// Default: a palette derived from the site's host, so each capsule has its
    /// own consistent colour identity (the Lagrange approach).
    #[default]
    Site,
    /// A neutral, un-themed palette.
    Plain,
    /// A warm fixed light palette.
    Light,
    /// A fixed dark palette.
    Dark,
    /// The host application's palette. The host supplies it (e.g. derived from
    /// its tinct theme seeds), so smolweb pages match the surrounding app chrome.
    App(SmolwebPalette),
    /// Follow the OS light/dark scheme. The host resolves the OS scheme and
    /// passes the matching theme; absent that, this is light.
    System,
}

/// A document palette as CSS colour strings (`#rrggbb` or `rgb(…)`). The fixed
/// themes construct one; a host supplies its own through [`SmolwebTheme::App`]
/// (e.g. mapped from tinct), the seam that lets smolweb pages match the app
/// theme.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmolwebPalette {
    /// Page background.
    pub bg: String,
    /// Body text.
    pub fg: String,
    /// Link colour.
    pub link: String,
    /// Quote text / border, and other muted accents.
    pub quote: String,
    /// Preformatted / code-block background.
    pub pre_bg: String,
}

impl SmolwebPalette {
    /// The palette for a document at `url` under `theme`. `url` seeds the
    /// per-site palette for [`SmolwebTheme::Site`]; the fixed and app themes
    /// ignore it.
    pub fn for_theme(theme: &SmolwebTheme, url: &str) -> Self {
        match theme {
            SmolwebTheme::Site => site(url),
            SmolwebTheme::Plain => fixed("#ffffff", "#1a1a1a", "#0b57d0", "#555555", "#f4f4f4"),
            // System is host-resolved to light or dark before it reaches here.
            SmolwebTheme::Light | SmolwebTheme::System => {
                fixed("#fbfaf7", "#23211c", "#1a6e57", "#5b574e", "#f0eee8")
            },
            SmolwebTheme::Dark => fixed("#16181c", "#e6e3dc", "#7db4ff", "#a8a49a", "#21242a"),
            SmolwebTheme::App(palette) => palette.clone(),
        }
    }
}

fn fixed(bg: &str, fg: &str, link: &str, quote: &str, pre_bg: &str) -> SmolwebPalette {
    SmolwebPalette {
        bg: bg.into(),
        fg: fg.into(),
        link: link.into(),
        quote: quote.into(),
        pre_bg: pre_bg.into(),
    }
}

/// A restrained capsule tint: legibility stays stable while related pages keep
/// a little identity. Emitted as `rgb()` so any CSS-colour parser reads it.
fn site(url: &str) -> SmolwebPalette {
    let hue = hue_from_host(url);
    let css = |saturation, lightness| {
        let [r, g, b] = hsl(f32::from(hue), saturation, lightness);
        format!(
            "rgb({}, {}, {})",
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (b * 255.0).round() as u8
        )
    };
    SmolwebPalette {
        bg: css(0.22, 0.975),
        fg: css(0.20, 0.14),
        link: css(0.62, 0.34),
        quote: css(0.15, 0.38),
        pre_bg: css(0.24, 0.93),
    }
}

/// Hash the URL's host into a hue in `0..360`: djb2 over the host bytes, so a
/// capsule keeps one identity across its pages.
fn hue_from_host(url: &str) -> u16 {
    let host = url
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or("");
    let hash = host.bytes().fold(5381_u32, |hash, byte| {
        hash.wrapping_mul(33).wrapping_add(u32::from(byte))
    });
    (hash % 360) as u16
}

fn hsl(hue: f32, saturation: f32, lightness: f32) -> [f32; 3] {
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let sector = hue.rem_euclid(360.0) / 60.0;
    let x = chroma * (1.0 - (sector.rem_euclid(2.0) - 1.0).abs());
    let (r, g, b) = match sector as u8 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let m = lightness - chroma / 2.0;
    [r + m, g + m, b + m]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn site_hue_is_stable_and_host_scoped() {
        // Same host, different paths -> same palette (one identity per capsule).
        let a = SmolwebPalette::for_theme(&SmolwebTheme::Site, "gemini://example.test/a");
        let b = SmolwebPalette::for_theme(&SmolwebTheme::Site, "gemini://example.test/b/c");
        assert_eq!(a, b);
        // Different hosts -> different palettes (with overwhelming likelihood).
        let other = SmolwebPalette::for_theme(&SmolwebTheme::Site, "gemini://elsewhere.test/");
        assert_ne!(a, other);
        assert!(a.bg.starts_with("rgb("), "parseable colours: {}", a.bg);
    }

    #[test]
    fn default_is_site() {
        assert_eq!(SmolwebTheme::default(), SmolwebTheme::Site);
    }

    #[test]
    fn app_passes_through_and_system_falls_back_to_light() {
        let palette = fixed("#102030", "#fafafa", "#33ccff", "#99aabb", "#0a1622");
        let app = SmolwebTheme::App(palette.clone());
        assert_eq!(SmolwebPalette::for_theme(&app, "gemini://x.test/"), palette);
        assert_eq!(
            SmolwebPalette::for_theme(&SmolwebTheme::System, ""),
            SmolwebPalette::for_theme(&SmolwebTheme::Light, "")
        );
    }
}
