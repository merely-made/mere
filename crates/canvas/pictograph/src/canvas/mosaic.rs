// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mosaic on the canvas (graph grammar, DECISIONS 2026-10-16; Mark: "mosaic
//! shouldn't be limited to the viewport, but should exist on an infinite
//! canvas too. Same behavior, just pannable, organizable, resizable").
//!
//! Tiles group by site and size by weight; selection lifts a tile's outline
//! but never reflows the mosaic. The person organises it: dropping a tile on
//! another moves it into that group just before it, dropping it on a group
//! appends it there, dragging a group's label band onto another group moves
//! the group before it, and a tile can grow or shrink. That arrangement is
//! view-local and survives re-projection; a new graph clears it.

use std::collections::HashMap;

use cartography::adapters::{MosaicAdapter, channels_read};
use cartography::{AxisValue, LayoutStrategy, ProjectedRegion, ProjectionRequest};
use kernel::geometry::PortablePoint;
use kernel::graph::{Graph, NodeKey};

use super::Canvas;
use super::cartography_scene::CartographySceneOptions;
use crate::signals::ChannelRegistry;

/// The canvas strategy id.
pub const MOSAIC_ARRANGEMENT: &str = MosaicAdapter::PROJECTION_ID;
/// The smallest and largest size factor a tile can take.
pub const TILE_SCALE_RANGE: (f32, f32) = (0.25, 8.0);

/// The person's arrangement of the mosaic.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MosaicState {
    /// Nodes moved out of their site's group.
    pub group_of: HashMap<NodeKey, String>,
    pub group_order: Vec<String>,
    pub member_order: Vec<NodeKey>,
    pub scale: HashMap<NodeKey, f32>,
    /// A group whose label band is being dragged.
    pub(crate) group_press: Option<String>,
}

impl MosaicState {
    pub(crate) fn retain(&mut self, live: impl Fn(&NodeKey) -> bool) {
        self.group_of.retain(|key, _| live(key));
        self.member_order.retain(|key| live(key));
        self.scale.retain(|key, _| live(key));
    }
}

/// Where a tile goes when it is dropped.
#[derive(Clone, Debug, PartialEq)]
pub enum MosaicTarget {
    /// Just before this member, in its group.
    Tile(uuid::Uuid),
    /// At the end of this group.
    Group(String),
}

/// Project the mosaic under `state` at `(width, height)`.
pub fn project_mosaic(
    registry: &mut ChannelRegistry,
    graph: &Graph,
    state: &MosaicState,
    width: u32,
    height: u32,
    extents: Option<&HashMap<NodeKey, (f32, f32)>>,
) -> cartography::Projection {
    let mut options = CartographySceneOptions::canvas_pixels(width, height);
    options.extents = extents.cloned();
    let mut axis: HashMap<NodeKey, AxisValue> = registry
        .sites(graph)
        .iter()
        .map(|(key, site)| (*key, AxisValue::Categorical(site.clone())))
        .collect();
    for (key, group) in &state.group_of {
        if graph.get_node(*key).is_some() {
            axis.insert(*key, AxisValue::Categorical(group.clone()));
        }
    }
    let signals = registry.disclose(graph, channels_read(MOSAIC_ARRANGEMENT), None);
    let mut intent = options.to_view_intent();
    intent.axis_values = Some(axis);
    let request = ProjectionRequest {
        graph,
        signals: &signals,
        intent,
    };
    MosaicAdapter {
        config: sceno::Mosaic {
            group_order: state.group_order.clone(),
            ..sceno::Mosaic::default()
        },
        order: state.member_order.clone(),
        scale: state.scale.clone(),
    }
    .project(&request)
}

/// A region's axis-aligned bounds: tiles and groups are rectangles.
fn bounds(region: &ProjectedRegion) -> (f32, f32, f32, f32) {
    region.contour.iter().fold(
        (
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ),
        |(x0, y0, x1, y1), p| (x0.min(p.x), y0.min(p.y), x1.max(p.x), y1.max(p.y)),
    )
}

fn holds(region: &ProjectedRegion, at: PortablePoint) -> bool {
    let (x0, y0, x1, y1) = bounds(region);
    at.x >= x0 && at.x <= x1 && at.y >= y0 && at.y <= y1
}

impl Canvas {
    fn mosaic_active(&self) -> bool {
        self.active_strategy.as_deref() == Some(MOSAIC_ARRANGEMENT)
    }

    pub fn mosaic_state(&self) -> &MosaicState {
        &self.mosaic
    }

    /// The tile under world point `at`, other than `except`.
    fn tile_at(&self, at: PortablePoint, except: Option<NodeKey>) -> Option<NodeKey> {
        self.strategy_regions
            .iter()
            .filter(|r| r.label.is_none() && holds(r, at))
            .filter_map(|r| r.members.first().copied())
            .find(|key| Some(*key) != except)
    }

    /// The group under world point `at`; with `band`, only its label band.
    fn group_at(&self, at: PortablePoint, band: bool) -> Option<String> {
        let header = sceno::Mosaic::default().header;
        self.strategy_regions
            .iter()
            .filter(|r| r.label.is_some() && holds(r, at))
            .find(|r| !band || at.y <= bounds(r).1 + header)
            .and_then(|r| r.label.clone())
    }

    fn group_of_key(&self, key: NodeKey) -> Option<String> {
        self.strategy_regions
            .iter()
            .find(|r| r.label.is_some() && r.members.contains(&key))
            .and_then(|r| r.label.clone())
    }

    /// Move `member`'s tile to `target`. Returns whether anything moved.
    pub fn mosaic_move_member(&mut self, member: uuid::Uuid, target: MosaicTarget) -> bool {
        let Some(key) = self.graph.get_node_key_by_id(member) else {
            return false;
        };
        let (group, before) = match target {
            MosaicTarget::Tile(other) => {
                let Some(other) = self.graph.get_node_key_by_id(other).filter(|o| *o != key) else {
                    return false;
                };
                let Some(group) = self.group_of_key(other) else {
                    return false;
                };
                (group, Some(other))
            },
            MosaicTarget::Group(group) => (group, None),
        };
        self.place_in_mosaic(key, group, before);
        self.rearrange_mosaic();
        true
    }

    fn place_in_mosaic(&mut self, key: NodeKey, group: String, before: Option<NodeKey>) {
        self.mosaic.group_of.insert(key, group);
        // The whole current order, so a move is relative to what is drawn.
        if self.mosaic.member_order.is_empty() {
            self.mosaic.member_order = self
                .strategy_regions
                .iter()
                .filter(|r| r.label.is_none())
                .filter_map(|r| r.members.first().copied())
                .collect();
        }
        self.mosaic.member_order.retain(|k| *k != key);
        let at = before
            .and_then(|b| self.mosaic.member_order.iter().position(|k| *k == b))
            .unwrap_or(self.mosaic.member_order.len());
        self.mosaic.member_order.insert(at, key);
    }

    /// Move `group` just before `before` in the mosaic's group order.
    pub fn mosaic_move_group(&mut self, group: &str, before: &str) -> bool {
        if group == before {
            return false;
        }
        let mut order = if self.mosaic.group_order.is_empty() {
            self.strategy_regions
                .iter()
                .filter_map(|r| r.label.clone())
                .collect()
        } else {
            self.mosaic.group_order.clone()
        };
        if !order.iter().any(|g| g == before) {
            return false;
        }
        order.retain(|g| g != group);
        let at = order
            .iter()
            .position(|g| g == before)
            .unwrap_or(order.len());
        order.insert(at, group.to_owned());
        self.mosaic.group_order = order;
        self.rearrange_mosaic();
        true
    }

    /// Grow (`factor` > 1) or shrink every selected tile. Returns whether
    /// anything was selected.
    pub fn resize_selected_tiles(&mut self, factor: f32) -> bool {
        if self.selected.is_empty() || !factor.is_finite() || factor <= 0.0 {
            return false;
        }
        let (low, high) = TILE_SCALE_RANGE;
        for key in self.selected.clone() {
            let scale = self.mosaic.scale.entry(key).or_insert(1.0);
            *scale = (*scale * factor).clamp(low, high);
        }
        self.rearrange_mosaic();
        true
    }

    /// A dragged tile was released: file it where it landed, or snap it back.
    pub(crate) fn mosaic_drop(&mut self, key: NodeKey) {
        if !self.mosaic_active() {
            return;
        }
        let at = self.screen_to_world(self.cursor);
        let at = PortablePoint::new(at.x, at.y);
        if let Some(other) = self.tile_at(at, Some(key))
            && let Some(group) = self.group_of_key(other)
        {
            self.place_in_mosaic(key, group, Some(other));
        } else if let Some(group) = self.group_at(at, false) {
            self.place_in_mosaic(key, group, None);
        }
        self.rearrange_mosaic();
    }

    /// A press on a group's label band starts moving the group.
    pub(crate) fn begin_mosaic_group_press(&mut self) -> bool {
        if !self.mosaic_active() {
            return false;
        }
        let at = self.screen_to_world(self.cursor);
        self.mosaic.group_press = self.group_at(PortablePoint::new(at.x, at.y), true);
        self.mosaic.group_press.is_some()
    }

    /// End a group press: move the group before the one it was dropped on.
    pub(crate) fn end_mosaic_group_press(&mut self) -> bool {
        let Some(group) = self.mosaic.group_press.take() else {
            return false;
        };
        let at = self.screen_to_world(self.cursor);
        if let Some(onto) = self.group_at(PortablePoint::new(at.x, at.y), false) {
            self.mosaic_move_group(&group, &onto);
        }
        true
    }

    /// Re-project the mosaic in place under the person's arrangement.
    fn rearrange_mosaic(&mut self) {
        if !self.mosaic_active() {
            return;
        }
        let extents = self.strategy_extents();
        let projection = project_mosaic(
            &mut self.channels,
            &self.graph,
            &self.mosaic,
            self.view_w,
            self.view_h,
            Some(&extents),
        );
        let positions: Vec<_> = projection
            .nodes
            .iter()
            .map(|n| (n.node, n.position))
            .collect();
        self.apply_strategy_positions(&positions);
        self.strategy_regions = projection.regions;
    }
}
