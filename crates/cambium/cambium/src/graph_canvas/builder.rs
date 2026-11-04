/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

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
            cull_labels: true,
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

    pub(super) fn relation_line(&self, kind: &str) -> RelationLine {
        self.relation_lines
            .iter()
            .find(|(named, _)| named == kind)
            .map_or(RelationLine::Solid, |(_, line)| *line)
    }
}
