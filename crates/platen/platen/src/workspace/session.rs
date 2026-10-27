// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The host's scoped Forme editing session. Draft history is private; saved
//! layout history and the canonical workspace commit together in one slot.
//! No document content, resource metadata or graph source is snapshotted.
use super::model::{FormeWorkspace, NoPreferences, WorkspacePreferences};
use crate::projection_geometry::TreeGeometry;
use backstep::History;
use forme::Arrangement;
use kernel::graph::Graph;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const HISTORY_LIMIT: usize = 32;
const RECORD_LIMIT: usize = 4 * 1024 * 1024;

/// Temporary presentation of the same decision used to build a drop candidate.
/// Bounds refer to the original target cell, before the candidate rearranges it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DropEffect {
    Stack,
    Split(workbench::Edge),
    Add,
}
impl DropEffect {
    pub fn label(self) -> &'static str {
        match self {
            Self::Stack => "Stack as tabs",
            Self::Split(workbench::Edge::Left) => "Split left",
            Self::Split(workbench::Edge::Right) => "Split right",
            Self::Split(workbench::Edge::Top) => "Split above",
            Self::Split(workbench::Edge::Bottom) => "Split below",
            Self::Add => "Add to Forme",
        }
    }
    pub fn preview_bounds(self, [x, y, r, b]: [f32; 4]) -> [f32; 4] {
        match self {
            Self::Split(workbench::Edge::Left) => [x, y, x + (r - x) / 2., b],
            Self::Split(workbench::Edge::Right) => [x + (r - x) / 2., y, r, b],
            Self::Split(workbench::Edge::Top) => [x, y, r, y + (b - y) / 2.],
            Self::Split(workbench::Edge::Bottom) => [x, y + (b - y) / 2., r, b],
            _ => [x, y, r, b],
        }
    }
}
pub struct DropProposal<P = NoPreferences> {
    pub workspace: FormeWorkspace<P>,
    pub bounds: [f32; 4],
    pub effect: DropEffect,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LayoutState {
    arrangement: Arrangement,
    geometry: Option<TreeGeometry>,
    bounds: [f32; 4],
}
impl LayoutState {
    fn capture<P: WorkspacePreferences>(workspace: &FormeWorkspace<P>) -> Self {
        Self {
            arrangement: workspace.document.arrangement.clone(),
            geometry: workspace.geometry.clone(),
            bounds: workspace.bounds,
        }
    }
    fn restore<P: WorkspacePreferences>(&self, workspace: &mut FormeWorkspace<P>) {
        workspace.document.arrangement = self.arrangement.clone();
        workspace.geometry = self.geometry.clone();
        workspace.bounds = self.bounds;
    }
}

/// Current flat extension of the v1 workspace record. Its history version is
/// required so obsolete flat records cannot be mistaken for a current save.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct SavedRecord<P> {
    #[serde(flatten)]
    workspace: FormeWorkspace<P>,
    history_version: u16,
    layout_undo: Vec<LayoutState>,
    layout_redo: Vec<LayoutState>,
}
impl<P: WorkspacePreferences> SavedRecord<P> {
    fn new(workspace: FormeWorkspace<P>) -> Self {
        Self {
            workspace,
            history_version: 1,
            layout_undo: Vec::new(),
            layout_redo: Vec::new(),
        }
    }
    fn validate(&self, graph_id: Uuid) -> Result<(), String> {
        self.workspace.validate(graph_id)?;
        if self.history_version != 1
            || self.layout_undo.len() > HISTORY_LIMIT
            || self.layout_redo.len() > HISTORY_LIMIT
        {
            return Err("unsupported or oversized Forme layout history".into());
        }
        for state in self.layout_undo.iter().chain(&self.layout_redo) {
            let mut workspace = self.workspace.clone();
            state.restore(&mut workspace);
            workspace.validate(graph_id)?;
        }
        Ok(())
    }
}
struct Draft<P> {
    workspace: FormeWorkspace<P>,
    history: History<LayoutState, String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SaveKind<P = NoPreferences> {
    Apply,
    Undo,
    Redo,
    Visibility,
    Preferences {
        saved_base: P,
        saved_value: P,
        draft_base: Option<P>,
        draft_value: Option<P>,
        update_timestamp: bool,
    },
}
/// Prepared writes never change committed state until the store acknowledges
/// them. The session is frozen while its proposal is in flight.
#[derive(Clone)]
pub struct PreparedSave<P = NoPreferences> {
    next: SavedRecord<P>,
    kind: SaveKind<P>,
    token: Uuid,
}
impl<P: WorkspacePreferences + Serialize> PreparedSave<P> {
    pub fn graph_id(&self) -> Uuid {
        self.next.workspace.document.graph_id
    }
    pub fn bytes(&self) -> Result<Vec<u8>, String> {
        self.next.validate(self.next.workspace.document.graph_id)?;
        let bytes = serde_json::to_vec(&self.next).map_err(|e| e.to_string())?;
        if bytes.len() > RECORD_LIMIT {
            return Err("Forme history exceeds 4 MiB".into());
        }
        Ok(bytes)
    }
}
pub struct FormeSession<P: WorkspacePreferences = NoPreferences> {
    saved: SavedRecord<P>,
    draft: Option<Draft<P>>,
    busy: bool,
    pending: Option<Uuid>,
}
impl<P: WorkspacePreferences + Serialize + for<'de> Deserialize<'de>> FormeSession<P> {
    pub fn new(workspace: FormeWorkspace<P>) -> Self {
        Self {
            saved: SavedRecord::new(workspace),
            draft: None,
            busy: false,
            pending: None,
        }
    }
    pub fn view(&self) -> &FormeWorkspace<P> {
        self.draft
            .as_ref()
            .map_or(&self.saved.workspace, |d| &d.workspace)
    }
    pub fn committed(&self) -> &FormeWorkspace<P> {
        &self.saved.workspace
    }
    pub fn editing(&self) -> bool {
        self.draft.is_some()
    }
    pub fn busy(&self) -> bool {
        self.busy
    }
    pub fn dirty(&self) -> bool {
        self.draft.as_ref().is_some_and(|d| {
            LayoutState::capture(&d.workspace) != LayoutState::capture(&self.saved.workspace)
        })
    }
    pub fn can_undo(&self) -> bool {
        !self.busy
            && self
                .draft
                .as_ref()
                .map_or(!self.saved.layout_undo.is_empty(), |d| d.history.can_undo())
    }
    pub fn can_redo(&self) -> bool {
        !self.busy
            && self
                .draft
                .as_ref()
                .map_or(!self.saved.layout_redo.is_empty(), |d| d.history.can_redo())
    }
    /// Build a preference save by applying the same narrow edit independently
    /// to the committed preferences and any open draft. Unrelated draft-only
    /// navigation state can therefore never leak into the saved workspace.
    pub fn preferences_update(
        &self,
        update_timestamp: bool,
        update: impl Fn(&mut P),
    ) -> SaveKind<P> {
        let mut saved_value = self.saved.workspace.preferences.clone();
        update(&mut saved_value);
        let draft_base = self
            .draft
            .as_ref()
            .map(|draft| draft.workspace.preferences.clone());
        let draft_value = self.draft.as_ref().map(|draft| {
            let mut value = draft.workspace.preferences.clone();
            update(&mut value);
            value
        });
        SaveKind::Preferences {
            saved_base: self.saved.workspace.preferences.clone(),
            saved_value,
            draft_base,
            draft_value,
            update_timestamp,
        }
    }
    fn idle(&self) -> Result<(), String> {
        if self.busy {
            Err("Wait for the Forme save to finish".into())
        } else {
            Ok(())
        }
    }
    pub fn unlock(&mut self) -> Result<(), String> {
        self.idle()?;
        if self.draft.is_none() {
            let mut workspace = self.saved.workspace.clone();
            workspace.locked = false;
            self.draft = Some(Draft {
                workspace,
                history: History::new().with_cap(200).with_window(400),
            });
        }
        Ok(())
    }
    /// Record one accepted layout gesture. The host supplies a coalescing key
    /// for repeated divider samples, and no key for a completed drag/drop.
    pub fn edit(
        &mut self,
        key: Option<String>,
        now_ms: u64,
        change: impl FnOnce(&mut FormeWorkspace<P>) -> Result<(), String>,
    ) -> Result<(), String> {
        self.idle()?;
        let draft = self
            .draft
            .as_mut()
            .ok_or("Unlock the forme to change its arrangement")?;
        let mut next = draft.workspace.clone();
        change(&mut next)?;
        if next.document.id != draft.workspace.document.id
            || next.document.graph_id != draft.workspace.document.graph_id
        {
            return Err("An arrangement edit cannot replace its Forme identity".into());
        }
        next.validate(draft.workspace.document.graph_id)?;
        let before = LayoutState::capture(&draft.workspace);
        if LayoutState::capture(&next) != before {
            draft.history.record(before, key, now_ms);
        }
        draft.workspace = next;
        Ok(())
    }
    pub fn draft_undo(&mut self, redo: bool) -> Result<bool, String> {
        self.idle()?;
        let draft = self.draft.as_mut().ok_or("No Forme draft is open")?;
        let current = LayoutState::capture(&draft.workspace);
        let state = if redo {
            draft.history.redo(current)
        } else {
            draft.history.undo(current)
        };
        if let Some(state) = state {
            state.restore(&mut draft.workspace);
            Ok(true)
        } else {
            Ok(false)
        }
    }
    pub fn break_gesture(&mut self) {
        if let Some(d) = &mut self.draft {
            d.history.break_run();
        }
    }
    /// Builds the same graph-admitted edge candidate as a pointer drag. The
    /// embedding host owns installation into its actual canvas surface.
    pub fn edge_preview_candidate(
        &self,
        owner: Uuid,
        moving: Uuid,
        target: Uuid,
        edge: workbench::Edge,
        graph: &Graph,
    ) -> Result<FormeWorkspace<P>, String> {
        self.idle()?;
        if !self.editing() {
            return Err("Unlock the forme to preview a move".into());
        }
        self.view().validate(owner)?;
        let mut candidate = self.view().clone();
        let tile = candidate
            .tile_for_member(moving)
            .ok_or("unknown moving access")?;
        let to = candidate
            .tile_for_member(target)
            .ok_or("unknown target access")?;
        candidate.event(
            workbench::TileEvent::Dragged {
                tile,
                to: workbench::DropTarget::Edge { tile: to, edge },
            },
            graph,
        )?;
        candidate.project_region(owner, graph)?;
        Ok(candidate)
    }
    /// Resolve a handle's world-space drop against the draft's original cells,
    /// not against its changing preview. Central drops tab; edge drops split.
    pub fn drop_preview(
        &self,
        member: Uuid,
        point: (f32, f32),
        graph: &Graph,
    ) -> Result<Option<FormeWorkspace<P>>, String> {
        Ok(self
            .drop_proposal(member, point, graph)?
            .map(|p| p.workspace))
    }
    pub fn drop_proposal(
        &self,
        member: Uuid,
        point: (f32, f32),
        graph: &Graph,
    ) -> Result<Option<DropProposal<P>>, String> {
        self.idle()?;
        if !self.editing() {
            return Err("Unlock the forme to drag tile handles".into());
        }
        if !point.0.is_finite() || !point.1.is_finite() {
            return Err("Invalid drop position".into());
        }
        let region = self.view().region();
        let [left, top, right, bottom] = region.bounds;
        if point.0 < left || point.0 > right || point.1 < top || point.1 > bottom {
            return Ok(None);
        }
        let mut next = self.view().clone();
        next.open(member, graph)?;
        let mut bounds = region.bounds;
        let mut effect = DropEffect::Add;
        let target = region.cells.iter().find(|cell| {
            let [x, y, r, b] = region.cell_world_bounds(cell);
            graph.get_node_by_id(cell.member).is_some()
                && cell.member != member
                && point.0 >= x
                && point.0 <= r
                && point.1 >= y
                && point.1 <= b
        });
        if let Some(cell) = target {
            let [x, y, r, b] = region.cell_world_bounds(cell);
            bounds = [x, y, r, b];
            let (u, v) = ((point.0 - x) / (r - x), (point.1 - y) / (b - y));
            use workbench::{DropTarget, Edge, TileEvent, TilePath};
            let edges = [
                (u, Edge::Left),
                (1. - u, Edge::Right),
                (v, Edge::Top),
                (1. - v, Edge::Bottom),
            ];
            let (distance, edge) = edges
                .into_iter()
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .unwrap();
            let to = if distance < 0.25 {
                effect = DropEffect::Split(edge);
                DropTarget::Edge {
                    tile: next
                        .tile_for_member(cell.member)
                        .ok_or("unknown drop cell")?,
                    edge,
                }
            } else {
                effect = DropEffect::Stack;
                let stack = next
                    .geometry
                    .as_ref()
                    .and_then(|g| stack_path(g, cell.member))
                    .ok_or("unknown tab group")?;
                DropTarget::Stack {
                    stack: TilePath(stack),
                    index: usize::MAX,
                }
            };
            next.event(
                TileEvent::Dragged {
                    tile: next.tile_for_member(member).ok_or("unknown handle")?,
                    to,
                },
                graph,
            )?;
        } else if self.view().members().contains(&member) {
            return Ok(None);
        }
        next.validate(next.document.graph_id)?;
        Ok(Some(DropProposal {
            workspace: next,
            bounds,
            effect,
        }))
    }
    pub fn discard(&mut self) -> Result<(), String> {
        self.idle()?;
        self.draft = None;
        // Legacy unlocked records are treated as committed, not resurrected drafts.
        self.saved.workspace.locked = true;
        Ok(())
    }
    pub fn prepare_save(
        &mut self,
        kind: SaveKind<P>,
        now_ms: u64,
    ) -> Result<PreparedSave<P>, String> {
        self.idle()?;
        let mut next = self.saved.clone();
        match kind.clone() {
            SaveKind::Apply => {
                let draft = self.draft.as_ref().ok_or("No Forme draft is open")?;
                if self.dirty() {
                    next.layout_undo.push(LayoutState::capture(&next.workspace));
                    if next.layout_undo.len() > HISTORY_LIMIT {
                        next.layout_undo.remove(0);
                    }
                    next.layout_redo.clear();
                }
                next.workspace = draft.workspace.clone();
            },
            SaveKind::Undo | SaveKind::Redo => {
                if self.editing() {
                    return Err("Apply or discard the Forme draft before saved undo".into());
                }
                let current = LayoutState::capture(&next.workspace);
                let (from, to) = if kind == SaveKind::Undo {
                    (&mut next.layout_undo, &mut next.layout_redo)
                } else {
                    (&mut next.layout_redo, &mut next.layout_undo)
                };
                let state = from.pop().ok_or("No saved Forme change to restore")?;
                to.push(current);
                state.restore(&mut next.workspace);
            },
            SaveKind::Preferences {
                saved_base,
                saved_value: preferences,
                draft_base,
                draft_value,
                ..
            } => {
                let current_draft = self
                    .draft
                    .as_ref()
                    .map(|draft| draft.workspace.preferences.clone());
                if self.saved.workspace.preferences != saved_base || current_draft != draft_base {
                    return Err("Preferences changed since this save was prepared".into());
                }
                next.workspace.preferences = preferences;
                if let (Some(draft), Some(preferences)) = (&self.draft, draft_value) {
                    let mut candidate = draft.workspace.clone();
                    candidate.preferences = preferences;
                    candidate.validate(candidate.document.graph_id)?;
                }
            },
            SaveKind::Visibility => next.workspace.visible = !next.workspace.visible,
        }
        next.workspace.locked = true;
        if !matches!(
            kind,
            SaveKind::Preferences {
                update_timestamp: false,
                ..
            }
        ) {
            if next.workspace.document.created_at_ms == 0 {
                next.workspace.document.created_at_ms = now_ms;
            }
            next.workspace.document.updated_at_ms = now_ms;
        }
        next.validate(next.workspace.document.graph_id)?;
        self.busy = true;
        let token = Uuid::new_v4();
        self.pending = Some(token);
        Ok(PreparedSave { next, kind, token })
    }
    /// A failed write leaves the complete draft and its history available.
    /// Saved undo/redo also advances only after acknowledgement.
    pub fn finish_save(
        &mut self,
        proposal: PreparedSave<P>,
        result: Result<(), String>,
    ) -> Result<(), String> {
        if self.pending != Some(proposal.token) {
            return Err("Forme save belongs to another or completed request".into());
        }
        self.pending = None;
        self.busy = false;
        result?;
        match proposal.kind {
            SaveKind::Preferences {
                draft_value: preferences,
                ..
            } => {
                if let (Some(draft), Some(preferences)) = (&mut self.draft, preferences) {
                    draft.workspace.preferences = preferences;
                }
            },
            SaveKind::Visibility => {
                if let Some(draft) = &mut self.draft {
                    draft.workspace.visible = proposal.next.workspace.visible;
                }
            },
            _ => self.draft = None,
        }
        self.saved = proposal.next;
        Ok(())
    }
    pub fn from_bytes(bytes: Option<&[u8]>, graph_id: Uuid, graph: &Graph) -> Result<Self, String> {
        let mut saved = match bytes {
            Some(bytes) if bytes.len() > RECORD_LIMIT => {
                return Err("Forme record exceeds 4 MiB".into());
            },
            Some(bytes) => serde_json::from_slice::<SavedRecord<P>>(bytes)
                .map_err(|e| format!("Invalid Forme history: {e}"))?,
            None => SavedRecord::new(FormeWorkspace::<P>::new(graph_id)),
        };
        saved.validate(graph_id)?;
        // Reconcile every reachable layout against current graph membership;
        // history may never resurrect a removed source node.
        saved.workspace.reconcile(graph);
        for state in saved.layout_undo.iter_mut().chain(&mut saved.layout_redo) {
            let mut workspace = saved.workspace.clone();
            state.restore(&mut workspace);
            workspace.reconcile(graph);
            *state = LayoutState::capture(&workspace);
        }
        saved.workspace.locked = true;
        saved.validate(graph_id)?;
        Ok(Self {
            saved,
            draft: None,
            busy: false,
            pending: None,
        })
    }
}

#[derive(Clone, Default)]
pub struct FormeWriteFence {
    next: u64,
    pending: std::collections::BTreeMap<u64, Uuid>,
}
impl FormeWriteFence {
    pub fn retire(&mut self, owner: Uuid) -> u64 {
        self.next = self
            .next
            .checked_add(1)
            .expect("Forme write fence exhausted");
        self.pending.insert(self.next, owner);
        self.next
    }
    pub fn finish(&mut self, ticket: u64) {
        self.pending.remove(&ticket);
    }
    pub fn blocks(&self, owner: Uuid) -> bool {
        self.pending.values().any(|o| *o == owner)
    }
}

/// An async load is admitted only by its current request and actual App owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormeLoadTicket {
    pub owner: Uuid,
    epoch: u64,
}
#[derive(Default)]
pub struct FormeLoadGate {
    epoch: u64,
    active: Option<FormeLoadTicket>,
}
impl FormeLoadGate {
    pub fn request(&mut self, owner: Uuid) -> FormeLoadTicket {
        self.epoch = self
            .epoch
            .checked_add(1)
            .expect("Forme load epoch exhausted");
        let ticket = FormeLoadTicket {
            owner,
            epoch: self.epoch,
        };
        self.active = Some(ticket);
        ticket
    }
    pub fn active(&self) -> Option<FormeLoadTicket> {
        self.active
    }
    pub fn accepts(&self, ticket: FormeLoadTicket, actual: Option<Uuid>) -> bool {
        self.active == Some(ticket) && actual == Some(ticket.owner)
    }
    pub fn finish_loaded<P: WorkspacePreferences + Serialize + for<'de> Deserialize<'de>>(
        &mut self,
        ticket: FormeLoadTicket,
        actual: Option<Uuid>,
        loaded: &FormeSession<P>,
    ) -> bool {
        if loaded.committed().document.graph_id != ticket.owner {
            return false;
        }
        self.finish_error(ticket, actual)
    }
    pub fn finish_error(&mut self, ticket: FormeLoadTicket, actual: Option<Uuid>) -> bool {
        if !self.accepts(ticket, actual) {
            return false;
        }
        self.active = None;
        true
    }
}
fn stack_path(geometry: &TreeGeometry, member: Uuid) -> Option<Vec<usize>> {
    match geometry {
        TreeGeometry::Stack { members, .. } => members.contains(&member).then(Vec::new),
        TreeGeometry::Split { children, .. } => {
            children.iter().enumerate().find_map(|(index, branch)| {
                let mut path = stack_path(&branch.node, member)?;
                path.insert(0, index);
                Some(path)
            })
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::graph::apply::{GraphDelta, add_node, apply_graph_delta};

    fn fixture() -> (Graph, FormeSession, [Uuid; 3]) {
        let mut graph = Graph::new();
        let members = [Uuid::from_u128(1), Uuid::from_u128(2), Uuid::from_u128(3)];
        for member in members {
            add_node(
                &mut graph,
                Some(member),
                "https://example.org/".into(),
                Default::default(),
            );
        }
        let mut workspace = FormeWorkspace::<NoPreferences>::new(Uuid::from_u128(10));
        workspace.open(members[0], &graph).unwrap();
        (graph, FormeSession::new(workspace), members)
    }
    fn persisted(session: &FormeSession) -> Vec<u8> {
        serde_json::to_vec(&session.saved).unwrap()
    }
    fn commit(session: &mut FormeSession, kind: SaveKind, now_ms: u64) -> Vec<u8> {
        let proposal = session.prepare_save(kind, now_ms).unwrap();
        let bytes = proposal.bytes().unwrap();
        session.finish_save(proposal, Ok(())).unwrap();
        bytes
    }

    #[test]
    fn draft_discard_and_acknowledged_apply_preserve_history_boundaries() {
        let (graph, mut session, [a, b, _]) = fixture();
        session.unlock().unwrap();
        session.edit(None, 1, |w| w.open(b, &graph)).unwrap();
        assert_eq!(session.view().members(), vec![a, b]);
        assert!(session.draft_undo(false).unwrap());
        assert_eq!(session.view().members(), vec![a]);
        session.discard().unwrap();
        assert_eq!(session.committed().members(), vec![a]);

        session.unlock().unwrap();
        session.edit(None, 2, |w| w.open(b, &graph)).unwrap();
        let proposal = session.prepare_save(SaveKind::Apply, 3).unwrap();
        assert!(!proposal.bytes().unwrap().is_empty());
        session.finish_save(proposal, Ok(())).unwrap();
        assert_eq!(session.committed().members(), vec![a, b]);
    }

    #[test]
    fn failed_write_retains_draft_and_stale_ack_cannot_commit() {
        let (graph, mut session, [a, b, _]) = fixture();
        session.unlock().unwrap();
        session.edit(None, 1, |w| w.open(b, &graph)).unwrap();
        let proposal = session.prepare_save(SaveKind::Apply, 2).unwrap();
        let bytes = proposal.bytes().unwrap();
        assert!(
            session
                .finish_save(proposal.clone(), Err("offline".into()))
                .is_err()
        );
        assert!(session.editing());
        assert_eq!(session.committed().members(), vec![a]);
        assert!(session.finish_save(proposal, Ok(())).is_err());
        let restored =
            FormeSession::<NoPreferences>::from_bytes(Some(&bytes), Uuid::from_u128(10), &graph)
                .unwrap();
        assert_eq!(restored.committed().members(), vec![a, b]);
    }

    #[test]
    fn load_gate_rejects_superseded_requests_and_mismatched_owners() {
        let mut gate = FormeLoadGate::default();
        let old = gate.request(Uuid::from_u128(1));
        let active = gate.request(Uuid::from_u128(2));
        assert!(!gate.accepts(old, Some(old.owner)));
        assert!(!gate.accepts(active, Some(old.owner)));
        assert!(gate.accepts(active, Some(active.owner)));
        assert!(gate.finish_error(active, Some(active.owner)));
        assert_eq!(gate.active(), None);
    }

    #[test]
    fn current_record_requires_an_explicit_history_version() {
        let (graph, session, _) = fixture();
        let legacy_flat_workspace = serde_json::to_vec(session.view()).unwrap();
        assert!(
            FormeSession::<NoPreferences>::from_bytes(
                Some(&legacy_flat_workspace),
                Uuid::from_u128(10),
                &graph
            )
            .is_err()
        );
        assert_eq!(session.committed().document.graph_id, Uuid::from_u128(10));
    }

    fn member_identities(workspace: &FormeWorkspace) -> Vec<(Uuid, Uuid)> {
        use forme::ArrangementNodeKind;
        workspace
            .document
            .arrangement
            .nodes()
            .filter_map(|node| match node.kind {
                ArrangementNodeKind::MemberIntent { member } => Some((member, node.id.as_uuid())),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn preview_is_non_mutating_and_geometry_edits_retain_forme_member_identity() {
        let (graph, mut session, [a, b, c]) = fixture();
        let id = session.committed().document.id;
        let root = session.committed().document.arrangement.root();
        let identities = member_identities(session.committed());
        session.unlock().unwrap();
        session
            .edit(None, 0, |workspace| workspace.open(b, &graph))
            .unwrap();
        let undo_before_preview = session.can_undo();
        let region = session.view().region();
        let cell = &region.cells[0];
        let [x, y, r, bottom] = region.cell_world_bounds(cell);
        let candidate = session
            .drop_preview(c, (x + 1., (y + bottom) / 2.), &graph)
            .unwrap()
            .unwrap();
        assert_eq!(
            session.view().members().len(),
            2,
            "preview does not commit membership"
        );
        assert_eq!(
            session.can_undo(),
            undo_before_preview,
            "preview does not change draft history"
        );
        assert!(session.draft_undo(false).unwrap());
        assert_eq!(session.view().members(), vec![a]);
        assert!(!session.can_undo(), "preview adds no second undo entry");
        assert!(session.draft_undo(true).unwrap());
        assert_eq!(session.view().members(), vec![a, b]);
        assert_eq!(candidate.document.id, id);
        assert_eq!(candidate.document.arrangement.root(), root);
        for identity in &identities {
            assert!(member_identities(&candidate).contains(identity));
        }
        session
            .edit(None, 1, |workspace| {
                *workspace = candidate;
                Ok(())
            })
            .unwrap();
        assert_eq!(session.view().document.arrangement.root(), root);
        for identity in identities {
            assert!(member_identities(session.view()).contains(&identity));
        }
        assert!(r > x);
    }

    #[test]
    fn draft_history_coalesces_dividers_and_scopes_nested_membership_undo() {
        let (graph, mut session, [a, b, c]) = fixture();
        let original = LayoutState::capture(session.committed());
        session.unlock().unwrap();
        session
            .edit(None, 0, |w| {
                w.open(b, &graph)?;
                w.open(c, &graph)
            })
            .unwrap();
        let added = LayoutState::capture(session.view());
        session
            .edit(None, 1, |w| {
                let mut layout = w.layout();
                layout.move_to_slot_of(c, b);
                layout.split_beside_axis(b, a, workbench::SplitAxis::Column, true);
                w.keep_layout(&layout);
                Ok(())
            })
            .unwrap();
        let nested = LayoutState::capture(session.view());
        for (time, fraction) in [(100, 0.4), (200, 0.3), (300, 0.2)] {
            session
                .edit(Some("divider:root".into()), time, |w| {
                    let mut layout = w.layout();
                    assert!(layout.set_split_fractions(&[], &[fraction, 1. - fraction]));
                    w.keep_layout(&layout);
                    Ok(())
                })
                .unwrap();
        }
        let sized = LayoutState::capture(session.view());
        session.draft_undo(false).unwrap();
        assert_eq!(LayoutState::capture(session.view()), nested);
        session.draft_undo(false).unwrap();
        assert_eq!(LayoutState::capture(session.view()), added);
        session.draft_undo(false).unwrap();
        assert_eq!(LayoutState::capture(session.view()), original);
        assert!(!session.can_undo());
        for _ in 0..3 {
            assert!(session.draft_undo(true).unwrap());
        }
        assert_eq!(LayoutState::capture(session.view()), sized);
    }

    #[test]
    fn discard_keeps_graph_and_other_workspace_unchanged_and_resets_draft_history() {
        let (graph, mut session, [_, _, c]) = fixture();
        let (_, other, _) = fixture();
        let other_before = serde_json::to_vec(other.committed()).unwrap();
        let original = LayoutState::capture(session.committed());
        session.unlock().unwrap();
        session
            .edit(None, 0, |w| {
                w.open(c, &graph)?;
                w.bounds[0] -= 50.;
                Ok(())
            })
            .unwrap();
        session.discard().unwrap();
        assert!(session.view().locked);
        assert_eq!(LayoutState::capture(session.view()), original);
        assert!(graph.get_node_by_id(c).is_some());
        assert_eq!(serde_json::to_vec(other.committed()).unwrap(), other_before);
        session.unlock().unwrap();
        assert!(!session.can_undo());
        assert!(!session.can_redo());
    }

    #[test]
    fn saved_redo_visibility_and_source_are_reconciled_across_reopen() {
        let (mut graph, mut session, [_, _, c]) = fixture();
        let graph_id = session.committed().document.graph_id;
        session.unlock().unwrap();
        session
            .edit(None, 0, |w| {
                w.open(Uuid::from_u128(2), &graph)?;
                w.open(c, &graph)
            })
            .unwrap();
        let applied = commit(&mut session, SaveKind::Apply, 10);
        let tile_id = session.committed().tile_for_member(c);
        session.unlock().unwrap();
        session
            .edit(None, 1, |w| {
                w.bounds[0] -= 50.;
                Ok(())
            })
            .unwrap();
        let visible = commit(&mut session, SaveKind::Visibility, 11);
        assert!(session.editing() && session.dirty());
        session.discard().unwrap();
        assert!(!session.view().visible);
        let key = graph.get_node_key_by_id(c).unwrap();
        apply_graph_delta(
            &mut graph,
            GraphDelta::SetNodeTitle {
                key,
                title: "source edit after layout save".into(),
            },
        );
        let mut reopened =
            FormeSession::<NoPreferences>::from_bytes(Some(&visible), graph_id, &graph).unwrap();
        commit(&mut reopened, SaveKind::Undo, 12);
        assert_eq!(reopened.view().members().len(), 1);
        assert!(!reopened.view().visible);
        let mut reopened = FormeSession::<NoPreferences>::from_bytes(
            Some(&persisted(&reopened)),
            graph_id,
            &graph,
        )
        .unwrap();
        commit(&mut reopened, SaveKind::Redo, 13);
        assert_eq!(reopened.view().members().len(), 3);
        assert_eq!(reopened.view().tile_for_member(c), tile_id);
        assert_eq!(
            graph.get_node(key).unwrap().title,
            "source edit after layout save"
        );
        assert!(
            FormeSession::<NoPreferences>::from_bytes(Some(&applied), graph_id, &graph).is_ok()
        );
    }

    #[test]
    fn removed_graph_members_are_not_resurrected_by_saved_undo_history() {
        let (mut graph, mut session, [_, _, c]) = fixture();
        let graph_id = session.committed().document.graph_id;
        session.unlock().unwrap();
        session
            .edit(None, 0, |w| {
                w.open(Uuid::from_u128(2), &graph)?;
                w.open(c, &graph)
            })
            .unwrap();
        let mut bytes = commit(&mut session, SaveKind::Apply, 1);
        session.unlock().unwrap();
        session
            .edit(None, 2, |w| {
                w.bounds[0] -= 50.;
                Ok(())
            })
            .unwrap();
        bytes = commit(&mut session, SaveKind::Apply, 3);
        let key = graph.get_node_key_by_id(c).unwrap();
        apply_graph_delta(&mut graph, GraphDelta::RemoveNode { key });
        let mut reopened =
            FormeSession::<NoPreferences>::from_bytes(Some(&bytes), graph_id, &graph).unwrap();
        commit(&mut reopened, SaveKind::Undo, 4);
        assert!(!reopened.view().members().contains(&c));
        assert!(
            !reopened
                .view()
                .region()
                .cells
                .iter()
                .any(|cell| cell.member == c)
        );
    }

    #[test]
    fn center_edge_and_outside_drop_proposals_keep_draft_state_until_accepted() {
        let (graph, mut session, [_, b, c]) = fixture();
        session.unlock().unwrap();
        session.edit(None, 0, |w| w.open(b, &graph)).unwrap();
        let region = session.view().region();
        let [x, y, r, bottom] = region.cell_world_bounds(&region.cells[0]);
        let center = session
            .drop_proposal(c, ((x + r) / 2., (y + bottom) / 2.), &graph)
            .unwrap()
            .unwrap();
        assert_eq!(center.effect, DropEffect::Stack);
        assert_eq!(center.workspace.members().len(), 3);
        let edge = session
            .drop_proposal(c, ((x + r) / 2., y + 1.), &graph)
            .unwrap()
            .unwrap();
        assert_eq!(edge.effect, DropEffect::Split(workbench::Edge::Top));
        assert_eq!(
            edge.effect.preview_bounds(edge.bounds),
            [x, y, r, (y + bottom) / 2.]
        );
        assert!(
            session
                .drop_preview(c, (region.bounds[2] + 10., y), &graph)
                .unwrap()
                .is_none()
        );
        assert_eq!(session.view().members().len(), 2);
        assert!(session.dirty());
    }

    #[test]
    fn collapsed_world_geometry_is_rejected_without_changing_draft_or_saved_bytes() {
        let (graph, mut session, [a, b, _]) = fixture();
        session.unlock().unwrap();
        session
            .edit(None, 0, |w| {
                w.open(b, &graph)?;
                let mut layout = w.layout();
                layout.split_beside_axis(a, b, workbench::SplitAxis::Row, true);
                w.keep_layout(&layout);
                Ok(())
            })
            .unwrap();
        let original = LayoutState::capture(session.view());
        let invalid = session.edit(None, 1, |w| {
            let Some(TreeGeometry::Split { children, .. }) = &mut w.geometry else {
                panic!("split expected")
            };
            children[0].fraction = f32::MIN_POSITIVE;
            children[1].fraction = f32::MAX / 2.;
            Ok(())
        });
        assert!(invalid.is_err());
        assert_eq!(LayoutState::capture(session.view()), original);
        let before = persisted(&session);
        let valid_workspace = session.view().clone();
        let Some(TreeGeometry::Split { children, .. }) =
            &mut session.draft.as_mut().unwrap().workspace.geometry
        else {
            panic!("split expected")
        };
        children[0].fraction = f32::MIN_POSITIVE;
        children[1].fraction = f32::MAX / 2.;
        assert!(
            session
                .view()
                .validate(session.view().document.graph_id)
                .is_err()
        );
        let after = serde_json::to_vec(&session.saved).unwrap();
        assert_eq!(after, before);
        let mut record = SavedRecord::new(valid_workspace.clone());
        let mut collapsed = LayoutState::capture(&valid_workspace);
        let Some(TreeGeometry::Split { children, .. }) = &mut collapsed.geometry else {
            panic!("split expected")
        };
        children[0].fraction = f32::MIN_POSITIVE;
        children[1].fraction = f32::MAX / 2.;
        record.layout_undo.push(collapsed);
        let corrupt = serde_json::to_vec(&record).unwrap();
        assert!(
            FormeSession::<NoPreferences>::from_bytes(
                Some(&corrupt),
                session.committed().document.graph_id,
                &graph
            )
            .is_err()
        );
    }

    #[test]
    fn failed_saved_undo_keeps_cursor_and_malformed_history_is_rejected() {
        let (graph, mut session, [_, b, _]) = fixture();
        session.unlock().unwrap();
        session.edit(None, 0, |w| w.open(b, &graph)).unwrap();
        commit(&mut session, SaveKind::Apply, 1);
        let current = LayoutState::capture(session.committed());
        let proposal = session.prepare_save(SaveKind::Undo, 2).unwrap();
        assert!(
            session
                .finish_save(proposal, Err("offline".into()))
                .is_err()
        );
        assert_eq!(LayoutState::capture(session.committed()), current);
        assert!(session.can_undo());
        assert!(!session.can_redo());
        let mut record: serde_json::Value = serde_json::from_slice(&persisted(&session)).unwrap();
        record["history_version"] = 99.into();
        let corrupt = serde_json::to_vec(&record).unwrap();
        assert!(
            FormeSession::<NoPreferences>::from_bytes(
                Some(&corrupt),
                session.committed().document.graph_id,
                &graph
            )
            .is_err()
        );
    }
}
