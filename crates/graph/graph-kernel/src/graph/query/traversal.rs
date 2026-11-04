// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Graph {
    /// Iterate outgoing neighbor keys for a node
    pub fn out_neighbors(&self, key: NodeKey) -> impl Iterator<Item = NodeKey> + '_ {
        self.inner
            .inner()
            .edges_directed(key, Direction::Outgoing)
            .filter(|edge| edge.weight().has_held_topology())
            .map(|edge| edge.target())
    }

    /// Iterate incoming neighbor keys for a node
    pub fn in_neighbors(&self, key: NodeKey) -> impl Iterator<Item = NodeKey> + '_ {
        self.inner
            .inner()
            .edges_directed(key, Direction::Incoming)
            .filter(|edge| edge.weight().has_held_topology())
            .map(|edge| edge.source())
    }

    /// Iterate undirected neighbor keys for a node, Surface stratum only.
    /// [`projected_neighbors`](Self::projected_neighbors) reads both strata.
    pub fn neighbors_undirected(&self, key: NodeKey) -> impl Iterator<Item = NodeKey> + '_ {
        let inner = self.inner.inner();
        let outgoing = inner
            .edges_directed(key, Direction::Outgoing)
            .filter(|edge| edge.weight().has_held_topology())
            .map(|edge| edge.target());
        let incoming = inner
            .edges_directed(key, Direction::Incoming)
            .filter(|edge| edge.weight().has_held_topology() && edge.source() != edge.target())
            .map(|edge| edge.source());
        outgoing.chain(incoming)
    }

    /// Every node adjacent to `key` in either direction across both strata: the
    /// Surface neighbours, plus the surfaces showing the far end of each Resource
    /// relation of the resource `key` shows. One entry per incident held relation,
    /// so parallel relations repeat; a self-loop is read once, from its outgoing
    /// side, so a node adjacent to itself appears once. Hides are a view concern
    /// and do not apply here.
    pub fn projected_neighbors(&self, key: NodeKey) -> impl Iterator<Item = NodeKey> + '_ {
        let incoming = self
            .projected_incoming_relations(key)
            .filter(|(_, relation, _)| !self.is_self_loop(*relation));
        self.projected_outgoing_relations(key)
            .chain(incoming)
            .map(|(neighbor, _, _)| neighbor)
    }

    /// Whether `relation` joins one node, or one resource, to itself.
    pub(super) fn is_self_loop(&self, relation: RelationKey) -> bool {
        match relation {
            RelationKey::Surface(edge) => self
                .inner
                .inner()
                .edge_endpoints(edge)
                .is_some_and(|(from, to)| from == to),
            RelationKey::Resource(edge) => self
                .resources
                .inner()
                .edge_endpoints(edge.raw())
                .is_some_and(|(from, to)| from == to),
        }
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
        let mut distances = HashMap::from([(source, 0)]);
        let mut queue = VecDeque::from([source]);
        while let Some(node) = queue.pop_front() {
            let next_distance = distances[&node] + 1;
            for neighbor in self.neighbors_undirected(node) {
                if let std::collections::hash_map::Entry::Vacant(entry) = distances.entry(neighbor)
                {
                    entry.insert(next_distance);
                    queue.push_back(neighbor);
                }
            }
        }
        distances
    }

    /// Nodes with no incoming or outgoing edges.
    pub fn orphan_node_keys(&self) -> Vec<NodeKey> {
        self.inner
            .inner()
            .node_indices()
            .filter(|&key| self.neighbors_undirected(key).next().is_none())
            .collect()
    }

    /// Shortest undirected path between two nodes using unit edge weights.
    pub fn shortest_path(&self, from: NodeKey, to: NodeKey) -> Option<Vec<NodeKey>> {
        if self.get_node(from).is_none() || self.get_node(to).is_none() {
            return None;
        }
        let mut previous = HashMap::from([(from, from)]);
        let mut queue = VecDeque::from([from]);
        while let Some(node) = queue.pop_front() {
            if node == to {
                break;
            }
            for neighbor in self.neighbors_undirected(node) {
                if let std::collections::hash_map::Entry::Vacant(entry) = previous.entry(neighbor) {
                    entry.insert(node);
                    queue.push_back(neighbor);
                }
            }
        }
        if !previous.contains_key(&to) {
            return None;
        }
        let mut path = vec![to];
        while *path.last().unwrap() != from {
            path.push(previous[path.last().unwrap()]);
        }
        path.reverse();
        Some(path)
    }

    /// Reachability in the undirected graph.
    pub fn is_reachable(&self, from: NodeKey, to: NodeKey) -> bool {
        if self.get_node(from).is_none() || self.get_node(to).is_none() {
            return false;
        }
        self.shortest_path(from, to).is_some()
    }

    /// Weakly connected components of the surface stratum (undirected projection),
    /// a topology metric like [`Self::orphan_node_keys`]. Resource relations and
    /// shown bindings do not join here; the semantic Component is
    /// [`Self::component_members`].
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

    /// Surfaces reachable from `seed` through either stratum's undirected relations.
    /// Shown bindings join at zero hops, including through unshown resources.
    /// Empty if `seed` is not a surface in the graph. `selectors` is the
    /// **edge projection**: only edges matching a selector are followed (an empty slice
    /// follows every family). So the *same* nodes can be one Component under one projection
    /// and a different one under another. The **Component** subgraph's derivation. (Subgraph
    /// derivation, Phase 3 — selectors.)
    pub fn component_members(
        &self,
        seed: uuid::Uuid,
        selectors: &[super::super::RelationSelector],
    ) -> Vec<uuid::Uuid> {
        self.bfs_members(seed, None, selectors)
    }

    /// The **Ego** neighborhood of `seed`: surfaces within `radius` undirected
    /// relation hops across both strata. Shown bindings cost zero, so radius 0
    /// also includes surfaces showing the seed's resource. Uses the `selectors`
    /// edge projection (empty = all families). The Ego subgraph's derivation. (Subgraph
    /// derivation, Phase 3 — selectors.)
    pub fn ego_members(
        &self,
        seed: uuid::Uuid,
        radius: u8,
        selectors: &[super::super::RelationSelector],
    ) -> Vec<uuid::Uuid> {
        self.bfs_members(seed, Some(radius), selectors)
    }

    /// Collapse zero-hop shown bindings, then walk relation hops breadth-first.
    pub(super) fn bfs_members(
        &self,
        seed: uuid::Uuid,
        max_depth: Option<u8>,
        selectors: &[super::super::RelationSelector],
    ) -> Vec<uuid::Uuid> {
        let Some((start, _)) = self.get_node_by_id(seed) else {
            return Vec::new();
        };
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        enum WalkNode {
            Surface(NodeKey),
            Resource(Uuid),
        }
        let owner = |surface| {
            self.shown_resource_id(surface)
                .filter(|resource| self.resource(*resource).is_some())
                .map(WalkNode::Resource)
                .unwrap_or(WalkNode::Surface(surface))
        };
        let mut shown = HashMap::<Uuid, Vec<NodeKey>>::new();
        for (key, _) in self.nodes() {
            if let WalkNode::Resource(resource) = owner(key) {
                shown.entry(resource).or_default().push(key);
            }
        }
        for surfaces in shown.values_mut() {
            surfaces.sort_by_key(|key| key.index());
        }
        let start = owner(start);
        let mut seen = HashSet::new();
        let mut order = Vec::new();
        let mut queue = VecDeque::new();
        seen.insert(start);
        queue.push_back((start, 0usize));
        while let Some((current, depth)) = queue.pop_front() {
            let surfaces = match current {
                WalkNode::Surface(key) => vec![key],
                WalkNode::Resource(resource) => shown.get(&resource).cloned().unwrap_or_default(),
            };
            if current == start {
                order.push(seed);
            }
            for key in &surfaces {
                let id = self.get_node(*key).expect("walk surface exists").id;
                if id != seed {
                    order.push(id);
                }
            }
            if max_depth.is_some_and(|m| depth >= usize::from(m)) {
                continue;
            }
            let mut neighbors = Vec::new();
            for key in surfaces {
                for neighbor in self.neighbors_undirected_sorted(key) {
                    if selectors.is_empty() || self.edge_matches_selectors(key, neighbor, selectors)
                    {
                        neighbors.push(owner(neighbor));
                    }
                }
            }
            if let WalkNode::Resource(resource) = current {
                let key = self
                    .resources
                    .key_of(&resource)
                    .expect("walk resource exists");
                for direction in [Direction::Outgoing, Direction::Incoming] {
                    for edge in self.resources.inner().edges_directed(key, direction) {
                        if !edge.weight().has_held_topology() {
                            continue;
                        }
                        if !selectors.is_empty()
                            && !selectors
                                .iter()
                                .any(|&selector| edge.weight().has_relation(selector))
                        {
                            continue;
                        }
                        let neighbor = match direction {
                            Direction::Outgoing => edge.target(),
                            Direction::Incoming => edge.source(),
                        };
                        neighbors.push(WalkNode::Resource(
                            self.resources
                                .node(neighbor)
                                .expect("resource endpoint exists")
                                .id(),
                        ));
                    }
                }
            }
            neighbors.sort_unstable();
            neighbors.dedup();
            for neighbor in neighbors {
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
    pub(super) fn edge_matches_selectors(
        &self,
        a: NodeKey,
        b: NodeKey,
        selectors: &[super::super::RelationSelector],
    ) -> bool {
        self.edges_between_undirected(a, b)
            .any(|(_, payload)| selectors.iter().any(|&s| payload.has_relation(s)))
    }

    /// Strongly connected components in the directed graph.
    pub fn strongly_connected_components(&self) -> Vec<Vec<NodeKey>> {
        let mut active = DiGraph::<(), ()>::new();
        let mut keys = Vec::new();
        let mut mapped = HashMap::new();
        for key in self.inner.inner().node_indices() {
            let mapped_key = active.add_node(());
            keys.push(key);
            mapped.insert(key, mapped_key);
        }
        for edge in self
            .inner
            .inner()
            .edge_references()
            .filter(|edge| edge.weight().has_held_topology())
        {
            active.add_edge(mapped[&edge.source()], mapped[&edge.target()], ());
        }
        kosaraju_scc(&active)
            .into_iter()
            .map(|component| {
                component
                    .into_iter()
                    .map(|node| keys[node.index()])
                    .collect()
            })
            .collect()
    }
}
