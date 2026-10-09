// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The channel registry: every source a law, overlay or slot reads, named by
//! id and resolved here, host-side. (Dynamics grammar plan, G2; F4, "Portable
//! shape now, in seiche": sources resolve host-side and the spec names
//! channels by id.)
//!
//! A channel is a family and an option, written `family.option`:
//! `kind.site`, `groups.meaning`, `mass.pagerank`, `depth.focus`,
//! `pairs.blend`, `distances.hops`. The option ids are the ones the saved
//! scene already stores per slot. Values come from the builders the laws
//! read ([`LawInputs`]) and the canvas's channel registry (the Louvain
//! partition, the sites, the affinity signals, the Meaning snapshot), so a
//! channel is one computation whoever reads it.
//!
//! Cartography's disclosures joined in G2b (F38, F55, "Reuse, else new
//! families"): Columns (by site) and (by cluster) read `groups.site` and
//! `groups.cluster`, and the rest take families of their own,
//! `order.recency`, `order.timeline`, `rings.focus`, `coords.spectral`,
//! `weight.degree`, `weight.recency` (F132, the Spiral's rungs),
//! `importance.degree` and `importance.betweenness`. The
//! bridge nodes are one channel, `groups.bridges`, whose metric (betweenness
//! or articulation) is the canvas's chosen bridge metric rather than part of
//! the id (F85, "groups.bridges").
//!
//! **The two degree ids name two roles over two computations** (F147,
//! correcting F144). `mass.degree` is a body's mass, what Gravity (Orbit)
//! and Density move by and the hub overlays weigh by: each node's degree over
//! the physics view's edges, the visible relation cells, so a pair joined by
//! two relations counts twice and a hidden edge not at all
//! (`LawInputs::mass_values`; Orbit and Density read 1 + it, the hub
//! overlays `ln(1 + degree)`). `weight.degree` is an arrangement's weight,
//! what Radial spreads its rings by: 1 + each node's undirected neighbours
//! over every kernel edge, hidden ones included (the registry's
//! `degree_weights`). They agree on a simple graph with nothing hidden and
//! may differ elsewhere; neither value moved when the ids were ruled.

use kernel::graph::{Graph, Node, NodeKey};

use super::physics_catalog::{LawInputs, PhysicsDepthSource, PhysicsKindSource, PhysicsMassSource};
use super::{
    AFFINITY_MIN_SIMILARITY, AffinityBlend, BRIDGE_THRESHOLD, Canvas, blend_affinity_pairs,
};
use crate::signals::ImportanceMetric;

/// A node's site: its URL authority. The one function behind the site
/// channel and Columns (by site)'s column key.
pub(crate) fn site_of(node: &Node) -> &str {
    Graph::url_grouping_key(node.url())
}

/// A channel's family: what kind of value it carries and who reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChannelFamily {
    /// A kind per node, folded to at most eight: Kinds' kinds.
    Kind,
    /// A group per node, unfolded: Group pull, and G3's grouped laws; and
    /// the bridge nodes (`groups.bridges`), a member set.
    Groups,
    /// A weight per node: a body's mass, Orbit's and Density's, and the hub
    /// overlays' weights, over the physics view's edges (F147).
    Mass,
    /// A depth per node: the Depth overlay.
    Depth,
    /// Weighted node pairs: the affinity force.
    Pairs,
    /// Graph distances between connected pairs: Stress.
    Distances,
    /// An order over the nodes: the Spiral's and the Timeline's.
    Order,
    /// Ring indices from the focus: Radial's.
    Rings,
    /// Coordinates per node: Spectral's.
    Coords,
    /// A weight per node an arrangement reads: Radial's weighted policy, the
    /// Spiral's rungs.
    Weight,
    /// Normalized importance per node: size by importance, the gloss.
    Importance,
}

impl ChannelFamily {
    pub const ALL: [ChannelFamily; 11] = [
        ChannelFamily::Kind,
        ChannelFamily::Groups,
        ChannelFamily::Mass,
        ChannelFamily::Depth,
        ChannelFamily::Pairs,
        ChannelFamily::Distances,
        ChannelFamily::Order,
        ChannelFamily::Rings,
        ChannelFamily::Coords,
        ChannelFamily::Weight,
        ChannelFamily::Importance,
    ];

    pub fn id(self) -> &'static str {
        match self {
            ChannelFamily::Kind => "kind",
            ChannelFamily::Groups => "groups",
            ChannelFamily::Mass => "mass",
            ChannelFamily::Depth => "depth",
            ChannelFamily::Pairs => "pairs",
            ChannelFamily::Distances => "distances",
            ChannelFamily::Order => "order",
            ChannelFamily::Rings => "rings",
            ChannelFamily::Coords => "coords",
            ChannelFamily::Weight => "weight",
            ChannelFamily::Importance => "importance",
        }
    }
}

/// Which order: most recently visited first, or the graph's enumeration
/// order (the one every score's ordinal follows).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OrderSource {
    Recency,
    Timeline,
}

/// Which weight: degree plus one, or recency in `0..=1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WeightSource {
    Degree,
    Recency,
}

/// One channel of the registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Channel {
    Kind(PhysicsKindSource),
    Groups(PhysicsKindSource),
    Mass(PhysicsMassSource),
    Depth(PhysicsDepthSource),
    /// Structural Jaccard, content (the Meaning pairs, or a host's), or
    /// their noisy-OR blend.
    Pairs(AffinityBlend),
    /// Shortest paths in hops, a hop costing `1 / multiplicity`.
    Distances,
    /// The recency order or the enumeration order.
    Order(OrderSource),
    /// Breadth-first rings from the focus.
    Rings,
    /// The graph Laplacian's coordinates.
    Coords,
    /// Degree plus one over every edge, or each node's recency.
    Weight(WeightSource),
    /// Importance by degree or betweenness, normalized.
    Importance(ImportanceMetric),
    /// The bridge nodes, under the canvas's bridge metric (betweenness
    /// brokers or articulation points).
    Bridges,
}

const ORDERS: [OrderSource; 2] = [OrderSource::Recency, OrderSource::Timeline];
const WEIGHTS: [WeightSource; 2] = [WeightSource::Degree, WeightSource::Recency];

fn weight_option(weight: WeightSource) -> &'static str {
    match weight {
        WeightSource::Degree => "degree",
        WeightSource::Recency => "recency",
    }
}
const IMPORTANCE: [ImportanceMetric; 2] = [ImportanceMetric::Degree, ImportanceMetric::Betweenness];

fn order_option(order: OrderSource) -> &'static str {
    match order {
        OrderSource::Recency => "recency",
        OrderSource::Timeline => "timeline",
    }
}

fn pairs_option(blend: AffinityBlend) -> &'static str {
    match blend {
        AffinityBlend::StructuralOnly => "structural",
        AffinityBlend::ContentOnly => "content",
        AffinityBlend::Blend => "blend",
    }
}

const PAIRS: [AffinityBlend; 3] = [
    AffinityBlend::StructuralOnly,
    AffinityBlend::ContentOnly,
    AffinityBlend::Blend,
];

impl Channel {
    /// Every channel, family by family.
    pub fn all() -> Vec<Channel> {
        let mut all = Vec::new();
        all.extend(PhysicsKindSource::ALL.map(Channel::Kind));
        all.extend(PhysicsKindSource::ALL.map(Channel::Groups));
        all.extend(PhysicsMassSource::ALL.map(Channel::Mass));
        all.extend(PhysicsDepthSource::ALL.map(Channel::Depth));
        all.extend(PAIRS.map(Channel::Pairs));
        all.push(Channel::Distances);
        all.extend(ORDERS.map(Channel::Order));
        all.extend([Channel::Rings, Channel::Coords]);
        all.extend(WEIGHTS.map(Channel::Weight));
        all.extend(IMPORTANCE.map(Channel::Importance));
        all.push(Channel::Bridges);
        all
    }

    pub fn family(self) -> ChannelFamily {
        match self {
            Channel::Kind(_) => ChannelFamily::Kind,
            Channel::Groups(_) => ChannelFamily::Groups,
            Channel::Mass(_) => ChannelFamily::Mass,
            Channel::Depth(_) => ChannelFamily::Depth,
            Channel::Pairs(_) => ChannelFamily::Pairs,
            Channel::Distances => ChannelFamily::Distances,
            Channel::Order(_) => ChannelFamily::Order,
            Channel::Rings => ChannelFamily::Rings,
            Channel::Coords => ChannelFamily::Coords,
            Channel::Weight(_) => ChannelFamily::Weight,
            Channel::Importance(_) => ChannelFamily::Importance,
            Channel::Bridges => ChannelFamily::Groups,
        }
    }

    /// The option's id within its family.
    pub fn option(self) -> &'static str {
        match self {
            Channel::Kind(source) | Channel::Groups(source) => source.id(),
            Channel::Mass(source) => source.id(),
            Channel::Depth(source) => source.id(),
            Channel::Pairs(blend) => pairs_option(blend),
            Channel::Distances => "hops",
            Channel::Order(order) => order_option(order),
            Channel::Rings => "focus",
            Channel::Coords => "spectral",
            Channel::Weight(weight) => weight_option(weight),
            Channel::Importance(metric) => metric.as_code(),
            Channel::Bridges => "bridges",
        }
    }

    /// `family.option`.
    pub fn id(self) -> String {
        format!("{}.{}", self.family().id(), self.option())
    }

    /// The channel with this id; `None` for an unknown family or option.
    pub fn parse(id: &str) -> Option<Channel> {
        let (family, option) = id.split_once('.')?;
        match family {
            "kind" => PhysicsKindSource::parse(option).map(Channel::Kind),
            "groups" if option == "bridges" => Some(Channel::Bridges),
            "groups" => PhysicsKindSource::parse(option).map(Channel::Groups),
            "mass" => PhysicsMassSource::parse(option).map(Channel::Mass),
            "depth" => PhysicsDepthSource::parse(option).map(Channel::Depth),
            "pairs" => PAIRS
                .into_iter()
                .find(|blend| pairs_option(*blend) == option)
                .map(Channel::Pairs),
            "distances" => (option == "hops").then_some(Channel::Distances),
            "order" => ORDERS
                .into_iter()
                .find(|order| order_option(*order) == option)
                .map(Channel::Order),
            "rings" => (option == "focus").then_some(Channel::Rings),
            "coords" => (option == "spectral").then_some(Channel::Coords),
            "weight" => WEIGHTS
                .into_iter()
                .find(|weight| weight_option(*weight) == option)
                .map(Channel::Weight),
            "importance" => IMPORTANCE
                .into_iter()
                .find(|metric| metric.as_code() == option)
                .map(Channel::Importance),
            _ => None,
        }
    }
}

/// A channel's values.
#[derive(Clone, Debug, PartialEq)]
pub enum ChannelValues {
    /// A kind or group per node, in key order.
    Groups(Vec<(NodeKey, u32)>),
    /// A weight per node, in key order.
    Weights(Vec<(NodeKey, f32)>),
    /// A depth per node, in key order.
    Depths(Vec<(NodeKey, u32)>),
    /// Weighted pairs.
    Pairs(Vec<(NodeKey, NodeKey, f32)>),
    /// Every node, in the order's order.
    Order(Vec<NodeKey>),
    /// A ring per node the focus reaches, in key order; empty without a focus.
    Rings(Vec<(NodeKey, u32)>),
    /// A coordinate pair per node, in key order; empty for a graph with no
    /// structure to project.
    Coords(Vec<(NodeKey, (f32, f32))>),
    /// Nodes, in key order.
    Nodes(Vec<NodeKey>),
}

fn in_key_order<V: Copy>(map: &std::collections::HashMap<NodeKey, V>) -> Vec<(NodeKey, V)> {
    let mut values: Vec<(NodeKey, V)> = map.iter().map(|(key, value)| (*key, *value)).collect();
    values.sort_by_key(|(key, _)| key.index());
    values
}

impl Canvas {
    /// The graph inputs every channel resolves through, with the partition
    /// and the Meaning snapshot as they stand.
    fn channel_inputs(&mut self) -> LawInputs<'_> {
        self.channels.sites(&self.graph);
        LawInputs::new(
            &self.graph,
            &self.hidden_edges,
            self.channels.community_held(),
            self.channels.sites_fresh(&self.graph),
        )
        .with_meaning(self.meaning.snapshot())
        .with_registry(&self.channels, self.physics_view_key())
    }

    /// Resolve a channel against the current graph and the registry. It reads
    /// the Louvain partition and the Meaning snapshot as they stand (cluster
    /// and meaning read site before either exists); a reader that needs them
    /// fresh asks for them first, as a law build does. The registry's other
    /// facts are brought up to the graph as they are read.
    pub fn channel_values(&mut self, channel: Channel) -> ChannelValues {
        match channel {
            Channel::Order(OrderSource::Recency) => {
                return ChannelValues::Order(self.channels.recency(&self.graph).order.clone());
            },
            Channel::Order(OrderSource::Timeline) => {
                return ChannelValues::Order(self.channels.enumeration(&self.graph).to_vec());
            },
            Channel::Rings => {
                let rings = match self.focused_key() {
                    Some(focus) => in_key_order(self.channels.rings(&self.graph, focus)),
                    None => Vec::new(),
                };
                return ChannelValues::Rings(rings);
            },
            Channel::Coords => {
                let iterations = cartography::adapters::SpectralAdapter::default().iterations;
                return ChannelValues::Coords(in_key_order(
                    self.channels.spectral(&self.graph, iterations),
                ));
            },
            Channel::Weight(WeightSource::Degree) => {
                return ChannelValues::Weights(in_key_order(
                    self.channels.degree_weights(&self.graph),
                ));
            },
            Channel::Weight(WeightSource::Recency) => {
                return ChannelValues::Weights(in_key_order(
                    &self.channels.recency(&self.graph).values,
                ));
            },
            Channel::Importance(metric) => {
                return ChannelValues::Weights(in_key_order(
                    self.channels.importance(&self.graph, metric),
                ));
            },
            Channel::Bridges => {
                let mut nodes = self
                    .channels
                    .bridges(&self.graph, self.bridge_metric, BRIDGE_THRESHOLD)
                    .bridges
                    .clone();
                nodes.sort_by_key(|key| key.index());
                return ChannelValues::Nodes(nodes);
            },
            Channel::Pairs(blend) if blend != AffinityBlend::ContentOnly => {
                self.channels
                    .structural_affinity(&self.graph, AFFINITY_MIN_SIMILARITY);
            },
            _ => {},
        }
        let focus = self.focused_key();
        let inputs = self.channel_inputs();
        match channel {
            Channel::Kind(source) => ChannelValues::Groups(
                inputs
                    .kinds(source)
                    .0
                    .into_iter()
                    .map(|(key, kind)| (key, u32::from(kind)))
                    .collect(),
            ),
            Channel::Groups(source) => ChannelValues::Groups(inputs.groups(source)),
            Channel::Mass(source) => ChannelValues::Weights(inputs.mass_values(source)),
            Channel::Depth(source) => ChannelValues::Depths(inputs.depth_values(source, focus)),
            Channel::Pairs(blend) => {
                let structural = (blend != AffinityBlend::ContentOnly)
                    .then(|| {
                        self.channels
                            .structural_affinity_fresh(&self.graph, AFFINITY_MIN_SIMILARITY)
                    })
                    .flatten();
                let content = (blend != AffinityBlend::StructuralOnly)
                    .then_some(self.content_affinity.as_deref())
                    .flatten();
                ChannelValues::Pairs(blend_affinity_pairs(structural, content))
            },
            Channel::Distances => ChannelValues::Pairs(inputs.weighted_distances()),
            Channel::Order(_)
            | Channel::Rings
            | Channel::Coords
            | Channel::Weight(_)
            | Channel::Importance(_)
            | Channel::Bridges => unreachable!("resolved from the registry above"),
        }
    }

    /// The groups channel for any kind channel, as G3's grouped laws will
    /// read it: every node's group, dense, in key order.
    pub fn channel_groups(&mut self, source: PhysicsKindSource) -> Vec<(NodeKey, u32)> {
        self.channel_inputs().groups(source)
    }
}
