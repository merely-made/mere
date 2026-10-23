// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shared host composition of DOM, Sprigging and producer semantics.
//! AccessKit remains the existing contributor vocabulary. This copied read is
//! not a presentation revision, target lease or operation-outcome authority.
use crate::a11y_provenance::{A11yProvenance, NodeTarget, SemanticNodeIdentity};
use crate::owned_layout::{custom_leaf_key, flat_nodes};
use crate::{
    LeafRegistry, NodeId, OwnedLayout, ProducedAction, ProducerRegistry, ProducerRole,
    ProducerSemantics,
};
use accesskit::{Action, NodeId as A11yNodeId, Rect, Role, TreeUpdate};
use document_session_api::{DocumentA11yProjection, DocumentA11yRole};
use layout_dom_api::LayoutDom;
use std::collections::{HashMap, HashSet};

/// A producer contribution refused by this composition read. These identify
/// ambiguous declarations, not producer generations or operation outcomes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProducerSemanticIssue {
    DuplicateChildKey { producer: u64, key: u64 },
    DuplicateActionId { producer: u64, key: u64, id: String },
    NodeIdCollision { producer: u64, id: A11yNodeId },
}

pub struct SemanticProjection {
    pub tree: TreeUpdate,
    produced_nodes: HashMap<A11yNodeId, NodeTarget>,
    pub dom_targets: HashMap<A11yNodeId, NodeId>,
    pub produced_targets: HashMap<A11yNodeId, ProducedAction>,
    pub producer_slots: HashMap<u64, NodeId>,
    /// Refused contributions; their slot overrides, children and action routes
    /// are absent. The underlying DOM observation remains available.
    pub producer_issues: Vec<ProducerSemanticIssue>,
}
/// Bindings for a complete publication. They are metadata, not another tree.
pub struct PublicationBindings {
    pub identities: HashMap<A11yNodeId, SemanticNodeIdentity>,
    pub requests: HashMap<u64, crate::A11yRequest>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PublicationIssue {
    OwnerEnded,
    UnknownNode(A11yNodeId),
    DuplicateNode(A11yNodeId),
    RouteMismatch(A11yNodeId),
    ContributorEnded(A11yNodeId),
}
impl SemanticProjection {
    /// Freeze all node identities and action routes from one authored read.
    /// Unknown/duplicate nodes or inconsistent route metadata refuse publication.
    pub fn publication_bindings(
        &self,
        owner: &crate::A11yOwner,
        dom: &crate::WindowDom<'_>,
        leaves: &LeafRegistry<u64>,
        producers: &ProducerRegistry,
    ) -> Result<PublicationBindings, PublicationIssue> {
        if !owner.is_live() {
            return Err(PublicationIssue::OwnerEnded);
        }
        let dom_nodes: HashMap<_, _> = flat_nodes(dom)
            .into_iter()
            .map(|node| (A11yNodeId(dom.opaque_id(node)), node))
            .collect();
        let mut identities = HashMap::new();
        for (id, _) in &self.tree.nodes {
            let target = self
                .produced_nodes
                .get(id)
                .cloned()
                .or_else(|| dom_nodes.get(id).map(|node| NodeTarget::Node(*node)))
                .ok_or(PublicationIssue::UnknownNode(*id))?;
            let identity = SemanticNodeIdentity(A11yProvenance::capture_node(
                owner, target, dom, leaves, producers,
            ));
            if !identity.0.contributors_live() {
                return Err(PublicationIssue::ContributorEnded(*id));
            }
            if identities.insert(*id, identity).is_some() {
                return Err(PublicationIssue::DuplicateNode(*id));
            }
        }
        let mut requests = HashMap::new();
        for (id, mut request) in self
            .dom_targets
            .iter()
            .map(|(id, node)| {
                (
                    *id,
                    crate::A11yRequest::node(crate::A11yAction::Focus, *node),
                )
            })
            .chain(self.produced_targets.iter().map(|(id, target)| {
                (
                    *id,
                    crate::A11yRequest::produced(crate::A11yAction::Focus, target.clone()),
                )
            }))
        {
            let identity = identities
                .get(&id)
                .ok_or(PublicationIssue::UnknownNode(id))?;
            if identity.0.target_matches(&request.target) {
                request.provenance = Some(identity.0.clone());
            } else {
                return Err(PublicationIssue::RouteMismatch(id));
            }
            requests.insert(id.0, request);
        }
        Ok(PublicationBindings {
            identities,
            requests,
        })
    }
    /// Actual DOM membership, supplied by composition rather than inferred
    /// from the numeric shape of a generated semantic ID.
    pub fn dom_node(&self, id: u64) -> Option<NodeId> {
        self.dom_targets.get(&A11yNodeId(id)).copied()
    }
    pub fn produced_action(&self, id: u64) -> Option<&ProducedAction> {
        self.produced_targets.get(&A11yNodeId(id))
    }
    /// Lower a DOM observation before optional contributions. The usual host
    /// entry point is `project_semantics`, which binds the current providers.
    pub fn from_document(projection: DocumentA11yProjection) -> Self {
        let multiline: HashSet<_> = projection
            .nodes()
            .iter()
            .filter(|node| node.role == DocumentA11yRole::TextField && node.state.multiline)
            .map(|node| A11yNodeId(node.id.get()))
            .collect();
        let mut tree = genet_render::accesskit_tree_from_projection(projection);
        for (id, node) in &mut tree.nodes {
            if multiline.contains(id) {
                node.set_role(Role::MultilineTextInput);
            }
        }
        Self {
            tree,
            produced_nodes: HashMap::new(),
            dom_targets: HashMap::new(),
            produced_targets: HashMap::new(),
            producer_slots: HashMap::new(),
            producer_issues: Vec::new(),
        }
    }
}

/// Compose once for a read, using the same fully seeded leaf callback in every
/// consumer. Producers speak for their slots; otherwise an author's name wins
/// over a leaf label. Host geometry, tree edges and restrictive states remain
/// host-owned. Provider keys and generated IDs retain their existing lifetimes.
pub fn project_semantics<D: LayoutDom<NodeId = NodeId>>(
    dom: &D,
    layout: &OwnedLayout,
    leaves: &LeafRegistry<u64>,
    producers: &ProducerRegistry,
    focus: Option<u64>,
) -> SemanticProjection {
    let mut result =
        SemanticProjection::from_document(crate::document_projection(dom, layout, focus));
    let indices: HashMap<_, _> = result
        .tree
        .nodes
        .iter()
        .enumerate()
        .map(|(i, (id, _))| (*id, i))
        .collect();
    let mut reserved_ids = indices.keys().copied().collect();
    let mut nodes = Vec::new();
    let mut slot_counts = HashMap::<u64, usize>::new();
    for node in flat_nodes(dom) {
        let key = custom_leaf_key(dom, node);
        if let Some(key) = key {
            *slot_counts.entry(key).or_default() += 1;
        }
        nodes.push((node, key));
    }
    for (node, key) in nodes {
        let id = A11yNodeId(dom.opaque_id(node));
        let Some(&index) = indices.get(&id) else {
            continue;
        };
        // Rendering refuses a producer bound to multiple DOM owners. Do not
        // publish actionable children or choose one arbitrary semantic owner.
        if let Some(key) = key.filter(|key| !producers.contains(*key) || slot_counts[key] == 1) {
            if let Some(semantics) = producers.semantics(key) {
                match add_producer_semantics(
                    &mut result.tree,
                    id,
                    key,
                    semantics,
                    layout.scrolled_content_origin(dom, node),
                    &mut result.produced_targets,
                    &mut result.produced_nodes,
                    &mut reserved_ids,
                ) {
                    Ok(()) => {
                        result.producer_slots.insert(key, node);
                    },
                    Err(issue) => result.producer_issues.push(issue),
                }
            } else if let Some(leaf) = leaves.get(&key) {
                let access = &mut result.tree.nodes[index].1;
                let label = access.label().map(str::to_owned);
                let bounds = access.bounds();
                let transform = access.transform().cloned();
                let children = access.children().to_vec();
                let (disabled, hidden, read_only) = (
                    access.is_disabled(),
                    access.is_hidden(),
                    access.is_read_only(),
                );
                leaf.accessibility(access);
                if let Some(label) = label {
                    access.set_label(label);
                }
                if let Some(bounds) = bounds {
                    access.set_bounds(bounds);
                } else {
                    access.clear_bounds();
                }
                if let Some(transform) = transform {
                    access.set_transform(transform);
                } else {
                    access.clear_transform();
                }
                access.set_children(children);
                if disabled {
                    access.set_disabled();
                }
                if hidden {
                    access.set_hidden();
                }
                if read_only {
                    access.set_read_only();
                }
                restrict_actions(access);
            }
        }
        if !result.tree.nodes[index].1.is_hidden() {
            result.dom_targets.insert(id, node);
        }
    }
    let by_id: HashMap<_, _> = result
        .tree
        .nodes
        .iter()
        .map(|(id, node)| (*id, node))
        .collect();
    let mut visible = HashSet::new();
    let mut pending = result
        .tree
        .tree
        .as_ref()
        .map(|tree| vec![tree.root])
        .unwrap_or_default();
    while let Some(id) = pending.pop() {
        if let Some(node) = by_id.get(&id) {
            if !node.is_hidden() && visible.insert(id) {
                pending.extend(node.children());
            }
        }
    }
    if !visible.contains(&result.tree.focus) {
        if let Some(tree) = &result.tree.tree {
            result.tree.focus = tree.root;
        }
    }
    result.dom_targets.retain(|id, _| visible.contains(id));
    result.produced_targets.retain(|id, _| visible.contains(id));
    result
}

fn restrict_actions(node: &mut accesskit::Node) {
    if node.is_disabled() || node.is_hidden() {
        node.clear_actions();
    }
    if node.is_read_only() {
        for action in [Action::SetValue, Action::Increment, Action::Decrement] {
            node.remove_action(action);
        }
    }
}

/// Write a producer's own semantics onto its slot's node, and what it draws
/// as child nodes in the host's current untransformed, scroll-adjusted content
/// space. Forward CSS transforms and clipping remain a supplier mapping gate.
/// A drawn node's actions become
/// its Button children, each taking Click (`producer` is the slot's registry
/// key, which a reader's click on one names in `produced_map`).
fn add_producer_semantics(
    tree: &mut TreeUpdate,
    slot: A11yNodeId,
    producer: u64,
    semantics: ProducerSemantics,
    content_origin: Option<(f64, f64)>,
    produced_map: &mut HashMap<A11yNodeId, ProducedAction>,
    produced_nodes: &mut HashMap<A11yNodeId, NodeTarget>,
    reserved_ids: &mut HashSet<A11yNodeId>,
) -> Result<(), ProducerSemanticIssue> {
    let Some(index) = tree.nodes.iter().position(|(id, _)| *id == slot) else {
        return Ok(());
    };
    // Reserve the complete contribution before changing the slot or any route.
    // An invalid later child must not leave an actionable prefix behind.
    reserve_producer_ids(slot, producer, &semantics, reserved_ids)?;
    let (origin, disabled, hidden) = {
        let access = &mut tree.nodes[index].1;
        if let Some(role) = semantics.role {
            access.set_role(accesskit_role(role));
        }
        if let Some(name) = semantics.name {
            access.set_label(name);
        }
        restrict_actions(access);
        (
            content_origin.filter(|_| access.bounds().is_some()),
            access.is_disabled(),
            access.is_hidden(),
        )
    };
    let mut children = Vec::new();
    let mut buttons = Vec::new();
    for child in semantics.children {
        let id = produced_node_id(slot, child.key, None);
        produced_nodes.insert(
            id,
            NodeTarget::Produced {
                slot: producer,
                key: child.key,
                action: None,
            },
        );
        let mut access = accesskit::Node::new(accesskit_role(child.role));
        access.set_label(child.name);
        let [x, y, width, height] = child.rect.map(f64::from);
        let bounds = origin
            .filter(|_| {
                [x, y, width, height].iter().all(|v| v.is_finite()) && width >= 0.0 && height >= 0.0
            })
            .map(|origin| Rect {
                x0: origin.0 + x,
                y0: origin.1 + y,
                x1: origin.0 + x + width,
                y1: origin.1 + y + height,
            });
        if let Some(bounds) = bounds {
            access.set_bounds(bounds);
        }
        if disabled {
            access.set_disabled();
        }
        if hidden {
            access.set_hidden();
        }
        for action in child.actions {
            let button_id = produced_node_id(slot, child.key, Some(&action.id));
            produced_nodes.insert(
                button_id,
                NodeTarget::Produced {
                    slot: producer,
                    key: child.key,
                    action: Some(action.id.clone()),
                },
            );
            let mut button = accesskit::Node::new(Role::Button);
            button.set_label(action.label);
            button.set_description(action.description);
            if let Some(bounds) = bounds {
                button.set_bounds(bounds);
            }
            if disabled {
                button.set_disabled();
            }
            if hidden {
                button.set_hidden();
            }
            button.add_action(Action::Click);
            button.add_action(Action::Focus);
            restrict_actions(&mut button);
            access.push_child(button_id);
            if !disabled && !hidden {
                produced_map.insert(
                    button_id,
                    ProducedAction {
                        slot: producer,
                        key: child.key,
                        id: action.id,
                    },
                );
            }
            buttons.push((button_id, button));
        }
        children.push((id, access));
    }
    let ids: Vec<A11yNodeId> = children.iter().map(|(id, _)| *id).collect();
    for id in &ids {
        tree.nodes[index].1.push_child(*id);
    }
    tree.nodes.extend(children);
    tree.nodes.extend(buttons);
    Ok(())
}

fn reserve_producer_ids(
    slot: A11yNodeId,
    producer: u64,
    semantics: &ProducerSemantics,
    reserved: &mut HashSet<A11yNodeId>,
) -> Result<(), ProducerSemanticIssue> {
    let mut keys = HashSet::new();
    let mut candidate_ids = HashSet::new();
    let mut reserve = |id| {
        if reserved.contains(&id) || !candidate_ids.insert(id) {
            Err(ProducerSemanticIssue::NodeIdCollision { producer, id })
        } else {
            Ok(())
        }
    };
    for child in &semantics.children {
        if !keys.insert(child.key) {
            return Err(ProducerSemanticIssue::DuplicateChildKey {
                producer,
                key: child.key,
            });
        }
        reserve(produced_node_id(slot, child.key, None))?;
        let mut actions = HashSet::new();
        for action in &child.actions {
            if !actions.insert(action.id.as_str()) {
                return Err(ProducerSemanticIssue::DuplicateActionId {
                    producer,
                    key: child.key,
                    id: action.id.clone(),
                });
            }
            reserve(produced_node_id(slot, child.key, Some(&action.id)))?;
        }
    }
    reserved.extend(candidate_ids);
    Ok(())
}

/// A drawn node's id, or its action button's: stable for the slot, the
/// node's key and the action, however the producer orders its nodes, in a
/// high-bit range. Composition checks it against all current DOM and generated
/// IDs. This hash is not a generation token or a collision-proof identity.
fn produced_node_id(slot: A11yNodeId, key: u64, action: Option<&str>) -> A11yNodeId {
    // FNV-1a: fixed, so an id is the same on every frame and every run.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    };
    eat(&slot.0.to_le_bytes());
    eat(&key.to_le_bytes());
    if let Some(action) = action {
        eat(&[1]);
        eat(action.as_bytes());
    }
    A11yNodeId((1 << 63) | (hash >> 1))
}

fn accesskit_role(role: ProducerRole) -> Role {
    match role {
        ProducerRole::List => Role::List,
        ProducerRole::ListItem => Role::ListItem,
        ProducerRole::Group => Role::Group,
        ProducerRole::Image => Role::Image,
        ProducerRole::GraphicsObject => Role::GraphicsObject,
    }
}

#[cfg(test)]
mod tests;
