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
    AttributeView, DoctypeView, FormControlState, LayoutDom, LocalName, Namespace, NodeKind,
    QualName, QuirksMode, ShadowRootInit,
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

    /// Whether `node` lies in this window's subtree. A shadow root has no
    /// parent; its host's place decides.
    pub fn contains(&self, node: NodeId) -> bool {
        ancestor_or_self(self.dom, node, self.root)
    }

    /// Whether `node` lives under another window: live, in the document, and
    /// not in this window's subtree. Never true for a single window.
    pub fn elsewhere(&self, node: NodeId) -> bool {
        self.dom.is_live(node)
            && !self.contains(node)
            && ancestor_or_self(self.dom, node, self.dom.document())
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
        if self.root == self.dom.document() {
            return self.dom.shadow_roots();
        }
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

    fn form_control_state(&self, id: NodeId) -> Option<&FormControlState> {
        if self.is_root(id) {
            None
        } else {
            self.dom.form_control_state(id)
        }
    }

    fn doctype_data(&self, id: NodeId) -> Option<DoctypeView<'_>> {
        self.dom.doctype_data(id)
    }
}

/// Whether `ancestor` is `node` or encloses it, through shadow hosts. A node
/// retired along the way has no ancestors to read, so it is not enclosed.
pub(crate) fn ancestor_or_self(dom: &ScriptedDom, node: NodeId, ancestor: NodeId) -> bool {
    let mut current = Some(node);
    while let Some(id) = current {
        if !dom.is_live(id) {
            return false;
        }
        if id == ancestor {
            return true;
        }
        current = dom.parent(id).or_else(|| dom.shadow_host(id));
    }
    false
}

#[cfg(test)]
mod tests {
    use layout_dom_api::{LayoutDomMut as _, ShadowRootInit};

    use super::*;

    fn element(dom: &mut ScriptedDom, parent: NodeId) -> NodeId {
        let node = dom.create_element(QualName::new(
            None,
            Namespace::from(""),
            LocalName::from("div"),
        ));
        dom.append_child(parent, node);
        node
    }

    #[test]
    fn native_form_state_reads_through_the_window_view() {
        let mut dom = ScriptedDom::new();
        let doc = dom.document();
        let window = element(&mut dom, doc);
        let input = dom.create_element(cambium::html_qual("input"));
        dom.set_attribute(input, cambium::attr_qual("value"), "default");
        dom.append_child(window, input);
        dom.set_form_control_value(input, "current").unwrap();

        let view = WindowDom::new(&dom, window);
        let state = view
            .form_control_state(input)
            .expect("the live arena state");
        assert_eq!(state, dom.form_control_state(input).unwrap());
        assert_eq!(state.value, "current");
        assert_eq!(
            view.attribute(input, &Namespace::from(""), &LocalName::from("value")),
            Some("default")
        );
        assert!(
            WindowDom::document(&dom)
                .form_control_state(input)
                .is_some()
        );
        assert!(
            WindowDom::new(&dom, input)
                .form_control_state(input)
                .is_none(),
            "a window root is exposed as a document, not as a control"
        );
    }

    /// A shadow tree nested in another: the whole document passes every shadow
    /// root straight through, and a window holds the nested one when it holds
    /// the outer host.
    #[test]
    fn nested_shadow_trees_belong_to_the_window_holding_their_hosts() {
        let mut dom = ScriptedDom::new();
        let doc = dom.document();
        let window = element(&mut dom, doc);
        let outer_host = element(&mut dom, window);
        let outer = dom.attach_shadow_unchecked(outer_host, ShadowRootInit::default(), false);
        let inner_host = element(&mut dom, outer);
        let inner = dom.attach_shadow_unchecked(inner_host, ShadowRootInit::default(), false);

        let mut all = WindowDom::document(&dom).shadow_roots();
        all.sort_by_key(|n| n.raw());
        let mut expected = dom.shadow_roots();
        expected.sort_by_key(|n| n.raw());
        assert_eq!(all, expected, "the document passes straight through");
        assert!(all.contains(&inner));

        let view = WindowDom::new(&dom, window);
        assert!(
            view.contains(inner_host),
            "through the outer shadow root's host"
        );
        let roots = view.shadow_roots();
        assert!(roots.contains(&outer) && roots.contains(&inner));
        assert!(!view.elsewhere(inner_host));
    }
}
