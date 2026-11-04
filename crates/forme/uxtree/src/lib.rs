// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! # UxTree
//!
//! Portable a11y / automation tree for the
//! [`mere`](https://crates.io/crates/mere) browser.
//!
//! Projects mere's portable structural elements ([`inker::EngineDocument`]
//! and `platen` workbenches) into
//! a tree of [AccessKit](https://accesskit.dev) `Node`s with stable,
//! deterministic [`NodeId`]s. The same projection feeds screen-reader
//! a11y, automation tooling, and inspector overlays.
//!
//! ## Stable identifiers
//!
//! Every projected node gets an [`accesskit::NodeId`] derived from a domain
//! path. The same document always projects to the same IDs — automation
//! and tests can pin to a specific node without relying on bounds or
//! render order. See [`node_id_for_path`].
//!
//! ## What this crate does NOT own
//!
//! - **Bounds**: gpui (or whichever host) fills `Node::set_bounds` after
//!   layout; the projection here is structural only.
//! - **Event synthesis**: the host owns input. Automation tools build
//!   on top of this tree but route events through host-specific paths.
//! - **Host coupling**: this crate depends on `accesskit` and `inker`,
//!   nothing else. No gpui types here.

#![doc(html_root_url = "https://docs.rs/uxtree/0.0.1")]

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use accesskit::{Action, Node, NodeId, Role, TreeId, TreeInfo, TreeUpdate};
use inker::{Block, EngineDocument, FoldState, InlineSpan, MenuRow, inline_text};

pub mod graft;

pub use graft::{
    ActionTarget, Composition, GraftError, GraftTable, Grafts, as_subtree, graft_node,
    tree_id_for_path,
};

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Lifecycle stage marker.
pub const STAGE: &str = "pre-alpha";

/// A built tree of AccessKit nodes — the result of projecting a portable
/// mere structural element into AccessKit's data model.
///
/// `nodes` is the flat list of `(id, node)` pairs in insertion order;
/// `root` is the entry point. Order is deterministic: the root is always
/// pushed last, after all its descendants, so consumers can build an
/// `accesskit::TreeUpdate` by zipping `nodes` directly.
pub struct UxTree {
    pub root: NodeId,
    pub nodes: Vec<(NodeId, Node)>,
}

impl UxTree {
    /// Convert this tree into an [`accesskit::TreeUpdate`] suitable for
    /// pushing to AccessKit's platform adapters
    /// (`accesskit_windows::Adapter`, `accesskit_unix::Adapter`,
    /// `accesskit_macos::Adapter`) — the same shape accesskit_winit
    /// would emit if the host were winit-based.
    ///
    /// The host provides `focus` (the currently-focused node) so OS a11y
    /// APIs can announce the active element. `None` means no focused
    /// node — useful for inspector dumps and tests.
    ///
    /// Note on OS integration: gpui doesn't currently expose window
    /// handles publicly, so `accesskit_windows::Adapter::with_hwnd`
    /// and equivalents on other platforms can't be constructed without
    /// a gpui patch. Producing the [`TreeUpdate`] is the foundation;
    /// pushing it to the OS is a separate piece of work.
    pub fn to_tree_update(&self, focus: Option<NodeId>) -> TreeUpdate {
        TreeUpdate {
            nodes: self.nodes.iter().map(|(id, n)| (*id, n.clone())).collect(),
            tree: Some(TreeInfo::new(self.root)),
            tree_id: TreeId::ROOT,
            focus: focus.unwrap_or(self.root),
        }
    }
}

/// Hash a domain path to a stable [`NodeId`]. The same path always
/// produces the same id — automation and snapshot tests can pin against
/// it across runs.
pub fn node_id_for_path(path: &str) -> NodeId {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hasher);
    NodeId(hasher.finish())
}

/// Stitch a set of subtrees under a new root. The host typically calls
/// this once per frame to merge mere-domain projections (workbench,
/// orrery, gloss, apparatus, …) into a single application-root tree
/// for accesskit consumption.
///
/// `root_path` produces the stable id for the new root via
/// [`node_id_for_path`]. `root` is the AccessKit node to install at the
/// root (caller chooses role + label + any other properties); its
/// `children` are overwritten with the subtree roots.
pub fn stitch(root_path: &str, root: Node, subtrees: Vec<UxTree>) -> UxTree {
    let root_id = node_id_for_path(root_path);
    let mut nodes = Vec::new();
    let mut child_ids = Vec::with_capacity(subtrees.len());
    for sub in subtrees {
        child_ids.push(sub.root);
        nodes.extend(sub.nodes);
    }
    let mut root = root;
    root.set_children(child_ids);
    nodes.push((root_id, root));
    tracing::debug!(
        root_path,
        subtree_count = child_ids_len(&nodes, root_id),
        node_count = nodes.len(),
        "stitched uxtree subtrees under host root"
    );
    UxTree {
        root: root_id,
        nodes,
    }
}

fn child_ids_len(nodes: &[(NodeId, Node)], root_id: NodeId) -> usize {
    nodes
        .iter()
        .find(|(id, _)| *id == root_id)
        .map(|(_, n)| n.children().len())
        .unwrap_or(0)
}

/// Project an [`EngineDocument`] into an [`UxTree`] rooted at a
/// `Role::Document` node. Children come from the document's blocks; inline
/// spans are flattened into the parent block's accessible name today
/// (with `Role::Link` children as the one exception, since links are
/// interactive and need addressable identity). Collapsible sections take
/// their authored state; see [`project_document_with_folds`].
pub fn project_document(doc: &EngineDocument) -> UxTree {
    project_document_with_folds(doc, &FoldState::default())
}

/// Project under a reader's fold state (plan decision 13). A collapsible
/// heading reports expanded or collapsed and offers the opposite action;
/// blocks inside a closed extent are omitted. Paths keep original block
/// indices, so node ids do not move when a fold toggles.
pub fn project_document_with_folds(doc: &EngineDocument, folds: &FoldState) -> UxTree {
    let mut nodes = Vec::new();
    let root_path = engine_root_path(&doc.address);
    let root_id = node_id_for_path(&root_path);
    let navigation = &doc.navigation;
    let (hidden, headings) = if navigation.is_current(&doc.blocks) {
        let headings: HashMap<usize, bool> = (0..navigation.folds.len())
            .map(|fold| {
                (
                    navigation.folds[fold].heading,
                    folds.is_open(navigation, fold),
                )
            })
            .collect();
        (folds.hidden(navigation), headings)
    } else {
        Default::default()
    };

    let mut child_ids = Vec::new();
    for (i, block) in doc.blocks.iter().enumerate() {
        if hidden.iter().any(|range| range.contains(&i)) {
            continue;
        }
        let path = format!("{root_path}#blocks/{i}");
        let id = project_block(block, &path, &mut nodes);
        if let Some(&open) = headings.get(&i)
            && let Some((_, heading)) = nodes.last_mut().filter(|(last, _)| *last == id)
        {
            heading.set_expanded(open);
            heading.add_action(if open {
                Action::Collapse
            } else {
                Action::Expand
            });
        }
        child_ids.push(id);
    }

    let mut root = Node::new(Role::Document);
    if let Some(title) = &doc.title {
        root.set_label(title.clone());
    }
    if let Some(lang) = &doc.lang {
        root.set_language(lang.clone());
    }
    root.set_children(child_ids);

    nodes.push((root_id, root));
    tracing::debug!(
        address = %doc.address,
        node_count = nodes.len(),
        "projected EngineDocument into UxTree"
    );
    UxTree {
        root: root_id,
        nodes,
    }
}

fn engine_root_path(address: &str) -> String {
    format!("engine:{address}")
}

fn project_block(block: &Block, path: &str, nodes: &mut Vec<(NodeId, Node)>) -> NodeId {
    if let Block::Presented { block, .. } = block {
        return project_block(block, path, nodes);
    }
    let id = node_id_for_path(path);
    let node = match block {
        Block::Presented { .. } => unreachable!("presentation unwrapped above"),
        Block::Heading { level, spans } => {
            let mut n = Node::new(Role::Heading);
            n.set_label(inline_text(spans));
            n.set_level(*level as usize);
            attach_link_children(spans, path, "heading-link", nodes, &mut n);
            n
        },
        Block::Table { header, rows, .. } => {
            // A flat accessible label until row / cell child projection lands;
            // tables are unreachable until the parsers emit them.
            let mut n = Node::new(Role::Table);
            let cells: Vec<String> = header
                .iter()
                .chain(rows.iter().flatten())
                .map(|c| inline_text(c))
                .collect();
            n.set_label(cells.join(" | "));
            n
        },
        Block::Paragraph { spans } => {
            let mut n = Node::new(Role::Paragraph);
            n.set_label(inline_text(spans));
            attach_link_children(spans, path, "paragraph-link", nodes, &mut n);
            n
        },
        Block::CodeBlock { language, text } => {
            // AccessKit 0.24 uses one Role::Code for both inline and block
            // code; block context is implicit from the parent.
            let mut n = Node::new(Role::Code);
            n.set_value(text.clone());
            if let Some(lang) = language {
                // Programming-language hint, not natural-language. Stored
                // as an annotation; AccessKit has no dedicated role
                // property for this, so we use the description.
                n.set_description(format!("language: {lang}"));
            }
            n
        },
        Block::Quote { blocks } => {
            let mut n = Node::new(Role::Blockquote);
            let mut child_ids = Vec::with_capacity(blocks.len());
            for (i, inner) in blocks.iter().enumerate() {
                let inner_path = format!("{path}/quote/{i}");
                child_ids.push(project_block(inner, &inner_path, nodes));
            }
            n.set_children(child_ids);
            n
        },
        Block::List { ordered, items } => {
            let mut n = Node::new(Role::List);
            if *ordered {
                n.set_description("ordered");
            }
            let mut item_ids = Vec::with_capacity(items.len());
            for (i, item_blocks) in items.iter().enumerate() {
                let item_path = format!("{path}/list/{i}");
                let item_id = node_id_for_path(&item_path);
                let mut item_node = Node::new(Role::ListItem);
                let mut item_children = Vec::with_capacity(item_blocks.len());
                for (j, inner) in item_blocks.iter().enumerate() {
                    let inner_path = format!("{item_path}/item/{j}");
                    item_children.push(project_block(inner, &inner_path, nodes));
                }
                item_node.set_children(item_children);
                nodes.push((item_id, item_node));
                item_ids.push(item_id);
            }
            n.set_children(item_ids);
            n
        },
        Block::Image { url, alt } => {
            let mut n = Node::new(Role::Image);
            n.set_label(alt.clone());
            n.set_value(url.clone());
            n
        },
        Block::Preformatted { text } => {
            // No dedicated Role::Pre in AccessKit 0.24; reuse Role::Code
            // since the semantic ("verbatim text, preserve whitespace") is
            // the same for screen readers.
            let mut n = Node::new(Role::Code);
            n.set_value(text.clone());
            n
        },
        Block::Rule => Node::new(Role::Splitter),
        Block::FeedHeader {
            title,
            subtitle,
            summary,
            source_url,
        } => {
            let mut n = Node::new(Role::Section);
            n.set_label(title.clone());
            // Concatenate the optional fields into the description so the
            // a11y projection has a reasonable accessible description even
            // without consuming sub-nodes.
            let mut bits: Vec<String> = Vec::new();
            if let Some(s) = subtitle {
                bits.push(s.clone());
            }
            if let Some(s) = summary {
                bits.push(s.clone());
            }
            if let Some(url) = source_url {
                bits.push(url.clone());
            }
            if !bits.is_empty() {
                n.set_description(bits.join(" — "));
            }
            n
        },
        Block::FeedEntry {
            title,
            date,
            summary,
            article_url,
            source_url,
            content_address,
            ..
        } => {
            let mut n = Node::new(Role::Article);
            n.set_label(title.clone());
            // The entry's own document is somewhere to go, so it is a link.
            if let Some(address) = content_address {
                let link_id = node_id_for_path(&format!("{path}/read"));
                let mut link = Node::new(Role::Link);
                link.set_label("Read here");
                link.set_value(address.clone());
                nodes.push((link_id, link));
                n.set_children(vec![link_id]);
            }
            let mut bits: Vec<String> = Vec::new();
            if let Some(d) = date {
                bits.push(d.clone());
            }
            if let Some(s) = summary {
                bits.push(s.clone());
            }
            if let Some(url) = article_url {
                bits.push(url.clone());
            }
            if let Some(url) = source_url {
                bits.push(url.clone());
            }
            if !bits.is_empty() {
                n.set_description(bits.join(" — "));
            }
            n
        },
        Block::MetadataRow { label, value } => {
            // No definition-list role pair in AccessKit 0.24's stable
            // surface; project as a Group whose label is the term and
            // value is the value, so screen readers can announce both.
            let mut n = Node::new(Role::Group);
            n.set_label(label.clone());
            n.set_value(value.clone());
            n
        },
        Block::Badge { text } => {
            let mut n = Node::new(Role::Note);
            n.set_label(text.clone());
            n
        },
        Block::Menu { rows } => project_menu(rows, path, nodes),
        // A kind this projection does not know yet stays visible as a note
        // naming it, rather than vanishing from the tree.
        other => {
            let mut n = Node::new(Role::Note);
            n.set_label(format!("unsupported block: {}", other.kind_name()));
            n
        },
    };
    nodes.push((id, node));
    id
}

/// A menu as a list whose items name their kind in the description. A row
/// with a target holds one link; a search row holds a button for its
/// submission endpoint; other rows keep any inline links of their label.
fn project_menu(rows: &[MenuRow], path: &str, nodes: &mut Vec<(NodeId, Node)>) -> Node {
    let mut list = Node::new(Role::List);
    list.set_description("menu");
    let mut item_ids = Vec::with_capacity(rows.len());
    for (i, row) in rows.iter().enumerate() {
        let item_path = format!("{path}/menu/{i}");
        let item_id = node_id_for_path(&item_path);
        let label = inline_text(&row.label);
        let mut item = Node::new(Role::ListItem);
        item.set_label(label.clone());
        item.set_description(row.kind.name());
        if let Some(url) = &row.target {
            let link_id = node_id_for_path(&format!("{item_path}/link"));
            let mut link = Node::new(Role::Link);
            link.set_label(label);
            link.set_value(url.clone());
            nodes.push((link_id, link));
            item.set_children(vec![link_id]);
        } else if let Some(target) = submit_target(&row.label) {
            let button_id = node_id_for_path(&format!("{item_path}/submit"));
            let mut button = Node::new(Role::Button);
            button.set_label(label);
            button.set_value(target.to_string());
            nodes.push((button_id, button));
            item.set_children(vec![button_id]);
        } else {
            attach_link_children(&row.label, &item_path, "menu-link", nodes, &mut item);
        }
        nodes.push((item_id, item));
        item_ids.push(item_id);
    }
    list.set_children(item_ids);
    list
}

/// The first submission endpoint in a span list.
fn submit_target(spans: &[InlineSpan]) -> Option<&str> {
    spans.iter().find_map(|span| match span {
        InlineSpan::Submit { target, .. } => Some(target.as_str()),
        InlineSpan::Presented { spans, .. }
        | InlineSpan::Emphasis(spans)
        | InlineSpan::Strong(spans) => submit_target(spans),
        _ => None,
    })
}

/// Walk inline spans for network and in-page links and project each as a
/// `Role::Link` child of the enclosing block. Other inline styling
/// (Code/Emphasis/Strong) stays as text in the parent's accessible name;
/// links are interactive so they need their own addressable identity.
fn attach_link_children(
    spans: &[InlineSpan],
    parent_path: &str,
    tag: &str,
    nodes: &mut Vec<(NodeId, Node)>,
    parent: &mut Node,
) {
    let mut existing = parent.children().to_vec();
    let mut counter = 0usize;
    walk_inline_links(spans, &mut |link_url, link_title, link_spans| {
        let link_path = format!("{parent_path}/{tag}/{counter}");
        let link_id = node_id_for_path(&link_path);
        let mut link_node = Node::new(Role::Link);
        let label = inline_text(link_spans);
        if !label.is_empty() {
            link_node.set_label(label);
        }
        if let Some(t) = link_title {
            link_node.set_description(t.clone());
        }
        // An in-page link has no URL, so no host can treat it as navigation.
        if let Some(url) = link_url {
            link_node.set_value(url.to_string());
        }
        nodes.push((link_id, link_node));
        existing.push(link_id);
        counter += 1;
    });
    if !existing.is_empty() {
        parent.set_children(existing);
    }
}

fn walk_inline_links<'a, F>(spans: &'a [InlineSpan], f: &mut F)
where
    F: FnMut(Option<&'a str>, Option<&'a String>, &'a [InlineSpan]),
{
    for span in spans {
        match span {
            InlineSpan::Link {
                url, title, spans, ..
            } => {
                f(Some(url.as_str()), title.as_ref(), spans.as_slice());
                walk_inline_links(spans, f);
            },
            InlineSpan::InPage { spans, .. } => {
                f(None, None, spans.as_slice());
                walk_inline_links(spans, f);
            },
            InlineSpan::Emphasis(inner)
            | InlineSpan::Strong(inner)
            | InlineSpan::Presented { spans: inner, .. }
            | InlineSpan::Submit { spans: inner, .. } => {
                walk_inline_links(inner, f);
            },
            InlineSpan::Text(_)
            | InlineSpan::Code(_)
            | InlineSpan::LineBreak
            | InlineSpan::SoftBreak => {},
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
