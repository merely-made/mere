// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Graph query + traversal — node / edge lookups, neighbor
//! iteration, BFS / shortest-path / connected-components / SCC, and
//! the derived-edge views (semantic / arrangement / containment).
//!
//! Extracted from `graph/mod.rs` per the 2026-05-11 kernel
//! decomposition pass. These are the read-only and read-mostly
//! methods on `Graph`; the mutating counterparts live in `mod.rs`
//! (node-property setters), `edge_ops.rs` (edge mutators), and
//! `snapshot.rs` (serialization).

use std::collections::{HashMap, HashSet, VecDeque};

use petgraph::Direction;
use petgraph::algo::{astar, dijkstra, has_path_connecting, kosaraju_scc};
use petgraph::visit::{EdgeRef, IntoEdgeReferences, UndirectedAdaptor};
use uuid::Uuid;

use super::edge_payload::EdgePayload;
use super::edge_taxonomy::{ContainmentSubKind, EdgeAssertion, RelationSelector};
use super::identity::{EdgeKey, NodeKey};
use super::node::Node;
use super::snapshot::containment_parent_url;
use super::{ArrangementEdgeView, ContainmentEdgeView, Graph, RelationView, SemanticEdgeView};

/// Expand one pair-local edge bucket into its [`RelationView`] rows: one
/// per semantic statement (`Semantic(sub_kind)` when recognized, else
/// `OpenPredicate`), one for traversal-sidecar presence, and one per
/// sub-kind in each of the remaining families. Shared by every relation
/// iterator so per-node and whole-graph reads agree row for row.
///
/// Row families agree with [`EdgePayload::families`] for every edge
/// shape (Q1 ruling, 2026-09-12; pinned by `row_family_parity_tests`).
fn relation_rows(from: NodeKey, to: NodeKey, payload: &EdgePayload) -> Vec<RelationView> {
    use super::edge_taxonomy::RelationKind;
    let mut out: Vec<RelationView> = Vec::new();
    for statement in payload.semantic_statements() {
        let kind = match statement.recognized_sub_kind {
            Some(sub_kind) => RelationKind::Semantic(sub_kind),
            None => RelationKind::OpenPredicate,
        };
        out.push(RelationView { from, to, kind });
    }
    // Traversal is event-shaped, no sub-kind. Presence of the
    // traversal sidecar yields one row, matching
    // `has_family(Traversal)`; an event-free sidecar (reachable
    // only by deserializing such a snapshot) still counts.
    // Family-aware view callers reach for the typed payload for
    // actual events.
    if payload.traversal_data().is_some() {
        out.push(RelationView {
            from,
            to,
            kind: RelationKind::Traversal,
        });
    }
    if let Some(containment) = payload.containment_data() {
        for &sub_kind in &containment.sub_kinds {
            out.push(RelationView {
                from,
                to,
                kind: RelationKind::Containment(sub_kind),
            });
        }
    }
    if let Some(arrangement) = payload.arrangement_data() {
        for &sub_kind in &arrangement.sub_kinds {
            out.push(RelationView {
                from,
                to,
                kind: RelationKind::Arrangement(sub_kind),
            });
        }
    }
    if let Some(imported) = payload.imported_data() {
        for &sub_kind in &imported.sub_kinds {
            out.push(RelationView {
                from,
                to,
                kind: RelationKind::Imported(sub_kind),
            });
        }
    }
    if let Some(prov) = payload.provenance_data() {
        for &sub_kind in &prov.sub_kinds {
            out.push(RelationView {
                from,
                to,
                kind: RelationKind::Provenance(sub_kind),
            });
        }
    }
    out
}

impl Graph {
    /// Get a node by key
    pub fn get_node(&self, key: NodeKey) -> Option<&Node> {
        self.inner.node(key)
    }

    /// Get a mutable node by key.
    pub(crate) fn get_node_mut(&mut self, key: NodeKey) -> Option<&mut Node> {
        self.inner.node_mut(key)
    }

    /// Find a node whose AddressClaims include the given address (any
    /// `AddressRole`). Returns an arbitrary matching node when multiple
    /// nodes share the address — the caller is expected to handle
    /// duplicates explicitly when relevant.
    ///
    /// Canonical address lookup per the [node identity + duplicates
    /// brief](https://github.com/merely-made/mere/blob/main/design_docs/mere_docs/research/2026-05-18_node_identity_and_duplicates_brief.md);
    /// supersedes the URL-string-keyed `get_node_by_url` (which is
    /// retained as a transitional shim — see below).
    pub fn find_node_by_address(
        &self,
        address: &crate::address::Address,
    ) -> Option<(NodeKey, &Node)> {
        let url = address.as_url_str();
        let key = self.url_to_nodes.get(url)?.last().copied()?;
        Some((key, self.inner.node(key)?))
    }

    /// Convenience wrapper over [`Graph::find_node_by_address`] for
    /// callers that have a URL string rather than a typed [`Address`].
    /// Builds the [`Address`] internally and delegates.
    ///
    /// Use this for tests and quick lookups; prefer
    /// [`Graph::find_node_by_address`] in code paths that already have a
    /// typed [`Address`] in hand.
    pub fn get_node_by_url(&self, url: &str) -> Option<(NodeKey, &Node)> {
        let address = crate::address::address_from_url(url);
        self.find_node_by_address(&address)
    }

    /// Get all node keys currently mapped to a URL.
    pub fn get_nodes_by_url(&self, url: &str) -> Vec<NodeKey> {
        self.url_to_nodes.get(url).cloned().unwrap_or_default()
    }

    /// Get a node by UUID.
    pub fn get_node_by_id(&self, id: Uuid) -> Option<(NodeKey, &Node)> {
        let key = self.inner.key_of(&id)?;
        Some((key, self.inner.node(key)?))
    }

    /// Get node key by UUID.
    pub fn get_node_key_by_id(&self, id: Uuid) -> Option<NodeKey> {
        self.inner.key_of(&id)
    }

    /// Iterate over all nodes as (key, node) pairs
    pub fn nodes(&self) -> impl Iterator<Item = (NodeKey, &Node)> {
        self.inner.nodes()
    }

    /// Iterate over all stored relations, one row per recognized
    /// statement/family carried by the pair-local edge bucket.
    /// One `EdgePayload` between two nodes may therefore yield
    /// multiple semantic rows.
    ///
    /// Canonical read surface per the 2026-05-11 relation-taxonomy
    /// plan §2. Stage 4 removed the legacy `EdgeType`-flavoured
    /// `edges()` iterator that this replaces.
    pub fn relations(&self) -> impl Iterator<Item = RelationView> + '_ {
        self.inner
            .inner()
            .edge_references()
            .flat_map(|edge| relation_rows(edge.source(), edge.target(), edge.weight()))
    }

    /// Relations whose `from` endpoint is `key`, expanded with exactly the
    /// row semantics of [`Self::relations`] but visiting only that node's
    /// outgoing edges. Cost is proportional to the node's out-degree, not
    /// the graph's edge count.
    pub fn outgoing_relations(&self, key: NodeKey) -> impl Iterator<Item = RelationView> + '_ {
        self.inner
            .inner()
            .edges_directed(key, Direction::Outgoing)
            .flat_map(|edge| relation_rows(edge.source(), edge.target(), edge.weight()))
    }

    /// Relations whose `to` endpoint is `key`, expanded with exactly the
    /// row semantics of [`Self::relations`] but visiting only that node's
    /// incoming edges. Cost is proportional to the node's in-degree, not
    /// the graph's edge count.
    pub fn incoming_relations(&self, key: NodeKey) -> impl Iterator<Item = RelationView> + '_ {
        self.inner
            .inner()
            .edges_directed(key, Direction::Incoming)
            .flat_map(|edge| relation_rows(edge.source(), edge.target(), edge.weight()))
    }

    pub fn semantic_edges(&self) -> impl Iterator<Item = SemanticEdgeView> + '_ {
        self.inner.inner().edge_references().flat_map(|edge| {
            let from = edge.source();
            let to = edge.target();
            let payload = edge.weight();
            payload
                .semantic_statements()
                .iter()
                .filter_map(|statement| {
                    statement
                        .recognized_sub_kind
                        .map(|sub_kind| SemanticEdgeView {
                            from,
                            to,
                            sub_kind,
                            label: statement.label.clone(),
                        })
                })
                .collect::<Vec<_>>()
                .into_iter()
        })
    }

    pub fn arrangement_edges(&self) -> impl Iterator<Item = ArrangementEdgeView> + '_ {
        self.inner.inner().edge_references().flat_map(|e| {
            let from = e.source();
            let to = e.target();
            e.weight()
                .arrangement_data()
                .map(|data| {
                    data.sub_kinds
                        .iter()
                        .copied()
                        .map(move |sub_kind| ArrangementEdgeView { from, to, sub_kind })
                })
                .into_iter()
                .flatten()
        })
    }

    pub fn containment_edges(&self) -> impl Iterator<Item = ContainmentEdgeView> + '_ {
        self.inner.inner().edge_references().flat_map(|e| {
            let from = e.source();
            let to = e.target();
            e.weight()
                .containment_data()
                .map(|data| {
                    data.sub_kinds
                        .iter()
                        .copied()
                        .map(move |sub_kind| ContainmentEdgeView { from, to, sub_kind })
                })
                .into_iter()
                .flatten()
        })
    }

    /// Rebuild derived containment edges from current node URLs.
    ///
    /// Derived relations are additive/read-only and are never persisted.
    /// Derive the containment this one node's address implies, at the moment
    /// the node is created.
    ///
    /// The **UrlPath half** of [`rebuild_derived_containment_relations`],
    /// applied to a single node. It exists because that rebuild runs only on
    /// snapshot load, which used to mean a node visited under a folder had no
    /// containment edge until the session was saved and reloaded. Anything
    /// reading containment live (a graph behavior's watch, which names a
    /// region by its container) saw nothing there. Deriving at creation makes
    /// containment mean the same thing live and loaded.
    ///
    /// The **Domain half is deliberately not here.** A domain anchor is the
    /// shallowest node on a host, so minting one node can re-anchor every
    /// other node sharing that host: a whole-graph fact, which stays with the
    /// whole-graph rebuild. This one is purely local, which is what makes it
    /// cheap enough to run per node.
    pub fn derive_containment_for(&mut self, key: NodeKey) {
        let Some(node) = self.get_node(key) else {
            return;
        };
        let Ok(parsed) = url::Url::parse(node.url()) else {
            return;
        };
        let Some(parent_url) = containment_parent_url(&parsed) else {
            return;
        };
        let Some((parent_key, _)) = self.get_node_by_url(&parent_url) else {
            return;
        };
        if parent_key == key {
            return;
        }
        // Child to container, the direction the rebuild uses.
        let _ = self.assert_relation(
            key,
            parent_key,
            EdgeAssertion::Containment {
                sub_kind: ContainmentSubKind::UrlPath,
            },
        );
    }

    pub(crate) fn rebuild_derived_containment_relations(&mut self) {
        let edge_ids: Vec<EdgeKey> = self.inner.inner().edge_indices().collect();
        let mut empty_edges = Vec::new();
        // This rebuild retracts containment relations and removes empty edges directly on `inner`
        // (bypassing the bumping API), so track the structural change and bump once. The
        // `assert_relation` calls below bump on their own when they re-derive the edges.
        let mut changed = false;
        for edge_id in edge_ids {
            if let Some(payload) = self.inner.edge_mut(edge_id) {
                let mut removed_any = false;
                removed_any |= payload
                    .retract_relation(RelationSelector::Containment(ContainmentSubKind::UrlPath));
                removed_any |= payload
                    .retract_relation(RelationSelector::Containment(ContainmentSubKind::Domain));
                if removed_any {
                    changed = true;
                }
                if removed_any && payload.is_empty() {
                    empty_edges.push(edge_id);
                }
            }
        }
        for edge_id in empty_edges {
            let _ = self.inner.disconnect(edge_id);
        }
        if changed {
            self.bump_revision();
        }

        let mut domain_anchor: HashMap<String, (NodeKey, usize, Uuid)> = HashMap::new();
        for (key, node) in self.nodes() {
            let Ok(parsed) = url::Url::parse(node.url()) else {
                continue;
            };
            let depth = parsed.path_segments().map_or(0, |segments| {
                segments.filter(|segment| !segment.is_empty()).count()
            });

            let Some(host) = parsed.host_str() else {
                continue;
            };
            let host = host.to_ascii_lowercase();
            let candidate = (key, depth, node.id);
            domain_anchor
                .entry(host)
                .and_modify(|current| {
                    if candidate.1 < current.1
                        || (candidate.1 == current.1 && candidate.2 < current.2)
                    {
                        *current = candidate;
                    }
                })
                .or_insert(candidate);
        }

        let mut url_parent_edges = Vec::new();
        let mut domain_edges = Vec::new();
        for (key, node) in self.nodes() {
            let Ok(parsed) = url::Url::parse(node.url()) else {
                continue;
            };

            if let Some(parent_url) = containment_parent_url(&parsed)
                && let Some((parent_key, _)) = self.get_node_by_url(&parent_url)
                && parent_key != key
            {
                url_parent_edges.push((key, parent_key));
            }

            if let Some(host) = parsed.host_str() {
                let host = host.to_ascii_lowercase();
                if let Some((anchor_key, _, _)) = domain_anchor.get(&host)
                    && *anchor_key != key
                {
                    domain_edges.push((key, *anchor_key));
                }
            }
        }

        for (from, to) in url_parent_edges {
            let _ = self.assert_relation(
                from,
                to,
                EdgeAssertion::Containment {
                    sub_kind: ContainmentSubKind::UrlPath,
                },
            );
        }
        for (from, to) in domain_edges {
            let _ = self.assert_relation(
                from,
                to,
                EdgeAssertion::Containment {
                    sub_kind: ContainmentSubKind::Domain,
                },
            );
        }
    }

    /// Iterate outgoing neighbor keys for a node
    pub fn out_neighbors(&self, key: NodeKey) -> impl Iterator<Item = NodeKey> + '_ {
        self.inner
            .inner()
            .neighbors_directed(key, Direction::Outgoing)
    }

    /// Iterate incoming neighbor keys for a node
    pub fn in_neighbors(&self, key: NodeKey) -> impl Iterator<Item = NodeKey> + '_ {
        self.inner
            .inner()
            .neighbors_directed(key, Direction::Incoming)
    }

    /// Iterate undirected neighbor keys for a node.
    pub fn neighbors_undirected(&self, key: NodeKey) -> impl Iterator<Item = NodeKey> + '_ {
        self.inner.inner().neighbors_undirected(key)
    }

    /// Undirected neighbors sorted by stable node-key order.
    pub fn neighbors_undirected_sorted(&self, key: NodeKey) -> Vec<NodeKey> {
        let mut neighbors: Vec<NodeKey> = self
            .neighbors_undirected(key)
            .filter(|neighbor| *neighbor != key && self.get_node(*neighbor).is_some())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        neighbors.sort_by_key(|neighbor| neighbor.index());
        neighbors
    }

    /// Seed nodes plus one-hop undirected neighbors for frame import workflows.
    pub fn connected_frame_import_nodes(&self, seeds: &[NodeKey]) -> Vec<NodeKey> {
        let mut out = HashSet::new();
        for seed in seeds {
            if self.get_node(*seed).is_none() {
                continue;
            }
            out.insert(*seed);
            out.extend(self.neighbors_undirected(*seed));
        }
        let mut nodes: Vec<NodeKey> = out
            .into_iter()
            .filter(|key| self.get_node(*key).is_some())
            .collect();
        nodes.sort_by_key(|key| key.index());
        nodes
    }

    /// Connected candidate expansion around a source node with depth annotations.
    ///
    /// `max_depth` currently supports 1 or 2, matching connected-open scope behavior.
    pub fn connected_candidates_with_depth(
        &self,
        source: NodeKey,
        max_depth: u8,
    ) -> Vec<(NodeKey, u8)> {
        if self.get_node(source).is_none() || max_depth == 0 {
            return Vec::new();
        }

        let mut out = Vec::new();
        let mut visited = HashSet::from([source]);

        let depth1 = self.neighbors_undirected_sorted(source);
        for neighbor in depth1 {
            if visited.insert(neighbor) {
                out.push((neighbor, 1));
            }
        }

        if max_depth < 2 {
            return out;
        }

        let depth1_nodes: Vec<NodeKey> = out
            .iter()
            .filter_map(|(node, depth)| (*depth == 1).then_some(*node))
            .collect();
        for depth1_node in depth1_nodes {
            for neighbor in self.neighbors_undirected_sorted(depth1_node) {
                if visited.insert(neighbor) {
                    out.push((neighbor, 2));
                }
            }
        }

        out
    }

    /// Undirected hop distances from `source` using unit edge weights.
    pub fn hop_distances_from(&self, source: NodeKey) -> HashMap<NodeKey, usize> {
        if self.get_node(source).is_none() {
            return HashMap::new();
        }
        dijkstra(&UndirectedAdaptor(self.inner.inner()), source, None, |_| {
            1_usize
        })
        .into_iter()
        .collect()
    }

    /// Nodes with no incoming or outgoing edges.
    pub fn orphan_node_keys(&self) -> Vec<NodeKey> {
        let inner = self.inner.inner();
        inner
            .node_indices()
            .filter(|&key| {
                inner
                    .edges_directed(key, Direction::Outgoing)
                    .next()
                    .is_none()
                    && inner
                        .edges_directed(key, Direction::Incoming)
                        .next()
                        .is_none()
            })
            .collect()
    }

    /// Shortest undirected path between two nodes using unit edge weights.
    pub fn shortest_path(&self, from: NodeKey, to: NodeKey) -> Option<Vec<NodeKey>> {
        if self.get_node(from).is_none() || self.get_node(to).is_none() {
            return None;
        }
        astar(
            &UndirectedAdaptor(self.inner.inner()),
            from,
            |node| node == to,
            |_| 1_usize,
            |_| 0_usize,
        )
        .map(|(_, path)| path)
    }

    /// Reachability in the undirected graph.
    pub fn is_reachable(&self, from: NodeKey, to: NodeKey) -> bool {
        if self.get_node(from).is_none() || self.get_node(to).is_none() {
            return false;
        }
        has_path_connecting(&UndirectedAdaptor(self.inner.inner()), from, to, None)
    }

    /// Weakly connected components (undirected projection).
    pub fn weakly_connected_components(&self) -> Vec<Vec<NodeKey>> {
        let mut visited = HashSet::new();
        let mut components = Vec::new();
        for start in self.inner.inner().node_indices() {
            if !visited.insert(start) {
                continue;
            }
            let mut component = Vec::new();
            let mut stack = vec![start];
            while let Some(current) = stack.pop() {
                component.push(current);
                for neighbor in self.neighbors_undirected(current) {
                    if visited.insert(neighbor) {
                        stack.push(neighbor);
                    }
                }
            }
            components.push(component);
        }
        components
    }

    /// The connected component of `seed` — it plus every node reachable through relations
    /// (undirected), breadth-first. Empty if `seed` is not in the graph. `selectors` is the
    /// **edge projection**: only edges matching a selector are followed (an empty slice
    /// follows every family). So the *same* nodes can be one Component under one projection
    /// and a different one under another. The **Component** subgraph's derivation. (Subgraph
    /// derivation, Phase 3 — selectors.)
    pub fn component_members(
        &self,
        seed: uuid::Uuid,
        selectors: &[super::RelationSelector],
    ) -> Vec<uuid::Uuid> {
        self.bfs_members(seed, None, selectors)
    }

    /// The **Ego** neighborhood of `seed`: itself plus every node within `radius`
    /// undirected hops, breadth-first (`radius` 0 = just the seed), over the `selectors`
    /// edge projection (empty = all families). The Ego subgraph's derivation. (Subgraph
    /// derivation, Phase 3 — selectors.)
    pub fn ego_members(
        &self,
        seed: uuid::Uuid,
        radius: u8,
        selectors: &[super::RelationSelector],
    ) -> Vec<uuid::Uuid> {
        self.bfs_members(seed, Some(radius), selectors)
    }

    /// Breadth-first member uuids from `seed` over undirected neighbors, bounded to
    /// `max_depth` hops (`None` = unbounded = the whole component) and to the `selectors`
    /// edge projection. Shared by [`component_members`](Self::component_members) and
    /// [`ego_members`](Self::ego_members).
    fn bfs_members(
        &self,
        seed: uuid::Uuid,
        max_depth: Option<u8>,
        selectors: &[super::RelationSelector],
    ) -> Vec<uuid::Uuid> {
        let Some((start, _)) = self.get_node_by_id(seed) else {
            return Vec::new();
        };
        let mut seen = HashSet::new();
        let mut order = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        seen.insert(start);
        queue.push_back((start, 0u8));
        while let Some((key, depth)) = queue.pop_front() {
            if let Some(node) = self.get_node(key) {
                order.push(node.id);
            }
            if max_depth.is_some_and(|m| depth >= m) {
                continue;
            }
            for neighbor in self.neighbors_undirected_sorted(key) {
                // Edge projection: only follow an edge matching a selector (empty = all
                // families). This is what makes the same nodes derive a different shape
                // under a different relation projection. (Subgraph derivation — selectors.)
                if !selectors.is_empty() && !self.edge_matches_selectors(key, neighbor, selectors) {
                    continue;
                }
                if seen.insert(neighbor) {
                    queue.push_back((neighbor, depth + 1));
                }
            }
        }
        order
    }

    /// Whether any edge between `a` and `b` (either direction, every
    /// parallel edge) matches any of `selectors` (a relation family or
    /// sub-kind). Used by the selector-projected derivation walk. Scans
    /// the whole pair via [`Self::edges_between_undirected`] so an
    /// antiparallel pair with different families (a->b Semantic, b->a
    /// Traversal) is crossed from either end (Q2 ruling, 2026-09-12).
    fn edge_matches_selectors(
        &self,
        a: NodeKey,
        b: NodeKey,
        selectors: &[super::RelationSelector],
    ) -> bool {
        self.edges_between_undirected(a, b)
            .any(|(_, payload)| selectors.iter().any(|&s| payload.has_relation(s)))
    }

    /// Strongly connected components in the directed graph.
    pub fn strongly_connected_components(&self) -> Vec<Vec<NodeKey>> {
        kosaraju_scc(self.inner.inner())
    }

    /// Check if a directed edge exists from `from` to `to`
    pub fn has_edge_between(&self, from: NodeKey, to: NodeKey) -> bool {
        self.inner.inner().find_edge(from, to).is_some()
    }

    /// Count of nodes in the graph
    pub fn node_count(&self) -> usize {
        self.inner.node_count()
    }

    /// Count of edges in the graph
    pub fn edge_count(&self) -> usize {
        self.inner.edge_count()
    }
}

#[cfg(test)]
mod derivation_tests {
    use super::*;
    use crate::graph::SemanticSubKind;
    use euclid::default::Point2D;

    /// An A–B–C chain plus an isolated D. Returns the graph and `[a, b, c, d]` uuids.
    fn chain_plus_isolate() -> (Graph, [uuid::Uuid; 4]) {
        let mut g = Graph::new();
        let a = g.add_node("https://a".to_string(), Point2D::new(0.0, 0.0));
        let b = g.add_node("https://b".to_string(), Point2D::new(1.0, 0.0));
        let c = g.add_node("https://c".to_string(), Point2D::new(2.0, 0.0));
        let d = g.add_node("https://d".to_string(), Point2D::new(9.0, 9.0));
        let sem = || EdgeAssertion::Semantic {
            sub_kind: SemanticSubKind::Hyperlink,
            label: None,
            decay_progress: None,
        };
        g.assert_relation(a, b, sem());
        g.assert_relation(b, c, sem());
        let ids = [
            g.get_node(a).unwrap().id,
            g.get_node(b).unwrap().id,
            g.get_node(c).unwrap().id,
            g.get_node(d).unwrap().id,
        ];
        (g, ids)
    }

    #[test]
    fn component_members_is_the_whole_connected_component() {
        let (g, [a, _b, _c, d]) = chain_plus_isolate();
        assert_eq!(g.component_members(a, &[]).len(), 3, "A reaches B and C");
        assert_eq!(g.component_members(d, &[]).len(), 1, "D is isolated");
        assert!(
            g.component_members(uuid::Uuid::nil(), &[]).is_empty(),
            "unknown seed: empty"
        );
    }

    #[test]
    fn ego_members_is_radius_bounded() {
        let (g, [a, _b, _c, _d]) = chain_plus_isolate();
        assert_eq!(
            g.ego_members(a, 0, &[]).len(),
            1,
            "radius 0 is just the seed"
        );
        assert_eq!(
            g.ego_members(a, 1, &[]).len(),
            2,
            "radius 1 reaches B, not C"
        );
        assert_eq!(g.ego_members(a, 2, &[]).len(), 3, "radius 2 reaches C");
    }

    #[test]
    fn the_selector_projection_changes_the_derived_shape() {
        use crate::graph::{ContainmentSubKind, EdgeFamily, RelationSelector};
        // A —Semantic→ B —Containment→ C. The same three nodes, two relation families.
        let mut g = Graph::new();
        let a = g.add_node("https://a".to_string(), Point2D::new(0.0, 0.0));
        let b = g.add_node("https://b".to_string(), Point2D::new(1.0, 0.0));
        let c = g.add_node("https://c".to_string(), Point2D::new(2.0, 0.0));
        g.assert_relation(
            a,
            b,
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Hyperlink,
                label: None,
                decay_progress: None,
            },
        );
        g.assert_relation(
            b,
            c,
            EdgeAssertion::Containment {
                sub_kind: ContainmentSubKind::CollectionMember,
            },
        );
        let a_id = g.get_node(a).unwrap().id;

        let semantic = [RelationSelector::Family(EdgeFamily::Semantic)];
        let containment = [RelationSelector::Family(EdgeFamily::Containment)];
        // Whole graph under no projection; Semantic stops at B; Containment never leaves A.
        assert_eq!(
            g.component_members(a_id, &[]).len(),
            3,
            "all families: A, B, C"
        );
        assert_eq!(
            g.component_members(a_id, &semantic).len(),
            2,
            "Semantic projection: A, B (B→C is Containment, not followed)"
        );
        assert_eq!(
            g.component_members(a_id, &containment).len(),
            1,
            "Containment projection: just A (no Containment edge from A)"
        );
    }

    /// Q2 (ruled 2026-09-12): an antiparallel pair carrying different
    /// families on each arc (n1->n2 Semantic hyperlink, n2->n1 Traversal,
    /// the shape `tests/snapshot_basic.rs::test_snapshot_preserves_edge_types`
    /// pins as two separate edges) is crossed by a selector-projected walk
    /// from either end, because `edge_matches_selectors` scans the whole
    /// pair rather than the first arc `find_edge_key` happens to return.
    #[test]
    fn antiparallel_pair_is_crossed_from_either_end() {
        use crate::graph::{EdgeFamily, NavigationTrigger, RelationSelector};
        let mut g = Graph::new();
        let n1 = g.add_node("https://a".to_string(), Point2D::new(0.0, 0.0));
        let n2 = g.add_node("https://b".to_string(), Point2D::new(1.0, 0.0));
        g.assert_relation(
            n1,
            n2,
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Hyperlink,
                label: None,
                decay_progress: None,
            },
        );
        assert!(g.append_traversal(n2, n1, NavigationTrigger::LinkClick, Some(1_000_000)));
        assert_eq!(g.edge_count(), 2, "two arcs, one per direction");
        let pair: Vec<_> = g.edges_between_undirected(n1, n2).collect();
        assert_eq!(pair.len(), 2, "the pair primitive sees both arcs");
        assert_eq!(
            g.edges_between_undirected(n2, n1).count(),
            2,
            "and from the other end"
        );
        let id1 = g.get_node(n1).unwrap().id;
        let id2 = g.get_node(n2).unwrap().id;

        for family in [EdgeFamily::Traversal, EdgeFamily::Semantic] {
            let sel = [RelationSelector::Family(family)];
            for (seed, other) in [(id1, id2), (id2, id1)] {
                let members = g.component_members(seed, &sel);
                assert!(
                    members.contains(&seed) && members.contains(&other),
                    "{family:?} projection seeded at {seed} must cross the pair; got {members:?}"
                );
            }
        }
        // A family on neither arc still excludes.
        let sel = [RelationSelector::Family(EdgeFamily::Containment)];
        assert_eq!(g.component_members(id1, &sel), vec![id1]);
        assert_eq!(g.component_members(id2, &sel), vec![id2]);
    }
}

/// Q1 (ruled 2026-09-12): `has_family` / `families()` and the row set
/// from `relation_rows` agree for every family on every edge shape,
/// including the two that used to diverge (open-predicate-only Semantic,
/// event-free Traversal sidecar).
#[cfg(test)]
mod row_family_parity_tests {
    use super::*;
    use crate::graph::{
        ArrangementSubKind, EdgeFamily, ImportedSubKind, NavigationTrigger, ProvenanceSubKind,
        RelationKind, SemanticSubKind,
    };
    use euclid::default::Point2D;
    use std::collections::BTreeSet;

    fn row_families(
        g: &Graph,
    ) -> Vec<(NodeKey, NodeKey, BTreeSet<EdgeFamily>, BTreeSet<EdgeFamily>)> {
        g.inner
            .inner()
            .edge_references()
            .map(|e| {
                let rows = relation_rows(e.source(), e.target(), e.weight());
                let from_rows: BTreeSet<EdgeFamily> =
                    rows.iter().map(|r| r.kind.family()).collect();
                (e.source(), e.target(), e.weight().families(), from_rows)
            })
            .collect()
    }

    #[test]
    fn families_equal_row_families_for_every_edge_shape() {
        let mut g = Graph::new();
        let mut node = |g: &mut Graph, i: usize| {
            g.add_node(format!("https://n{i}"), Point2D::new(i as f32, 0.0))
        };
        let hub = node(&mut g, 0);
        let open = node(&mut g, 1);
        let hyper = node(&mut g, 2);
        let trav = node(&mut g, 3);
        let cont = node(&mut g, 4);
        let arr = node(&mut g, 5);
        let imp = node(&mut g, 6);
        let prov = node(&mut g, 7);
        let mixed = node(&mut g, 8);
        let sem = || EdgeAssertion::Semantic {
            sub_kind: SemanticSubKind::Hyperlink,
            label: None,
            decay_progress: None,
        };

        // Open predicate only: Semantic to `has_family`, formerly no rows.
        let open_key = g
            .assert_semantic_predicate(hub, open, "https://schema.org/citation".to_string())
            .expect("open-predicate edge");
        g.assert_relation(hub, hyper, sem());
        assert!(g.append_traversal(hub, trav, NavigationTrigger::LinkClick, Some(1)));
        g.assert_relation(
            hub,
            cont,
            EdgeAssertion::Containment {
                sub_kind: ContainmentSubKind::CollectionMember,
            },
        );
        g.assert_relation(
            hub,
            arr,
            EdgeAssertion::Arrangement {
                sub_kind: ArrangementSubKind::FrameMember,
            },
        );
        g.assert_relation(
            hub,
            imp,
            EdgeAssertion::Imported {
                sub_kind: ImportedSubKind::BookmarkFolder,
            },
        );
        g.assert_relation(
            hub,
            prov,
            EdgeAssertion::Provenance {
                sub_kind: ProvenanceSubKind::ClippedFrom,
            },
        );
        // Mixed: recognized statement + open statement + traversal on one edge.
        g.assert_relation(hub, mixed, sem());
        g.assert_semantic_predicate(hub, mixed, "https://example.org/related".to_string());
        assert!(g.append_traversal(hub, mixed, NavigationTrigger::Back, Some(2)));

        // The open-predicate edge yields exactly one OpenPredicate row.
        let open_rows = relation_rows(hub, open, g.get_edge(open_key).unwrap());
        assert_eq!(
            open_rows.iter().map(|r| r.kind).collect::<Vec<_>>(),
            vec![RelationKind::OpenPredicate]
        );

        // Live graph: every edge agrees.
        for (from, to, families, from_rows) in row_families(&g) {
            assert_eq!(families, from_rows, "live edge {from:?}->{to:?}");
        }

        // Event-free Traversal sidecar: unreachable from the writers
        // (`push_traversal` always records metrics), so reach it through
        // a snapshot whose traversal record has no events and zero
        // metrics. `has_family(Traversal)` is sidecar presence, and the
        // row must follow it.
        let mut snapshot = g.to_snapshot();
        let mut stripped = 0;
        for edge in &mut snapshot.edges {
            if let Some(t) = edge.traversal.as_mut() {
                t.traversals.clear();
                t.metrics = Default::default();
                stripped += 1;
            }
        }
        assert_eq!(stripped, 2, "the traversal-only edge and the mixed edge");
        let restored = Graph::from_snapshot(&snapshot);
        // Note: the restored graph is not asserted edge-for-edge equal to
        // the live one. `snapshot/from.rs` only restores `UrlPath` /
        // `Domain` containment sub-kinds, so the CollectionMember edge
        // does not come back; that gap predates this test and is not
        // what it pins. Parity is checked on every edge that survives.
        let mut saw_event_free_traversal = 0;
        let mut saw_open_predicate = 0;
        for (from, to, families, from_rows) in row_families(&restored) {
            assert_eq!(families, from_rows, "restored edge {from:?}->{to:?}");
            let payload = restored
                .find_edge_key(from, to)
                .and_then(|k| restored.get_edge(k))
                .unwrap();
            if families.contains(&EdgeFamily::Traversal) {
                assert!(payload.traversals().is_empty());
                assert_eq!(payload.metrics().total_navigations, 0);
                saw_event_free_traversal += 1;
            }
            saw_open_predicate += relation_rows(from, to, payload)
                .iter()
                .filter(|r| r.kind == RelationKind::OpenPredicate)
                .count();
        }
        assert_eq!(
            saw_event_free_traversal, 2,
            "traversal-only and mixed edges"
        );
        assert_eq!(saw_open_predicate, 2, "open-predicate-only and mixed edges");
    }

    #[test]
    fn open_predicate_tag_round_trips_and_keeps_the_semantic_family_byte() {
        let tag = RelationKind::OpenPredicate.tag();
        assert_eq!(
            tag >> 24,
            RelationKind::Semantic(SemanticSubKind::Hyperlink).tag() >> 24
        );
        assert_eq!(
            RelationKind::from_tag(tag),
            Some(RelationKind::OpenPredicate)
        );
        assert_eq!(RelationKind::OpenPredicate.family(), EdgeFamily::Semantic);
    }
}
