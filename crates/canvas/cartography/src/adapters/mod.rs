// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! [`LayoutStrategy`] adapters over the `scenograph` solvers.
//!
//! Each adapter does three things: pick an [`Arrangement`] and its config,
//! disclose whatever the arrangement needs to read, and hand the score to
//! `scenomise`. Nothing here places anything — the placement lives in
//! `scenomise`, where it is portable and testable without a graph.
//!
//! These arrived from `crates/canvas/arrangements`, which the
//! [scenograph absorption plan](../../../../design_docs/mere_docs/implementation_strategy/2026-08-22_scenograph_absorption_plan.md)
//! retired. They live here because they are the graph-bound half: they read
//! `kernel::graph::Graph`, and cartography already owns [`LayoutStrategy`].
//!
//! Live force physics is `seiche`'s domain and has no adapter here.

use std::collections::HashMap;

use kernel::graph::NodeKey;
use sceno::Arrangement;

use crate::projection::Projection;
use crate::request::{AxisValue, ProjectionRequest};
use crate::signals::{COORDS_HOST, COORDS_SPECTRAL, ORDER_TIMELINE, RINGS_FOCUS, WEIGHT_DEGREE};
use crate::strategy::LayoutStrategy;

#[cfg(test)]
mod parity;
pub mod score;

pub use score::{
    Disclosures, empty_projection, faulted_projection, project_arrangement, project_score,
    score_from_request,
};

/// Declare an adapter whose whole job is a config, an id, and the disclosures
/// it reads from the request.
macro_rules! analytic_adapter {
    (
        $(#[$meta:meta])*
        $name:ident, $id:literal, $config:ty, $variant:path
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Default, PartialEq)]
        pub struct $name {
            pub config: $config,
        }

        impl $name {
            pub const PROJECTION_ID: &'static str = $id;
        }

        impl LayoutStrategy for $name {
            fn projection_id(&self) -> &'static str {
                Self::PROJECTION_ID
            }

            fn project(&self, request: &ProjectionRequest<'_>) -> Projection {
                project_arrangement(
                    Self::PROJECTION_ID,
                    request,
                    $variant(self.config.clone()),
                    &Disclosures::from_intent(request),
                    Vec::new(),
                )
            }
        }
    };
}

/// How a grid picks its column count.
///
/// This stays adapter-side rather than moving into [`sceno::Grid`] because two
/// of the three modes need the item count, and a persisted score is supposed to
/// mean the same thing however many items it happens to carry. The adapter
/// resolves the count per request and the score records the number it chose.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum GridColumns {
    /// `ceil(sqrt(n))`, keeping the grid roughly square.
    #[default]
    Auto,
    /// A fixed count. Zero falls back to [`GridColumns::Auto`].
    Explicit(u32),
    /// The count best approximating a width/height ratio. `2.0` prefers wide
    /// grids, `0.5` tall ones.
    AspectRatio(f32),
}

impl GridColumns {
    fn resolve(self, count: usize) -> u32 {
        let auto = || (count as f32).sqrt().ceil().max(1.0) as u32;
        match self {
            Self::Auto | Self::Explicit(0) => auto(),
            Self::Explicit(columns) => columns.max(1),
            // columns × rows ≈ n and columns / rows = ratio, so
            // columns ≈ sqrt(n × ratio).
            Self::AspectRatio(ratio) => {
                let raw = (count as f32 * ratio.max(0.01)).sqrt().ceil() as u32;
                raw.max(1).min(count.max(1) as u32)
            },
        }
    }
}

/// Regular cell grid; items flow left-to-right from their ordinal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridAdapter {
    pub origin: sceno::Vec2,
    /// Centre-to-centre spacing. The score carries this as `gap` with a
    /// zero-size cell, since the adapter spaces by pitch rather than by
    /// measured extent.
    pub pitch: f32,
    pub columns: GridColumns,
}

impl Default for GridAdapter {
    fn default() -> Self {
        Self {
            origin: sceno::Vec2::ZERO,
            pitch: 120.0,
            columns: GridColumns::Auto,
        }
    }
}

impl GridAdapter {
    pub const PROJECTION_ID: &'static str = "grid.default";
}

impl LayoutStrategy for GridAdapter {
    fn projection_id(&self) -> &'static str {
        Self::PROJECTION_ID
    }

    fn project(&self, request: &ProjectionRequest<'_>) -> Projection {
        let count = request
            .signals
            .order(ORDER_TIMELINE)
            .map_or(0, |order| order.order.len());
        project_arrangement(
            Self::PROJECTION_ID,
            request,
            Arrangement::Grid(sceno::Grid {
                origin: self.origin,
                cell: sceno::Vec2::ZERO,
                columns: self.columns.resolve(count),
                gap: self.pitch,
            }),
            &Disclosures::from_intent(request),
            Vec::new(),
        )
    }
}

analytic_adapter!(
    /// Golden-angle spiral. The product-free score path in
    /// [`crate::spiral_score`] is the one the canvas uses; this is the plain
    /// strategy-shaped form.
    PhyllotaxisAdapter,
    "phyllotaxis.default",
    sceno::Spiral,
    Arrangement::Spiral
);

analytic_adapter!(
    /// Penrose aperiodic tiling; items take vertices in ordinal order.
    PenroseAdapter,
    "penrose.default",
    sceno::Penrose,
    Arrangement::Penrose
);

analytic_adapter!(
    /// L-system fractal path; items take positions along a turtle walk.
    LSystemAdapter,
    "lsystem.default",
    sceno::LSystem,
    Arrangement::LSystem
);

analytic_adapter!(
    /// Numeric axis. Reads `ViewIntent::axis_values`; the caller decides what
    /// the axis means.
    TimelineAdapter,
    "timeline.default",
    sceno::Timeline,
    Arrangement::Timeline
);

analytic_adapter!(
    /// Categorical columns. Reads `ViewIntent::axis_values`; the caller decides
    /// what the columns are.
    KanbanAdapter,
    "kanban.default",
    sceno::Kanban,
    Arrangement::Kanban
);

/// The strategies that lay out from the graph and the registry's facts alone,
/// needing no focus, axis or clusters. A host offering a layout choice over a
/// bare graph picks from these; Spectral reads the coordinates the host's
/// channel registry disclosed (`IntelligenceSignals::spectral`).
pub const GRAPH_ONLY_STRATEGIES: &[&str] = &[
    PhyllotaxisAdapter::PROJECTION_ID,
    GridAdapter::PROJECTION_ID,
    SpectralAdapter::PROJECTION_ID,
    PenroseAdapter::PROJECTION_ID,
    LSystemAdapter::PROJECTION_ID,
];

/// Project `request` with the graph-only strategy `id`, or `None` when `id`
/// names another strategy or none. The canvas's dispatch and the mere view
/// share this table.
pub fn project_graph_only(id: &str, request: &ProjectionRequest<'_>) -> Option<Projection> {
    Some(match id {
        PhyllotaxisAdapter::PROJECTION_ID => PhyllotaxisAdapter::default().project(request),
        GridAdapter::PROJECTION_ID => GridAdapter::default().project(request),
        // Positions from the graph Laplacian's smallest eigenvectors, so the
        // layout reflects connectivity: clusters separate, paths unroll.
        SpectralAdapter::PROJECTION_ID => SpectralAdapter::default().project(request),
        PenroseAdapter::PROJECTION_ID => PenroseAdapter::default().project(request),
        LSystemAdapter::PROJECTION_ID => LSystemAdapter::default().project(request),
        _ => return None,
    })
}

// No `StackAdapter`. `sceno::Stack` exists and is solved by `scenomise`, but
// nothing on this side asks for it: `stack.default` is absent from the canvas's
// `CANVAS_LAYOUT_STRATEGIES`, and mer3ly builds its own score with its own
// topological-rank producer rather than going through a cartography adapter. An
// adapter with no caller is a guess about a future one, and it would have to be
// re-derived against whatever that caller actually needs. Add it when something
// asks.

/// The coordinates under `id` as the arrangement reads them, or the fault.
fn read_coords(
    request: &ProjectionRequest<'_>,
    id: &str,
) -> (
    HashMap<NodeKey, sceno::Vec2>,
    Vec<crate::signals::SignalFault>,
) {
    match request.signals.coords(id) {
        Ok(coords) => (
            coords
                .coords
                .iter()
                .map(|(key, (x, y))| (*key, sceno::Vec2::new(*x, *y)))
                .collect(),
            Vec::new(),
        ),
        Err(fault) => (HashMap::new(), vec![fault]),
    }
}

/// The channels an arrangement reads from the request's signals, by
/// projection id: the order every score takes (F84), and the arrangement's
/// own, under each adapter's default configuration (Radial's weighted policy
/// also reads `weight.degree`). A host discloses these before projecting.
pub fn channels_read(id: &str) -> &'static [&'static str] {
    match id {
        SpectralAdapter::PROJECTION_ID => &[ORDER_TIMELINE, COORDS_SPECTRAL],
        SemanticEmbeddingAdapter::PROJECTION_ID => &[ORDER_TIMELINE, COORDS_HOST],
        RadialAdapter::PROJECTION_ID => &[ORDER_TIMELINE, RINGS_FOCUS],
        _ => &[ORDER_TIMELINE],
    }
}

/// Placement at coordinates a dimensionality reduction produced.
///
/// Reads `coords.host` — a host-run UMAP, t-SNE, or PCA.
/// The arrangement it emits is the same one [`SpectralAdapter`] emits; only the
/// producer differs.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SemanticEmbeddingAdapter {
    pub config: sceno::Embedded,
}

impl SemanticEmbeddingAdapter {
    pub const PROJECTION_ID: &'static str = "semantic.embedding";
}

impl LayoutStrategy for SemanticEmbeddingAdapter {
    fn projection_id(&self) -> &'static str {
        Self::PROJECTION_ID
    }

    fn project(&self, request: &ProjectionRequest<'_>) -> Projection {
        let (embedding, faults) = read_coords(request, COORDS_HOST);
        project_arrangement(
            Self::PROJECTION_ID,
            request,
            Arrangement::Embedded(self.config.clone()),
            &Disclosures::default().with_embedding(embedding),
            faults,
        )
    }
}

/// Placement at coordinates the graph Laplacian produced, so the layout
/// reflects connectivity: clusters separate spatially and a path unrolls into a
/// line.
///
/// The coordinates are a disclosure (`coords.spectral`), which the host's
/// channel registry computes once per structural revision (dynamics grammar
/// plan, G2b). An edgeless or symmetric graph's are empty, and every node
/// rings out; without the channel the projection reports it missing (F86).
#[derive(Debug, Clone, PartialEq)]
pub struct SpectralAdapter {
    pub config: sceno::Embedded,
    /// Power-iteration count the host's registry should produce the
    /// coordinates with. A producer parameter, not a placement one, which is
    /// why it sits here rather than in the arrangement.
    pub iterations: usize,
}

impl Default for SpectralAdapter {
    fn default() -> Self {
        Self {
            config: sceno::Embedded {
                // 320, not `Embedded`'s 400: the spectral strategy has always
                // fit its coordinates into this reach, and a score stored
                // against the old strategy must still land where it landed.
                scale: 320.0,
                ..sceno::Embedded::default()
            },
            iterations: 200,
        }
    }
}

impl SpectralAdapter {
    pub const PROJECTION_ID: &'static str = "spectral.default";
}

impl LayoutStrategy for SpectralAdapter {
    fn projection_id(&self) -> &'static str {
        Self::PROJECTION_ID
    }

    fn project(&self, request: &ProjectionRequest<'_>) -> Projection {
        let (embedding, faults) = read_coords(request, COORDS_SPECTRAL);
        project_arrangement(
            Self::PROJECTION_ID,
            request,
            Arrangement::Embedded(sceno::Embedded {
                // An edgeless or perfectly symmetric graph discloses no
                // coordinates at all. Ringing them out beats stacking every
                // node on the origin, which is what a collapse would do.
                fallback: sceno::EmbeddingFallback::RingOutside,
                ..self.config.clone()
            }),
            &Disclosures::default().with_embedding(embedding),
            faults,
        )
    }
}

/// Concentric rings around `ViewIntent::focus`.
///
/// Reads each node's ring from `rings.focus`, which the host's channel
/// registry walks from the focus, and the weighted policy's weights from
/// `weight.degree` (dynamics grammar plan, F86: Radial's rings are a signal,
/// not view configuration). Without a focus there is nothing to ring around,
/// and the projection is empty.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RadialAdapter {
    pub config: sceno::Radial,
}

impl RadialAdapter {
    pub const PROJECTION_ID: &'static str = "radial.default";
}

impl LayoutStrategy for RadialAdapter {
    fn projection_id(&self) -> &'static str {
        Self::PROJECTION_ID
    }

    fn project(&self, request: &ProjectionRequest<'_>) -> Projection {
        if request.intent.focus.is_none() {
            return empty_projection(Self::PROJECTION_ID);
        }

        let mut faults = Vec::new();
        let mut disclosures = Disclosures::default();
        match request.signals.rings(RINGS_FOCUS) {
            Ok(rings) => {
                disclosures = disclosures.with_axis(
                    rings
                        .rings
                        .iter()
                        .map(|(key, ring)| (*key, AxisValue::Numeric(f64::from(*ring))))
                        .collect(),
                );
            },
            Err(fault) => faults.push(fault),
        }
        // Only the weighted policy reads them.
        if matches!(
            self.config.angular_policy,
            sceno::RadialAngularPolicy::Weighted
        ) {
            match request.signals.weights(WEIGHT_DEGREE) {
                Ok(weights) => {
                    disclosures =
                        disclosures.with_weight(weights.weights.iter().copied().collect());
                },
                Err(fault) => faults.push(fault),
            }
        }
        project_arrangement(
            Self::PROJECTION_ID,
            request,
            Arrangement::Radial(self.config),
            &disclosures,
            faults,
        )
    }
}
