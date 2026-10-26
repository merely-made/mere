// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A saved Forme workspace and its portable workbench projection.
//! Host preferences are a typed serialized payload supplied by the embedding app.

use crate::{
    TileLayout,
    projection_geometry::{Axis, TreeGeometry},
};
use forme::FormeDocument;
use kernel::graph::Graph;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use uuid::Uuid;
use workbench::{ContentSource, DropTarget, SplitAxis, Tile, TileEvent, TileId, TileTree};

pub const WORKSPACE_VERSION: u16 = 1;

/// The preferences payload remains application-owned while the layout reducer
/// receives only the few navigation facts needed to order a read or validate it.
pub trait WorkspacePreferences: Clone + Default + PartialEq {
    fn focused_reading(&self) -> Option<Uuid> {
        None
    }
    fn set_focused_reading(&mut self, _member: Option<Uuid>) {}
    fn visit_reading(&mut self, _member: Uuid) {}
    fn reading_after_close(&self, members: &[Uuid], stack: &[Uuid], closed: Uuid) -> Option<Uuid> {
        let _ = (members, stack, closed);
        None
    }
    fn validate(&self, _graph_id: Uuid, _members: &[Uuid]) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NoPreferences {}
impl WorkspacePreferences for NoPreferences {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormeWorkspace<P = NoPreferences> {
    pub version: u16,
    pub document: FormeDocument,
    pub geometry: Option<TreeGeometry>,
    /// World-space outer region, separate from the relative tile geometry.
    pub bounds: [f32; 4],
    pub locked: bool,
    pub visible: bool,
    /// Host-owned persisted preferences, flattened to preserve the host wire shape.
    #[serde(flatten)]
    pub preferences: P,
    tile_ids: BTreeMap<Uuid, u64>,
}

impl<P: WorkspacePreferences> FormeWorkspace<P> {
    pub fn new(graph_id: Uuid) -> Self {
        Self {
            version: WORKSPACE_VERSION,
            document: FormeDocument::new(graph_id, Some("Workbench".into())),
            geometry: None,
            bounds: [-240., -150., 240., 150.],
            locked: true,
            visible: true,
            tile_ids: BTreeMap::new(),
            preferences: P::default(),
        }
    }
    pub fn layout(&self) -> TileLayout {
        TileLayout::from_arrangement(&self.document.arrangement, self.geometry.as_ref())
    }
    pub fn keep_layout(&mut self, layout: &TileLayout) {
        let (mut arrangement, geometry) = layout.to_arrangement();
        arrangement.preserve_member_identity(&self.document.arrangement);
        self.document.arrangement = arrangement;
        self.geometry = geometry;
    }
    /// Open or focus an existing access, keyed by surface UUID rather than address.
    pub fn open(&mut self, member: Uuid, graph: &Graph) -> Result<(), String> {
        graph
            .get_node_by_id(member)
            .ok_or("the access is no longer present")?;
        if !self.members().contains(&member) && self.members().len() >= 256 {
            return Err("Workbench tile limit reached".into());
        }
        if !self.tile_ids.contains_key(&member) {
            if self.tile_ids.len() >= 1024 {
                return Err("Workbench access limit reached".into());
            }
            let next = self
                .tile_ids
                .values()
                .max()
                .copied()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or("Workbench tile identity limit reached")?;
            self.tile_ids.insert(member, next);
        }
        let mut layout = self.layout();
        if layout.has_tile(member) {
            layout.activate(member);
        } else {
            layout.open_tile(member);
        }
        self.keep_layout(&layout);
        self.preferences.set_focused_reading(Some(member));
        self.preferences.visit_reading(member);
        Ok(())
    }
    /// Follow within the current reading stack; existing arranged cells retain their place.
    pub fn read(&mut self, member: Uuid, graph: &Graph) -> Result<(), String> {
        let already_open = self.members().contains(&member);
        let origin = self
            .preferences
            .focused_reading()
            .filter(|origin| self.members().contains(origin));
        self.open(member, graph)?;
        if !already_open && let Some(origin) = origin {
            let mut layout = self.layout();
            layout.move_to_stack_index_of(member, origin, usize::MAX);
            self.keep_layout(&layout);
        }
        Ok(())
    }
    pub fn tile_tree(&self, graph: &Graph) -> Option<TileTree> {
        // Saved opaque keys survive closing/reopening other tabs and never name a Resource.
        let mut layout = self.layout();
        for member in layout.open_members() {
            if graph.get_node_by_id(member).is_none() {
                layout.close_tile(member);
            }
        }
        layout.to_tile_tree(|member| Tile {
            id: TileId(self.tile_ids[&member]),
            title: graph
                .get_node_by_id(member)
                .map(|(_, n)| n.title.clone())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| member.to_string()),
            content: ContentSource::Open {
                // This is a shared Forme member contract, not an app pane kind.
                kind: "forme.member".into(),
                id: member.to_string(),
            },
            accent: None,
        })
    }
    pub fn members(&self) -> Vec<Uuid> {
        let mut members = self.document.arrangement.referenced_members();
        members.sort();
        members.dedup();
        members
    }
    /// Project this owner's accesses without deleting missing members from the
    /// saved or draft layout. Graph redo can make those same accesses visible.
    pub fn project_region(
        &self,
        owner: Uuid,
        graph: &Graph,
    ) -> Result<Option<WorkspaceRegion>, String> {
        self.validate(owner)?;
        let mut region = self.region();
        region
            .cells
            .retain(|cell| graph.get_node_by_id(cell.member).is_some());
        Ok((!region.cells.is_empty()).then_some(region))
    }
    pub fn member_for_tile(&self, tile: TileId) -> Option<Uuid> {
        self.tile_ids
            .iter()
            .find(|(_, id)| **id == tile.0)
            .map(|(member, _)| *member)
            .filter(|member| self.members().contains(member))
    }
    pub fn tile_for_member(&self, member: Uuid) -> Option<TileId> {
        self.tile_ids.get(&member).copied().map(TileId)
    }
    pub fn event(&mut self, event: TileEvent, graph: &Graph) -> Result<(), String> {
        // Projected split paths may differ while missing accesses are masked.
        // Keep those stored identities; reject path-based edits until restored.
        if matches!(&event, TileEvent::DividerMoved { .. })
            && self
                .members()
                .iter()
                .any(|m| graph.get_node_by_id(*m).is_none())
        {
            return Err("Restore missing accesses before editing split dividers".into());
        }
        let admitted = |tile| {
            self.member_for_tile(tile)
                .filter(|m| graph.get_node_by_id(*m).is_some())
                .ok_or("the access is no longer present")
        };
        let focused = match &event {
            TileEvent::Activated(tile) => Some(admitted(*tile)?),
            TileEvent::Closed(tile) => {
                let member = admitted(*tile)?;
                let stack = self
                    .layout()
                    .slot_views()
                    .find(|slot| slot.members.contains(&member))
                    .map(|slot| slot.members.to_vec())
                    .unwrap_or_default();
                self.preferences
                    .reading_after_close(&self.members(), &stack, member)
            },
            _ => self.preferences.focused_reading(),
        };
        let mut layout = self.layout();
        match event {
            TileEvent::Activated(tile) => {
                let member = admitted(tile)?;
                layout.activate(member);
            },
            TileEvent::Closed(tile) => {
                layout.close_tile(admitted(tile)?);
            },
            TileEvent::DividerMoved { split, fractions } => {
                if self.locked {
                    return Err("Unlock the forme to change its arrangement".into());
                }
                if fractions.iter().any(|f| !f.is_finite() || *f <= 0.) {
                    return Err("Split shares must be finite and positive".into());
                }
                let current = layout.split_fractions(&split.0).ok_or("unknown split")?;
                if fractions.len() != current.len() {
                    return Err("Provide one split share for each region".into());
                }
                if !layout.set_split_fractions(&split.0, &fractions) {
                    return Err("unknown split".into());
                }
            },
            TileEvent::Dragged { tile, to } => {
                if self.locked {
                    return Err("Unlock the forme to change its arrangement".into());
                }
                let member = admitted(tile)?;
                let tree = self.tile_tree(graph).ok_or("empty forme")?;
                let target = match &to {
                    DropTarget::Stack { stack, .. } => match tree_at(&tree, &stack.0) {
                        Some(TileTree::Stack(s)) => s
                            .tabs
                            .iter()
                            .find(|t| t.id != tile)
                            .map(|t| t.id)
                            .or(Some(tile)),
                        _ => None,
                    },
                    DropTarget::Edge { tile: target, .. } => Some(*target),
                    _ => return Err("This host keeps tiles in its workbench".into()),
                }
                .and_then(|t| self.member_for_tile(t))
                .filter(|member| graph.get_node_by_id(*member).is_some())
                .ok_or("unknown drop region")?;
                match to {
                    DropTarget::Stack { index, .. } => {
                        layout.move_to_stack_index_of(member, target, index);
                    },
                    DropTarget::Edge { edge, .. } => {
                        use workbench::Edge;
                        let (axis, after) = match edge {
                            Edge::Left => (SplitAxis::Row, false),
                            Edge::Right => (SplitAxis::Row, true),
                            Edge::Top => (SplitAxis::Column, false),
                            Edge::Bottom => (SplitAxis::Column, true),
                        };
                        if member == target {
                            layout.split_out(member, axis, after);
                        } else {
                            layout.split_beside_axis(member, target, axis, after);
                        }
                    },
                    _ => {},
                }
            },
        }
        let mut candidate = self.clone();
        candidate.preferences.set_focused_reading(focused);
        if let Some(member) = focused {
            candidate.preferences.visit_reading(member);
        }
        candidate.keep_layout(&layout);
        candidate.validate(self.document.graph_id)?;
        *self = candidate;
        Ok(())
    }
    pub fn region(&self) -> WorkspaceRegion {
        let mut cells = Vec::new();
        if let Some(geometry) = &self.geometry {
            cells_of(geometry, [0., 0., 1., 1.], &mut cells);
        }
        WorkspaceRegion {
            id: self.document.id.as_uuid(),
            bounds: self.bounds,
            cells,
            locked: self.locked,
            visible: self.visible,
        }
    }
    pub fn validate(&self, graph_id: Uuid) -> Result<(), String> {
        self.preferences
            .validate(graph_id, &self.tile_ids.keys().copied().collect::<Vec<_>>())?;
        if self.version != WORKSPACE_VERSION {
            return Err("unsupported forme workspace version".into());
        }
        if self.document.graph_id != graph_id {
            return Err("forme belongs to another graph session".into());
        }
        if !valid_bounds(self.bounds) {
            return Err("invalid forme extent".into());
        }
        let ids: HashSet<_> = self.tile_ids.values().copied().collect();
        if self.tile_ids.len() > 1024
            || ids.len() != self.tile_ids.len()
            || ids.contains(&0)
            || self
                .members()
                .iter()
                .any(|m| !self.tile_ids.contains_key(m))
        {
            return Err("invalid stable tile identities".into());
        }
        if let Some(g) = &self.geometry {
            let mut seen = HashSet::new();
            validate_geometry(g, 0, &mut seen)?;
            if seen.len() > 256 {
                return Err("forme exceeds workspace limits".into());
            }
            if seen != self.members().into_iter().collect::<HashSet<_>>() {
                return Err("forme membership and geometry disagree".into());
            }
            let region = self.region();
            if region.cells.iter().any(|cell| {
                !positive_bounds(cell.bounds) || !positive_bounds(region.cell_world_bounds(cell))
            }) {
                return Err("Forme geometry collapses a tile region".into());
            }
        } else if !self.members().is_empty() {
            return Err("forme geometry is missing".into());
        }
        Ok(())
    }
}
impl<P: WorkspacePreferences> FormeWorkspace<P> {
    /// Reconcile a decoded workspace against its owner graph without storage or UI effects.
    pub fn reconcile(&mut self, graph: &Graph) {
        let mut layout = self.layout();
        for member in layout.open_members() {
            if graph.get_node_by_id(member).is_none() {
                layout.close_tile(member);
            }
        }
        self.keep_layout(&layout);
    }
}
fn valid_bounds(b: [f32; 4]) -> bool {
    let (w, h) = (b[2] - b[0], b[3] - b[1]);
    b.iter().all(|v| v.is_finite()) && w.is_finite() && h.is_finite() && w >= 80. && h >= 80.
}
fn positive_bounds(b: [f32; 4]) -> bool {
    b.iter().all(|v| v.is_finite()) && b[2] > b[0] && b[3] > b[1]
}
fn validate_geometry(
    g: &TreeGeometry,
    depth: usize,
    seen: &mut HashSet<Uuid>,
) -> Result<(), String> {
    if depth > 32 || seen.len() > 256 {
        return Err("forme exceeds workspace limits".into());
    }
    match g {
        TreeGeometry::Stack { members, active } => {
            if members.is_empty() || *active >= members.len() {
                return Err("invalid tab stack".into());
            }
            for m in members {
                if !seen.insert(*m) {
                    return Err("duplicate access in forme".into());
                }
            }
        },
        TreeGeometry::Split { children, .. } => {
            if children.len() < 2 || children.len() > 256 {
                return Err("invalid split".into());
            }
            let total: f32 = children.iter().map(|b| b.fraction).sum();
            if !total.is_finite() || total <= 0. {
                return Err("invalid split shares".into());
            }
            for b in children {
                if !b.fraction.is_finite() || b.fraction <= 0. {
                    return Err("invalid split share".into());
                }
                validate_geometry(&b.node, depth + 1, seen)?;
            }
        },
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegionCell {
    pub member: Uuid,
    pub bounds: [f32; 4],
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceRegion {
    pub id: Uuid,
    pub bounds: [f32; 4],
    pub cells: Vec<RegionCell>,
    pub locked: bool,
    pub visible: bool,
}
impl WorkspaceRegion {
    pub fn cell_world_bounds(&self, cell: &RegionCell) -> [f32; 4] {
        let [x, y, r, b] = self.bounds;
        [
            x + (r - x) * cell.bounds[0],
            y + (b - y) * cell.bounds[1],
            x + (r - x) * cell.bounds[2],
            y + (b - y) * cell.bounds[3],
        ]
    }
}
fn cells_of(g: &TreeGeometry, b: [f32; 4], out: &mut Vec<RegionCell>) {
    match g {
        TreeGeometry::Stack { members, active } => {
            if let Some(member) = members.get(*active) {
                out.push(RegionCell {
                    member: *member,
                    bounds: b,
                });
            }
        },
        TreeGeometry::Split { axis, children } => {
            let total: f32 = children.iter().map(|c| c.fraction).sum();
            let i = if *axis == Axis::Row { 0 } else { 1 };
            let length = b[i + 2] - b[i];
            let mut start = b[i];
            for (index, child) in children.iter().enumerate() {
                let mut cb = b;
                cb[i] = start;
                start = if index + 1 == children.len() {
                    b[i + 2]
                } else {
                    start + length * child.fraction / total
                };
                cb[i + 2] = start;
                cells_of(&child.node, cb, out);
            }
        },
    }
}
fn tree_at<'a>(tree: &'a TileTree, path: &[usize]) -> Option<&'a TileTree> {
    if path.is_empty() {
        return Some(tree);
    }
    if let TileTree::Split { children, .. } = tree {
        tree_at(&children.get(path[0])?.tree, &path[1..])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::graph::apply::{GraphDelta, add_node, apply_graph_delta};

    fn graph() -> (Graph, [Uuid; 2]) {
        let mut graph = Graph::new();
        let members = [Uuid::from_u128(1), Uuid::from_u128(2)];
        for member in members {
            add_node(
                &mut graph,
                Some(member),
                "https://example.org/same".into(),
                Default::default(),
            );
        }
        (graph, members)
    }

    #[test]
    fn duplicate_addresses_keep_stable_member_and_tile_identity() {
        let (graph, [a, b]) = graph();
        let mut workspace = FormeWorkspace::<NoPreferences>::new(Uuid::from_u128(10));
        workspace.open(a, &graph).unwrap();
        workspace.open(b, &graph).unwrap();
        let tile = workspace.tile_for_member(b).unwrap();
        workspace.event(TileEvent::Closed(tile), &graph).unwrap();
        workspace.open(b, &graph).unwrap();
        assert_eq!(workspace.tile_for_member(b), Some(tile));
        assert_ne!(workspace.tile_for_member(a), Some(tile));
        assert_eq!(workspace.members().len(), 2);
    }

    #[test]
    fn region_projection_is_layout_only_and_uses_world_bounds() {
        let (graph, [a, b]) = graph();
        let mut workspace = FormeWorkspace::<NoPreferences>::new(Uuid::from_u128(10));
        workspace.open(a, &graph).unwrap();
        workspace.open(b, &graph).unwrap();
        workspace.bounds = [10., 20., 210., 120.];
        let region = workspace
            .project_region(workspace.document.graph_id, &graph)
            .unwrap()
            .unwrap();
        assert_eq!(region.cells.len(), 2);
        let mut world: Vec<_> = region
            .cells
            .iter()
            .map(|cell| region.cell_world_bounds(cell))
            .collect();
        world.sort_by(|a, b| a[0].total_cmp(&b[0]));
        assert_eq!(world, vec![[10., 20., 110., 120.], [110., 20., 210., 120.]]);
        assert_eq!(workspace.members().len(), 2);
    }

    #[test]
    fn invalid_divider_edit_leaves_the_canonical_arrangement_unchanged() {
        let (graph, [a, b]) = graph();
        let mut workspace = FormeWorkspace::<NoPreferences>::new(Uuid::from_u128(10));
        workspace.open(a, &graph).unwrap();
        workspace.open(b, &graph).unwrap();
        workspace.locked = false;
        let before = workspace.document.arrangement.clone();
        let result = workspace.event(
            TileEvent::DividerMoved {
                split: workbench::TilePath(vec![]),
                fractions: vec![0.0],
            },
            &graph,
        );
        assert!(result.is_err());
        assert_eq!(workspace.document.arrangement, before);
    }

    #[test]
    fn current_workspace_json_roundtrips_the_flat_host_preferences_payload() {
        #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
        struct Preferences {
            marker: u32,
        }
        impl WorkspacePreferences for Preferences {}
        let (graph, [a, _]) = graph();
        let mut workspace = FormeWorkspace::<Preferences>::new(Uuid::from_u128(10));
        workspace.open(a, &graph).unwrap();
        workspace.preferences.marker = 42;
        let bytes = serde_json::to_vec(&workspace).unwrap();
        let restored: FormeWorkspace<Preferences> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(restored.preferences.marker, 42);
        assert_eq!(restored.members(), vec![a]);
    }

    #[test]
    fn edge_drag_creates_nested_projection_and_locked_focus_close_remain_available() {
        let (mut graph, [a, b]) = graph();
        let c = Uuid::from_u128(3);
        add_node(
            &mut graph,
            Some(c),
            "https://example.org/other".into(),
            Default::default(),
        );
        let mut workspace = FormeWorkspace::<NoPreferences>::new(Uuid::from_u128(10));
        for member in [a, b, c] {
            workspace.open(member, &graph).unwrap();
        }
        workspace.locked = false;
        workspace
            .event(
                TileEvent::Dragged {
                    tile: workspace.tile_for_member(c).unwrap(),
                    to: DropTarget::Edge {
                        tile: workspace.tile_for_member(b).unwrap(),
                        edge: workbench::Edge::Bottom,
                    },
                },
                &graph,
            )
            .unwrap();
        let before = workspace.geometry.clone();
        let encoded = serde_json::to_vec(&workspace).unwrap();
        let mut restored: FormeWorkspace = serde_json::from_slice(&encoded).unwrap();
        restored.validate(workspace.document.graph_id).unwrap();
        assert_eq!(restored.geometry, before);
        assert_eq!(restored.region().cells.len(), 3);
        restored.locked = true;
        let tile = restored.tile_for_member(b).unwrap();
        restored.event(TileEvent::Activated(tile), &graph).unwrap();
        restored
            .event(
                TileEvent::Closed(restored.tile_for_member(a).unwrap()),
                &graph,
            )
            .unwrap();
        assert_eq!(restored.members(), vec![b, c]);
    }

    #[test]
    fn shared_resource_addresses_remain_distinct_access_tiles() {
        let (graph, [a, b]) = graph();
        let key_a = graph.get_node_by_id(a).unwrap().0;
        let key_b = graph.get_node_by_id(b).unwrap().0;
        assert_ne!(key_a, key_b);
        assert_eq!(
            graph.shown_resource_id(key_a),
            graph.shown_resource_id(key_b)
        );
        let mut workspace = FormeWorkspace::<NoPreferences>::new(Uuid::from_u128(10));
        workspace.open(a, &graph).unwrap();
        workspace.open(b, &graph).unwrap();
        workspace.open(a, &graph).unwrap();
        assert_eq!(workspace.layout().tile_count(), 2);
        let region = workspace.region();
        assert_eq!(region.cells.len(), 2);
        let mut cells = region.cells.iter().collect::<Vec<_>>();
        cells.sort_by(|a, b| a.bounds[0].total_cmp(&b.bounds[0]));
        assert!(cells[0].bounds[2] <= cells[1].bounds[0]);
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.fields().count(), 0);
    }

    #[test]
    fn malformed_and_collapsed_geometry_is_rejected_before_projection() {
        let (graph, [a, b]) = graph();
        let mut workspace = FormeWorkspace::<NoPreferences>::new(Uuid::from_u128(10));
        workspace.open(a, &graph).unwrap();
        workspace.open(b, &graph).unwrap();
        let valid_geometry = workspace.geometry.clone();
        workspace.geometry = Some(TreeGeometry::Stack {
            members: vec![a, a],
            active: 0,
        });
        assert!(workspace.validate(workspace.document.graph_id).is_err());
        workspace.geometry = Some(TreeGeometry::Stack {
            members: vec![a, b],
            active: 99,
        });
        assert!(workspace.validate(workspace.document.graph_id).is_err());
        workspace.geometry = valid_geometry;
        workspace.bounds = [
            1.0e20_f32,
            0.0,
            f32::from_bits(1.0e20_f32.to_bits() + 1),
            100.0,
        ];
        assert!(workspace.validate(workspace.document.graph_id).is_err());
    }

    #[test]
    fn world_coordinate_rounding_cannot_collapse_projected_cells() {
        let (graph, [a, b]) = graph();
        let mut workspace = FormeWorkspace::<NoPreferences>::new(Uuid::from_u128(10));
        workspace.open(a, &graph).unwrap();
        workspace.open(b, &graph).unwrap();
        workspace.locked = false;
        let mut layout = workspace.layout();
        layout.split_beside_axis(a, b, SplitAxis::Row, true);
        workspace.keep_layout(&layout);
        let left = 1.0e20_f32;
        workspace.bounds = [left, 0.0, f32::from_bits(left.to_bits() + 1), 1000.0];
        assert!(workspace.validate(workspace.document.graph_id).is_err());
    }

    #[test]
    fn split_fraction_extremes_and_divider_arity_fail_without_mutation() {
        let (graph, [a, b]) = graph();
        let mut workspace = FormeWorkspace::<NoPreferences>::new(Uuid::from_u128(10));
        workspace.open(a, &graph).unwrap();
        workspace.open(b, &graph).unwrap();
        workspace.locked = false;
        let original = serde_json::to_vec(&workspace).unwrap();
        for fractions in [vec![], vec![0.2], vec![0.2, 0.3, 0.5], vec![0.0, 1.0]] {
            assert!(
                workspace
                    .event(
                        TileEvent::DividerMoved {
                            split: workbench::TilePath(vec![]),
                            fractions
                        },
                        &graph
                    )
                    .is_err()
            );
            assert_eq!(serde_json::to_vec(&workspace).unwrap(), original);
        }
        let Some(TreeGeometry::Split { children, .. }) = &mut workspace.geometry else {
            panic!("split expected")
        };
        children[0].fraction = f32::MIN_POSITIVE;
        children[1].fraction = f32::MAX / 2.;
        assert!(workspace.validate(workspace.document.graph_id).is_err());
    }

    #[test]
    fn locked_arrangements_still_allow_focus_and_close_and_reconcile_missing_members() {
        let (mut graph, [a, b]) = graph();
        let mut workspace = FormeWorkspace::<NoPreferences>::new(Uuid::from_u128(10));
        workspace.open(a, &graph).unwrap();
        workspace.open(b, &graph).unwrap();
        let before = workspace.geometry.clone();
        assert!(
            workspace
                .event(
                    TileEvent::DividerMoved {
                        split: workbench::TilePath(vec![]),
                        fractions: vec![0.3, 0.7]
                    },
                    &graph
                )
                .is_err()
        );
        assert_eq!(workspace.geometry, before);
        workspace
            .event(
                TileEvent::Activated(workspace.tile_for_member(b).unwrap()),
                &graph,
            )
            .unwrap();
        workspace
            .event(
                TileEvent::Closed(workspace.tile_for_member(a).unwrap()),
                &graph,
            )
            .unwrap();
        assert_eq!(workspace.members(), vec![b]);

        let key = graph.get_node_key_by_id(b).unwrap();
        apply_graph_delta(&mut graph, GraphDelta::RemoveNode { key });
        workspace.reconcile(&graph);
        assert!(workspace.members().is_empty());
        assert_eq!(graph.node_count(), 1);
        assert!(workspace.validate(Uuid::from_u128(99)).is_err());
    }

    #[test]
    fn stack_drop_order_keeps_active_member_and_opaque_tile_ids() {
        let (mut graph, ids) = graph();
        let c = Uuid::from_u128(3);
        add_node(
            &mut graph,
            Some(c),
            "https://example.org/third".into(),
            Default::default(),
        );
        let mut workspace = FormeWorkspace::<NoPreferences>::new(Uuid::from_u128(10));
        for id in [ids[0], ids[1], c] {
            workspace.open(id, &graph).unwrap();
        }
        let mut layout = workspace.layout();
        layout.stack_all();
        workspace.keep_layout(&layout);
        workspace.locked = false;
        workspace
            .event(
                TileEvent::Dragged {
                    tile: workspace.tile_for_member(c).unwrap(),
                    to: DropTarget::Stack {
                        stack: workbench::TilePath(vec![]),
                        index: 0,
                    },
                },
                &graph,
            )
            .unwrap();
        assert_eq!(
            workspace.geometry,
            Some(TreeGeometry::Stack {
                members: vec![c, ids[0], ids[1]],
                active: 0
            })
        );
        workspace
            .event(
                TileEvent::Activated(workspace.tile_for_member(ids[1]).unwrap()),
                &graph,
            )
            .unwrap();
        let bytes = serde_json::to_vec(&workspace).unwrap();
        let restored: FormeWorkspace = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(restored.geometry, workspace.geometry);
        assert_eq!(restored.region().cells[0].member, ids[1]);
    }
}
