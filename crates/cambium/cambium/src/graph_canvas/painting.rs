/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

impl<Id: PartialEq, Kind> GraphCanvasSwatch<Id, Kind> {
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
}
