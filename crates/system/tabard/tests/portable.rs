// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::fs::{self, OpenOptions};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use tabard::Theme;
use tabard::library::ThemeLibraryStore;
use tabard::portable::{WriteMode, parse_theme_json, theme_json, write_artifact};
use tabard::theme::registry::{Harmony, Mode, THEME_ID_DEFAULT, ThemeRegistry, ThemeSource};
use tabard::workshop::{Edit, ThemeDraft, WorkshopError};
use tinct::Srgb;

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct Directory(PathBuf);

impl Directory {
    fn new() -> Self {
        for _ in 0..32 {
            let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("tabard-portable-{}-{sequence}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create portable test directory: {error}"),
            }
        }
        panic!("could not create test directory");
    }

    fn assert_no_temporary_files(&self) {
        assert!(fs::read_dir(&self.0).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".tmp")
        }));
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn authored() -> Theme {
    let mut theme = ThemeRegistry::default()
        .theme_def(THEME_ID_DEFAULT)
        .unwrap()
        .clone();
    theme.id = "theme:portable".into();
    theme.name = "Portable ink".into();
    theme.source = ThemeSource::User;
    theme
}

#[test]
fn portable_roundtrip_preserves_authored_fields_and_normalizes_identity() {
    let mut theme = authored();
    theme.id = " THEME:PORTABLE ".into();
    theme.high_contrast = true;
    theme.harmony = Harmony::Locked {
        secondary_deg: -30.0,
        tertiary_deg: 180.0,
    };
    theme.seeds.primary = Srgb::rgba(0x22, 0x66, 0xaa, 0x80);
    theme.seeds.text_body = Some(Srgb::rgb(0xee, 0xee, 0xee));
    theme.seeds.text_header = Some(Srgb::WHITE);
    theme
        .mode_sheets
        .insert("dark".into(), vec![":root { color: #ffffff; }".into()]);
    theme
        .mode_sheets
        .insert("custom:ink".into(), vec!["p { font-weight: bold; }".into()]);
    let json = theme_json(&theme).unwrap();
    assert!(json.ends_with('\n'));
    assert!(json.contains('\n'));
    theme.id = "theme:portable".into();
    let parsed = parse_theme_json(&json).unwrap();
    assert_eq!(parsed, theme);
    assert_eq!(
        parse_theme_json(&theme_json(&parsed).unwrap()).unwrap(),
        parsed
    );
}

#[test]
fn corrupt_or_inadmissible_payloads_return_visible_errors() {
    for json in [
        "",
        "not JSON",
        "[]",
        "{}",
        r#"{"id":"x","name":"x","seeds":null}"#,
    ] {
        assert_eq!(
            parse_theme_json(json).unwrap_err().kind(),
            ErrorKind::InvalidData
        );
    }
    let valid = serde_json::to_value(authored()).unwrap();
    for (field, bad) in [
        ("id", serde_json::json!(" \t")),
        ("name", serde_json::json!(" \n")),
        ("source", serde_json::json!("Unknown")),
        ("seeds", serde_json::json!({})),
        ("high_contrast", serde_json::json!("yes")),
        ("mode_sheets", serde_json::json!({"dark": 4})),
    ] {
        let mut value = valid.clone();
        value[field] = bad;
        assert_eq!(
            parse_theme_json(&value.to_string()).unwrap_err().kind(),
            ErrorKind::InvalidData,
            "invalid {field}"
        );
    }
    let mut theme = authored();
    theme.harmony = Harmony::Locked {
        secondary_deg: f32::NAN,
        tertiary_deg: 120.0,
    };
    assert_eq!(
        theme_json(&theme).unwrap_err().kind(),
        ErrorKind::InvalidInput
    );
    assert!(matches!(
        ThemeDraft::from_theme(theme),
        Err(WorkshopError::InvalidTheme(_))
    ));
}

#[test]
fn builtin_artifacts_are_inert_until_explicitly_forked_to_user_identity() {
    let registry = ThemeRegistry::default();
    let builtin = registry.theme_def(THEME_ID_DEFAULT).unwrap().clone();
    let active = registry.active_theme();
    let parsed = parse_theme_json(&theme_json(&builtin).unwrap()).unwrap();
    assert_eq!(parsed, builtin);
    assert!(matches!(
        ThemeDraft::from_theme(parsed.clone()),
        Err(WorkshopError::BuiltInRequiresFork)
    ));
    assert!(matches!(
        ThemeDraft::fork_theme(&registry, &parsed, THEME_ID_DEFAULT, "Override"),
        Err(WorkshopError::IdentityInUse)
    ));
    let draft =
        ThemeDraft::fork_theme(&registry, &parsed, " THEME:IMPORTED ", "Imported ink").unwrap();
    assert_eq!(draft.theme().id, "theme:imported");
    assert_eq!(draft.theme().source, ThemeSource::User);
    assert_eq!(draft.theme().seeds, builtin.seeds);
    assert_eq!(draft.theme().harmony, builtin.harmony);
    assert_eq!(draft.theme().mode_sheets, builtin.mode_sheets);
    assert!(draft.is_dirty());
    assert!(registry.theme_def("theme:imported").is_none());
    assert_eq!(registry.theme_def(THEME_ID_DEFAULT), Some(&builtin));
    assert_eq!(registry.active_theme(), active);
}

#[test]
fn imported_draft_checks_commit_collisions_and_copy_preserves_unregistered_source() {
    let mut registry = ThemeRegistry::default();
    let source = authored();
    let mut draft = ThemeDraft::from_theme(source.clone()).unwrap();
    assert!(draft.is_dirty());
    assert!(registry.theme_def(&source.id).is_none());
    let copy =
        ThemeDraft::fork_theme(&registry, &source, "theme:copy", "Portable ink copy").unwrap();
    assert_eq!(copy.theme().seeds, source.seeds);
    assert!(registry.theme_def(&source.id).is_none());
    let mut collision = source.clone();
    collision.name = "Existing definition".into();
    registry.add_user_theme(collision.clone()).unwrap();
    assert_eq!(
        draft.commit(&mut registry),
        Err(WorkshopError::IdentityInUse)
    );
    assert!(draft.is_dirty());
    assert_eq!(registry.theme_def(&source.id), Some(&collision));
    assert!(matches!(
        ThemeDraft::fork_theme(&registry, &source, " THEME:PORTABLE ", "Collision"),
        Err(WorkshopError::IdentityInUse)
    ));
    let mut forged_user = registry.theme_def(THEME_ID_DEFAULT).unwrap().clone();
    forged_user.source = ThemeSource::User;
    assert!(matches!(
        ThemeDraft::from_theme(forged_user),
        Err(WorkshopError::IdentityInUse)
    ));
}

#[test]
fn invalid_new_draft_edit_cannot_change_registry_or_save_point() {
    let mut registry = ThemeRegistry::default();
    let mut draft = ThemeDraft::from_theme(authored()).unwrap();
    draft.commit(&mut registry).unwrap();
    let original = draft.theme().clone();
    draft.edit(Edit::Harmony(Harmony::Locked {
        secondary_deg: f32::INFINITY,
        tertiary_deg: 0.0,
    }));
    assert!(matches!(
        draft.commit(&mut registry),
        Err(WorkshopError::InvalidTheme(_))
    ));
    assert_eq!(registry.theme_def(&original.id), Some(&original));
    assert!(draft.is_dirty());
    draft.discard();
    assert_eq!(draft.theme(), &original);
}

#[test]
fn authored_default_mode_updates_both_flags_in_one_history_action() {
    let mut theme = authored();
    theme
        .mode_sheets
        .insert("dark".into(), vec!["p { color: red; }".into()]);
    let mut draft = ThemeDraft::from_theme(theme.clone()).unwrap();
    assert!(!draft.edit(Edit::DefaultMode(Mode::Custom("unavailable".into()))));
    assert_eq!(draft.theme(), &theme);
    assert!(draft.edit(Edit::DefaultMode(Mode::HcLight)));
    let mut expected = theme.clone();
    expected.seeds.dark = false;
    expected.high_contrast = true;
    assert_eq!(draft.theme(), &expected);
    assert!(draft.undo());
    assert_eq!(draft.theme(), &theme);
    assert!(!draft.undo());
    assert!(draft.redo());
    assert_eq!(draft.theme(), &expected);
}

#[test]
fn exporting_a_draft_does_not_save_it_and_never_clobbers_by_default() {
    let directory = Directory::new();
    let library_path = directory.0.join("library.json");
    let mut library = ThemeLibraryStore::load(&library_path).unwrap();
    let original = authored();
    library.save(std::slice::from_ref(&original)).unwrap();
    let library_bytes = fs::read(&library_path).unwrap();
    let mut draft = ThemeDraft::from_theme(original).unwrap();
    draft.edit(Edit::Name("Exported unsaved changes".into()));
    let export_path = directory.0.join("theme.json");
    let bytes = theme_json(draft.theme()).unwrap();
    write_artifact(&export_path, &bytes, WriteMode::default()).unwrap();
    assert_eq!(
        parse_theme_json(&fs::read_to_string(&export_path).unwrap()).unwrap(),
        *draft.theme()
    );
    assert!(draft.is_dirty());
    assert_eq!(fs::read(&library_path).unwrap(), library_bytes);
    assert_eq!(
        write_artifact(&export_path, "replacement", WriteMode::CreateNew)
            .unwrap_err()
            .kind(),
        ErrorKind::AlreadyExists
    );
    assert_eq!(fs::read_to_string(&export_path).unwrap(), bytes);
    write_artifact(&export_path, "replacement", WriteMode::Replace).unwrap();
    assert_eq!(fs::read_to_string(&export_path).unwrap(), "replacement");
    assert_eq!(fs::read(&library_path).unwrap(), library_bytes);
    directory.assert_no_temporary_files();
}

#[test]
fn blocked_and_failed_replacement_preserve_existing_artifact() {
    let directory = Directory::new();
    let path = directory.0.join("artifact.json");
    write_artifact(&path, "original", WriteMode::CreateNew).unwrap();
    let lock_path = directory.0.join("artifact.json.lock");
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&lock_path)
        .unwrap();
    lock.lock().unwrap();
    assert_eq!(
        write_artifact(&path, "replacement", WriteMode::Replace)
            .unwrap_err()
            .kind(),
        ErrorKind::WouldBlock
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    drop(lock);
    fs::remove_file(&lock_path).unwrap();
    fs::create_dir(&lock_path).unwrap();
    assert!(write_artifact(&path, "replacement", WriteMode::Replace).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    fs::remove_dir(&lock_path).unwrap();
    write_artifact(&path, "replacement", WriteMode::Replace).unwrap();
    let directory_target = directory.0.join("directory");
    fs::create_dir(&directory_target).unwrap();
    assert!(write_artifact(&directory_target, "replacement", WriteMode::Replace).is_err());
    assert!(directory_target.is_dir());
    directory.assert_no_temporary_files();
}

#[cfg(unix)]
#[test]
fn explicit_replacement_preserves_existing_permissions_and_refuses_symlink_clobber() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let directory = Directory::new();
    let path = directory.0.join("artifact.json");
    write_artifact(&path, "original", WriteMode::CreateNew).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
    write_artifact(&path, "replacement", WriteMode::Replace).unwrap();
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o640
    );
    let link = directory.0.join("link.json");
    symlink(directory.0.join("missing.json"), &link).unwrap();
    assert_eq!(
        write_artifact(&link, "replacement", WriteMode::CreateNew)
            .unwrap_err()
            .kind(),
        ErrorKind::AlreadyExists
    );
    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    directory.assert_no_temporary_files();
}
