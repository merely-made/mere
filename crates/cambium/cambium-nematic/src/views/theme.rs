/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Theming for smolweb views.
//!
//! The theme and its palette are tabard's ([`tabard::smolweb`]): a per-site
//! palette by default, or the `Plain`, `Light`, `Dark`, `App` and `System`
//! presets. A view emits semantic elements with classes; [`stylesheet`]
//! renders the palette as the CSS for those classes. The host applies it the
//! way it applies any document stylesheet (the retained Livery scripted lane
//! collects inline sheets before cascade).

pub use tabard::smolweb::{SmolwebPalette, SmolwebTheme};

/// The CSS for a smolweb document under `theme`. `site_url` seeds the per-site
/// palette for [`SmolwebTheme::Site`] (ignored by the fixed and app themes).
pub fn stylesheet(theme: SmolwebTheme, site_url: &str) -> String {
    render_css(&SmolwebPalette::for_theme(&theme, site_url))
}

fn render_css(p: &SmolwebPalette) -> String {
    let SmolwebPalette {
        bg,
        fg,
        link,
        quote,
        pre_bg,
    } = p;
    format!(
        ".gemtext {{ background:{bg}; color:{fg}; padding:1.5rem 2rem; \
line-height:1.5; font-family:serif; max-width:48rem; }}
.gemtext-h1 {{ font-size:1.8rem; font-weight:700; margin:1.2rem 0 0.6rem; }}
.gemtext-h2 {{ font-size:1.4rem; font-weight:700; margin:1.1rem 0 0.5rem; }}
.gemtext-h3 {{ font-size:1.15rem; font-weight:700; margin:1rem 0 0.4rem; }}
.gemtext-text {{ margin:0.5rem 0; }}
.gemtext-linkline {{ margin:0.25rem 0; }}
.gemtext-link {{ color:{link}; text-decoration:none; }}
.gemtext-link:hover {{ text-decoration:underline; }}
.gemtext-list {{ margin:0.5rem 0; padding-left:1.5rem; }}
.gemtext-item {{ margin:0.15rem 0; }}
.gemtext-quote {{ margin:0.6rem 0; padding:0.2rem 0 0.2rem 1rem; \
border-left:3px solid {quote}; color:{quote}; font-style:italic; }}
.gemtext-pre {{ background:{pre_bg}; padding:0.75rem 1rem; overflow-x:auto; \
font-family:monospace; white-space:pre; margin:0.6rem 0; }}
.gopher {{ background:{bg}; color:{fg}; padding:1.5rem 2rem; line-height:1.5; \
font-family:serif; max-width:48rem; }}
.gopher-info {{ background:{pre_bg}; font-family:monospace; white-space:pre; \
overflow-x:auto; padding:0.5rem 1rem; margin:0.4rem 0; }}
.gopher-error {{ color:{quote}; font-style:italic; margin:0.25rem 0; }}
.gopher-itemline {{ margin:0.2rem 0; }}
.gopher-type {{ color:{quote}; font-family:monospace; font-size:0.85em; \
margin-right:0.5rem; }}
.gopher-link {{ color:{link}; text-decoration:none; }}
.gopher-link:hover {{ text-decoration:underline; }}
.feed {{ background:{bg}; color:{fg}; padding:1.5rem 2rem; line-height:1.5; \
font-family:serif; max-width:48rem; }}
.feed-title {{ font-size:1.8rem; font-weight:700; margin:0.5rem 0 0.2rem; }}
.feed-subtitle {{ color:{quote}; margin:0 0 1rem; }}
.feed-entry {{ border-top:1px solid {pre_bg}; padding:0.9rem 0; }}
.feed-entry-title {{ font-size:1.2rem; font-weight:600; margin:0 0 0.2rem; }}
.feed-entry-link {{ color:{link}; text-decoration:none; }}
.feed-entry-link:hover {{ text-decoration:underline; }}
.feed-entry-date {{ color:{quote}; font-size:0.85em; }}
.feed-entry-summary {{ margin:0.3rem 0 0; }}
"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_render_their_classes() {
        for theme in [SmolwebTheme::Plain, SmolwebTheme::Light, SmolwebTheme::Dark] {
            let css = stylesheet(theme, "");
            assert!(css.contains(".gemtext-link"));
            assert!(css.contains(".gemtext-pre"));
        }
    }

    #[test]
    fn app_theme_uses_the_host_palette() {
        let palette = SmolwebPalette {
            bg: "#102030".into(),
            fg: "#fafafa".into(),
            link: "#33ccff".into(),
            quote: "#99aabb".into(),
            pre_bg: "#0a1622".into(),
        };
        let css = stylesheet(SmolwebTheme::App(palette.clone()), "gemini://x.test/");
        // The host colours appear verbatim, and the site is ignored.
        assert!(css.contains("#102030"), "uses the host background");
        assert!(css.contains("color:#33ccff"), "uses the host link colour");
        assert_eq!(
            css,
            stylesheet(SmolwebTheme::App(palette), "gemini://elsewhere.test/"),
            "App does not derive a per-site palette"
        );
    }
}
