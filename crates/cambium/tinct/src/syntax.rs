// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Perceptual syntax-highlight palette: a contrast-gated colour per highlight
//! role, derived from the same [`Seeds`] the base [`Palette`] uses.
//!
//! A host's lexer (the `illume` text lexer, say) emits its own token kinds; the
//! host maps each onto a [`SyntaxRole`] here, and the colour is derived. The role
//! set is small and canonical on purpose, so any highlighter fits and the
//! derivation stays coherent: accents are OKLCH hue-steps fanned off the brand
//! primary at a shared chroma, each nudged in lightness until it clears WCAG
//! contrast against the surface; muted roles ride the base palette's dim text;
//! emphasis carries no own hue (the host applies weight / italic). Reseed the
//! theme and the whole syntax palette rotates with the brand.

use crate::oklch::Oklch;
use crate::{ModeProfile, Palette, Seeds, Srgb, best_on, contrast, derive_palette_with};

/// A canonical highlight role. A host maps its lexer's finer token kinds onto
/// these, and [`derive_syntax_palette`] gives each a themed colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SyntaxRole {
    // Prose / document structure.
    Heading,
    Emphasis,
    Strong,
    Link,
    Quote,
    Verbatim,
    // Code tokens.
    Keyword,
    Type,
    Function,
    String,
    Number,
    Comment,
    Punctuation,
    // Inline entities (any prose, the omnibar included).
    Url,
    Mention,
    Tag,
}

impl SyntaxRole {
    /// Every role, in declaration order.
    pub const ALL: [SyntaxRole; 16] = [
        Self::Heading,
        Self::Emphasis,
        Self::Strong,
        Self::Link,
        Self::Quote,
        Self::Verbatim,
        Self::Keyword,
        Self::Type,
        Self::Function,
        Self::String,
        Self::Number,
        Self::Comment,
        Self::Punctuation,
        Self::Url,
        Self::Mention,
        Self::Tag,
    ];

    /// The role's lowercase key, as token and stylesheet names use it.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Heading => "heading",
            Self::Emphasis => "emphasis",
            Self::Strong => "strong",
            Self::Link => "link",
            Self::Quote => "quote",
            Self::Verbatim => "verbatim",
            Self::Keyword => "keyword",
            Self::Type => "type",
            Self::Function => "function",
            Self::String => "string",
            Self::Number => "number",
            Self::Comment => "comment",
            Self::Punctuation => "punctuation",
            Self::Url => "url",
            Self::Mention => "mention",
            Self::Tag => "tag",
        }
    }
}

/// A themed colour per [`SyntaxRole`], plus the surface it was gated against (so a
/// host can re-check or blend). Look a role up with [`SyntaxPalette::role`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SyntaxPalette {
    pub surface: Srgb,
    pub heading: Srgb,
    pub emphasis: Srgb,
    pub strong: Srgb,
    pub link: Srgb,
    pub quote: Srgb,
    pub verbatim: Srgb,
    pub keyword: Srgb,
    pub type_: Srgb,
    pub function: Srgb,
    pub string: Srgb,
    pub number: Srgb,
    pub comment: Srgb,
    pub punctuation: Srgb,
    pub url: Srgb,
    pub mention: Srgb,
    pub tag: Srgb,
}

impl SyntaxPalette {
    /// The colour for `role`.
    pub fn role(&self, role: SyntaxRole) -> Srgb {
        match role {
            SyntaxRole::Heading => self.heading,
            SyntaxRole::Emphasis => self.emphasis,
            SyntaxRole::Strong => self.strong,
            SyntaxRole::Link => self.link,
            SyntaxRole::Quote => self.quote,
            SyntaxRole::Verbatim => self.verbatim,
            SyntaxRole::Keyword => self.keyword,
            SyntaxRole::Type => self.type_,
            SyntaxRole::Function => self.function,
            SyntaxRole::String => self.string,
            SyntaxRole::Number => self.number,
            SyntaxRole::Comment => self.comment,
            SyntaxRole::Punctuation => self.punctuation,
            SyntaxRole::Url => self.url,
            SyntaxRole::Mention => self.mention,
            SyntaxRole::Tag => self.tag,
        }
    }
}

/// WCAG contrast floor a syntax colour must clear against the surface.
const MIN_CONTRAST: f64 = 4.5;
/// High-contrast syntax roles use WCAG's enhanced text contrast floor.
const HC_MIN_CONTRAST: f64 = 7.0;
/// Shared chroma for the fanned accents (readable saturation, not neon).
const ACCENT_C: f64 = 0.13;

/// Nudge `col`'s lightness toward the text end until it clears `minimum`
/// against `surface` (or it hits the lightness rail), then return sRGB. Measures
/// contrast on the post-gamut-clamp colour, so out-of-gamut accents still gate.
fn gate(mut col: Oklch, surface: Srgb, dark: bool, minimum: f64) -> Srgb {
    for _ in 0..40 {
        if contrast(col.to_srgb(), surface) >= minimum {
            break;
        }
        col = if dark {
            col.lighten(0.02)
        } else {
            col.darken(0.02)
        };
        if !(0.04..=0.96).contains(&col.l) {
            break;
        }
    }
    col.to_srgb()
}

/// Retain the authored/gated colour whenever it already clears the floor.
/// Extreme seed chroma can exhaust the lightness rail after gamut clipping;
/// use the existing contrast-picked text helper, then a pure extreme if needed.
fn gated_text(color: Srgb, surface: Srgb, dark: bool, minimum: f64) -> Srgb {
    if contrast(color, surface) >= minimum {
        return color;
    }
    let adjusted = gate(Oklch::from_srgb(color), surface, dark, minimum);
    if contrast(adjusted, surface) >= minimum {
        return adjusted;
    }
    let text = best_on(surface);
    if contrast(text, surface) >= minimum {
        return text;
    }
    if contrast(Srgb::WHITE, surface) >= contrast(Srgb::BLACK, surface) {
        Srgb::WHITE
    } else {
        Srgb::BLACK
    }
}

/// Derive a contrast-gated [`SyntaxPalette`] from the seeds. Accents fan off the
/// brand primary's hue at [`ACCENT_C`] and a per-mode base lightness, each gated
/// against the derived surface; muted roles ride the base palette's dim text;
/// emphasis / strong carry no own hue (the host applies weight).
pub fn derive_syntax_palette(seeds: &Seeds) -> SyntaxPalette {
    derive(seeds, ModeProfile::from_seeds(seeds), false)
}

/// Derive syntax colours for the exact explicit base-palette profile.
///
/// Every role is gated against [`SyntaxPalette::surface`], which equals
/// [`derive_palette_with`] for this profile. The contrast floor is 4.5:1 at
/// normal contrast and 7:1 at high contrast, including muted roles and authored
/// body/header overrides. `mode`, rather than `Seeds::dark`, chooses the scheme.
/// [`derive_syntax_palette`] retains its original normal-contrast behaviour,
/// including using authored body/header and derived dim colours unchanged.
pub fn derive_syntax_palette_with(seeds: &Seeds, mode: ModeProfile) -> SyntaxPalette {
    derive(seeds, mode, true)
}

fn derive(seeds: &Seeds, mode: ModeProfile, enforce_floor: bool) -> SyntaxPalette {
    let base: Palette = derive_palette_with(seeds, mode);
    let surface = base.surface;
    let dark = mode.dark;
    let minimum = if mode.high_contrast {
        HC_MIN_CONTRAST
    } else {
        MIN_CONTRAST
    };
    let base_l = if dark { 0.74 } else { 0.46 };
    let primary_h = Oklch::from_srgb(seeds.primary).h;
    let text = |color: Srgb| {
        if enforce_floor {
            gated_text(color, surface, dark, minimum)
        } else {
            color
        }
    };
    let accent_gate = |color: Oklch| {
        let color = gate(color, surface, dark, minimum);
        text(color)
    };

    // An accent `offset` degrees off the primary hue: shared chroma + base
    // lightness, gated for contrast against the surface.
    let accent = |offset: f64| -> Srgb {
        let col = Oklch {
            l: base_l,
            c: ACCENT_C,
            h: primary_h,
        }
        .rotate_hue(offset);
        accent_gate(col)
    };

    SyntaxPalette {
        surface,
        // Structure: heading is the brand primary, prominent; emphasis / strong
        // ride the text tiers (weight + italic carry them); quote is dimmed.
        heading: accent_gate(
            Oklch::from_srgb(seeds.primary)
                .with_c(ACCENT_C * 1.2)
                .with_l(base_l),
        ),
        emphasis: text(base.text),
        strong: text(base.text_header),
        link: accent(-40.0),
        quote: text(base.text_dim),
        verbatim: accent(180.0),
        // Code accents, fanned around the wheel so kinds stay distinguishable.
        keyword: accent(0.0),
        function: accent(40.0),
        type_: accent(80.0),
        string: accent(150.0),
        number: accent(210.0),
        comment: text(base.text_dim),
        punctuation: text(base.text_dim),
        // Inline entities: links / urls share the navigational hue; mention + tag
        // take their own.
        url: accent(-40.0),
        mention: accent(115.0),
        tag: accent(245.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeds() -> Seeds {
        Seeds {
            primary: Srgb::rgb(0x33, 0x66, 0xC8),
            secondary: Srgb::rgb(0x2E, 0x9D, 0xA6),
            tertiary: Srgb::rgb(0xE0, 0xA8, 0x46),
            neutral: Srgb::rgb(0x10, 0x14, 0x22),
            text_header: None,
            text_body: None,
            success: Srgb::rgb(0x4F, 0xB3, 0x6E),
            danger: Srgb::rgb(0xD5, 0x4E, 0x4E),
            dark: true,
        }
    }

    #[test]
    fn fanned_accents_clear_contrast() {
        let pal = derive_syntax_palette(&seeds());
        for role in [
            SyntaxRole::Heading,
            SyntaxRole::Keyword,
            SyntaxRole::Type,
            SyntaxRole::Function,
            SyntaxRole::String,
            SyntaxRole::Number,
            SyntaxRole::Link,
            SyntaxRole::Url,
            SyntaxRole::Mention,
            SyntaxRole::Tag,
            SyntaxRole::Verbatim,
        ] {
            let c = contrast(pal.role(role), pal.surface);
            assert!(c >= 4.0, "{role:?} contrast {c:.2} too low against surface");
        }
    }

    #[test]
    fn accents_are_distinct() {
        let pal = derive_syntax_palette(&seeds());
        assert_ne!(pal.keyword, pal.string);
        assert_ne!(pal.string, pal.number);
        assert_ne!(pal.keyword, pal.function);
        assert_ne!(pal.mention, pal.tag);
    }

    #[test]
    fn role_lookup_matches_fields() {
        let pal = derive_syntax_palette(&seeds());
        assert_eq!(pal.role(SyntaxRole::Keyword), pal.keyword);
        assert_eq!(pal.role(SyntaxRole::Tag), pal.tag);
        assert_eq!(pal.role(SyntaxRole::Type), pal.type_);
    }

    #[test]
    fn light_mode_also_gates() {
        let mut s = seeds();
        s.dark = false;
        let pal = derive_syntax_palette(&s);
        assert!(contrast(pal.keyword, pal.surface) >= 4.0);
        assert!(contrast(pal.string, pal.surface) >= 4.0);
    }

    #[test]
    fn all_covers_every_role() {
        // ALL and `role` agree, so a host can build a full lookup table.
        let pal = derive_syntax_palette(&seeds());
        for role in SyntaxRole::ALL {
            let _ = pal.role(role);
        }
        assert_eq!(SyntaxRole::ALL.len(), 16);
    }

    #[test]
    fn explicit_profiles_gate_against_the_exact_rendered_surface() {
        let seeds = seeds();
        let modes = [
            ModeProfile::LIGHT,
            ModeProfile::DARK,
            ModeProfile::HC_LIGHT,
            ModeProfile::HC_DARK,
        ];
        let mut surfaces = Vec::new();
        for mode in modes {
            let syntax = derive_syntax_palette_with(&seeds, mode);
            let palette = derive_palette_with(&seeds, mode);
            assert_eq!(syntax.surface, palette.surface, "{mode:?}");
            let minimum = if mode.high_contrast {
                HC_MIN_CONTRAST
            } else {
                MIN_CONTRAST
            };
            for role in SyntaxRole::ALL {
                let ratio = contrast(syntax.role(role), syntax.surface);
                assert!(
                    ratio >= minimum,
                    "{mode:?} {role:?} contrast {ratio:.3} below {minimum}"
                );
            }
            let opposite_seeds = Seeds {
                dark: !seeds.dark,
                ..seeds
            };
            assert_eq!(
                syntax,
                derive_syntax_palette_with(&opposite_seeds, mode),
                "explicit profile must own the scheme"
            );
            assert!(
                !surfaces.contains(&syntax.surface),
                "{mode:?} must have its own derived surface"
            );
            surfaces.push(syntax.surface);
        }
    }

    #[test]
    fn adversarial_chroma_and_text_overrides_clear_each_profiles_floor() {
        let colors = [
            Srgb::BLACK,
            Srgb::WHITE,
            Srgb::GRAY,
            Srgb::rgb(255, 0, 0),
            Srgb::rgb(0, 255, 0),
            Srgb::rgb(0, 0, 255),
            Srgb::rgb(255, 0, 255),
            Srgb::rgb(255, 255, 0),
            Srgb::rgb(0, 255, 255),
        ];
        for neutral in colors {
            for primary in colors {
                for mode in [
                    ModeProfile::LIGHT,
                    ModeProfile::DARK,
                    ModeProfile::HC_LIGHT,
                    ModeProfile::HC_DARK,
                ] {
                    let mut adversarial = Seeds {
                        neutral,
                        primary,
                        ..seeds()
                    };
                    let surface = derive_palette_with(&adversarial, mode).surface;
                    // Explicit authored overrides are allowed by the base
                    // palette. Syntax text must still remain legible.
                    adversarial.text_body = Some(surface);
                    adversarial.text_header = Some(surface);
                    let syntax = derive_syntax_palette_with(&adversarial, mode);
                    assert_eq!(syntax.surface, surface);
                    let minimum = if mode.high_contrast {
                        HC_MIN_CONTRAST
                    } else {
                        MIN_CONTRAST
                    };
                    for role in SyntaxRole::ALL {
                        let ratio = contrast(syntax.role(role), surface);
                        assert!(
                            ratio >= minimum,
                            "{mode:?}, neutral {neutral:?}, primary {primary:?}, {role:?}: {ratio:.3} below {minimum}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn legacy_api_retains_normal_text_tiers_and_existing_accent_derivation() {
        for dark in [false, true] {
            let seeds = Seeds { dark, ..seeds() };
            let legacy = derive_syntax_palette(&seeds);
            let explicit = derive_syntax_palette_with(&seeds, ModeProfile::from_seeds(&seeds));
            // Default normal palettes already clear the stronger API's floor,
            // so the entire existing result is unchanged for those seeds.
            assert_eq!(legacy, explicit);
            let authored = Seeds {
                text_body: Some(legacy.surface),
                text_header: Some(legacy.surface),
                ..seeds
            };
            let legacy = derive_syntax_palette(&authored);
            let base = derive_palette_with(&authored, ModeProfile::from_seeds(&authored));
            assert_eq!(legacy.emphasis, base.text);
            assert_eq!(legacy.strong, base.text_header);
            assert_eq!(legacy.quote, base.text_dim);
            assert_eq!(legacy.comment, base.text_dim);
            assert_eq!(legacy.punctuation, base.text_dim);
            assert_eq!(legacy.keyword, explicit.keyword);
            assert_eq!(legacy.heading, explicit.heading);
        }
    }
}
