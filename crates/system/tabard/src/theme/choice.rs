// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The theme a user chose, and where it is kept: a theme id with an optional
//! mode, the store seam, an in-memory store and an atomic file store. Hosts
//! present the choice their own way; Pelt's appearance setting is one.
//! Moved from Pelt's `appearance` module on 2026-09-24.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::theme::registry::{Mode, THEME_ID_DARK, THEME_ID_DEFAULT, THEME_ID_LIGHT};

/// A chosen theme: its id, and the mode to derive it in (`None` is the
/// theme's own default mode). Serialized as `theme_id` and `theme_mode`, the
/// mode by its key (`"dark"`, `"hc_light"`, `"custom:…"`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeChoice {
    pub theme_id: String,
    #[serde(default, with = "mode_key")]
    pub theme_mode: Option<Mode>,
}

impl ThemeChoice {
    pub fn new(theme_id: impl Into<String>, theme_mode: Option<Mode>) -> Self {
        Self {
            theme_id: theme_id.into(),
            theme_mode,
        }
    }

    /// Read a stored choice: the JSON form, or a legacy `dark` / `light` line
    /// from Pelt's one-line appearance file.
    pub fn parse(contents: &str) -> Option<Self> {
        match contents.trim() {
            "dark" => Some(Self::new(THEME_ID_DARK, Some(Mode::Dark))),
            "light" => Some(Self::new(THEME_ID_LIGHT, Some(Mode::Light))),
            json => serde_json::from_str(json).ok(),
        }
    }
}

impl Default for ThemeChoice {
    /// The registry's default theme, which is dark, in dark mode.
    fn default() -> Self {
        Self::new(THEME_ID_DEFAULT, Some(Mode::Dark))
    }
}

/// A mode by its key. An unknown key reads as no mode (the theme's default)
/// rather than failing the whole choice.
mod mode_key {
    use serde::{Deserialize, Deserializer, Serializer};

    use crate::theme::registry::Mode;

    pub fn serialize<S: Serializer>(mode: &Option<Mode>, serializer: S) -> Result<S::Ok, S::Error> {
        match mode {
            Some(mode) => serializer.serialize_some(&mode.as_key()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Mode>, D::Error> {
        let key = Option::<String>::deserialize(deserializer)?;
        Ok(key.and_then(|key| Mode::from_key(&key)))
    }
}

/// Where a theme choice is kept: the current choice, setting it, and whether
/// it outlives the process.
pub trait ThemeChoiceStore {
    fn choice(&self) -> &ThemeChoice;
    fn set_choice(&mut self, choice: ThemeChoice) -> io::Result<()>;

    fn is_persistent(&self) -> bool {
        false
    }
}

impl<T: ThemeChoiceStore + ?Sized> ThemeChoiceStore for Box<T> {
    fn choice(&self) -> &ThemeChoice {
        (**self).choice()
    }

    fn set_choice(&mut self, choice: ThemeChoice) -> io::Result<()> {
        (**self).set_choice(choice)
    }

    fn is_persistent(&self) -> bool {
        (**self).is_persistent()
    }
}

/// A choice kept for the life of the process.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InMemoryThemeChoiceStore {
    choice: ThemeChoice,
}

impl InMemoryThemeChoiceStore {
    pub fn new(choice: ThemeChoice) -> Self {
        Self { choice }
    }
}

impl ThemeChoiceStore for InMemoryThemeChoiceStore {
    fn choice(&self) -> &ThemeChoice {
        &self.choice
    }

    fn set_choice(&mut self, choice: ThemeChoice) -> io::Result<()> {
        self.choice = choice;
        Ok(())
    }
}

/// A choice kept in a small JSON file. A write goes to a synced temporary
/// file that is renamed over the old one, so a reader finds the old choice or
/// the new one, never a partial file. std's rename replaces the target in one
/// step on every platform (`MoveFileExW` with `MOVEFILE_REPLACE_EXISTING` on
/// Windows), so the store needs no platform code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileThemeChoiceStore {
    path: PathBuf,
    choice: ThemeChoice,
}

impl FileThemeChoiceStore {
    /// Loads the choice at `path`. A missing or unreadable choice starts at
    /// the default; other I/O failures stay visible to the caller.
    pub fn load(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        let choice = match fs::read_to_string(&path) {
            Ok(contents) => ThemeChoice::parse(&contents).unwrap_or_default(),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::InvalidData
                ) =>
            {
                ThemeChoice::default()
            },
            Err(error) => return Err(error),
        };
        Ok(Self { path, choice })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl ThemeChoiceStore for FileThemeChoiceStore {
    fn choice(&self) -> &ThemeChoice {
        &self.choice
    }

    fn set_choice(&mut self, choice: ThemeChoice) -> io::Result<()> {
        let contents = serde_json::to_string(&choice).map_err(io::Error::other)?;
        let temporary = temporary_path(&self.path);
        let written = (|| {
            let mut file = fs::File::create(&temporary)?;
            file.write_all(contents.as_bytes())?;
            file.write_all(b"\n")?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temporary, &self.path)
        })();
        if let Err(error) = written {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }
        self.choice = choice;
        Ok(())
    }

    fn is_persistent(&self) -> bool {
        true
    }
}

static TEMPORARY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn temporary_path(path: &Path) -> PathBuf {
    let sequence = TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    path.with_extension(format!(
        "tabard-theme-{}-{sequence}.tmp",
        std::process::id()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "tabard-{name}-{}-{}",
            std::process::id(),
            TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn legacy_lines_and_json_both_read() {
        assert_eq!(
            ThemeChoice::parse("dark\n"),
            Some(ThemeChoice::new(THEME_ID_DARK, Some(Mode::Dark)))
        );
        assert_eq!(
            ThemeChoice::parse("light"),
            Some(ThemeChoice::new(THEME_ID_LIGHT, Some(Mode::Light)))
        );
        let custom = ThemeChoice::new("user:solar", Some(Mode::Custom("solar".into())));
        let json = serde_json::to_string(&custom).unwrap();
        assert_eq!(
            json,
            r#"{"theme_id":"user:solar","theme_mode":"custom:solar"}"#
        );
        assert_eq!(ThemeChoice::parse(&json), Some(custom));
        // No mode is the theme's default; an unknown mode reads as no mode.
        assert_eq!(
            ThemeChoice::parse(r#"{"theme_id":"theme:light"}"#),
            Some(ThemeChoice::new(THEME_ID_LIGHT, None))
        );
        assert_eq!(
            ThemeChoice::parse(r#"{"theme_id":"theme:dark","theme_mode":"sepia"}"#),
            Some(ThemeChoice::new(THEME_ID_DARK, None))
        );
        assert_eq!(ThemeChoice::parse("blue"), None);
    }

    #[test]
    fn in_memory_store_updates_live_value() {
        let mut store = InMemoryThemeChoiceStore::default();
        assert!(!store.is_persistent());
        assert_eq!(store.choice(), &ThemeChoice::default());
        let light = ThemeChoice::new(THEME_ID_LIGHT, Some(Mode::Light));
        store.set_choice(light.clone()).unwrap();
        assert_eq!(store.choice(), &light);
    }

    #[test]
    fn file_store_defaults_missing_or_invalid_and_round_trips() {
        let path = scratch("choice");
        let _ = fs::remove_file(&path);
        let mut store = FileThemeChoiceStore::load(&path).unwrap();
        assert_eq!(store.choice(), &ThemeChoice::default());
        assert!(store.is_persistent());
        let light = ThemeChoice::new(THEME_ID_LIGHT, Some(Mode::Light));
        store.set_choice(light.clone()).unwrap();
        assert_eq!(FileThemeChoiceStore::load(&path).unwrap().choice(), &light);
        let unset_mode = ThemeChoice::new("user:mine", None);
        store.set_choice(unset_mode.clone()).unwrap();
        assert_eq!(
            FileThemeChoiceStore::load(&path).unwrap().choice(),
            &unset_mode
        );
        fs::write(&path, "blue").unwrap();
        assert_eq!(
            FileThemeChoiceStore::load(&path).unwrap().choice(),
            &ThemeChoice::default()
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn an_existing_pelt_appearance_file_reads() {
        let path = scratch("pelt-appearance");
        fs::write(&path, "light\n").unwrap();
        let mut store = FileThemeChoiceStore::load(&path).unwrap();
        assert_eq!(
            store.choice(),
            &ThemeChoice::new(THEME_ID_LIGHT, Some(Mode::Light))
        );
        // The next write replaces the legacy line with the JSON form.
        store
            .set_choice(ThemeChoice::new(THEME_ID_DARK, Some(Mode::Dark)))
            .unwrap();
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "{\"theme_id\":\"theme:dark\",\"theme_mode\":\"dark\"}\n"
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn file_store_surfaces_nonrecoverable_load_errors() {
        let path = scratch("choice-directory");
        fs::create_dir(&path).unwrap();
        assert!(FileThemeChoiceStore::load(&path).is_err());
        fs::remove_dir(path).unwrap();
    }
}
