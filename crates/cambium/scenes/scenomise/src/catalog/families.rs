// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

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
            Self::Partition => "partition",
            Self::Sets => "sets",
            Self::Mosaic => "mosaic",
        }
    }

    /// Older ids that saved recipes carry.
    pub fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::Grid => &[GRID_ARRANGEMENT_ID, "grid.ordered"],
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
            Self::Kanban | Self::Partition | Self::Sets | Self::Mosaic => {
                ChannelUse::CategoricalAxis
            },
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
            Self::Partition => vec![
                measured("width", "Width", OptionKind::Positive, AREA),
                measured("height", "Height", OptionKind::Positive, AREA),
                finite("whole", "Measured whole (0: the items' total)", 0.0),
                choice("form", "Form", PARTITION_FORMS),
                finite("gap", "Gap", Partition::default().gap),
            ],
            Self::Sets => vec![
                measured("width", "Width", OptionKind::Positive, AREA),
                measured("height", "Height", OptionKind::Positive, AREA),
                OptionSpec::new("sets", "Sets", OptionKind::List, OptionDefault::Empty),
            ],
            Self::Mosaic => vec![
                measured(
                    "unit_area",
                    "Area per unit of weight",
                    OptionKind::Positive,
                    "the largest item's pitch, squared",
                ),
                OptionSpec::new(
                    "aspect",
                    "Aspect",
                    OptionKind::Positive,
                    OptionDefault::Value(Mosaic::default().aspect.to_string()),
                ),
                finite("gap", "Gap", Mosaic::default().gap),
                finite("header", "Group label band", Mosaic::default().header),
                OptionSpec::new(
                    "group_order",
                    "Group order",
                    OptionKind::List,
                    OptionDefault::Empty,
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
            Self::Partition => (
                "Partition",
                "Items share a measured whole, by area or by angle.",
                &["hierarchical"],
            ),
            Self::Sets => (
                "Sets",
                "Items sit in the overlap of the sets they belong to.",
                &["spatial-memory"],
            ),
            Self::Mosaic => (
                "Mosaic",
                "Items tile by weight in groups, and never reflow.",
                &["spatial-memory", "hierarchical"],
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

    pub(super) fn read_arrangement(self, read: &mut Options, measure: &Measure) -> Arrangement {
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
            Self::Partition => Arrangement::Partition(Partition {
                bounds: area(read, measure),
                whole: Some(read.finite("whole", 0.0)).filter(|whole| *whole > 0.0),
                form: read.choice("form", PartitionForm::default(), PARTITION_FORMS),
                gap: read.finite("gap", Partition::default().gap).max(0.0),
            }),
            Self::Sets => Arrangement::Setscape(Setscape {
                bounds: area(read, measure),
                sets: read.list("sets"),
            }),
            Self::Mosaic => {
                let default = Mosaic::default();
                Arrangement::Mosaic(Mosaic {
                    origin: Vec2::ZERO,
                    unit_area: read.positive("unit_area", measure.pitch() * measure.pitch()),
                    aspect: read.positive("aspect", default.aspect),
                    gap: read.finite("gap", default.gap).max(0.0),
                    header: read.finite("header", default.header).max(0.0),
                    group_order: read.list("group_order"),
                })
            },
        }
    }
}
