// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::fs::{self, OpenOptions};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::json;
use tabard::Theme;
use tabard::library::ThemeLibraryStore;
use tabard::theme::registry::{Harmony, THEME_ID_DEFAULT, ThemeRegistry, ThemeSource};

static DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        for _ in 0..32 {
            let sequence = DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "tabard-library-test-{}-{sequence}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create test directory: {error}"),
            }
        }
        panic!("could not create unique test directory");
    }

    fn library(&self) -> PathBuf {
        self.0.join("themes.json")
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn authored(id: &str) -> Theme {
    let registry = ThemeRegistry::default();
    let mut theme = registry.theme_def(THEME_ID_DEFAULT).unwrap().clone();
    theme.id = id.into();
    theme.name = "My ink".into();
    theme.source = ThemeSource::User;
    theme
}

fn load_error(path: PathBuf) -> std::io::Error {
    ThemeLibraryStore::load(path).unwrap_err()
}

#[test]
fn missing_library_loads_without_creating_files_and_saves_authored_shape() {
    let directory = TestDirectory::new();
    let path = directory.0.join("nested/themes.json");
    let mut library = ThemeLibraryStore::load(&path).unwrap();
    assert_eq!(library.path(), path);
    assert!(library.themes().is_empty());
    assert!(!path.parent().unwrap().exists());

    let mut theme = authored("theme:ink");
    theme.high_contrast = true;
    theme.harmony = Harmony::Locked {
        secondary_deg: 120.0,
        tertiary_deg: 240.0,
    };
    theme
        .mode_sheets
        .insert("dark".into(), vec![":root { color: #ffffff; }".into()]);
    library.save(&[theme.clone()]).unwrap();
    assert_eq!(library.themes(), &[theme.clone()]);
    let document: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(document["version"], 1);
    assert_eq!(document["themes"][0], serde_json::to_value(&theme).unwrap());
    let mut reopened = ThemeLibraryStore::load(&path).unwrap();
    assert_eq!(reopened.themes(), &[theme]);
    reopened.save(&[]).unwrap();
    assert!(ThemeLibraryStore::load(&path).unwrap().themes().is_empty());
}

#[test]
fn corrupt_and_unsupported_libraries_are_visible_and_left_untouched() {
    let directory = TestDirectory::new();
    let path = directory.library();
    for bytes in [
        b"not json".as_slice(),
        b"".as_slice(),
        br#"{"version":2,"themes":[]}"#.as_slice(),
        br#"{"version":1,"themes":[],"future_data":true}"#.as_slice(),
        br#"{"version":1,"themes":[{"id":"incomplete"}]}"#.as_slice(),
    ] {
        fs::write(&path, bytes).unwrap();
        let error = load_error(path.clone());
        assert_eq!(error.kind(), ErrorKind::InvalidData, "{error}");
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
}

#[test]
fn inadmissible_definitions_are_rejected_on_both_load_and_save() {
    let directory = TestDirectory::new();
    let path = directory.library();
    let mut library = ThemeLibraryStore::load(&path).unwrap();
    let valid = authored("theme:ink");
    library.save(&[valid.clone()]).unwrap();
    let original_bytes = fs::read(&path).unwrap();
    let mut blank_name = valid.clone();
    blank_name.name = " \n ".into();
    let mut falsely_builtin = valid.clone();
    falsely_builtin.source = ThemeSource::BuiltIn;
    let inadmissible = [
        vec![authored(" \t")],
        vec![blank_name],
        vec![authored(&format!(
            " {} ",
            THEME_ID_DEFAULT.to_ascii_uppercase()
        ))],
        vec![falsely_builtin],
        vec![valid.clone(), authored(" THEME:INK ")],
    ];
    for themes in inadmissible {
        let error = library.save(&themes).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidInput, "{error}");
        assert_eq!(library.themes(), &[valid.clone()]);
        assert_eq!(fs::read(&path).unwrap(), original_bytes);
        let bytes = serde_json::to_vec(&json!({"version": 1, "themes": themes})).unwrap();
        fs::write(&path, &bytes).unwrap();
        let error = load_error(path.clone());
        assert_eq!(error.kind(), ErrorKind::InvalidData, "{error}");
        assert_eq!(fs::read(&path).unwrap(), bytes);
        fs::write(&path, &original_bytes).unwrap();
    }
    // Serde JSON converts NaN floats to null; reject these before writing a
    // library that could not be read back.
    let mut nonfinite = valid.clone();
    nonfinite.harmony = Harmony::Locked {
        secondary_deg: f32::NAN,
        tertiary_deg: 120.0,
    };
    assert_eq!(
        library.save(&[nonfinite]).unwrap_err().kind(),
        ErrorKind::InvalidInput
    );
    assert_eq!(fs::read(&path).unwrap(), original_bytes);
}

#[test]
fn stale_writers_refuse_changes_deletions_and_newly_created_libraries() {
    let directory = TestDirectory::new();
    let path = directory.library();
    let mut first = ThemeLibraryStore::load(&path).unwrap();
    let mut initially_missing = ThemeLibraryStore::load(&path).unwrap();
    let original = authored("theme:ink");
    first.save(&[original.clone()]).unwrap();
    assert_eq!(
        initially_missing.save(&[]).unwrap_err().kind(),
        ErrorKind::WouldBlock
    );
    assert!(initially_missing.themes().is_empty());

    let mut stale = ThemeLibraryStore::load(&path).unwrap();
    let replacement = authored("theme:other");
    first.save(&[replacement.clone()]).unwrap();
    let bytes = fs::read(&path).unwrap();
    assert_eq!(stale.save(&[]).unwrap_err().kind(), ErrorKind::WouldBlock);
    assert_eq!(stale.themes(), &[original]);
    assert_eq!(fs::read(&path).unwrap(), bytes);

    // An unrelated editor's corrupt bytes are preserved too.
    fs::write(&path, b"external editor is still writing").unwrap();
    assert_eq!(first.save(&[]).unwrap_err().kind(), ErrorKind::WouldBlock);
    assert_eq!(
        fs::read(&path).unwrap(),
        b"external editor is still writing"
    );
    assert_eq!(first.themes(), &[replacement.clone()]);
    fs::remove_file(&path).unwrap();
    assert_eq!(first.save(&[]).unwrap_err().kind(), ErrorKind::WouldBlock);
    assert_eq!(first.themes(), &[replacement]);
    assert!(!path.exists());
}

#[test]
fn busy_lock_preserves_memory_and_disk_then_allows_retry() {
    let directory = TestDirectory::new();
    let path = directory.library();
    let mut library = ThemeLibraryStore::load(&path).unwrap();
    let original = authored("theme:ink");
    library.save(&[original.clone()]).unwrap();
    let bytes = fs::read(&path).unwrap();
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .open(directory.0.join("themes.json.lock"))
        .unwrap();
    lock.lock().unwrap();
    assert_eq!(library.save(&[]).unwrap_err().kind(), ErrorKind::WouldBlock);
    assert_eq!(library.themes(), &[original]);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    drop(lock);
    library.save(&[]).unwrap();
    assert!(library.themes().is_empty());
}

#[test]
fn failed_filesystem_save_preserves_snapshot_and_allows_retry() {
    let directory = TestDirectory::new();
    let path = directory.library();
    let mut library = ThemeLibraryStore::load(&path).unwrap();
    let original = authored("theme:ink");
    library.save(&[original.clone()]).unwrap();
    let bytes = fs::read(&path).unwrap();
    // This deterministic failure works even if tests run with privileged uid.
    fs::remove_file(directory.0.join("themes.json.lock")).unwrap();
    fs::create_dir(directory.0.join("themes.json.lock")).unwrap();
    assert!(library.save(&[]).is_err());
    assert_eq!(library.themes(), &[original]);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    fs::remove_dir(directory.0.join("themes.json.lock")).unwrap();
    library.save(&[]).unwrap();
    assert!(ThemeLibraryStore::load(&path).unwrap().themes().is_empty());
    assert!(fs::read_dir(&directory.0).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp")
    }));
}
