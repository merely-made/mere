// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Attributed Djot containers. Domain attributes remain opaque and editable;
//! parsing a container never grants it evaluation or source-write authority.
use super::*;
pub use illume::blocks::{Div, divs, update_opening, wrap};
use std::collections::BTreeMap;

pub(super) fn parse(source: &str) -> (Vec<Block>, Vec<DocumentDiagnostic>) {
    super::structure::parse(source)
}

/// Write a generic div without assigning semantics to its attributes. The
/// outer fence is longer than any colon run in its body, so nesting is stable.
pub fn emit(attributes: &BTreeMap<String, String>, blocks: &[Block], out: &mut String) {
    let body = blocks_to_djot(blocks);
    out.push_str(&wrap(attributes, &body));
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

    #[test]
    fn opening_update_changes_only_target_declaration_and_keeps_parent_prefixes() {
        let source = "> {#card owner=human}\n> ::: panel\n> α\n> :::\n";
        let target = divs(source).remove(0);
        let updated = update_opening(
            source,
            &target,
            &BTreeMap::from([
                ("id".into(), "card".into()),
                ("owner".into(), "human".into()),
                ("status".into(), "ready".into()),
            ]),
        )
        .unwrap();
        assert!(
            updated.starts_with("> {id=\"card\" owner=\"human\" status=\"ready\"}\n> ::: panel\n"),
            "updated source: {updated:?}; target range: {:?}, body start: {}",
            target.range,
            target.body.start
        );
        assert!(updated.ends_with("> α\n> :::\n"));

        let source = "> ::: panel\n> α\n> :::\n";
        let target = divs(source).remove(0);
        let updated = update_opening(
            source,
            &target,
            &BTreeMap::from([("label".into(), "quoted".into())]),
        )
        .unwrap();
        assert!(updated.starts_with("> {label=\"quoted\"}\n> ::: panel\n"));
        assert!(updated.ends_with("> α\n> :::\n"));
    }

    #[test]
    fn opening_update_rejects_forged_targets_and_multiline_values() {
        let source = "::: panel\nα\n:::\n";
        let target = divs(source).remove(0);
        let mut forged = target.clone();
        forged.attributes.insert("id".into(), "other".into());
        assert!(update_opening(source, &forged, &BTreeMap::new()).is_err());
        assert!(
            update_opening(
                source,
                &target,
                &BTreeMap::from([("label".into(), "one\ntwo".into())]),
            )
            .is_err()
        );
    }

    #[test]
    fn opening_update_uses_parser_range_for_braces_inside_quoted_json() {
        let source = r#"{#card json="{\"quote\":\"a } { value\"}" custom="keep"}
::: panel
body { bytes } α
:::
"#;
        let target = divs(source).remove(0);
        let mut attributes = target.attributes.clone();
        attributes.insert("status".into(), "ready".into());
        let updated = update_opening(source, &target, &attributes).unwrap();
        let updated_target = divs(&updated).remove(0);
        assert_eq!(updated_target.attributes, attributes);
        assert_eq!(&updated[updated_target.body.clone()], "body { bytes } α\n");
        assert!(updated.contains("custom=\"keep\""));
    }
}
