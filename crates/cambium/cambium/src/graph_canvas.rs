/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A bounded, interactive viewport over Sprigging's shared graph canvas.
//!
//! The graph remains one paint leaf. Each node also gets a small, absolutely
//! positioned native button in the view layer, projected through the same
//! [`GraphViewport`] as the leaf. Paint, hit targets, keyboard focus, and hover
//! therefore stay aligned without teaching a paint leaf about app actions.

use sceno::Vec2;
use sprigging::{
    ColorF, GraphAtlasCompoundPath, GraphAtlasPolygon, GraphCanvas, GraphGlyphNode,
    GraphGlyphRelation, GraphViewport, RelationLine, Size,
};

use crate::GraphCanvasAtlas;
#[cfg(test)]
use crate::GraphCanvasAtlasField;
use crate::component::{ComponentView, component};
use crate::{
    FocusEvent, FocusPhase, GenetCtx, GenetElement, HoverEvent, HoverPhase, OptionalAction,
    PointerClick, PointerEvent, PointerPhase, View, WheelEvent, custom_leaf, el, focusable,
    on_click, on_focus, on_hover, on_pointer, on_wheel, request_focus,
};

/// Structural classes emitted by [`graph_canvas_swatch`]. Hosts own the palette.
pub const GRAPH_CANVAS_SWATCH_CSS: &str = r#"
.graph-canvas-swatch {
    background-color: rgba(127, 127, 127, 0.06);
    border: 1px solid rgba(127, 127, 127, 0.28);
    border-radius: 7px;
}
.graph-canvas-swatch-node {
    background-color: transparent;
    border: 0;
    border-radius: 999px;
    cursor: pointer;
    padding: 0;
    touch-action: none;
    user-select: none;
}
.graph-canvas-swatch-node:focus-visible {
    outline: 1px solid currentColor;
    outline-offset: 1px;
}
.graph-canvas-swatch-relation {
    background: transparent;
    border: 0;
    cursor: pointer;
    padding: 0;
    touch-action: manipulation;
}
.graph-canvas-swatch-relation:focus-visible {
    outline: 1px solid currentColor;
    outline-offset: 1px;
}
.graph-canvas-swatch-relation.emphasized::after {
    background-color: currentColor;
    content: "";
    height: 3px;
    left: 0;
    opacity: 0.72;
    position: absolute;
    top: 50%;
    transform: translateY(-50%);
    width: 100%;
}
.graph-canvas-swatch-label {
    display: block;
    font-size: 10px;
    line-height: 1;
    white-space: nowrap;
    opacity: 0.85;
    pointer-events: none;
    z-index: 1;
}
.graph-canvas-swatch-labels,
.graph-canvas-swatch-relation-targets,
.graph-canvas-swatch-targets {
    pointer-events: none;
}
/* A relation cell spans its whole segment and its rotation makes it a
   stacking context, so it would win the press at both endpoint nodes. Node
   targets stack above the cells; a press on a node's centre is the node's. */
.graph-canvas-swatch-targets {
    z-index: 1;
}
.graph-canvas-swatch-relation,
.graph-canvas-swatch-node {
    pointer-events: auto;
}
.graph-canvas-swatch-label.selected {
    font-weight: bold;
    opacity: 1;
}
.graph-canvas-swatch-label.focused,
.graph-canvas-swatch-label.hovered {
    opacity: 1;
}
.graph-canvas-swatch-expand {
    background-color: rgba(127, 127, 127, 0.10);
    border: 0;
    border-radius: 4px;
    cursor: pointer;
    font-size: 10px;
    padding: 2px 5px;
}
"#;

/// One node in the app-facing subgraph contract.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphCanvasNode<Id, Kind> {
    pub id: Id,
    pub kind: Kind,
    /// Normalized `0..1` scene position.
    pub position: (f32, f32),
    /// Accessible name for the node's native hit target.
    pub label: String,
    /// An optional stable key emitted as `data-key` on the node's hit target,
    /// for targeting by identity when the `label` is not unique (two nodes may
    /// share a title). The component does not interpret it — a driver or test
    /// selects on it; a screen reader still reads `label`.
    pub key: Option<String>,
}

/// One captured node-motion event from a [`GraphCanvasSwatch`].
///
/// `position` is in the graph's normalized `0..=1` coordinate system, after
/// the Swatch has inverted its current viewport. A consumer owns the response:
/// it may write a local position override, hand the position to a solver, or
/// ignore the gesture. The component never edits graph truth.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphCanvasNodeDrag<Id> {
    pub id: Id,
    pub phase: PointerPhase,
    pub position: (f32, f32),
}

/// A leaf-local rectangular region anchored to one projected graph node.
///
/// Consumers use this for richer overlays whose contents remain app-owned: a
/// document preview, inspector, or editable card can occupy the node without
/// duplicating the canvas viewport math. The rectangle is clamped inside the
/// current canvas even when the node sits on an edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphCanvasNodeRegion {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

/// An app-owned rectangular overlay that occupies one graph node.
///
/// Cambium uses the requested size to derive the same clamped region exposed
/// by [`GraphCanvasSwatch::projected_node_footprint`], then terminates incident
/// relation routes at that region's perimeter. The overlay contents remain the
/// consumer's responsibility.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphCanvasNodeFootprint<Id> {
    pub id: Id,
    pub width: f32,
    pub height: f32,
}

/// One app-facing edge. Endpoints that are absent from the subgraph are skipped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphCanvasEdge<Id> {
    pub from: Id,
    pub to: Id,
}

/// One routed connection in a graph-canvas Swatch. A source link carries its
/// canonical key; choosing that stroke opens the link before a member is picked.
///
/// A disclosed `link_key` groups parallel reasons into one stroke for that
/// stratum and appearance. Anonymous non-link geometry retains its caller ID.
/// Visibility, marks and emphasis are projection state and never change truth.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphCanvasRelation<Id> {
    pub id: String,
    /// Present for source links; absent for geometry without a disclosed link.
    pub link_key: Option<sceno::LinkKey>,
    pub marks: sceno::LinkMarks,
    /// View-local additive contribution; paint bounds its visual width.
    pub strength: f32,
    pub from: Id,
    pub to: Id,
    pub kind: String,
    pub label: String,
    /// Authored normalized route. Three or more points preserve an explicit
    /// polyline; member multiplicity never creates additional lanes.
    pub route: Vec<(f32, f32)>,
    pub visible: bool,
    pub emphasized: bool,
}

/// The bounded graph data, independent of view state and palette.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphCanvasSubgraph<Id, Kind> {
    pub nodes: Vec<GraphCanvasNode<Id, Kind>>,
    pub edges: Vec<GraphCanvasEdge<Id>>,
}

/// A card-sized graph viewport. The consumer stores this beside app state,
/// rebuilds its leaf with [`GraphCanvasSwatch::paint_leaf`], and renders the
/// matching view with [`graph_canvas_swatch`].
#[derive(Clone, Debug, PartialEq)]
pub struct GraphCanvasSwatch<Id, Kind> {
    pub leaf_key: u64,
    pub graph: GraphCanvasSubgraph<Id, Kind>,
    /// Relation cells rendered in preference to the legacy endpoint-only
    /// [`GraphCanvasSubgraph::edges`] when nonempty.
    pub relations: Vec<GraphCanvasRelation<Id>>,
    /// The line each relation kind is painted with; see
    /// [`with_relation_lines`](Self::with_relation_lines).
    pub relation_lines: Vec<(String, RelationLine)>,
    /// App-owned rectangular overlays currently occupying graph nodes.
    pub node_footprints: Vec<GraphCanvasNodeFootprint<Id>>,
    /// Extra attributes on individual node targets, as `(id, name, value)`:
    /// what a consumer's node kind says that the shared target cannot, such
    /// as `aria-expanded` on a node that opens a card or `aria-haspopup` on
    /// one that opens a popover. See [`with_node_attr`](Self::with_node_attr).
    pub node_attrs: Vec<(Id, String, String)>,
    /// The node whose target takes keyboard focus when this request first
    /// appears, such as the trigger a dismissed popover or a collapsed card
    /// returns focus to. Holding the same request does not steal focus on
    /// later rebuilds.
    pub focus_request: Option<Id>,
    /// Shorter visible text for individual nodes' labels, as `(id, text)`.
    /// A node's [`GraphCanvasNode::label`] stays its accessible name; the
    /// visible label can say less, such as a name without the state words the
    /// accessible name carries. See [`with_visible_label`](Self::with_visible_label).
    pub visible_labels: Vec<(Id, String)>,
    /// Where individual nodes' visible labels go, overriding the canvas's own
    /// choice: for a node beside an app-owned control the label must not run
    /// under. See [`with_label_placement`](Self::with_label_placement).
    pub label_placements: Vec<(Id, GraphLabelPlacement)>,
    /// Optional fixed-world atlas beneath the graph labels and callouts.
    /// Geographic field identity refers to existing graph node ids, while the
    /// atlas's paint and picking geometry remain caller-produced projection data.
    pub atlas: Option<GraphCanvasAtlas<Id>>,
    pub selected: Option<Id>,
    pub focus: Option<Id>,
    pub hovered: Option<Id>,
    pub viewport: GraphViewport,
    pub width: u32,
    pub height: u32,
    pub node_radius: f32,
    pub edge_width: f32,
    pub hit_size: f32,
    pub label: String,
    /// Whether the Expand affordance renders. Defaults on (the original
    /// behavior); a consumer whose swatch has no "fuller view" to expand into
    /// switches it off rather than wiring a lying no-op chip.
    pub show_expand: bool,
    /// Whether each node's `label` also renders as visible text beside its hit
    /// target (it is always the accessible name). Defaults off — a dense
    /// minimap reads better bare; an overview where identity is the point
    /// (sessions, clusters) switches it on.
    ///
    /// Visible labels carry the same `selected` / `focused` / `hovered`
    /// modifiers as the node buttons, so a consumer can emphasize the label of
    /// the node that matters without hand-rolling a parallel label layer.
    pub show_labels: bool,
    /// With labels shown, leave out a label whose box would print over one
    /// already shown: selected, focused, then hovered nodes get first choice,
    /// then the rest in node order. Alternate sides are tried before culling.
    /// A node without a shown label keeps its accessible name. Defaults on.
    pub cull_labels: bool,
    /// Let captured `Move` events update the model used by the custom leaf
    /// without rebuilding the retained DOM until release. Defaults off because
    /// the consumer must refresh that leaf independently each presented frame.
    pub defer_drag_rebuild: bool,
}

/// The width a node label is laid out at, before it is clamped to the room its
/// side leaves.
const GRAPH_LABEL_WIDTH: f32 = 160.0;

/// A label character's estimated width at the labels' 10 px size.
const LABEL_CHAR_WIDTH: f32 = 5.6;

/// The most characters a visible label shows before it is cut: as many as its
/// width holds at the 4.6 px a label character measures natively (the width
/// estimate above stays conservative, for culling).
const GRAPH_LABEL_CHARS: usize = ((GRAPH_LABEL_WIDTH - 2.0) / 4.6) as usize;

/// Where a node's visible label sits against its node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphLabelPlacement {
    Above,
    Below,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug)]
struct LabelBox {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

#[derive(Clone, Copy, Debug)]
struct ResolvedGraphLabel {
    placement: GraphLabelPlacement,
    bounds: LabelBox,
}

/// What a [`graph_canvas`] reports to its parent. Hover, focus, and relation
/// emphasis are deliberately absent: they are presentation state the component
/// owns, not events an application needs.
#[derive(Clone, Debug, PartialEq)]
pub enum GraphCanvasEvent<Id> {
    /// A node's hit target was activated.
    Activate(Id),
    /// The Expand affordance was activated.
    Expand,
    /// A node was dragged. `position` is in normalized graph coordinates.
    Drag(GraphCanvasNodeDrag<Id>),
    /// A relation cell was activated.
    RelationActivate(String),
    /// The canvas background was dragged. `delta` is in viewport pan units —
    /// the fraction of the canvas the pointer moved on each axis — so a
    /// consumer applies it to [`GraphViewport::pan`] directly. The component
    /// does not mutate the viewport itself: the viewport stays consumer-owned
    /// state, exactly like selection.
    Pan { delta: (f32, f32) },
    /// The wheel turned over the canvas. Multiply the consumer's zoom by
    /// `factor` (greater than one zooms in).
    Zoom { factor: f32 },
}

impl<Id> crate::Action for GraphCanvasEvent<Id> {}

/// What [`graph_canvas_with_wheel`] reports: the established
/// [`GraphCanvasEvent`]s, with the wheel passed through whole in place of
/// [`GraphCanvasEvent::Zoom`]. A separate type, as [`GraphAtlasEvent`] is, so a
/// consumer that matches [`GraphCanvasEvent`] exhaustively keeps compiling.
#[derive(Clone, Debug)]
pub enum GraphCanvasInput<Id> {
    /// Any event [`graph_canvas`] reports other than a wheel notch.
    Event(GraphCanvasEvent<Id>),
    /// A wheel notch over the canvas, untouched: both axes of `delta`, the
    /// pointer's canvas-local position (`local`, in the canvas's `size`) and the
    /// modifiers. The host's own scroll default is not suppressed; the consumer
    /// calls [`WheelEvent::prevent_default`] when its policy takes the notch,
    /// and leaves it alone when the page should scroll.
    Wheel(WheelEvent),
}

impl<Id> crate::Action for GraphCanvasInput<Id> {}

/// Atlas-only interaction events. Kept separate from [`GraphCanvasEvent`] so
/// the established generic graph callback remains source-compatible.
#[derive(Clone, Debug, PartialEq)]
pub enum GraphAtlasEvent<Id> {
    Activate(Id),
    Drag(GraphCanvasNodeDrag<Id>),
    Hover(Option<Id>),
    Focus(Option<Id>),
}

impl<Id> crate::Action for GraphAtlasEvent<Id> {}

/// Component-owned interaction state for [`graph_canvas`].
struct GraphCanvasLocal<Id> {
    /// Background-drag anchor for pointer panning: the last pointer position,
    /// leaf-local device px, while a background drag is held.
    pan_anchor: Option<(f32, f32)>,
    hovered: Option<Id>,
    focus: Option<Id>,
    hovered_relation: Option<String>,
}

struct AtlasCanvasLocal<Id> {
    captured: Option<Id>,
    down: Option<(f32, f32)>,
}

#[cfg(test)]
#[path = "graph_canvas/tests.rs"]
mod tests;

#[path = "graph_canvas/builder.rs"]
mod builder;

#[path = "graph_canvas/painting.rs"]
mod painting;

#[path = "graph_canvas/projection.rs"]
mod projection;

#[path = "graph_canvas/node_view.rs"]
mod node_view;

pub use node_view::graph_canvas_swatch_with_focus_and_drag_and_relations;

#[path = "graph_canvas/canvas_view.rs"]
mod canvas_view;

pub use canvas_view::{graph_atlas_swatch, graph_canvas, graph_canvas_with_wheel};

#[path = "graph_canvas/geometry.rs"]
mod geometry;

use geometry::{
    graph_position_at, graph_position_at_unclamped, label_text_width, rectangle_ray_exit,
    route_midpoint_segment,
};

pub(crate) use geometry::middle_cut;

#[path = "graph_canvas/label_layout.rs"]
mod label_layout;

#[path = "graph_canvas/swatch_view.rs"]
mod swatch_view;

pub use swatch_view::{
    graph_canvas_swatch, graph_canvas_swatch_with_drag,
    graph_canvas_swatch_with_drag_and_relations, graph_canvas_swatch_with_focus,
    graph_canvas_swatch_with_focus_and_drag,
};
