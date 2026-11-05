// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Host-neutral projection authoring state.
//!
//! The editor owns a draft and the validation needed to turn it into a
//! definition. Its panels are ordinary [`workbench::Tile`]s, so a host can
//! split, stack, and tear them out through the same reusable workspace contract
//! as any other Genet host. It deliberately has no graph, endpoint, or authority
//! handle. A host supplies [`ProjectionDefinitionSink`] when it elects to
//! persist a valid definition.
//!
//! Behind the `authoring` feature: the editor carries state between calls
//! (a draft, an undo history, a workbench), which the closed-form solvers do
//! not, so a consumer that only places scenes does not take it.

use std::mem::{Discriminant, discriminant};

use backstep::History;
use scenograph::{
    Appearance, Arrangement, Channel, DynamicsSlot, Encoding, Interaction, ProjectionDefinition,
    ProjectionDraft, Provenance, Reading, RevisionEvidence, SourceBinding, ValidationIssue,
};
use serde::{Deserialize, Serialize};
use workbench::{ContentSource, Tile, TileEvent, TileId, TileTree, Workbench, WorkbenchOutcome};

use crate::arrangement::with_kind;
use crate::registry::SolverRegistry;

/// The plain panels a host may expose for projection authoring.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionPanel {
    #[default]
    Source,
    Reading,
    Encoding,
    Arrangement,
    Interaction,
    Preview,
    Provenance,
}

impl ProjectionPanel {
    pub const ALL: [Self; 7] = [
        Self::Source,
        Self::Reading,
        Self::Encoding,
        Self::Arrangement,
        Self::Interaction,
        Self::Preview,
        Self::Provenance,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Source => "Source",
            Self::Reading => "Reading",
            Self::Encoding => "Encoding",
            Self::Arrangement => "Arrangement",
            Self::Interaction => "Interaction",
            Self::Preview => "Preview",
            Self::Provenance => "Provenance",
        }
    }

    /// Stable Workbench identity for this editor tool.
    pub const fn tile_id(self) -> TileId {
        TileId(match self {
            Self::Source => 1,
            Self::Reading => 2,
            Self::Encoding => 3,
            Self::Arrangement => 4,
            Self::Interaction => 5,
            Self::Preview => 6,
            Self::Provenance => 7,
        })
    }

    /// Open content-lane id the host resolves to the panel's contents.
    pub const fn content_id(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Reading => "reading",
            Self::Encoding => "encoding",
            Self::Arrangement => "arrangement",
            Self::Interaction => "interaction",
            Self::Preview => "preview",
            Self::Provenance => "provenance",
        }
    }

    /// Recover the editor tool represented by a Workbench tile.
    pub const fn from_tile_id(id: TileId) -> Option<Self> {
        match id.0 {
            1 => Some(Self::Source),
            2 => Some(Self::Reading),
            3 => Some(Self::Encoding),
            4 => Some(Self::Arrangement),
            5 => Some(Self::Interaction),
            6 => Some(Self::Preview),
            7 => Some(Self::Provenance),
            _ => None,
        }
    }
}

/// The `ContentSource::Open` kind every editor panel tile carries; the host
/// resolves it with [`ProjectionPanel::content_id`].
pub const PROJECTION_EDITOR_PANEL_KIND: &str = "scenomise.projection-editor.panel";

fn projection_editor_workbench() -> Workbench {
    let tabs = ProjectionPanel::ALL
        .into_iter()
        .map(|panel| Tile {
            id: panel.tile_id(),
            title: panel.label().to_owned(),
            content: ContentSource::Open {
                kind: PROJECTION_EDITOR_PANEL_KIND.to_owned(),
                id: panel.content_id().to_owned(),
            },
            accent: None,
        })
        .collect();
    Workbench::new(TileTree::stack(tabs, 0))
}

/// Typed messages understood by the reducer. Hosts can map these to any UI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EditorAction {
    SelectPanel(ProjectionPanel),
    SetId(String),
    SetLabel(String),
    SetSource(SourceBinding),
    SetReading(Reading),
    SetEncoding(Encoding),
    SetArrangement(Arrangement),
    SetDynamics(Option<DynamicsSlot>),
    ApplyComparison {
        arrangement: Arrangement,
        dynamics: Option<DynamicsSlot>,
    },
    SetInteraction(Interaction),
    SetAppearance(Appearance),
    SetProvenance(Provenance),
}

/// The result of reducing one editor action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReduceResult {
    Changed,
    PanelChanged,
}

/// How long edits to one field keep coalescing into one undo step, between
/// records, by the host's clock.
pub const COALESCE_WINDOW_MS: u64 = 400;

/// One undo step: the draft and the panel it was edited in. The workbench
/// layout is workspace, not document, and is never recorded.
type EditorStep = (ProjectionDraft, ProjectionPanel);

/// Stateful editor boundary, independent of a widget toolkit or host.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectionEditor {
    draft: ProjectionDraft,
    panel: ProjectionPanel,
    workspace: Workbench,
    history: History<EditorStep, Discriminant<EditorAction>>,
}

impl ProjectionEditor {
    pub fn new(draft: ProjectionDraft) -> Self {
        Self {
            draft,
            panel: ProjectionPanel::default(),
            workspace: projection_editor_workbench(),
            history: History::new().with_window(COALESCE_WINDOW_MS),
        }
    }

    pub fn draft(&self) -> &ProjectionDraft {
        &self.draft
    }

    pub fn panel(&self) -> ProjectionPanel {
        self.panel
    }

    /// The reusable split/tab arrangement for the editor's tool panels.
    pub fn workspace(&self) -> &Workbench {
        &self.workspace
    }

    /// Apply a Workbench gesture without granting graph or persistence authority.
    ///
    /// A valid outside drop is returned as a typed host effect and retains the
    /// panel until the host accepts tearout custody.
    pub fn apply_workspace(&mut self, event: &TileEvent) -> WorkbenchOutcome {
        let outcome = self.workspace.apply(event);
        if outcome.changed() {
            if let TileEvent::Activated(id) = event
                && let Some(panel) = ProjectionPanel::from_tile_id(*id)
            {
                self.panel = panel;
            }
            if self.workspace.tree().find(self.panel.tile_id()).is_none()
                && let Some(panel) = first_active_panel(self.workspace.tree())
            {
                self.panel = panel;
            }
        }
        outcome
    }

    /// Apply one action. A draft edit that changes the draft records an undo
    /// step first, coalescing with the run in progress when it sets the same
    /// field within [`COALESCE_WINDOW_MS`] of `now_ms`, the host's clock.
    pub fn reduce(&mut self, action: EditorAction, now_ms: u64) -> ReduceResult {
        if let EditorAction::SelectPanel(panel) = action {
            self.show_panel(panel);
            return ReduceResult::PanelChanged;
        }
        let before = (self.draft.clone(), self.panel);
        let key = discriminant(&action);
        let result = self.apply_draft(action);
        if self.draft != before.0 {
            self.history.record(before, Some(key), now_ms);
        }
        result
    }

    /// Restore the draft before the last undo step and return to the panel it
    /// was edited in. `false` when there is nothing to undo.
    pub fn undo(&mut self) -> bool {
        let current = (self.draft.clone(), self.panel);
        self.history
            .undo(current)
            .map(|step| self.restore(step))
            .is_some()
    }

    /// Re-apply the last undone step. `false` when there is nothing to redo.
    pub fn redo(&mut self) -> bool {
        let current = (self.draft.clone(), self.panel);
        self.history
            .redo(current)
            .map(|step| self.restore(step))
            .is_some()
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Whether the draft differs from its last save (or from where it opened).
    pub fn is_dirty(&self) -> bool {
        self.history.is_dirty()
    }

    /// End the coalescing run, so the next edit is its own undo step (the end
    /// of a drag, focus leaving a field).
    pub fn break_run(&mut self) {
        self.history.break_run();
    }

    fn restore(&mut self, (draft, panel): EditorStep) {
        self.draft = draft;
        self.show_panel(panel);
    }

    fn show_panel(&mut self, panel: ProjectionPanel) {
        let _ = self.workspace.apply(&TileEvent::Activated(panel.tile_id()));
        self.panel = panel;
    }

    fn apply_draft(&mut self, action: EditorAction) -> ReduceResult {
        match action {
            EditorAction::SelectPanel(panel) => {
                self.show_panel(panel);
                ReduceResult::PanelChanged
            },
            EditorAction::SetId(value) => {
                self.draft.id = value;
                ReduceResult::Changed
            },
            EditorAction::SetLabel(value) => {
                self.draft.label = value;
                ReduceResult::Changed
            },
            EditorAction::SetSource(value) => {
                self.draft.source = value;
                ReduceResult::Changed
            },
            EditorAction::SetReading(value) => {
                self.draft.reading = value;
                ReduceResult::Changed
            },
            EditorAction::SetEncoding(value) => {
                self.draft.encoding = value;
                ReduceResult::Changed
            },
            EditorAction::SetArrangement(value) => {
                self.draft.arrangement = value;
                ReduceResult::Changed
            },
            EditorAction::SetDynamics(value) => {
                self.draft.dynamics = value;
                ReduceResult::Changed
            },
            EditorAction::ApplyComparison {
                arrangement,
                dynamics,
            } => {
                self.draft.arrangement = arrangement;
                self.draft.dynamics = dynamics;
                ReduceResult::Changed
            },
            EditorAction::SetInteraction(value) => {
                self.draft.interaction = value;
                ReduceResult::Changed
            },
            EditorAction::SetAppearance(value) => {
                self.draft.appearance = value;
                ReduceResult::Changed
            },
            EditorAction::SetProvenance(value) => {
                self.draft.provenance = value;
                ReduceResult::Changed
            },
        }
    }

    pub fn validate(&self) -> Result<(), Vec<ValidationIssue>> {
        self.draft.validate()
    }

    /// Persist only a validated, immutable definition through host policy.
    /// A successful save marks the draft clean.
    pub fn save<S: ProjectionDefinitionSink>(
        &mut self,
        sink: &mut S,
    ) -> Result<(), SaveError<S::Error>> {
        let definition = self.draft.to_definition().map_err(SaveError::Invalid)?;
        sink.save(&definition).map_err(SaveError::Sink)?;
        self.history.mark_saved();
        Ok(())
    }

    /// Take `draft`, read back from the store after an undo or redo of a save,
    /// as one undo step on the draft's own history, and read clean against
    /// the store (SE20).
    pub fn load_stored(&mut self, draft: ProjectionDraft) {
        if draft != self.draft {
            self.history
                .record((self.draft.clone(), self.panel), None, 0);
            self.draft = draft;
        }
        self.history.mark_saved();
    }

    /// The store no longer holds this draft's definition: it reads unsaved.
    pub fn forget_saved(&mut self) {
        self.history.forget_saved();
    }

    /// Mark the draft clean after the host persisted it by another path.
    pub fn mark_saved(&mut self) {
        self.history.mark_saved();
    }
}

fn first_active_panel(tree: &TileTree) -> Option<ProjectionPanel> {
    match tree {
        TileTree::Stack(stack) => stack
            .tabs
            .get(stack.active)
            .and_then(|tile| ProjectionPanel::from_tile_id(tile.id)),
        TileTree::Split { children, .. } => children
            .iter()
            .find_map(|branch| first_active_panel(&branch.tree)),
    }
}

/// Host persistence is the only effect exposed by the editor.
pub trait ProjectionDefinitionSink {
    type Error;

    fn save(&mut self, definition: &ProjectionDefinition) -> Result<(), Self::Error>;
}

#[derive(Debug, Eq, PartialEq)]
pub enum SaveError<E> {
    Invalid(Vec<ValidationIssue>),
    Sink(E),
}

impl ProjectionEditor {
    /// Host-neutral conversion of a field edit to a reducer action.
    /// Set one draft field by its dotted name, as a form's text box does.
    /// `registry` is the host's, so switching `arrangement.kind` keeps only the
    /// options the new arrangement declares, custom solvers included. An unknown
    /// field changes nothing.
    pub fn edit_field(
        &mut self,
        field: &str,
        value: &str,
        now_ms: u64,
        registry: &SolverRegistry,
    ) -> Result<(), String> {
        let mut draft = self.draft().clone();
        let action = match field {
            "dynamics.spec" => EditorAction::SetDynamics(if value.trim().is_empty() {
                None
            } else {
                Some(
                    DynamicsSlot::from_json(1, value).unwrap_or_else(|_| DynamicsSlot {
                        version: 1,
                        spec: value.to_string(),
                    }),
                )
            }),
            "source.authority" => {
                draft.source.authority = value.to_string();
                EditorAction::SetSource(draft.source)
            },
            "source.domain" => {
                draft.source.domain = value.to_string();
                EditorAction::SetSource(draft.source)
            },
            "source.resource" => {
                draft.source.resource = value.to_string();
                EditorAction::SetSource(draft.source)
            },
            "reading.key" => {
                draft.reading.key = value.to_string();
                EditorAction::SetReading(draft.reading)
            },
            "encoding.x" => {
                draft.encoding.x = Channel::Field(value.to_string());
                EditorAction::SetEncoding(draft.encoding)
            },
            "encoding.y" => {
                draft.encoding.y = Channel::Field(value.to_string());
                EditorAction::SetEncoding(draft.encoding)
            },
            "encoding.label" => {
                draft.encoding.label = Some(Channel::Field(value.to_string()));
                EditorAction::SetEncoding(draft.encoding)
            },
            "arrangement.kind" => {
                EditorAction::SetArrangement(with_kind(&draft.arrangement, value, registry))
            },
            "arrangement.direction" => {
                draft.arrangement.direction = value.to_string();
                EditorAction::SetArrangement(draft.arrangement)
            },
            "arrangement.spacing" => {
                draft.arrangement.spacing = value.parse().unwrap_or(0);
                EditorAction::SetArrangement(draft.arrangement)
            },
            "appearance.realization" => {
                draft.appearance.realization = value.to_string();
                EditorAction::SetAppearance(draft.appearance)
            },
            "appearance.title" => {
                draft.appearance.title = value.to_string();
                EditorAction::SetAppearance(draft.appearance)
            },
            "provenance.author" => {
                draft.provenance.author = value.to_string();
                EditorAction::SetProvenance(draft.provenance)
            },
            "provenance.source_revision" => {
                draft.provenance.source_revision = Some(value.into());
                draft.provenance.revision_evidence = RevisionEvidence::PublicGeneration;
                EditorAction::SetProvenance(draft.provenance)
            },
            "provenance.note" => {
                draft.provenance.note = value.to_string();
                EditorAction::SetProvenance(draft.provenance)
            },
            _ => return Err(format!("Unknown projection field: {field}")),
        };
        self.reduce(action, now_ms);
        Ok(())
    }
}

#[cfg(test)]
#[path = "authoring/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "authoring/field_adapter_tests.rs"]
mod field_adapter_tests;
