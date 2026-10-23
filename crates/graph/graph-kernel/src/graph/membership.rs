// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Runtime membership witnesses, authored at the kernel's insertion boundary.
//! These are not persisted identity, a graph revision, or an authorization lease.
use super::{Graph, Node, NodeKey};
use uuid::Uuid;

/// One live surface membership. Cloning a graph preserves it; removing and
/// recreating a member creates a new witness, even with the same UUID/key.
/// Snapshot materialization starts new runtime lives. There is no numeric or
/// serialization API: durable identity remains the member UUID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SurfaceNodeLifetime(u64);

impl SurfaceNodeLifetime {
    fn fresh() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(
            NEXT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
                .expect("surface membership lifetime space exhausted"),
        )
    }
}

impl Graph {
    /// Observe membership without allocating or changing its life.
    pub fn surface_node_lifetime(&self, id: Uuid) -> Option<SurfaceNodeLifetime> {
        self.inner.key_of(&id)?;
        self.surface_node_lifetimes.get(&id).copied()
    }

    /// All surface insertions, including copy and snapshot materialization,
    /// share this boundary. An upsert is an edit of an existing membership.
    pub(super) fn insert_surface_node(&mut self, node: Node) -> NodeKey {
        let id = node.id;
        let key = self.inner.insert(node);
        self.surface_node_lifetimes
            .entry(id)
            .or_insert_with(SurfaceNodeLifetime::fresh);
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordinary_edits_upserts_and_clones_keep_membership_but_recreation_does_not() {
        let mut graph = Graph::new();
        let id = Uuid::from_u128(1);
        let key = graph.add_node_with_id(id, "https://original.example".into(), Default::default());
        let old = graph.surface_node_lifetime(id).unwrap();
        let mut candidate = graph.clone();
        candidate.set_node_title(key, "Ordinary edit".into());
        assert_eq!(candidate.surface_node_lifetime(id), Some(old));
        assert_eq!(
            candidate.add_node_with_id(id, "https://updated.example".into(), Default::default()),
            key
        );
        assert_eq!(candidate.surface_node_lifetime(id), Some(old));
        assert!(candidate.remove_node(key));
        assert_eq!(candidate.surface_node_lifetime(id), None);
        let next =
            candidate.add_node_with_id(id, "https://recreated.example".into(), Default::default());
        assert_eq!(key, next, "exercise graph slot reuse");
        assert_ne!(candidate.surface_node_lifetime(id), Some(old));
        assert_eq!(
            graph.surface_node_lifetime(id),
            Some(old),
            "detached candidate cannot mutate live identity"
        );
    }
    #[test]
    fn copies_share_the_insert_boundary_and_remove_never_keeps_a_dead_witness() {
        let mut donor = Graph::new();
        let source_id = Uuid::from_u128(2);
        let source = donor.add_node_with_id(
            source_id,
            "https://source.example".into(),
            Default::default(),
        );
        let mut graph = Graph::new();
        let id = Uuid::from_u128(3);
        let key = graph.copy_node_from_with_id(
            id,
            donor.get_node(source).unwrap(),
            None,
            Default::default(),
        );
        let old = graph.surface_node_lifetime(id).unwrap();
        assert_eq!(
            graph.copy_node_from_with_id(
                id,
                donor.get_node(source).unwrap(),
                None,
                Default::default()
            ),
            key
        );
        assert_eq!(graph.surface_node_lifetime(id), Some(old));
        assert!(graph.remove_node(key));
        assert!(!graph.remove_node(key));
        assert!(graph.surface_node_lifetimes.is_empty());
        graph.copy_node_from_with_id(
            id,
            donor.get_node(source).unwrap(),
            None,
            Default::default(),
        );
        assert_ne!(graph.surface_node_lifetime(id), Some(old));
        assert_ne!(
            graph.surface_node_lifetime(id),
            donor.surface_node_lifetime(source_id)
        );
    }
    #[test]
    fn persisted_content_restores_with_new_runtime_membership() {
        let mut graph = Graph::new();
        let id = Uuid::from_u128(4);
        graph.add_node_with_id(id, "https://persisted.example".into(), Default::default());
        let snapshot = graph.to_snapshot();
        let restored = Graph::from_snapshot(&snapshot);
        assert!(restored.get_node_by_id(id).is_some());
        assert_eq!(
            restored.get_node_by_id(id).unwrap().1.title,
            graph.get_node_by_id(id).unwrap().1.title
        );
        assert_ne!(
            graph.surface_node_lifetime(id),
            restored.surface_node_lifetime(id)
        );
    }
}
