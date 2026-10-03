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
//! read ([`LawInputs`]) and the caches the canvas keeps (the Louvain
//! partition, the Meaning snapshot, the affinity signals), so a channel is
//! one computation whoever reads it.

use kernel::graph::{Graph, Node, NodeKey};

use super::physics_catalog::{LawInputs, PhysicsDepthSource, PhysicsKindSource, PhysicsMassSource};
use super::{AFFINITY_MIN_SIMILARITY, AffinityBlend, Canvas, blend_affinity_pairs};

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
    /// A group per node, unfolded: Group pull, and G3's grouped laws.
    Groups,
    /// A weight per node: Orbit's masses, the hub overlays' weights.
    Mass,
    /// A depth per node: the Depth overlay.
    Depth,
    /// Weighted node pairs: the affinity force.
    Pairs,
    /// Graph distances between connected pairs: Stress.
    Distances,
}

impl ChannelFamily {
    pub const ALL: [ChannelFamily; 6] = [
        ChannelFamily::Kind,
        ChannelFamily::Groups,
        ChannelFamily::Mass,
        ChannelFamily::Depth,
        ChannelFamily::Pairs,
        ChannelFamily::Distances,
    ];

    pub fn id(self) -> &'static str {
        match self {
            ChannelFamily::Kind => "kind",
            ChannelFamily::Groups => "groups",
            ChannelFamily::Mass => "mass",
            ChannelFamily::Depth => "depth",
            ChannelFamily::Pairs => "pairs",
            ChannelFamily::Distances => "distances",
        }
    }
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
            "groups" => PhysicsKindSource::parse(option).map(Channel::Groups),
            "mass" => PhysicsMassSource::parse(option).map(Channel::Mass),
            "depth" => PhysicsDepthSource::parse(option).map(Channel::Depth),
            "pairs" => PAIRS
                .into_iter()
                .find(|blend| pairs_option(*blend) == option)
                .map(Channel::Pairs),
            "distances" => (option == "hops").then_some(Channel::Distances),
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
}

impl Canvas {
    /// The graph inputs every channel resolves through, with the partition
    /// and the Meaning snapshot as they stand.
    fn channel_inputs(&self) -> LawInputs<'_> {
        LawInputs::new(
            &self.graph,
            &self.hidden_edges,
            self.community_cache.as_ref(),
        )
        .with_meaning(self.meaning.snapshot())
    }

    /// Resolve a channel against the current graph and caches. It reads the
    /// Louvain partition and the Meaning snapshot as they stand (cluster and
    /// meaning read site before either exists); a reader that needs them
    /// fresh asks for them first, as a law build does.
    pub fn channel_values(&self, channel: Channel) -> ChannelValues {
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
            Channel::Depth(source) => {
                ChannelValues::Depths(inputs.depth_values(source, self.focused_key()))
            },
            Channel::Pairs(blend) => {
                let structural = (blend != AffinityBlend::ContentOnly).then(|| {
                    match (&self.affinity_cache, self.affinity_cache_revision) {
                        (Some(cache), revision) if revision == self.graph.revision() => {
                            cache.clone()
                        },
                        _ => crate::signals::structural_affinity(
                            &self.graph,
                            AFFINITY_MIN_SIMILARITY,
                        ),
                    }
                });
                let content = (blend != AffinityBlend::StructuralOnly)
                    .then_some(self.content_affinity.as_deref())
                    .flatten();
                ChannelValues::Pairs(blend_affinity_pairs(structural.as_ref(), content))
            },
            Channel::Distances => ChannelValues::Pairs(inputs.weighted_distances()),
        }
    }

    /// The groups channel for any kind channel, as G3's grouped laws will
    /// read it: every node's group, dense, in key order.
    pub fn channel_groups(&self, source: PhysicsKindSource) -> Vec<(NodeKey, u32)> {
        self.channel_inputs().groups(source)
    }
}
