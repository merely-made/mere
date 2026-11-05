// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// The portable analytic arrangements.
///
/// Every family here is closed-form: a score in, placed instances out, no
/// iteration and no persistent solver state. Live force physics is `seiche`'s
/// domain and deliberately has no variant.
///
/// The named variants stay exhaustively matchable, which is what lets `solve`
/// prove it handles every family it claims to. [`Arrangement::Custom`] is the
/// escape for solvers registered at runtime — a consumer outside this
/// workspace can add an arrangement without a change here, at the cost of an
/// untyped config.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Arrangement {
    Spiral(Spiral),
    Grid(Grid),
    Geographic(Geographic),
    Hulls(Hulls),
    Stack(Stack),
    Penrose(Penrose),
    LSystem(LSystem),
    Timeline(Timeline),
    Kanban(Kanban),
    Embedded(Embedded),
    Radial(Radial),
    /// Shares of a measured whole, bounded by the viewport.
    Partition(crate::Partition),
    /// Fixed set regions; membership picks the region.
    Setscape(crate::Setscape),
    /// Tiles by weight on the infinite canvas, grouped, never reflowing.
    Mosaic(crate::Mosaic),
    /// A solver registered at runtime, named by id, configured opaquely.
    ///
    /// The id is matched against the `scenograph` solver registry. An
    /// unregistered id is a solve error, not a silent fallback: placing items
    /// by an arrangement nobody asked for is the same class of failure as
    /// dropping a pin.
    Custom {
        id: String,
        config: serde_json::Value,
    },
}

/// A golden-angle-family spiral. Spacing is a user-configurable lower bound;
/// solvers grow it when measured footprints require more clearance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spiral {
    pub center: Vec2,
    pub spacing: f32,
    pub angle_radians: f32,
    pub curve: SpiralCurve,
}

impl Default for Spiral {
    fn default() -> Self {
        Self {
            center: Vec2::ZERO,
            spacing: 22.0,
            angle_radians: 2.399_963_3,
            curve: SpiralCurve::SquareRoot,
        }
    }
}

/// Radius growth as a function of an item's ordinal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SpiralCurve {
    #[default]
    SquareRoot,
    Linear,
    Quadratic,
    Logarithmic,
}

/// A regular cell grid. Explicit cells preserve authored grid coordinates;
/// items without one flow left-to-right from their ordinal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Grid {
    pub origin: Vec2,
    pub cell: Vec2,
    pub columns: u32,
    pub gap: f32,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            origin: Vec2::ZERO,
            cell: Vec2::new(64.0, 64.0),
            columns: 8,
            gap: 4.0,
        }
    }
}

/// An affine-free geographic ground plane. Product adapters retain their
/// geocoding and fact selection; the score maps already-disclosed coordinates
/// into scene units.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Geographic {
    pub origin: Vec2,
    pub units_per_coordinate: f32,
    pub invert_y: bool,
}

impl Default for Geographic {
    fn default() -> Self {
        Self {
            origin: Vec2::ZERO,
            units_per_coordinate: 1.0,
            invert_y: true,
        }
    }
}

/// A bounded nearest-site partition: every coordinate-placed item is a site,
/// and each site's cell is the part of `bounds` nearer to it than to any other
/// site. The solver emits one scene [`Region`](crate::Region) per site, with
/// the cell as its polygon contour.
///
/// The regions *tile* the bounds: they cover it, they do not overlap, and a
/// point's cell is decided by the nearest-site rule rather than drawn. That is
/// what distinguishes Hulls from a derived cluster halo, and it is the same
/// rule Mesocosm's `Places::at` runs inside the simulation, so a hulls scene
/// over its places matches simulation truth exactly instead of approximating
/// it. What a cell *means* (a lineage's range, a faction's reach, a fertility
/// field) stays with the vessel; the contract carries only geometry and the
/// member it belongs to.
///
/// Coordinates map through origin/scale like [`Geographic`], so a product
/// adapter discloses positions in its own units.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hulls {
    pub origin: Vec2,
    pub units_per_coordinate: f32,
    pub invert_y: bool,
    /// The outer boundary the cells tile, in scene units. Cells are clipped to
    /// it, so the partition is total over exactly this much world.
    pub bounds: Rect,
}

impl Default for Hulls {
    fn default() -> Self {
        Self {
            origin: Vec2::ZERO,
            units_per_coordinate: 1.0,
            invert_y: true,
            bounds: Rect::new(Vec2::new(-256.0, -256.0), Size2::new(512.0, 512.0)),
        }
    }
}

/// A layered stack. Layer index arrives per item on [`ScoreItem::axis`]; the
/// producer decides what a layer means (topological rank, dependency depth,
/// generation) and which direction it flows.
///
/// The flow choice — whether sources precede targets or the reverse — lives
/// with the producer that computed the ranks, not here. By the time a score
/// exists the ranks are already numbers, and reversing them is the producer's
/// arithmetic, not a placement policy.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Stack {
    /// Distance between successive layers.
    pub layer_gap: f32,
    /// Distance between items within one layer.
    pub row_gap: f32,
    pub center: Vec2,
}

impl Default for Stack {
    fn default() -> Self {
        Self {
            layer_gap: 180.0,
            row_gap: 110.0,
            center: Vec2::ZERO,
        }
    }
}

/// A Penrose aperiodic tiling; items take tiling vertices in ordinal order.
///
/// Assignment strategy is deliberately absent. The `arrangements` original
/// carried a five-variant `NodeAssignmentStrategy` (subgraph-aware,
/// domain-clustered, UDC-clustered, edge-affinity) of which four were
/// documented as falling back to the fifth and none were implemented. All five
/// describe an *ordering*, and [`ScoreItem::ordinal`] is already this
/// contract's ordering channel: a producer that wants subgraph-adjacent items
/// on adjacent vertices sorts them into adjacent ordinals.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Penrose {
    pub variant: PenroseVariant,
    pub subdivision_count: SubdivisionCount,
    pub unused_vertices: UnusedVertexPolicy,
    pub center: Vec2,
    /// World-unit scale of the initial tile ring.
    pub tile_scale: f32,
}

impl Default for Penrose {
    fn default() -> Self {
        Self {
            variant: PenroseVariant::default(),
            subdivision_count: SubdivisionCount::default(),
            unused_vertices: UnusedVertexPolicy::default(),
            center: Vec2::ZERO,
            tile_scale: 400.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PenroseVariant {
    /// P3 — thin + thick golden rhombi. Smoother, more uniform.
    #[default]
    Rhombus,
    /// P2 — kite + dart. Chunkier, more distinct local motifs.
    KiteDart,
}

/// How many deflation steps to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SubdivisionCount {
    /// Smallest depth such that vertex count ≥ item count.
    #[default]
    Auto,
    /// Explicit depth; useful for artistic control and regression tests.
    Explicit(u8),
}

/// What becomes of tiling vertices no item claimed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UnusedVertexPolicy {
    /// Unused vertices remain empty; the tiling's full extent is visible as
    /// gaps around placed items. Reveals aperiodic structure.
    #[default]
    LeaveEmpty,
    /// Clip the layout bounds to the convex hull of used vertices. Tighter
    /// result; hides tiling periphery.
    ClipToHull,
    /// Hide the tiling backdrop entirely where the visual cost is not worth it
    /// (mobile, low-power paths).
    HideTiling,
}

/// A fractal path walked by an L-system turtle; items take path positions in
/// ordinal order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LSystem {
    pub grammar: LSystemGrammar,
    pub iteration_depth: IterationDepth,
    /// Origin for the turtle's starting position.
    pub origin: Vec2,
    /// Bounding-box edge length in world units. The walked path is normalized
    /// to fit this extent.
    pub size: f32,
    /// Rotation of the whole path around `origin`, in radians.
    pub rotation: f32,
    /// Reverse the item-to-position assignment.
    pub reverse_order: bool,
}

impl Default for LSystem {
    fn default() -> Self {
        Self {
            grammar: LSystemGrammar::default(),
            iteration_depth: IterationDepth::default(),
            origin: Vec2::ZERO,
            size: 400.0,
            rotation: 0.0,
            reverse_order: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LSystemGrammar {
    #[default]
    Hilbert,
    Koch,
    Dragon,
    /// A grammar supplied by the host, named by id. Distinct from
    /// [`Arrangement::Custom`]: the arrangement is still this one, only the
    /// production rules come from elsewhere.
    Named(String),
}

/// Iteration depth selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum IterationDepth {
    /// Smallest depth whose expansion yields at least as many positions as
    /// there are items.
    #[default]
    Auto,
    /// Explicit fractal depth; useful for deterministic comparisons or
    /// artistic control.
    Explicit(u8),
}

/// A numeric axis. Items place along it by [`ScoreItem::axis`]
/// ([`AxisValue::Numeric`]); items sharing a coordinate stack into rows.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Timeline {
    /// World-space origin of the axis (its leftmost edge).
    pub origin: Vec2,
    /// World-unit length of the axis.
    pub axis_length: f32,
    /// Vertical spacing between rows when items share nearby coordinates.
    pub row_gap: f32,
    pub fallback: TimelineFallback,
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            origin: Vec2::ZERO,
            axis_length: 800.0,
            row_gap: 40.0,
            fallback: TimelineFallback::default(),
        }
    }
}

/// Treatment for items with no [`AxisValue::Numeric`] disclosure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TimelineFallback {
    /// Leave unassigned items where they are.
    #[default]
    LeaveInPlace,
    /// Stack unassigned items vertically below the axis origin.
    StackBelowOrigin,
    /// Stack unassigned items vertically past the axis end.
    StackPastEnd,
}

/// Categorical columns. Items place by [`ScoreItem::axis`]
/// ([`AxisValue::Categorical`]) into named buckets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Kanban {
    pub origin: Vec2,
    /// Horizontal spacing between columns.
    pub column_gap: f32,
    /// Vertical spacing between entries within a column.
    pub row_gap: f32,
    /// Canonical ordering of columns, left to right. Items whose tag is absent
    /// here go to a trailing column.
    pub column_order: Vec<String>,
    /// Include the trailing column for unrecognized tags.
    pub include_other_column: bool,
}

impl Default for Kanban {
    fn default() -> Self {
        Self {
            origin: Vec2::ZERO,
            column_gap: 240.0,
            row_gap: 80.0,
            column_order: Vec::new(),
            include_other_column: true,
        }
    }
}

/// Placement at coordinates someone else computed: the solver applies origin,
/// scale and rotation to each item's [`ScoreItem::embedding`] and places the
/// result.
///
/// Normalizing into a sane range is the producer's job, not the solver's. Only
/// the producer knows whether its coordinates are a unit square, a covariance
/// spread, or eigenvector components a few thousandths wide, so it is the only
/// side that can rescale without guessing.
///
/// One arrangement serves every producer of 2-D coordinates. A dimensionality
/// reduction (UMAP, t-SNE, PCA) and a spectral decomposition (the two smallest
/// non-trivial eigenvectors of a graph Laplacian) disagree about everything
/// except their output shape, and the output shape is all a placement needs.
/// These were two separate layouts in `arrangements`, in different files under
/// different names, differing only in a power-iteration count that was never a
/// placement parameter at all.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Embedded {
    pub origin: Vec2,
    /// World-unit scale applied to the disclosed coordinates, which producers
    /// typically supply in `[-1, 1]` or `[0, 1]`.
    pub scale: f32,
    /// Rotation applied to the scaled coordinates, in radians.
    pub rotation: f32,
    pub fallback: EmbeddingFallback,
}

impl Default for Embedded {
    fn default() -> Self {
        Self {
            origin: Vec2::ZERO,
            scale: 400.0,
            rotation: 0.0,
            fallback: EmbeddingFallback::default(),
        }
    }
}

/// Treatment for items with no [`ScoreItem::embedding`] disclosure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EmbeddingFallback {
    /// Leave unembedded items where they are.
    #[default]
    LeaveInPlace,
    /// Place unembedded items at `origin`.
    CollapseToOrigin,
    /// Place unembedded items on a deterministic ring outside the embedded
    /// cluster, positions derived from a stable hash of the source reference.
    RingOutside,
}

/// Concentric rings. Ring index arrives per item on [`ScoreItem::axis`]
/// ([`AxisValue::Numeric`]); ring `n` sits at radius `n × ring_spacing`.
///
/// No focal item and no graph. The `arrangements` original walked a breadth
/// first search from a focus node, which is why its config was generic over a
/// node id — the only such generic in that crate. The walk is the producer's
/// work and its result is one number per item, so what reaches a score is the
/// ring, and the focus is simply whichever item the producer numbered zero.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Radial {
    /// World-space position of ring zero.
    pub center: Vec2,
    /// Radial distance between successive rings.
    pub ring_spacing: f32,
    pub angular_policy: RadialAngularPolicy,
    /// Global rotation applied to every ring, in radians. Zero puts the first
    /// angular slot on the +x axis.
    pub rotation_offset: f32,
    pub unreachable_policy: RadialUnreachablePolicy,
}

impl Default for Radial {
    fn default() -> Self {
        Self {
            center: Vec2::ZERO,
            ring_spacing: 120.0,
            angular_policy: RadialAngularPolicy::default(),
            rotation_offset: 0.0,
            unreachable_policy: RadialUnreachablePolicy::default(),
        }
    }
}

/// How a ring distributes its items around the circle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RadialAngularPolicy {
    /// Every item on a ring gets an equal slot.
    #[default]
    Uniform,
    /// Slot width is proportional to [`ScoreItem::weight`], so a producer that
    /// discloses adjacency degree gets hub-and-satellite structure. Items with
    /// no weight are treated as uniform.
    Weighted,
    /// Stable order derived from a hash of the source reference —
    /// deterministic, and independent of both ordinal and weight.
    HashSorted,
}

/// Treatment for items the producer could not assign a ring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RadialUnreachablePolicy {
    /// Place them on an outer ring, one beyond the deepest assigned ring.
    #[default]
    OuterRing,
    /// Collapse them to `center`, hiding disconnected structure.
    Center,
    /// Leave them where they are.
    LeaveInPlace,
}
