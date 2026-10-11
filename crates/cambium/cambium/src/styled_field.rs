/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The field-rendering layer: one style-aware field body shared by the plain
//! [`text_field`](crate::text_field) / [`textarea`](crate::textarea) and the
//! highlighting [`styled_textarea`].
//!
//! It renders a [`TextInput`]'s text as the children of the field element:
//! unstyled runs as text nodes, styled runs as `<span class="…">` runs a host's
//! stylesheet themes, with the IME preedit and ghost-completion spans spliced at
//! the caret. The plain field passes no styles (the empty case), so there is one
//! body, not a styled fork of the plain one.
//!
//! Styling carries a *class*, not inline CSS, so the host themes the highlight
//! through one stylesheet (the colours derive from tinct's syntax palette). The
//! runs concatenate to the same text, so the host's `caret_rect` lines up exactly
//! as over the plain field (which is already several inline nodes: text, the
//! preedit span, text, the ghost span). Style ranges are byte ranges over the same
//! buffer the host highlighted. Invalid bounds, including those that split UTF-8
//! characters, are ignored.

use std::ops::Range;

use crate::controls::{TextInput, edit, edit_multiline};
use crate::pod::GenetElement;
use crate::{AnyView, GenetCtx, KeyEvent, el, on_key};

/// A styled run over the field text: a byte `range` painted with a CSS `class`
/// (`"syntax-keyword"`) the host's stylesheet themes. Ranges may overlap and nest
/// (a heading containing emphasis, a code block containing a keyword);
/// The field builder flattens them innermost-wins into non-overlapping runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyleRange {
    pub range: Range<usize>,
    pub class: String,
}

/// One child of a field element: a text node or a styled `<span>`, type-erased so
/// the field's children are a uniform `Vec`.
///
/// The defaults name the editable-field shape. A read-only projection such as
/// Turnstone's omnibar mirror supplies its own parent `State` and `Action` while
/// reusing the same runs, preedit, ghost, and caret rendering.
pub type FieldChild<State = TextInput, Action = ()> =
    Box<dyn AnyView<State, Action, GenetCtx, GenetElement>>;

/// Flatten possibly-overlapping `styles` over `len` bytes into non-overlapping
/// runs, the innermost (smallest) range winning on overlap. Runs cover `0..len`
/// with no gaps; ranges with invalid bounds are ignored. A `None` class is
/// unstyled text. Empty `styles` yields a single
/// `None` run (the plain field).
fn flatten(text: &str, styles: &[StyleRange]) -> Vec<(Range<usize>, Option<String>)> {
    let len = text.len();
    // Paint per byte, largest range first so a smaller (inner) range overwrites it.
    let mut ordered: Vec<&StyleRange> = styles
        .iter()
        .filter(|s| {
            s.range.start < s.range.end
                && s.range.end <= len
                && text.is_char_boundary(s.range.start)
                && text.is_char_boundary(s.range.end)
        })
        .collect();
    ordered.sort_by_key(|s| std::cmp::Reverse(s.range.end - s.range.start));
    let mut paint: Vec<Option<&str>> = vec![None; len];
    for s in ordered {
        for b in s.range.clone() {
            paint[b] = Some(s.class.as_str());
        }
    }
    // Coalesce adjacent bytes carrying the same class into one run.
    let mut runs = Vec::new();
    let mut i = 0;
    while i < len {
        let class = paint[i];
        let start = i;
        while i < len && paint[i] == class {
            i += 1;
        }
        runs.push((start..i, class.map(str::to_string)));
    }
    runs
}

/// Emit the `runs` clipped to `[lo, hi)` over `text`: a styled run as a
/// `<span class="…">`, an unstyled run as a bare text node.
fn emit<State: 'static, Action: 'static>(
    kids: &mut Vec<FieldChild<State, Action>>,
    text: &str,
    runs: &[(Range<usize>, Option<String>)],
    lo: usize,
    hi: usize,
) {
    for (r, class) in runs {
        let start = r.start.max(lo);
        let end = r.end.min(hi);
        if start >= end {
            continue;
        }
        let slice = text[start..end].to_string();
        match class {
            Some(c) => kids.push(Box::new(
                el::<_, State, Action>("span", slice).attr("class", c.clone()),
            )),
            None => kids.push(Box::new(slice)),
        }
    }
}

/// Render committed text as styled runs without installing an editor.
///
/// This shares the same overlap handling as editable fields but has no caret,
/// IME, ghost completion, or input routing. Ranges outside the text or whose
/// bounds split a UTF-8 character are ignored; every source byte is preserved.
pub fn styled_text_children<State: 'static, Action: 'static>(
    text: &str,
    styles: &[StyleRange],
) -> Vec<FieldChild<State, Action>> {
    let runs = flatten(text, styles);
    let mut kids = Vec::new();
    emit(&mut kids, text, &runs, 0, text.len());
    kids
}

/// The children of a field element: the committed text as (styled) runs split at
/// the caret to splice the IME preedit (an underlined span), then the ghost suffix.
/// Empty `styles` renders the plain field (unstyled text nodes); non-empty paints
/// the highlight classes. This is the one body behind the plain and styled fields.
pub(crate) fn field_children(input: &TextInput, styles: &[StyleRange]) -> Vec<FieldChild> {
    field_children_impl::<TextInput, ()>(input, styles, false)
}

/// The class on the in-flow caret span a [`caret_text_field`] renders, for the
/// host's stylesheet to theme (colour, weight — e.g. `.field-caret { color: … }`).
pub const FIELD_CARET_CLASS: &str = "field-caret";

/// The class carried by an in-progress IME preedit span.
pub const FIELD_PREEDIT_CLASS: &str = "field-preedit";

/// The one field body, with or without a rendered caret. `show_caret` splices a
/// `<span class="field-caret">▍</span>` at the caret split, *after* the preedit
/// (composition text lands before the caret, matching platform IME fields).
fn field_children_impl<State: 'static, Action: 'static>(
    input: &TextInput,
    styles: &[StyleRange],
    show_caret: bool,
) -> Vec<FieldChild<State, Action>> {
    let text = input.text();
    let (before, preedit, after) = input.render_parts();
    let start = before.len();
    let end = text.len() - after.len();
    let runs = flatten(text, styles);

    let mut kids: Vec<FieldChild<State, Action>> = Vec::new();
    emit(&mut kids, text, &runs, 0, start);
    let preedit_caret = input
        .composition()
        .and_then(|composition| composition.selection.map(|(_, focus)| focus))
        .unwrap_or(preedit.len())
        .min(preedit.len());
    let preedit_caret = (0..=preedit_caret)
        .rev()
        .find(|&byte| preedit.is_char_boundary(byte))
        .unwrap_or(0);
    if !preedit[..preedit_caret].is_empty() {
        kids.push(Box::new(
            el::<_, State, Action>("span", preedit[..preedit_caret].to_owned())
                .attr("class", FIELD_PREEDIT_CLASS)
                .attr("style", "text-decoration-line: underline;"),
        ));
    }
    if show_caret {
        kids.push(Box::new(
            el::<_, State, Action>("span", "▍").attr("class", FIELD_CARET_CLASS),
        ));
    }
    if !preedit[preedit_caret..].is_empty() {
        kids.push(Box::new(
            el::<_, State, Action>("span", preedit[preedit_caret..].to_owned())
                .attr("class", FIELD_PREEDIT_CLASS)
                .attr("style", "text-decoration-line: underline;"),
        ));
    }
    emit(&mut kids, text, &runs, end, text.len());
    let ghost = input.ghost();
    if !ghost.is_empty() {
        kids.push(Box::new(
            el::<_, State, Action>("span", ghost.to_string())
                .attr("style", "color: #8b91a0; font-style: italic;"),
        ));
    }
    kids
}

/// Render the children of a caret-bearing field without installing an editor.
///
/// This is the controlled/mirror counterpart to [`caret_text_field`]. The
/// caller owns the semantic container and all input routing; Cambium supplies
/// only the exact text, style, IME-preedit, ghost, and caret projection.
pub fn caret_field_children<State: 'static, Action: 'static>(
    input: &TextInput,
    styles: &[StyleRange],
) -> Vec<FieldChild<State, Action>> {
    field_children_impl::<State, Action>(input, styles, true)
}

/// A multi-line text field rendered with per-range syntax highlighting from
/// `styles` (the [`textarea`](crate::textarea) sibling that paints a host's
/// classes). Same `edit_multiline` handler and caret / IME behaviour as the plain
/// field; only the rendering carries the classes. The host recomputes `styles`
/// from the buffer (for example at view build) and passes them in.
pub fn styled_textarea(input: &TextInput, styles: &[StyleRange]) -> crate::TextField {
    on_key(
        el::<_, TextInput, ()>("div", field_children(input, styles))
            .attr("role", "textbox")
            .attr("aria-multiline", "true")
            .attr("data-cambium-text-value", input.text()),
        edit_multiline as fn(&mut TextInput, KeyEvent),
    )
}

/// A single-line text field with per-range highlighting from `styles` — the
/// [`text_field`](crate::text_field) sibling (the `edit` handler and single-line
/// textbox metadata). Same caret / IME behaviour as the plain field; only the rendering carries
/// the classes. Lets a host highlight the omnibar (urls, command tokens) the way the
/// editor highlights a note.
pub fn styled_text_field(input: &TextInput, styles: &[StyleRange]) -> crate::TextField {
    on_key(
        el::<_, TextInput, ()>("div", field_children(input, styles))
            .attr("role", "textbox")
            .attr("data-cambium-text-value", input.text()),
        edit as fn(&mut TextInput, KeyEvent),
    )
}

/// A single-line field that renders its own caret: the `▍` glyph as an in-flow
/// `<span class="field-caret">` at the caret split (after any preedit), themed by
/// the host through [`FIELD_CARET_CLASS`]. For hosts that paint the whole scene
/// from the DOM (no overlay layer) — turnstone's omnibar is the first consumer.
///
/// Trade-off vs [`text_field`](crate::text_field): the glyph is in flow, so the
/// runs no longer concatenate to exactly the committed text and a `caret_rect`
/// overlay would land a glyph-width off past the split. A host draws the caret
/// one way or the other — pick this constructor *instead of* an overlay, not
/// alongside one. Focus-conditional display is the host's call too: build with
/// this constructor when focused, [`styled_text_field`] when not.
pub fn caret_text_field(input: &TextInput, styles: &[StyleRange]) -> crate::TextField {
    on_key(
        el::<_, TextInput, ()>("div", caret_field_children(input, styles))
            .attr("role", "textbox")
            .attr("data-cambium-text-value", input.text()),
        edit as fn(&mut TextInput, KeyEvent),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatten_empty_styles_is_one_unstyled_run() {
        assert_eq!(flatten("abcde", &[]), vec![(0..5, None)]);
    }

    #[test]
    fn flatten_innermost_wins() {
        // Outer 0..10 "a" with inner 2..5 "b": the inner range overrides.
        let styles = vec![
            StyleRange {
                range: 0..10,
                class: "a".into(),
            },
            StyleRange {
                range: 2..5,
                class: "b".into(),
            },
        ];
        assert_eq!(
            flatten("abcdefghij", &styles),
            vec![
                (0..2, Some("a".into())),
                (2..5, Some("b".into())),
                (5..10, Some("a".into())),
            ]
        );
    }

    #[test]
    fn flatten_drops_out_of_range_styles() {
        let styles = vec![StyleRange {
            range: 3..99,
            class: "x".into(),
        }];
        // end past len is filtered, leaving a plain run.
        assert_eq!(flatten("abcd", &styles), vec![(0..4, None)]);
    }

    #[test]
    fn flatten_ignores_ranges_that_split_utf8_and_preserves_overlapping_text() {
        let text = "aé🦀z";
        let styles = vec![
            StyleRange {
                range: 0..text.len(),
                class: "outer".into(),
            },
            StyleRange {
                range: 1..7,
                class: "inner".into(),
            },
            StyleRange {
                range: 2..5,
                class: "invalid".into(),
            },
            StyleRange {
                range: 6..99,
                class: "outside".into(),
            },
            StyleRange {
                range: Range { start: 7, end: 1 },
                class: "reversed".into(),
            },
        ];
        let runs = flatten(text, &styles);
        assert_eq!(
            runs,
            vec![
                (0..1, Some("outer".into())),
                (1..7, Some("inner".into())),
                (7..8, Some("outer".into())),
            ]
        );
        let rendered: String = runs.iter().map(|(range, _)| &text[range.clone()]).collect();
        assert_eq!(rendered, text);
        assert!(flatten("", &styles).is_empty());
    }

    #[test]
    fn readonly_styled_children_preserve_utf8_text_with_no_field_artifacts() {
        use crate::{DomHandle, runner::GenetAppRunner};
        use genet_scripted_dom::ScriptedDom;
        use layout_dom_api::{LayoutDom, LocalName, Namespace};
        use std::cell::RefCell;
        use std::rc::Rc;

        const SOURCE: &str = "aé🦀z\n";
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let runner = GenetAppRunner::new(
            dom.clone(),
            |_: &()| {
                el(
                    "pre",
                    styled_text_children::<(), ()>(
                        SOURCE,
                        &[
                            StyleRange {
                                range: 0..SOURCE.len(),
                                class: "outer".into(),
                            },
                            StyleRange {
                                range: 1..7,
                                class: "inner".into(),
                            },
                            StyleRange {
                                range: 2..5,
                                class: "invalid".into(),
                            },
                        ],
                    ),
                )
            },
            (),
        );
        let dom = runner.dom();
        let dom = dom.borrow();
        let mut rendered = String::new();
        let mut classes = Vec::new();
        for kid in dom.dom_children(runner.root()) {
            classes.push(
                dom.attribute(kid, &Namespace::from(""), &LocalName::from("class"))
                    .unwrap()
                    .to_owned(),
            );
            for text in dom.dom_children(kid) {
                rendered.push_str(dom.text(text).unwrap());
            }
        }
        assert_eq!(rendered, SOURCE);
        assert_eq!(classes, vec!["outer", "inner", "outer"]);
    }

    /// The caret field mounts with the `▍` span *at the split*: text-before,
    /// `<span class="field-caret">`, text-after — the in-flow caret a
    /// scene-painting host renders instead of an overlay.
    #[test]
    fn caret_field_renders_the_caret_span_at_the_split() {
        use std::cell::RefCell;
        use std::rc::Rc;

        use genet_scripted_dom::ScriptedDom;
        use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};

        use crate::runner::GenetAppRunner;
        use crate::{DomHandle, TextField};

        let mut input = TextInput::default();
        input.insert_str("abcd");
        input.set_caret_byte(2, false);

        fn view(s: &TextInput) -> TextField {
            super::caret_text_field(s, &[])
        }
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let runner = GenetAppRunner::new(dom.clone(), view, input);

        let dom = runner.dom();
        let dom = dom.borrow();
        let kids: Vec<_> = dom.dom_children(runner.root()).collect();
        assert_eq!(kids.len(), 3, "text-before, caret span, text-after");
        assert_eq!(dom.text(kids[0]), Some("ab"));
        assert_eq!(dom.kind(kids[1]), NodeKind::Element);
        assert_eq!(
            dom.attribute(kids[1], &Namespace::from(""), &LocalName::from("class")),
            Some(super::FIELD_CARET_CLASS)
        );
        assert_eq!(dom.text(kids[2]), Some("cd"));
    }

    #[test]
    fn textbox_metadata_contains_committed_text_while_visual_runs_keep_transients() {
        use std::cell::RefCell;
        use std::rc::Rc;

        use genet_scripted_dom::ScriptedDom;
        use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};

        use crate::runner::GenetAppRunner;
        use crate::{DomHandle, TextField};

        let mut input = TextInput::new("abc");
        input.set_caret_byte(1, false);
        input.set_preedit("XY");
        input.set_ghost("ghost");
        fn view(input: &TextInput) -> TextField {
            super::styled_text_field(
                input,
                &[StyleRange {
                    range: 0..3,
                    class: "syntax-token".into(),
                }],
            )
        }
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let runner = GenetAppRunner::new(dom.clone(), view, input);
        let dom = runner.dom();
        let dom = dom.borrow();
        let field = runner.root();
        assert_eq!(
            dom.attribute(field, &Namespace::from(""), &LocalName::from("role")),
            Some("textbox")
        );
        assert_eq!(
            dom.attribute(
                field,
                &Namespace::from(""),
                &LocalName::from("data-cambium-text-value")
            ),
            Some("abc"),
            "preedit and ghost text do not enter the accessible committed value"
        );
        let children: Vec<_> = dom.dom_children(field).collect();
        assert_eq!(
            children.len(),
            4,
            "highlighted before/after, preedit, ghost"
        );
        assert_eq!(dom.kind(children[0]), NodeKind::Element);
        assert_eq!(
            dom.attribute(children[0], &Namespace::from(""), &LocalName::from("class")),
            Some("syntax-token")
        );
        assert_eq!(
            dom.attribute(children[1], &Namespace::from(""), &LocalName::from("class")),
            Some(FIELD_PREEDIT_CLASS)
        );
        assert_eq!(
            dom.dom_children(children[1])
                .filter_map(|child| dom.text(child))
                .collect::<String>(),
            "XY"
        );
        assert_eq!(
            dom.dom_children(children[3])
                .filter_map(|child| dom.text(child))
                .collect::<String>(),
            "ghost"
        );
        fn all_text(dom: &ScriptedDom, node: genet_scripted_dom::NodeId, out: &mut String) {
            if dom.kind(node) == NodeKind::Text {
                out.push_str(dom.text(node).unwrap_or(""));
            }
            for child in dom.dom_children(node) {
                all_text(dom, child, out);
            }
        }
        let mut rendered = String::new();
        all_text(&dom, field, &mut rendered);
        assert_eq!(rendered, "aXYbcghost");
    }

    #[test]
    fn multiline_textbox_exposes_multiline_semantics() {
        use std::cell::RefCell;
        use std::rc::Rc;

        use genet_scripted_dom::ScriptedDom;
        use layout_dom_api::{LayoutDom, LocalName, Namespace};

        use crate::runner::GenetAppRunner;
        use crate::{DomHandle, TextField};

        fn view(input: &TextInput) -> TextField {
            super::styled_textarea(input, &[])
        }
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let runner = GenetAppRunner::new(dom.clone(), view, TextInput::new("first\nsecond"));
        let dom = runner.dom();
        let dom = dom.borrow();
        let field = runner.root();
        assert_eq!(
            dom.attribute(field, &Namespace::from(""), &LocalName::from("role")),
            Some("textbox")
        );
        assert_eq!(
            dom.attribute(
                field,
                &Namespace::from(""),
                &LocalName::from("aria-multiline")
            ),
            Some("true")
        );
        assert_eq!(
            dom.attribute(
                field,
                &Namespace::from(""),
                &LocalName::from("data-cambium-text-value")
            ),
            Some("first\nsecond")
        );
    }
}
