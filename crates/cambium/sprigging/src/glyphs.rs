// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! First catalog leaves (tier-2 Path-A glyphs from the widget catalog,
//! `design_docs/archive_docs/2026-10-28/2026-07-08_chisel_widget_catalog.md`): [`GraphCanvas`], [`Meter`],
//! [`Knob`]. Pure geometry: data in, `PaintCmd`s out. Labels/values belong in
//! DOM siblings, interaction in the view layer over the leaf.

use paint_list_api::ColorF;
use paint_list_api::items::DashPattern;

use crate::path::Path;
use crate::{Leaf, PaintCx, Size, SizeHint, round_stroke};

/// One node of a [`GraphCanvas`], in normalized `0..1` coordinates (scaled to
/// the leaf's box at paint time).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphGlyphNode {
    pub x: f32,
    pub y: f32,
    pub color: ColorF,
}

/// One independently painted relation route in normalized graph coordinates.
///
/// The route is already resolved by the caller. Sprigging owns projection and
/// paint only. Resting links and authored multi-segment routes travel
/// through the same leaf path; member count does not create lanes.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphGlyphRelation {
    pub points: Vec<(f32, f32)>,
    pub emphasized: bool,
    pub line: RelationLine,
    pub start_arrow: bool,
    pub end_arrow: bool,
    pub opposed: bool,
    pub strength: f32,
}

fn relation_mark_paths(
    points: &[(f32, f32)],
    start: bool,
    end: bool,
    opposed: bool,
) -> Vec<Vec<(f32, f32)>> {
    let segments = points
        .windows(2)
        .filter_map(|p| {
            let dx = p[1].0 - p[0].0;
            let dy = p[1].1 - p[0].1;
            let length = dx.hypot(dy);
            (length.is_finite() && length > 0.001).then_some((
                p[0],
                dx / length,
                dy / length,
                length,
            ))
        })
        .collect::<Vec<_>>();
    let total = segments.iter().map(|s| s.3).sum::<f32>();
    if total <= 0.001 {
        return Vec::new();
    }
    let sample = |fraction: f32| {
        let mut remaining = total * fraction;
        for &(p, dx, dy, length) in &segments {
            if remaining <= length {
                return ((p.0 + dx * remaining, p.1 + dy * remaining), (dx, dy));
            }
            remaining -= length;
        }
        (*points.last().unwrap(), (1.0, 0.0))
    };
    let mut marks = Vec::new();
    let size = (total * 0.12).clamp(2.0, 8.0);
    for (enabled, fraction, direction) in [(start, 0.22, -1.0), (end, 0.78, 1.0)] {
        if !enabled {
            continue;
        }
        let (tip, (dx, dy)) = sample(fraction);
        let back = (tip.0 - dx * size * direction, tip.1 - dy * size * direction);
        marks.push(vec![
            (back.0 - dy * size * 0.55, back.1 + dx * size * 0.55),
            tip,
            (back.0 + dy * size * 0.55, back.1 - dx * size * 0.55),
        ]);
    }
    if opposed {
        let (center, (dx, dy)) = sample(0.5);
        for sign in [-1.0, 1.0] {
            let v = (
                (dx - dy * sign) * size * 0.55,
                (dy + dx * sign) * size * 0.55,
            );
            marks.push(vec![
                (center.0 - v.0, center.1 - v.1),
                (center.0 + v.0, center.1 + v.1),
            ]);
        }
    }
    marks
}

/// How a relation's line is drawn, so kinds of relation differ by more than
/// colour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RelationLine {
    #[default]
    Solid,
    /// Twice the edge width.
    Heavy,
    Dashed,
    Dotted,
}

/// A fixed leaf-local atlas polygon painted beneath graph routes and nodes.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphAtlasPolygon {
    pub points: Vec<(f32, f32)>,
    pub fill: ColorF,
    pub stroke: Option<ColorF>,
    pub stroke_width: f32,
}

/// One filled path with all of its contours retained together. Opposite-wound
/// inner contours therefore remain counters under the paint list's non-zero
/// fill rule (for example the hole in `O` or `a`).
#[derive(Clone, Debug, PartialEq)]
pub struct GraphAtlasCompoundPath {
    pub contours: Vec<Vec<(f32, f32)>>,
    pub fill: ColorF,
    pub stroke: Option<ColorF>,
    pub stroke_width: f32,
}

/// Pane-local camera for a [`GraphCanvas`]. Pan is expressed in normalized
/// scene coordinates; zoom is centred on `(0.5, 0.5)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphViewport {
    pub pan: (f32, f32),
    pub zoom: f32,
}

impl Default for GraphViewport {
    fn default() -> Self {
        Self {
            pan: (0.0, 0.0),
            zoom: 1.0,
        }
    }
}

impl GraphViewport {
    /// Project one normalized scene point into a leaf-local box. View-layer hit
    /// targets use this same function, keeping paint and interaction aligned.
    pub fn project(self, point: (f32, f32), size: Size, inset: f32) -> (f32, f32) {
        let zoom = self.zoom.max(0.01);
        let x = (point.0 - 0.5) * zoom + 0.5 + self.pan.0;
        let y = (point.1 - 0.5) * zoom + 0.5 + self.pan.1;
        let width = (size.width - 2.0 * inset).max(0.0);
        let height = (size.height - 2.0 * inset).max(0.0);
        (inset + x * width, inset + y * height)
    }
}

/// A node-link canvas shared by full panes and bounded glyphs such as link
/// previews, breadcrumb thumbnails, and hover cards. Layout is caller-computed;
/// normalized node coordinates are projected through a pane-local viewport.
pub struct GraphCanvas {
    nodes: Vec<GraphGlyphNode>,
    /// Index pairs into `nodes`; out-of-range pairs are skipped.
    edges: Vec<(u16, u16)>,
    /// Routed relations. Kept beside legacy endpoint edges so existing glyph
    /// callers do not need to manufacture routes.
    relations: Vec<GraphGlyphRelation>,
    atlas_polygons: Vec<GraphAtlasPolygon>,
    atlas_paths: Vec<GraphAtlasCompoundPath>,
    pub node_radius: f32,
    pub edge_width: f32,
    pub edge_color: ColorF,
    /// Ring around the selected node.
    pub selection_color: ColorF,
    /// Outer ring around the keyboard-focused node.
    pub focus_color: ColorF,
    viewport: GraphViewport,
    selected: Option<u16>,
    focus: Option<u16>,
    hovered: Option<u16>,
    intrinsic: Size,
    dirty: bool,
}

impl GraphCanvas {
    pub fn new(nodes: Vec<GraphGlyphNode>, edges: Vec<(u16, u16)>, intrinsic: Size) -> Self {
        Self {
            nodes,
            edges,
            relations: Vec::new(),
            atlas_polygons: Vec::new(),
            atlas_paths: Vec::new(),
            node_radius: 2.5,
            edge_width: 1.0,
            edge_color: ColorF {
                r: 0.5,
                g: 0.5,
                b: 0.55,
                a: 1.0,
            },
            selection_color: ColorF {
                r: 0.95,
                g: 0.72,
                b: 0.25,
                a: 1.0,
            },
            focus_color: ColorF {
                r: 0.30,
                g: 0.58,
                b: 0.95,
                a: 1.0,
            },
            viewport: GraphViewport::default(),
            selected: None,
            focus: None,
            hovered: None,
            intrinsic,
            dirty: true,
        }
    }

    /// Replace the graph data (layout recompute stays with the caller).
    pub fn set_graph(&mut self, nodes: Vec<GraphGlyphNode>, edges: Vec<(u16, u16)>) {
        self.nodes = nodes;
        self.edges = edges;
        self.dirty = true;
    }

    /// Replace independently routed relations. Points are normalized and pass
    /// through the same viewport as nodes and endpoint-only edges.
    pub fn set_relations(&mut self, relations: Vec<GraphGlyphRelation>) {
        if self.relations != relations {
            self.relations = relations;
            self.dirty = true;
        }
    }

    /// Replace caller-projected fixed-world terrain and field paint.
    pub fn set_atlas_polygons(&mut self, polygons: Vec<GraphAtlasPolygon>) {
        if self.atlas_polygons != polygons {
            self.atlas_polygons = polygons;
            self.dirty = true;
        }
    }

    pub fn set_atlas_paths(&mut self, paths: Vec<GraphAtlasCompoundPath>) {
        if self.atlas_paths != paths {
            self.atlas_paths = paths;
            self.dirty = true;
        }
    }

    /// Change the pane-local camera. A bounded swatch and a full canvas use the
    /// same projection; only their viewport and measured box differ.
    pub fn set_viewport(&mut self, viewport: GraphViewport) {
        if self.viewport != viewport {
            self.viewport = viewport;
            self.dirty = true;
        }
    }

    pub fn viewport(&self) -> GraphViewport {
        self.viewport
    }

    /// Set transient and durable emphasis by node index.
    pub fn set_emphasis(
        &mut self,
        selected: Option<u16>,
        focus: Option<u16>,
        hovered: Option<u16>,
    ) {
        if (self.selected, self.focus, self.hovered) != (selected, focus, hovered) {
            self.selected = selected;
            self.focus = focus;
            self.hovered = hovered;
            self.dirty = true;
        }
    }

    /// Project a node through the exact camera and inset used by [`Leaf::paint`].
    pub fn node_local_position(&self, index: usize, size: Size) -> Option<(f32, f32)> {
        let node = self.nodes.get(index)?;
        Some(
            self.viewport
                .project((node.x, node.y), size, self.node_radius + self.edge_width),
        )
    }
}

/// Compatibility name for the original miniature-only graph leaf. `GraphGlyph`
/// and `GraphCanvas` are the same renderer; the latter names its use for both
/// bounded swatches and full canvas panes.
pub type GraphGlyph = GraphCanvas;

/// A level meter bar: track + proportional fill + optional peak tick.
/// Vertical fills bottom-up, horizontal left-to-right. Value/peak are `0..=1`.
pub struct Meter {
    value: f32,
    peak: Option<f32>,
    pub vertical: bool,
    pub track_color: ColorF,
    pub fill_color: ColorF,
    pub peak_color: ColorF,
    /// Peak tick thickness, device px.
    pub peak_thickness: f32,
    intrinsic: Size,
    dirty: bool,
}

impl Meter {
    pub fn new(vertical: bool, intrinsic: Size) -> Self {
        Self {
            value: 0.0,
            peak: None,
            vertical,
            track_color: ColorF {
                r: 0.12,
                g: 0.12,
                b: 0.14,
                a: 1.0,
            },
            fill_color: ColorF {
                r: 0.30,
                g: 0.80,
                b: 0.35,
                a: 1.0,
            },
            peak_color: ColorF {
                r: 0.95,
                g: 0.85,
                b: 0.25,
                a: 1.0,
            },
            peak_thickness: 2.0,
            intrinsic,
            dirty: true,
        }
    }

    /// Set level + peak, clamped to `0..=1`; dirties only on change.
    pub fn set_level(&mut self, value: f32, peak: Option<f32>) {
        let value = value.clamp(0.0, 1.0);
        let peak = peak.map(|p| p.clamp(0.0, 1.0));
        if value != self.value || peak != self.peak {
            self.value = value;
            self.peak = peak;
            self.dirty = true;
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

impl Leaf for Meter {
    /// A level meter announces as [`accesskit::Role::Meter`] carrying its normalized level.
    /// Read-only: a meter reports, it is not actuated, so it declares no action
    /// and never lands in the host's routable set.
    fn accessibility(&self, node: &mut accesskit::Node) {
        node.set_role(accesskit::Role::Meter);
        node.set_numeric_value(self.value as f64);
        node.set_min_numeric_value(0.0);
        node.set_max_numeric_value(1.0);
    }

    fn measure(&mut self, _known: SizeHint, _available: SizeHint) -> Size {
        self.intrinsic
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        let s = cx.size();
        cx.fill_rect(0.0, 0.0, s.width, s.height, self.track_color);
        if self.value > 0.0 {
            if self.vertical {
                let fh = s.height * self.value;
                cx.fill_rect(0.0, s.height - fh, s.width, fh, self.fill_color);
            } else {
                cx.fill_rect(0.0, 0.0, s.width * self.value, s.height, self.fill_color);
            }
        }
        if let Some(p) = self.peak {
            let t = self.peak_thickness;
            if self.vertical {
                let y = ((1.0 - p) * s.height).min(s.height - t);
                cx.fill_rect(0.0, y, s.width, t, self.peak_color);
            } else {
                let x = (p * s.width - t).max(0.0);
                cx.fill_rect(x, 0.0, t, s.height, self.peak_color);
            }
        }
        self.dirty = false;
    }

    fn paint_dirty(&self) -> bool {
        self.dirty
    }
}

/// Sweep geometry shared by the knob arcs: 270 degrees from lower-left
/// (135 deg, screen convention y-down) clockwise through up to lower-right.
const KNOB_START: f32 = 135.0 * std::f32::consts::PI / 180.0;
const KNOB_SWEEP: f32 = 270.0 * std::f32::consts::PI / 180.0;

/// A rotary knob: track arc + value arc + needle. Value is `0..=1`. Pointer
/// interaction belongs to the wrapping view (`on_pointer` maps drag delta to
/// `set_value`); the leaf only paints.
pub struct Knob {
    value: f32,
    pub track_color: ColorF,
    pub value_color: ColorF,
    pub needle_color: ColorF,
    pub arc_width: f32,
    intrinsic: Size,
    dirty: bool,
}

impl Knob {
    pub fn new(intrinsic: Size) -> Self {
        Self {
            value: 0.0,
            track_color: ColorF {
                r: 0.20,
                g: 0.20,
                b: 0.24,
                a: 1.0,
            },
            value_color: ColorF {
                r: 0.40,
                g: 0.60,
                b: 0.95,
                a: 1.0,
            },
            needle_color: ColorF {
                r: 0.90,
                g: 0.90,
                b: 0.92,
                a: 1.0,
            },
            arc_width: 3.0,
            intrinsic,
            dirty: true,
        }
    }

    /// Set the value, clamped to `0..=1`; dirties only on change.
    pub fn set_value(&mut self, value: f32) {
        let value = value.clamp(0.0, 1.0);
        if value != self.value {
            self.value = value;
            self.dirty = true;
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

impl Leaf for Knob {
    /// A knob announces as [`accesskit::Role::Slider`] carrying its normalized value, and
    /// declares the actions assistive tech and automation invoke on a slider.
    /// Declaring them here is what puts the knob in the host's routable set, so
    /// a leaf interior is actuated through the same path a `<button>` is.
    fn accessibility(&self, node: &mut accesskit::Node) {
        node.set_role(accesskit::Role::Slider);
        node.set_numeric_value(self.value as f64);
        node.set_min_numeric_value(0.0);
        node.set_max_numeric_value(1.0);
        node.add_action(accesskit::Action::SetValue);
        node.add_action(accesskit::Action::Increment);
        node.add_action(accesskit::Action::Decrement);
    }

    fn measure(&mut self, _known: SizeHint, _available: SizeHint) -> Size {
        self.intrinsic
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        let s = cx.size();
        let (cx0, cy0) = (s.width * 0.5, s.height * 0.5);
        let r = (s.width.min(s.height) * 0.5 - self.arc_width).max(1.0);
        // Track: the full sweep.
        cx.stroke_path(
            Path::new()
                .arc(cx0, cy0, r, KNOB_START, KNOB_START + KNOB_SWEEP)
                .build(),
            round_stroke(self.track_color, self.arc_width),
        );
        // Value arc over it.
        let a = KNOB_START + KNOB_SWEEP * self.value;
        if self.value > 0.0 {
            cx.stroke_path(
                Path::new().arc(cx0, cy0, r, KNOB_START, a).build(),
                round_stroke(self.value_color, self.arc_width),
            );
        }
        // Needle from 30% radius out to the arc, at the value angle.
        cx.stroke_path(
            Path::polyline(&[
                (cx0 + 0.3 * r * a.cos(), cy0 + 0.3 * r * a.sin()),
                (cx0 + r * a.cos(), cy0 + r * a.sin()),
            ]),
            round_stroke(self.needle_color, self.arc_width * 0.66),
        );
        self.dirty = false;
    }

    fn paint_dirty(&self) -> bool {
        self.dirty
    }
}

#[cfg(test)]
#[path = "glyphs/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "glyphs/link_mark_tests.rs"]
mod link_mark_tests;

#[path = "glyphs/graph_paint.rs"]
mod graph_paint;
