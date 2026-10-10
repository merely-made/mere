// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::fs;
use std::path::{Path, PathBuf};

use cambium::{FileEvent, OpenedFile, TextInput};
use tabard::Theme;
use tabard::library::ThemeLibraryStore;
use tabard::portable::{parse_theme_json, theme_json};
use tabard::theme::choice::{FileThemeChoiceStore, ThemeChoiceStore};
use tabard::theme::registry::{Harmony, Mode, THEME_ID_DEFAULT, ThemeRegistry, ThemeSource};
use tabard_workshop::{ExportFormat, WorkshopState};

fn user(id: &str, name: &str) -> Theme {
    let mut theme = ThemeRegistry::default()
        .theme_def(THEME_ID_DEFAULT)
        .unwrap()
        .clone();
    theme.id = id.into();
    theme.name = name.into();
    theme.source = ThemeSource::User;
    theme
}

fn library(themes: &[Theme]) -> (tempfile::TempDir, PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("themes.json");
    ThemeLibraryStore::load(&path)
        .unwrap()
        .save(themes)
        .unwrap();
    (directory, path)
}

fn rename(state: &mut WorkshopState, name: &str) {
    *state.text_field_mut("name").unwrap() = TextInput::new(name);
    state.sync_controls();
}

fn preference_path(path: &Path) -> PathBuf {
    let mut path = path.as_os_str().to_os_string();
    path.push(".workshop.json");
    PathBuf::from(path)
}

#[test]
fn host_owned_export_paths_preserve_existing_and_missing_files_without_a_library() {
    let directory = tempfile::tempdir().unwrap();
    let existing = directory.path().join("appearance.json");
    let missing = directory.path().join("new-document.djot");
    fs::write(&existing, b"owned application bytes").unwrap();
    let mut state = WorkshopState::in_memory();
    state.set_protected_export_paths(vec![existing.clone(), missing.clone()]);
    for path in [&existing, &missing] {
        state.request_export();
        let artifact = state.take_export().unwrap();
        state.complete_export(artifact, Some(path.clone()));
        assert!(state.status().contains("protected application files"));
        assert!(state.replacement_path().is_none());
    }
    assert_eq!(fs::read(&existing).unwrap(), b"owned application bytes");
    assert!(!missing.exists());
}

#[test]
fn newly_protected_paths_are_checked_again_before_export_replacement() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("document.djot");
    fs::write(&path, b"original document").unwrap();
    let mut state = WorkshopState::in_memory();
    state.request_export();
    let artifact = state.take_export().unwrap();
    state.complete_export(artifact, Some(path.clone()));
    assert!(state.replacement_path().is_some());
    let alias = directory.path().join(".").join("document.djot");
    state.set_protected_export_paths(vec![alias]);
    state.replace_export();
    assert!(state.status().contains("protected application files"));
    assert_eq!(fs::read(&path).unwrap(), b"original document");
}

#[test]
fn imported_collision_and_builtin_become_unsaved_copies_without_replacing_sources() {
    let existing = user("theme:existing", "Existing");
    let (_directory, path) = library(std::slice::from_ref(&existing));
    let mut state = WorkshopState::load(&path).unwrap();
    let active = state.registry().active_theme();
    let original_bytes = fs::read(&path).unwrap();
    let mut imported = existing.clone();
    imported.id = " THEME:EXISTING ".into();
    imported.harmony = Harmony::Locked {
        secondary_deg: 30.0,
        tertiary_deg: -30.0,
    };
    imported
        .mode_sheets
        .insert("dark".into(), vec!["p { color: red; }".into()]);
    state.import_theme_json(&theme_json(&imported).unwrap());
    assert_eq!(state.draft_theme().id, "theme:import-1");
    assert_eq!(state.draft_theme().source, ThemeSource::User);
    assert_eq!(state.draft_theme().harmony, imported.harmony);
    assert_eq!(state.draft_theme().mode_sheets, imported.mode_sheets);
    assert_eq!(state.registry().theme_def(&existing.id), Some(&existing));
    assert!(state.registry().theme_def("theme:import-1").is_none());
    assert!(state.is_dirty());
    assert_eq!(fs::read(&path).unwrap(), original_bytes);
    state.save();
    assert!(!state.is_dirty());
    let first_copy = state.draft_theme().clone();
    let builtin = state
        .registry()
        .theme_def(THEME_ID_DEFAULT)
        .unwrap()
        .clone();
    state.import_theme_json(&theme_json(&builtin).unwrap());
    assert_eq!(state.draft_theme().id, "theme:import-2");
    assert_eq!(state.draft_theme().source, ThemeSource::User);
    assert_eq!(state.registry().theme_def(THEME_ID_DEFAULT), Some(&builtin));
    assert_eq!(
        state.registry().theme_def(&first_copy.id),
        Some(&first_copy)
    );
    assert_eq!(state.registry().active_theme(), active);
}

#[test]
fn corrupt_utf8_cancelled_and_multiple_file_imports_preserve_current_editor() {
    let (_directory, path) = library(&[user("theme:existing", "Existing")]);
    let mut state = WorkshopState::load(&path).unwrap();
    let original = state.draft_theme().clone();
    let original_bytes = fs::read(&path).unwrap();
    state.import_theme_json("corrupt JSON");
    assert!(state.status().contains("Could not import"));
    state.request_import();
    assert!(state.import_requested());
    state.accept_import(FileEvent::default());
    assert!(!state.import_requested());
    assert!(state.status().contains("cancelled"));
    let file = OpenedFile {
        name: "invalid.json".into(),
        media_type: None,
        last_modified_ms: None,
        bytes: vec![0xff, 0xff],
    };
    state.accept_import(FileEvent {
        files: vec![file.clone()],
    });
    assert!(state.status().contains("UTF-8"));
    state.accept_import(FileEvent {
        files: vec![file.clone(), file],
    });
    assert!(state.status().contains("one theme"));
    assert_eq!(state.draft_theme(), &original);
    assert_eq!(fs::read(&path).unwrap(), original_bytes);
}

#[test]
fn pending_edits_block_import_and_delete_without_losing_the_text() {
    let (_directory, path) = library(&[user("theme:existing", "Existing")]);
    let mut state = WorkshopState::load(&path).unwrap();
    *state.text_field_mut("name").unwrap() = TextInput::new("Unsaved title");
    state.request_import();
    assert!(!state.import_requested());
    assert_eq!(state.draft_theme().name, "Unsaved title");
    state.import_theme_json(&theme_json(&user("theme:other", "Other")).unwrap());
    assert_eq!(state.draft_theme().name, "Unsaved title");
    state.request_delete();
    assert!(!state.delete_requested());
    assert!(state.is_dirty());
    assert_eq!(
        WorkshopState::load(&path).unwrap().draft_theme().name,
        "Existing"
    );
}

#[test]
fn draft_export_is_a_snapshot_and_collision_replacement_is_explicit() {
    let (directory, path) = library(&[user("theme:existing", "Existing")]);
    let mut state = WorkshopState::load(&path).unwrap();
    let library_bytes = fs::read(&path).unwrap();
    rename(&mut state, "Unsaved exported title");
    state.request_export();
    let artifact = state.take_export().unwrap();
    assert_eq!(
        parse_theme_json(&artifact.contents).unwrap().name,
        "Unsaved exported title"
    );
    rename(&mut state, "Newer edits during dialog");
    let export_path = directory.path().join(&artifact.suggested_name);
    fs::write(&export_path, "prior contents").unwrap();
    state.complete_export(artifact.clone(), Some(export_path.clone()));
    assert_eq!(state.replacement_path(), Some(export_path.as_path()));
    assert_eq!(fs::read_to_string(&export_path).unwrap(), "prior contents");
    state.cancel_export();
    assert!(state.replacement_path().is_none());
    assert_eq!(fs::read_to_string(&export_path).unwrap(), "prior contents");
    state.complete_export(artifact.clone(), Some(export_path.clone()));
    state.replace_export();
    assert!(state.replacement_path().is_none());
    assert_eq!(fs::read_to_string(&export_path).unwrap(), artifact.contents);
    assert_eq!(state.draft_theme().name, "Newer edits during dialog");
    assert!(state.is_dirty());
    assert_eq!(fs::read(&path).unwrap(), library_bytes);
}

#[test]
fn strict_mode_exports_match_preview_and_refuse_authored_sheet_projection() {
    let mut source = user("theme:colors", "Colors");
    source.harmony = Harmony::Locked {
        secondary_deg: 120.0,
        tertiary_deg: 240.0,
    };
    let (_directory, path) = library(&[source.clone()]);
    let mut state = WorkshopState::load(&path).unwrap();
    state.set_mode(Mode::HcLight);
    for format in [ExportFormat::Css, ExportFormat::Dtcg] {
        state.set_export_format(format);
        state.request_export();
        let artifact = state.take_export().unwrap();
        let expected = if format == ExportFormat::Css {
            source
                .css_custom_properties_for_mode(&Mode::HcLight)
                .unwrap()
        } else {
            source.design_tokens_json_for_mode(&Mode::HcLight).unwrap()
        };
        assert_eq!(artifact.contents, expected);
        assert!(artifact.suggested_name.contains("hc_light"));
    }
    source
        .mode_sheets
        .insert("dark".into(), vec!["p { color: red; }".into()]);
    state.import_theme_json(&theme_json(&source).unwrap());
    state.set_mode(Mode::Dark);
    for format in [ExportFormat::Css, ExportFormat::Dtcg] {
        state.set_export_format(format);
        state.request_export();
        assert!(state.take_export().is_none());
        assert!(state.status().contains("authored stylesheet"));
    }
    state.set_export_format(ExportFormat::ThemeJson);
    state.request_export();
    let artifact = state.take_export().unwrap();
    assert_eq!(
        parse_theme_json(&artifact.contents).unwrap().mode_sheets,
        source.mode_sheets
    );
}

#[test]
fn filenames_are_safe_and_exports_cannot_replace_library_or_preferences() {
    let (directory, path) = library(&[user("theme:existing", "Existing")]);
    let mut state = WorkshopState::load(&path).unwrap();
    state.import_theme_json(&theme_json(&user("../../theme:../evil\\name", "Portable")).unwrap());
    state.request_export();
    let artifact = state.take_export().unwrap();
    assert_eq!(Path::new(&artifact.suggested_name).components().count(), 1);
    assert!(!artifact.suggested_name.contains('/'));
    assert!(!artifact.suggested_name.contains('\\'));
    let original_bytes = fs::read(&path).unwrap();
    let settings = preference_path(&path);
    let library_lock = directory.path().join("themes.json.lock");
    let settings_lock = directory.path().join("themes.json.workshop.json.lock");
    for protected in [&path, &settings, &library_lock, &settings_lock] {
        state.complete_export(artifact.clone(), Some(protected.clone()));
        assert!(state.replacement_path().is_none());
        assert!(state.status().contains("separate"));
        state.replace_export();
        assert_eq!(fs::read(&path).unwrap(), original_bytes);
    }
    #[cfg(unix)]
    {
        let alias = directory.path().join("library-alias.json");
        std::os::unix::fs::symlink(&path, &alias).unwrap();
        state.complete_export(artifact, Some(alias));
        assert!(state.status().contains("separate"));
        assert!(state.replacement_path().is_none());
        assert_eq!(fs::read(&path).unwrap(), original_bytes);
    }
}

#[cfg(unix)]
#[test]
fn missing_editor_files_are_protected_through_symlinked_parent_aliases() {
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir().unwrap();
    let actual = directory.path().join("actual");
    fs::create_dir_all(actual.join("child")).unwrap();
    let alias = directory.path().join("alias");
    let child_alias = directory.path().join("child-alias");
    symlink(&actual, &alias).unwrap();
    symlink(actual.join("child"), &child_alias).unwrap();
    // More than one missing directory exercises longest-existing-ancestor
    // resolution rather than just canonicalizing the immediate parent.
    let library = actual.join("future/subdir/themes.json");
    let mut state = WorkshopState::load(&library).unwrap();
    state.request_export();
    let artifact = state.take_export().unwrap();
    let prefixes = [
        alias.join("future/subdir"),
        alias.join("missing/../future/subdir"),
        child_alias.join("../future/subdir"),
    ];
    for prefix in &prefixes {
        for name in [
            "themes.json",
            "themes.json.workshop.json",
            "themes.json.lock",
            "themes.json.workshop.json.lock",
        ] {
            state.complete_export(artifact.clone(), Some(prefix.join(name)));
            assert!(
                state.status().contains("separate"),
                "{}: {}",
                prefix.join(name).display(),
                state.status()
            );
            assert!(state.replacement_path().is_none());
            state.replace_export();
            assert!(!library.exists());
            assert!(!preference_path(&library).exists());
            assert!(
                !actual.join("future").exists(),
                "a rejected export creates no directories"
            );
            assert!(!actual.join("missing").exists());
        }
    }
    // A different artifact path through that alias remains a valid export.
    let destination = alias.join("future/subdir/exported.theme.json");
    state.complete_export(artifact.clone(), Some(destination));
    assert!(state.status().starts_with("Exported"));
    assert_eq!(
        fs::read_to_string(actual.join("future/subdir/exported.theme.json")).unwrap(),
        artifact.contents
    );
    assert!(!library.exists());
    assert!(
        state.is_dirty(),
        "export does not save the new authored draft"
    );
}

#[test]
fn failed_replace_keeps_pending_artifact_and_allows_retry() {
    let directory = tempfile::tempdir().unwrap();
    let mut state = WorkshopState::in_memory();
    state.request_export();
    let artifact = state.take_export().unwrap();
    let path = directory.path().join("occupied.json");
    fs::write(&path, "original").unwrap();
    state.complete_export(artifact.clone(), Some(path.clone()));
    let lock_path = directory.path().join("occupied.json.lock");
    fs::remove_file(&lock_path).unwrap();
    fs::create_dir(&lock_path).unwrap();
    state.replace_export();
    assert!(state.status().contains("Could not replace"));
    assert_eq!(state.replacement_path(), Some(path.as_path()));
    assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    fs::remove_dir(&lock_path).unwrap();
    state.replace_export();
    assert!(state.replacement_path().is_none());
    assert_eq!(fs::read_to_string(&path).unwrap(), artifact.contents);
    assert!(state.is_dirty());
}

#[test]
fn delete_confirmation_and_failed_persistence_preserve_current_library() {
    let first = user("theme:first", "First");
    let second = user("theme:second", "Second");
    let (directory, path) = library(&[first.clone(), second.clone()]);
    let mut state = WorkshopState::load(&path).unwrap();
    let active = state.registry().active_theme();
    let original_bytes = fs::read(&path).unwrap();
    state.confirm_delete();
    assert_eq!(state.draft_theme(), &second);
    state.request_delete();
    assert!(state.delete_requested());
    state.cancel_delete();
    assert_eq!(fs::read(&path).unwrap(), original_bytes);
    state.request_delete();
    let lock = directory.path().join("themes.json.lock");
    fs::remove_file(&lock).unwrap();
    fs::create_dir(&lock).unwrap();
    state.confirm_delete();
    assert!(state.status().contains("Could not delete"));
    assert!(state.delete_requested());
    assert_eq!(state.draft_theme(), &second);
    assert_eq!(state.registry().theme_def(&second.id), Some(&second));
    assert_eq!(fs::read(&path).unwrap(), original_bytes);
    fs::remove_dir(&lock).unwrap();
    state.confirm_delete();
    assert!(!state.delete_requested());
    assert_eq!(state.draft_theme(), &first);
    assert!(state.registry().theme_def(&second.id).is_none());
    assert_eq!(ThemeLibraryStore::load(&path).unwrap().themes(), &[first]);
    assert_eq!(state.registry().active_theme(), active);
}

#[test]
fn deleting_last_user_retains_builtins_and_pristine_copies_cannot_be_deleted() {
    let only = user("theme:only", "Only");
    let (_directory, path) = library(&[only]);
    let mut state = WorkshopState::load(&path).unwrap();
    let builtins: Vec<_> = state
        .registry()
        .list()
        .into_iter()
        .filter(|theme| theme.source == ThemeSource::BuiltIn)
        .cloned()
        .collect();
    state.request_delete();
    state.confirm_delete();
    assert!(ThemeLibraryStore::load(&path).unwrap().themes().is_empty());
    for builtin in &builtins {
        assert_eq!(state.registry().theme_def(&builtin.id), Some(builtin));
    }
    assert!(state.is_dirty());
    state.request_delete();
    assert!(!state.delete_requested());
    assert!(state.status().contains("saved user"));
}

#[test]
fn remembered_selection_and_preview_mode_survive_without_activating_host_theme() {
    let first = user("theme:first", "First");
    let second = user("theme:second", "Second");
    let (_directory, path) = library(&[first.clone(), second.clone()]);
    let mut state = WorkshopState::load(&path).unwrap();
    let active = state.registry().active_theme();
    let original_bytes = fs::read(&path).unwrap();
    state.select_theme(&first.id);
    state.set_mode(Mode::HcLight);
    assert!(preference_path(&path).is_file());
    let reopened = WorkshopState::load(&path).unwrap();
    assert_eq!(reopened.draft_theme(), &first);
    assert_eq!(reopened.mode(), &Mode::HcLight);
    assert_eq!(reopened.registry().active_theme(), active);
    assert_eq!(fs::read(&path).unwrap(), original_bytes);
    // A removed remembered identity falls back to a remaining registered user.
    ThemeLibraryStore::load(&path)
        .unwrap()
        .save(std::slice::from_ref(&second))
        .unwrap();
    let reopened = WorkshopState::load(&path).unwrap();
    assert_eq!(reopened.draft_theme(), &second);
    assert!(reopened.status().contains("unavailable"));
    assert_eq!(reopened.registry().active_theme(), active);
}

#[test]
fn a_remembered_custom_mode_requires_a_preserved_authored_sheet() {
    let mode = Mode::Custom("ink".into());
    let mut source = user("theme:custom", "Custom ink");
    source
        .mode_sheets
        .insert(mode.as_key(), vec!["p { color: red; }".into()]);
    let (_directory, path) = library(&[source.clone()]);
    let mut state = WorkshopState::load(&path).unwrap();
    state.set_mode(mode.clone());
    let reopened = WorkshopState::load(&path).unwrap();
    assert_eq!(reopened.mode(), &mode);
    assert_eq!(reopened.draft_theme().mode_sheets, source.mode_sheets);
    source.mode_sheets.clear();
    ThemeLibraryStore::load(&path)
        .unwrap()
        .save(&[source])
        .unwrap();
    let reopened = WorkshopState::load(&path).unwrap();
    assert!(!matches!(reopened.mode(), Mode::Custom(_)));
}

#[test]
fn removing_custom_stylesheet_by_apply_or_clear_restores_and_remembers_authored_default() {
    for use_clear in [false, true] {
        let mode = Mode::Custom("ink".into());
        let mut source = user("theme:custom-removal", "Custom ink");
        source.seeds.dark = false;
        source.high_contrast = true;
        source
            .mode_sheets
            .insert(mode.as_key(), vec!["p { color: red; }".into()]);
        let canonical_sheet = "p { color: blue; }";
        source
            .mode_sheets
            .insert(Mode::HcLight.as_key(), vec![canonical_sheet.into()]);
        let (_directory, path) = library(&[source]);
        let mut state = WorkshopState::load(&path).unwrap();
        state.set_mode(mode.clone());
        if use_clear {
            state.clear_stylesheet();
        } else {
            *state.text_field_mut("mode-sheet").unwrap() = TextInput::new(" \n ");
            state.apply_stylesheet();
        }
        assert_eq!(state.mode(), &Mode::HcLight);
        assert!(state.draft_theme().mode_sheet(&mode).is_none());
        assert_eq!(
            state.text_field("mode-sheet").unwrap().text(),
            canonical_sheet
        );
        assert_eq!(state.authored_sheet(), canonical_sheet);
        assert!(!state.has_pending_fields());
        assert!(state.status().contains("Custom stylesheet removed"));
        assert!(state.is_dirty());
        let preferences = FileThemeChoiceStore::load(preference_path(&path)).unwrap();
        assert_eq!(preferences.choice().theme_mode, Some(Mode::HcLight));
        // A redo can remove the sheet after the author returns to that custom
        // mode. History must keep the current picker/preview valid too.
        state.undo();
        state.set_mode(mode.clone());
        assert_eq!(state.mode(), &mode);
        state.redo();
        assert_eq!(state.mode(), &Mode::HcLight);
        assert_eq!(state.authored_sheet(), canonical_sheet);
        state.save();
        let reopened = WorkshopState::load(&path).unwrap();
        assert_eq!(reopened.mode(), &Mode::HcLight);
        assert!(reopened.draft_theme().mode_sheet(&mode).is_none());
        assert_eq!(reopened.authored_sheet(), canonical_sheet);
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
#[test]
fn editor_destinations_are_protected_through_case_variants_before_and_after_save() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("themes.json");
    let mut state = WorkshopState::load(&path).unwrap();
    state.request_export();
    let artifact = state.take_export().unwrap();
    for saved in [false, true] {
        if saved {
            state.save();
            assert!(!state.is_dirty());
        }
        let snapshot = fs::read(&path).ok();
        for name in [
            "THEMES.JSON",
            "THEMES.JSON.WORKSHOP.JSON",
            "THEMES.JSON.LOCK",
            "THEMES.JSON.WORKSHOP.JSON.LOCK",
        ] {
            state.complete_export(artifact.clone(), Some(directory.path().join(name)));
            assert!(
                state.status().contains("separate"),
                "{name}: {}",
                state.status()
            );
            assert!(state.replacement_path().is_none());
            state.replace_export();
            assert_eq!(fs::read(&path).ok(), snapshot);
        }
    }
}

#[cfg(target_os = "macos")]
#[test]
fn non_unicode_editor_paths_cannot_skip_case_alias_protection() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let directory = tempfile::tempdir().unwrap();
    let path = directory
        .path()
        .join(OsString::from_vec(b"the\xffmes.json".to_vec()));
    let mut state = WorkshopState::load(&path).unwrap();
    state.request_export();
    let artifact = state.take_export().unwrap();
    let alias = directory
        .path()
        .join(OsString::from_vec(b"THE\xffMES.JSON".to_vec()));
    state.complete_export(artifact, Some(alias));
    assert!(state.status().contains("separate"));
    assert!(state.replacement_path().is_none());
    assert!(!path.exists());
}

#[test]
fn protected_storage_directories_cover_future_files_and_recheck_replacement() {
    let directory = tempfile::tempdir().unwrap();
    let storage = directory.path().join("listener-data");
    let generated = storage.join("next-generation/state-42.json");
    let mut state = WorkshopState::in_memory();
    state.set_protected_export_directories(vec![storage.clone()]);
    state.request_export();
    let artifact = state.take_export().unwrap();
    state.complete_export(artifact.clone(), Some(generated.clone()));
    assert!(state.status().contains("protected application files"));
    assert!(!generated.exists());
    assert!(!storage.exists());
    let adjacent = directory.path().join("listener-data-export/theme.json");
    state.complete_export(artifact.clone(), Some(adjacent.clone()));
    assert_eq!(fs::read_to_string(&adjacent).unwrap(), artifact.contents);
    state.set_protected_export_directories(vec![]);
    fs::create_dir_all(&storage).unwrap();
    let current = storage.join("state-42.json");
    fs::write(&current, b"owned generation").unwrap();
    state.complete_export(artifact, Some(current.clone()));
    assert!(state.replacement_path().is_some());
    state.set_protected_export_directories(vec![storage]);
    state.replace_export();
    assert!(state.status().contains("protected application files"));
    assert_eq!(fs::read(current).unwrap(), b"owned generation");
}

#[cfg(unix)]
#[test]
fn directory_export_guard_protects_aliases_and_owned_symlink_entries() {
    let directory = tempfile::tempdir().unwrap();
    let storage = directory.path().join("listener-data");
    fs::create_dir_all(&storage).unwrap();
    let alias = directory.path().join("storage-alias");
    std::os::unix::fs::symlink(&storage, &alias).unwrap();
    let outside = directory.path().join("outside.json");
    fs::write(&outside, b"outside bytes").unwrap();
    let link = storage.join("owned-link.json");
    std::os::unix::fs::symlink(&outside, &link).unwrap();
    let mut state = WorkshopState::in_memory();
    state.set_protected_export_directories(vec![storage.clone()]);
    for path in [
        alias.join("future/state-43.json"),
        link.clone(),
        storage.join("missing/../state-44.json"),
    ] {
        state.request_export();
        let artifact = state.take_export().unwrap();
        state.complete_export(artifact, Some(path));
        assert!(state.status().contains("protected application files"));
        assert!(state.replacement_path().is_none());
    }
    assert!(fs::symlink_metadata(link).unwrap().file_type().is_symlink());
    assert_eq!(fs::read(outside).unwrap(), b"outside bytes");
    assert!(!storage.join("state-44.json").exists());
}
