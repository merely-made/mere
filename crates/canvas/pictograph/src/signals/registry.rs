// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The channel registry: every graph fact an arrangement or a law reads,
//! computed once per key and held for every reader (dynamics grammar plan,
//! G2b; F38, "Move them all now"; F53, "Canvas-free, in pictograph").
//!
//! It owns no graph: each read names the graph and the registry checks the
//! fact's key against it, recomputing only when the key moved. Keys are the
//! kernel's revisions: structure for the topology facts, the URL grouping for
//! sites, structure and visits for recency (F54), plus a fact's own
//! parameters (a focus, a metric, an iteration count). One registry serves one
//! graph; a host that swaps its graph resets it.
//!
//! Each producer counts its runs ([`RegistryRuns`]), so a reader can assert
//! one computation per key.

use std::collections::HashMap;

use kernel::graph::{Graph, NodeKey};

use super::producers::{self, Recency};
use super::{
    AffinityScores, BridgeMetric, BridgeNodes, ClusterSet, ImportanceMetric, bridges,
    community_louvain, importance, structural_affinity,
};

/// How many times each producer ran.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RegistryRuns {
    pub community: u64,
    pub bridges: u64,
    pub affinity: u64,
    pub importance: u64,
    pub recency: u64,
    pub enumeration: u64,
    pub spectral: u64,
    pub rings: u64,
    pub degree_weights: u64,
    pub sites: u64,
}

/// One cached fact and the key it was computed for.
#[derive(Clone, Debug)]
struct Slot<K, V> {
    key: Option<K>,
    value: Option<V>,
}

impl<K, V> Default for Slot<K, V> {
    fn default() -> Self {
        Self {
            key: None,
            value: None,
        }
    }
}

impl<K: PartialEq + Copy, V> Slot<K, V> {
    fn fresh(&self, key: K) -> bool {
        self.key == Some(key) && self.value.is_some()
    }

    /// The value for `key`, computing it (and counting the run) when stale.
    fn get(&mut self, key: K, runs: &mut u64, compute: impl FnOnce() -> V) -> &V {
        if !self.fresh(key) {
            self.value = Some(compute());
            self.key = Some(key);
            *runs += 1;
        }
        self.value.as_ref().expect("filled above")
    }

    fn fresh_value(&self, key: K) -> Option<&V> {
        self.fresh(key).then(|| self.value.as_ref()).flatten()
    }
}

/// See the module doc.
#[derive(Clone, Debug, Default)]
pub struct ChannelRegistry {
    community: Slot<u64, ClusterSet>,
    bridges_betweenness: Slot<(u64, u32), BridgeNodes>,
    bridges_articulation: Slot<(u64, u32), BridgeNodes>,
    affinity: Slot<(u64, u32), AffinityScores>,
    importance_degree: Slot<u64, HashMap<NodeKey, f32>>,
    importance_betweenness: Slot<u64, HashMap<NodeKey, f32>>,
    recency: Slot<(u64, u64), Recency>,
    enumeration: Slot<u64, Vec<NodeKey>>,
    spectral: Slot<(u64, usize), HashMap<NodeKey, (f32, f32)>>,
    rings: Slot<(u64, NodeKey), HashMap<NodeKey, u32>>,
    degree_weights: Slot<u64, HashMap<NodeKey, f32>>,
    sites: Slot<(u64, u64), HashMap<NodeKey, String>>,
    runs: RegistryRuns,
}

impl ChannelRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Every producer's run count.
    pub fn runs(&self) -> RegistryRuns {
        self.runs
    }

    /// The Louvain partition (`groups.cluster`), keyed by structure.
    pub fn community(&mut self, graph: &Graph) -> &ClusterSet {
        self.community
            .get(graph.revision(), &mut self.runs.community, || {
                community_louvain(graph)
            })
    }

    /// The partition held, fresh or not (the last good one while a newer is
    /// computed elsewhere).
    pub fn community_held(&self) -> Option<&ClusterSet> {
        self.community.value.as_ref()
    }

    /// Whether the partition held is for `graph`'s structure as it stands.
    pub fn community_is_fresh(&self, graph: &Graph) -> bool {
        self.community.fresh(graph.revision())
    }

    /// A partition computed off-thread for structural revision `revision`:
    /// taken, and counted, unless one for it is already held.
    pub fn offer_community(&mut self, revision: u64, clusters: ClusterSet) -> bool {
        if self.community.fresh(revision) {
            return false;
        }
        self.community.value = Some(clusters);
        self.community.key = Some(revision);
        self.runs.community += 1;
        true
    }

    /// The bridge set under `metric` (`bridges.betweenness` or
    /// `bridges.articulation`), one cache per metric, keyed by structure and
    /// the threshold.
    pub fn bridges(&mut self, graph: &Graph, metric: BridgeMetric, threshold: f32) -> &BridgeNodes {
        let slot = match metric {
            BridgeMetric::Betweenness => &mut self.bridges_betweenness,
            BridgeMetric::Articulation => &mut self.bridges_articulation,
        };
        slot.get(
            (graph.revision(), threshold.to_bits()),
            &mut self.runs.bridges,
            || bridges(graph, metric, threshold),
        )
    }

    /// The bridge set held under `metric`, fresh or not.
    pub fn bridges_held(&self, metric: BridgeMetric) -> Option<&BridgeNodes> {
        match metric {
            BridgeMetric::Betweenness => self.bridges_betweenness.value.as_ref(),
            BridgeMetric::Articulation => self.bridges_articulation.value.as_ref(),
        }
    }

    /// Drop the bridge sets, so the next read recomputes.
    pub fn forget_bridges(&mut self) {
        self.bridges_betweenness = Slot::default();
        self.bridges_articulation = Slot::default();
    }

    /// Structural Jaccard affinity (`pairs.structural`) at or above
    /// `min_similarity`, keyed by structure and the floor.
    pub fn structural_affinity(&mut self, graph: &Graph, min_similarity: f32) -> &AffinityScores {
        self.affinity.get(
            (graph.revision(), min_similarity.to_bits()),
            &mut self.runs.affinity,
            || structural_affinity(graph, min_similarity),
        )
    }

    /// The affinity held for `graph`'s structure at `min_similarity`, if fresh.
    pub fn structural_affinity_fresh(
        &self,
        graph: &Graph,
        min_similarity: f32,
    ) -> Option<&AffinityScores> {
        self.affinity
            .fresh_value((graph.revision(), min_similarity.to_bits()))
    }

    /// Importance by `metric` (`importance.degree`, `importance.betweenness`),
    /// normalized `0..=1`, one cache per metric, keyed by structure.
    pub fn importance(
        &mut self,
        graph: &Graph,
        metric: ImportanceMetric,
    ) -> &HashMap<NodeKey, f32> {
        let slot = match metric {
            ImportanceMetric::Degree => &mut self.importance_degree,
            ImportanceMetric::Betweenness => &mut self.importance_betweenness,
        };
        slot.get(graph.revision(), &mut self.runs.importance, || {
            importance(graph, metric).weights.into_iter().collect()
        })
    }

    /// The importance held under `metric`, fresh or not.
    pub fn importance_held(&self, metric: ImportanceMetric) -> Option<&HashMap<NodeKey, f32>> {
        match metric {
            ImportanceMetric::Degree => self.importance_degree.value.as_ref(),
            ImportanceMetric::Betweenness => self.importance_betweenness.value.as_ref(),
        }
    }

    /// The recency order and values (`order.recency`), keyed by structure and
    /// visits (F54).
    pub fn recency(&mut self, graph: &Graph) -> &Recency {
        self.recency.get(
            (graph.revision(), graph.visit_revision()),
            &mut self.runs.recency,
            || producers::recency(graph),
        )
    }

    /// The recency held, fresh or not.
    pub fn recency_held(&self) -> Option<&Recency> {
        self.recency.value.as_ref()
    }

    /// The enumeration order (`order.timeline`): the order every score's
    /// ordinal follows and a timeline lays its axis along, keyed by structure.
    pub fn enumeration(&mut self, graph: &Graph) -> &[NodeKey] {
        self.enumeration
            .get(graph.revision(), &mut self.runs.enumeration, || {
                producers::enumeration_order(graph)
            })
    }

    /// Spectral coordinates (`coords.spectral`) after `iterations` power
    /// iterations, keyed by structure and the count.
    pub fn spectral(&mut self, graph: &Graph, iterations: usize) -> &HashMap<NodeKey, (f32, f32)> {
        self.spectral.get(
            (graph.revision(), iterations),
            &mut self.runs.spectral,
            || producers::spectral_coords(graph, iterations),
        )
    }

    /// Ring indices from `focus` (`rings.focus`), keyed by structure and the
    /// focus.
    pub fn rings(&mut self, graph: &Graph, focus: NodeKey) -> &HashMap<NodeKey, u32> {
        self.rings
            .get((graph.revision(), focus), &mut self.runs.rings, || {
                producers::radial_rings(graph, focus)
            })
    }

    /// Degree plus one (`weight.degree`), keyed by structure.
    pub fn degree_weights(&mut self, graph: &Graph) -> &HashMap<NodeKey, f32> {
        self.degree_weights
            .get(graph.revision(), &mut self.runs.degree_weights, || {
                producers::degree_weights(graph)
            })
    }

    /// Each node's site (`groups.site`: its URL authority), keyed by structure
    /// and the URL grouping.
    pub fn sites(&mut self, graph: &Graph) -> &HashMap<NodeKey, String> {
        self.sites.get(
            (graph.revision(), graph.url_grouping_revision()),
            &mut self.runs.sites,
            || {
                graph
                    .nodes()
                    .map(|(key, node)| (key, Graph::url_grouping_key(node.url()).to_string()))
                    .collect()
            },
        )
    }

    /// The sites held for `graph` as it stands, if fresh.
    pub fn sites_fresh(&self, graph: &Graph) -> Option<&HashMap<NodeKey, String>> {
        self.sites
            .fresh_value((graph.revision(), graph.url_grouping_revision()))
    }
}
