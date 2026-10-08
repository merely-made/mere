/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Syntax-highlighted text fields (the `highlight` feature).
//!
//! This is the assembly point that joins a lexer ([`illume`]) to a palette
//! ([`tinct`]) over Genet's styled field, so any Genet host gets highlighted
//! djot/code/entity editing for free — the omnibar, a note editor, a chat line,
//! a script prompt. The two libraries stay independent of each other; Cambium
//! is the one place allowed to know both, mapping illume's fine-grained
//! [`SyntaxKind`] (what was lexed) onto tinct's canonical [`SyntaxRole`] (what
//! colour), then naming a `syntax-*` CSS class a host stylesheet themes.
//!
//! Two halves, kept apart so colours stay themeable the Genet/stylo way:
//! - [`note_styles`] / [`entity_styles`] produce the `(range, class)`
//!   [`StyleRange`]s the styled field paints;
//! - [`syntax_css`] derives the actual colours from a theme's seeds (perceptual,
//!   contrast-gated) as `.syntax-* { color }` rules for the host stylesheet.
//!
//! The host picks the mode per surface with [`highlighted_textarea`] (multi-line
//! notes: djot structure + code injection + entities) and
//! [`highlighted_text_field`] (single-line: entities only, for an omnibar).
//! [`highlighted_code`] reuses those runs for read-only code with a local palette
//! and [`SYNTAX_HIGHLIGHT_CSS`], without editor or focus behavior.

use illume::{Span, SyntaxKind, default_pack, entities, highlight};
use tinct::{
    ModeProfile, Seeds, SyntaxPalette, SyntaxRole, derive_syntax_palette,
    derive_syntax_palette_with,
};

use crate::styled_field::{StyleRange, styled_text_children, styled_text_field, styled_textarea};
use crate::{El, FieldChild, TextInput, el};

/// Which passes to run over a surface's text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Highlight {
    /// A full note: djot structure + polyglot injection (illume's default pack)
    /// plus inline prose entities (urls, mentions, tags).
    Note,
    /// Inline prose entities only (urls, mentions, tags, emails) — for a single-line
    /// surface like the omnibar, which is not a djot document.
    Entities,
}

/// Map an illume lexer kind onto a tinct highlight role. `None` for kinds that
/// carry no colour of their own: a code/raw block region (its inner tokens are
/// coloured instead), bare identifiers, and the few structural kinds without a role.
fn syntax_role(kind: SyntaxKind) -> Option<SyntaxRole> {
    use SyntaxKind as K;
    use SyntaxRole as R;
    Some(match kind {
        K::Heading => R::Heading,
        K::Emphasis => R::Emphasis,
        K::Strong => R::Strong,
        K::Verbatim => R::Verbatim,
        K::Link | K::Image => R::Link,
        K::Blockquote => R::Quote,
        K::Keyword => R::Keyword,
        K::StringLit => R::String,
        K::Number => R::Number,
        K::Comment => R::Comment,
        K::Function => R::Function,
        K::Type => R::Type,
        K::Punctuation => R::Punctuation,
        K::Url | K::Email => R::Url,
        K::Mention => R::Mention,
        K::Tag => R::Tag,
        K::Strikethrough
        | K::Mark
        | K::Math
        | K::CodeBlock
        | K::RawBlock
        | K::Div
        | K::Identifier => return None,
    })
}

// Keep role classes and the local palette stylesheet in one vocabulary.
macro_rules! syntax_classes {
    ($($role:ident => $name:literal),+ $(,)?) => {
        /// The CSS class a host stylesheet themes for `role`.
        pub fn role_class(role: SyntaxRole) -> &'static str {
            match role {
                $(SyntaxRole::$role => concat!("syntax-", $name),)+
            }
        }

        /// Local palette rules for [`highlighted_code`]. Append this to the host
        /// stylesheet; each code view supplies its own inherited color variables.
        pub const SYNTAX_HIGHLIGHT_CSS: &str = concat!(
            ".syntax-highlight { white-space: pre; font-family: monospace; }\n",
            $(".syntax-highlight .syntax-", $name,
              " { color: var(--syntax-", $name, "); }\n",)+
        );
    };
}

syntax_classes! {
    Heading => "heading",
    Emphasis => "emphasis",
    Strong => "strong",
    Link => "link",
    Quote => "quote",
    Verbatim => "verbatim",
    Keyword => "keyword",
    Type => "type",
    Function => "function",
    String => "string",
    Number => "number",
    Comment => "comment",
    Punctuation => "punctuation",
    Url => "url",
    Mention => "mention",
    Tag => "tag",
}

/// Push a span's themed [`StyleRange`] onto `out`, skipping kinds with no role.
fn push_styled(out: &mut Vec<StyleRange>, span: Span) {
    if let Some(role) = syntax_role(span.kind) {
        out.push(StyleRange {
            range: span.range,
            class: role_class(role).to_string(),
        });
    }
}

/// The styled ranges for a full note: djot structure + polyglot injection
/// (illume's default pack) plus inline prose entities, each mapped to its themed
/// class. The styled field paints these; unmapped kinds are skipped.
pub fn note_styles(text: &str) -> Vec<StyleRange> {
    let registry = default_pack();
    let mut styles = Vec::new();
    for span in highlight(text, &registry) {
        push_styled(&mut styles, span);
    }
    for span in entities(text) {
        push_styled(&mut styles, span);
    }
    styles
}

/// Highlight code using Illume's curated language pack. Language labels are
/// case-insensitive; an unknown language yields no styles and renders plain.
pub fn code_styles(text: &str, language: &str) -> Vec<StyleRange> {
    let mut styles = Vec::new();
    for span in default_pack().lex(language, text).unwrap_or_default() {
        push_styled(&mut styles, span);
    }
    styles
}

/// A read-only code view using the same lexer, role classes, and styled runs as
/// highlighted editors. It installs no editor handlers or focus target.
///
/// Append [`SYNTAX_HIGHLIGHT_CSS`] to the host stylesheet. The view carries its
/// palette locally, so adjacent specimens can show different modes without
/// changing global editor colors. Unknown language labels preserve plain text.
pub fn highlighted_code<State: 'static, Action: 'static>(
    text: &str,
    language: &str,
    palette: &SyntaxPalette,
) -> El<Vec<FieldChild<State, Action>>, State, Action> {
    let mut style = format!(
        "background-color: rgb({}, {}, {}); color: rgb({}, {}, {});",
        palette.surface.r,
        palette.surface.g,
        palette.surface.b,
        palette.emphasis.r,
        palette.emphasis.g,
        palette.emphasis.b,
    );
    for role in SyntaxRole::ALL {
        let color = palette.role(role);
        use std::fmt::Write;
        // `role_class` is also the variable suffix, avoiding a second role map.
        let _ = write!(
            style,
            " --{}: rgb({}, {}, {});",
            role_class(role),
            color.r,
            color.g,
            color.b
        );
    }
    el(
        "pre",
        styled_text_children(text, &code_styles(text, language)),
    )
    .attr("class", "syntax-highlight")
    .attr("data-language", language)
    .attr("style", style)
}

/// The styled ranges for a single-line surface (an omnibar): only the inline
/// entities (urls, mentions, tags, emails). Unlike [`note_styles`] it runs no djot
/// structure pass (the omnibar is not a note).
pub fn entity_styles(text: &str) -> Vec<StyleRange> {
    let mut styles = Vec::new();
    for span in entities(text) {
        push_styled(&mut styles, span);
    }
    styles
}

/// The styled ranges for `text` under `mode`.
pub fn styles_for(text: &str, mode: Highlight) -> Vec<StyleRange> {
    match mode {
        Highlight::Note => note_styles(text),
        Highlight::Entities => entity_styles(text),
    }
}

/// A multi-line text field that highlights its buffer under `mode` — the styled
/// [`textarea`](crate::textarea) with the lexer wired in. The host recomputes the
/// styles from the buffer at view-build; pair with [`syntax_css`] in the host
/// stylesheet to colour the classes.
pub fn highlighted_textarea(input: &TextInput, mode: Highlight) -> crate::TextField {
    styled_textarea(input, &styles_for(input.text(), mode))
}

/// A single-line text field that highlights its buffer under `mode` — the styled
/// [`text_field`](crate::text_field) sibling of [`highlighted_textarea`], for an
/// omnibar or other one-line input.
pub fn highlighted_text_field(input: &TextInput, mode: Highlight) -> crate::TextField {
    styled_text_field(input, &styles_for(input.text(), mode))
}

/// `.syntax-* { color }` rules colouring the highlight classes from tinct's derived
/// syntax palette, one per role, for the host stylesheet. Derived from the active
/// theme's `seeds` (perceptual, contrast-gated against the surface), so the syntax
/// colours track a theme switch; the styled field's spans carry the classes these
/// rules theme.
pub fn syntax_css(seeds: &Seeds) -> Vec<String> {
    palette_css(&derive_syntax_palette(seeds))
}

/// Highlight colors derived against an explicitly selected surface mode,
/// including Tinct's stronger high-contrast target. Use this when the surface
/// mode differs from the authored seed preference.
pub fn syntax_css_with(seeds: &Seeds, mode: ModeProfile) -> Vec<String> {
    palette_css(&derive_syntax_palette_with(seeds, mode))
}

fn palette_css(palette: &SyntaxPalette) -> Vec<String> {
    SyntaxRole::ALL
        .iter()
        .map(|&role| {
            let c = palette.role(role);
            format!(
                ".{} {{ color: rgb({}, {}, {}); }}",
                role_class(role),
                c.r,
                c.g,
                c.b
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeds() -> Seeds {
        Seeds {
            primary: tinct::Srgb::rgb(0x33, 0x66, 0xC8),
            secondary: tinct::Srgb::rgb(0x2E, 0x9D, 0xA6),
            tertiary: tinct::Srgb::rgb(0xE0, 0xA8, 0x46),
            neutral: tinct::Srgb::rgb(0x10, 0x14, 0x22),
            text_header: None,
            text_body: None,
            success: tinct::Srgb::rgb(0x4F, 0xB3, 0x6E),
            danger: tinct::Srgb::rgb(0xD5, 0x4E, 0x4E),
            dark: true,
        }
    }

    #[test]
    fn note_styles_highlight_structure_and_entities() {
        let styles = note_styles("# Title\n\nsee @ada and https://ex.com\n");
        let classes: Vec<&str> = styles.iter().map(|s| s.class.as_str()).collect();
        assert!(classes.contains(&"syntax-heading"), "{classes:?}");
        assert!(classes.contains(&"syntax-mention"), "{classes:?}");
        assert!(classes.contains(&"syntax-url"), "{classes:?}");
    }

    #[test]
    fn entity_styles_skip_djot_structure() {
        // A leading `#` is a tag here (entities pass), not a heading (no djot pass).
        let styles = entity_styles("visit https://ex.com or @ada #web");
        let classes: Vec<&str> = styles.iter().map(|s| s.class.as_str()).collect();
        assert!(classes.contains(&"syntax-url"), "{classes:?}");
        assert!(classes.contains(&"syntax-mention"), "{classes:?}");
        assert!(classes.contains(&"syntax-tag"), "{classes:?}");
        assert!(!classes.contains(&"syntax-heading"), "{classes:?}");
    }

    #[test]
    fn code_styles_use_the_registered_rust_lexer() {
        let code = "// café\nfn main() { let answer = 42; let title = \"hello\"; }";
        let styles = code_styles(code, "RUST");
        for (class, expected) in [
            ("syntax-comment", "// café"),
            ("syntax-keyword", "fn"),
            ("syntax-keyword", "let"),
            ("syntax-number", "42"),
            ("syntax-string", "\"hello\""),
        ] {
            assert!(
                styles.iter().any(|style| {
                    style.class == class && &code[style.range.clone()] == expected
                }),
                "missing {class} over {expected:?}: {styles:?}"
            );
        }
        assert_eq!(styles, code_styles(code, "rs"));
        assert!(code_styles(code, "unknown-language").is_empty());
    }

    #[test]
    fn explicit_mode_css_matches_each_tinct_role() {
        let seeds = seeds();
        for mode in [
            ModeProfile::LIGHT,
            ModeProfile::DARK,
            ModeProfile::HC_LIGHT,
            ModeProfile::HC_DARK,
        ] {
            let palette = derive_syntax_palette_with(&seeds, mode);
            let rules = syntax_css_with(&seeds, mode);
            assert_eq!(rules.len(), SyntaxRole::ALL.len());
            for (role, rule) in SyntaxRole::ALL.into_iter().zip(rules) {
                let color = palette.role(role);
                assert_eq!(
                    rule,
                    format!(
                        ".{} {{ color: rgb({}, {}, {}); }}",
                        role_class(role),
                        color.r,
                        color.g,
                        color.b,
                    )
                );
            }
        }
    }

    #[test]
    fn legacy_css_preserves_seed_mode_derivation() {
        for dark in [false, true] {
            let seeds = Seeds { dark, ..seeds() };
            assert_eq!(
                syntax_css(&seeds),
                palette_css(&derive_syntax_palette(&seeds))
            );
        }
    }

    #[test]
    fn highlighted_code_mounts_plain_text_and_local_palette_without_editing() {
        use crate::{DomHandle, runner::GenetAppRunner};
        use genet_scripted_dom::ScriptedDom;
        use layout_dom_api::{LayoutDom, LocalName, Namespace};
        use std::cell::RefCell;
        use std::rc::Rc;

        const SOURCE: &str = "let label = \"café 🦀\";\n// readable source\n";
        let palette = derive_syntax_palette_with(&seeds(), ModeProfile::HC_DARK);
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let runner = GenetAppRunner::new(
            dom.clone(),
            move |_: &()| highlighted_code::<(), ()>(SOURCE, "rust", &palette),
            (),
        );
        let dom = runner.dom();
        let dom = dom.borrow();
        let empty = Namespace::from("");
        let attr = |name: &str| dom.attribute(runner.root(), &empty, &LocalName::from(name));
        assert_eq!(attr("class"), Some("syntax-highlight"));
        assert_eq!(attr("tabindex"), None);
        assert_eq!(attr("contenteditable"), None);
        assert_eq!(attr("data-language"), Some("rust"));
        let style = attr("style").unwrap();
        assert!(style.contains(&format!(
            "background-color: rgb({}, {}, {});",
            palette.surface.r, palette.surface.g, palette.surface.b
        )));
        for role in SyntaxRole::ALL {
            let color = palette.role(role);
            assert!(style.contains(&format!(
                "--{}: rgb({}, {}, {});",
                role_class(role),
                color.r,
                color.g,
                color.b
            )));
            assert!(SYNTAX_HIGHLIGHT_CSS.contains(&format!(
                ".syntax-highlight .{} {{ color: var(--{}); }}",
                role_class(role),
                role_class(role)
            )));
        }
        let mut rendered = String::new();
        for kid in dom.dom_children(runner.root()) {
            if let Some(text) = dom.text(kid) {
                rendered.push_str(text);
            } else {
                for run in dom.dom_children(kid) {
                    rendered.push_str(dom.text(run).unwrap());
                }
            }
        }
        assert_eq!(rendered, SOURCE);
    }

    #[test]
    fn every_role_has_a_distinct_class() {
        let mut seen = std::collections::HashSet::new();
        for role in SyntaxRole::ALL {
            assert!(
                seen.insert(role_class(role)),
                "duplicate class for {role:?}"
            );
        }
    }
}
