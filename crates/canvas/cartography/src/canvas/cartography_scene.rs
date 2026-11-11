// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Cartography projection-request derivation.
//!
//! Translates platen's reducer-owned graph state into a
//! [`crate::ProjectionRequest`] and dispatches it through a
//! chosen [`crate::LayoutStrategy`].
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

use crate::{
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

    pub(crate) fn to_view_intent(&self) -> ViewIntent {
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
) -> crate::Projection {
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
    // The graph grammar's bounded scenes (DECISIONS 2026-10-16): Partition subdivides the
    // viewport by site, sized by degree; Sets draws the four most common tags as fixed regions;
    // Provenance stands sources left, one column per derivation step.
    ("partition.default", "Partition (by site)"),
    ("sets.default", "Sets (by tag)"),
    ("provenance.default", "Provenance"),
    // Tiles by weight on the infinite canvas, grouped by site, organised by the person.
    ("mosaic.default", "Mosaic (by site)"),
];

/// The main canvas's strategy result. Existing strategies emit positions;
/// the P3 Spiral also returns the portable score it realized.
#[derive(Clone, Debug, PartialEq)]
pub struct CanvasStrategyProjection {
    pub positions: Vec<(NodeKey, PortablePoint)>,
    pub score: Option<sceno::Score>,
    pub coverage: kernel::graph::CoverageNote,
    /// Regions a bounded arrangement draws (Partition, Sets).
    pub regions: Vec<crate::ProjectedRegion>,
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
) -> crate::MereSpiralProjection {
    let order = if recent_first {
        crate::ORDER_RECENCY
    } else {
        crate::ORDER_TIMELINE
    };
    let signals = registry.disclose(graph, &[order, crate::WEIGHT_RECENCY], focus);
    crate::project_spiral_score_for_view(
        graph, &signals, order, extents, focus, zoom_level, previous,
    )
}

/// Compute an canvas layout strategy's full [`crate::Projection`] — positions **plus**
/// signal-driven [`overlays`](crate::Projection::overlays) (community halos + bridge emphasis,
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
    clusters: Option<&crate::ClusterSet>,
    bridges: Option<&crate::BridgeNodes>,
) -> crate::Projection {
    let mut projection = project_canvas_dispatch(
        registry, id, graph, focus, width, height, clusters, None, false,
    );
    projection.overlays = signal_overlays(clusters, bridges);
    projection
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
    clusters: Option<&crate::ClusterSet>,
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
    clusters: Option<&crate::ClusterSet>,
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
        if id == super::mosaic::MOSAIC_ARRANGEMENT {
            let projection = super::mosaic::project_mosaic(
                &mut self.channels,
                &self.graph,
                &self.mosaic,
                width,
                height,
                extents,
            );
            return CanvasStrategyProjection {
                positions: projection
                    .nodes
                    .iter()
                    .map(|n| (n.node, n.position))
                    .collect(),
                score: None,
                coverage: projection.metadata.coverage,
                regions: projection.regions,
            };
        }
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
    clusters: Option<&crate::ClusterSet>,
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
    clusters: Option<&crate::ClusterSet>,
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
            regions: Vec::new(),
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
        regions: projection.regions,
    }
}

#[cfg(test)]
#[path = "cartography_scene/tests.rs"]
mod tests;

#[path = "cartography_scene/projection.rs"]
mod projection;

use projection::project_canvas_dispatch;

pub use projection::{project_canvas_subgraph, signal_overlays};
