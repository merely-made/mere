// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Graph-structural **signal producer** — the graph-signals-layer plan's `intel/signals`.
//!
//! Computes per-node / per-pair signals from the kernel [`Graph`] and hands them to
//! cartography's narrow [`cartography::IntelligenceSignals`] contract. This crate OWNS the production
//! lifecycle; cartography keeps only the contract (so cartography never depends on a producer).
//!
//! The [`ChannelRegistry`] holds every one of them, and the disclosures cartography's
//! arrangements read (spectral coordinates, rings, degree weights, the recency and
//! enumeration orders, sites), computed once per key (dynamics grammar plan, G2b).

use std::collections::{HashMap, VecDeque};

use cartography::ImportanceWeights;
use kernel::graph::{Graph, NodeKey};

pub use cartography::{AffinityScores, BridgeNodes, Cluster, ClusterSet, Overlay};

mod affinity;
mod bridges;
mod community;
mod importance;
mod producers;
mod registry;
#[cfg(test)]
mod registry_tests;

pub use producers::{
    Recency, degree_weights, enumeration_order, radial_rings, recency, spectral_coords,
};
pub use registry::{ChannelRegistry, RegistryRuns};

pub use affinity::*;
pub use bridges::*;
pub use community::*;
pub use importance::*;

/// The minimal graph view the structural signals need: the node set and undirected
/// adjacency. Every signal here (degree, betweenness, community, bridges, affinity) is
/// pure topology, so binding them to this seam instead of a concrete graph makes them
/// run on any graph that can enumerate its nodes and a node's undirected neighbours.
/// mere's `kernel::graph::Graph` and the generic `chartulary::Graph` both implement it,
/// so the same algorithms serve the browser and any substrate graph — the analytics
/// become promotable. (Graph signals — the promotable seam.)
pub trait TopologyView {
    /// Every node's stable key.
    fn node_keys(&self) -> impl Iterator<Item = NodeKey> + '_;

    /// The neighbours of `key` in either direction, one entry per incident edge
    /// (parallel edges repeat; the signals collapse multiplicity where they need
    /// distinct adjacency).
    fn neighbors_undirected(&self, key: NodeKey) -> impl Iterator<Item = NodeKey> + '_;
}

impl TopologyView for Graph {
    fn node_keys(&self) -> impl Iterator<Item = NodeKey> + '_ {
        self.nodes().map(|(key, _)| key)
    }

    fn neighbors_undirected(&self, key: NodeKey) -> impl Iterator<Item = NodeKey> + '_ {
        // A self-loop bucket appears in both directions; count it once.
        self.projected_outgoing_relations(key)
            .chain(
                self.projected_incoming_relations(key)
                    .filter(move |(neighbor, _, _)| *neighbor != key),
            )
            .map(|(neighbor, _, _)| neighbor)
    }
}

impl<N: chartulary::Identified, E> TopologyView for chartulary::Graph<N, E> {
    fn node_keys(&self) -> impl Iterator<Item = NodeKey> + '_ {
        self.nodes().map(|(key, _)| key)
    }

    fn neighbors_undirected(&self, key: NodeKey) -> impl Iterator<Item = NodeKey> + '_ {
        chartulary::Graph::neighbors_undirected(self, key)
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod substrate_tests {
    use super::*;
    use chartulary::{Container, Graph as ChartGraph, Recognized, Relation, RelationClass};

    /// The structural signals run on a generic `chartulary::Graph`, not just mere's
    /// `kernel::graph::Graph` — the promotability payoff of the `TopologyView` seam.
    #[test]
    fn structural_signals_run_on_a_generic_chartulary_graph() {
        // A star: hub `h` cited by three leaves.
        let mut g: ChartGraph<Container, Relation> = ChartGraph::new();
        let h = g.insert(Container::new("h"));
        for leaf in ["a", "b", "c"] {
            let k = g.insert(Container::new(leaf));
            g.connect(
                k,
                h,
                Relation::new(RelationClass::recognized(Recognized::Cites)),
            );
        }

        // Degree importance ranks the hub top (it has all three edges).
        let weights = degree_importance(&g);
        let hub = g.key_of(&"h".to_string()).expect("hub present");
        let hub_weight = weights
            .weights
            .iter()
            .find(|(key, _)| *key == hub)
            .map(|(_, weight)| *weight);
        assert_eq!(hub_weight, Some(1.0), "the hub is the most-connected node");

        // The heavier signals accept the generic graph too (compile + run proof).
        let _clusters = community_louvain(&g);
        let _affinity = structural_affinity(&g, 0.0);
        let _articulation = articulation_points(&g);
    }
}
