// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Preserve structural parents around attributed containers. Inline/semantic
//! leaf lowering reuses the existing Djot implementation, not another grammar.
use super::*;
use std::collections::BTreeMap;

enum Kind {
    Root,
    Div(BTreeMap<String, String>),
    Quote,
    List(bool),
    Item,
    Leaf,
}
struct Frame {
    tag: u8,
    kind: Kind,
    start: usize,
    blocks: Vec<Block>,
    items: Vec<Vec<Block>>,
}
fn tag(container: &Container<'_>) -> Option<u8> {
    Some(match container {
        Container::Div { .. } => 1,
        Container::Blockquote => 2,
        Container::List { .. } => 3,
        Container::ListItem | Container::TaskListItem { .. } => 4,
        Container::Paragraph => 5,
        Container::Heading { .. } => 6,
        Container::CodeBlock { .. } => 7,
        Container::DescriptionList => 8,
        Container::Table => 9,
        _ => return None,
    })
}
pub(super) fn parse(source: &str) -> (Vec<Block>, Vec<DocumentDiagnostic>) {
    let mut frames = vec![Frame {
        tag: 0,
        kind: Kind::Root,
        start: 0,
        blocks: Vec::new(),
        items: Vec::new(),
    }];
    let mut diagnostics = Vec::new();
    let mut pending = None;
    for (event, range) in Parser::new(source).into_offset_iter() {
        match event {
            Event::Attributes(_) => pending = Some(range.start),
            Event::Start(container, attrs) => {
                if matches!(frames.last().unwrap().kind, Kind::Leaf) {
                    continue;
                }
                let Some(tag) = tag(&container) else { continue };
                let kind = match container {
                    Container::Div { class } => {
                        let mut attributes: BTreeMap<_, _> = attrs
                            .unique_pairs()
                            .map(|(k, v)| (k.to_string(), v.to_string()))
                            .collect();
                        if !class.is_empty() {
                            let classes = attributes.entry("class".into()).or_default();
                            if !classes.split_whitespace().any(|s| s == class.as_ref()) {
                                if !classes.is_empty() {
                                    classes.push(' ')
                                }
                                classes.push_str(&class);
                            }
                        }
                        Kind::Div(attributes)
                    },
                    Container::Blockquote => Kind::Quote,
                    Container::List { kind, .. } => {
                        Kind::List(matches!(kind, jotdown::ListKind::Ordered { .. }))
                    },
                    Container::ListItem | Container::TaskListItem { .. } => Kind::Item,
                    _ => Kind::Leaf,
                };
                frames.push(Frame {
                    tag,
                    kind,
                    start: pending.take().unwrap_or(range.start),
                    blocks: Vec::new(),
                    items: Vec::new(),
                });
            },
            Event::End(container) if tag(&container) == frames.last().map(|f| f.tag) => {
                let frame = frames.pop().unwrap();
                let parent = frames.last_mut().unwrap();
                match frame.kind {
                    Kind::Div(attributes) => {
                        let class = attributes.get("class").map(String::as_str).unwrap_or("");
                        let simple = frame.blocks.len() <= 1
                            && frame
                                .blocks
                                .iter()
                                .all(|b| matches!(b, Block::Paragraph { .. }));
                        if simple
                            && div_spec(class).is_some()
                            && attributes
                                .keys()
                                .all(|k| matches!(k.as_str(), "class" | "title" | "url" | "date"))
                        {
                            let (blocks, notes) =
                                parse_legacy_body(&source[frame.start..range.end]);
                            parent.blocks.extend(blocks);
                            diagnostics.extend(notes);
                        } else {
                            parent.blocks.push(Block::Div {
                                attributes,
                                blocks: frame.blocks,
                            });
                        }
                    },
                    Kind::Quote => parent.blocks.push(Block::Quote {
                        blocks: frame.blocks,
                    }),
                    Kind::List(ordered) => parent.blocks.push(Block::List {
                        ordered,
                        items: frame.items,
                    }),
                    Kind::Item => parent.items.push(frame.blocks),
                    Kind::Leaf => {
                        let raw = &source[frame.start..range.end];
                        let (mut blocks, notes) = parse_legacy_body(raw);
                        // A leaf outside the current lowering vocabulary stays
                        // visible and source-preserving rather than disappearing.
                        if blocks.is_empty() && !raw.trim().is_empty() {
                            blocks.push(Block::CodeBlock {
                                language: Some("djot".into()),
                                text: raw.into(),
                            });
                        }
                        parent.blocks.extend(blocks);
                        diagnostics.extend(notes);
                    },
                    Kind::Root => unreachable!(),
                }
                pending = None;
            },
            Event::ThematicBreak(_) if !matches!(frames.last().unwrap().kind, Kind::Leaf) => {
                frames.last_mut().unwrap().blocks.push(Block::Rule)
            },
            _ => {},
        }
    }
    (frames.remove(0).blocks, diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lists_and_quotes_remain_parents_of_domain_blocks() {
        let source = "> ::: metre\n> the night\n> :::\n\n- one\n- two\n\n{#music}\n::: progression\n- I\n- V\n:::\n";
        let (blocks, _) = parse(source);
        let Block::Quote { blocks: quoted } = &blocks[0] else {
            panic!("quote lost: {blocks:?}")
        };
        assert!(matches!(&quoted[0], Block::Div { .. }));
        assert!(matches!(&blocks[1],Block::List{items,..} if items.len()==2));
        let Block::Div { blocks: music, .. } = &blocks[2] else {
            panic!("music lost")
        };
        assert!(matches!(&music[0],Block::List{items,..} if items.len()==2));
        let reparsed = parse(&blocks_to_djot(&blocks)).0;
        assert_eq!(reparsed, blocks);
    }
}
