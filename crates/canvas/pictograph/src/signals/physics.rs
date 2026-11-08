// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The physics view's channels (dynamics grammar plan, G2c; F21, F149):
//! `mass.degree`, `mass.pagerank`, the colouring, island and degree-band
//! groups, the three depths, `distances.hops` and `edges.spanning`.
//!
//! Each producer reads a node list (in key order) and an edge list, one
//! tuple per visible relation cell: the physics view, the graph's relations
//! less the canvas's hidden ones. The registry caches them for the view's
//! readers, keyed by [`PhysicsViewKey`] (F178) and the fact's own parameter;
//! a grouping's subgraphs, the board's items and the catalog's nominal call
//! the producers directly (F179). The algorithms run over a petgraph view of
//! the edges: the directed multigraph for PageRank, layering, dominators and
//! components; the undirected simple graph (one edge per pair, cost
//! `1 / multiplicity`) for the colouring, the spanning tree and the shortest
//! paths. Gated on `canvas`, which carries petgraph, so a signals-only host
//! sees no new dependency.

use kernel::graph::NodeKey;

/// The algorithms moved to `seiche::catalog::algo` (engine step 2b, milestone 2): they
/// are kernel-free, and a grouped law builds per-subgroup forces from the
/// subgroup's own inputs on the engine's side. Re-exported under their old paths.
pub use seiche::catalog::algo::{
    component_groups, coloring_groups, degree_bands, focus_depths, hop_distances, layer_depths,
    mass_degree, page_rank_weights, root_depths, spanning_edges,
};

/// The physics view's key (F178): the graph's source-qualified structural input and the
/// canvas's view revision, which a hide or show that changes the view bumps.
/// The caller passes it in, so the registry holds no canvas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PhysicsViewKey {
    pub structure: kernel::graph::GraphInputKey,
    pub view: u64,
}

/// The physics view a read names: its key, its nodes in key order and its
/// edges.
#[derive(Clone, Copy, Debug)]
pub struct PhysicsView<'a> {
    pub key: PhysicsViewKey,
    pub nodes: &'a [NodeKey],
    pub edges: &'a [(NodeKey, NodeKey)],
}

/// How many times each physics producer ran in a registry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhysicsRuns {
    pub mass_degree: u64,
    pub pagerank: u64,
    pub coloring: u64,
    pub components: u64,
    pub degree_bands: u64,
    pub depth_roots: u64,
    pub depth_layers: u64,
    pub depth_focus: u64,
    pub distances: u64,
    pub spanning: u64,
}
