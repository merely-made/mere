// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Graph {
    /// Get a mutable edge payload by key, noted as written for the handle index.
    pub(crate) fn get_edge_mut(&mut self, key: EdgeKey) -> Option<&mut EdgePayload> {
        self.note_relation_handles(RelationKey::Surface(key));
        self.inner.edge_mut(key)
    }

    /// Get an edge payload by key.
    pub fn get_edge(&self, key: EdgeKey) -> Option<&EdgePayload> {
        self.inner.edge(key)
    }

    /// Find the first directed edge key between two nodes.
    pub fn find_edge_key(&self, from: NodeKey, to: NodeKey) -> Option<EdgeKey> {
        self.inner.inner().find_edge(from, to)
    }

    /// Every edge between `a` and `b` in either direction: all `a->b`
    /// edges first, then all `b->a` edges, each run in petgraph's
    /// `edges_connecting` order (deterministic for a given graph). The
    /// substrate is a multigraph, so a pair may carry an antiparallel
    /// or parallel set; read predicates that mean "the pair" rather
    /// than "the first arc" go through here instead of
    /// [`Self::find_edge_key`]. Self-loops (`a == b`) are yielded once.
    pub fn edges_between_undirected(
        &self,
        a: NodeKey,
        b: NodeKey,
    ) -> impl Iterator<Item = (EdgeKey, &EdgePayload)> + '_ {
        use petgraph::visit::EdgeRef;
        let inner = self.inner.inner();
        let forward = inner.edges_connecting(a, b);
        let backward = inner.edges_connecting(b, a).filter(move |_| a != b);
        forward
            .chain(backward)
            .map(|edge| (edge.id(), edge.weight()))
    }
}
