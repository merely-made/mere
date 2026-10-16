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
    /// already shown: the selected, focused and hovered nodes' first and always,
    /// then the rest in node order. A node without a shown label keeps its
    /// accessible name. Defaults off, which keeps every label.
    pub cull_labels: bool,
    /// Let captured `Move` events update the model used by the custom leaf
    /// without rebuilding the retained DOM until release. Defaults off because
    /// the consumer must refresh that leaf independently each presented frame.
    pub defer_drag_rebuild: bool,
}

impl<Id, Kind> GraphCanvasSwatch<Id, Kind> {
    /// Build a quiet, card-sized viewport. Dimensions remain configurable so a
    /// host can match its panel density without forking the component.
    pub fn new(leaf_key: u64, graph: GraphCanvasSubgraph<Id, Kind>) -> Self {
        Self {
            leaf_key,
            graph,
            relations: Vec::new(),
            relation_lines: Vec::new(),
            node_footprints: Vec::new(),
            node_attrs: Vec::new(),
            focus_request: None,
            visible_labels: Vec::new(),
            label_placements: Vec::new(),
            atlas: None,
            selected: None,
            focus: None,
            hovered: None,
            viewport: GraphViewport::default(),
            width: 260,
            height: 128,
            node_radius: 5.0,
            edge_width: 1.0,
            // Keep the painted dot compact while leaving a touch-sized native
            // target around it. Consumers can opt into a larger visual radius
            // without sacrificing a dependable base interaction area.
            hit_size: 44.0,
            label: "Related graph".to_string(),
            show_expand: true,
            show_labels: false,
            cull_labels: false,
            defer_drag_rebuild: false,
        }
    }

    #[must_use]
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Show or hide the Expand affordance (see [`Self::show_expand`]).
    #[must_use]
    pub fn with_expand(mut self, on: bool) -> Self {
        self.show_expand = on;
        self
    }

    /// Render each node's label as visible text (see [`Self::show_labels`]).
    #[must_use]
    pub fn with_node_labels(mut self, on: bool) -> Self {
        self.show_labels = on;
        self
    }

    /// Leave out a visible label that would print over one already shown
    /// (see [`Self::cull_labels`]).
    #[must_use]
    pub fn with_label_culling(mut self, on: bool) -> Self {
        self.cull_labels = on;
        self
    }

    /// Paint live drag positions through the custom leaf and settle native hit
    /// targets on release instead of rebuilding them for every sampled move.
    #[must_use]
    pub fn with_deferred_drag_rebuild(mut self, on: bool) -> Self {
        self.defer_drag_rebuild = on;
        self
    }

    /// Attach caller-owned fixed-world paint, field and route geometry.
    /// The atlas is cloned with the swatch, so no scene borrow outlives the
    /// source projection that produced it.
    #[must_use]
    pub fn with_atlas(mut self, atlas: GraphCanvasAtlas<Id>) -> Self {
        self.atlas = Some(atlas);
        self
    }

    /// Supply identity-bearing relation cells. When present, these replace the
    /// legacy endpoint-only edges in the retained paint projection.
    #[must_use]
    pub fn with_relations(mut self, relations: Vec<GraphCanvasRelation<Id>>) -> Self
    where
        Id: PartialEq,
    {
        self.relations.clear();
        for mut relation in relations {
            if let Some(key) = &relation.link_key {
                relation.id = key.stable_id();
                if relation.visible {
                    if let Some(peer) = self.relations.iter_mut().find(|peer| {
                        peer.visible
                            && peer.link_key.as_ref() == Some(key)
                            && ((peer.from == relation.from && peer.to == relation.to)
                                || (peer.from == relation.to && peer.to == relation.from))
                    }) {
                        let marks = if peer.from == relation.from {
                            relation.marks
                        } else {
                            relation.marks.reversed()
                        };
                        peer.marks.merge(marks);
                        peer.strength += relation.strength.max(0.0);
                        peer.emphasized |= relation.emphasized;
                        continue;
                    }
                }
            }
            self.relations.push(relation);
        }
        self
    }

    /// Paint relations of each named kind with their own line, so kinds differ
    /// by more than colour. A kind not listed is painted solid.
    #[must_use]
    pub fn with_relation_lines(mut self, lines: Vec<(String, RelationLine)>) -> Self {
        self.relation_lines = lines;
        self
    }

    fn relation_line(&self, kind: &str) -> RelationLine {
        self.relation_lines
            .iter()
            .find(|(named, _)| named == kind)
            .map_or(RelationLine::Solid, |(_, line)| *line)
    }
}

impl<Id: PartialEq, Kind> GraphCanvasSwatch<Id, Kind> {
    /// Declare that an app-owned rectangular overlay occupies this node.
    ///
    /// Repeating an id replaces its earlier footprint, which makes the builder
    /// safe to apply after consumer-specific view state has been reconciled.
    #[must_use]
    pub fn with_node_footprint(mut self, id: Id, width: f32, height: f32) -> Self {
        self.node_footprints.retain(|footprint| footprint.id != id);
        self.node_footprints
            .push(GraphCanvasNodeFootprint { id, width, height });
        self
    }

    /// Set an attribute on one node's target (see [`Self::node_attrs`]).
    /// Repeating a name on the same node replaces its earlier value. The
    /// target's own `class`, `type`, `aria-label`, `data-node-index`,
    /// `data-key` and `style` are the canvas's and are not overridden.
    #[must_use]
    pub fn with_node_attr(
        mut self,
        id: Id,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        let name = name.into();
        self.node_attrs
            .retain(|(peer, peer_name, _)| !(*peer == id && *peer_name == name));
        self.node_attrs.push((id, name, value.into()));
        self
    }

    /// Show `text` as one node's visible label instead of its accessible
    /// name (see [`Self::visible_labels`]). Repeating an id replaces it.
    #[must_use]
    pub fn with_visible_label(mut self, id: Id, text: impl Into<String>) -> Self {
        self.visible_labels.retain(|(peer, _)| *peer != id);
        self.visible_labels.push((id, text.into()));
        self
    }

    /// Put one node's visible label at `placement` (see
    /// [`Self::label_placements`]). Repeating an id replaces it.
    #[must_use]
    pub fn with_label_placement(mut self, id: Id, placement: GraphLabelPlacement) -> Self {
        self.label_placements.retain(|(peer, _)| *peer != id);
        self.label_placements.push((id, placement));
        self
    }

    /// The text a node's visible label shows: its own visible text, or its
    /// accessible name, cut in the middle when it is longer than a label holds
    /// ([`middle_cut`]).
    fn label_text(&self, index: usize) -> std::borrow::Cow<'_, str> {
        let node = &self.graph.nodes[index];
        let text = self
            .visible_labels
            .iter()
            .find(|(id, _)| *id == node.id)
            .map_or(node.label.as_str(), |(_, text)| text.as_str());
        middle_cut(text, GRAPH_LABEL_CHARS)
    }

    /// Ask for keyboard focus on one node's target (see
    /// [`Self::focus_request`]).
    #[must_use]
    pub fn with_focus_request(mut self, id: Option<Id>) -> Self {
        self.focus_request = id;
        self
    }

    fn node_index(&self, id: Option<&Id>) -> Option<u16> {
        let id = id?;
        self.graph
            .nodes
            .iter()
            .position(|node| &node.id == id)
            .and_then(|index| u16::try_from(index).ok())
    }

    /// Build the Sprigging leaf for this viewport. Node kind is kept in the
    /// contract; the caller's palette resolves it to paint, so product-specific
    /// kinds do not leak into Cambium.
    pub fn paint_leaf(&self, color_for_kind: impl Fn(&Kind) -> ColorF) -> GraphCanvas {
        let nodes = self
            .graph
            .nodes
            .iter()
            .map(|node| GraphGlyphNode {
                x: node.position.0,
                y: node.position.1,
                color: color_for_kind(&node.kind),
            })
            .collect();
        let edges = if self.relations.is_empty() {
            self.graph
                .edges
                .iter()
                .filter_map(|edge| {
                    let from = self.node_index(Some(&edge.from))?;
                    let to = self.node_index(Some(&edge.to))?;
                    Some((from, to))
                })
                .collect()
        } else {
            Vec::new()
        };
        let mut leaf = GraphCanvas::new(
            if self.atlas.is_some() {
                Vec::new()
            } else {
                nodes
            },
            if self.atlas.is_some() {
                Vec::new()
            } else {
                edges
            },
            Size {
                width: self.width as f32,
                height: self.height as f32,
            },
        );
        leaf.node_radius = self.node_radius;
        leaf.edge_width = self.edge_width;
        // Atlas fields are the geographic visual identity. Graph nodes remain
        // callout/label state, so the old mandatory circular markers do not
        // paint over the caller's authored field polygons.
        if self.atlas.is_some() {
            leaf.node_radius = 0.0;
        }
        if self.atlas.is_none() && !self.relations.is_empty() {
            leaf.set_relations(
                self.resolved_relation_routes()
                    .into_iter()
                    .map(|(relation, points)| GraphGlyphRelation {
                        points,
                        emphasized: relation.emphasized,
                        line: self.relation_line(&relation.kind),
                        start_arrow: relation.marks.reverse,
                        end_arrow: relation.marks.forward,
                        opposed: relation.marks.opposed,
                        strength: relation.strength,
                    })
                    .collect(),
            );
        }
        leaf.set_viewport(self.viewport);
        if let Some(atlas) = &self.atlas {
            let view = atlas.view(self.viewport, self.width as f32, self.height as f32);
            leaf.set_atlas_polygons(
                atlas
                    .projected_paint(view)
                    .into_iter()
                    .chain(atlas.projected_routes(view))
                    .chain(atlas.projected_field_feedback(
                        view,
                        self.selected.as_ref(),
                        self.hovered.as_ref().or(self.focus.as_ref()),
                    ))
                    .chain(atlas.projected_callouts(view, &self.graph.nodes))
                    .map(|shape| GraphAtlasPolygon {
                        points: shape.points,
                        fill: shape.fill,
                        stroke: shape.stroke,
                        stroke_width: shape.stroke_width,
                    })
                    .collect(),
            );
            leaf.set_atlas_paths(
                atlas
                    .projected_callout_paths(view, &self.graph.nodes)
                    .into_iter()
                    .map(|path| GraphAtlasCompoundPath {
                        contours: path.contours,
                        fill: path.fill,
                        stroke: path.stroke,
                        stroke_width: path.stroke_width,
                    })
                    .collect(),
            );
        }
        leaf.set_emphasis(
            self.node_index(self.selected.as_ref()),
            self.node_index(self.focus.as_ref()),
            self.node_index(self.hovered.as_ref()),
        );
        leaf
    }

    /// The exact leaf-local point for every node. Useful to hosts that need a
    /// second overlay beside the built-in hit targets.
    pub fn projected_positions(&self) -> Vec<(&Id, (f32, f32))> {
        let size = Size {
            width: self.width as f32,
            height: self.height as f32,
        };
        let inset = self.node_radius + self.edge_width;
        self.graph
            .nodes
            .iter()
            .map(|node| (&node.id, self.viewport.project(node.position, size, inset)))
            .collect()
    }

    /// Place a rectangular overlay around one node using the canvas's exact
    /// viewport projection. Oversized requests shrink to the canvas bounds.
    pub fn projected_node_region(
        &self,
        id: &Id,
        width: f32,
        height: f32,
    ) -> Option<GraphCanvasNodeRegion> {
        let (_, center) = self
            .projected_positions()
            .into_iter()
            .find(|(candidate, _)| *candidate == id)?;
        let canvas_width = self.width as f32;
        let canvas_height = self.height as f32;
        let width = width.max(1.0).min(canvas_width);
        let height = height.max(1.0).min(canvas_height);
        Some(GraphCanvasNodeRegion {
            left: (center.0 - width / 2.0).clamp(0.0, canvas_width - width),
            top: (center.1 - height / 2.0).clamp(0.0, canvas_height - height),
            width,
            height,
        })
    }

    /// Resolve the declared rectangular footprint for one node through this
    /// canvas's current viewport and clamping rules.
    pub fn projected_node_footprint(&self, id: &Id) -> Option<GraphCanvasNodeRegion> {
        let footprint = self
            .node_footprints
            .iter()
            .rev()
            .find(|footprint| &footprint.id == id)?;
        self.projected_node_region(id, footprint.width, footprint.height)
    }

    /// Put a visible node label across the quieter axis at that node. A
    /// horizontal lane gets labels above or below it; a vertical lane gets
    /// labels beside it. This avoids the old failure where every label sat to
    /// the right and a right-going relation was therefore painted through its
    /// source label.
    /// Which nodes' labels show, in node order: all of them unless
    /// [`Self::cull_labels`] is on. Label boxes are estimated from the label's
    /// length at the label's 10 px size. A node an app-owned overlay occupies
    /// ([`Self::node_footprints`]) shows the overlay, never its label as well,
    /// which would print across the overlay's edge.
    fn shown_labels(&self) -> Vec<bool> {
        let count = self.graph.nodes.len();
        let occupied = |index: usize| {
            let id = &self.graph.nodes[index].id;
            self.node_footprints
                .iter()
                .any(|footprint| &footprint.id == id)
        };
        if !self.cull_labels {
            return (0..count).map(|index| !occupied(index)).collect();
        }
        const HEIGHT: f32 = 14.0;
        let clearance = self.node_radius + 7.0;
        let canvas_width = self.width as f32;
        let positions: Vec<(f32, f32)> = self
            .projected_positions()
            .into_iter()
            .map(|(_, at)| at)
            .collect();
        // The boxes `GraphLabelPlacement::style` lays out.
        let rect = |index: usize| {
            let (x, y) = positions[index];
            let width = label_text_width(&self.label_text(index));
            match self.label_placement(index) {
                placement @ (GraphLabelPlacement::Above | GraphLabelPlacement::Below) => {
                    let top = if placement == GraphLabelPlacement::Above {
                        y - clearance - HEIGHT
                    } else {
                        y + clearance
                    };
                    let left = (x - width * 0.5).clamp(0.0, (canvas_width - width).max(0.0));
                    (left, top, width, HEIGHT)
                },
                GraphLabelPlacement::Left => {
                    (x - clearance - width, y - HEIGHT * 0.5, width, HEIGHT)
                },
                GraphLabelPlacement::Right => (x + clearance, y - HEIGHT * 0.5, width, HEIGHT),
            }
        };
        let overlaps = |a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)| {
            a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.3 && b.1 < a.1 + a.3
        };
        let pinned = |index: usize| {
            let id = &self.graph.nodes[index].id;
            [&self.selected, &self.focus, &self.hovered]
                .into_iter()
                .any(|marked| marked.as_ref() == Some(id))
        };
        let order = (0..count)
            .filter(|&index| pinned(index))
            .chain((0..count).filter(|&index| !pinned(index)));
        let mut shown = vec![false; count];
        let mut kept = Vec::new();
        for index in order.filter(|&index| !occupied(index)) {
            let boxed = rect(index);
            if pinned(index) || !kept.iter().any(|&other| overlaps(boxed, other)) {
                shown[index] = true;
                kept.push(boxed);
            }
        }
        shown
    }

    fn label_placement(&self, index: usize) -> GraphLabelPlacement {
        let Some(node) = self.graph.nodes.get(index) else {
            return GraphLabelPlacement::Right;
        };
        if let Some((_, placement)) = self.label_placements.iter().find(|(id, _)| *id == node.id) {
            return *placement;
        }
        let mut horizontal = 0.0_f32;
        let mut vertical = 0.0_f32;
        let mut add_neighbor = |neighbor: &Id| {
            let Some(peer) = self.graph.nodes.iter().find(|peer| &peer.id == neighbor) else {
                return;
            };
            horizontal += (peer.position.0 - node.position.0).abs();
            vertical += (peer.position.1 - node.position.1).abs();
        };
        if self.relations.is_empty() {
            for edge in &self.graph.edges {
                if edge.from == node.id {
                    add_neighbor(&edge.to);
                } else if edge.to == node.id {
                    add_neighbor(&edge.from);
                }
            }
        } else {
            for relation in self.relations.iter().filter(|relation| relation.visible) {
                if relation.from == node.id {
                    add_neighbor(&relation.to);
                } else if relation.to == node.id {
                    add_neighbor(&relation.from);
                }
            }
        }

        let size = Size {
            width: self.width as f32,
            height: self.height as f32,
        };
        let position =
            self.viewport
                .project(node.position, size, self.node_radius + self.edge_width);
        if horizontal > 0.0 && horizontal >= vertical {
            if position.1 >= self.height as f32 * 0.28 {
                GraphLabelPlacement::Above
            } else {
                GraphLabelPlacement::Below
            }
        } else {
            self.side_with_room(position.0)
        }
    }

    /// Right when a whole label fits there or the right has the more room;
    /// otherwise left. A narrow canvas then clips fewer labels than a fixed
    /// share of its width would.
    fn side_with_room(&self, x: f32) -> GraphLabelPlacement {
        let right = self.width as f32 - x;
        if right >= GRAPH_LABEL_WIDTH || right >= x {
            GraphLabelPlacement::Right
        } else {
            GraphLabelPlacement::Left
        }
    }

    /// Visible relation cells with their projected polylines. This is shared by
    /// retained paint and native relation targets, so a relation is neither
    /// hittable when hidden nor represented at stale endpoint geometry.
    pub fn projected_relations(&self) -> Vec<(&GraphCanvasRelation<Id>, Vec<(f32, f32)>)> {
        let size = Size {
            width: self.width as f32,
            height: self.height as f32,
        };
        let inset = self.node_radius + self.edge_width;
        self.resolved_relation_routes()
            .into_iter()
            .map(|(relation, route)| {
                (
                    relation,
                    route
                        .into_iter()
                        .map(|point| self.viewport.project(point, size, inset))
                        .collect(),
                )
            })
            .collect()
    }

    /// Apply view-local node footprints to the normalized relation routes.
    /// Clipping happens in leaf pixels so the perimeter remains exact under a
    /// non-square canvas, pan, or zoom, then returns to graph coordinates for
    /// the shared Sprigging paint path.
    fn resolved_relation_routes(&self) -> Vec<(&GraphCanvasRelation<Id>, Vec<(f32, f32)>)> {
        let size = Size {
            width: self.width as f32,
            height: self.height as f32,
        };
        let inset = self.node_radius + self.edge_width;
        self.relation_routes()
            .into_iter()
            .map(|(relation, route)| {
                let from_region = self.projected_node_footprint(&relation.from);
                let to_region = self.projected_node_footprint(&relation.to);
                if from_region.is_none() && to_region.is_none() {
                    return (relation, route);
                }
                let mut projected = route
                    .into_iter()
                    .map(|point| self.viewport.project(point, size, inset))
                    .collect::<Vec<_>>();
                if projected.len() >= 2 {
                    if let Some(region) = from_region {
                        let center = self.projected_position(&relation.from);
                        if let Some(center) = center {
                            projected[0] = rectangle_ray_exit(center, projected[1], region);
                        }
                    }
                    if let Some(region) = to_region {
                        let center = self.projected_position(&relation.to);
                        if let Some(center) = center {
                            let last = projected.len() - 1;
                            projected[last] =
                                rectangle_ray_exit(center, projected[last - 1], region);
                        }
                    }
                }
                let route = projected
                    .into_iter()
                    .map(|point| graph_position_at_unclamped(self.viewport, size, inset, point))
                    .collect();
                (relation, route)
            })
            .collect()
    }

    fn projected_position(&self, id: &Id) -> Option<(f32, f32)> {
        let node = self.graph.nodes.iter().find(|node| &node.id == id)?;
        Some(self.viewport.project(
            node.position,
            Size {
                width: self.width as f32,
                height: self.height as f32,
            },
            self.node_radius + self.edge_width,
        ))
    }

    /// Resolve one path per disclosed link appearance. The only fixed offset
    /// separates the two strata when both are present; it never depends on
    /// assertion count. Explicit authored paths retain their interiors.
    fn relation_routes(&self) -> Vec<(&GraphCanvasRelation<Id>, Vec<(f32, f32)>)> {
        let visible = self
            .relations
            .iter()
            .filter(|r| r.visible)
            .collect::<Vec<_>>();
        visible
            .iter()
            .filter_map(|&relation| {
                let a = self
                    .graph
                    .nodes
                    .iter()
                    .find(|n| n.id == relation.from)?
                    .position;
                let b = self
                    .graph
                    .nodes
                    .iter()
                    .find(|n| n.id == relation.to)?
                    .position;
                if relation.route.len() > 2 {
                    return Some((relation, relation.route.clone()));
                }
                if relation.from == relation.to {
                    return Some((
                        relation,
                        vec![a, (a.0 + 0.08, a.1 - 0.08), (a.0 + 0.108, a.1 + 0.08), a],
                    ));
                }
                let two_strata = relation.link_key.as_ref().filter(|key| {
                    visible.iter().any(|peer| {
                        peer.link_key
                            .as_ref()
                            .is_some_and(|other| key.stratum != other.stratum)
                            && ((peer.from == relation.from && peer.to == relation.to)
                                || (peer.from == relation.to && peer.to == relation.from))
                    })
                });
                let Some(key) = two_strata else {
                    return Some((relation, vec![a, b]));
                };
                let size = Size {
                    width: self.width as f32,
                    height: self.height as f32,
                };
                let inset = self.node_radius + self.edge_width;
                let a_px = self.viewport.project(a, size, inset);
                let b_px = self.viewport.project(b, size, inset);
                let dx = b_px.0 - a_px.0;
                let dy = b_px.1 - a_px.1;
                let length = dx.hypot(dy).max(0.0001);
                // Normalize by endpoint order so reversing a route keeps its lane.
                let orientation = if a.0 < b.0 || (a.0 == b.0 && a.1 <= b.1) {
                    1.0
                } else {
                    -1.0
                };
                let offset = orientation
                    * match key.stratum {
                        sceno::LinkStratum::Surface => -3.0,
                        sceno::LinkStratum::Resource => 3.0,
                    };
                let normal = (-dy / length * offset, dx / length * offset);
                let route = vec![
                    a,
                    graph_position_at_unclamped(
                        self.viewport,
                        size,
                        inset,
                        (a_px.0 + dx * 0.28 + normal.0, a_px.1 + dy * 0.28 + normal.1),
                    ),
                    graph_position_at_unclamped(
                        self.viewport,
                        size,
                        inset,
                        (a_px.0 + dx * 0.72 + normal.0, a_px.1 + dy * 0.72 + normal.1),
                    ),
                    b,
                ];
                Some((relation, route))
            })
            .collect()
    }

    /// Convert a leaf-local pointer position back into the graph's normalized
    /// coordinate system. This is the inverse of the projection used by paint
    /// and native node targets, clamped to the graph's visible bounds so a
    /// captured drag cannot create an unreachable position.
    pub fn graph_position_at(&self, local: (f32, f32)) -> (f32, f32) {
        graph_position_at(
            self.viewport,
            Size {
                width: self.width as f32,
                height: self.height as f32,
            },
            self.node_radius + self.edge_width,
            local,
        )
    }
}

/// Follow a ray from a point inside a rectangle to the first perimeter it
/// exits. A clamped node can already sit on that perimeter, in which case the
/// unchanged origin is the truthful endpoint.
fn rectangle_ray_exit(
    origin: (f32, f32),
    toward: (f32, f32),
    region: GraphCanvasNodeRegion,
) -> (f32, f32) {
    const EPSILON: f32 = 0.0001;
    let dx = toward.0 - origin.0;
    let dy = toward.1 - origin.1;
    if dx.abs() < EPSILON && dy.abs() < EPSILON {
        return origin;
    }

    let right = region.left + region.width;
    let bottom = region.top + region.height;
    let tx = if dx > EPSILON {
        (right - origin.0) / dx
    } else if dx < -EPSILON {
        (region.left - origin.0) / dx
    } else {
        f32::INFINITY
    };
    let ty = if dy > EPSILON {
        (bottom - origin.1) / dy
    } else if dy < -EPSILON {
        (region.top - origin.1) / dy
    } else {
        f32::INFINITY
    };
    let distance = tx.max(0.0).min(ty.max(0.0));
    if !distance.is_finite() {
        return origin;
    }
    (
        (origin.0 + dx * distance).clamp(region.left, right),
        (origin.1 + dy * distance).clamp(region.top, bottom),
    )
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

/// A label's width estimated from its length at the labels' 10 px size, no
/// wider than [`GRAPH_LABEL_WIDTH`].
fn label_text_width(label: &str) -> f32 {
    (label.chars().count() as f32 * LABEL_CHAR_WIDTH + 2.0).min(GRAPH_LABEL_WIDTH)
}

/// `text` cut to at most `max` characters by dropping its middle, so a long
/// label keeps both its start and its distinguishing end, as in
/// `Session (fork of … blake3:f612a))`. Names that differ only at their end
/// (two forks, two codicils) stay told apart, which an end ellipsis loses.
/// Text that fits is returned as it is.
pub(crate) fn middle_cut(text: &str, max: usize) -> std::borrow::Cow<'_, str> {
    const MARK: &str = " … ";
    let count = text.chars().count();
    if count <= max || max <= MARK.chars().count() + 1 {
        return std::borrow::Cow::Borrowed(text);
    }
    // How far a cut moves to land between words rather than inside one.
    const SNAP: usize = 6;
    let chars: Vec<char> = text.chars().collect();
    let keep = max - MARK.chars().count();
    // The end is what tells long names apart. It starts at a word: the next
    // one when that is near, else the one it falls in.
    let mut tail_start = count - keep / 2;
    let at_word = |at: usize| at == 0 || chars[at - 1].is_whitespace();
    if !at_word(tail_start) {
        if let Some(ahead) = (0..SNAP)
            .take_while(|ahead| tail_start + ahead < count)
            .find(|ahead| chars[tail_start + ahead].is_whitespace())
        {
            tail_start += ahead + 1;
        } else if let Some(back) = (1..=SNAP)
            .take_while(|back| *back < tail_start)
            .find(|back| chars[tail_start - back].is_whitespace())
        {
            tail_start -= back - 1;
        }
    }
    // The start takes the rest, ending between words when one is near.
    let mut head_end = keep.saturating_sub(count - tail_start).clamp(1, tail_start);
    if !at_word(head_end)
        && let Some(back) = (1..=SNAP)
            .take_while(|back| *back < head_end)
            .find(|back| chars[head_end - back].is_whitespace())
    {
        head_end -= back;
    }
    let start: String = chars[..head_end].iter().collect();
    let end: String = chars[tail_start..].iter().collect();
    std::borrow::Cow::Owned(format!("{}{MARK}{}", start.trim_end(), end.trim_start()))
}

/// Where a node's visible label sits against its node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphLabelPlacement {
    Above,
    Below,
    Left,
    Right,
}

impl GraphLabelPlacement {
    fn name(self) -> &'static str {
        match self {
            Self::Above => "above",
            Self::Below => "below",
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    /// A label box anchored to its node. The box hugs the node without
    /// relying on `text-align` inside a fixed width, which Genet did not honour
    /// for these absolutely placed spans: a left label is anchored by its
    /// right edge, and an above or below label is centred from `text_width`,
    /// the estimate [`label_text_width`] gives.
    fn style(self, x: f32, y: f32, clearance: f32, canvas: Size, text_width: f32) -> String {
        const LABEL_WIDTH: f32 = GRAPH_LABEL_WIDTH;
        const LABEL_HEIGHT: f32 = 14.0;
        let centred_left = (x - text_width * 0.5).clamp(0.0, (canvas.width - text_width).max(0.0));
        let centred_top =
            (y - LABEL_HEIGHT * 0.5).clamp(0.0, (canvas.height - LABEL_HEIGHT).max(0.0));
        match self {
            Self::Above => format!(
                "position:absolute;left:{centred_left}px;top:{}px;width:{text_width}px;text-align:center;overflow:hidden;text-overflow:ellipsis;",
                (y - clearance - LABEL_HEIGHT).max(0.0)
            ),
            Self::Below => format!(
                "position:absolute;left:{centred_left}px;top:{}px;width:{text_width}px;text-align:center;overflow:hidden;text-overflow:ellipsis;",
                (y + clearance).min((canvas.height - LABEL_HEIGHT).max(0.0))
            ),
            Self::Left => {
                let room = (x - clearance).clamp(0.0, LABEL_WIDTH);
                format!(
                    "position:absolute;right:{}px;top:{centred_top}px;max-width:{room}px;text-align:right;overflow:hidden;text-overflow:ellipsis;",
                    (canvas.width - (x - clearance)).max(0.0)
                )
            },
            Self::Right => {
                let left = (x + clearance).min(canvas.width);
                let width = (canvas.width - left).clamp(0.0, LABEL_WIDTH);
                format!(
                    "position:absolute;left:{left}px;top:{centred_top}px;width:{width}px;text-align:left;overflow:hidden;text-overflow:ellipsis;"
                )
            },
        }
    }
}

fn graph_position_at(
    viewport: GraphViewport,
    size: Size,
    inset: f32,
    local: (f32, f32),
) -> (f32, f32) {
    let width = (size.width - 2.0 * inset).max(0.0);
    let height = (size.height - 2.0 * inset).max(0.0);
    let zoom = viewport.zoom.max(0.01);
    let x = if width > 0.0 {
        (local.0 - inset) / width
    } else {
        0.5
    };
    let y = if height > 0.0 {
        (local.1 - inset) / height
    } else {
        0.5
    };
    (
        ((x - 0.5 - viewport.pan.0) / zoom + 0.5).clamp(0.0, 1.0),
        ((y - 0.5 - viewport.pan.1) / zoom + 0.5).clamp(0.0, 1.0),
    )
}

fn graph_position_at_unclamped(
    viewport: GraphViewport,
    size: Size,
    inset: f32,
    local: (f32, f32),
) -> (f32, f32) {
    let width = (size.width - 2.0 * inset).max(0.0);
    let height = (size.height - 2.0 * inset).max(0.0);
    let zoom = viewport.zoom.max(0.01);
    let x = if width > 0.0 {
        (local.0 - inset) / width
    } else {
        0.5
    };
    let y = if height > 0.0 {
        (local.1 - inset) / height
    } else {
        0.5
    };
    (
        (x - 0.5 - viewport.pan.0) / zoom + 0.5,
        (y - 0.5 - viewport.pan.1) / zoom + 0.5,
    )
}

/// The segment crossing a polyline's half-length point. Relation hit targets
/// sit on this segment, sharing the same path as the painted link.
fn route_midpoint_segment(points: &[(f32, f32)]) -> Option<((f32, f32), (f32, f32))> {
    let segments = points
        .windows(2)
        .map(|pair| {
            let dx = pair[1].0 - pair[0].0;
            let dy = pair[1].1 - pair[0].1;
            (pair[0], pair[1], (dx * dx + dy * dy).sqrt())
        })
        .collect::<Vec<_>>();
    let total = segments.iter().map(|(_, _, length)| *length).sum::<f32>();
    let mut walked = 0.0;
    for (from, to, length) in &segments {
        if walked + length >= total * 0.5 {
            return Some((*from, *to));
        }
        walked += length;
    }
    segments.last().map(|(from, to, _)| (*from, *to))
}

/// Render a bounded graph canvas with one native node target per painted node.
///
/// `on_node_click` owns navigation or staging. `on_node_hover` normally writes
/// the supplied id into [`GraphCanvasSwatch::hovered`] (and clears it on leave),
/// after which the host refreshes the registered leaf. `on_expand` switches to
/// the app's full-canvas route. The component does not invent those policies.
pub fn graph_canvas_swatch<State, AppAction, Id, Kind, Click, ClickOut, Hover, Expand, ExpandOut>(
    swatch: &GraphCanvasSwatch<Id, Kind>,
    on_node_click: Click,
    on_node_hover: Hover,
    on_expand: Expand,
) -> impl View<State, AppAction, GenetCtx, Element = GenetElement>
where
    State: 'static,
    AppAction: 'static,
    Id: Clone + PartialEq + 'static,
    ClickOut: OptionalAction<AppAction>,
    Click: Fn(&mut State, Id) -> ClickOut + Clone + 'static,
    Hover: Fn(&mut State, Option<Id>) + Clone + 'static,
    ExpandOut: OptionalAction<AppAction>,
    Expand: Fn(&mut State) -> ExpandOut + Clone + 'static,
{
    graph_canvas_swatch_with_focus_and_drag_and_relations(
        swatch,
        on_node_click,
        on_node_hover,
        |_state: &mut State, _id: Option<Id>| {},
        |_state: &mut State, _event: GraphCanvasNodeDrag<Id>| {},
        |_state: &mut State, _id: String| {},
        |_state: &mut State, _id: Option<String>| {},
        on_expand,
        |_state: &mut State, _event: PointerEvent| {},
        |_state: &mut State, _event: WheelEvent| {},
    )
}

/// Render a graph-canvas Swatch with captured node motion. This is the
/// interaction variant of [`graph_canvas_swatch`]: click and hover retain their
/// existing meanings while `on_node_drag` receives Down/Move/Up in normalized
/// graph coordinates.
pub fn graph_canvas_swatch_with_drag<
    State,
    AppAction,
    Id,
    Kind,
    Click,
    ClickOut,
    Hover,
    Drag,
    DragOut,
    Expand,
    ExpandOut,
>(
    swatch: &GraphCanvasSwatch<Id, Kind>,
    on_node_click: Click,
    on_node_hover: Hover,
    on_node_drag: Drag,
    on_expand: Expand,
) -> impl View<State, AppAction, GenetCtx, Element = GenetElement>
where
    State: 'static,
    AppAction: 'static,
    Id: Clone + PartialEq + 'static,
    ClickOut: OptionalAction<AppAction>,
    Click: Fn(&mut State, Id) -> ClickOut + Clone + 'static,
    Hover: Fn(&mut State, Option<Id>) + Clone + 'static,
    DragOut: OptionalAction<AppAction>,
    Drag: Fn(&mut State, GraphCanvasNodeDrag<Id>) -> DragOut + Clone + 'static,
    ExpandOut: OptionalAction<AppAction>,
    Expand: Fn(&mut State) -> ExpandOut + Clone + 'static,
{
    graph_canvas_swatch_with_focus_and_drag_and_relations(
        swatch,
        on_node_click,
        on_node_hover,
        |_state: &mut State, _id: Option<Id>| {},
        on_node_drag,
        |_state: &mut State, _id: String| {},
        |_state: &mut State, _id: Option<String>| {},
        on_expand,
        |_state: &mut State, _event: PointerEvent| {},
        |_state: &mut State, _event: WheelEvent| {},
    )
}

/// Render a graph-canvas swatch whose retained paint focus follows the native
/// node button that pointer, keyboard, or programmatic focus selected.
///
/// `on_node_focus` normally writes its value into
/// [`GraphCanvasSwatch::focus`] before rebuilding the matching paint leaf.
pub fn graph_canvas_swatch_with_focus<
    State,
    AppAction,
    Id,
    Kind,
    Click,
    ClickOut,
    Hover,
    Focus,
    Expand,
    ExpandOut,
>(
    swatch: &GraphCanvasSwatch<Id, Kind>,
    on_node_click: Click,
    on_node_hover: Hover,
    on_node_focus: Focus,
    on_expand: Expand,
) -> impl View<State, AppAction, GenetCtx, Element = GenetElement>
where
    State: 'static,
    AppAction: 'static,
    Id: Clone + PartialEq + 'static,
    ClickOut: OptionalAction<AppAction>,
    Click: Fn(&mut State, Id) -> ClickOut + Clone + 'static,
    Hover: Fn(&mut State, Option<Id>) + Clone + 'static,
    Focus: Fn(&mut State, Option<Id>) + Clone + 'static,
    ExpandOut: OptionalAction<AppAction>,
    Expand: Fn(&mut State) -> ExpandOut + Clone + 'static,
{
    graph_canvas_swatch_with_focus_and_drag_and_relations(
        swatch,
        on_node_click,
        on_node_hover,
        on_node_focus,
        |_state: &mut State, _event: GraphCanvasNodeDrag<Id>| {},
        |_state: &mut State, _id: String| {},
        |_state: &mut State, _id: Option<String>| {},
        on_expand,
        |_state: &mut State, _event: PointerEvent| {},
        |_state: &mut State, _event: WheelEvent| {},
    )
}

/// Render a graph-canvas Swatch with captured node motion and independently
/// targetable relation cells. Relation callbacks receive a consumer-owned
/// stable relation id, so equal endpoint pairs remain separate cells.
pub fn graph_canvas_swatch_with_drag_and_relations<
    State,
    AppAction,
    Id,
    Kind,
    Click,
    ClickOut,
    Hover,
    Drag,
    DragOut,
    RelationClick,
    RelationClickOut,
    RelationHover,
    Expand,
    ExpandOut,
>(
    swatch: &GraphCanvasSwatch<Id, Kind>,
    on_node_click: Click,
    on_node_hover: Hover,
    on_node_drag: Drag,
    on_relation_click: RelationClick,
    on_relation_hover: RelationHover,
    on_expand: Expand,
) -> impl View<State, AppAction, GenetCtx, Element = GenetElement>
where
    State: 'static,
    AppAction: 'static,
    Id: Clone + PartialEq + 'static,
    ClickOut: OptionalAction<AppAction>,
    Click: Fn(&mut State, Id) -> ClickOut + Clone + 'static,
    Hover: Fn(&mut State, Option<Id>) + Clone + 'static,
    DragOut: OptionalAction<AppAction>,
    Drag: Fn(&mut State, GraphCanvasNodeDrag<Id>) -> DragOut + Clone + 'static,
    RelationClickOut: OptionalAction<AppAction>,
    RelationClick: Fn(&mut State, String) -> RelationClickOut + Clone + 'static,
    RelationHover: Fn(&mut State, Option<String>) + Clone + 'static,
    ExpandOut: OptionalAction<AppAction>,
    Expand: Fn(&mut State) -> ExpandOut + Clone + 'static,
{
    graph_canvas_swatch_with_focus_and_drag_and_relations(
        swatch,
        on_node_click,
        on_node_hover,
        |_state: &mut State, _id: Option<Id>| {},
        on_node_drag,
        on_relation_click,
        on_relation_hover,
        on_expand,
        |_state: &mut State, _event: PointerEvent| {},
        |_state: &mut State, _event: WheelEvent| {},
    )
}

/// Render a graph-canvas Swatch with focus and captured node-motion callbacks.
///
/// The component emits motion only; the consumer keeps ownership of position,
/// solver, and persistence policy. Existing callers can keep using
/// [`graph_canvas_swatch`] or [`graph_canvas_swatch_with_focus`].
pub fn graph_canvas_swatch_with_focus_and_drag<
    State,
    AppAction,
    Id,
    Kind,
    Click,
    ClickOut,
    Hover,
    Focus,
    Drag,
    DragOut,
    Expand,
    ExpandOut,
>(
    swatch: &GraphCanvasSwatch<Id, Kind>,
    on_node_click: Click,
    on_node_hover: Hover,
    on_node_focus: Focus,
    on_node_drag: Drag,
    on_expand: Expand,
) -> impl View<State, AppAction, GenetCtx, Element = GenetElement>
where
    State: 'static,
    AppAction: 'static,
    Id: Clone + PartialEq + 'static,
    ClickOut: OptionalAction<AppAction>,
    Click: Fn(&mut State, Id) -> ClickOut + Clone + 'static,
    Hover: Fn(&mut State, Option<Id>) + Clone + 'static,
    Focus: Fn(&mut State, Option<Id>) + Clone + 'static,
    DragOut: OptionalAction<AppAction>,
    Drag: Fn(&mut State, GraphCanvasNodeDrag<Id>) -> DragOut + Clone + 'static,
    ExpandOut: OptionalAction<AppAction>,
    Expand: Fn(&mut State) -> ExpandOut + Clone + 'static,
{
    graph_canvas_swatch_with_focus_and_drag_and_relations(
        swatch,
        on_node_click,
        on_node_hover,
        on_node_focus,
        on_node_drag,
        |_state: &mut State, _id: String| {},
        |_state: &mut State, _id: Option<String>| {},
        on_expand,
        |_state: &mut State, _event: PointerEvent| {},
        |_state: &mut State, _event: WheelEvent| {},
    )
}

/// The full graph-canvas interaction surface, including independently
/// targetable relation cells.
pub fn graph_canvas_swatch_with_focus_and_drag_and_relations<
    State,
    AppAction,
    Id,
    Kind,
    Click,
    ClickOut,
    Hover,
    Focus,
    Drag,
    DragOut,
    RelationClick,
    RelationClickOut,
    RelationHover,
    Expand,
    ExpandOut,
    Pan,
    PanOut,
    Zoom,
    ZoomOut,
>(
    swatch: &GraphCanvasSwatch<Id, Kind>,
    on_node_click: Click,
    on_node_hover: Hover,
    on_node_focus: Focus,
    on_node_drag: Drag,
    on_relation_click: RelationClick,
    on_relation_hover: RelationHover,
    on_expand: Expand,
    on_background_pointer: Pan,
    on_wheel_event: Zoom,
) -> impl View<State, AppAction, GenetCtx, Element = GenetElement>
where
    State: 'static,
    AppAction: 'static,
    Id: Clone + PartialEq + 'static,
    // The activating handlers may emit an action (returning `()` keeps them
    // silent, which is what every state-mutating consumer does). The hover and
    // focus handlers stay silent by construction: emphasis is presentation
    // state, never an app-facing event.
    ClickOut: OptionalAction<AppAction>,
    Click: Fn(&mut State, Id) -> ClickOut + Clone + 'static,
    Hover: Fn(&mut State, Option<Id>) + Clone + 'static,
    Focus: Fn(&mut State, Option<Id>) + Clone + 'static,
    DragOut: OptionalAction<AppAction>,
    Drag: Fn(&mut State, GraphCanvasNodeDrag<Id>) -> DragOut + Clone + 'static,
    RelationClickOut: OptionalAction<AppAction>,
    RelationClick: Fn(&mut State, String) -> RelationClickOut + Clone + 'static,
    RelationHover: Fn(&mut State, Option<String>) + Clone + 'static,
    ExpandOut: OptionalAction<AppAction>,
    Expand: Fn(&mut State) -> ExpandOut + Clone + 'static,
    // The background pointer and wheel handlers receive the raw events: pan
    // anchoring and zoom stepping are caller policy ([`graph_canvas`] turns
    // them into [`GraphCanvasEvent::Pan`] / [`GraphCanvasEvent::Zoom`]).
    // Returning `()` keeps a caller that has no viewport events silent.
    PanOut: OptionalAction<AppAction>,
    Pan: Fn(&mut State, PointerEvent) -> PanOut + Clone + 'static,
    ZoomOut: OptionalAction<AppAction>,
    Zoom: Fn(&mut State, WheelEvent) -> ZoomOut + Clone + 'static,
{
    let positions = swatch.projected_positions();
    let hit_size = swatch.hit_size.max(1.0);
    let size = Size {
        width: swatch.width as f32,
        height: swatch.height as f32,
    };
    let inset = swatch.node_radius + swatch.edge_width;
    let viewport = swatch.viewport;
    let defer_drag_rebuild = swatch.defer_drag_rebuild;
    // Visible node labels (opt-in): plain positioned text beside each node,
    // aria-hidden (the button already carries the accessible name) and
    // pointer-transparent (they must not steal the node's clicks).
    let labels: Vec<_> = if swatch.show_labels {
        let shown = swatch.shown_labels();
        swatch
            .graph
            .nodes
            .iter()
            .zip(swatch.projected_positions())
            .enumerate()
            .filter(|(index, _)| shown[*index])
            .map(|(index, (node, (_, (x, y))))| {
                // Same state modifiers the node buttons carry. A label layer
                // that cannot say which node is selected forces a consumer to
                // keep hand-rolling the whole layer for the sake of one
                // emphasis, which is what Isometry's pointcrawl was doing: the
                // party's location is the first thing that layer has to say.
                let mut class = String::from("graph-canvas-swatch-label");
                if swatch.selected.as_ref() == Some(&node.id) {
                    class.push_str(" selected");
                }
                if swatch.focus.as_ref() == Some(&node.id) {
                    class.push_str(" focused");
                }
                if swatch.hovered.as_ref() == Some(&node.id) {
                    class.push_str(" hovered");
                }
                let placement = swatch.label_placement(index);
                el::<_, State, AppAction>("span", swatch.label_text(index).into_owned())
                    .attr("class", class)
                    .attr("aria-hidden", "true")
                    .attr("data-label-placement", placement.name())
                    .attr(
                        "style",
                        placement.style(
                            x,
                            y,
                            swatch.node_radius + 7.0,
                            size,
                            label_text_width(&swatch.label_text(index)),
                        ),
                    )
            })
            .collect()
    } else {
        Vec::new()
    };
    let relation_targets: Vec<_> = swatch
        .projected_relations()
        .into_iter()
        .filter_map(|(relation, route)| {
            let ((from_x, from_y), (to_x, to_y)) = route_midpoint_segment(&route)?;
            let dx = to_x - from_x;
            let dy = to_y - from_y;
            let length = (dx * dx + dy * dy).sqrt().max(1.0);
            let midpoint_x = (from_x + to_x) / 2.0;
            let midpoint_y = (from_y + to_y) / 2.0;
            let angle = dy.atan2(dx).to_degrees();
            let mut class = String::from("graph-canvas-swatch-relation");
            if relation.emphasized {
                class.push_str(" emphasized");
            }
            let label = format!("{}: {}", relation.kind, relation.label);
            let label = if relation.link_key.is_some() {
                format!("{label}; {}{}",
                    match (relation.marks.forward, relation.marks.reverse) {
                        (true, true) => "both directions",
                        (true, false) => "forward",
                        (false, true) => "reverse",
                        _ => "direction unavailable",
                    },
                    if relation.marks.opposed { "; opposing assertions" } else { "" })
            } else {
                label
            };
            let mut target = el::<_, State, AppAction>("button", ())
                .attr("class", class)
                .attr("type", "button")
                .attr("aria-label", label)
                .attr("data-relation-id", relation.id.clone())
                .attr("data-link-direction", match (relation.marks.forward,relation.marks.reverse) {(true,true)=>"both",(true,false)=>"forward",(false,true)=>"reverse",_=>"none"})
                .attr("data-link-opposed", if relation.marks.opposed {"true"} else {"false"})
                // For the host's sheet: a line style per kind, so kinds differ
                // by more than colour.
                .attr("data-kind", relation.kind.clone())
                .attr(
                    "style",
                    format!(
                        "position:absolute;left:{}px;top:{}px;width:{length}px;height:20px;transform:rotate({angle}deg);",
                        midpoint_x - length / 2.0,
                        midpoint_y - 10.0,
                    ),
                );
            if relation.emphasized {
                target = target.attr("aria-current", "true");
            }
            let click = on_relation_click.clone();
            let click_id = relation.id.clone();
            let hover = on_relation_hover.clone();
            let hover_id = relation.id.clone();
            Some(on_hover(
                on_click(target, move |state: &mut State, _: PointerClick| {
                    click(state, click_id.clone())
                }),
                move |state: &mut State, event: HoverEvent| match event.phase {
                    HoverPhase::Enter => hover(state, Some(hover_id.clone())),
                    HoverPhase::Leave => hover(state, None),
                    HoverPhase::Move => {}
                },
            ))
        })
        .collect();
    let targets: Vec<_> = swatch
        .graph
        .nodes
        .iter()
        .zip(positions)
        .enumerate()
        .map(|(index, (node, (_, (x, y))))| {
            let selected = swatch.selected.as_ref() == Some(&node.id);
            let focused = swatch.focus.as_ref() == Some(&node.id);
            let hovered = swatch.hovered.as_ref() == Some(&node.id);
            let mut class = String::from("graph-canvas-swatch-node");
            if selected {
                class.push_str(" selected");
            }
            if focused {
                class.push_str(" focused");
            }
            if hovered {
                class.push_str(" hovered");
            }
            let mut target = el::<_, State, AppAction>("button", ())
                .attr("class", class)
                .attr("type", "button")
                .attr("aria-label", node.label.clone())
                .attr("data-node-index", index.to_string())
                .attr(
                    "style",
                    format!(
                        "position:absolute;left:{}px;top:{}px;width:{hit_size}px;height:{hit_size}px;",
                        x - hit_size / 2.0,
                        y - hit_size / 2.0,
                    ),
                );
            if selected {
                target = target.attr("aria-current", "true");
            }
            if let Some(key) = &node.key {
                target = target.attr("data-key", key.clone());
            }
            const OWN: [&str; 6] = [
                "class",
                "type",
                "aria-label",
                "data-node-index",
                "data-key",
                "style",
            ];
            for (_, name, value) in swatch
                .node_attrs
                .iter()
                .filter(|(id, name, _)| *id == node.id && !OWN.contains(&name.as_str()))
            {
                target = target.attr(name.clone(), value.clone());
            }
            let target = request_focus(target, swatch.focus_request.as_ref() == Some(&node.id));

            let click = on_node_click.clone();
            let click_id = node.id.clone();
            let hover = on_node_hover.clone();
            let enter_id = node.id.clone();
            let focus = on_node_focus.clone();
            let focus_id = node.id.clone();
            let drag = on_node_drag.clone();
            let drag_id = node.id.clone();
            let target_left = x - hit_size / 2.0;
            let target_top = y - hit_size / 2.0;
            on_pointer(focusable(on_focus(
                on_hover(
                    on_click(target, move |state: &mut State, _: PointerClick| {
                        click(state, click_id.clone())
                    }),
                    move |state: &mut State, event: HoverEvent| match event.phase {
                        HoverPhase::Enter => hover(state, Some(enter_id.clone())),
                        HoverPhase::Leave => hover(state, None),
                        HoverPhase::Move => {}
                    },
                ),
                move |state: &mut State, event: FocusEvent| match event.phase {
                    FocusPhase::Gained => focus(state, Some(focus_id.clone())),
                    FocusPhase::Lost => focus(state, None),
                },
            )), move |state: &mut State, event: PointerEvent| {
                if defer_drag_rebuild && matches!(event.phase, PointerPhase::Move) {
                    event.defer_rebuild();
                }
                let position = graph_position_at(
                    viewport,
                    size,
                    inset,
                    (target_left + event.local.0, target_top + event.local.1),
                );
                drag(
                    state,
                    GraphCanvasNodeDrag {
                        id: drag_id.clone(),
                        phase: event.phase,
                        position,
                    },
                )
            })
        })
        .collect();

    let expand = on_expand.clone();
    let expand_buttons: Vec<_> = if swatch.show_expand {
        vec![on_click(
            el::<_, State, AppAction>("button", "Expand")
                .attr("class", "graph-canvas-swatch-expand")
                .attr("type", "button")
                .attr("aria-label", "Expand graph")
                .attr("style", "position:absolute;right:5px;top:5px;"),
            move |state: &mut State, _: PointerClick| expand(state),
        )]
    } else {
        Vec::new()
    };

    let root = el(
        "div",
        (
            custom_leaf::<State, AppAction>(swatch.leaf_key, swatch.width, swatch.height)
                .attr("aria-hidden", "true"),
            el("div", labels)
                .attr("class", "graph-canvas-swatch-labels")
                .attr(
                    "style",
                    format!(
                        "position:absolute;left:0;top:0;width:{}px;height:{}px;",
                        swatch.width, swatch.height
                    ),
                ),
            el("div", relation_targets)
                .attr("class", "graph-canvas-swatch-relation-targets")
                .attr(
                    "style",
                    format!(
                        "position:absolute;left:0;top:0;width:{}px;height:{}px;",
                        swatch.width, swatch.height
                    ),
                ),
            el("div", targets)
                .attr("class", "graph-canvas-swatch-targets")
                .attr(
                    "style",
                    format!(
                        "position:absolute;left:0;top:0;width:{}px;height:{}px;",
                        swatch.width, swatch.height
                    ),
                ),
            expand_buttons,
        ),
    )
    .attr("class", "graph-canvas-swatch")
    .attr("role", "group")
    .attr("aria-label", swatch.label.clone())
    // `overflow:hidden` crops the native node, label, and relation targets a
    // panned or zoomed viewport projects outside this box; the swatch is their
    // `position:relative` containing block, so the clip applies to them. The
    // painted leaf clips itself (see sprigging's GraphCanvas paint).
    .attr(
        "style",
        format!(
            "position:relative;display:block;overflow:hidden;width:{}px;height:{}px;max-width:100%;",
            swatch.width, swatch.height
        ),
    );
    // Background drags and wheel notches reach these root handlers only when
    // no inner target claims the event: a node's own pointer handler is the
    // innermost element under a node drag, so panning never fights dragging.
    on_wheel(on_pointer(root, on_background_pointer), on_wheel_event)
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

/// A graph-canvas Swatch that owns its own emphasis.
///
/// Selection stays application truth and arrives on the `swatch` props (a node
/// is selected because the graph says so). Hover, keyboard focus emphasis, and
/// relation hover are pointer-and-focus presentation state, so they live in the
/// component: an application no longer stores a hover field purely to route it
/// back into the view on the next rebuild. The `hovered` and `focus` fields of
/// the passed `swatch` are ignored for that reason.
///
/// The parent receives only [`GraphCanvasEvent`]s. For the callback-per-axis
/// form (an application that genuinely wants to own emphasis, e.g. to mirror it
/// across two views), the `graph_canvas_swatch*` family remains. Woodshed's
/// Related panel is that case: its hover is shared with a neighbor list, and
/// the two cross-highlight.
///
/// This is a state-ownership boundary, not a compile-time one. Measured with
/// `cargo llvm-lines`, the swatch costs the same either way (~1.6k lines, 11
/// instantiations); the boundary moves that instantiation from the app's state
/// type onto `GraphCanvasLocal<Id>` rather than removing it.
pub fn graph_canvas<State, A, Output, Id, Kind, F>(
    swatch: &GraphCanvasSwatch<Id, Kind>,
    on_event: F,
) -> impl View<State, A, GenetCtx, Element = GenetElement> + use<State, A, Output, Id, Kind, F>
where
    State: 'static,
    A: 'static,
    Output: OptionalAction<A> + 'static,
    Id: Clone + PartialEq + 'static,
    Kind: Clone + PartialEq + 'static,
    F: Fn(&mut State, GraphCanvasEvent<Id>) -> Output + 'static,
{
    graph_canvas_core(
        swatch,
        |event| event,
        |event: WheelEvent| {
            // A notch toward the top zooms in, matching every map surface; the
            // host's own scroll default is suppressed so the page does not
            // scroll under the zoom.
            if event.delta.1 == 0.0 {
                return None;
            }
            event.prevent_default();
            Some(GraphCanvasEvent::Zoom {
                factor: if event.delta.1 > 0.0 { 1.2 } else { 1.0 / 1.2 },
            })
        },
        on_event,
    )
}

/// [`graph_canvas`] with the wheel passed through whole, for a consumer that
/// applies its own wheel policy (a wheel-mode choice, pointer-anchored zoom, a
/// modifier that always zooms) instead of taking a fixed zoom factor.
///
/// Every other event is the same [`GraphCanvasEvent`] [`graph_canvas`] reports,
/// wrapped in [`GraphCanvasInput::Event`].
pub fn graph_canvas_with_wheel<State, A, Output, Id, Kind, F>(
    swatch: &GraphCanvasSwatch<Id, Kind>,
    on_event: F,
) -> impl View<State, A, GenetCtx, Element = GenetElement> + use<State, A, Output, Id, Kind, F>
where
    State: 'static,
    A: 'static,
    Output: OptionalAction<A> + 'static,
    Id: Clone + PartialEq + 'static,
    Kind: Clone + PartialEq + 'static,
    F: Fn(&mut State, GraphCanvasInput<Id>) -> Output + 'static,
{
    graph_canvas_core(
        swatch,
        GraphCanvasInput::Event,
        |event: WheelEvent| Some(GraphCanvasInput::Wheel(event)),
        on_event,
    )
}

/// The graph canvas, generic over the event a child reports to its own
/// component boundary: `lift` wraps each [`GraphCanvasEvent`] and `wheel` turns
/// a wheel notch into one (or none).
fn graph_canvas_core<State, A, Output, Id, Kind, E, Lift, Wheel, F>(
    swatch: &GraphCanvasSwatch<Id, Kind>,
    lift: Lift,
    wheel: Wheel,
    on_event: F,
) -> impl View<State, A, GenetCtx, Element = GenetElement>
+ use<State, A, Output, Id, Kind, E, Lift, Wheel, F>
where
    State: 'static,
    A: 'static,
    Output: OptionalAction<A> + 'static,
    Id: Clone + PartialEq + 'static,
    Kind: Clone + PartialEq + 'static,
    E: crate::Action + 'static,
    Lift: Fn(GraphCanvasEvent<Id>) -> E + Clone + 'static,
    Wheel: Fn(WheelEvent) -> Option<E> + Clone + 'static,
    F: Fn(&mut State, E) -> Output + 'static,
{
    component(
        swatch.clone(),
        |_props: &GraphCanvasSwatch<Id, Kind>| GraphCanvasLocal {
            pan_anchor: None,
            hovered: None,
            focus: None,
            hovered_relation: None,
        },
        // Emphasis is never parent-controlled: a props change (new graph data,
        // a new selection) leaves the pointer where it is.
        |_prev: &GraphCanvasSwatch<Id, Kind>,
         _next: &GraphCanvasSwatch<Id, Kind>,
         _local: &mut GraphCanvasLocal<Id>| {},
        move |props: &GraphCanvasSwatch<Id, Kind>, local: &GraphCanvasLocal<Id>| {
            let lift = lift.clone();
            let wheel = wheel.clone();
            let mut swatch = props.clone();
            swatch.hovered = local.hovered.clone();
            swatch.focus = local.focus.clone();
            if let Some(id) = &local.hovered_relation {
                for relation in &mut swatch.relations {
                    if &relation.id == id {
                        relation.emphasized = true;
                    }
                }
            }
            Box::new(graph_canvas_swatch_with_focus_and_drag_and_relations(
                &swatch,
                {
                    let lift = lift.clone();
                    move |_: &mut GraphCanvasLocal<Id>, id: Id| lift(GraphCanvasEvent::Activate(id))
                },
                |local: &mut GraphCanvasLocal<Id>, id: Option<Id>| local.hovered = id,
                |local: &mut GraphCanvasLocal<Id>, id: Option<Id>| local.focus = id,
                {
                    let lift = lift.clone();
                    move |_: &mut GraphCanvasLocal<Id>, drag: GraphCanvasNodeDrag<Id>| {
                        lift(GraphCanvasEvent::Drag(drag))
                    }
                },
                {
                    let lift = lift.clone();
                    move |_: &mut GraphCanvasLocal<Id>, id: String| {
                        lift(GraphCanvasEvent::RelationActivate(id))
                    }
                },
                |local: &mut GraphCanvasLocal<Id>, id: Option<String>| {
                    local.hovered_relation = id;
                },
                {
                    let lift = lift.clone();
                    move |_: &mut GraphCanvasLocal<Id>| lift(GraphCanvasEvent::Expand)
                },
                {
                    let lift = lift.clone();
                    let width = (props.width as f32).max(1.0);
                    let height = (props.height as f32).max(1.0);
                    move |local: &mut GraphCanvasLocal<Id>, event: PointerEvent| match event.phase {
                        PointerPhase::Down => {
                            local.pan_anchor = Some(event.local);
                            None
                        },
                        PointerPhase::Move => {
                            let Some(anchor) = local.pan_anchor else {
                                return None;
                            };
                            let delta = (event.local.0 - anchor.0, event.local.1 - anchor.1);
                            local.pan_anchor = Some(event.local);
                            if delta == (0.0, 0.0) {
                                return None;
                            }
                            Some(lift(GraphCanvasEvent::Pan {
                                delta: (delta.0 / width, delta.1 / height),
                            }))
                        },
                        PointerPhase::Up | PointerPhase::Cancel => {
                            local.pan_anchor = None;
                            None
                        },
                    }
                },
                move |_: &mut GraphCanvasLocal<Id>, event: WheelEvent| wheel(event),
            )) as ComponentView<GraphCanvasLocal<Id>, E>
        },
        on_event,
    )
    .memo()
}

/// An atlas-specific swatch interaction surface.
///
/// Geographic field buttons exist solely for keyboard focus and activation.
/// They deliberately have `pointer-events:none`: the full-size surface below
/// is the only pointer route, so a field's rectangular semantic box can never
/// turn an empty polygon corner into a hit. Pointer capture preserves the
/// selected field across a drag even after the cursor leaves its footprint.
pub fn graph_atlas_swatch<State, A, Output, Id, Kind, F>(
    swatch: &GraphCanvasSwatch<Id, Kind>,
    on_event: F,
) -> impl View<State, A, GenetCtx, Element = GenetElement> + use<State, A, Output, Id, Kind, F>
where
    State: 'static,
    A: 'static,
    Output: OptionalAction<A> + 'static,
    Id: Clone + PartialEq + 'static,
    Kind: Clone + PartialEq + 'static,
    F: Fn(&mut State, GraphAtlasEvent<Id>) -> Output + 'static,
{
    component(
        swatch.clone(),
        |_props: &GraphCanvasSwatch<Id, Kind>| AtlasCanvasLocal {
            captured: None,
            down: None,
        },
        |_prev, _next, _local| {},
        |props: &GraphCanvasSwatch<Id, Kind>, _local: &AtlasCanvasLocal<Id>| {
            let atlas = props.atlas.as_ref().expect("graph_atlas_swatch requires swatch.atlas");
            let view = atlas.view(props.viewport, props.width as f32, props.height as f32);
            let field_buttons: Vec<_> = atlas
                .focus_order()
                .into_iter()
                .filter_map(|field| {
                    let bounds = field.footprint.bounds()?;
                    let top_left = view.project(Vec2::new(
                        field.anchor.x + bounds.origin.x,
                        field.anchor.y + bounds.origin.y,
                    ));
                    let bottom_right = view.project(Vec2::new(
                        field.anchor.x + bounds.origin.x + bounds.size.w,
                        field.anchor.y + bounds.origin.y + bounds.size.h,
                    ));
                    let id = field.id.clone();
                    let focus_id = id.clone();
                    Some(on_focus(focusable(on_click(
                        el::<_, AtlasCanvasLocal<Id>, GraphAtlasEvent<Id>>("button", ())
                            .attr("type", "button")
                            .attr("class", "graph-canvas-atlas-field")
                            .attr("aria-label", field.label.clone())
                            .attr(
                                "style",
                                format!(
                                    "position:absolute;pointer-events:none;background:transparent;border:0;padding:0;left:{}px;top:{}px;width:{}px;height:{}px;",
                                    top_left.0.min(bottom_right.0),
                                    top_left.1.min(bottom_right.1),
                                    (bottom_right.0 - top_left.0).abs().max(1.0),
                                    (bottom_right.1 - top_left.1).abs().max(1.0),
                                ),
                            ),
                        move |_: &mut AtlasCanvasLocal<Id>, _: PointerClick| {
                            GraphAtlasEvent::Activate(id.clone())
                        },
                    )), move |_: &mut AtlasCanvasLocal<Id>, event: FocusEvent| {
                        GraphAtlasEvent::Focus(if matches!(event.phase, FocusPhase::Gained) { Some(focus_id.clone()) } else { None })
                    }))
                })
                .collect();
            let atlas_for_pointer = atlas.clone();
            let atlas_for_hover = atlas.clone();
            let viewport = props.viewport;
            let width = props.width;
            let height = props.height;
            let defer_drag = props.defer_drag_rebuild;
            let root = el(
                "div",
                (
                    custom_leaf::<AtlasCanvasLocal<Id>, GraphAtlasEvent<Id>>(
                        props.leaf_key,
                        width,
                        height,
                    )
                    .attr("aria-hidden", "true"),
                    el("div", field_buttons).attr(
                        "style",
                        format!(
                            "position:absolute;left:0;top:0;width:{width}px;height:{height}px;"
                        ),
                    ),
                ),
            )
            .attr("class", "graph-canvas-swatch graph-canvas-atlas-swatch")
            .attr("role", "group")
            .attr("aria-label", props.label.clone())
            .attr(
                "style",
                format!(
                    "position:relative;display:block;overflow:hidden;width:{width}px;height:{height}px;max-width:100%;"
                ),
            );
            Box::new(on_hover(on_pointer(root, move |local: &mut AtlasCanvasLocal<Id>, event: PointerEvent| {
                if event.button != crate::PointerButton::Primary { return None; }
                if defer_drag && matches!(event.phase, PointerPhase::Move) {
                    event.defer_rebuild();
                }
                let view = atlas_for_pointer.view(viewport, width as f32, height as f32);
                let position = view.unproject_normalized(event.local);
                match event.phase {
                    PointerPhase::Down => {
                        local.down = Some(event.local);
                        local.captured = atlas_for_pointer.pick(view, event.local).map(|field| field.id.clone());
                    }
                    PointerPhase::Move => {}
                    PointerPhase::Up => {}
                    PointerPhase::Cancel => { local.captured = None; local.down = None; return None; }
                }
                let Some(id) = local.captured.clone() else { return None; };
                let Some(position) = position else { return None; };
                let drag = GraphAtlasEvent::Drag(GraphCanvasNodeDrag { id: id.clone(), phase: event.phase, position });
                if matches!(event.phase, PointerPhase::Up) {
                    local.captured = None;
                    local.down = None;
                }
                Some(drag)
            }), move |_: &mut AtlasCanvasLocal<Id>, event: HoverEvent| {
                if defer_drag { event.defer_rebuild(); }
                let picked = if matches!(event.phase, HoverPhase::Leave) { None } else {
                    atlas_for_hover.pick(atlas_for_hover.view(viewport, width as f32, height as f32), event.local).map(|field| field.id.clone())
                };
                Some(GraphAtlasEvent::Hover(picked))
            })) as ComponentView<AtlasCanvasLocal<Id>, GraphAtlasEvent<Id>>
        },
        on_event,
    )
    .memo()
}

struct AtlasCanvasLocal<Id> {
    captured: Option<Id>,
    down: Option<(f32, f32)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AnyView, DomHandle, GenetAppRunner};
    use genet_scripted_dom::{NodeId, ScriptedDom};
    use layout_dom_api::{LayoutDom, LocalName, Namespace};
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    #[derive(Default)]
    struct State {
        clicked: Vec<u8>,
        hovered: Option<u8>,
        focused: Option<u8>,
        dragged: Vec<GraphCanvasNodeDrag<u8>>,
        relation_clicked: Vec<String>,
        relation_hovered: Option<String>,
        expanded: bool,
        focus_request: Option<u8>,
    }

    type TestView = Box<dyn AnyView<State, (), GenetCtx, GenetElement>>;

    fn model(hovered: Option<u8>, focused: Option<u8>) -> GraphCanvasSwatch<u8, &'static str> {
        let mut swatch = GraphCanvasSwatch::new(
            77,
            GraphCanvasSubgraph {
                nodes: vec![
                    GraphCanvasNode {
                        id: 1,
                        kind: "document",
                        position: (0.1, 0.5),
                        label: "First node".into(),
                        key: None,
                    },
                    GraphCanvasNode {
                        id: 2,
                        kind: "person",
                        position: (0.9, 0.5),
                        label: "Second node".into(),
                        key: None,
                    },
                ],
                edges: vec![GraphCanvasEdge { from: 1, to: 2 }],
            },
        )
        .with_size(240, 112);
        swatch.selected = Some(1);
        swatch.focus = focused;
        swatch.hovered = hovered;
        swatch
    }

    fn view(state: &State) -> TestView {
        let swatch = model(state.hovered, state.focused);
        Box::new(graph_canvas_swatch_with_focus(
            &swatch,
            |state: &mut State, id| state.clicked.push(id),
            |state: &mut State, id| state.hovered = id,
            |state: &mut State, id| state.focused = id,
            |state: &mut State| state.expanded = true,
        ))
    }

    #[derive(Default)]
    struct ViewportState {
        pans: Vec<(f32, f32)>,
        zooms: Vec<f32>,
    }

    type ViewportView = Box<dyn AnyView<ViewportState, (), GenetCtx, GenetElement>>;

    fn viewport_view(_state: &ViewportState) -> ViewportView {
        let swatch = model(None, None);
        Box::new(graph_canvas(
            &swatch,
            |state: &mut ViewportState, event: GraphCanvasEvent<u8>| match event {
                GraphCanvasEvent::Pan { delta } => state.pans.push(delta),
                GraphCanvasEvent::Zoom { factor } => state.zooms.push(factor),
                _ => {},
            },
        ))
    }

    /// The viewport stays consumer state: a background drag reports pan deltas
    /// in canvas fractions, a wheel notch reports a zoom factor, and neither
    /// mutates the swatch.
    #[test]
    fn background_drag_pans_and_wheel_zooms() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::<_, _, _, ()>::new(
            dom.clone(),
            viewport_view,
            ViewportState::default(),
        );
        let root = runner.root();
        let canvas = find_attr(&dom.borrow(), root, "class", "graph-canvas-swatch")
            .expect("canvas root element");

        let size = (240.0, 112.0);
        runner.dispatch_pointer_down(
            canvas,
            PointerEvent::new(PointerPhase::Down, (10.0, 10.0), size),
        );
        runner.dispatch_pointer_move(PointerEvent::new(PointerPhase::Move, (34.0, 24.0), size));
        runner.dispatch_pointer_up(PointerEvent::new(PointerPhase::Up, (34.0, 24.0), size));
        assert_eq!(runner.state().pans, vec![(24.0 / 240.0, 14.0 / 112.0)]);

        runner.dispatch_wheel(canvas, WheelEvent::new((0.0, 3.0), (5.0, 5.0), size));
        runner.dispatch_wheel(canvas, WheelEvent::new((0.0, -3.0), (5.0, 5.0), size));
        assert_eq!(runner.state().zooms, vec![1.2, 1.0 / 1.2]);
    }

    #[derive(Default)]
    struct WheelState {
        wheels: Vec<((f32, f32), (f32, f32), (f32, f32), bool, bool)>,
        others: usize,
        take: bool,
    }

    type WheelView = Box<dyn AnyView<WheelState, (), GenetCtx, GenetElement>>;

    fn wheel_view(state: &WheelState) -> WheelView {
        let swatch = model(None, None);
        let _ = state;
        Box::new(graph_canvas_with_wheel(
            &swatch,
            |state: &mut WheelState, input: GraphCanvasInput<u8>| match input {
                GraphCanvasInput::Wheel(event) => {
                    state.wheels.push((
                        event.delta,
                        event.local,
                        event.size,
                        event.modifiers.ctrl,
                        event.modifiers.meta,
                    ));
                    if state.take {
                        event.prevent_default();
                    }
                },
                GraphCanvasInput::Event(_) => state.others += 1,
            },
        ))
    }

    /// A consumer that opts in sees the whole wheel notch (both axes, where the
    /// pointer was, the modifiers) and decides whether the host's scroll default
    /// runs. A wheel that is not asked for as a zoom is not turned into one.
    #[test]
    fn the_wheel_reaches_an_opting_in_consumer_whole() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner =
            GenetAppRunner::<_, _, _, ()>::new(dom.clone(), wheel_view, WheelState::default());
        let root = runner.root();
        let canvas = find_attr(&dom.borrow(), root, "class", "graph-canvas-swatch")
            .expect("canvas root element");
        let size = (240.0, 112.0);

        let mut event = WheelEvent::new((7.0, -3.0), (100.0, 40.0), size);
        event.modifiers.ctrl = true;
        runner.dispatch_wheel(canvas, event);
        assert_eq!(
            runner.state().wheels,
            vec![((7.0, -3.0), (100.0, 40.0), size, true, false)]
        );
        assert!(!runner.default_prevented(), "the consumer did not take it");

        runner.update(|state| state.take = true);
        runner.dispatch_wheel(canvas, WheelEvent::new((0.0, 0.0), (1.0, 2.0), size));
        assert_eq!(
            runner.state().wheels.len(),
            2,
            "a zero notch is still reported"
        );
        assert!(runner.default_prevented(), "the consumer took it");
        assert_eq!(runner.state().others, 0);
    }

    /// The established form is unchanged: a notch is a fixed zoom factor, the
    /// host's scroll default is suppressed, and a zero notch reports nothing.
    #[test]
    fn the_established_wheel_zoom_is_unchanged() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::<_, _, _, ()>::new(
            dom.clone(),
            viewport_view,
            ViewportState::default(),
        );
        let root = runner.root();
        let canvas = find_attr(&dom.borrow(), root, "class", "graph-canvas-swatch")
            .expect("canvas root element");
        let size = (240.0, 112.0);
        runner.dispatch_wheel(canvas, WheelEvent::new((9.0, 3.0), (5.0, 5.0), size));
        assert!(runner.default_prevented());
        runner.dispatch_wheel(canvas, WheelEvent::new((9.0, 0.0), (5.0, 5.0), size));
        assert!(
            !runner.default_prevented(),
            "a zero notch is left to the host"
        );
        assert_eq!(runner.state().zooms, vec![1.2]);
    }

    fn attr<'a>(dom: &'a ScriptedDom, node: NodeId, name: &str) -> Option<&'a str> {
        dom.attribute(node, &Namespace::from(""), &LocalName::from(name))
    }

    fn find_attr(dom: &ScriptedDom, root: NodeId, name: &str, value: &str) -> Option<NodeId> {
        if attr(dom, root, name) == Some(value) {
            return Some(root);
        }
        dom.dom_children(root)
            .find_map(|child| find_attr(dom, child, name, value))
    }

    #[test]
    fn node_targets_route_click_hover_and_expand() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::<_, _, _, ()>::new(dom.clone(), view, State::default());
        let root = runner.root();
        let first =
            find_attr(&dom.borrow(), root, "aria-label", "First node").expect("first node target");
        runner.dispatch_hover(
            first,
            HoverEvent::new(HoverPhase::Enter, (2.0, 2.0), (20.0, 20.0)),
        );
        assert_eq!(runner.state().hovered, Some(1));
        runner.dispatch_click(first, PointerClick::at((2.0, 2.0)));
        assert_eq!(runner.state().clicked, [1]);
        assert_eq!(runner.focus(), Some(first));
        assert_eq!(runner.state().focused, Some(1));

        let expand =
            find_attr(&dom.borrow(), root, "aria-label", "Expand graph").expect("expand target");
        runner.dispatch_click(expand, PointerClick::at((2.0, 2.0)));
        assert!(runner.state().expanded);
    }

    fn attributed_view(state: &State) -> TestView {
        let swatch = model(state.hovered, state.focused)
            .with_node_attr(2, "aria-expanded", "false")
            .with_node_attr(2, "aria-expanded", "true")
            .with_node_attr(2, "data-tier", "session")
            .with_node_attr(2, "aria-label", "Not the canvas's name")
            .with_focus_request(state.focus_request);
        Box::new(graph_canvas_swatch_with_focus(
            &swatch,
            |state: &mut State, id| state.clicked.push(id),
            |state: &mut State, id| state.hovered = id,
            |state: &mut State, id| state.focused = id,
            |state: &mut State| state.expanded = true,
        ))
    }

    /// A consumer's node kind rides on the shared target as attributes, and a
    /// requested node takes focus once, without the canvas's own attributes
    /// being overridden.
    #[test]
    fn node_attrs_and_a_focus_request_reach_one_nodes_target() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner =
            GenetAppRunner::<_, _, _, ()>::new(dom.clone(), attributed_view, State::default());
        let root = runner.root();
        let second = find_attr(&dom.borrow(), root, "aria-label", "Second node")
            .expect("the canvas keeps its own accessible name");
        let first = find_attr(&dom.borrow(), root, "aria-label", "First node").unwrap();
        {
            let dom = dom.borrow();
            assert_eq!(attr(&dom, second, "aria-expanded"), Some("true"));
            assert_eq!(attr(&dom, second, "data-tier"), Some("session"));
            assert_eq!(attr(&dom, first, "aria-expanded"), None);
        }
        assert_eq!(runner.focus(), None);
        runner.update(|state| state.focus_request = Some(2));
        let second = find_attr(&dom.borrow(), root, "aria-label", "Second node").unwrap();
        assert_eq!(runner.focus(), Some(second), "the request takes focus");
        let first = find_attr(&dom.borrow(), root, "aria-label", "First node").unwrap();
        runner.set_focus(Some(first));
        runner.update(|state| state.hovered = Some(1));
        assert_eq!(
            runner.focus(),
            Some(first),
            "a held request does not steal focus again"
        );
    }

    fn labelled_view(state: &State) -> TestView {
        let swatch = model(state.hovered, state.focused).with_node_labels(true);
        Box::new(graph_canvas_swatch_with_focus(
            &swatch,
            |state: &mut State, id| state.clicked.push(id),
            |state: &mut State, id| state.hovered = id,
            |state: &mut State, id| state.focused = id,
            |state: &mut State| state.expanded = true,
        ))
    }

    /// Collect the class attribute of every element whose *base* class is
    /// `base`. Matching the first token rather than a prefix keeps the
    /// `-labels` container out of a search for `-label` elements.
    fn classes_of(dom: &ScriptedDom, root: NodeId, base: &str, out: &mut Vec<String>) {
        if let Some(class) = attr(dom, root, "class") {
            if class.split_whitespace().next() == Some(base) {
                out.push(class.to_string());
            }
        }
        for child in dom.dom_children(root) {
            classes_of(dom, child, base, out);
        }
    }

    fn short_labelled_view(_state: &State) -> TestView {
        let swatch = model(None, None)
            .with_node_labels(true)
            .with_visible_label(2, "Second")
            .with_label_placement(1, GraphLabelPlacement::Left);
        Box::new(graph_canvas_swatch_with_focus(
            &swatch,
            |state: &mut State, id| state.clicked.push(id),
            |state: &mut State, id| state.hovered = id,
            |state: &mut State, id| state.focused = id,
            |state: &mut State| state.expanded = true,
        ))
    }

    /// A node's visible label can say less than its accessible name, and a
    /// host can put a label on the side it needs.
    #[test]
    fn a_visible_label_can_be_shorter_and_placed_by_the_host() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let runner =
            GenetAppRunner::<_, _, _, ()>::new(dom.clone(), short_labelled_view, State::default());
        let root = runner.root();
        let dom = dom.borrow();
        let second = find_attr(&dom, root, "aria-label", "Second node")
            .expect("the accessible name stays whole");
        assert_eq!(attr(&dom, second, "aria-label"), Some("Second node"));
        let left = find_attr(&dom, root, "data-label-placement", "left")
            .expect("node 1's label goes left as asked");
        assert_eq!(
            dom.text(dom.dom_children(left).next().unwrap()),
            Some("First node")
        );
        let mut labels = Vec::new();
        classes_of(&dom, root, "graph-canvas-swatch-label", &mut labels);
        assert_eq!(labels.len(), 2);
        let texts: Vec<String> = {
            fn walk(dom: &ScriptedDom, node: NodeId, out: &mut Vec<String>) {
                if attr(dom, node, "class")
                    .is_some_and(|c| c.starts_with("graph-canvas-swatch-label "))
                    || attr(dom, node, "class") == Some("graph-canvas-swatch-label")
                {
                    let text = dom
                        .dom_children(node)
                        .filter_map(|child| dom.text(child).map(str::to_string))
                        .collect::<String>();
                    out.push(text);
                }
                for child in dom.dom_children(node).collect::<Vec<_>>() {
                    walk(dom, child, out);
                }
            }
            let mut out = Vec::new();
            walk(&dom, root, &mut out);
            out
        };
        assert!(texts.contains(&"Second".to_string()), "{texts:?}");
        assert!(!texts.contains(&"Second node".to_string()), "{texts:?}");
    }

    /// A long label is cut in its middle, keeping its start and the end that
    /// tells it apart, at word boundaries; a short one is left whole.
    #[test]
    fn a_long_label_is_cut_in_the_middle() {
        let fork = "Session (fork of Session (from blake3:f612a))";
        let cut = middle_cut(fork, GRAPH_LABEL_CHARS);
        assert!(cut.chars().count() <= GRAPH_LABEL_CHARS, "{cut}");
        assert!(cut.starts_with("Session"), "{cut}");
        assert_eq!(cut, "Session (fork of … blake3:f612a))");
        let picked = middle_cut(
            "Session (picked from Evening reading (from blake3:f612a))",
            GRAPH_LABEL_CHARS,
        );
        assert!(picked.starts_with("Session (picked"), "{picked}");
        assert!(picked.ends_with("blake3:f612a))"), "{picked}");
        assert!(picked.chars().count() <= GRAPH_LABEL_CHARS, "{picked}");
        assert!(cut.contains(" … "), "{cut}");
        let other = middle_cut(
            "Session (fork of Session (from blake3:0c0de))",
            GRAPH_LABEL_CHARS,
        );
        assert_ne!(cut, other, "names that differ at the end stay apart");
        assert_eq!(
            middle_cut("Evening reading", GRAPH_LABEL_CHARS),
            "Evening reading"
        );
        assert_eq!(
            middle_cut("abcdefghijklmnopqrstuvwxyz0123456789", 10)
                .chars()
                .count(),
            10,
            "a name without spaces is cut at the budget"
        );
        // The canvas shows the cut and keeps the whole accessible name.
        let swatch = model(None, None)
            .with_node_labels(true)
            .with_visible_label(2, fork);
        assert_eq!(swatch.label_text(1), cut);
        assert_eq!(swatch.graph.nodes[1].label, "Second node");
    }

    /// A node an overlay occupies shows the overlay, not its label too, with
    /// and without culling.
    #[test]
    fn an_occupied_node_draws_no_label() {
        for cull in [false, true] {
            let swatch = model(None, None)
                .with_node_labels(true)
                .with_label_culling(cull)
                .with_node_footprint(1, 120.0, 60.0);
            assert_eq!(swatch.shown_labels(), [false, true], "cull {cull}");
        }
    }

    /// Visible labels carry the node's state, not just its text.
    ///
    /// Without this a consumer that needs to emphasize one label (which node is
    /// "here") has to hand-roll the entire label layer to get it, which defeats
    /// the point of the component rendering labels at all.
    #[test]
    fn visible_labels_carry_node_state() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let state = State {
            hovered: Some(2),
            ..State::default()
        };
        let runner = GenetAppRunner::<_, _, _, ()>::new(dom.clone(), labelled_view, state);
        let root = runner.root();
        let mut found = Vec::new();
        classes_of(&dom.borrow(), root, "graph-canvas-swatch-label", &mut found);

        assert_eq!(found.len(), 2, "one label per node: {found:?}");
        // `model` selects node 1; the test state hovers node 2.
        assert!(
            found
                .iter()
                .any(|c| c == "graph-canvas-swatch-label selected"),
            "the selected node's label must say so: {found:?}"
        );
        assert!(
            found
                .iter()
                .any(|c| c == "graph-canvas-swatch-label hovered"),
            "the hovered node's label must say so: {found:?}"
        );

        let swatch = model(None, None).with_node_labels(true);
        assert_eq!(swatch.label_placement(0), GraphLabelPlacement::Above);
        assert_eq!(swatch.label_placement(1), GraphLabelPlacement::Above);

        // On a narrow canvas a node two thirds across has no room for a whole
        // label on its right, and more on its left.
        let lone = |x: f32| {
            GraphCanvasSwatch::new(
                7,
                GraphCanvasSubgraph {
                    nodes: vec![GraphCanvasNode {
                        id: 1,
                        kind: "document",
                        position: (x, 0.5),
                        label: "Archive 2025, unavailable".into(),
                        key: None,
                    }],
                    edges: Vec::new(),
                },
            )
            .with_size(280, 400)
        };
        assert_eq!(lone(0.68).label_placement(0), GraphLabelPlacement::Left);
        assert_eq!(lone(0.30).label_placement(0), GraphLabelPlacement::Right);
        // A left label is held by its right edge beside the node, so it hugs
        // the node whatever width its text takes.
        let size = Size {
            width: 280.0,
            height: 400.0,
        };
        let style = GraphLabelPlacement::Left.style(190.0, 200.0, 12.0, size, 100.0);
        assert!(style.contains("right:102px"), "{style}");
        assert!(!style.contains("left:"), "{style}");

        // Three nodes stacked closer than a label's height: culling keeps the
        // first, and a focused node keeps its label whatever it covers.
        let crowded = GraphCanvasSwatch::new(
            8,
            GraphCanvasSubgraph {
                nodes: (1..=3)
                    .map(|id| GraphCanvasNode {
                        id,
                        kind: "document",
                        position: (0.3, 0.5 + id as f32 * 0.01),
                        label: format!("A long label for node {id}"),
                        key: None,
                    })
                    .collect(),
                edges: Vec::new(),
            },
        )
        .with_size(280, 400)
        .with_node_labels(true);
        assert_eq!(crowded.shown_labels(), [true, true, true], "culling is off");
        let culled = crowded.with_label_culling(true);
        assert_eq!(culled.shown_labels(), [true, false, false]);
        let mut focused = culled;
        focused.focus = Some(3);
        assert_eq!(focused.shown_labels(), [false, false, true]);
        assert!(
            find_attr(&dom.borrow(), root, "data-label-placement", "above").is_some(),
            "a horizontal graph publishes its label-clear placement"
        );
    }

    #[test]
    fn paint_leaf_and_hit_targets_share_projection() {
        let swatch = model(Some(1), Some(2));
        let leaf = swatch.paint_leaf(|kind| match *kind {
            "document" => ColorF {
                r: 0.2,
                g: 0.4,
                b: 0.8,
                a: 1.0,
            },
            _ => ColorF {
                r: 0.7,
                g: 0.4,
                b: 0.3,
                a: 1.0,
            },
        });
        let projected = swatch.projected_positions();
        assert_eq!(
            leaf.node_local_position(
                0,
                Size {
                    width: 240.0,
                    height: 112.0
                }
            ),
            Some(projected[0].1)
        );
    }

    #[test]
    fn node_regions_share_projection_and_clamp_inside_the_canvas() {
        let swatch = model(None, None);
        let projected = swatch.projected_positions();
        let region = swatch
            .projected_node_region(&1, 120.0, 80.0)
            .expect("node region");
        let center = projected[0].1;
        assert!(center.0 >= region.left && center.0 <= region.left + region.width);
        assert!(center.1 >= region.top && center.1 <= region.top + region.height);
        assert!(region.left >= 0.0 && region.top >= 0.0);
        assert!(region.left + region.width <= swatch.width as f32);
        assert!(region.top + region.height <= swatch.height as f32);

        let full = swatch
            .projected_node_region(&2, 10_000.0, 10_000.0)
            .expect("oversized region");
        assert_eq!(
            full,
            GraphCanvasNodeRegion {
                left: 0.0,
                top: 0.0,
                width: swatch.width as f32,
                height: swatch.height as f32,
            }
        );
        assert!(swatch.projected_node_region(&99, 10.0, 10.0).is_none());
    }

    #[test]
    fn node_footprints_clip_incident_relation_routes_to_the_card_perimeter() {
        let relations = vec![
            GraphCanvasRelation {
                link_key: None,
                marks: sceno::LinkMarks::default(),
                strength: 1.0,
                id: "outgoing".into(),
                from: 1,
                to: 2,
                kind: "First".into(),
                label: "Outgoing relation".into(),
                route: Vec::new(),
                visible: true,
                emphasized: false,
            },
            GraphCanvasRelation {
                link_key: None,
                marks: sceno::LinkMarks::default(),
                strength: 1.0,
                id: "incoming".into(),
                from: 2,
                to: 1,
                kind: "Second".into(),
                label: "Incoming relation".into(),
                route: Vec::new(),
                visible: true,
                emphasized: false,
            },
        ];
        let compact = model(None, None).with_relations(relations.clone());
        let compact_routes = compact.projected_relations();
        let swatch = model(None, None)
            .with_relations(relations)
            .with_node_footprint(1, 80.0, 60.0);
        let region = swatch
            .projected_node_footprint(&1)
            .expect("declared footprint");
        let routes = swatch.projected_relations();
        let on_perimeter = |point: (f32, f32)| {
            let right = region.left + region.width;
            let bottom = region.top + region.height;
            let on_vertical =
                (point.0 - region.left).abs() < 0.01 || (point.0 - right).abs() < 0.01;
            let on_horizontal =
                (point.1 - region.top).abs() < 0.01 || (point.1 - bottom).abs() < 0.01;
            (on_vertical && point.1 >= region.top && point.1 <= bottom)
                || (on_horizontal && point.0 >= region.left && point.0 <= right)
        };

        assert_eq!(routes.len(), 2);
        assert!(on_perimeter(routes[0].1[0]), "outgoing source is clipped");
        assert!(
            on_perimeter(*routes[1].1.last().expect("incoming endpoint")),
            "incoming target is clipped"
        );
        assert_eq!(
            *routes[0].1.last().expect("unoccupied target"),
            *compact_routes[0].1.last().expect("compact target")
        );
        assert_eq!(routes[1].1[0], compact_routes[1].1[0]);
    }

    #[test]
    fn graph_position_inverts_the_viewport_projection() {
        let mut swatch = model(None, None);
        swatch.viewport = GraphViewport {
            pan: (0.12, -0.08),
            zoom: 1.35,
        };
        let point = (0.73, 0.21);
        let size = Size {
            width: swatch.width as f32,
            height: swatch.height as f32,
        };
        let leaf = swatch
            .viewport
            .project(point, size, swatch.node_radius + swatch.edge_width);
        let recovered = swatch.graph_position_at(leaf);
        assert!((recovered.0 - point.0).abs() < 1e-5, "{recovered:?}");
        assert!((recovered.1 - point.1).abs() < 1e-5, "{recovered:?}");
    }

    fn drag_view(state: &State) -> TestView {
        let swatch = model(state.hovered, state.focused);
        Box::new(graph_canvas_swatch_with_drag(
            &swatch,
            |state: &mut State, id| state.clicked.push(id),
            |state: &mut State, id| state.hovered = id,
            |state: &mut State, event| state.dragged.push(event),
            |state: &mut State| state.expanded = true,
        ))
    }

    fn relation_view(state: &State) -> TestView {
        let swatch = model(state.hovered, state.focused).with_relations(vec![
            GraphCanvasRelation {
                link_key: None,
                marks: sceno::LinkMarks::default(),
                strength: 1.0,
                id: "cites".into(),
                from: 1,
                to: 2,
                kind: "Citation".into(),
                label: "First node cites Second node".into(),
                route: Vec::new(),
                visible: true,
                emphasized: true,
            },
            GraphCanvasRelation {
                link_key: None,
                marks: sceno::LinkMarks::default(),
                strength: 1.0,
                id: "quotes".into(),
                from: 1,
                to: 2,
                kind: "Quotation".into(),
                label: "First node quotes Second node".into(),
                route: Vec::new(),
                visible: true,
                emphasized: false,
            },
            GraphCanvasRelation {
                link_key: None,
                marks: sceno::LinkMarks::default(),
                strength: 1.0,
                id: "hidden".into(),
                from: 1,
                to: 2,
                kind: "Hidden".into(),
                label: "This relation is hidden".into(),
                route: Vec::new(),
                visible: false,
                emphasized: false,
            },
        ]);
        Box::new(graph_canvas_swatch_with_drag_and_relations(
            &swatch,
            |state: &mut State, id| state.clicked.push(id),
            |state: &mut State, id| state.hovered = id,
            |state: &mut State, event| state.dragged.push(event),
            |state: &mut State, id| state.relation_clicked.push(id),
            |state: &mut State, id| state.relation_hovered = id,
            |state: &mut State| state.expanded = true,
        ))
    }

    #[test]
    fn unbound_routes_keep_independent_targets_and_visibility() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner =
            GenetAppRunner::<_, _, _, ()>::new(dom.clone(), relation_view, State::default());
        let root = runner.root();
        let cites = find_attr(&dom.borrow(), root, "data-relation-id", "cites")
            .expect("citation relation target");
        let quotes = find_attr(&dom.borrow(), root, "data-relation-id", "quotes")
            .expect("quotation relation target");
        assert_ne!(cites, quotes, "parallel cells must get separate targets");
        assert!(
            find_attr(&dom.borrow(), root, "data-relation-id", "hidden").is_none(),
            "hidden cells must not remain hittable"
        );
        assert_eq!(attr(&dom.borrow(), cites, "aria-current"), Some("true"));
        assert_eq!(attr(&dom.borrow(), cites, "data-kind"), Some("Citation"));
        assert_eq!(attr(&dom.borrow(), quotes, "data-kind"), Some("Quotation"));

        runner.dispatch_hover(
            quotes,
            HoverEvent::new(HoverPhase::Enter, (2.0, 2.0), (20.0, 20.0)),
        );
        runner.dispatch_click(cites, PointerClick::at((2.0, 2.0)));
        assert_eq!(runner.state().relation_hovered.as_deref(), Some("quotes"));
        assert_eq!(runner.state().relation_clicked, ["cites"]);

        let swatch = model(None, None).with_relations(vec![GraphCanvasRelation {
            link_key: None,
            marks: sceno::LinkMarks::default(),
            strength: 1.0,
            id: "hidden".into(),
            from: 1,
            to: 2,
            kind: "Hidden".into(),
            label: "This relation is hidden".into(),
            route: Vec::new(),
            visible: false,
            emphasized: false,
        }]);
        assert!(swatch.projected_relations().is_empty());
    }

    #[test]
    fn one_link_target_keeps_both_directions_opposition_and_strength_without_fanning() {
        let key = sceno::LinkKey::new(sceno::LinkStratum::Surface, "1", "2");
        let relation = |id: &str, from, to, marks| GraphCanvasRelation {
            id: id.into(),
            link_key: Some(key.clone()),
            marks,
            strength: 1.0,
            from,
            to,
            kind: "reason".into(),
            label: id.into(),
            route: Vec::new(),
            visible: true,
            emphasized: false,
        };
        let swatch = model(None, None).with_relations(vec![
            relation(
                "one",
                1,
                2,
                sceno::LinkMarks {
                    forward: true,
                    reverse: false,
                    opposed: true,
                },
            ),
            relation(
                "two",
                2,
                1,
                sceno::LinkMarks {
                    forward: true,
                    reverse: false,
                    opposed: false,
                },
            ),
        ]);
        assert_eq!(swatch.relations.len(), 1);
        let (link, route) = &swatch.projected_relations()[0];
        assert_eq!(link.id, key.stable_id());
        assert_eq!(
            link.marks,
            sceno::LinkMarks {
                forward: true,
                reverse: true,
                opposed: true
            }
        );
        assert_eq!(link.strength, 2.0);
        assert_eq!(
            route.len(),
            2,
            "assertion count does not bend the resting path"
        );
    }

    #[test]
    fn stratum_strokes_remain_separate_without_assertion_lanes() {
        let relation = |stratum| GraphCanvasRelation {
            id: "ignored".into(),
            link_key: Some(sceno::LinkKey::new(stratum, "1", "2")),
            marks: sceno::LinkMarks {
                forward: true,
                ..Default::default()
            },
            strength: 1.0,
            from: 1,
            to: 2,
            kind: "link".into(),
            label: "source pair".into(),
            route: Vec::new(),
            visible: true,
            emphasized: false,
        };
        let swatch = model(None, None).with_relations(vec![
            relation(sceno::LinkStratum::Surface),
            relation(sceno::LinkStratum::Resource),
        ]);
        let routes = swatch.projected_relations();
        assert_eq!(routes.len(), 2);
        assert_ne!(routes[0].0.id, routes[1].0.id);
        assert!(
            (routes[0].1[1].1 - routes[1].1[1].1).abs() > 5.9,
            "both stratum strokes have distinct paint and pick paths"
        );
        let mut hidden = swatch.clone();
        hidden.relations[1].visible = false;
        assert_eq!(hidden.projected_relations().len(), 1);
        assert_eq!(hidden.projected_relations()[0].1.len(), 2);
    }

    #[test]
    fn node_drag_captures_and_reports_normalized_graph_positions() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner =
            GenetAppRunner::<_, _, _, ()>::new(dom.clone(), drag_view, State::default());
        let root = runner.root();
        let first =
            find_attr(&dom.borrow(), root, "aria-label", "First node").expect("first node target");

        runner.dispatch_pointer_down(
            first,
            PointerEvent::new(PointerPhase::Down, (10.0, 10.0), (20.0, 20.0)),
        );
        assert_eq!(runner.pointer_capture(), Some(first));
        runner.dispatch_pointer_move(PointerEvent::new(
            PointerPhase::Move,
            (120.0, 10.0),
            (20.0, 20.0),
        ));
        runner.dispatch_pointer_up(PointerEvent::new(
            PointerPhase::Up,
            (120.0, 10.0),
            (20.0, 20.0),
        ));

        let dragged = &runner.state().dragged;
        assert_eq!(
            dragged.iter().map(|event| event.phase).collect::<Vec<_>>(),
            vec![PointerPhase::Down, PointerPhase::Move, PointerPhase::Up]
        );
        assert!(
            dragged[1].position.0 > dragged[0].position.0,
            "a rightward drag advances the normalized x position: {dragged:?}"
        );
        assert!(
            dragged.iter().all(|event| event.id == 1
                && (0.0..=1.0).contains(&event.position.0)
                && (0.0..=1.0).contains(&event.position.1)),
            "every emitted event keeps the node id and clamps graph coordinates: {dragged:?}"
        );
        assert_eq!(runner.pointer_capture(), None, "Up releases capture");
    }

    /// The component form's app state: selection truth in, events out. There
    /// is deliberately no hover or focus field here — that is the whole point.
    #[derive(Default)]
    struct AppState {
        selected: Option<u8>,
        events: Vec<GraphCanvasEvent<u8>>,
    }

    type AppView = Box<dyn AnyView<AppState, (), GenetCtx, GenetElement>>;

    fn component_view(state: &AppState) -> AppView {
        let mut swatch = model(None, None);
        swatch.selected = state.selected;
        Box::new(graph_canvas(
            &swatch,
            |state: &mut AppState, event: GraphCanvasEvent<u8>| {
                if let GraphCanvasEvent::Activate(id) = &event {
                    state.selected = Some(*id);
                }
                state.events.push(event);
            },
        ))
    }

    #[test]
    fn component_owns_emphasis_and_reports_only_events() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::<_, _, _, ()>::new(
            dom.clone(),
            component_view,
            AppState {
                selected: Some(1),
                events: Vec::new(),
            },
        );
        let root = runner.root();
        let second =
            find_attr(&dom.borrow(), root, "aria-label", "Second node").expect("second node");

        // Hover renders emphasis without the application storing anything.
        runner.dispatch_hover(
            second,
            HoverEvent::new(HoverPhase::Enter, (2.0, 2.0), (20.0, 20.0)),
        );
        assert!(
            runner.state().events.is_empty(),
            "emphasis is not an application event"
        );
        assert_eq!(
            attr(&dom.borrow(), second, "class"),
            Some("graph-canvas-swatch-node hovered"),
            "the component rendered its own hover emphasis"
        );

        runner.dispatch_hover(
            second,
            HoverEvent::new(HoverPhase::Leave, (2.0, 2.0), (20.0, 20.0)),
        );
        assert_eq!(
            attr(&dom.borrow(), second, "class"),
            Some("graph-canvas-swatch-node"),
            "leaving clears the component's emphasis"
        );

        // Activation is an event, and the selection it drives is parent truth
        // that comes back through props.
        runner.dispatch_click(second, PointerClick::at((2.0, 2.0)));
        assert_eq!(runner.state().events, [GraphCanvasEvent::Activate(2)]);
        assert_eq!(runner.state().selected, Some(2));
        assert_eq!(
            attr(&dom.borrow(), second, "class"),
            Some("graph-canvas-swatch-node selected focused"),
            "parent-owned selection renders beside the component's own focus emphasis"
        );

        let expand =
            find_attr(&dom.borrow(), root, "aria-label", "Expand graph").expect("expand target");
        runner.dispatch_click(expand, PointerClick::at((2.0, 2.0)));
        assert_eq!(
            runner.state().events,
            [GraphCanvasEvent::Activate(2), GraphCanvasEvent::Expand]
        );
    }

    struct AtlasState {
        events: Vec<GraphAtlasEvent<u8>>,
        deferred: bool,
        builds: Rc<Cell<u32>>,
    }

    impl Default for AtlasState {
        fn default() -> Self {
            Self {
                events: Vec::new(),
                deferred: false,
                builds: Rc::new(Cell::new(0)),
            }
        }
    }

    type AtlasView = Box<dyn AnyView<AtlasState, (), GenetCtx, GenetElement>>;

    fn atlas_view(state: &AtlasState) -> AtlasView {
        use sceno::{Footprint, Rect, Size2, Vec2};

        state.builds.set(state.builds.get() + 1);

        let swatch = GraphCanvasSwatch::new(
            77,
            GraphCanvasSubgraph {
                nodes: vec![GraphCanvasNode {
                    id: 1,
                    kind: (),
                    position: (0.5, 0.5),
                    label: "Tower".into(),
                    key: None,
                }],
                edges: Vec::new(),
            },
        )
        .with_size(200, 100)
        .with_expand(false)
        .with_atlas(
            GraphCanvasAtlas::new(Rect::new(Vec2::ZERO, Size2::new(100.0, 50.0))).with_fields(
                vec![GraphCanvasAtlasField {
                    id: 1,
                    anchor: Vec2::new(50.0, 25.0),
                    footprint: Footprint::Polygon {
                        points: vec![
                            Vec2::new(-10.0, -10.0),
                            Vec2::new(10.0, 0.0),
                            Vec2::new(-10.0, 10.0),
                        ],
                    },
                    priority: 1,
                    label: "Tower field".into(),
                }],
            ),
        );
        let swatch = if state.deferred {
            swatch.with_deferred_drag_rebuild(true)
        } else {
            swatch
        };
        Box::new(graph_atlas_swatch(
            &swatch,
            |state: &mut AtlasState, event| {
                state.events.push(event);
            },
        ))
    }

    #[test]
    fn atlas_pointer_uses_polygon_not_semantic_button_bounds_and_captures_drag() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner =
            GenetAppRunner::<_, _, _, ()>::new(dom.clone(), atlas_view, AtlasState::default());
        let root = find_attr(
            &dom.borrow(),
            runner.root(),
            "class",
            "graph-canvas-swatch graph-canvas-atlas-swatch",
        )
        .expect("atlas root");
        let size = (200.0, 100.0);
        // (115, 30) lies in the triangle's bounding rectangle but outside its
        // actual edge, proving the full-surface handler does not accept it.
        runner.dispatch_pointer_down(
            root,
            PointerEvent::new(PointerPhase::Down, (115.0, 30.0), size),
        );
        runner.dispatch_pointer_up(PointerEvent::new(PointerPhase::Up, (115.0, 30.0), size));
        assert!(runner.state().events.is_empty());

        runner.dispatch_pointer_down(
            root,
            PointerEvent::new(PointerPhase::Down, (100.0, 50.0), size),
        );
        runner.dispatch_pointer_move(PointerEvent::new(PointerPhase::Move, (180.0, 50.0), size));
        runner.dispatch_pointer_up(PointerEvent::new(PointerPhase::Up, (180.0, 50.0), size));
        let drags: Vec<_> = runner
            .state()
            .events
            .iter()
            .filter_map(|event| match event {
                GraphAtlasEvent::Drag(drag) => Some(drag),
                _ => None,
            })
            .collect();
        assert_eq!(drags.len(), 3, "down/move/up retain the geographic field");
        assert!(drags.iter().all(|drag| drag.id == 1));
        assert!(
            drags[1].position.0 > 0.5,
            "inverse atlas view remains normalized"
        );
    }

    #[test]
    fn deferred_atlas_hover_and_move_do_not_rebuild_the_logic_view() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let builds = Rc::new(Cell::new(0));
        let mut runner = GenetAppRunner::<_, _, _, ()>::new(
            dom.clone(),
            atlas_view,
            AtlasState {
                events: Vec::new(),
                deferred: true,
                builds: builds.clone(),
            },
        );
        let root = find_attr(
            &dom.borrow(),
            runner.root(),
            "class",
            "graph-canvas-swatch graph-canvas-atlas-swatch",
        )
        .expect("atlas root");
        let size = (200.0, 100.0);
        assert_eq!(builds.get(), 1);
        for _ in 0..3 {
            runner.dispatch_hover(root, HoverEvent::new(HoverPhase::Move, (100.0, 50.0), size));
        }
        runner.dispatch_pointer_down(
            root,
            PointerEvent::new(PointerPhase::Down, (100.0, 50.0), size),
        );
        runner.dispatch_pointer_move(PointerEvent::new(PointerPhase::Move, (120.0, 50.0), size));
        runner.dispatch_pointer_move(PointerEvent::new(PointerPhase::Move, (140.0, 50.0), size));
        assert_eq!(
            builds.get(),
            2,
            "Down rebuilds once; deferred hover and move do not"
        );
        runner.dispatch_pointer_up(PointerEvent::new(PointerPhase::Up, (140.0, 50.0), size));
        assert_eq!(
            builds.get(),
            3,
            "release reconciles the retained surface once"
        );
    }

    #[test]
    fn atlas_field_focus_and_enter_emit_focus_then_activation() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner =
            GenetAppRunner::<_, _, _, ()>::new(dom.clone(), atlas_view, AtlasState::default());
        let field = find_attr(&dom.borrow(), runner.root(), "aria-label", "Tower field")
            .expect("semantic geographic field button");
        runner.set_focus(Some(field));
        runner.dispatch_key(crate::KeyEvent::new(crate::Key::Named(
            crate::NamedKey::Enter,
        )));
        assert_eq!(
            runner.state().events,
            [
                GraphAtlasEvent::Focus(Some(1)),
                GraphAtlasEvent::Activate(1)
            ]
        );
    }
}
