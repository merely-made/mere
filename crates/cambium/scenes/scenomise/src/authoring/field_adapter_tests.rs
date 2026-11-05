// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::tests::valid_draft;
use super::*;
use crate::arrangement::with_option;

#[test]
fn field_edit_uses_the_draft_history_and_rejects_unknown_fields_atomically() {
    let mut editor = ProjectionEditor::new(valid_draft());
    let original = editor.draft().clone();
    editor
        .edit_field(
            "appearance.title",
            "Retained title",
            1,
            &SolverRegistry::new(),
        )
        .unwrap();
    assert_eq!(editor.draft().appearance.title, "Retained title");
    assert!(editor.undo());
    assert_eq!(editor.draft(), &original);
    let before = editor.draft().clone();
    assert!(
        editor
            .edit_field("appearance.unknown", "ignored", 2, &SolverRegistry::new())
            .is_err()
    );
    assert_eq!(editor.draft(), &before);
}

#[test]
fn dynamics_field_preserves_slot_history_and_refuses_invalid_json() {
    let mut editor = ProjectionEditor::new(valid_draft());
    let json = r#"{"root":{"preset":"spring.rapier"}}"#;
    editor
        .edit_field("dynamics.spec", json, 1, &SolverRegistry::new())
        .unwrap();
    let slot = editor.draft().dynamics.clone();
    assert_eq!(slot, Some(DynamicsSlot::from_json(1, json).unwrap()));
    assert!(editor.undo());
    assert_eq!(editor.draft().dynamics, None);
    assert!(editor.redo());
    assert_eq!(editor.draft().dynamics, slot);
    editor.break_run();
    editor
        .edit_field("dynamics.spec", "not JSON", 1000, &SolverRegistry::new())
        .unwrap();
    assert_eq!(editor.draft().dynamics.as_ref().unwrap().spec, "not JSON");
    assert!(editor.draft().to_definition().is_err());
    editor.break_run();
    editor
        .edit_field("dynamics.spec", "", 2000, &SolverRegistry::new())
        .unwrap();
    assert_eq!(editor.draft().dynamics, None);
    assert!(editor.undo());
    assert!(editor.draft().to_definition().is_err());
}

#[test]
fn arrangement_field_switch_uses_declared_options_and_invalid_input_stays_refused() {
    let mut editor = ProjectionEditor::new(valid_draft());
    editor.reduce(
        EditorAction::SetArrangement(with_option(&editor.draft().arrangement, "columns", "3")),
        1,
    );
    editor
        .edit_field("arrangement.kind", "spiral", 1000, &SolverRegistry::new())
        .unwrap();
    assert!(!editor.draft().arrangement.options.contains_key("columns"));
    editor
        .edit_field(
            "arrangement.spacing",
            "not a number",
            2000,
            &SolverRegistry::new(),
        )
        .unwrap();
    assert!(editor.draft().to_definition().is_err());
}
