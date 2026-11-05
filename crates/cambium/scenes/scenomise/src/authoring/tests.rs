// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use scenograph::PROJECTION_DEFINITION_VERSION;
use workbench::{DropTarget, WorkbenchEffect};

#[test]
fn dynamics_is_edited_as_one_undo_step_and_saved_in_the_definition() {
    let mut editor = ProjectionEditor::new(valid_draft());
    let slot = DynamicsSlot::from_json(1, r#"{"root":{"preset":"spring.rapier"}}"#).unwrap();
    editor.reduce(EditorAction::SetDynamics(Some(slot.clone())), 0);
    assert_eq!(editor.draft().dynamics, Some(slot.clone()));
    assert!(editor.undo());
    assert_eq!(editor.draft().dynamics, None);
    assert!(editor.redo());
    struct Sink(Option<ProjectionDefinition>);
    impl ProjectionDefinitionSink for Sink {
        type Error = String;
        fn save(&mut self, definition: &ProjectionDefinition) -> Result<(), String> {
            self.0 = Some(definition.clone());
            Ok(())
        }
    }
    let mut sink = Sink(None);
    editor.save(&mut sink).unwrap();
    assert_eq!(sink.0.unwrap().dynamics, Some(slot));
}

pub(super) fn valid_draft() -> ProjectionDraft {
    ProjectionDraft {
        dynamics: None,
        version: PROJECTION_DEFINITION_VERSION,
        id: "notes-by-topic".into(),
        label: "Notes by topic".into(),
        source: SourceBinding {
            authority: "local-device".into(),
            domain: "notes".into(),
            resource: "graph:notes".into(),
        },
        reading: Reading {
            kind: "nodes".into(),
            key: "topic".into(),
            value: None,
        },
        encoding: Encoding {
            x: Channel::Field("topic_x".into()),
            y: Channel::Field("topic_y".into()),
            color: Some(Channel::Field("kind".into())),
            label: Some(Channel::Field("title".into())),
        },
        arrangement: Arrangement::default(),
        interaction: Interaction::default(),
        appearance: Appearance {
            realization: "canvas".into(),
            title: "Notes by topic".into(),
            theme: "light".into(),
        },
        provenance: Provenance {
            author: "mark".into(),
            source_revision: Some("rev-7".into()),
            revision_evidence: RevisionEvidence::PublicGeneration,
            note: "fixture".into(),
        },
    }
}

fn label(editor: &ProjectionEditor) -> &str {
    &editor.draft().label
}

#[test]
fn undo_restores_the_draft_and_returns_to_its_panel() {
    let mut editor = ProjectionEditor::new(valid_draft());
    editor.reduce(EditorAction::SelectPanel(ProjectionPanel::Provenance), 0);
    editor.reduce(EditorAction::SetLabel("Edited".into()), 0);
    editor.reduce(EditorAction::SelectPanel(ProjectionPanel::Preview), 10);
    assert!(editor.undo());
    assert_eq!(label(&editor), "Notes by topic");
    assert_eq!(editor.panel(), ProjectionPanel::Provenance);
    assert!(!editor.can_undo(), "panel changes record nothing");
    assert!(editor.redo());
    assert_eq!(label(&editor), "Edited");
    assert!(editor.can_undo() && !editor.can_redo());
}

#[test]
fn edits_to_one_field_coalesce_within_the_window() {
    let mut editor = ProjectionEditor::new(valid_draft());
    for (at, text) in [(1_000, "N"), (1_200, "Ne"), (1_500, "New")] {
        editor.reduce(EditorAction::SetLabel(text.into()), at);
    }
    editor.reduce(
        EditorAction::SetLabel("New!".into()),
        1_500 + COALESCE_WINDOW_MS + 1,
    );
    assert!(editor.undo());
    assert_eq!(label(&editor), "New", "the lapsed edit is its own step");
    assert!(editor.undo());
    assert_eq!(
        label(&editor),
        "Notes by topic",
        "the typed run is one step"
    );
    assert!(!editor.can_undo());
}

#[test]
fn another_field_or_a_broken_run_starts_a_step() {
    let mut editor = ProjectionEditor::new(valid_draft());
    editor.reduce(EditorAction::SetLabel("A".into()), 0);
    editor.reduce(EditorAction::SetId("a".into()), 1);
    editor.break_run();
    editor.reduce(EditorAction::SetId("ab".into()), 2);
    assert!(editor.undo());
    assert_eq!(editor.draft().id, "a");
    assert!(editor.undo());
    assert_eq!(editor.draft().id, "notes-by-topic");
    assert!(editor.undo());
    assert_eq!(label(&editor), "Notes by topic");
}

#[test]
fn an_unchanged_value_records_nothing() {
    let mut editor = ProjectionEditor::new(valid_draft());
    editor.reduce(EditorAction::SetLabel("Notes by topic".into()), 0);
    assert!(!editor.can_undo());
    assert!(!editor.is_dirty());
}

#[test]
fn a_save_marks_clean_and_undoing_past_it_reads_dirty() {
    let mut editor = ProjectionEditor::new(valid_draft());
    let mut sink = FixtureSink { saved: Vec::new() };
    assert!(!editor.is_dirty());
    editor.reduce(EditorAction::SetLabel("Saved".into()), 0);
    assert!(editor.is_dirty());
    editor.save(&mut sink).expect("valid draft saves");
    assert!(!editor.is_dirty());
    editor.reduce(EditorAction::SetLabel("After".into()), 10_000);
    assert!(editor.is_dirty());
    editor.undo();
    assert!(!editor.is_dirty(), "back at the save");
    editor.undo();
    assert!(editor.is_dirty(), "before the save");
}

#[test]
fn reducer_edits_typed_draft_and_panel() {
    let mut editor = ProjectionEditor::new(valid_draft());
    assert_eq!(
        editor.reduce(EditorAction::SetLabel("Edited".into()), 0),
        ReduceResult::Changed
    );
    assert_eq!(
        editor.reduce(EditorAction::SelectPanel(ProjectionPanel::Preview), 0),
        ReduceResult::PanelChanged
    );
    assert_eq!(editor.draft().label, "Edited");
    assert_eq!(editor.panel(), ProjectionPanel::Preview);
}

#[test]
fn editor_panels_are_open_workbench_tiles() {
    let editor = ProjectionEditor::new(valid_draft());
    let tiles = editor.workspace().tree().tiles();
    assert_eq!(tiles.len(), ProjectionPanel::ALL.len());
    for panel in ProjectionPanel::ALL {
        let tile = editor
            .workspace()
            .tree()
            .find(panel.tile_id())
            .expect("every editor panel has a tile");
        assert_eq!(tile.title, panel.label());
        assert_eq!(
            tile.content,
            ContentSource::Open {
                kind: PROJECTION_EDITOR_PANEL_KIND.to_owned(),
                id: panel.content_id().to_owned(),
            }
        );
    }
}

#[test]
fn workspace_activation_selects_panel_and_tearout_preserves_it() {
    let mut editor = ProjectionEditor::new(valid_draft());
    assert_eq!(
        editor.apply_workspace(&TileEvent::Activated(ProjectionPanel::Preview.tile_id())),
        WorkbenchOutcome::Applied
    );
    assert_eq!(editor.panel(), ProjectionPanel::Preview);

    let before = editor.workspace().tree().clone();
    assert_eq!(
        editor.apply_workspace(&TileEvent::Dragged {
            tile: ProjectionPanel::Preview.tile_id(),
            to: DropTarget::Outside,
        }),
        WorkbenchOutcome::Effect(WorkbenchEffect::TearOut {
            tile: ProjectionPanel::Preview.tile_id(),
        })
    );
    assert_eq!(editor.workspace().tree(), &before);
    assert_eq!(editor.panel(), ProjectionPanel::Preview);
}

#[test]
fn validation_reports_useful_fields() {
    let mut draft = valid_draft();
    draft.source.domain.clear();
    draft.arrangement.spacing = 0;
    draft
        .provenance
        .source_revision
        .as_mut()
        .expect("fixture has source revision")
        .clear();
    let issues = draft.validate().expect_err("invalid draft");
    assert!(issues.iter().any(|issue| issue.field == "source.domain"));
    assert!(
        issues
            .iter()
            .any(|issue| issue.field == "arrangement.spacing")
    );
    assert!(
        issues
            .iter()
            .any(|issue| issue.field == "provenance.source_revision")
    );
}

#[test]
fn unsupported_version_cannot_be_promoted_or_saved() {
    let mut draft = valid_draft();
    draft.version += 1;
    let issues = draft.to_definition().expect_err("version is invalid");
    assert!(issues.iter().any(|issue| issue.field == "version"));
}

struct FixtureSink {
    saved: Vec<ProjectionDefinition>,
}

impl ProjectionDefinitionSink for FixtureSink {
    type Error = &'static str;

    fn save(&mut self, definition: &ProjectionDefinition) -> Result<(), Self::Error> {
        self.saved.push(definition.clone());
        Ok(())
    }
}

#[test]
fn save_uses_fixture_sink_and_serialization_is_repeatable() {
    let mut editor = ProjectionEditor::new(valid_draft());
    let mut sink = FixtureSink { saved: Vec::new() };
    editor.save(&mut sink).expect("valid draft saves");
    assert_eq!(sink.saved[0].id, valid_draft().id);
    let definition = editor.draft().to_definition().expect("valid definition");
    let first = definition.to_json_bytes().expect("definition serializes");
    let second = definition.to_json_bytes().expect("definition serializes");
    assert_eq!(first, second);
}

#[test]
fn invalid_draft_never_reaches_sink() {
    let mut draft = valid_draft();
    draft.id.clear();
    let mut editor = ProjectionEditor::new(draft);
    let mut sink = FixtureSink { saved: Vec::new() };
    let error = editor
        .save(&mut sink)
        .expect_err("invalid draft is refused");
    assert!(matches!(error, SaveError::Invalid(_)));
    assert!(sink.saved.is_empty());
}
