// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The graph-reading constructors of [`LawInputs`]: what the canvas resolves
//! from its graph, its hides, its registry's channels and its Louvain and
//! Meaning snapshots into the engine's inputs (`seiche::catalog`), which
//! borrow none of them. The builders themselves are the engine's; the physics
//! channels the registry caches reach them through [`RegistryChannels`].

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use kernel::graph::{Graph, NodeKey};
use seiche::catalog::{Channels, LawInputs, MeaningGroups};

#[cfg(test)]
use crate::canvas::seiche_bridge::visible_relation_edges;
use crate::canvas::seiche_bridge::visible_relation_edges_with_assertion_hides;
use crate::signals::physics::{PhysicsView, PhysicsViewKey};
use crate::signals::{ChannelRegistry, ClusterSet};

/// The registry serving the physics view at one key, as the engine's
/// [`Channels`]: each read is the registry's, one computation per key.
struct RegistryChannels<'a> {
    registry: &'a ChannelRegistry,
    key: PhysicsViewKey,
}

impl RegistryChannels<'_> {
    fn view<'v>(&self, nodes: &'v [NodeKey], edges: &'v [(NodeKey, NodeKey)]) -> PhysicsView<'v> {
        PhysicsView {
            key: self.key,
            nodes,
            edges,
        }
    }
}

impl Channels for RegistryChannels<'_> {
    fn mass_degree(&self, nodes: &[NodeKey], edges: &[(NodeKey, NodeKey)]) -> Vec<(NodeKey, f32)> {
        self.registry.mass_degree(self.view(nodes, edges))
    }

    fn mass_pagerank(
        &self,
        nodes: &[NodeKey],
        edges: &[(NodeKey, NodeKey)],
    ) -> Vec<(NodeKey, f32)> {
        self.registry.mass_pagerank(self.view(nodes, edges))
    }

    fn groups_coloring(
        &self,
        nodes: &[NodeKey],
        edges: &[(NodeKey, NodeKey)],
    ) -> Vec<(NodeKey, u32)> {
        self.registry.groups_coloring(self.view(nodes, edges))
    }

    fn groups_component(
        &self,
        nodes: &[NodeKey],
        edges: &[(NodeKey, NodeKey)],
    ) -> Vec<(NodeKey, u32)> {
        self.registry.groups_component(self.view(nodes, edges))
    }

    fn groups_degree(
        &self,
        nodes: &[NodeKey],
        edges: &[(NodeKey, NodeKey)],
    ) -> Vec<(NodeKey, u32)> {
        self.registry.groups_degree(self.view(nodes, edges))
    }

    fn depth_roots(&self, nodes: &[NodeKey], edges: &[(NodeKey, NodeKey)]) -> Vec<(NodeKey, u32)> {
        self.registry.depth_roots(self.view(nodes, edges))
    }

    fn depth_layers(&self, nodes: &[NodeKey], edges: &[(NodeKey, NodeKey)]) -> Vec<(NodeKey, u32)> {
        self.registry.depth_layers(self.view(nodes, edges))
    }

    fn depth_focus(
        &self,
        nodes: &[NodeKey],
        edges: &[(NodeKey, NodeKey)],
        focus: NodeKey,
    ) -> Vec<(NodeKey, u32)> {
        self.registry.depth_focus(self.view(nodes, edges), focus)
    }

    fn edges_spanning(
        &self,
        nodes: &[NodeKey],
        edges: &[(NodeKey, NodeKey)],
    ) -> Vec<(NodeKey, NodeKey)> {
        self.registry.edges_spanning(self.view(nodes, edges))
    }

    fn distances_hops(
        &self,
        nodes: &[NodeKey],
        edges: &[(NodeKey, NodeKey)],
    ) -> Vec<(NodeKey, NodeKey, f32)> {
        self.registry.distances_hops(self.view(nodes, edges))
    }
}

/// Every node's Louvain community index, a cluster at a time.
fn cluster_groups(clusters: &ClusterSet) -> Vec<(NodeKey, u32)> {
    let mut groups = Vec::new();
    for (i, cluster) in clusters.clusters.iter().enumerate() {
        for &member in &cluster.members {
            groups.push((member, i as u32));
        }
    }
    groups
}

/// The inputs over `graph`, reading `sites` when the caller's registry
/// holds them (computing them here otherwise, as a stats reader does).
#[cfg(test)]
pub(crate) fn law_inputs_over<'a>(
    graph: &Graph,
    hidden_edges: &HashSet<crate::canvas::EdgeCell>,
    clusters: Option<&ClusterSet>,
    sites: Option<&'a HashMap<NodeKey, String>>,
) -> LawInputs<'a> {
    law_inputs_with_edges(
        graph,
        visible_relation_edges(graph, hidden_edges),
        clusters,
        sites,
    )
}

/// Construct graph inputs for Canvas callers that immediately install the
/// complete link/assertion-hide-aware topology with [`with_link_hides`].
/// This avoids materializing and discarding the older cell-only grouping.
pub(crate) fn law_inputs_without_relation_edges<'a>(
    graph: &Graph,
    clusters: Option<&ClusterSet>,
    sites: Option<&'a HashMap<NodeKey, String>>,
) -> LawInputs<'a> {
    law_inputs_with_edges(graph, Vec::new(), clusters, sites)
}

fn law_inputs_with_edges<'a>(
    graph: &Graph,
    edges: Vec<(NodeKey, NodeKey)>,
    clusters: Option<&ClusterSet>,
    sites: Option<&'a HashMap<NodeKey, String>>,
) -> LawInputs<'a> {
    let sites = match sites {
        Some(sites) => Cow::Borrowed(sites),
        None => Cow::Owned(
            graph
                .nodes()
                .map(|(key, node)| (key, crate::canvas::channels::site_of(node).to_string()))
                .collect(),
        ),
    };
    LawInputs::from_parts(
        graph.nodes().map(|(key, _)| key).collect(),
        edges,
        HashMap::new(),
    )
    .with_sites(sites)
    .with_clusters(clusters.map(|set| Cow::Owned(cluster_groups(set))))
}

/// A controller's explicit transient topology, without source assertions.
pub(crate) fn with_occurrence_edges<'a>(
    inputs: LawInputs<'a>,
    edges: Option<&[(NodeKey, NodeKey)]>,
) -> LawInputs<'a> {
    match edges {
        Some(edges) => inputs.with_edges(edges.to_vec()),
        None => inputs,
    }
}

/// The visible edges once the link and assertion hides are applied.
pub(crate) fn with_link_hides<'a>(
    inputs: LawInputs<'a>,
    graph: &Graph,
    hidden_edges: &HashSet<crate::canvas::EdgeCell>,
    hidden_links: &HashSet<scenograph::link::LinkKey>,
    hidden_assertions: &HashSet<String>,
) -> LawInputs<'a> {
    inputs.with_edges(visible_relation_edges_with_assertion_hides(
        graph,
        hidden_edges,
        hidden_links,
        hidden_assertions,
    ))
}

/// Read Meaning groups from `snapshot` (site until one exists).
pub(crate) fn with_meaning<'a>(
    inputs: LawInputs<'a>,
    snapshot: Option<&'a crate::canvas::meaning::MeaningSnapshot>,
) -> LawInputs<'a> {
    inputs.with_meaning(snapshot.map(|snapshot| MeaningGroups {
        groups: Cow::Borrowed(&snapshot.groups),
        unknown: snapshot.clusters.clusters.len() as u32,
    }))
}

/// Read the physics channels from `registry`, these inputs being the physics
/// view at `key` (F178).
pub(crate) fn with_registry<'a>(
    inputs: LawInputs<'a>,
    registry: &'a ChannelRegistry,
    key: PhysicsViewKey,
) -> LawInputs<'a> {
    inputs.with_channels(Box::new(RegistryChannels { registry, key }))
}
