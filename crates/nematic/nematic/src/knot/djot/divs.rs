// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Attributed Djot containers. Domain attributes remain opaque and editable;
//! parsing a container never grants it evaluation or source-write authority.
use super::*;
use std::{collections::BTreeMap, ops::Range};

/// A source-bound div. Ranges include its attribute/opening and closing fences;
/// the body range excludes them. Nested members keep their own source ranges.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Div {
    pub range: Range<usize>,
    pub body: Range<usize>,
    pub attributes: BTreeMap<String, String>,
    pub children: Vec<Div>,
}

/// Source structure, independent of domain rules or the current presentation.
pub fn divs(source: &str) -> Vec<Div> {
    let mut roots = Vec::new();
    let mut stack: Vec<Div> = Vec::new();
    let mut pending = None;
    for (event, range) in Parser::new(source).into_offset_iter() {
        match event {
            Event::Attributes(_) => pending = Some(range.start),
            Event::Start(Container::Div { class }, attrs) => {
                let mut attributes: BTreeMap<_, _> = attrs
                    .unique_pairs()
                    .map(|(key, value)| (key.to_string(), value.to_string()))
                    .collect();
                if !class.is_empty() {
                    let classes = attributes.entry("class".into()).or_default();
                    if !classes
                        .split_whitespace()
                        .any(|value| value == class.as_ref())
                    {
                        if !classes.is_empty() {
                            classes.push(' ');
                        }
                        classes.push_str(&class);
                    }
                }
                stack.push(Div {
                    range: pending.take().unwrap_or(range.start)..source.len(),
                    body: range.end..source.len(),
                    attributes,
                    children: Vec::new(),
                });
            },
            Event::End(Container::Div { .. }) => {
                if let Some(mut div) = stack.pop() {
                    div.body.end = range.start.max(div.body.start);
                    div.range.end = range.end.max(div.body.end);
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(div);
                    } else {
                        roots.push(div);
                    }
                }
                pending = None;
            },
            _ => pending = None,
        }
    }
    roots
}

pub(super) fn parse(source: &str) -> (Vec<Block>, Vec<DocumentDiagnostic>) {
    super::structure::parse(source)
}

/// Write a generic div without assigning semantics to its attributes. The
/// outer fence is longer than any colon run in its body, so nesting is stable.
pub fn emit(attributes: &BTreeMap<String, String>, blocks: &[Block], out: &mut String) {
    let body = blocks_to_djot(blocks);
    out.push_str(&wrap(attributes, &body));
}

/// Wrap authored source without reparsing/reformatting its contents. Useful for
/// applying domain declarations to existing text through an ordinary text edit.
pub fn wrap(attributes: &BTreeMap<String, String>, body: &str) -> String {
    let fence = ":".repeat(
        body.lines()
            .map(|line| line.trim_start().chars().take_while(|c| *c == ':').count())
            .max()
            .unwrap_or(0)
            .saturating_add(1)
            .max(3),
    );
    let mut out = String::new();
    if !attributes.is_empty() {
        out.push('{');
        for (i, (key, value)) in attributes.iter().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            out.push_str(key);
            out.push_str("=\"");
            out.push_str(&value.replace('\\', "\\\\").replace('"', "\\\""));
            out.push('"');
        }
        out.push_str("}\n");
    }
    out.push_str(&fence);
    out.push('\n');
    out.push_str(body);
    if !body.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&fence);
    out.push('\n');
    out
}

pub(super) fn emit_inline(spans: &[InlineSpan], out: &mut String) {
    for span in spans {
        match span {
            InlineSpan::Text(text) => {
                for c in text.chars() {
                    if "\\[]{}_*`<>".contains(c) {
                        out.push('\\');
                    }
                    out.push(c);
                }
            },
            InlineSpan::Link {
                url,
                title,
                predicate,
                spans,
            } => {
                out.push('[');
                emit_inline(spans, out);
                out.push_str("](");
                out.push_str(&url.replace('\\', "\\\\").replace(')', "\\)"));
                out.push(')');
                let mut attrs = BTreeMap::new();
                if let Some(value) = title {
                    attrs.insert("title", value);
                }
                if let Some(value) = predicate {
                    attrs.insert("rel", value);
                }
                if !attrs.is_empty() {
                    out.push('{');
                    for (i, (key, value)) in attrs.iter().enumerate() {
                        if i > 0 {
                            out.push(' ');
                        }
                        out.push_str(key);
                        out.push_str("=\"");
                        out.push_str(&value.replace('\\', "\\\\").replace('"', "\\\""));
                        out.push('"');
                    }
                    out.push('}');
                }
            },
            InlineSpan::Presented { spans, .. } => emit_inline(spans, out),
            InlineSpan::LineBreak => out.push_str("\\\n"),
            InlineSpan::SoftBreak => out.push('\n'),
            _ => out.push_str(&inline_text(std::slice::from_ref(span))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_domain_attributes_and_content_round_trip_without_interpretation() {
        let source = "before\n\n{#stanza template=\"local:form\" authored=\"u /\"}\n:::: metre\nfirst [source](https://example.test/)\n\n{#phrase units=beats}\n::: progression\nI IV V\n:::\n::::\n\nafter\n";
        let (blocks, diagnostics) = parse(source);
        assert!(diagnostics.is_empty());
        assert_eq!(parse(&blocks_to_djot(&blocks)).0, blocks);
        let Block::Div {
            attributes,
            blocks: children,
        } = &blocks[1]
        else {
            panic!("container lost")
        };
        assert_eq!(attributes["id"], "stanza");
        assert_eq!(attributes["template"], "local:form");
        assert!(matches!(&children[1], Block::Div { .. }));
        let tree = divs(source);
        assert_eq!(&source[tree[0].children[0].body.clone()], "I IV V\n");
        assert!(source[tree[0].range.clone()].contains("#stanza"));
    }
    #[test]
    fn wrapper_preserves_authored_body_and_escaped_attributes() {
        let attrs = BTreeMap::from([
            ("class".into(), "metre".into()),
            ("label".into(), "say \"hello\" \\ here".into()),
        ]);
        let body = "α  β\n\n::: child\ntext\n:::\n";
        let source = wrap(&attrs, body);
        let tree = divs(&source);
        assert_eq!(tree[0].attributes, attrs);
        assert_eq!(&source[tree[0].body.clone()], body);
    }
}
