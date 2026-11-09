// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
use super::*;
fn scope() -> Scope {
    Scope {
        session: Uuid::from_u128(1),
        persona: "persona:test".into(),
    }
}
fn spec() -> QueryViewSpec {
    QueryViewSpec::select("Held sources", "SELECT ?member WHERE { ?member ?p ?o }")
}
fn open(instance: u128) -> Editor {
    let mut editor = Editor::new(Uuid::from_u128(instance));
    editor.open(scope(), None, spec());
    editor
}
fn save(editor: &mut Editor) -> SaveRequest {
    match editor.save(spec()).unwrap() {
        Effect::Save(request) => request,
        _ => panic!("expected save"),
    }
}
fn answer(request: &SaveRequest, result: SaveResult) -> Completion {
    Completion {
        id: request.id,
        scope: request.scope.clone(),
        result,
    }
}

#[test]
fn only_owner_acknowledgement_advances_the_expected_definition() {
    let mut editor = open(10);
    let request = save(&mut editor);
    assert_eq!(editor.owner(), None);
    assert!(!editor.editable());
    assert!(editor.save(spec()).is_err());
    let owner = Uuid::from_u128(20);
    let access = Uuid::from_u128(21);
    assert_eq!(
        editor.complete(answer(
            &request,
            SaveResult::Committed {
                owner,
                access: Some(access)
            }
        )),
        Some(access)
    );
    let next = save(&mut editor);
    assert_eq!(next.owner, Some(owner));
    assert_eq!(next.expected, Some(request.spec));
}
#[test]
fn unconfirmed_write_retries_the_same_operation_and_cannot_create_again() {
    let mut editor = open(10);
    let request = save(&mut editor);
    editor.complete(answer(
        &request,
        SaveResult::Unconfirmed {
            reason: "store unavailable".into(),
        },
    ));
    assert!(!editor.editable());
    let Effect::Retry { id, scope } = editor.retry().unwrap() else {
        panic!("expected retry")
    };
    assert!(id.same_operation(request.id));
    assert_eq!(id.attempt, request.id.attempt + 1);
    assert_eq!(scope, request.scope);
    // A duplicate result from the failed attempt cannot finish this retry.
    editor.complete(answer(
        &request,
        SaveResult::Unconfirmed {
            reason: "late duplicate".into(),
        },
    ));
    assert_eq!(editor.state(), &SaveState::Saving(id));
    let request = SaveRequest { id, ..request };
    assert!(editor.retry().is_none());
    assert!(editor.save(spec()).is_err());
    editor.complete(answer(
        &request,
        SaveResult::Committed {
            owner: Uuid::from_u128(2),
            access: None,
        },
    ));
    assert_eq!(editor.status(), "Query view saved");
    assert!(editor.editable());
}
#[test]
fn completion_cannot_cross_instance_binding_or_owner() {
    let mut first = open(10);
    let mut second = open(11);
    let request = save(&mut first);
    let second_request = save(&mut second);
    let success = answer(
        &request,
        SaveResult::Committed {
            owner: Uuid::from_u128(2),
            access: Some(Uuid::from_u128(3)),
        },
    );
    assert!(second.complete(success.clone()).is_none());
    assert_eq!(second.state(), &SaveState::Saving(second_request.id));
    first.retire();
    first.open(scope(), None, spec());
    let reopened = save(&mut first);
    assert!(first.complete(success).is_none());
    assert_eq!(first.state(), &SaveState::Saving(reopened.id));
    let mut foreign = answer(&reopened, SaveResult::Stale);
    foreign.scope.persona = "other owner".into();
    first.complete(foreign);
    assert_eq!(first.state(), &SaveState::Saving(reopened.id));
}
#[test]
fn refusal_keeps_draft_editable_and_duplicate_completion_is_ignored() {
    let mut editor = open(10);
    let request = save(&mut editor);
    editor.complete(answer(
        &request,
        SaveResult::Refused {
            reason: "authority refused".into(),
        },
    ));
    assert!(editor.editable());
    assert_eq!(editor.draft(), Some(&spec()));
    editor.complete(answer(
        &request,
        SaveResult::Committed {
            owner: Uuid::from_u128(5),
            access: None,
        },
    ));
    assert_eq!(editor.owner(), None);
    assert_eq!(editor.status(), "authority refused");
}
#[test]
fn invalid_input_starts_no_operation_and_stale_response_retires_the_draft() {
    let mut editor = open(10);
    let mut invalid = spec();
    invalid.max_members = 0;
    assert!(editor.save(invalid).is_err());
    assert_eq!(editor.state(), &SaveState::Idle);
    let request = save(&mut editor);
    editor.complete(answer(&request, SaveResult::Stale));
    assert!(editor.scope().is_none());
    assert!(editor.draft().is_none());
}
