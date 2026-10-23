// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native ROOT publication namespace. Source IDs never enter native routing.
use accesskit::{NodeId, TreeId, TreeUpdate};
use cambium_rootstock::{A11yRequest, PublicationBindings, SemanticNodeIdentity};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone)]
struct Entry {
    id: NodeId,
    identity: SemanticNodeIdentity,
}

#[derive(Default)]
pub(super) struct NativeIds {
    entries: HashMap<NodeId, Entry>,
}

pub(super) struct Prepared {
    pub tree: TreeUpdate,
    pub namespace: NativeIds,
    pub requests: HashMap<NodeId, A11yRequest>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Refusal {
    ForeignTree,
    IncompleteTree,
    DuplicateNode(NodeId),
    MissingIdentity(NodeId),
    DanglingReference(NodeId),
    NotATree(NodeId),
    UnreachableNode,
}

fn allocate_id() -> NodeId {
    // Process-wide, including across adapter/host destruction and recreation.
    // Unpublished drafts may consume IDs; no ended publication may recycle one.
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NodeId(
        NEXT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .expect("native accessibility IDs exhausted"),
    )
}

impl NativeIds {
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// A candidate only. The installed namespace advances after install/update.
    /// On failure neither current entries nor routing is modified by preparation.
    pub fn prepare(
        &self,
        mut tree: TreeUpdate,
        mut bindings: PublicationBindings,
    ) -> Result<Prepared, Refusal> {
        if tree.tree_id != TreeId::ROOT || tree.nodes.iter().any(|(_, n)| n.tree_id().is_some()) {
            return Err(Refusal::ForeignTree);
        }
        let root = tree.tree.as_ref().ok_or(Refusal::IncompleteTree)?.root;
        let mut nodes = HashMap::new();
        for (id, node) in &tree.nodes {
            if nodes.insert(*id, node).is_some() {
                return Err(Refusal::DuplicateNode(*id));
            }
        }
        // A full snapshot must not hand an orphan, cycle, or repeated child to
        // a platform consumer. Relation edges are validated by the schema hook.
        let mut visited = HashSet::new();
        let mut pending = vec![root];
        while let Some(id) = pending.pop() {
            if !visited.insert(id) {
                return Err(Refusal::NotATree(id));
            }
            let node = nodes.get(&id).ok_or(Refusal::DanglingReference(id))?;
            pending.extend(node.children());
        }
        if visited.len() != nodes.len() {
            return Err(Refusal::UnreachableNode);
        }
        let mut entries = HashMap::new();
        for (source, _) in &tree.nodes {
            let identity = bindings
                .identities
                .remove(source)
                .ok_or(Refusal::MissingIdentity(*source))?;
            let id = self
                .entries
                .get(source)
                .filter(|old| old.identity == identity)
                .map_or_else(allocate_id, |old| old.id);
            entries.insert(*source, Entry { id, identity });
        }
        let map = |source: NodeId| {
            entries
                .get(&source)
                .map(|entry| entry.id)
                .ok_or(Refusal::DanglingReference(source))
        };
        for (id, node) in &mut tree.nodes {
            *id = map(*id)?;
            node.try_map_node_ids(map)?;
        }
        tree.tree.as_mut().unwrap().root = map(root)?;
        tree.focus = map(tree.focus)?;
        let requests = bindings
            .requests
            .into_iter()
            .map(|(id, request)| Ok((map(NodeId(id))?, request)))
            .collect::<Result<HashMap<_, _>, Refusal>>()?;
        Ok(Prepared {
            tree,
            namespace: NativeIds { entries },
            requests,
        })
    }
}
