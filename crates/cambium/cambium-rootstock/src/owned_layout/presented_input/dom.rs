// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Owned layout-DOM facts. No new identities or mutable-arena borrows survive.

use super::{HashMap, LayoutDom, NodeId, WindowDom};
use layout_dom_api::{
    AttributeView, DoctypeView, FormControlState, LocalName, Namespace, NodeKind, QualName,
    QuirksMode, ShadowRootInit,
};

#[derive(Clone, Debug)]
struct Node {
    parent: Option<NodeId>,
    previous: Option<NodeId>,
    next: Option<NodeId>,
    children: Vec<NodeId>,
    flat_children: Vec<NodeId>,
    kind: NodeKind,
    name: Option<QualName>,
    attributes: Vec<(QualName, String)>,
    text: Option<String>,
    form: Option<FormControlState>,
    doctype: Option<(String, String, String)>,
    shadow_init: Option<ShadowRootInit>,
    shadow_root: Option<NodeId>,
    shadow_host: Option<NodeId>,
    assigned_slot: Option<NodeId>,
    assigned_nodes: Vec<NodeId>,
}

#[derive(Clone, Debug)]
pub(super) struct FrozenDom {
    root: NodeId,
    quirks: QuirksMode,
    shadow_roots: Vec<NodeId>,
    nodes: HashMap<NodeId, Node>,
}

impl FrozenDom {
    pub(super) fn capture(dom: &WindowDom<'_>) -> Self {
        let mut nodes = HashMap::new();
        let mut pending = vec![dom.document()];
        // DOM children retain unslotted light children and inactive slot
        // fallback. Attached shadow roots add their separate node trees.
        // A flat-only traversal would lose both categories of input facts.
        while let Some(id) = pending.pop() {
            if nodes.contains_key(&id) || !dom.is_live(id) || !dom.contains(id) {
                continue;
            }
            let children: Vec<_> = dom.dom_children(id).collect();
            pending.extend(children.iter().copied());
            pending.extend(dom.shadow_root(id));
            nodes.insert(
                id,
                Node {
                    parent: dom.parent(id),
                    previous: dom.prev_sibling(id),
                    next: dom.next_sibling(id),
                    children,
                    flat_children: dom.flat_children(id).collect(),
                    kind: dom.kind(id),
                    name: dom.element_name(id).cloned(),
                    attributes: dom
                        .attributes(id)
                        .map(|attr| (attr.name.clone(), attr.value.to_owned()))
                        .collect(),
                    text: dom.text(id).map(str::to_owned),
                    form: dom.form_control_state(id),
                    doctype: dom.doctype_data(id).map(|data| {
                        (
                            data.name.to_owned(),
                            data.public_id.to_owned(),
                            data.system_id.to_owned(),
                        )
                    }),
                    shadow_init: dom.shadow_init(id),
                    shadow_root: dom.shadow_root(id),
                    shadow_host: dom.shadow_host(id),
                    assigned_slot: dom.assigned_slot(id),
                    assigned_nodes: dom.assigned_nodes(id),
                },
            );
        }
        Self {
            root: dom.document(),
            quirks: dom.quirks_mode(),
            shadow_roots: dom
                .shadow_roots()
                .into_iter()
                .filter(|id| nodes.contains_key(id))
                .collect(),
            nodes,
        }
    }

    fn scoped(&self, id: Option<NodeId>) -> Option<NodeId> {
        id.filter(|id| self.nodes.contains_key(id))
    }
}

impl LayoutDom for FrozenDom {
    type NodeId = NodeId;

    fn document(&self) -> NodeId {
        self.root
    }
    fn is_live(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }
    fn quirks_mode(&self) -> QuirksMode {
        self.quirks
    }
    fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.scoped(self.nodes[&id].parent)
    }
    fn prev_sibling(&self, id: NodeId) -> Option<NodeId> {
        self.scoped(self.nodes[&id].previous)
    }
    fn next_sibling(&self, id: NodeId) -> Option<NodeId> {
        self.scoped(self.nodes[&id].next)
    }
    fn dom_children(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes[&id]
            .children
            .iter()
            .copied()
            .filter(|id| self.is_live(*id))
    }
    fn flat_children(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes[&id]
            .flat_children
            .iter()
            .copied()
            .filter(|id| self.is_live(*id))
    }
    fn kind(&self, id: NodeId) -> NodeKind {
        self.nodes[&id].kind
    }
    fn opaque_id(&self, id: NodeId) -> u64 {
        id.raw()
    }
    fn element_name(&self, id: NodeId) -> Option<&QualName> {
        self.nodes[&id].name.as_ref()
    }
    fn attribute(&self, id: NodeId, ns: &Namespace, local: &LocalName) -> Option<&str> {
        self.nodes[&id]
            .attributes
            .iter()
            .find(|(name, _)| &name.ns == ns && &name.local == local)
            .map(|(_, value)| value.as_str())
    }
    fn attributes(&self, id: NodeId) -> impl Iterator<Item = AttributeView<'_>> + '_ {
        self.nodes[&id]
            .attributes
            .iter()
            .map(|(name, value)| AttributeView { name, value })
    }
    fn text(&self, id: NodeId) -> Option<&str> {
        self.nodes[&id].text.as_deref()
    }
    fn form_control_state(&self, id: NodeId) -> Option<FormControlState> {
        self.nodes[&id].form.clone()
    }
    fn doctype_data(&self, id: NodeId) -> Option<DoctypeView<'_>> {
        self.nodes[&id]
            .doctype
            .as_ref()
            .map(|(name, public_id, system_id)| DoctypeView {
                name,
                public_id,
                system_id,
            })
    }
    fn has_shadow_trees(&self) -> bool {
        !self.shadow_roots.is_empty()
    }
    fn shadow_init(&self, id: NodeId) -> Option<ShadowRootInit> {
        self.nodes[&id].shadow_init
    }
    fn shadow_root(&self, id: NodeId) -> Option<NodeId> {
        self.scoped(self.nodes[&id].shadow_root)
    }
    fn shadow_host(&self, id: NodeId) -> Option<NodeId> {
        self.scoped(self.nodes[&id].shadow_host)
    }
    fn shadow_roots(&self) -> Vec<NodeId> {
        self.shadow_roots.clone()
    }
    fn assigned_slot(&self, id: NodeId) -> Option<NodeId> {
        self.scoped(self.nodes[&id].assigned_slot)
    }
    fn assigned_nodes(&self, id: NodeId) -> Vec<NodeId> {
        self.nodes[&id]
            .assigned_nodes
            .iter()
            .copied()
            .filter(|id| self.is_live(*id))
            .collect()
    }
}
