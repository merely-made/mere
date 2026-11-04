// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Leaf for GraphCanvas {
    /// A node-link glyph announces as a graphics object. Its interior structure
    /// (which node, which link) is not reachable through a single AccessKit node;
    /// publishing the nodes as semantic children is the leaf-publication work the
    /// native-automation plan scopes to phase 2 proper. Until then a screen reader
    /// at least learns that this is a graph and how big it is.
    ///
    /// The name is a **fallback only**. This glyph is reused as a button face, a
    /// link preview, and a breadcrumb thumbnail, so the author who placed it knows
    /// what it depicts and says so with `aria-label`; a generic self-description
    /// must never overwrite that.
    fn accessibility(&self, node: &mut accesskit::Node) {
        node.set_role(accesskit::Role::GraphicsObject);
        if node.label().is_none() {
            node.set_label(format!(
                "graph: {} nodes, {} links",
                self.nodes.len(),
                self.edges.len() + self.relations.len()
            ));
        }
    }

    fn measure(&mut self, _known: SizeHint, _available: SizeHint) -> Size {
        self.intrinsic
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        let s = cx.size();
        // A panned or zoomed viewport projects positions outside the box;
        // this canvas must not paint into its neighbours.
        cx.push_clip_rect(0.0, 0.0, s.width, s.height);
        // Inset so node circles at 0/1 coordinates stay inside the box.
        let inset = self.node_radius + self.edge_width;
        let place = |n: &GraphGlyphNode| self.viewport.project((n.x, n.y), s, inset);
        for polygon in &self.atlas_polygons {
            if polygon.points.len() < 3 {
                continue;
            }
            let mut path = Path::new();
            for (index, &(x, y)) in polygon.points.iter().enumerate() {
                path = if index == 0 {
                    path.move_to(x, y)
                } else {
                    path.line_to(x, y)
                };
            }
            let path = path.close().build();
            cx.fill_path(path.clone(), polygon.fill);
            if let Some(stroke) = polygon.stroke {
                cx.stroke_path(path, round_stroke(stroke, polygon.stroke_width.max(0.1)));
            }
        }
        for compound in &self.atlas_paths {
            let mut path = Path::new();
            let mut meaningful = false;
            for contour in &compound.contours {
                if contour.len() < 3 {
                    continue;
                }
                meaningful = true;
                for (index, &(x, y)) in contour.iter().enumerate() {
                    path = if index == 0 {
                        path.move_to(x, y)
                    } else {
                        path.line_to(x, y)
                    };
                }
                path = path.close();
            }
            if !meaningful {
                continue;
            }
            let path = path.build();
            cx.fill_path(path.clone(), compound.fill);
            if let Some(stroke) = compound.stroke {
                cx.stroke_path(path, round_stroke(stroke, compound.stroke_width.max(0.1)));
            }
        }
        // Edges under nodes.
        for &(a, b) in &self.edges {
            let (Some(na), Some(nb)) = (self.nodes.get(a as usize), self.nodes.get(b as usize))
            else {
                continue;
            };
            let (ax, ay) = place(na);
            let (bx, by) = place(nb);
            cx.stroke_path(
                Path::polyline(&[(ax, ay), (bx, by)]),
                round_stroke(self.edge_color, self.edge_width),
            );
        }
        for relation in &self.relations {
            if relation.points.len() < 2 {
                continue;
            }
            let points = relation
                .points
                .iter()
                .map(|point| self.viewport.project(*point, s, inset))
                .collect::<Vec<_>>();
            let heavy = relation.emphasized || relation.line == RelationLine::Heavy;
            let contribution = if relation.strength.is_finite() {
                relation.strength.max(1.0).sqrt().min(4.0)
            } else {
                1.0
            };
            let mut stroke = round_stroke(
                if relation.emphasized {
                    self.selection_color
                } else {
                    self.edge_color
                },
                if heavy {
                    self.edge_width * contribution.max(2.0)
                } else {
                    self.edge_width * contribution
                },
            );
            // Round caps turn the dotted pattern's short dashes into dots.
            stroke.dash = match relation.line {
                RelationLine::Dashed => Some(vec![6.0, 4.0]),
                RelationLine::Dotted => Some(vec![0.5, 3.5]),
                RelationLine::Solid | RelationLine::Heavy => None,
            }
            .map(|intervals| DashPattern {
                intervals,
                offset: 0.0,
            });
            let mark_stroke = round_stroke(
                if relation.emphasized {
                    self.selection_color
                } else {
                    self.edge_color
                },
                self.edge_width.max(1.5),
            );
            cx.stroke_path(Path::polyline(&points), stroke);
            for mark in relation_mark_paths(
                &points,
                relation.start_arrow,
                relation.end_arrow,
                relation.opposed,
            ) {
                cx.stroke_path(Path::polyline(&mark), mark_stroke.clone());
            }
        }
        for (index, n) in self.nodes.iter().enumerate() {
            let (x, y) = place(n);
            if self.hovered == u16::try_from(index).ok() {
                let mut wash = n.color;
                wash.a *= 0.24;
                cx.fill_path(Path::circle(x, y, self.node_radius * 1.9), wash);
            }
            if self.selected == u16::try_from(index).ok() {
                cx.stroke_path(
                    Path::circle(x, y, self.node_radius + 2.0),
                    round_stroke(self.selection_color, 1.5),
                );
            }
            if self.focus == u16::try_from(index).ok() {
                cx.stroke_path(
                    Path::circle(x, y, self.node_radius + 4.0),
                    round_stroke(self.focus_color, 1.0),
                );
            }
            cx.fill_path(Path::circle(x, y, self.node_radius), n.color);
        }
        cx.pop_clip();
        self.dirty = false;
    }

    fn paint_dirty(&self) -> bool {
        self.dirty
    }
}
