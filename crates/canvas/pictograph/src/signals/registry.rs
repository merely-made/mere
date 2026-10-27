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
//! kernel's process-local input versions: structure for topology, URL grouping for
//! sites, structure and visits for recency (F54), plus a fact's own
//! parameters (a focus, a metric, an iteration count). Ordinary clones retain input versions; independent graphs and divergent
//! candidates cannot collide through equal local counters. Binding a different
//! graph source drops held values while retaining producer counters.
//!
//! Each producer counts its runs ([`RegistryRuns`]), so a reader can assert
//! one computation per key.
//!
//! The physics view's channels joined in G2c (F21, F149): `mass.*`, the
//! colouring, island and degree-band groups, `depth.*`, `distances.hops`
//! and `edges.spanning` ([`super::physics`]). They read the physics view,
//! the graph's relations less the canvas's hidden ones, so they key by the
//! [`PhysicsViewKey`] the caller passes in (F178) and the fact's own
//! parameter, and count their runs in [`PhysicsRuns`]. They are read
//! through `&self` while a law build holds the registry's other facts, so
//! they sit in a cell; a read hands out a copy of the cached values.

use std::collections::HashMap;

use cartography::{
    COORDS_SPECTRAL, ImportanceWeights, IntelligenceSignals, NodeEmbeddings, NodeOrder, NodeRings,
    ORDER_RECENCY, ORDER_TIMELINE, RINGS_FOCUS, Signal, WEIGHT_DEGREE, WEIGHT_RECENCY,
};
use kernel::graph::{Graph, GraphInputKey, NodeKey};

#[cfg(feature = "canvas")]
use super::physics::{self, PhysicsRuns, PhysicsView, PhysicsViewKey};
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
        self.value.as_ref().filter(|_| self.fresh(key))
    }
}

/// See the module doc.
#[derive(Clone, Debug, Default)]
pub struct ChannelRegistry {
    source: Option<GraphInputKey>,
    community: Slot<GraphInputKey, ClusterSet>,
    bridges_betweenness: Slot<(GraphInputKey, u32), BridgeNodes>,
    bridges_articulation: Slot<(GraphInputKey, u32), BridgeNodes>,
    affinity: Slot<(GraphInputKey, u32), AffinityScores>,
    importance_degree: Slot<GraphInputKey, HashMap<NodeKey, f32>>,
    importance_betweenness: Slot<GraphInputKey, HashMap<NodeKey, f32>>,
    recency: Slot<(GraphInputKey, GraphInputKey), Recency>,
    enumeration: Slot<GraphInputKey, Vec<NodeKey>>,
    spectral: Slot<(GraphInputKey, usize), HashMap<NodeKey, (f32, f32)>>,
    rings: Slot<(GraphInputKey, NodeKey), HashMap<NodeKey, u32>>,
    degree_weights: Slot<GraphInputKey, HashMap<NodeKey, f32>>,
    sites: Slot<(GraphInputKey, GraphInputKey), HashMap<NodeKey, String>>,
    runs: RegistryRuns,
    #[cfg(feature = "canvas")]
    physics: std::cell::RefCell<PhysicsSlots>,
}

/// The physics view's facts and their run counts (G2c).
#[cfg(feature = "canvas")]
#[derive(Clone, Debug, Default)]
struct PhysicsSlots {
    mass_degree: Slot<PhysicsViewKey, Vec<(NodeKey, f32)>>,
    pagerank: Slot<PhysicsViewKey, Vec<(NodeKey, f32)>>,
    coloring: Slot<PhysicsViewKey, Vec<(NodeKey, u32)>>,
    components: Slot<PhysicsViewKey, Vec<(NodeKey, u32)>>,
    degree_bands: Slot<PhysicsViewKey, Vec<(NodeKey, u32)>>,
    depth_roots: Slot<PhysicsViewKey, Vec<(NodeKey, u32)>>,
    depth_layers: Slot<PhysicsViewKey, Vec<(NodeKey, u32)>>,
    depth_focus: Slot<(PhysicsViewKey, NodeKey), Vec<(NodeKey, u32)>>,
    distances: Slot<PhysicsViewKey, Vec<(NodeKey, NodeKey, f32)>>,
    spanning: Slot<PhysicsViewKey, Vec<(NodeKey, NodeKey)>>,
    runs: PhysicsRuns,
}

/// One physics fact over `view`, from its slot or computed and counted.
#[cfg(feature = "canvas")]
macro_rules! physics_fact {
    ($(#[$meta:meta])* $name:ident, $slot:ident, $ty:ty, $producer:path) => {
        $(#[$meta])*
        pub fn $name(&self, view: PhysicsView<'_>) -> Vec<$ty> {
            let mut slots = self.physics.borrow_mut();
            let PhysicsSlots { $slot, runs, .. } = &mut *slots;
            $slot
                .get(view.key, &mut runs.$slot, || $producer(view.nodes, view.edges))
                .clone()
        }
    };
}

#[cfg(feature = "canvas")]
impl ChannelRegistry {
    /// Every physics producer's run count.
    pub fn physics_runs(&self) -> PhysicsRuns {
        self.physics.borrow().runs
    }

    physics_fact!(
        /// `mass.degree`, a body's mass before its reader's transform: each
        /// node's degree over the physics view's edges (F147).
        mass_degree, mass_degree, (NodeKey, f32), physics::mass_degree
    );
    physics_fact!(
        /// `mass.pagerank`: PageRank over the physics view, mean one.
        mass_pagerank, pagerank, (NodeKey, f32), physics::page_rank_weights
    );
    physics_fact!(
        /// `groups.coloring`: a proper colouring of the physics view.
        groups_coloring, coloring, (NodeKey, u32), physics::coloring_groups
    );
    physics_fact!(
        /// `groups.component`: the physics view's islands.
        groups_component, components, (NodeKey, u32), physics::component_groups
    );
    physics_fact!(
        /// `groups.degree`: degree bands over the physics view.
        groups_degree, degree_bands, (NodeKey, u32), physics::degree_bands
    );
    physics_fact!(
        /// `depth.roots`: depth from the physics view's roots.
        depth_roots, depth_roots, (NodeKey, u32), physics::root_depths
    );
    physics_fact!(
        /// `depth.layers`: the physics view's Sugiyama layers.
        depth_layers, depth_layers, (NodeKey, u32), physics::layer_depths
    );
    physics_fact!(
        /// `distances.hops`: shortest paths over the physics view.
        distances_hops, distances, (NodeKey, NodeKey, f32), physics::hop_distances
    );
    physics_fact!(
        /// `edges.spanning`: the physics view's spanning tree, Stress's
        /// skeleton (F174).
        edges_spanning, spanning, (NodeKey, NodeKey), physics::spanning_edges
    );

    /// `depth.focus`: dominator depth from `focus`, keyed by the view and the
    /// focus; a focus outside the view reads `depth.roots`.
    pub fn depth_focus(&self, view: PhysicsView<'_>, focus: NodeKey) -> Vec<(NodeKey, u32)> {
        if !view.nodes.contains(&focus) {
            return self.depth_roots(view);
        }
        let mut slots = self.physics.borrow_mut();
        let PhysicsSlots {
            depth_focus, runs, ..
        } = &mut *slots;
        depth_focus
            .get((view.key, focus), &mut runs.depth_focus, || {
                physics::focus_depths(view.nodes, view.edges, focus)
            })
            .clone()
    }
}

impl ChannelRegistry {
    /// Bind held values to one materialized graph source. Preserve producer
    /// counters, but never carry last-good values into a different source.
    pub fn bind_graph(&mut self, graph: &Graph) -> bool {
        let key = graph.structural_input_key();
        let changed = self.source.is_some_and(|source| !source.same_source(key));
        if changed {
            let runs = self.runs;
            #[cfg(feature = "canvas")]
            let physics_runs = self.physics.get_mut().runs;
            *self = Self::default();
            self.runs = runs;
            #[cfg(feature = "canvas")]
            {
                self.physics.get_mut().runs = physics_runs;
            }
        }
        self.source = Some(key);
        changed
    }

    pub fn new() -> Self {
        Self::default()
    }

    /// Every producer's run count.
    pub fn runs(&self) -> RegistryRuns {
        self.runs
    }

    /// The Louvain partition (`groups.cluster`), keyed by structure.
    pub fn community(&mut self, graph: &Graph) -> &ClusterSet {
        self.bind_graph(graph);
        self.community.get(
            graph.structural_input_key(),
            &mut self.runs.community,
            || community_louvain(graph),
        )
    }

    /// The partition held, fresh or not (the last good one while a newer is
    /// computed elsewhere).
    pub fn community_held(&self) -> Option<&ClusterSet> {
        self.community.value.as_ref()
    }

    /// Whether the partition held is for `graph`'s structure as it stands.
    pub fn community_is_fresh(&self, graph: &Graph) -> bool {
        self.community.fresh(graph.structural_input_key())
    }

    /// A partition computed off-thread for structural revision `revision`:
    /// taken, and counted, unless one for it is already held.
    pub fn offer_community(
        &mut self,
        graph: &Graph,
        revision: GraphInputKey,
        clusters: ClusterSet,
    ) -> bool {
        self.bind_graph(graph);
        if revision != graph.structural_input_key() {
            return false;
        }
        if self.community.fresh(revision) {
            return false;
        }
        self.community.value = Some(clusters);
        self.community.key = Some(revision);
        self.runs.community += 1;
        true
    }

    /// The bridge set under `metric` (`groups.bridges`, the metric its
    /// option, F85), one cache per metric, keyed by structure and the
    /// threshold.
    pub fn bridges(&mut self, graph: &Graph, metric: BridgeMetric, threshold: f32) -> &BridgeNodes {
        self.bind_graph(graph);
        let slot = match metric {
            BridgeMetric::Betweenness => &mut self.bridges_betweenness,
            BridgeMetric::Articulation => &mut self.bridges_articulation,
        };
        slot.get(
            (graph.structural_input_key(), threshold.to_bits()),
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
        self.bind_graph(graph);
        self.affinity.get(
            (graph.structural_input_key(), min_similarity.to_bits()),
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
            .fresh_value((graph.structural_input_key(), min_similarity.to_bits()))
    }

    /// Importance by `metric` (`importance.degree`, `importance.betweenness`),
    /// normalized `0..=1`, one cache per metric, keyed by structure.
    pub fn importance(
        &mut self,
        graph: &Graph,
        metric: ImportanceMetric,
    ) -> &HashMap<NodeKey, f32> {
        self.bind_graph(graph);
        let slot = match metric {
            ImportanceMetric::Degree => &mut self.importance_degree,
            ImportanceMetric::Betweenness => &mut self.importance_betweenness,
        };
        slot.get(
            graph.structural_input_key(),
            &mut self.runs.importance,
            || importance(graph, metric).weights.into_iter().collect(),
        )
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
        self.bind_graph(graph);
        self.recency.get(
            (graph.structural_input_key(), graph.visit_input_key()),
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
        self.bind_graph(graph);
        self.enumeration.get(
            graph.structural_input_key(),
            &mut self.runs.enumeration,
            || producers::enumeration_order(graph),
        )
    }

    /// Spectral coordinates (`coords.spectral`) after `iterations` power
    /// iterations, keyed by structure and the count.
    pub fn spectral(&mut self, graph: &Graph, iterations: usize) -> &HashMap<NodeKey, (f32, f32)> {
        self.bind_graph(graph);
        self.spectral.get(
            (graph.structural_input_key(), iterations),
            &mut self.runs.spectral,
            || producers::spectral_coords(graph, iterations),
        )
    }

    /// Ring indices from `focus` (`rings.focus`), keyed by structure and the
    /// focus.
    pub fn rings(&mut self, graph: &Graph, focus: NodeKey) -> &HashMap<NodeKey, u32> {
        self.bind_graph(graph);
        self.rings.get(
            (graph.structural_input_key(), focus),
            &mut self.runs.rings,
            || producers::radial_rings(graph, focus),
        )
    }

    /// Degree plus one (`weight.degree`), keyed by structure: an
    /// arrangement's weight, Radial's rings, over every kernel edge. A body's
    /// mass is `mass.degree`, another computation over the physics view's
    /// edges (F147; `canvas::channels`).
    pub fn degree_weights(&mut self, graph: &Graph) -> &HashMap<NodeKey, f32> {
        self.bind_graph(graph);
        self.degree_weights.get(
            graph.structural_input_key(),
            &mut self.runs.degree_weights,
            || producers::degree_weights(graph),
        )
    }

    /// Each node's site (`groups.site`: its URL authority), keyed by structure
    /// and the URL grouping.
    pub fn sites(&mut self, graph: &Graph) -> &HashMap<NodeKey, String> {
        self.bind_graph(graph);
        self.sites.get(
            (graph.structural_input_key(), graph.url_grouping_input_key()),
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
            .fresh_value((graph.structural_input_key(), graph.url_grouping_input_key()))
    }

    /// The keyed signals a projection request carries for `channels`
    /// (cartography's ids, `adapters::channels_read`), each read from this
    /// registry once per key (dynamics grammar plan, F84 and F86). Rings need
    /// `focus`; an id the registry does not compute, or rings without a
    /// focus, is left out, and the adapter reading it reports it missing.
    pub fn disclose(
        &mut self,
        graph: &Graph,
        channels: &[&str],
        focus: Option<NodeKey>,
    ) -> IntelligenceSignals {
        self.bind_graph(graph);
        let mut signals = IntelligenceSignals::new();
        for &id in channels {
            let signal = match id {
                ORDER_TIMELINE => Signal::Order(NodeOrder {
                    order: self.enumeration(graph).to_vec(),
                }),
                ORDER_RECENCY => Signal::Order(NodeOrder {
                    order: self.recency(graph).order.clone(),
                }),
                WEIGHT_RECENCY => Signal::Weights(ImportanceWeights {
                    weights: in_key_order(&self.recency(graph).values),
                }),
                COORDS_SPECTRAL => {
                    let iterations = cartography::adapters::SpectralAdapter::default().iterations;
                    Signal::Coords(NodeEmbeddings {
                        coords: in_key_order(self.spectral(graph, iterations)),
                    })
                },
                WEIGHT_DEGREE => Signal::Weights(ImportanceWeights {
                    weights: in_key_order(self.degree_weights(graph)),
                }),
                RINGS_FOCUS => match focus {
                    Some(focus) => Signal::Rings(NodeRings {
                        rings: in_key_order(self.rings(graph, focus)),
                    }),
                    None => continue,
                },
                _ => continue,
            };
            signals.insert(id, signal);
        }
        signals
    }
}

/// A fact's values in node-key order, so a request reads the same whatever
/// the hash order.
fn in_key_order<V: Copy>(values: &HashMap<NodeKey, V>) -> Vec<(NodeKey, V)> {
    let mut values: Vec<(NodeKey, V)> = values.iter().map(|(key, value)| (*key, *value)).collect();
    values.sort_by_key(|(key, _)| key.index());
    values
}
