// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The built-in arrangement catalog: sceno's eleven named families, by id.
//!
//! An authored recipe names its arrangement by id. An id found here resolves to
//! the family's typed [`sceno::Arrangement`] variant; any other id falls through
//! to the [`SolverRegistry`](crate::SolverRegistry) as `Arrangement::Custom`.
//! Each entry also declares how its family reads a recipe's x and y encoding,
//! so every family is authorable through the same grammar.
//!
//! Parameters a recipe leaves unset are measured from the items rather than
//! written in: a grid cell fits the largest item, gaps are that item plus the
//! authored spacing, and extents grow with the item count. Parameters nothing
//! can measure (a spiral's curve, a tiling's variant, a grammar, a rotation)
//! take sceno's own defaults. A named option overrides either; an option the
//! family does not read is refused rather than ignored.

use std::collections::BTreeMap;

use sceno::{
    Arrangement, Embedded, EmbeddingFallback, Geographic, Grid, Hulls, IterationDepth, Kanban,
    LSystem, LSystemGrammar, Penrose, PenroseVariant, Radial, RadialAngularPolicy,
    RadialUnreachablePolicy, Rect, Size2, Spiral, SpiralCurve, Stack, SubdivisionCount, Timeline,
    TimelineFallback, UnusedVertexPolicy, Vec2,
};
use scenograph::dataset::ProjectionFieldType;
use scenograph::options::{OptionDefault, OptionKind, OptionSpec};

use crate::projection::{GRID_ARRANGEMENT_ID, SCATTER_ARRANGEMENT_ID};
use crate::registry::{Disclosure, SolverCapability};

/// One of sceno's named arrangement families.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Family {
    Spiral,
    Grid,
    Geographic,
    Hulls,
    Stack,
    Penrose,
    LSystem,
    Timeline,
    Kanban,
    Embedded,
    Radial,
}

/// Every family in the catalog, in a stable order.
pub const FAMILIES: [Family; 11] = [
    Family::Spiral,
    Family::Grid,
    Family::Geographic,
    Family::Hulls,
    Family::Stack,
    Family::Penrose,
    Family::LSystem,
    Family::Timeline,
    Family::Kanban,
    Family::Embedded,
    Family::Radial,
];

/// How a family reads a recipe's x and y encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelUse {
    /// Numeric x and y rank into explicit grid cells.
    Cells,
    /// Numeric (x, y) is a disclosed coordinate.
    Coordinate,
    /// Numeric (x, y) is a 2D embedding.
    Embedding,
    /// Numeric x is the item's axis position.
    NumericAxis,
    /// Integral x is the item's layer, carried on the axis.
    IntegerAxis,
    /// Text x is the item's axis bucket.
    CategoricalAxis,
    /// Items are ordered by x, text or number.
    Order,
}

impl ChannelUse {
    /// The field types x may have.
    pub fn x_types(self) -> &'static [ProjectionFieldType] {
        match self {
            Self::Cells
            | Self::Coordinate
            | Self::Embedding
            | Self::NumericAxis
            | Self::IntegerAxis => &[ProjectionFieldType::Number],
            Self::CategoricalAxis => &[ProjectionFieldType::Text],
            Self::Order => &[ProjectionFieldType::Number, ProjectionFieldType::Text],
        }
    }

    /// Whether y is read. One-dimensional families ignore it.
    pub fn reads_y(self) -> bool {
        matches!(self, Self::Cells | Self::Coordinate | Self::Embedding)
    }
}

/// What the items measured, for parameters a recipe leaves unset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measure {
    /// The largest item footprint, as the host measured it.
    pub largest: Size2,
    pub count: usize,
    /// The recipe's spacing, in scene units.
    pub spacing: f32,
    /// The smallest and largest disclosed (x, y), in the source's units, for
    /// families that read coordinates.
    pub coordinates: Option<(Vec2, Vec2)>,
}

impl Measure {
    fn pitch_w(&self) -> f32 {
        self.largest.w + self.spacing
    }

    fn pitch_h(&self) -> f32 {
        self.largest.h + self.spacing
    }

    /// The spacing an item needs in every direction.
    fn pitch(&self) -> f32 {
        self.largest.w.max(self.largest.h) + self.spacing
    }

    /// Items per side of the square that holds them all.
    fn side(&self) -> f32 {
        (self.count.max(1) as f32).sqrt().ceil()
    }
}

/// An option the catalog refused, keyed by its authored field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionIssue {
    pub field: String,
    pub message: String,
}

impl Family {
    pub fn id(self) -> &'static str {
        match self {
            Self::Spiral => "spiral",
            Self::Grid => "grid",
            Self::Geographic => "geographic",
            Self::Hulls => "hulls",
            Self::Stack => "stack",
            Self::Penrose => "penrose",
            Self::LSystem => "lsystem",
            Self::Timeline => "timeline",
            Self::Kanban => "kanban",
            Self::Embedded => "embedded",
            Self::Radial => "radial",
        }
    }

    /// Older ids that saved recipes carry.
    pub fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::Grid => &[GRID_ARRANGEMENT_ID],
            Self::Geographic => &[SCATTER_ARRANGEMENT_ID],
            _ => &[],
        }
    }

    /// The family an authored id or alias names, if it is built in.
    pub fn resolve(id: &str) -> Option<Self> {
        FAMILIES
            .into_iter()
            .find(|family| family.id() == id || family.aliases().contains(&id))
    }

    pub fn channels(self) -> ChannelUse {
        match self {
            Self::Grid => ChannelUse::Cells,
            Self::Geographic | Self::Hulls => ChannelUse::Coordinate,
            Self::Embedded => ChannelUse::Embedding,
            Self::Timeline | Self::Radial => ChannelUse::NumericAxis,
            Self::Stack => ChannelUse::IntegerAxis,
            Self::Kanban => ChannelUse::CategoricalAxis,
            Self::Spiral | Self::Penrose | Self::LSystem => ChannelUse::Order,
        }
    }

    /// The options this family reads, declared as data: one declaration for
    /// refusal and for an editor's rows (Scenograph editor plan, ruling B, SE7).
    pub fn options(self) -> Vec<OptionSpec> {
        let finite = |key: &str, label: &str, default: f32| {
            OptionSpec::new(
                key,
                label,
                OptionKind::Finite,
                OptionDefault::Value(default.to_string()),
            )
        };
        let measured = |key: &str, label: &str, kind: OptionKind, words: &str| {
            OptionSpec::new(key, label, kind, OptionDefault::Measured(words.to_owned()))
        };
        let flag = |key: &str, label: &str, default: bool| {
            OptionSpec::new(
                key,
                label,
                OptionKind::Flag,
                OptionDefault::Value(default.to_string()),
            )
        };
        fn choice<T: PartialEq + Default>(
            key: &str,
            label: &str,
            table: &[(&str, T)],
        ) -> OptionSpec {
            let default = T::default();
            let name = table
                .iter()
                .find(|(_, value)| *value == default)
                .map(|(name, _)| (*name).to_owned())
                .expect("a choice table names its default");
            OptionSpec::new(
                key,
                label,
                OptionKind::Choice {
                    names: table.iter().map(|(name, _)| (*name).to_owned()).collect(),
                },
                OptionDefault::Value(name),
            )
        }
        let auto_depth = |key: &str, label: &str| {
            OptionSpec::new(key, label, OptionKind::Depth, OptionDefault::Auto)
        };
        let spacing = "the recipe's spacing";
        match self {
            Self::Spiral => vec![
                finite(
                    "angle_radians",
                    "Turn per item",
                    Spiral::default().angle_radians,
                ),
                choice("curve", "Curve", SPIRAL_CURVES),
            ],
            Self::Grid => vec![
                measured(
                    "cell_width",
                    "Cell width",
                    OptionKind::Positive,
                    "the largest item's width",
                ),
                measured(
                    "cell_height",
                    "Cell height",
                    OptionKind::Positive,
                    "the largest item's height",
                ),
                measured(
                    "columns",
                    "Columns",
                    OptionKind::Count,
                    "the side of the square that holds every item",
                ),
            ],
            Self::Geographic | Self::Hulls => vec![
                measured(
                    "units_per_coordinate",
                    "Units per coordinate",
                    OptionKind::Positive,
                    spacing,
                ),
                flag("invert_y", "Invert y", false),
            ],
            Self::Stack => vec![
                measured(
                    "layer_gap",
                    "Layer gap",
                    OptionKind::Positive,
                    "the largest item's height plus spacing",
                ),
                measured(
                    "row_gap",
                    "Row gap",
                    OptionKind::Positive,
                    "the largest item's width plus spacing",
                ),
            ],
            Self::Penrose => vec![
                choice("variant", "Variant", PENROSE_VARIANTS),
                auto_depth("subdivisions", "Subdivisions"),
                choice(
                    "unused_vertices",
                    "Unused vertices",
                    PENROSE_UNUSED_VERTICES,
                ),
                measured(
                    "tile_scale",
                    "Tile scale",
                    OptionKind::Positive,
                    "an item's pitch, scaled so the tiling's disc holds every item",
                ),
            ],
            Self::LSystem => vec![
                choice("grammar", "Grammar", LSYSTEM_GRAMMARS),
                auto_depth("depth", "Depth"),
                measured(
                    "size",
                    "Size",
                    OptionKind::Positive,
                    "the side of the square that holds every item, times an item's pitch",
                ),
                finite("rotation", "Rotation", 0.0),
                flag("reverse_order", "Reverse order", false),
            ],
            Self::Timeline => vec![
                measured(
                    "axis_length",
                    "Axis length",
                    OptionKind::Positive,
                    "the item count times an item's width plus spacing",
                ),
                measured(
                    "row_gap",
                    "Row gap",
                    OptionKind::Positive,
                    "the largest item's height plus spacing",
                ),
                choice("fallback", "Fallback", TIMELINE_FALLBACKS),
            ],
            Self::Kanban => vec![
                measured(
                    "column_gap",
                    "Column gap",
                    OptionKind::Positive,
                    "the largest item's width plus spacing",
                ),
                measured(
                    "row_gap",
                    "Row gap",
                    OptionKind::Positive,
                    "the largest item's height plus spacing",
                ),
                OptionSpec::new(
                    "column_order",
                    "Column order",
                    OptionKind::List,
                    OptionDefault::Empty,
                ),
                flag("include_other_column", "Include an other column", true),
            ],
            Self::Embedded => vec![
                measured("scale", "Scale", OptionKind::Positive, spacing),
                finite("rotation", "Rotation", 0.0),
                choice("fallback", "Fallback", EMBEDDING_FALLBACKS),
            ],
            Self::Radial => vec![
                measured(
                    "ring_spacing",
                    "Ring spacing",
                    OptionKind::Positive,
                    "an item's pitch",
                ),
                choice("angular_policy", "Angular policy", RADIAL_ANGULAR_POLICIES),
                finite("rotation_offset", "Rotation offset", 0.0),
                choice(
                    "unreachable_policy",
                    "Unreachable items",
                    RADIAL_UNREACHABLE_POLICIES,
                ),
            ],
        }
    }

    /// What the family advertises to pickers. Every built-in family is closed
    /// form, so every one replays.
    pub fn capability(self) -> SolverCapability {
        let (name, description, tags): (&str, &str, &[&str]) = match self {
            Self::Spiral => ("Spiral", "Items wind outward in order.", &["organic"]),
            Self::Grid => ("Grid", "Items take ranked cells.", &["spatial-memory"]),
            Self::Geographic => (
                "Geographic",
                "Items sit at their coordinates.",
                &["spatial-memory"],
            ),
            Self::Hulls => (
                "Hulls",
                "Each item owns its nearest region.",
                &["spatial-memory"],
            ),
            Self::Stack => (
                "Stack",
                "Items layer by an integral rank.",
                &["hierarchical"],
            ),
            Self::Penrose => (
                "Penrose",
                "Items take aperiodic tiling vertices in order.",
                &["organic"],
            ),
            Self::LSystem => (
                "L-system",
                "Items follow a fractal path in order.",
                &["organic"],
            ),
            Self::Timeline => (
                "Timeline",
                "Items place along a numeric axis.",
                &["time-axis"],
            ),
            Self::Kanban => ("Kanban", "Items sort into named columns.", &[]),
            Self::Embedded => (
                "Embedded",
                "Items sit at computed 2D coordinates.",
                &["spatial-memory"],
            ),
            Self::Radial => (
                "Radial",
                "Items ring outward by a numeric index.",
                &["hierarchical"],
            ),
        };
        let mut capability = SolverCapability::new(self.id(), name);
        capability.description = Some(description.into());
        capability.requires = match self.channels() {
            ChannelUse::Embedding => vec![Disclosure::Embedding],
            ChannelUse::NumericAxis | ChannelUse::IntegerAxis | ChannelUse::CategoricalAxis => {
                vec![Disclosure::Axis]
            },
            ChannelUse::Cells | ChannelUse::Coordinate | ChannelUse::Order => Vec::new(),
        };
        capability.tags = tags.iter().map(|tag| (*tag).to_owned()).collect();
        capability.options = self.options();
        capability
    }

    /// Build the typed arrangement from authored options and measured items.
    pub fn arrangement(
        self,
        options: &BTreeMap<String, String>,
        measure: &Measure,
    ) -> Result<Arrangement, Vec<OptionIssue>> {
        let mut read = Options::new(options, self.options());
        let arrangement = self.read_arrangement(&mut read, measure);
        read.finish(self)?;
        Ok(arrangement)
    }

    /// What each option is when the author leaves it out, for these items:
    /// the declaration's measured defaults resolved to numbers (SE43).
    pub fn resolved_defaults(self, measure: &Measure) -> BTreeMap<String, String> {
        let empty = BTreeMap::new();
        let mut read = Options::new(&empty, self.options());
        let _ = self.read_arrangement(&mut read, measure);
        read.resolved
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect()
    }

    fn read_arrangement(self, read: &mut Options, measure: &Measure) -> Arrangement {
        match self {
            Self::Spiral => {
                let default = Spiral::default();
                Arrangement::Spiral(Spiral {
                    center: Vec2::ZERO,
                    spacing: measure.spacing,
                    angle_radians: read.finite("angle_radians", default.angle_radians),
                    curve: read.choice("curve", default.curve, SPIRAL_CURVES),
                })
            },
            Self::Grid => Arrangement::Grid(Grid {
                origin: Vec2::ZERO,
                cell: Vec2::new(
                    read.positive("cell_width", measure.largest.w),
                    read.positive("cell_height", measure.largest.h),
                ),
                columns: read.count("columns", measure.side() as u32),
                gap: measure.spacing,
            }),
            Self::Geographic => Arrangement::Geographic(Geographic {
                origin: Vec2::ZERO,
                units_per_coordinate: read.positive("units_per_coordinate", measure.spacing),
                invert_y: read.flag("invert_y", false),
            }),
            Self::Hulls => {
                let units = read.positive("units_per_coordinate", measure.spacing);
                let invert_y = read.flag("invert_y", false);
                Arrangement::Hulls(Hulls {
                    origin: Vec2::ZERO,
                    units_per_coordinate: units,
                    invert_y,
                    bounds: hull_bounds(measure, units, invert_y),
                })
            },
            Self::Stack => Arrangement::Stack(Stack {
                layer_gap: read.positive("layer_gap", measure.pitch_h()),
                row_gap: read.positive("row_gap", measure.pitch_w()),
                center: Vec2::ZERO,
            }),
            Self::Penrose => Arrangement::Penrose(Penrose {
                variant: read.choice("variant", PenroseVariant::default(), PENROSE_VARIANTS),
                subdivision_count: match read.depth("subdivisions") {
                    Some(depth) => SubdivisionCount::Explicit(depth),
                    None => SubdivisionCount::Auto,
                },
                unused_vertices: read.choice(
                    "unused_vertices",
                    UnusedVertexPolicy::default(),
                    PENROSE_UNUSED_VERTICES,
                ),
                center: Vec2::ZERO,
                // The tiling fills a disc: give each item a pitch-sized share of it.
                tile_scale: read.positive(
                    "tile_scale",
                    measure.pitch() * (measure.count.max(1) as f32 / std::f32::consts::PI).sqrt(),
                ),
            }),
            Self::LSystem => Arrangement::LSystem(LSystem {
                grammar: read.choice("grammar", LSystemGrammar::default(), LSYSTEM_GRAMMARS),
                iteration_depth: match read.depth("depth") {
                    Some(depth) => IterationDepth::Explicit(depth),
                    None => IterationDepth::Auto,
                },
                origin: Vec2::ZERO,
                // The path fills a square with a pitch between neighbors.
                size: read.positive("size", measure.side() * measure.pitch()),
                rotation: read.finite("rotation", 0.0),
                reverse_order: read.flag("reverse_order", false),
            }),
            Self::Timeline => Arrangement::Timeline(Timeline {
                origin: Vec2::ZERO,
                axis_length: read.positive(
                    "axis_length",
                    measure.count.max(1) as f32 * measure.pitch_w(),
                ),
                row_gap: read.positive("row_gap", measure.pitch_h()),
                fallback: read.choice("fallback", TimelineFallback::default(), TIMELINE_FALLBACKS),
            }),
            Self::Kanban => Arrangement::Kanban(Kanban {
                origin: Vec2::ZERO,
                column_gap: read.positive("column_gap", measure.pitch_w()),
                row_gap: read.positive("row_gap", measure.pitch_h()),
                column_order: read.list("column_order"),
                include_other_column: read.flag("include_other_column", true),
            }),
            Self::Embedded => Arrangement::Embedded(Embedded {
                origin: Vec2::ZERO,
                scale: read.positive("scale", measure.spacing),
                rotation: read.finite("rotation", 0.0),
                fallback: read.choice(
                    "fallback",
                    EmbeddingFallback::default(),
                    EMBEDDING_FALLBACKS,
                ),
            }),
            Self::Radial => Arrangement::Radial(Radial {
                center: Vec2::ZERO,
                ring_spacing: read.positive("ring_spacing", measure.pitch()),
                angular_policy: read.choice(
                    "angular_policy",
                    RadialAngularPolicy::default(),
                    RADIAL_ANGULAR_POLICIES,
                ),
                rotation_offset: read.finite("rotation_offset", 0.0),
                unreachable_policy: read.choice(
                    "unreachable_policy",
                    RadialUnreachablePolicy::default(),
                    RADIAL_UNREACHABLE_POLICIES,
                ),
            }),
        }
    }
}

const SPIRAL_CURVES: &[(&str, SpiralCurve)] = &[
    ("square_root", SpiralCurve::SquareRoot),
    ("linear", SpiralCurve::Linear),
    ("quadratic", SpiralCurve::Quadratic),
    ("logarithmic", SpiralCurve::Logarithmic),
];

const PENROSE_VARIANTS: &[(&str, PenroseVariant)] = &[
    ("rhombus", PenroseVariant::Rhombus),
    ("kite_dart", PenroseVariant::KiteDart),
];

const PENROSE_UNUSED_VERTICES: &[(&str, UnusedVertexPolicy)] = &[
    ("leave_empty", UnusedVertexPolicy::LeaveEmpty),
    ("clip_to_hull", UnusedVertexPolicy::ClipToHull),
    ("hide_tiling", UnusedVertexPolicy::HideTiling),
];

const LSYSTEM_GRAMMARS: &[(&str, LSystemGrammar)] = &[
    ("hilbert", LSystemGrammar::Hilbert),
    ("koch", LSystemGrammar::Koch),
    ("dragon", LSystemGrammar::Dragon),
];

const TIMELINE_FALLBACKS: &[(&str, TimelineFallback)] = &[
    ("leave_in_place", TimelineFallback::LeaveInPlace),
    ("stack_below_origin", TimelineFallback::StackBelowOrigin),
    ("stack_past_end", TimelineFallback::StackPastEnd),
];

const EMBEDDING_FALLBACKS: &[(&str, EmbeddingFallback)] = &[
    ("leave_in_place", EmbeddingFallback::LeaveInPlace),
    ("collapse_to_origin", EmbeddingFallback::CollapseToOrigin),
    ("ring_outside", EmbeddingFallback::RingOutside),
];

const RADIAL_ANGULAR_POLICIES: &[(&str, RadialAngularPolicy)] = &[
    ("uniform", RadialAngularPolicy::Uniform),
    ("weighted", RadialAngularPolicy::Weighted),
    ("hash_sorted", RadialAngularPolicy::HashSorted),
];

const RADIAL_UNREACHABLE_POLICIES: &[(&str, RadialUnreachablePolicy)] = &[
    ("outer_ring", RadialUnreachablePolicy::OuterRing),
    ("center", RadialUnreachablePolicy::Center),
    ("leave_in_place", RadialUnreachablePolicy::LeaveInPlace),
];

/// The region Hulls tiles: the disclosed coordinates in scene units, with room
/// for an item beyond the outermost ones.
fn hull_bounds(measure: &Measure, units: f32, invert_y: bool) -> Rect {
    let margin = measure.pitch();
    let Some((min, max)) = measure.coordinates else {
        return Rect::new(
            Vec2::new(-margin, -margin),
            Size2::new(margin * 2.0, margin * 2.0),
        );
    };
    let flip = if invert_y { -1.0 } else { 1.0 };
    let (y0, y1) = (min.y * units * flip, max.y * units * flip);
    let left = min.x * units - margin;
    let top = y0.min(y1) - margin;
    Rect::new(
        Vec2::new(left, top),
        Size2::new(
            (max.x - min.x) * units + margin * 2.0,
            (y1 - y0).abs() + margin * 2.0,
        ),
    )
}

/// Reads named options against the family's declaration, recording what it
/// refused, what it read, and each default it used.
struct Options<'a> {
    raw: &'a BTreeMap<String, String>,
    declared: Vec<OptionSpec>,
    read: Vec<&'static str>,
    /// The defaults used for options left out, in the option's own spelling.
    resolved: Vec<(&'static str, String)>,
    issues: Vec<OptionIssue>,
}

impl<'a> Options<'a> {
    fn new(raw: &'a BTreeMap<String, String>, declared: Vec<OptionSpec>) -> Self {
        Self {
            raw,
            declared,
            read: Vec::new(),
            resolved: Vec::new(),
            issues: Vec::new(),
        }
    }

    fn get(&mut self, key: &'static str) -> Option<&'a str> {
        debug_assert!(
            self.declared.iter().any(|spec| spec.key == key),
            "a family reads `{key}` without declaring it"
        );
        self.read.push(key);
        self.raw.get(key).map(String::as_str)
    }

    fn default_used(&mut self, key: &'static str, value: impl ToString) {
        if !self.raw.contains_key(key) {
            self.resolved.push((key, value.to_string()));
        }
    }

    fn refuse(&mut self, key: &str, message: &str) {
        self.issues.push(OptionIssue {
            field: format!("arrangement.options.{key}"),
            message: message.into(),
        });
    }

    fn finite(&mut self, key: &'static str, default: f32) -> f32 {
        self.default_used(key, default);
        match self.get(key).map(str::parse::<f32>) {
            None => default,
            Some(Ok(value)) if value.is_finite() => value,
            Some(_) => {
                self.refuse(key, "needs a finite number");
                default
            },
        }
    }

    fn positive(&mut self, key: &'static str, default: f32) -> f32 {
        self.default_used(key, default);
        match self.get(key).map(str::parse::<f32>) {
            None => default,
            Some(Ok(value)) if value.is_finite() && value > 0.0 => value,
            Some(_) => {
                self.refuse(key, "needs a positive finite number");
                default
            },
        }
    }

    fn count(&mut self, key: &'static str, default: u32) -> u32 {
        self.default_used(key, default.max(1));
        match self.get(key).map(str::parse::<u32>) {
            None => default.max(1),
            Some(Ok(value)) if value > 0 => value,
            Some(_) => {
                self.refuse(key, "needs a positive whole number");
                default.max(1)
            },
        }
    }

    fn depth(&mut self, key: &'static str) -> Option<u8> {
        self.default_used(key, "auto");
        match self.get(key).map(str::parse::<u8>) {
            None => None,
            Some(Ok(value)) => Some(value),
            Some(Err(_)) => {
                self.refuse(key, "needs a whole number from 0 to 255");
                None
            },
        }
    }

    fn flag(&mut self, key: &'static str, default: bool) -> bool {
        self.default_used(key, default);
        match self.get(key) {
            None => default,
            Some("true") => true,
            Some("false") => false,
            Some(_) => {
                self.refuse(key, "needs true or false");
                default
            },
        }
    }

    fn choice<T: Clone + PartialEq>(
        &mut self,
        key: &'static str,
        default: T,
        table: &[(&str, T)],
    ) -> T {
        if let Some((name, _)) = table.iter().find(|(_, value)| *value == default) {
            self.default_used(key, name);
        }
        let Some(value) = self.get(key) else {
            return default;
        };
        match table.iter().find(|(name, _)| *name == value) {
            Some((_, choice)) => choice.clone(),
            None => {
                let names: Vec<_> = table.iter().map(|(name, _)| *name).collect();
                self.refuse(key, &format!("needs one of {}", names.join(", ")));
                default
            },
        }
    }

    fn list(&mut self, key: &'static str) -> Vec<String> {
        self.default_used(key, "");
        self.get(key)
            .map(|value| {
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|item| !item.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn finish(mut self, family: Family) -> Result<(), Vec<OptionIssue>> {
        let undeclared: Vec<_> = self
            .raw
            .keys()
            .filter(|key| !self.declared.iter().any(|spec| &spec.key == *key))
            .cloned()
            .collect();
        for key in undeclared {
            self.refuse(&key, &format!("{} does not read this option", family.id()));
        }
        if self.issues.is_empty() {
            Ok(())
        } else {
            Err(self.issues)
        }
    }
}

#[cfg(test)]
mod option_tests {
    use super::*;

    fn measure() -> Measure {
        Measure {
            largest: Size2::new(164.0, 68.0),
            count: 9,
            spacing: 16.0,
            coordinates: Some((Vec2::ZERO, Vec2::new(10.0, 10.0))),
        }
    }

    /// One declaration for refusal and for an editor's rows only holds if each
    /// family reads exactly what it declares (ruling B).
    #[test]
    fn every_family_reads_exactly_what_it_declares() {
        for family in FAMILIES {
            let empty = BTreeMap::new();
            let mut read = Options::new(&empty, family.options());
            let _ = family.read_arrangement(&mut read, &measure());
            let mut reads: Vec<&str> = read.read.clone();
            reads.sort_unstable();
            reads.dedup();
            let mut declared: Vec<String> =
                family.options().into_iter().map(|spec| spec.key).collect();
            declared.sort_unstable();
            assert_eq!(reads, declared, "{}", family.id());
        }
    }

    #[test]
    fn every_option_left_out_resolves_to_a_default() {
        for family in FAMILIES {
            let resolved = family.resolved_defaults(&measure());
            for spec in family.options() {
                assert!(
                    resolved.contains_key(&spec.key),
                    "{}: {}",
                    family.id(),
                    spec.key
                );
                if let OptionDefault::Value(value) = &spec.default {
                    assert_eq!(&resolved[&spec.key], value, "{}: {}", family.id(), spec.key);
                }
            }
        }
        let grid = Family::Grid.resolved_defaults(&measure());
        assert_eq!(grid["cell_width"], "164");
        assert_eq!(grid["columns"], "3");
        assert_eq!(
            Family::Spiral.resolved_defaults(&measure())["curve"],
            "square_root"
        );
        assert_eq!(
            Family::LSystem.resolved_defaults(&measure())["depth"],
            "auto"
        );
    }

    #[test]
    fn an_undeclared_key_is_refused_in_the_familys_words() {
        let options = BTreeMap::from([("stride".to_string(), "2".to_string())]);
        let refused = Family::Grid.arrangement(&options, &measure()).unwrap_err();
        assert_eq!(
            refused,
            [OptionIssue {
                field: "arrangement.options.stride".into(),
                message: "grid does not read this option".into(),
            }]
        );
    }
}
