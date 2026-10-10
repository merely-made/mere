// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The canvas's folds (mer3ly site plan, Rulings 165 and 166): several
//! non-overlapping [`FoldRecord`]s, as `ViewIntent.folds` and `sceno::Fold`
//! hold them. A selection folds into a summary body; a collapse keeps its
//! root drawn as a member stand-in carrying "+N". Folding, expanding and
//! relabelling are one reversible history, and none of them touches the
//! graph or its physics.

use curation::{FoldId, FoldRecord, FoldRule, FoldStandIn};
use kernel::geometry::PortablePoint;
use kernel::graph::{EdgeFamily, NodeKey};

use super::fold_projection::{
    FOLD_SUMMARY_RADIUS, FoldProjection, FoldTraversalDirection, edge_family_name, fold_keys,
    hierarchy_closure, project_fold,
};
use super::{Canvas, FoldViewState};

/// How near a stand-in's badge a press must land to reach it, screen px.
pub(crate) const FOLD_BADGE_REACH: f32 = 12.0;

/// One fold's "+N" as the canvas draws it, for a host that draws its own
/// node layer.
#[derive(Clone, Debug, PartialEq)]
pub struct FoldBadge {
    pub fold: FoldId,
    /// The member standing in, or `None` for a summary body.
    pub stand_in: Option<uuid::Uuid>,
    /// The badge's centre, screen px: a member's top-right corner, or a
    /// summary body's.
    pub at: (f32, f32),
    /// `+N`.
    pub text: String,
    pub label: Option<String>,
}

fn mint_fold_id() -> FoldId {
    FoldId::new(uuid::Uuid::new_v4().to_string())
}

impl Canvas {
    fn fold_view_state(&self) -> FoldViewState {
        FoldViewState {
            folds: self.folds.clone(),
            selected: self.selected.clone(),
            selected_edges: self.selected_edges.clone(),
        }
    }

    fn push_fold_undo(&mut self) {
        self.fold_undo.push(self.fold_view_state());
        self.fold_redo.clear();
    }

    fn apply_fold_view_state(&mut self, state: FoldViewState) {
        self.folds = state.folds;
        self.fold_press = None;
        self.selected = state.selected;
        self.selected_edges = state.selected_edges;
    }

    fn fold_view_state_is_current(&self, state: &FoldViewState) -> bool {
        state
            .folds
            .iter()
            .all(|record| fold_keys(&self.graph, record).is_some())
            && state
                .selected
                .iter()
                .all(|&key| self.graph.get_node(key).is_some())
    }

    fn member_id(&self, key: NodeKey) -> Option<String> {
        self.graph.get_node(key).map(|node| node.id.to_string())
    }

    fn fold_index(&self, id: &FoldId) -> Option<usize> {
        self.folds.iter().position(|record| &record.id == id)
    }

    /// Add `record` as one undo step. A fold sharing members with an active
    /// one is refused, unless `absorb` and that fold lies wholly inside the
    /// new one, which then replaces it (folds do not nest).
    fn add_fold(&mut self, record: FoldRecord, absorb: bool) -> Option<FoldId> {
        project_fold(&self.graph, &record)?;
        let mut absorbed = Vec::new();
        for (index, active) in self.folds.iter().enumerate() {
            if !active.overlaps(&record) {
                continue;
            }
            if absorb && active.members.iter().all(|member| record.contains(member)) {
                absorbed.push(index);
            } else {
                return None;
            }
        }
        let id = record.id.clone();
        self.push_fold_undo();
        for index in absorbed.into_iter().rev() {
            self.folds.remove(index);
        }
        self.folds.push(record);
        self.fold_press = None;
        self.selected.clear();
        self.selected_edges.clear();
        Some(id)
    }

    /// Fold the multi-selection into one summary body, recorded as a
    /// selection. Refused when fewer than two nodes are selected or one is
    /// already folded.
    ///
    /// The returned id is durable view-state identity. This method never adds,
    /// removes, or rewrites a graph node or relation.
    pub fn fold_selected(&mut self, source_scope: impl Into<String>) -> Option<FoldId> {
        let members: Vec<String> = self
            .selected
            .iter()
            .filter_map(|&key| self.member_id(key))
            .collect();
        let record = FoldRecord::selection(mint_fold_id(), source_scope, members).ok()?;
        self.add_fold(record, false)
    }

    /// Fold one root and every node reachable through the explicitly supplied
    /// relation family and direction, the root standing in with "+N" and the
    /// rule recorded as descendants. This does not guess hierarchy from the
    /// rest of the graph: callers choose both the family and traversal. A
    /// fold wholly inside the new one is absorbed into it; one partly inside
    /// refuses the collapse.
    pub fn collapse_descendants(
        &mut self,
        root: NodeKey,
        hierarchy_family: EdgeFamily,
        direction: FoldTraversalDirection,
        source_scope: impl Into<String>,
    ) -> Option<FoldId> {
        let root_id = self.member_id(root)?;
        let members: Vec<String> =
            hierarchy_closure(&self.graph, root, hierarchy_family, direction)
                .into_iter()
                .filter_map(|key| self.member_id(key))
                .collect();
        let record = FoldRecord::new(
            mint_fold_id(),
            source_scope,
            members,
            FoldStandIn::Member(root_id.clone()),
            Some(FoldRule::Descendants {
                root: root_id,
                family: edge_family_name(hierarchy_family).to_string(),
                direction: direction.into(),
            }),
            None,
        )
        .ok()?;
        self.add_fold(record, true)
    }

    /// Expand one fold, restoring its members at their unchanged layout
    /// positions and selecting them for immediate orientation.
    pub fn expand_fold(&mut self, id: &FoldId) -> bool {
        let Some(index) = self.fold_index(id) else {
            return false;
        };
        let Some(projection) = project_fold(&self.graph, &self.folds[index]) else {
            return false;
        };
        self.push_fold_undo();
        self.folds.remove(index);
        self.fold_press = None;
        self.selected = projection.members.into_iter().collect();
        self.selected_edges.clear();
        true
    }

    /// The fold covering `member`, as its stand-in or a hidden member.
    pub fn fold_of_member(&self, member: uuid::Uuid) -> Option<FoldId> {
        let member = member.to_string();
        self.folds
            .iter()
            .find(|record| record.contains(&member))
            .map(|record| record.id.clone())
    }

    /// Expand the fold a selected member stands in for, else the most recent
    /// fold. What a host's "expand fold" command does.
    pub fn expand_selected_fold(&mut self) -> bool {
        let selected = self
            .folds
            .iter()
            .find(|record| {
                record.stand_in_member().is_some_and(|member| {
                    self.selected
                        .iter()
                        .any(|&key| self.member_id(key).as_deref() == Some(member))
                })
            })
            .or(self.folds.last())
            .map(|record| record.id.clone());
        selected.is_some_and(|id| self.expand_fold(&id))
    }

    /// Give a fold the host's own words, or take them away, as one undo
    /// step. A blank label is refused.
    pub fn set_fold_label(&mut self, id: &FoldId, label: Option<String>) -> bool {
        let Some(index) = self.fold_index(id) else {
            return false;
        };
        let mut record = self.folds[index].clone();
        record.label = label;
        if record.validate().is_err() || record == self.folds[index] {
            return false;
        }
        self.push_fold_undo();
        self.folds[index] = record;
        true
    }

    /// Revert the last fold, expand or relabel. The source graph and its
    /// physics have never changed, so this only restores a prior fold set.
    pub fn undo_fold(&mut self) -> bool {
        let Some(previous) = self.fold_undo.pop() else {
            return false;
        };
        if !self.fold_view_state_is_current(&previous) {
            return false;
        }
        self.fold_redo.push(self.fold_view_state());
        self.apply_fold_view_state(previous);
        true
    }

    /// Reapply a previously undone fold, expand or relabel.
    pub fn redo_fold(&mut self) -> bool {
        let Some(next) = self.fold_redo.pop() else {
            return false;
        };
        if !self.fold_view_state_is_current(&next) {
            return false;
        }
        self.fold_undo.push(self.fold_view_state());
        self.apply_fold_view_state(next);
        true
    }

    /// Every fold that projects against the live graph, in the order they
    /// were made. A stale record remains durable state but does not
    /// partially render. (Was `active_fold_projection`, for one fold.)
    pub fn fold_projections(&self) -> Vec<FoldProjection> {
        self.folds
            .iter()
            .filter_map(|record| project_fold(&self.graph, record))
            .collect()
    }

    /// Where each summary body is drawn, world px: its members' centroid.
    /// A member stand-in is an ordinary visible node, so it is not here.
    /// (With [`Self::fold_hidden_keys`], the successor of reading
    /// `active_fold_projection().summary_center(..)` for framing.)
    pub fn fold_summary_centers(&self) -> Vec<PortablePoint> {
        self.fold_projections()
            .iter()
            .filter(|projection| projection.stand_in.is_none())
            .filter_map(|projection| projection.summary_center(|key| self.world_position(key)))
            .collect()
    }

    /// Every node a current fold hides, for one pass over many nodes; the
    /// per-node form is [`Self::node_in_scope`].
    pub fn fold_hidden_keys(&self) -> std::collections::HashSet<NodeKey> {
        self.fold_projections()
            .iter()
            .flat_map(FoldProjection::hidden)
            .collect()
    }

    /// The durable records a host places in the pane's view intent. They
    /// name members by stable id; nothing of the canvas's drawing or physics
    /// is in them.
    pub fn fold_records(&self) -> &[FoldRecord] {
        &self.folds
    }

    /// Replace the fold set with persisted records, keeping each that still
    /// projects completely and shares no member with one kept before it.
    /// Returns the records refused, intact, so the host can say why rather
    /// than silently fold a different set. History is kept: restoring is not
    /// an edit.
    pub fn restore_folds(
        &mut self,
        records: impl IntoIterator<Item = FoldRecord>,
    ) -> Vec<FoldRecord> {
        let mut kept: Vec<FoldRecord> = Vec::new();
        let mut refused = Vec::new();
        for record in records {
            let current = fold_keys(&self.graph, &record).is_some();
            if current && kept.iter().all(|active| !active.overlaps(&record)) {
                kept.push(record);
            } else {
                refused.push(record);
            }
        }
        if !kept.is_empty() {
            self.selected.clear();
            self.selected_edges.clear();
        }
        self.folds = kept;
        self.fold_press = None;
        refused
    }

    /// Whether a current fold hides `key`. A membership probe by id first,
    /// so only a fold that covers `key` is resolved against the graph.
    pub(crate) fn fold_hides(&self, key: NodeKey) -> bool {
        if self.folds.is_empty() {
            return false;
        }
        let Some(node) = self.graph.get_node(key) else {
            return false;
        };
        let mut buffer = uuid::Uuid::encode_buffer();
        let id: &str = node.id.hyphenated().encode_lower(&mut buffer);
        self.folds.iter().any(|record| {
            record.contains(id)
                && record.stand_in_member() != Some(id)
                && fold_keys(&self.graph, record).is_some()
        })
    }

    pub(crate) fn node_visible_in_canvas(&self, key: NodeKey) -> bool {
        self.node_in_scope(key)
    }

    /// Where a fold's badge is centred, screen px, given world positions.
    pub(crate) fn fold_badge_anchor(
        &self,
        projection: &FoldProjection,
        position_of: impl FnMut(NodeKey) -> Option<PortablePoint>,
    ) -> Option<(f32, f32)> {
        let z = self.camera.zoom;
        match projection.stand_in {
            Some(key) => {
                let mut position_of = position_of;
                let (x, y) = self.camera.to_screen(position_of(key)?);
                let half = self.node_size(key) * 0.5 * z;
                Some((x + half, y - self.node_height(key) * z - half))
            },
            None => {
                let (x, y) = self
                    .camera
                    .to_screen(projection.summary_center(position_of)?);
                let reach = FOLD_SUMMARY_RADIUS * z;
                Some((x + reach, y - reach))
            },
        }
    }

    fn world_position(&self, key: NodeKey) -> Option<PortablePoint> {
        self.view
            .position_of(key)
            .map(|position| PortablePoint::new(position.x, position.y))
    }

    /// Every current fold's badge at the canvas's present layout.
    pub fn fold_badges(&self) -> Vec<FoldBadge> {
        self.fold_projections()
            .into_iter()
            .filter_map(|projection| {
                let at = self.fold_badge_anchor(&projection, |key| self.world_position(key))?;
                Some(FoldBadge {
                    stand_in: projection
                        .stand_in
                        .and_then(|key| self.graph.get_node(key).map(|node| node.id)),
                    at,
                    text: projection.badge(),
                    label: projection.label,
                    fold: projection.fold_id,
                })
            })
            .collect()
    }

    /// The fold whose stand-in a press at `cursor` reaches: a summary body,
    /// or any stand-in's badge. A member stand-in's own body stays an
    /// ordinary node, picked and dragged as one.
    pub(crate) fn fold_stand_in_at_screen(&self, cursor: (f32, f32)) -> Option<FoldId> {
        let reaches =
            |(x, y): (f32, f32), radius: f32| (cursor.0 - x).hypot(cursor.1 - y) <= radius;
        self.fold_projections().into_iter().find_map(|projection| {
            let position_of = |key| self.world_position(key);
            let badge = self.fold_badge_anchor(&projection, position_of)?;
            let on_body = projection.stand_in.is_none()
                && projection
                    .summary_center(position_of)
                    .map(|center| self.camera.to_screen(center))
                    .is_some_and(|center| reaches(center, FOLD_SUMMARY_RADIUS * self.camera.zoom));
            (on_body || reaches(badge, FOLD_BADGE_REACH)).then_some(projection.fold_id)
        })
    }
}
