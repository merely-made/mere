// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Cartography projection-request derivation.
//!
//! Translates platen's reducer-owned graph state into a
//! [`cartography::ProjectionRequest`] and dispatches it through a
//! chosen [`cartography::LayoutStrategy`].
//!
//! Sibling of [`crate::canvas::canvas_scene`]: where `canvas_scene` builds a
//! `CanvasSceneInput` for graph-canvas to render directly,
//! `cartography_scene` builds the higher-level `ProjectionRequest`
//! that goes through cartography's strategy contract first — the
//! eventual home for graph-view layout selection (force-directed,
//! radial, phyllotaxis, cluster-collapsed, etc.) once the host wants
//! per-pane strategy choice.
//!
//! ## Two paths, same input shape
//!
//! For new code, prefer [`project_with`] (analytic strategies) over
//! building a `ProjectionRequest` by hand. It sources the right
//! `ViewIntent` / `IntelligenceSignals` shape from
//! [`CartographySceneOptions`] and dispatches through the strategy in
//! one call.
//!
//! `canvas_scene::build_canvas_scene_input` still works for the
//! existing graph-canvas-only render path; the two modules will
//! eventually share infrastructure once cartography becomes the
//! single dispatch surface, but until graph-canvas's existing direct-
//! render path is fully retired they coexist.

use std::collections::{HashMap, HashSet};

use cartography::{
    AxisValue, FormFactor, IntelligenceSignals, LayoutStrategy, ProjectionDimension,
    ProjectionRequest, TargetSize, ViewIntent,
};
use kernel::geometry::PortablePoint;
use kernel::graph::{EdgeAssertion, Graph, NodeKey, SemanticSubKind};

use crate::signals::ChannelRegistry;

/// Inputs the host supplies for a cartography projection.
///
/// Signals default to empty (`IntelligenceSignals::default()`), so
/// callers that don't have intelligence plumbed yet pass none and the
/// projection still works — strategies that read signals (e.g.
/// `KanbanAdapter`'s community columns, or the importance-driven size
/// encoding) just behave as if no signals were available.
#[derive(Clone, Debug)]
pub struct CartographySceneOptions {
    pub form_factor: FormFactor,
    pub dimension: ProjectionDimension,
    pub focus: Option<NodeKey>,
    pub target_size: TargetSize,
    /// Per-node visual extents `(w, h)` in px, threaded onto
    /// `ViewIntent::extents` so extent-aware strategies space placements to
    /// clear them. `None` = unmeasured (pre-P2 behavior). (Projection
    /// proofs — P2.)
    pub extents: Option<HashMap<NodeKey, (f32, f32)>>,
}

impl Default for CartographySceneOptions {
    fn default() -> Self {
        Self {
            form_factor: FormFactor::Canvas,
            dimension: ProjectionDimension::TwoD,
            focus: None,
            target_size: TargetSize::Default,
            extents: None,
        }
    }
}

impl CartographySceneOptions {
    pub fn canvas_pixels(width: u32, height: u32) -> Self {
        Self {
            target_size: TargetSize::Pixels { width, height },
            ..Self::default()
        }
    }

    pub fn minimap(width: f32, height: f32) -> Self {
        Self {
            form_factor: FormFactor::Minimap,
            target_size: TargetSize::Logical { width, height },
            ..Self::default()
        }
    }

    pub fn with_focus(mut self, focus: NodeKey) -> Self {
        self.focus = Some(focus);
        self
    }

    fn to_view_intent(&self) -> ViewIntent {
        let mut intent = ViewIntent::default();
        intent.form_factor = self.form_factor;
        intent.dimension = self.dimension;
        intent.focus = self.focus;
        intent.target_size = self.target_size;
        intent.extents = self.extents.clone();
        intent
    }
}

/// Build a [`ProjectionRequest`] borrowing the supplied graph and
/// signals.
///
/// The returned request has its `ViewIntent` constructed from
/// `options`. Callers can mutate the request before dispatching if
/// they need to set `axis_values`, `filter`, or other intent fields
/// not exposed on [`CartographySceneOptions`].
pub fn build_projection_request<'a>(
    graph: &'a Graph,
    signals: &'a IntelligenceSignals,
    options: &CartographySceneOptions,
) -> ProjectionRequest<'a> {
    ProjectionRequest {
        graph,
        signals,
        intent: options.to_view_intent(),
    }
}

/// Dispatch an analytic strategy. Convenience wrapper around
/// [`build_projection_request`] + `strategy.project(&request)`.
pub fn project_with<S: LayoutStrategy>(
    graph: &Graph,
    signals: &IntelligenceSignals,
    options: &CartographySceneOptions,
    strategy: &S,
) -> cartography::Projection {
    let request = build_projection_request(graph, signals, options);
    strategy.project(&request)
}

/// The graph-wide layout strategies the canvas's empty-canvas picker offers:
/// `(projection_id, label)`. The force-directed default (seiche physics) is the host's
/// `None`, not listed here. These analytic adapters lay out from the node set alone,
/// needing no selection focus. Focus-driven `radial.default` is *not* here — it rides
/// the selection menu (it centers on the selected node) and dispatches through
/// [`project_canvas_strategy`] all the same. Axis- and signal-driven strategies join
/// once their inputs are plumbed.
pub const CANVAS_LAYOUT_STRATEGIES: &[(&str, &str)] = &[
    // Labels use the plain arrangement register (Spiral, not Phyllotaxis)
    // per the projection-proofs naming table; ids stay technical.
    ("phyllotaxis.default", "Spiral"),
    ("grid.default", "Grid"),
    ("spectral.default", "Spectral"),
    ("penrose.default", "Penrose"),
    ("lsystem.default", "Fractal"),
    // Axis-driven, now dispatched: the host axes are graph-derived. Columns groups by URL host
    // (`by site`) or by graph-structural **community** (`by cluster`, the Louvain partition from
    // intel/signals); timeline orders by node-creation order. (Arrangements — kanban/timeline;
    // graph signals — community to columns.)
    ("kanban.default", "Columns (by site)"),
    ("kanban.community", "Columns (by cluster)"),
    ("timeline.default", "Timeline (by order)"),
];

/// The main canvas's strategy result. Existing strategies emit positions;
/// the P3 Spiral also returns the portable score it realized.
#[derive(Clone, Debug, PartialEq)]
pub struct CanvasStrategyProjection {
    pub positions: Vec<(NodeKey, PortablePoint)>,
    pub score: Option<sceno::Score>,
    pub coverage: kernel::graph::CoverageNote,
}

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
fn project_canvas_dispatch(
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
    // The axis an axis-driven arrangement reads, threaded on the intent (where `axis_values`
    // lives), from the registry's channel.
    let axis: Option<HashMap<NodeKey, AxisValue>> = match id {
        // Columns by site: a categorical column per URL authority.
        "kanban.default" => Some(
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
        // Grid, Spectral, Penrose and L-system: cartography keeps their table, shared with the
        // mere view.
        other => cartography::adapters::project_graph_only(other, &request).unwrap_or_else(|| {
            cartography::Projection::unavailable(graph, "projection strategy unavailable")
        }),
    }
}

/// The Spiral over the registry's recency: recent first (`order.recency`), or in the enumeration
/// order (`order.timeline`), its rungs by `weight.recency`, all handed in as keyed signals (F132).
fn spiral_in(
    registry: &mut ChannelRegistry,
    graph: &Graph,
    extents: Option<&HashMap<NodeKey, (f32, f32)>>,
    focus: Option<NodeKey>,
    recent_first: bool,
    zoom_level: f32,
    previous: Option<&sceno::Score>,
) -> cartography::MereSpiralProjection {
    let order = if recent_first {
        cartography::ORDER_RECENCY
    } else {
        cartography::ORDER_TIMELINE
    };
    let signals = registry.disclose(graph, &[order, cartography::WEIGHT_RECENCY], focus);
    cartography::project_spiral_score_for_view(
        graph, &signals, order, extents, focus, zoom_level, previous,
    )
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

/// Compute an canvas layout strategy's full [`cartography::Projection`] — positions **plus**
/// signal-driven [`overlays`](cartography::Projection::overlays) (community halos + bridge emphasis,
/// from [`signal_overlays`]). The gloss is the first consumer of the overlay channel: it paints the
/// overlays at its own lens positions, so a second lens can show the clusters/brokers under a
/// different layout than the main view. Positions-only callers use [`project_canvas_strategy`].
/// (Graph signals — P6b, the overlay pipe.) The facts come from `registry`, the one serving `graph`
/// (dynamics grammar plan, F87).
#[allow(clippy::too_many_arguments)]
pub fn project_canvas_lens(
    registry: &mut ChannelRegistry,
    id: &str,
    graph: &Graph,
    focus: Option<NodeKey>,
    width: u32,
    height: u32,
    clusters: Option<&cartography::ClusterSet>,
    bridges: Option<&cartography::BridgeNodes>,
) -> cartography::Projection {
    let mut projection = project_canvas_dispatch(
        registry, id, graph, focus, width, height, clusters, None, false,
    );
    projection.overlays = signal_overlays(clusters, bridges);
    projection
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
                decay_progress: None,
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

/// Compute an canvas layout strategy's node positions: the `(NodeKey, position)` pairs the canvas
/// applies through [`Canvas::apply_strategy_positions`](../../canvas). Empty for an unknown or
/// not-yet-wired id (the host then leaves the layout unchanged). The positions-only path the main
/// view uses; the gloss uses [`project_canvas_lens`] when it also needs the overlay channel. The
/// facts come from `registry`, the one serving `graph`: a canvas's own
/// ([`Canvas::registry_and_graph`](crate::canvas::Canvas::registry_and_graph)), or one a host
/// without a canvas holds across calls (dynamics grammar plan, F87).
#[allow(clippy::too_many_arguments)]
pub fn project_canvas_strategy(
    registry: &mut ChannelRegistry,
    id: &str,
    graph: &Graph,
    focus: Option<NodeKey>,
    width: u32,
    height: u32,
    clusters: Option<&cartography::ClusterSet>,
    extents: Option<&HashMap<NodeKey, (f32, f32)>>,
    recent_first: bool,
) -> Vec<(NodeKey, PortablePoint)> {
    project_canvas_dispatch(
        registry,
        id,
        graph,
        focus,
        width,
        height,
        clusters,
        extents,
        recent_first,
    )
    .nodes
    .iter()
    .map(|n| (n.node, n.position))
    .collect()
}

/// Like [`project_canvas_strategy`], with the product-free score when the
/// selected strategy has one. Mere builds the score; `scenomise` alone solves
/// its placement.
#[allow(clippy::too_many_arguments)]
pub fn project_canvas_strategy_with_score(
    registry: &mut ChannelRegistry,
    id: &str,
    graph: &Graph,
    focus: Option<NodeKey>,
    width: u32,
    height: u32,
    clusters: Option<&cartography::ClusterSet>,
    extents: Option<&HashMap<NodeKey, (f32, f32)>>,
    recent_first: bool,
) -> CanvasStrategyProjection {
    project_canvas_strategy_with_score_for_view(
        registry,
        id,
        graph,
        focus,
        width,
        height,
        clusters,
        extents,
        recent_first,
        1.0,
        None,
    )
}

impl crate::canvas::Canvas {
    /// This canvas's channel registry and the graph it serves, for the free projection functions
    /// (dynamics grammar plan, F87, "Registry argument": a canvas passes its own).
    pub fn registry_and_graph(&mut self) -> (&mut ChannelRegistry, &Graph) {
        (&mut self.channels, &self.graph)
    }

    /// Project arrangement `id` over this canvas's graph with the facts its
    /// channel registry holds: every disclosure the arrangement reads, the
    /// Louvain partition Kinds and Group pull by cluster read among them, each
    /// computed once per key. The host's entry for an arrangement, so a
    /// scene's arrangement and its dynamics read one binding. (Dynamics
    /// grammar plan, G2 and G2b; F21, "Two slots, one binding".)
    #[allow(clippy::too_many_arguments)]
    pub fn project_arrangement_for_view(
        &mut self,
        id: &str,
        width: u32,
        height: u32,
        extents: Option<&HashMap<NodeKey, (f32, f32)>>,
        recent_first: bool,
        zoom_level: f32,
        previous_score: Option<&sceno::Score>,
    ) -> CanvasStrategyProjection {
        let focus = self.focused_key();
        project_strategy_in(
            &mut self.channels,
            id,
            &self.graph,
            focus,
            width,
            height,
            None,
            extents,
            recent_first,
            zoom_level,
            previous_score,
        )
    }
}

/// Like [`project_canvas_strategy_with_score`], evaluated for the host's
/// current zoom and prior score. These two view facts affect only the selected
/// representation rung and its hysteresis; placement remains score-driven.
/// Reads its facts from `registry`, the one serving `graph` (F87).
#[allow(clippy::too_many_arguments)]
pub fn project_canvas_strategy_with_score_for_view(
    registry: &mut ChannelRegistry,
    id: &str,
    graph: &Graph,
    focus: Option<NodeKey>,
    width: u32,
    height: u32,
    clusters: Option<&cartography::ClusterSet>,
    extents: Option<&HashMap<NodeKey, (f32, f32)>>,
    recent_first: bool,
    zoom_level: f32,
    previous_score: Option<&sceno::Score>,
) -> CanvasStrategyProjection {
    project_strategy_in(
        registry,
        id,
        graph,
        focus,
        width,
        height,
        clusters,
        extents,
        recent_first,
        zoom_level,
        previous_score,
    )
}

#[allow(clippy::too_many_arguments)]
fn project_strategy_in(
    registry: &mut ChannelRegistry,
    id: &str,
    graph: &Graph,
    focus: Option<NodeKey>,
    width: u32,
    height: u32,
    clusters: Option<&cartography::ClusterSet>,
    extents: Option<&HashMap<NodeKey, (f32, f32)>>,
    recent_first: bool,
    zoom_level: f32,
    previous_score: Option<&sceno::Score>,
) -> CanvasStrategyProjection {
    if id == "phyllotaxis.default" {
        let result = spiral_in(
            registry,
            graph,
            extents,
            focus,
            recent_first,
            zoom_level,
            previous_score,
        );
        return CanvasStrategyProjection {
            positions: result
                .projection
                .nodes
                .iter()
                .map(|node| (node.node, node.position))
                .collect(),
            score: Some(result.score),
            coverage: result.projection.metadata.coverage,
        };
    }
    let projection = project_canvas_dispatch(
        registry,
        id,
        graph,
        focus,
        width,
        height,
        clusters,
        extents,
        recent_first,
    );
    CanvasStrategyProjection {
        positions: projection
            .nodes
            .iter()
            .map(|n| (n.node, n.position))
            .collect(),
        score: None,
        coverage: projection.metadata.coverage,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::geometry::PortablePoint;
    use kernel::graph::fixtures::GraphFixtures;
    use uuid::Uuid;

    fn triangle_graph() -> (Graph, [NodeKey; 3]) {
        let mut graph = Graph::new();
        let a = graph.add_node_with_id(
            Uuid::from_u128(1),
            "test://a".into(),
            PortablePoint::new(0.0, 0.0),
        );
        let b = graph.add_node_with_id(
            Uuid::from_u128(2),
            "test://b".into(),
            PortablePoint::new(100.0, 0.0),
        );
        let c = graph.add_node_with_id(
            Uuid::from_u128(3),
            "test://c".into(),
            PortablePoint::new(50.0, 86.6),
        );
        (graph, [a, b, c])
    }

    #[test]
    fn coverage_refreshes_without_recomputing_channels_or_geometry() {
        use kernel::graph::{CoverageLayer, CoverageLimit, CoverageNote};
        let (mut graph, _) = triangle_graph();
        let mut registry = ChannelRegistry::new();
        for id in ["phyllotaxis.default", "grid.default", "kanban.default"] {
            graph.set_known_coverage(Default::default());
            let before = project_canvas_strategy_with_score(
                &mut registry,
                id,
                &graph,
                None,
                800,
                600,
                None,
                None,
                false,
            );
            let runs = registry.runs();
            let note = CoverageNote {
                limits: CoverageLayer::ALL
                    .into_iter()
                    .map(|layer| CoverageLimit::new(layer, "host boundary"))
                    .collect(),
            };
            graph.set_known_coverage(note.clone());
            let after = project_canvas_strategy_with_score(
                &mut registry,
                id,
                &graph,
                None,
                800,
                600,
                None,
                None,
                false,
            );
            let mut before_positions = before.positions;
            let mut after_positions = after.positions;
            before_positions.sort_by_key(|(key, _)| *key);
            after_positions.sort_by_key(|(key, _)| *key);
            assert_eq!(before_positions, after_positions);
            assert_eq!(before.score, after.score);
            assert_eq!(registry.runs(), runs);
            assert!(
                note.limits
                    .iter()
                    .all(|limit| after.coverage.limits.contains(limit)),
                "{id}"
            );
        }
        for (id, reason) in [
            ("radial.default", "projection focus unavailable"),
            ("unknown", "projection strategy unavailable"),
        ] {
            let result = project_canvas_strategy_with_score(
                &mut registry,
                id,
                &graph,
                None,
                800,
                600,
                None,
                None,
                false,
            );
            assert!(result.positions.is_empty());
            assert!(result.coverage.limits.iter().any(|limit| limit.layer == CoverageLayer::Projection && limit.reason == reason));
            assert!(
                graph
                    .known_coverage()
                    .limits
                    .iter()
                    .all(|limit| result.coverage.limits.contains(limit))
            );
        }
    }

    #[test]
    fn project_spiral_uses_the_portable_score_solver() {
        let (graph, _) = triangle_graph();
        let projection = project_canvas_dispatch(
            &mut ChannelRegistry::new(),
            "phyllotaxis.default",
            &graph,
            None,
            800,
            600,
            None,
            None,
            false,
        );
        assert_eq!(projection.nodes.len(), 3);
        assert_eq!(
            projection.metadata.strategy_id.as_deref(),
            Some("phyllotaxis.default")
        );
        assert!(projection.metadata.settled);
    }

    #[test]
    fn kanban_community_groups_each_cluster_into_its_own_column() {
        // Two triangles {0,1,2} and {3,4,5} joined by one bridge edge => two Louvain communities,
        // so the cluster-kanban board lays them out as two columns.
        let mut graph = Graph::new();
        let n: Vec<NodeKey> = (0..6)
            .map(|i| graph.add_node(format!("test://{i}"), PortablePoint::new(i as f32, 0.0)))
            .collect();
        for &(a, b) in &[(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)] {
            graph.assert_semantic_predicate(n[a], n[b], "links".to_string());
        }
        let positions = project_canvas_strategy(
            &mut ChannelRegistry::new(),
            "kanban.community",
            &graph,
            None,
            800,
            600,
            None,
            None,
            false,
        );
        assert_eq!(positions.len(), 6, "every node is placed");
        let x_of = |key: NodeKey| positions.iter().find(|(k, _)| *k == key).unwrap().1.x;
        assert!(
            (x_of(n[0]) - x_of(n[1])).abs() < 0.001,
            "triangle A shares one column"
        );
        assert!((x_of(n[0]) - x_of(n[2])).abs() < 0.001);
        assert!(
            (x_of(n[3]) - x_of(n[4])).abs() < 0.001,
            "triangle B shares one column"
        );
        assert!(
            (x_of(n[0]) - x_of(n[3])).abs() > 0.001,
            "the two communities are distinct columns"
        );
    }

    #[test]
    fn project_canvas_strategy_radial_centers_on_focus_and_no_ops_without_one() {
        let (graph, [a, _, _]) = triangle_graph();
        // With a focus, radial lays out the whole graph (focus at center).
        let with_focus = project_canvas_strategy(
            &mut ChannelRegistry::new(),
            "radial.default",
            &graph,
            Some(a),
            800,
            600,
            None,
            None,
            false,
        );
        assert_eq!(
            with_focus.len(),
            3,
            "radial projects every node around the focus"
        );
        let focus_pos = with_focus.iter().find(|(k, _)| *k == a).unwrap().1;
        assert!(
            focus_pos.x.abs() < 0.001 && focus_pos.y.abs() < 0.001,
            "the focus sits at the radial center"
        );
        // Without a focus there is nothing to center on, so it leaves the layout alone.
        let no_focus = project_canvas_strategy(
            &mut ChannelRegistry::new(),
            "radial.default",
            &graph,
            None,
            800,
            600,
            None,
            None,
            false,
        );
        assert!(no_focus.is_empty(), "radial without a selection no-ops");
    }

    #[test]
    fn signal_overlays_builds_halos_in_cluster_order_then_bridge_emphasis() {
        use cartography::{BridgeNodes, Cluster, ClusterSet, Overlay};
        let (a, b, c) = (NodeKey::new(0), NodeKey::new(1), NodeKey::new(2));
        let clusters = ClusterSet {
            clusters: vec![
                Cluster {
                    id: "c0".into(),
                    label: Some("A".into()),
                    members: vec![a, b],
                    confidence: 1.0,
                },
                Cluster {
                    id: "c1".into(),
                    label: None,
                    members: vec![c],
                    confidence: 1.0,
                },
            ],
        };
        let bridges = BridgeNodes { bridges: vec![b] };
        let overlays = signal_overlays(Some(&clusters), Some(&bridges));
        assert_eq!(overlays.len(), 3, "two halos + one bridge emphasis");
        assert!(
            matches!(&overlays[0], Overlay::ClusterHalo { members, .. } if *members == vec![a, b]),
            "the first halo is the first cluster (order preserved for colour matching)"
        );
        assert!(
            matches!(&overlays[1], Overlay::ClusterHalo { cluster_id, .. } if cluster_id == "c1")
        );
        assert!(matches!(&overlays[2], Overlay::BridgeEmphasis { node, .. } if *node == b));
    }

    #[test]
    fn signal_overlays_empty_without_signals() {
        assert!(
            signal_overlays(None, None).is_empty(),
            "no signals => no overlays"
        );
    }

    #[test]
    fn project_canvas_lens_carries_the_community_halos() {
        // Two triangles + a bridge: the lens projection carries the two community halos on the
        // overlay channel, so the gloss can paint the clusters at its own (spectral) positions.
        let mut graph = Graph::new();
        let n: Vec<NodeKey> = (0..6)
            .map(|i| graph.add_node(format!("test://{i}"), PortablePoint::new(i as f32, 0.0)))
            .collect();
        for &(a, b) in &[(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)] {
            graph.assert_semantic_predicate(n[a], n[b], "links".to_string());
        }
        let clusters = crate::signals::community_louvain(&graph);
        let lens = project_canvas_lens(
            &mut ChannelRegistry::new(),
            "spectral.default",
            &graph,
            None,
            800,
            600,
            Some(&clusters),
            None,
        );
        assert_eq!(lens.nodes.len(), 6, "spectral lays out every node");
        let halos = lens
            .overlays
            .iter()
            .filter(|o| matches!(o, cartography::Overlay::ClusterHalo { .. }))
            .count();
        assert_eq!(halos, 2, "two communities => two halos ride the projection");
        // The positions-only path is unchanged (it drops the overlays).
        let positions = project_canvas_strategy(
            &mut ChannelRegistry::new(),
            "spectral.default",
            &graph,
            None,
            800,
            600,
            Some(&clusters),
            None,
            false,
        );
        assert_eq!(
            positions.len(),
            6,
            "the positions wrapper still returns every node"
        );
    }

    #[test]
    fn project_canvas_subgraph_lays_out_only_the_scope() {
        // Path a-b-c-d; scope to {a, b}. The subgraph projection returns positions only for the
        // scoped nodes, keyed by the ORIGINAL graph's keys (remapped via stable id). (P6c.)
        let mut graph = Graph::new();
        let n: Vec<NodeKey> = (0..4)
            .map(|i| graph.add_node(format!("test://{i}"), PortablePoint::new(i as f32, 0.0)))
            .collect();
        for &(a, b) in &[(0, 1), (1, 2), (2, 3)] {
            graph.assert_relation(
                n[a],
                n[b],
                EdgeAssertion::Semantic {
                    sub_kind: SemanticSubKind::Hyperlink,
                    label: None,
                    decay_progress: None,
                },
            );
        }
        let scope = vec![n[0], n[1]];
        let positions = project_canvas_subgraph(&graph, &scope, "spectral.default", None, 800, 600);
        assert_eq!(positions.len(), 2, "only the scoped nodes are laid out");
        let keys: HashSet<NodeKey> = positions.iter().map(|(k, _)| *k).collect();
        assert!(
            keys.contains(&n[0]) && keys.contains(&n[1]),
            "positions are keyed by the original graph keys"
        );
        assert!(
            !keys.contains(&n[2]) && !keys.contains(&n[3]),
            "out-of-scope nodes are absent from the lens"
        );
    }

    #[test]
    fn build_projection_request_threads_intent_from_options() {
        let (graph, [a, _, _]) = triangle_graph();
        let signals = IntelligenceSignals::default();
        let options = CartographySceneOptions::minimap(120.0, 90.0).with_focus(a);
        let request = build_projection_request(&graph, &signals, &options);
        assert_eq!(request.intent.form_factor, FormFactor::Minimap);
        assert_eq!(request.intent.focus, Some(a));
        assert_eq!(
            request.intent.target_size,
            TargetSize::Logical {
                width: 120.0,
                height: 90.0
            }
        );
    }

    #[test]
    fn options_default_is_canvas_form_factor() {
        let opts = CartographySceneOptions::default();
        assert_eq!(opts.form_factor, FormFactor::Canvas);
        assert_eq!(opts.dimension, ProjectionDimension::TwoD);
        assert_eq!(opts.target_size, TargetSize::Default);
    }
}
