// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use cambium::TextInput;
use tabard::library::ThemeLibraryStore;
use tabard::theme::registry::{Harmony, Mode, THEME_ID_DEFAULT, ThemeRegistry, ThemeSource};
use tabard::{Theme, ThemePresentation, resolve_theme_choice};
use tabard_workshop::WorkshopState;

fn builtin() -> Theme {
    ThemeRegistry::default()
        .theme_def(THEME_ID_DEFAULT)
        .unwrap()
        .clone()
}

#[test]
fn editing_a_builtin_forks_preserving_fields_and_saved_choice_requires_actual_save() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("themes.json");
    let mut state = WorkshopState::load(&path).unwrap();
    assert!(state.saved_choice().is_err());
    let active = state.registry().active_theme();
    let mut source = builtin();
    source.high_contrast = true;
    source.harmony = Harmony::Locked {
        secondary_deg: 30.0,
        tertiary_deg: -30.0,
    };
    let rules = vec![".chrome { color: #abcdef; }".into()];
    source
        .mode_sheets
        .insert("custom:concert".into(), rules.clone());
    state
        .edit_definition(&source, Some(Mode::Custom("concert".into())))
        .unwrap();
    let draft = state.draft_theme().clone();
    assert_ne!(draft.id, source.id);
    assert_eq!(draft.source, ThemeSource::User);
    assert_eq!(draft.seeds, source.seeds);
    assert_eq!(draft.harmony, source.harmony);
    assert_eq!(draft.high_contrast, source.high_contrast);
    assert_eq!(draft.mode_sheets, source.mode_sheets);
    assert!(state.has_changes());
    assert!(state.saved_choice().is_err());
    assert!(!path.exists());
    state.save();
    let choice = state.saved_choice().unwrap();
    assert_eq!(choice.theme_id, draft.id);
    assert_eq!(choice.theme_mode, Some(Mode::Custom("concert".into())));
    let resolved = resolve_theme_choice(state.registry(), &choice).unwrap();
    assert_eq!(
        resolved.presentation,
        ThemePresentation::AuthoredStylesheet(rules)
    );
    assert_eq!(state.registry().active_theme(), active);
    let persisted = ThemeLibraryStore::load(path).unwrap();
    assert_eq!(persisted.themes(), &[draft]);
}

#[test]
fn matching_saved_user_opens_in_place_but_external_collision_gets_a_fresh_copy() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("themes.json");
    let mut source = Theme::new("theme:user", "User", builtin().seeds);
    ThemeLibraryStore::load(&path)
        .unwrap()
        .save(&[source.clone()])
        .unwrap();
    let mut state = WorkshopState::load(&path).unwrap();
    state.edit_definition(&source, Some(Mode::HcLight)).unwrap();
    assert!(!state.has_changes());
    assert_eq!(state.saved_choice().unwrap().theme_id, source.id);
    assert_eq!(state.mode(), &Mode::HcLight);
    source.name = "External changed version".into();
    state.edit_definition(&source, None).unwrap();
    assert_ne!(state.draft_theme().id, source.id);
    assert!(state.has_changes());
    assert!(state.saved_choice().is_err());
    assert_eq!(state.registry().theme_def(&source.id).unwrap().name, "User");
    assert_eq!(
        ThemeLibraryStore::load(path).unwrap().themes()[0].name,
        "User"
    );
}

#[test]
fn invalid_or_staged_intake_is_transactional_and_name_buffers_cannot_apply() {
    let mut state = WorkshopState::in_memory();
    let before = state.draft_theme().clone();
    let before_mode = state.mode().clone();
    let mut invalid = builtin();
    invalid.name.clear();
    assert!(state.edit_definition(&invalid, None).is_err());
    assert!(
        state
            .edit_definition(&builtin(), Some(Mode::Custom("missing".into())))
            .is_err()
    );
    assert_eq!(state.draft_theme(), &before);
    assert_eq!(state.mode(), &before_mode);
    state.save();
    assert!(state.saved_choice().is_ok());
    for (field, value) in [
        ("name", "Staged name"),
        ("seed-hex", "#12"),
        ("mode-sheet", ".app { color: red; }"),
    ] {
        *state.text_field_mut(field).unwrap() = TextInput::new(value);
        let before = state.draft_theme().clone();
        assert!(state.saved_choice().is_err());
        assert!(state.edit_definition(&builtin(), None).is_err());
        assert_eq!(state.draft_theme(), &before);
        assert_eq!(state.text_field(field).unwrap().text(), value);
        state.discard();
    }
}

#[test]
fn a_failed_persistent_save_cannot_produce_an_application_choice() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("themes.json");
    let mut state = WorkshopState::load(&path).unwrap();
    state.edit_definition(&builtin(), None).unwrap();
    let draft = state.draft_theme().clone();
    std::fs::write(&path, "external change").unwrap();
    state.save();
    assert!(state.saved_choice().is_err());
    assert_eq!(state.draft_theme(), &draft);
    assert!(state.registry().theme_def(&draft.id).is_none());
    assert_eq!(std::fs::read_to_string(path).unwrap(), "external change");
}
