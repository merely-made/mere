// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// Dispatch `id` to its cartography adapter and project against `graph` at viewport
/// `(width, height)`, returning the full [`cartography::Projection`] (positions + edges; overlays
/// are added by [`project_canvas_lens`]). [`Projection::empty`](cartography::Projection::empty) for
/// an unknown or not-yet-wired id, or radial without a focus. Only the graph-only analytic
/// strategies in [`CANVAS_LAYOUT_STRATEGIES`] are dispatched here.
///
/// Every fact an arrangement reads comes from `registry` (dynamics grammar plan, G2b). The request
/// carries the registry's facts keyed by channel id (F86): the enumeration order every score's
/// ordinal follows (`order.timeline`, F84), the rings from the focus (`rings.focus`) and the
/// spectral coordinates (`coords.spectral`), as `cartography::adapters::channels_read` names them.
/// The host's axes stay on the intent as view configuration: the sites (`groups.site`) and the
/// partition (`groups.cluster`, unless `clusters` hands one in) as Columns', and the enumeration
/// order as Timeline's. Cartography computes none of them.
#[allow(clippy::too_many_arguments)]
pub(super) fn project_canvas_dispatch(
    registry: &mut ChannelRegistry,
    id: &str,
    graph: &Graph,
    focus: Option<NodeKey>,
    width: u32,
    height: u32,
    clusters: Option<&cartography::ClusterSet>,
    extents: Option<&HashMap<NodeKey, (f32, f32)>>,
    recent_first: bool,
) -> cartography::Projection {
    use cartography::adapters::{KanbanAdapter, RadialAdapter, TimelineAdapter, channels_read};
    let mut options = CartographySceneOptions::canvas_pixels(width, height);
    options.extents = extents.cloned();
    // P3's Spiral is the product-free score path, not an arrangements adapter.
    // Its ordinal carries the registry's recency order, its score carries measured
    // footprints, and `scenomise` is the only generic solver that realizes it.
    if id == "phyllotaxis.default" {
        return spiral_in(registry, graph, extents, focus, recent_first, 1.0, None).projection;
    }
    // A host without a canvas has no arrangement of the person's: plain by-site Mosaic.
    if id == super::super::mosaic::MOSAIC_ARRANGEMENT {
        let state = super::super::mosaic::MosaicState::default();
        return super::super::mosaic::project_mosaic(
            registry, graph, &state, width, height, extents,
        );
    }
    // The axis an axis-driven arrangement reads, threaded on the intent (where `axis_values`
    // lives), from the registry's channel.
    let mut provenance_columns = Vec::new();
    let axis: Option<HashMap<NodeKey, AxisValue>> = match id {
        // Columns by site: a categorical column per URL authority; Partition's groups likewise.
        "kanban.default" | "partition.default" => Some(
            registry
                .sites(graph)
                .iter()
                .map(|(key, site)| (*key, AxisValue::Categorical(site.clone())))
                .collect(),
        ),
        // Columns by cluster: the column key is the registry's Louvain partition, the one Kinds
        // and Group pull by cluster read, so the board groups by how the graph clusters.
        "kanban.community" => {
            let clusters = match clusters {
                Some(handed) => handed,
                None => registry.community(graph),
            };
            let mut axis: HashMap<NodeKey, AxisValue> = HashMap::new();
            for (i, cluster) in clusters.clusters.iter().enumerate() {
                let label = cluster
                    .label
                    .clone()
                    .unwrap_or_else(|| format!("Cluster {}", i + 1));
                for &member in &cluster.members {
                    axis.insert(member, AxisValue::Categorical(label.clone()));
                }
            }
            Some(axis)
        },
        // Timeline lays nodes along the enumeration order, the order every score's ordinal
        // follows.
        "timeline.default" => Some(
            registry
                .enumeration(graph)
                .iter()
                .enumerate()
                .map(|(i, key)| (*key, AxisValue::Numeric(i as f64)))
                .collect(),
        ),
        // Focus-driven: rings outward from `focus` (the pane's selection), which the request
        // carries as `rings.focus`. Without a focus there is no layout to compute, so leave the
        // canvas as-is.
        // Sets reads tags as memberships; the focus sits in the core.
        "sets.default" => {
            options.focus = focus;
            Some(super::super::scene_axes::tag_sets(graph))
        },
        "provenance.default" => {
            let (axis, order) = super::super::scene_axes::derivation_columns(graph);
            provenance_columns = order;
            Some(axis)
        },
        "radial.default" => {
            let Some(focus) = focus else {
                return cartography::Projection::unavailable(graph, "projection focus unavailable");
            };
            options.focus = Some(focus);
            None
        },
        _ => None,
    };
    let signals = registry.disclose(graph, channels_read(id), focus);
    let mut intent = options.to_view_intent();
    intent.axis_values = axis;
    let request = ProjectionRequest {
        graph,
        signals: &signals,
        intent,
    };
    match id {
        "kanban.default" | "kanban.community" => KanbanAdapter::default().project(&request),
        "timeline.default" => TimelineAdapter::default().project(&request),
        "radial.default" => RadialAdapter::default().project(&request),
        "partition.default" => cartography::adapters::PartitionAdapter::default().project(&request),
        "sets.default" => cartography::adapters::SetsAdapter::default().project(&request),
        "provenance.default" => KanbanAdapter {
            config: sceno::Kanban {
                column_order: provenance_columns,
                ..sceno::Kanban::default()
            },
        }
        .project(&request),
        // Grid, Spectral, Penrose and L-system: cartography keeps their table, shared with the
        // mere view.
        other => cartography::adapters::project_graph_only(other, &request).unwrap_or_else(|| {
            cartography::Projection::unavailable(graph, "projection strategy unavailable")
        }),
    }
}

/// The signal-driven **overlays** for a lens, in the cartography [`Overlay`](cartography::Overlay)
/// vocabulary: a `ClusterHalo` per community (in cluster order, so a consumer's colour index matches
/// the main view) and a `BridgeEmphasis` per structural broker. **Position-independent** — overlays
/// carry only node references, so a consumer paints them at whatever positions it renders (the gloss
/// at its lens positions, the main view at its live positions). The overlays are the same regardless
/// of the layout strategy, which is why this is a pure function of the signals, not the projection.
/// (Graph signals — P6b, the overlay pipe.)
pub fn signal_overlays(
    clusters: Option<&cartography::ClusterSet>,
    bridges: Option<&cartography::BridgeNodes>,
) -> Vec<cartography::Overlay> {
    use cartography::Overlay;
    let mut overlays = Vec::new();
    if let Some(set) = clusters {
        for cluster in &set.clusters {
            overlays.push(Overlay::ClusterHalo {
                cluster_id: cluster.id.clone(),
                members: cluster.members.clone(),
                label: cluster.label.clone(),
                confidence: cluster.confidence,
            });
        }
    }
    if let Some(b) = bridges {
        for &node in &b.bridges {
            overlays.push(Overlay::BridgeEmphasis { node, weight: 1.0 });
        }
    }
    overlays
}

/// Project a layout strategy over the **induced subgraph** of `scope` (the scoped nodes plus the
/// relations whose endpoints are both in scope), returning positions keyed by the **original**
/// graph's `NodeKey`s. Lets the gloss lens lay a selection out by *its own* structure (e.g. a
/// spectral of just the selection) rather than cropping the whole-graph layout. Nodes are re-added
/// by their stable id, so each result position remaps back to the live graph; the induced edges are
/// added once per pair (topology — multiplicity is a later fidelity step). (Graph signals — P6c, the
/// gloss subgraph re-layout.)
pub fn project_canvas_subgraph(
    graph: &Graph,
    scope: &[NodeKey],
    id: &str,
    focus: Option<NodeKey>,
    width: u32,
    height: u32,
) -> Vec<(NodeKey, PortablePoint)> {
    let scope_set: HashSet<NodeKey> = scope.iter().copied().collect();
    let mut sub = Graph::new();
    // Re-add each scoped node by its stable id (so positions remap back). Positions are
    // no longer graph truth (S2), so the induced subgraph carries none — the layout
    // strategy re-derives them from structure. A missing node is simply skipped.
    for &key in scope {
        if let Some(node) = graph.get_node(key) {
            let _ = kernel::graph::apply::add_node(
                &mut sub,
                Some(node.id),
                node.url().to_string(),
                PortablePoint::zero(),
            );
        }
    }
    // Scratch topology: one surface relation per scoped pair, with no self-loops.
    let mut seen: HashSet<(NodeKey, NodeKey)> = HashSet::new();
    for (_, r) in graph.projected_relations() {
        if r.from == r.to || !scope_set.contains(&r.from) || !scope_set.contains(&r.to) {
            continue;
        }
        let pair = if r.from <= r.to {
            (r.from, r.to)
        } else {
            (r.to, r.from)
        };
        if !seen.insert(pair) {
            continue;
        }
        let (Some(fa), Some(fb)) = (graph.get_node(r.from), graph.get_node(r.to)) else {
            continue;
        };
        let (Some(sa), Some(sb)) = (sub.get_node_key_by_id(fa.id), sub.get_node_key_by_id(fb.id))
        else {
            continue;
        };
        let _ = kernel::graph::apply::assert_relation(
            &mut sub,
            sa,
            sb,
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::UserGrouped,
                label: None,
            },
        );
    }
    // Map the focus into the subgraph (only if it is in scope).
    let sub_focus = focus
        .and_then(|f| graph.get_node(f))
        .and_then(|n| sub.get_node_key_by_id(n.id));
    // Project the subgraph, then remap each position back to the original graph via the node id.
    // Extents are keyed by the original graph's NodeKeys, which do not survive the subgraph
    // re-add, so the subgraph path stays unmeasured for now; recency ordering likewise off. The
    // subgraph is a graph of its own, built for this call, so its facts come from a registry of
    // its own: one registry serves one graph (F87; *Reading, not ruled*).
    project_canvas_strategy(
        &mut ChannelRegistry::new(),
        id,
        &sub,
        sub_focus,
        width,
        height,
        None,
        None,
        false,
    )
    .into_iter()
    .filter_map(|(sub_key, pos)| {
        let nid = sub.get_node(sub_key)?.id;
        let main_key = graph.get_node_key_by_id(nid)?;
        Some((main_key, pos))
    })
    .collect()
}
