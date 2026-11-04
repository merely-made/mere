/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

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
    pub(super) fn label_text(&self, index: usize) -> std::borrow::Cow<'_, str> {
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

    pub(super) fn node_index(&self, id: Option<&Id>) -> Option<u16> {
        let id = id?;
        self.graph
            .nodes
            .iter()
            .position(|node| &node.id == id)
            .and_then(|index| u16::try_from(index).ok())
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

    /// Resolve visible labels once, in screen space. Placement, collision tests
    /// and emitted CSS consume the same bounded boxes. Try the quiet-axis
    /// preference first, then the other sides before suppressing a label.
    /// Explicit host placements stay on their requested side.
    pub(super) fn resolved_labels(&self) -> Vec<Option<ResolvedGraphLabel>> {
        let positions = self.projected_positions();
        let canvas = Size {
            width: self.width as f32,
            height: self.height as f32,
        };
        let clearance = self.node_radius + 7.0;
        let mut result = vec![None; positions.len()];
        let priority = |index: usize| {
            let id = &self.graph.nodes[index].id;
            if self.selected.as_ref() == Some(id) {
                0
            } else if self.focus.as_ref() == Some(id) {
                1
            } else if self.hovered.as_ref() == Some(id) {
                2
            } else {
                3
            }
        };
        let mut order: Vec<_> = (0..positions.len()).collect();
        order.sort_by_key(|&index| (priority(index), index));
        let mut obstacles: Vec<_> = positions
            .iter()
            .map(|&(_, (x, y))| LabelBox {
                left: x - self.node_radius,
                top: y - self.node_radius,
                width: self.node_radius * 2.0,
                height: self.node_radius * 2.0,
            })
            .collect();
        for footprint in &self.node_footprints {
            if let Some(region) = self.projected_node_footprint(&footprint.id) {
                obstacles.push(LabelBox {
                    left: region.left,
                    top: region.top,
                    width: region.width,
                    height: region.height,
                });
            }
        }
        let mut kept: Vec<LabelBox> = Vec::new();
        for index in order {
            let node = &self.graph.nodes[index];
            let (_, (x, y)) = positions[index];
            // An offscreen node must not leave an orphaned label clamped to
            // the viewport edge. Its accessible target retains its full name.
            if x < 0.0
                || x > canvas.width
                || y < 0.0
                || y > canvas.height
                || self
                    .node_footprints
                    .iter()
                    .any(|footprint| footprint.id == node.id)
            {
                continue;
            }
            let preferred = self.label_placement(index);
            let explicit = self.label_placements.iter().any(|(id, _)| *id == node.id);
            let mut candidates = vec![preferred];
            if !explicit {
                for side in [
                    GraphLabelPlacement::Above,
                    GraphLabelPlacement::Below,
                    GraphLabelPlacement::Left,
                    GraphLabelPlacement::Right,
                ] {
                    if side != preferred {
                        candidates.push(side);
                    }
                }
            }
            let width = label_text_width(&self.label_text(index));
            let choices: Vec<_> = candidates
                .into_iter()
                .filter_map(|placement| {
                    let bounds = placement.bounds(x, y, clearance, canvas, width);
                    // Do not silently truncate a short name just to choose the
                    // preferred side when a different side holds the whole name.
                    ((explicit || bounds.width >= width.min(canvas.width))
                        && bounds.width > 0.0
                        && bounds.height > 0.0)
                        .then_some(ResolvedGraphLabel { placement, bounds })
                })
                .collect();
            let resolved = choices
                .iter()
                .find(|candidate| {
                    !kept
                        .iter()
                        .chain(obstacles.iter())
                        .any(|other| candidate.bounds.overlaps(*other))
                })
                .copied()
                .or_else(|| {
                    (!self.cull_labels)
                        .then(|| choices.first().copied())
                        .flatten()
                });
            if let Some(label) = resolved {
                kept.push(label.bounds);
            }
            result[index] = resolved;
        }
        result
    }

    #[cfg(test)]
    pub(super) fn shown_labels(&self) -> Vec<bool> {
        self.resolved_labels().iter().map(Option::is_some).collect()
    }

    pub(super) fn label_placement(&self, index: usize) -> GraphLabelPlacement {
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
    pub(super) fn side_with_room(&self, x: f32) -> GraphLabelPlacement {
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
    pub(super) fn resolved_relation_routes(
        &self,
    ) -> Vec<(&GraphCanvasRelation<Id>, Vec<(f32, f32)>)> {
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

    pub(super) fn projected_position(&self, id: &Id) -> Option<(f32, f32)> {
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
    pub(super) fn relation_routes(&self) -> Vec<(&GraphCanvasRelation<Id>, Vec<(f32, f32)>)> {
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
