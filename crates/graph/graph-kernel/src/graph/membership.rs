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
    /// share this boundary and its one contract: insert-if-missing. An id that
    /// is already a member keeps its node and membership untouched, and the
    /// flag is `false`; callers then skip their own indices and revisions.
    pub(super) fn insert_surface_node(&mut self, node: Node) -> (NodeKey, bool) {
        let id = node.id;
        if let Some(key) = self.inner.key_of(&id) {
            return (key, false);
        }
        let key = self.inner.insert(node);
        self.surface_node_lifetimes
            .insert(id, SurfaceNodeLifetime::fresh());
        (key, true)
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
    fn adding_an_existing_id_live_keeps_the_node_and_the_url_index_honest() {
        use crate::graph::apply::{GraphDelta, add_node, apply_graph_delta};
        use crate::graph::replay_captured_deltas;
        use std::sync::{Arc, Mutex};
        let (old, new, unrelated) = (
            "https://t.test/old",
            "https://t.test/new",
            "https://t.test/x",
        );
        let mut graph = Graph::new();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = seen.clone();
        graph.set_recorder(Some(Arc::new(move |delta| {
            sink.lock().unwrap().push(delta.clone())
        })));
        let id = Uuid::from_u128(101);
        let first = add_node(&mut graph, Some(id), old.into(), Default::default());
        let (recorded, revision) = (seen.lock().unwrap().len(), graph.revision());
        let second = add_node(&mut graph, Some(id), new.into(), Default::default());
        assert_eq!(first, second);
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.get_node(first).unwrap().url(), old);
        assert_eq!(graph.get_nodes_by_url(old), vec![first]);
        assert!(graph.get_nodes_by_url(new).is_empty());
        assert_eq!(
            seen.lock().unwrap().len(),
            recorded,
            "a no-op records nothing"
        );
        assert_eq!(graph.revision(), revision);
        let replayed = replay_captured_deltas(seen.lock().unwrap().clone());
        assert_eq!(
            replayed.get_node_by_id(id).unwrap().1.url(),
            old,
            "live and replay agree"
        );
        apply_graph_delta(&mut graph, GraphDelta::RemoveNode { key: first });
        assert!(graph.get_nodes_by_url(old).is_empty());
        let other = add_node(
            &mut graph,
            Some(Uuid::from_u128(102)),
            unrelated.into(),
            Default::default(),
        );
        assert_eq!(other, first, "arena key is recycled");
        assert!(
            graph.get_nodes_by_url(old).is_empty(),
            "the old address never resolves to the recycled slot"
        );
        assert_eq!(graph.get_nodes_by_url(unrelated), vec![other]);
    }
    #[test]
    fn copy_and_snapshot_onto_an_existing_id_keep_the_first_node_and_its_index() {
        let mut graph = Graph::new();
        let id = Uuid::from_u128(5);
        let key = graph.add_node_with_id(id, "https://first.example".into(), Default::default());
        let mut donor = Graph::new();
        let source = donor.add_node_with_id(
            Uuid::from_u128(6),
            "https://donor.example".into(),
            Default::default(),
        );
        let copy = graph.copy_node_from_with_id(
            id,
            donor.get_node(source).unwrap(),
            None,
            Default::default(),
        );
        assert_eq!(copy, key);
        assert_eq!(graph.get_node(key).unwrap().url(), "https://first.example");
        assert!(graph.get_nodes_by_url("https://donor.example").is_empty());
        let mut snapshot = graph.to_snapshot();
        let mut other = Graph::new();
        other.add_node_with_id(
            Uuid::from_u128(7),
            "https://repeat.example".into(),
            Default::default(),
        );
        let mut repeated = other.to_snapshot().nodes.remove(0);
        repeated.node_id = id.to_string();
        repeated.title = "Repeat".into();
        snapshot.nodes.push(repeated);
        let restored = Graph::from_snapshot(&snapshot);
        let (restored_key, node) = restored.get_node_by_id(id).unwrap();
        assert_eq!(
            (node.url(), node.title.as_str()),
            ("https://first.example", "https://first.example")
        );
        assert_eq!(
            restored.get_nodes_by_url("https://first.example"),
            vec![restored_key]
        );
        assert!(
            restored
                .get_nodes_by_url("https://repeat.example")
                .is_empty()
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
