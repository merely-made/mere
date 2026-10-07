// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Joining independent accessibility trees as AccessKit subtrees.
//!
//! A host that shows several retained sessions in one window (a contributed
//! pane, a document tile, a session projection) publishes one tree per session
//! instead of hashing every session's nodes into its own id space, as
//! [`stitch`](crate::stitch) does. Each session keeps its own `NodeId`s. The
//! host's tree carries one *graft node* per session, and AccessKit joins them:
//! the graft's only child is the session tree's root, focus follows the graft
//! chain, bounds compose through the graft's transform, and a screen reader's
//! action names its tree in [`ActionRequest::target_tree`].
//!
//! AccessKit 0.24 implements subtrees in the consumer layer under every pinned
//! adapter (`accesskit_windows` 0.32, `accesskit_unix` 0.21, `accesskit_macos`
//! 0.26). The consumer enforces its rules by panicking, inside the adapter, on
//! the host's frame. [`Composition::new`] checks the same rules first and
//! returns a [`GraftError`] instead.
//!
//! Ruled as the stack's join (app composition brief, AC2, 2026-10-07):
//! `design_docs/cambium_docs/research/2026-10-06_app_composition_brief.md`.

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use accesskit::{ActionRequest, Affine, Node, NodeId, Rect, Role, Tree, TreeId, TreeUpdate, Uuid};

/// Derive a stable [`TreeId`] from a domain path, as [`node_id_for_path`]
/// does for nodes: the same path always names the same tree, so automation
/// and tests can pin a session's tree across runs.
///
/// The path must be unique among the trees one host shows at a time. A host
/// that can show the same session twice includes an admission generation in
/// the path. The result is never [`TreeId::ROOT`].
///
/// [`node_id_for_path`]: crate::node_id_for_path
pub fn tree_id_for_path(path: &str) -> TreeId {
    let half = |salt: &str| {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        salt.hash(&mut hasher);
        path.hash(&mut hasher);
        hasher.finish()
    };
    let (high, low) = (half("uxtree.tree.high"), half("uxtree.tree.low"));
    // The nil UUID is the root tree's id; nudge the one colliding value.
    let low = if high == 0 && low == 0 { 1 } else { low };
    TreeId(Uuid::from_u64_pair(high, low))
}

/// A graft node for `tree`, placed at `origin` in the host's coordinates with
/// the given `size`.
///
/// The origin becomes the node's transform, so the guest's tree keeps its own
/// coordinates and AccessKit offsets every bounding box below the graft by the
/// origin. The node's own bounds are `(0, 0)` to `size` for the same reason:
/// Narrator stops at a node with no bounds, so a graft always carries them.
pub fn graft_node(role: Role, tree: TreeId, origin: (f64, f64), size: (f64, f64)) -> Node {
    let mut node = Node::new(role);
    node.set_tree_id(tree);
    node.set_transform(Affine::translate(origin));
    node.set_bounds(Rect::new(0.0, 0.0, size.0, size.1));
    node
}

/// Re-address a guest's own tree update as the subtree `tree`.
///
/// A guest builds its update as if it were a whole window: rooted, with tree
/// data, addressed to [`TreeId::ROOT`]. A subtree's first update must carry
/// tree data, so an update without it gets one rooted at the guest's first
/// node that no other node lists as a child.
pub fn as_subtree(mut update: TreeUpdate, tree: TreeId) -> TreeUpdate {
    update.tree_id = tree;
    if update.tree.is_none() {
        let children: HashSet<NodeId> = update
            .nodes
            .iter()
            .flat_map(|(_, node)| node.children().iter().copied())
            .collect();
        if let Some((root, _)) = update.nodes.iter().find(|(id, _)| !children.contains(id)) {
            update.tree = Some(Tree::new(*root));
        }
    }
    update
}

/// A host frame's trees broke a rule the AccessKit consumer enforces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraftError {
    /// A guest's tree has no graft node in the host's update.
    MissingGraft(TreeId),
    /// A graft node in the host's update has no guest tree this frame.
    MissingGuest(TreeId),
    /// Two graft nodes name the same tree.
    DuplicateGraft {
        tree: TreeId,
        first: NodeId,
        second: NodeId,
    },
    /// Two guests share one tree id.
    DuplicateGuest(TreeId),
    /// A graft node lists children; its only child comes from its subtree.
    GraftWithChildren(NodeId),
    /// A guest was addressed to the root tree, or to the host's own tree.
    NotASubtree(TreeId),
    /// A guest's update has no nodes, so it has no root.
    EmptyGuest(TreeId),
}

impl std::fmt::Display for GraftError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingGraft(tree) => write!(f, "guest tree {tree:?} has no graft node"),
            Self::MissingGuest(tree) => write!(f, "graft for {tree:?} has no guest tree"),
            Self::DuplicateGraft {
                tree,
                first,
                second,
            } => write!(f, "tree {tree:?} is grafted at {first:?} and {second:?}"),
            Self::DuplicateGuest(tree) => write!(f, "two guests share tree {tree:?}"),
            Self::GraftWithChildren(node) => write!(f, "graft node {node:?} lists children"),
            Self::NotASubtree(tree) => write!(f, "guest addressed to {tree:?} is not a subtree"),
            Self::EmptyGuest(tree) => write!(f, "guest tree {tree:?} has no nodes"),
        }
    }
}

impl std::error::Error for GraftError {}

/// One frame of a host's accessibility: its own tree and every grafted guest.
///
/// A composition is whole. Every graft in the host's update has its guest,
/// and every guest has its graft, so the adapter never sees a graft without a
/// subtree or a subtree without a graft. To retire a guest, leave both its
/// graft and its tree out of the next frame: AccessKit drops the subtree with
/// its graft. [`Grafts`] turns compositions into the adapter's updates.
#[derive(Clone, Debug, PartialEq)]
pub struct Composition {
    host: TreeUpdate,
    guests: Vec<TreeUpdate>,
}

impl Composition {
    /// Check `host` and `guests` against the consumer's rules and address each
    /// guest to its tree.
    pub fn new(host: TreeUpdate, guests: Vec<(TreeId, TreeUpdate)>) -> Result<Self, GraftError> {
        let mut grafts: HashMap<TreeId, NodeId> = HashMap::new();
        for (id, node) in &host.nodes {
            let Some(tree) = node.tree_id() else {
                continue;
            };
            if !node.children().is_empty() {
                return Err(GraftError::GraftWithChildren(*id));
            }
            if let Some(first) = grafts.insert(tree, *id) {
                return Err(GraftError::DuplicateGraft {
                    tree,
                    first,
                    second: *id,
                });
            }
        }

        let mut seen = HashSet::new();
        let mut addressed = Vec::with_capacity(guests.len());
        for (tree, update) in guests {
            if tree == TreeId::ROOT || tree == host.tree_id {
                return Err(GraftError::NotASubtree(tree));
            }
            if !seen.insert(tree) {
                return Err(GraftError::DuplicateGuest(tree));
            }
            if !grafts.contains_key(&tree) {
                return Err(GraftError::MissingGraft(tree));
            }
            if update.nodes.is_empty() {
                return Err(GraftError::EmptyGuest(tree));
            }
            addressed.push(as_subtree(update, tree));
        }
        if let Some(tree) = grafts.keys().find(|tree| !seen.contains(*tree)) {
            return Err(GraftError::MissingGuest(*tree));
        }

        Ok(Self {
            host,
            guests: addressed,
        })
    }

    /// The host's tree this frame.
    pub fn host(&self) -> &TreeUpdate {
        &self.host
    }
}

/// What one adapter holds between frames: the subtrees it has received, the
/// host's root, and the host focus it last applied.
///
/// AccessKit refuses focus on a graft whose subtree it has not received, and
/// a graft must exist before its subtree. So a frame that grafts a new guest
/// and focuses it in one go cannot send the host's focus with the host's
/// update. [`frame`](Self::frame) sends the host's update with the focus it
/// last applied, then the new guests, then one focus-only update, then the
/// guests the adapter already holds, so the screen reader hears one focus
/// move.
#[derive(Clone, Debug, Default)]
pub struct Grafts {
    live: HashSet<TreeId>,
    root: Option<NodeId>,
    focus: Option<NodeId>,
}

impl Grafts {
    pub fn new() -> Self {
        Self::default()
    }

    /// The update for an adapter's activation request, which takes the host's
    /// tree before any subtree exists. A focus on a graft moves to the host's
    /// root. The adapter holds no subtrees afterwards, so the next
    /// [`frame`](Self::frame) sends every guest as new.
    pub fn activate(&mut self, composition: &Composition) -> TreeUpdate {
        self.live.clear();
        let mut host = composition.host.clone();
        if let Some(tree) = &host.tree {
            self.root = Some(tree.root);
        }
        if grafted(&host).contains_key(&host.focus)
            && let Some(root) = self.root
        {
            host.focus = root;
        }
        self.focus = Some(host.focus);
        host
    }

    /// The frame's updates in the order the adapter must apply them:
    ///
    /// 1. the host's, so every graft exists before its subtree arrives;
    /// 2. each guest the adapter does not hold yet;
    /// 3. if the host's focus is on one of those, one update that moves the
    ///    focus there;
    /// 4. each guest the adapter already holds, so a guest that loses the
    ///    focus this frame changes its own focus only after the focus has
    ///    left it.
    ///
    /// Within 2 and 4, guests keep the composition's order.
    pub fn frame(&mut self, composition: Composition) -> Vec<TreeUpdate> {
        let Composition { mut host, guests } = composition;
        if let Some(tree) = &host.tree {
            self.root = Some(tree.root);
        }
        let grafts = grafted(&host);
        let wanted = host.focus;
        let waits = grafts
            .get(&wanted)
            .is_some_and(|tree| !self.live.contains(tree));
        if waits {
            let still_there = |id: &NodeId| {
                host.nodes.iter().any(|(node_id, node)| {
                    node_id == id && node.tree_id().is_none_or(|tree| self.live.contains(&tree))
                })
            };
            host.focus = self
                .focus
                .filter(still_there)
                .or(self.root)
                .unwrap_or(wanted);
        }
        let host_tree = host.tree_id;
        let (held, new): (Vec<_>, Vec<_>) = guests
            .into_iter()
            .partition(|guest| self.live.contains(&guest.tree_id));
        self.live = held.iter().chain(&new).map(|guest| guest.tree_id).collect();
        let mut updates = Vec::with_capacity(held.len() + new.len() + 2);
        updates.push(host);
        updates.extend(new);
        if waits {
            updates.push(TreeUpdate {
                nodes: Vec::new(),
                tree: None,
                tree_id: host_tree,
                focus: wanted,
            });
        }
        updates.extend(held);
        self.focus = Some(wanted);
        updates
    }
}

fn grafted(host: &TreeUpdate) -> HashMap<NodeId, TreeId> {
    host.nodes
        .iter()
        .filter_map(|(id, node)| node.tree_id().map(|tree| (*id, tree)))
        .collect()
}

/// Where a screen reader's action goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionTarget<'a, K> {
    /// The host's own tree.
    Host,
    /// A guest's tree; the request's `target_node` is in the guest's own ids.
    Guest(&'a K),
    /// A tree this host does not know, such as a guest retired since the
    /// screen reader last read the tree. Drop the request.
    Unknown(TreeId),
}

/// The host's table from guest tree to the session that owns it.
#[derive(Clone, Debug)]
pub struct GraftTable<K> {
    host: TreeId,
    guests: HashMap<TreeId, K>,
}

impl<K> GraftTable<K> {
    /// A table for a host whose own tree is `host`, usually [`TreeId::ROOT`].
    pub fn new(host: TreeId) -> Self {
        Self {
            host,
            guests: HashMap::new(),
        }
    }

    /// Record `key` as the owner of `tree`, returning any earlier owner.
    pub fn insert(&mut self, tree: TreeId, key: K) -> Option<K> {
        self.guests.insert(tree, key)
    }

    /// Forget `tree`, returning its owner.
    pub fn remove(&mut self, tree: TreeId) -> Option<K> {
        self.guests.remove(&tree)
    }

    /// The owner of `tree`.
    pub fn get(&self, tree: TreeId) -> Option<&K> {
        self.guests.get(&tree)
    }

    /// Route `request` by its `target_tree`.
    pub fn route(&self, request: &ActionRequest) -> ActionTarget<'_, K> {
        if request.target_tree == self.host {
            return ActionTarget::Host;
        }
        match self.guests.get(&request.target_tree) {
            Some(key) => ActionTarget::Guest(key),
            None => ActionTarget::Unknown(request.target_tree),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accesskit::Action;

    fn node(role: Role, children: &[u64]) -> Node {
        let mut node = Node::new(role);
        node.set_children(children.iter().copied().map(NodeId).collect::<Vec<_>>());
        node
    }

    fn host_with(grafts: &[(u64, TreeId)]) -> TreeUpdate {
        let mut nodes = vec![(
            NodeId(1),
            node(
                Role::Window,
                &grafts.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            ),
        )];
        for (id, tree) in grafts {
            nodes.push((
                NodeId(*id),
                graft_node(Role::Pane, *tree, (0.0, 0.0), (10.0, 10.0)),
            ));
        }
        TreeUpdate {
            nodes,
            tree: Some(Tree::new(NodeId(1))),
            tree_id: TreeId::ROOT,
            focus: NodeId(1),
        }
    }

    fn guest() -> TreeUpdate {
        TreeUpdate {
            nodes: vec![
                (NodeId(1), node(Role::Group, &[2])),
                (NodeId(2), node(Role::Button, &[])),
            ],
            tree: None,
            tree_id: TreeId::ROOT,
            focus: NodeId(1),
        }
    }

    #[test]
    fn tree_ids_are_stable_distinct_and_never_root() {
        assert_eq!(tree_id_for_path("pane/a#1"), tree_id_for_path("pane/a#1"));
        assert_ne!(tree_id_for_path("pane/a#1"), tree_id_for_path("pane/a#2"));
        assert_ne!(tree_id_for_path(""), TreeId::ROOT);
    }

    #[test]
    fn graft_node_carries_tree_origin_and_local_bounds() {
        let tree = tree_id_for_path("pane/a");
        let graft = graft_node(Role::Pane, tree, (40.0, 20.0), (100.0, 50.0));
        assert_eq!(graft.tree_id(), Some(tree));
        assert_eq!(graft.transform(), Some(&Affine::translate((40.0, 20.0))));
        assert_eq!(graft.bounds(), Some(Rect::new(0.0, 0.0, 100.0, 50.0)));
        assert!(graft.children().is_empty());
    }

    #[test]
    fn as_subtree_addresses_the_update_and_finds_its_root() {
        let tree = tree_id_for_path("pane/a");
        let update = as_subtree(guest(), tree);
        assert_eq!(update.tree_id, tree);
        assert_eq!(update.tree.map(|tree| tree.root), Some(NodeId(1)));
    }

    #[test]
    fn a_frame_sends_the_host_first() {
        let (a, b) = (tree_id_for_path("a"), tree_id_for_path("b"));
        let composition = Composition::new(
            host_with(&[(10, a), (11, b)]),
            vec![(b, guest()), (a, guest())],
        )
        .unwrap();
        let updates = Grafts::new().frame(composition);
        let order: Vec<TreeId> = updates.iter().map(|update| update.tree_id).collect();
        assert_eq!(order, vec![TreeId::ROOT, b, a]);
    }

    #[test]
    fn composition_refuses_what_the_consumer_would_panic_on() {
        let (a, b) = (tree_id_for_path("a"), tree_id_for_path("b"));
        assert_eq!(
            Composition::new(host_with(&[(10, a)]), vec![(a, guest()), (b, guest())]),
            Err(GraftError::MissingGraft(b))
        );
        assert_eq!(
            Composition::new(host_with(&[(10, a), (11, b)]), vec![(a, guest())]),
            Err(GraftError::MissingGuest(b))
        );
        assert_eq!(
            Composition::new(host_with(&[(10, a), (11, a)]), vec![(a, guest())]),
            Err(GraftError::DuplicateGraft {
                tree: a,
                first: NodeId(10),
                second: NodeId(11),
            })
        );
        assert_eq!(
            Composition::new(host_with(&[(10, a)]), vec![(a, guest()), (a, guest())]),
            Err(GraftError::DuplicateGuest(a))
        );
        assert_eq!(
            Composition::new(host_with(&[]), vec![(TreeId::ROOT, guest())]),
            Err(GraftError::NotASubtree(TreeId::ROOT))
        );
        let mut parent = host_with(&[(10, a)]);
        parent.nodes[1].1.set_children(vec![NodeId(99)]);
        assert_eq!(
            Composition::new(parent, vec![(a, guest())]),
            Err(GraftError::GraftWithChildren(NodeId(10)))
        );
        let empty = TreeUpdate {
            nodes: Vec::new(),
            ..guest()
        };
        assert_eq!(
            Composition::new(host_with(&[(10, a)]), vec![(a, empty)]),
            Err(GraftError::EmptyGuest(a))
        );
    }

    #[test]
    fn activation_moves_focus_off_a_graft() {
        let a = tree_id_for_path("a");
        let mut host = host_with(&[(10, a)]);
        host.focus = NodeId(10);
        let composition = Composition::new(host, vec![(a, guest())]).unwrap();
        let mut grafts = Grafts::new();
        assert_eq!(grafts.activate(&composition).focus, NodeId(1));
        // The new subtree arrives before the focus that names its graft.
        let updates = grafts.frame(composition.clone());
        let focus: Vec<(TreeId, NodeId)> = updates
            .iter()
            .map(|update| (update.tree_id, update.focus))
            .collect();
        assert_eq!(
            focus,
            vec![
                (TreeId::ROOT, NodeId(1)),
                (a, NodeId(1)),
                (TreeId::ROOT, NodeId(10))
            ]
        );
        // Once the adapter holds it, the host's update carries the focus.
        let updates = grafts.frame(composition);
        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].focus, NodeId(10));
    }

    #[test]
    fn table_routes_by_target_tree() {
        let (a, gone) = (tree_id_for_path("a"), tree_id_for_path("gone"));
        let mut table = GraftTable::new(TreeId::ROOT);
        table.insert(a, "session a");
        let request = |target_tree| ActionRequest {
            action: Action::Click,
            target_tree,
            target_node: NodeId(2),
            data: None,
        };
        assert_eq!(table.route(&request(TreeId::ROOT)), ActionTarget::Host);
        assert_eq!(table.route(&request(a)), ActionTarget::Guest(&"session a"));
        assert_eq!(table.route(&request(gone)), ActionTarget::Unknown(gone));
        assert_eq!(table.remove(a), Some("session a"));
        assert_eq!(table.route(&request(a)), ActionTarget::Unknown(a));
    }
}
