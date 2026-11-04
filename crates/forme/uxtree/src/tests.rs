// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use inker::{Block, DocumentProvenance, DocumentTrustState, EngineDocument, InlineSpan};

fn doc_with(blocks: Vec<Block>) -> EngineDocument {
    EngineDocument {
        address: "doc:test".to_string(),
        title: Some("Test Doc".to_string()),
        content_type: "text/plain".to_string(),
        lang: Some("en".to_string()),
        provenance: DocumentProvenance::default(),
        trust: DocumentTrustState::Unknown,
        diagnostics: Vec::new(),
        navigation: Default::default(),
        blocks,
    }
}

#[test]
fn document_root_is_role_document_with_title_and_lang() {
    let doc = doc_with(vec![]);
    let tree = project_document(&doc);
    let (_, root) = tree.nodes.iter().find(|(id, _)| *id == tree.root).unwrap();
    assert_eq!(root.role(), Role::Document);
    assert_eq!(root.label(), Some("Test Doc"));
    assert_eq!(root.language(), Some("en"));
}

#[test]
fn ids_are_deterministic() {
    let doc = doc_with(vec![Block::Paragraph {
        spans: vec![InlineSpan::Text("hi".to_string())],
    }]);
    let a = project_document(&doc);
    let b = project_document(&doc);
    assert_eq!(a.root, b.root);
    assert_eq!(
        a.nodes.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        b.nodes.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
    );
}

#[test]
fn menu_projects_typed_items_with_link_and_submit_children() {
    use inker::{MenuItemKind, MenuRow};
    let row = |kind, marker, label: &str, target: Option<&str>| MenuRow {
        kind,
        marker: Some(marker),
        label: vec![InlineSpan::Text(label.to_string())],
        target: target.map(str::to_string),
    };
    let doc = doc_with(vec![Block::Menu {
        rows: vec![
            row(MenuItemKind::Info, 'i', "Welcome", None),
            row(
                MenuItemKind::Directory,
                '1',
                "Phlog",
                Some("gopher://h/1/phlog"),
            ),
            MenuRow {
                kind: MenuItemKind::Search,
                marker: Some('7'),
                label: vec![InlineSpan::Submit {
                    target: "gopher://h/7/find".to_string(),
                    spans: vec![InlineSpan::Text("Search".to_string())],
                }],
                target: None,
            },
        ],
    }]);
    let tree = project_document(&doc);
    let node = |role: Role| -> Vec<&Node> {
        tree.nodes
            .iter()
            .map(|(_, n)| n)
            .filter(|n| n.role() == role)
            .collect()
    };
    let [list] = node(Role::List)[..] else {
        panic!("one list");
    };
    assert_eq!(list.description(), Some("menu"));
    let items = node(Role::ListItem);
    let kinds: Vec<_> = items.iter().map(|n| n.description().unwrap()).collect();
    assert_eq!(kinds, ["info", "directory", "search"]);
    let [link] = node(Role::Link)[..] else {
        panic!("one link");
    };
    assert_eq!(link.value(), Some("gopher://h/1/phlog"));
    let [button] = node(Role::Button)[..] else {
        panic!("one submit button");
    };
    assert_eq!(button.label(), Some("Search"));
    assert_eq!(button.value(), Some("gopher://h/7/find"));
}

#[test]
fn a_feed_entry_with_a_body_links_its_own_document() {
    let doc = doc_with(vec![Block::FeedEntry {
        title: "Post".to_string(),
        date: None,
        summary: None,
        article_url: Some("https://x.test/post".to_string()),
        source_url: None,
        published: None,
        updated: None,
        guid: Some("post-1".to_string()),
        enclosures: Vec::new(),
        content_address: Some("gemini://x.test/feed.xml#post-1".to_string()),
    }]);
    let tree = project_document(&doc);
    let link = tree
        .nodes
        .iter()
        .map(|(_, n)| n)
        .find(|n| n.role() == Role::Link)
        .expect("a read link");
    assert_eq!(link.label(), Some("Read here"));
    assert_eq!(link.value(), Some("gemini://x.test/feed.xml#post-1"));
}

#[test]
fn heading_carries_level_and_label() {
    let doc = doc_with(vec![Block::Heading {
        level: 2,
        spans: vec![InlineSpan::Text("Section".to_string())],
    }]);
    let tree = project_document(&doc);
    let heading = tree
        .nodes
        .iter()
        .map(|(_, n)| n)
        .find(|n| n.role() == Role::Heading)
        .expect("heading node");
    assert_eq!(heading.label(), Some("Section"));
    assert_eq!(heading.level(), Some(2));
}

#[test]
fn list_projects_with_listitem_children() {
    let doc = doc_with(vec![Block::List {
        ordered: false,
        items: vec![
            vec![Block::Paragraph {
                spans: vec![InlineSpan::Text("first".to_string())],
            }],
            vec![Block::Paragraph {
                spans: vec![InlineSpan::Text("second".to_string())],
            }],
        ],
    }]);
    let tree = project_document(&doc);
    let list_items = tree
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == Role::ListItem)
        .count();
    assert_eq!(list_items, 2);
}

#[test]
fn to_tree_update_preserves_root_and_node_count() {
    let mut root_node = Node::new(Role::Window);
    root_node.set_label("app");
    let root_id = node_id_for_path("app-root");
    let mut child_node = Node::new(Role::Group);
    child_node.set_label("child");
    let child_id = node_id_for_path("app-root/child");
    root_node.set_children(vec![child_id]);

    let tree = UxTree {
        root: root_id,
        nodes: vec![(child_id, child_node), (root_id, root_node)],
    };
    let update = tree.to_tree_update(None);

    assert_eq!(update.nodes.len(), 2);
    let tree_root = update.tree.as_ref().expect("tree present").root;
    assert_eq!(tree_root, root_id);
    // No focus override → focus defaults to root.
    assert_eq!(update.focus, root_id);

    let with_focus = tree.to_tree_update(Some(child_id));
    assert_eq!(with_focus.focus, child_id);
}

#[test]
fn stitch_combines_subtrees_under_a_new_root() {
    let mut subtree_a_root = Node::new(Role::Group);
    subtree_a_root.set_label("A");
    let subtree_a = UxTree {
        root: node_id_for_path("a"),
        nodes: vec![(node_id_for_path("a"), subtree_a_root)],
    };
    let mut subtree_b_root = Node::new(Role::Group);
    subtree_b_root.set_label("B");
    let subtree_b = UxTree {
        root: node_id_for_path("b"),
        nodes: vec![(node_id_for_path("b"), subtree_b_root)],
    };

    let mut host_root = Node::new(Role::Window);
    host_root.set_label("host");
    let stitched = stitch("host", host_root, vec![subtree_a, subtree_b]);

    let (_, root) = stitched
        .nodes
        .iter()
        .find(|(id, _)| *id == stitched.root)
        .unwrap();
    assert_eq!(root.role(), Role::Window);
    assert_eq!(root.children().len(), 2);
    assert_eq!(stitched.nodes.len(), 3);
}

#[test]
fn link_in_paragraph_projects_as_addressable_child() {
    let doc = doc_with(vec![Block::Paragraph {
        spans: vec![
            InlineSpan::Text("see ".to_string()),
            InlineSpan::Link {
                url: "https://example.test/".to_string(),
                title: None,
                spans: vec![InlineSpan::Text("the docs".to_string())],
                predicate: None,
            },
        ],
    }]);
    let tree = project_document(&doc);
    let link = tree
        .nodes
        .iter()
        .map(|(_, n)| n)
        .find(|n| n.role() == Role::Link)
        .expect("link node");
    assert_eq!(link.label(), Some("the docs"));
    assert_eq!(link.value(), Some("https://example.test/"));
}

#[test]
fn in_page_link_projects_as_a_link_without_a_url() {
    let doc = doc_with(vec![Block::Paragraph {
        spans: vec![InlineSpan::InPage {
            target: inker::InPageTarget {
                fragment: Some("setup".to_string()),
                block: Some(0),
            },
            spans: vec![InlineSpan::Text("Setup".to_string())],
        }],
    }]);
    let tree = project_document(&doc);
    let link = tree
        .nodes
        .iter()
        .map(|(_, n)| n)
        .find(|n| n.role() == Role::Link)
        .expect("in-page link node");
    assert_eq!(link.label(), Some("Setup"));
    assert_eq!(link.value(), None, "decision 13: no URL");
    let paragraph = tree
        .nodes
        .iter()
        .map(|(_, n)| n)
        .find(|n| n.role() == Role::Paragraph)
        .expect("paragraph node");
    assert_eq!(paragraph.label(), Some("Setup"));
}

#[test]
fn presentation_preserves_accessible_identity_and_nested_links() {
    let block = Block::Paragraph {
        spans: vec![InlineSpan::Link {
            url: "gemini://example.test/".into(),
            title: None,
            spans: vec![InlineSpan::Text("Read this".into())],
            predicate: None,
        }],
    };
    let plain = project_document(&doc_with(vec![block.clone()]));
    let Block::Paragraph { spans } = block else {
        unreachable!()
    };
    let styled = project_document(&doc_with(vec![Block::Presented {
        presentation: inker::BlockPresentation::default(),
        block: Box::new(Block::Paragraph {
            spans: vec![InlineSpan::Presented {
                presentation: inker::InlinePresentation::default(),
                spans,
            }],
        }),
    }]));
    assert_eq!(plain.root, styled.root);
    assert_eq!(plain.nodes, styled.nodes);
}

const NODE: &str = "923706ddc70d389bd3719258c41f6592";

fn micron(file: &str, source: &str) -> EngineDocument {
    use inker::{Engine, EngineInput};
    nematic::MicronEngine::new()
        .render(&EngineInput::new(format!("{NODE}:/page/{file}"), source))
        .unwrap()
}

fn block_label(block: &Block) -> Option<String> {
    match block {
        Block::Presented { block, .. } => block_label(block),
        Block::Heading { spans, .. } | Block::Paragraph { spans } => Some(inline_text(spans)),
        _ => None,
    }
}

fn labelled<'a>(tree: &'a UxTree, role: Role, label: &str) -> Option<(NodeId, &'a Node)> {
    tree.nodes
        .iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .map(|(id, node)| (*id, node))
}

#[test]
fn probes_05_17_fold_headings_carry_expanded_state_and_hidden_blocks_are_omitted() {
    for (file, source, target, links, ancestors) in [
        (
            "probe-nav-05-closed-target.mu",
            include_str!(
                "../../../nematic/nematic/tests/fixtures/micron/nomadnet-1.4.2/navigation/probe-nav-05-closed-target.mu"
            ),
            (Role::Heading, "Hidden Target"),
            &[
                "jump to the hidden heading",
                "jump to the hidden explicit anchor",
            ][..],
            &["Closed Outer"][..],
        ),
        (
            "probe-nav-17-nested-closed-target.mu",
            include_str!(
                "../../../nematic/nematic/tests/fixtures/micron/nomadnet-1.4.2/navigation/probe-nav-17-nested-closed-target.mu"
            ),
            (
                Role::Paragraph,
                "MARKER NESTED DEEP TARGET: bound inside both closed folds.",
            ),
            &["jump into two closed sections"],
            &["Outer Closed", "Inner Closed"],
        ),
    ] {
        let doc = micron(file, source);
        let mut unfolded = doc.clone();
        unfolded.navigation = Default::default();
        let unfolded = project_document(&unfolded);

        let authored = project_document(&doc);
        let (_, outer) = labelled(&authored, Role::Heading, ancestors[0]).unwrap();
        assert_eq!(outer.is_expanded(), Some(false), "{file}");
        assert!(outer.supports_action(Action::Expand), "{file}");
        assert!(
            labelled(&authored, target.0, target.1).is_none(),
            "{file}: hidden"
        );
        assert!(
            ancestors[1..]
                .iter()
                .all(|inner| labelled(&authored, Role::Heading, inner).is_none()),
            "{file}: a nested heading hides with its ancestor"
        );
        assert_eq!(
            labelled(&authored, Role::Heading, "Sentinel After").map(|(id, _)| id),
            labelled(&unfolded, Role::Heading, "Sentinel After").map(|(id, _)| id),
            "{file}: a block after a closed extent keeps its node id"
        );
        for link in links {
            let (_, node) = labelled(&authored, Role::Link, link).unwrap();
            assert_eq!(node.value(), None, "{file}: {link} has no URL");
        }

        let mut state = FoldState::default();
        let target_block = doc
            .blocks
            .iter()
            .position(|block| block_label(block).as_deref() == Some(target.1))
            .unwrap();
        assert!(state.open_ancestors(&doc.navigation, target_block));
        let revealed = project_document_with_folds(&doc, &state);
        for ancestor in ancestors {
            let (_, heading) = labelled(&revealed, Role::Heading, ancestor).unwrap();
            assert_eq!(heading.is_expanded(), Some(true), "{file}: {ancestor}");
            assert!(heading.supports_action(Action::Collapse));
        }
        assert_eq!(
            labelled(&revealed, target.0, target.1).map(|(id, _)| id),
            labelled(&unfolded, target.0, target.1).map(|(id, _)| id),
            "{file}: the revealed target has its unfolded node id"
        );
    }
}
