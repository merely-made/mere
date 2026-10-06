// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One window's view of a forest document (stack seams S28): the window-root
//! element presented as the document, so layout, hit testing, paint and
//! accessibility see that window's subtree and nothing else.
//!
//! Over a single-window document the root *is* the document and every answer
//! passes straight through, so the host drives one path for both.

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{
    AttributeView, DoctypeView, LayoutDom, LocalName, Namespace, NodeKind, QualName, QuirksMode,
    ShadowRootInit,
};

/// A window's subtree of `dom`, rooted at `root`.
///
/// The root answers as a document: no parent, no siblings, no element name or
/// attributes, so a `:root` rule matches the window's top element and a
/// stylesheet sees the window as its whole page. Everything below the root
/// reads through unchanged. The tree-root walks (`node_tree_root` and the
/// shadow ones built on it) are the trait's own, which walk `parent` and so
/// stop at the window root.
#[derive(Clone, Copy)]
pub struct WindowDom<'a> {
    dom: &'a ScriptedDom,
    root: NodeId,
}

impl<'a> WindowDom<'a> {
    /// The subtree of `dom` under `root`, which is the document for a
    /// single-window host and a window-root element under a forest.
    pub fn new(dom: &'a ScriptedDom, root: NodeId) -> Self {
        Self { dom, root }
    }

    /// The whole document, unscoped.
    pub fn document(dom: &'a ScriptedDom) -> Self {
        Self::new(dom, dom.document())
    }

    /// The document underneath, for the few reads that are not layout's.
    pub fn dom(&self) -> &'a ScriptedDom {
        self.dom
    }

    /// Whether `node` lies in this window's subtree.
    pub fn contains(&self, node: NodeId) -> bool {
        let mut current = Some(node);
        while let Some(id) = current {
            if id == self.root {
                return true;
            }
            current = self.dom.parent(id);
        }
        false
    }

    fn is_root(&self, id: NodeId) -> bool {
        id == self.root
    }
}

impl LayoutDom for WindowDom<'_> {
    type NodeId = NodeId;

    fn document(&self) -> NodeId {
        self.root
    }

    fn is_live(&self, id: NodeId) -> bool {
        self.dom.is_live(id)
    }

    fn quirks_mode(&self) -> QuirksMode {
        self.dom.quirks_mode()
    }

    fn parent(&self, id: NodeId) -> Option<NodeId> {
        if self.is_root(id) {
            None
        } else {
            self.dom.parent(id)
        }
    }

    fn prev_sibling(&self, id: NodeId) -> Option<NodeId> {
        if self.is_root(id) {
            None
        } else {
            self.dom.prev_sibling(id)
        }
    }

    fn next_sibling(&self, id: NodeId) -> Option<NodeId> {
        if self.is_root(id) {
            None
        } else {
            self.dom.next_sibling(id)
        }
    }

    fn dom_children(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.dom.dom_children(id)
    }

    fn flat_children(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.dom.flat_children(id)
    }

    fn has_shadow_trees(&self) -> bool {
        self.dom.has_shadow_trees()
    }

    fn shadow_init(&self, id: NodeId) -> Option<ShadowRootInit> {
        self.dom.shadow_init(id)
    }

    fn shadow_root(&self, id: NodeId) -> Option<NodeId> {
        self.dom.shadow_root(id)
    }

    fn shadow_roots(&self) -> Vec<NodeId> {
        self.dom
            .shadow_roots()
            .into_iter()
            .filter(|&shadow| {
                self.dom
                    .shadow_host(shadow)
                    .is_some_and(|host| self.contains(host))
            })
            .collect()
    }

    fn shadow_host(&self, id: NodeId) -> Option<NodeId> {
        self.dom.shadow_host(id)
    }

    fn assigned_slot(&self, id: NodeId) -> Option<NodeId> {
        self.dom.assigned_slot(id)
    }

    fn assigned_nodes(&self, id: NodeId) -> Vec<NodeId> {
        self.dom.assigned_nodes(id)
    }

    fn template_contents(&self, id: NodeId) -> Option<NodeId> {
        self.dom.template_contents(id)
    }

    fn kind(&self, id: NodeId) -> NodeKind {
        if self.is_root(id) {
            NodeKind::Document
        } else {
            self.dom.kind(id)
        }
    }

    fn opaque_id(&self, id: NodeId) -> u64 {
        self.dom.opaque_id(id)
    }

    fn element_name(&self, id: NodeId) -> Option<&QualName> {
        if self.is_root(id) {
            None
        } else {
            self.dom.element_name(id)
        }
    }

    fn attribute(&self, id: NodeId, ns: &Namespace, local: &LocalName) -> Option<&str> {
        if self.is_root(id) {
            None
        } else {
            self.dom.attribute(id, ns, local)
        }
    }

    fn attributes(&self, id: NodeId) -> impl Iterator<Item = AttributeView<'_>> + '_ {
        let root = self.is_root(id);
        self.dom.attributes(id).filter(move |_| !root)
    }

    fn text(&self, id: NodeId) -> Option<&str> {
        if self.is_root(id) {
            None
        } else {
            self.dom.text(id)
        }
    }

    fn doctype_data(&self, id: NodeId) -> Option<DoctypeView<'_>> {
        self.dom.doctype_data(id)
    }
}
