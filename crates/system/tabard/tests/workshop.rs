// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use tabard::theme::registry::{Mode, THEME_ID_DEFAULT, ThemeRegistry};
use tabard::theme::seed::derive_from_def_for_mode;
use tabard::workshop::{Edit, Preview, ThemeDraft, WorkshopError};

fn fork(registry: &ThemeRegistry) -> ThemeDraft {
    ThemeDraft::fork(registry, THEME_ID_DEFAULT, "theme:mine", "Mine").unwrap()
}

#[test]
fn draft_preview_and_commit_preserve_builtin_and_active_appearance() {
    let mut registry = ThemeRegistry::default();
    let original = registry.theme_def(THEME_ID_DEFAULT).unwrap().clone();
    let active = registry.active_theme().resolved_id;
    assert!(matches!(
        ThemeDraft::open(&registry, THEME_ID_DEFAULT),
        Err(WorkshopError::BuiltInRequiresFork)
    ));
    let mut draft = fork(&registry);
    let mut seeds = draft.theme().seeds;
    seeds.primary = tinct::Srgb::rgb(200, 20, 90);
    draft.edit(Edit::Seeds(seeds));
    for mode in [Mode::Light, Mode::Dark, Mode::HcLight, Mode::HcDark] {
        assert_eq!(
            draft.preview(&mode).unwrap(),
            Preview::Tokens(derive_from_def_for_mode(draft.theme(), &mode))
        );
    }
    assert!(registry.theme_def("theme:mine").is_none());
    assert!(draft.is_dirty());
    draft.commit(&mut registry).unwrap();
    assert!(!draft.is_dirty());
    assert_eq!(registry.theme_def(THEME_ID_DEFAULT), Some(&original));
    assert_eq!(registry.theme_def("theme:mine"), Some(draft.theme()));
    assert_eq!(registry.active_theme().resolved_id, active);
}

#[test]
fn undo_redo_branching_and_discard_return_to_save_point() {
    let mut registry = ThemeRegistry::default();
    let mut draft = fork(&registry);
    draft.commit(&mut registry).unwrap();
    let mut draft = ThemeDraft::open(&registry, "theme:mine").unwrap();
    draft.edit(Edit::Name("First".into()));
    draft.edit(Edit::Name("Second".into()));
    assert!(draft.undo());
    assert_eq!(draft.theme().name, "First");
    assert!(draft.redo());
    assert_eq!(draft.theme().name, "Second");
    draft.undo();
    // A no-op does not destroy the redo branch.
    assert!(!draft.edit(Edit::Name("First".into())));
    assert!(draft.redo());
    draft.undo();
    draft.edit(Edit::Name("Alternative".into()));
    assert!(!draft.redo());
    draft.discard();
    assert_eq!(draft.theme().name, "Mine");
    assert!(!draft.is_dirty());
    assert!(!draft.undo());
    assert_eq!(registry.theme_def("theme:mine").unwrap().name, "Mine");
}

#[test]
fn commit_refuses_concurrent_edits_deletion_and_fork_collisions() {
    let mut registry = ThemeRegistry::default();
    let mut first = fork(&registry);
    let mut collision = fork(&registry);
    first.commit(&mut registry).unwrap();
    assert_eq!(
        collision.commit(&mut registry),
        Err(WorkshopError::IdentityInUse)
    );
    let mut editor = ThemeDraft::open(&registry, "theme:mine").unwrap();
    editor.edit(Edit::Name("Draft name".into()));
    registry.rename_user_theme("theme:mine", "Another editor");
    assert_eq!(
        editor.commit(&mut registry),
        Err(WorkshopError::SourceChanged)
    );
    assert_eq!(
        registry.theme_def("theme:mine").unwrap().name,
        "Another editor"
    );
    assert!(editor.is_dirty());
    let mut editor = ThemeDraft::open(&registry, "theme:mine").unwrap();
    registry.remove_user_theme("theme:mine");
    assert_eq!(
        editor.commit(&mut registry),
        Err(WorkshopError::SourceChanged)
    );
    assert!(registry.theme_def("theme:mine").is_none());
}

#[test]
fn stylesheet_previews_never_pretend_to_be_derived_tokens() {
    let registry = ThemeRegistry::default();
    let mut draft = fork(&registry);
    let custom = Mode::Custom("evening".into());
    assert_eq!(
        draft.preview(&custom),
        Err(WorkshopError::CustomModeNeedsCalculator)
    );
    let rules = vec!["body { color: red; }".into()];
    for mode in [Mode::Dark, custom.clone()] {
        draft.edit(Edit::ModeSheet {
            mode: mode.clone(),
            rules: rules.clone(),
        });
        assert_eq!(draft.preview(&mode), Ok(Preview::Stylesheet(rules.clone())));
    }
    draft.edit(Edit::ModeSheet {
        mode: Mode::Dark,
        rules: vec![],
    });
    assert!(matches!(draft.preview(&Mode::Dark), Ok(Preview::Tokens(_))));
    draft.edit(Edit::ModeSheet {
        mode: custom.clone(),
        rules: vec![],
    });
    assert_eq!(
        draft.preview(&custom),
        Err(WorkshopError::CustomModeNeedsCalculator)
    );
}

#[test]
fn invalid_and_colliding_identities_never_change_registry() {
    let mut registry = ThemeRegistry::default();
    assert!(matches!(
        ThemeDraft::fork(&registry, THEME_ID_DEFAULT, "  ", "Mine"),
        Err(WorkshopError::InvalidIdentity)
    ));
    assert!(matches!(
        ThemeDraft::fork(&registry, THEME_ID_DEFAULT, " THEME:DEFAULT ", "Mine"),
        Err(WorkshopError::IdentityInUse)
    ));
    assert!(matches!(
        ThemeDraft::open(&registry, "missing"),
        Err(WorkshopError::UnknownTheme)
    ));
    let mut draft = fork(&registry);
    draft.edit(Edit::Name(" ".into()));
    assert_eq!(
        draft.commit(&mut registry),
        Err(WorkshopError::InvalidIdentity)
    );
    assert!(registry.theme_def("theme:mine").is_none());
    draft.discard();
    assert_eq!(draft.theme().name, "Mine");
    assert!(draft.is_dirty());
}
