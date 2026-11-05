// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::presentation::{Property, ViewState};

fn context(id: &str) -> Context {
    Context {
        id: id.into(),
        kind: PrimitiveKind::Node,
        facts: Default::default(),
        states: Default::default(),
        screen_size: Some(64.0),
        previous_matches: Default::default(),
    }
}
fn add(workspace: &mut RuleWorkspace, kind: PrimitiveKind) {
    workspace.begin().unwrap();
    workspace
        .edit(None, 1000, |draft| {
            draft.add(kind);
        })
        .unwrap();
}

#[test]
fn strict_parse_rejects_unknown_nested_fields_and_malformed_draft_remains_undoable() {
    let mut workspace = RuleWorkspace::new(SavedState::empty()).unwrap();
    add(&mut workspace, PrimitiveKind::Node);
    let valid = workspace.rules().unwrap();
    workspace
        .edit(None, 1010, |draft| {
            draft.rules[0].condition =
                r#"{"kind":"not","condition":{"kind":"always","surprise":true}}"#.into()
        })
        .unwrap();
    assert!(
        workspace
            .rules()
            .unwrap_err()
            .iter()
            .any(|issue| issue.field.ends_with("condition.surprise"))
    );
    workspace
        .edit(None, 1020, |draft| draft.rules[0].condition = "{".into())
        .unwrap();
    assert!(workspace.rules().is_err());
    assert!(workspace.prepare(SaveKind::Apply).is_err());
    workspace.undo_draft(false).unwrap();
    assert!(workspace.rules().is_err());
    workspace.undo_draft(false).unwrap();
    assert_eq!(workspace.rules().unwrap(), valid);
}

#[test]
fn reordering_tied_rules_changes_winner_and_draft_undo_restores_it() {
    let mut workspace = RuleWorkspace::new(SavedState::empty()).unwrap();
    add(&mut workspace, PrimitiveKind::Node);
    workspace
        .edit(None, 1010, |draft| {
            draft.add(PrimitiveKind::Node);
        })
        .unwrap();
    workspace
        .edit(None, 1020, |draft| {
            draft.rules[1].effects = r#"[{"kind":"details","visible":false}]"#.into()
        })
        .unwrap();
    let before = workspace.preview(&context("one")).unwrap();
    assert_eq!(before.winners[&Property::Details].rule_id, "node-2");
    workspace
        .edit(None, 1030, |draft| draft.rules.swap(0, 1))
        .unwrap();
    assert_eq!(
        workspace.preview(&context("one")).unwrap().winners[&Property::Details].rule_id,
        "node-1"
    );
    workspace.undo_draft(false).unwrap();
    assert_eq!(workspace.preview(&context("one")).unwrap(), before);
}

#[test]
fn draft_preview_is_pure_and_hysteresis_is_occurrence_local() {
    let mut workspace = RuleWorkspace::new(SavedState::empty()).unwrap();
    add(&mut workspace, PrimitiveKind::Node);
    workspace
        .edit(None, 1010, |draft| {
            draft.rules[0].target = r#"{"kind":"node","ids":["first","second"]}"#.into();
            draft.rules[0].condition =
                r#"{"kind":"screen_size","enter_below":48,"exit_above":84}"#.into();
        })
        .unwrap();
    let mut first = context("first");
    first.previous_matches.insert("node-1".into());
    first.states.insert(ViewState::Selected);
    let before = first.clone();
    assert!(
        workspace
            .preview(&first)
            .unwrap()
            .matched_rules
            .contains("node-1")
    );
    assert_eq!(first, before);
    assert!(
        workspace
            .preview(&context("second"))
            .unwrap()
            .winners
            .is_empty()
    );
    assert!(
        workspace
            .preview(&context("other"))
            .unwrap()
            .winners
            .is_empty()
    );
    assert_eq!(workspace.committed(), &SavedState::empty().rules);
}

#[test]
fn bounds_and_noop_apply_are_refused() {
    let mut workspace = RuleWorkspace::new(SavedState::empty()).unwrap();
    workspace.begin().unwrap();
    assert!(workspace.prepare(SaveKind::Apply).is_err());
    add(&mut workspace, PrimitiveKind::Node);
    assert!(
        workspace
            .edit(None, 1010, |draft| draft.rules[0].label =
                "x".repeat(RECORD_LIMIT))
            .is_err()
    );
    assert!(workspace.rules().is_ok());
}

#[test]
fn saved_history_is_capped_at_thirty_two_steps() {
    let mut workspace = RuleWorkspace::new(SavedState::empty()).unwrap();
    add(&mut workspace, PrimitiveKind::Node);
    let apply = workspace.prepare(SaveKind::Apply).unwrap();
    workspace.finish(apply.id, Ok(())).unwrap();
    for index in 0..35 {
        workspace.begin().unwrap();
        workspace
            .edit(None, 2000 + index, |draft| {
                draft.rules[0].label = format!("label {index}")
            })
            .unwrap();
        let apply = workspace.prepare(SaveKind::Apply).unwrap();
        workspace.finish(apply.id, Ok(())).unwrap();
    }
    for _ in 0..32 {
        let undo = workspace.prepare(SaveKind::Undo).unwrap();
        workspace.finish(undo.id, Ok(())).unwrap();
    }
    assert!(!workspace.can_undo());
    assert!(workspace.can_redo());
}
