// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Authored-library transactions and host-independent import/export requests.
//! Native hosts choose paths; this model owns validation and publication.

use std::io;
use std::path::{Component, Path, PathBuf};

use cambium::FileEvent;
use tabard::portable::{WriteMode, parse_theme_json, theme_json, write_artifact};
use tabard::theme::choice::{FileThemeChoiceStore, ThemeChoice, ThemeChoiceStore};
use tabard::theme::registry::{Mode, ThemeSource};
use tabard::theme::seed::default_mode_for_def;
use tabard::workshop::ThemeDraft;

use crate::WorkshopState;
use crate::state::registry_copy;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ExportFormat {
    #[default]
    ThemeJson,
    Css,
    Dtcg,
}

impl ExportFormat {
    pub const ALL: [Self; 3] = [Self::ThemeJson, Self::Css, Self::Dtcg];

    pub fn label(self) -> &'static str {
        match self {
            Self::ThemeJson => "Theme JSON",
            Self::Css => "CSS colors",
            Self::Dtcg => "DTCG tokens",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::ThemeJson => "theme.json",
            Self::Css => "css",
            Self::Dtcg => "tokens.json",
        }
    }

    pub fn as_key(self) -> &'static str {
        match self {
            Self::ThemeJson => "theme-json",
            Self::Css => "css",
            Self::Dtcg => "dtcg",
        }
    }
}

/// A captured draft export. Edits made while a host chooses its destination
/// cannot change the artifact or clear the draft's authored save point.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportArtifact {
    pub suggested_name: String,
    pub contents: String,
    pub format: ExportFormat,
}

impl WorkshopState {
    pub fn import_requested(&self) -> bool {
        self.import_requested
    }
    pub fn delete_requested(&self) -> bool {
        self.delete_requested
    }
    pub fn export_format(&self) -> ExportFormat {
        self.export_format
    }
    pub fn replacement_path(&self) -> Option<&Path> {
        self.pending_replace
            .as_ref()
            .map(|(_, path)| path.as_path())
    }
    pub fn set_export_format(&mut self, format: ExportFormat) {
        self.export_format = format;
    }

    pub fn request_import(&mut self) {
        if !self.prepare_edits() {
            return;
        }
        if self.has_changes() {
            self.status = "Save or discard your changes before importing a theme.".into();
            return;
        }
        self.import_requested = true;
        self.status = "Choose a theme JSON file.".into();
    }

    pub fn accept_import(&mut self, event: FileEvent) {
        self.import_requested = false;
        if event.files.is_empty() {
            self.status = "Import cancelled".into();
            return;
        }
        if event.files.len() != 1 {
            self.status = "Choose one theme JSON file to import.".into();
            return;
        }
        match std::str::from_utf8(&event.files[0].bytes) {
            Ok(json) => self.import_theme_json(json),
            Err(error) => {
                self.status = format!("Could not import: the theme file is not UTF-8 ({error}).")
            },
        }
    }

    /// Import into a new draft, preserving existing library definitions even
    /// when the source is a built-in or its normalized identity is occupied.
    pub fn import_theme_json(&mut self, json: &str) {
        if !self.prepare_edits() {
            return;
        }
        if self.has_changes() {
            self.status = "Save or discard your changes before importing a theme.".into();
            return;
        }
        let theme = match parse_theme_json(json) {
            Ok(theme) => theme,
            Err(error) => {
                self.status = format!("Could not import: {error}");
                return;
            },
        };
        let needs_copy =
            theme.source != ThemeSource::User || self.registry.theme_def(&theme.id).is_some();
        let result = if needs_copy {
            let id = (1u64..)
                .map(|number| format!("theme:import-{number}"))
                .find(|id| self.registry.theme_def(id).is_none() && self.draft_theme().id != *id)
                .expect("a fresh imported identity is available");
            ThemeDraft::fork_theme(&self.registry, &theme, &id, &theme.name)
        } else {
            ThemeDraft::from_theme(theme)
        };
        match result {
            Ok(draft) => {
                self.mode = default_mode_for_def(draft.theme());
                self.opening = draft.theme().clone();
                self.draft = draft;
                self.unsaved_intake = true;
                self.delete_requested = false;
                self.refresh_controls();
                self.status = if needs_copy {
                    format!(
                        "Imported as a new user copy ({}). Save to add it to your library.",
                        self.draft_theme().id
                    )
                } else {
                    "Theme imported. Save to add it to your library.".into()
                };
            },
            Err(error) => self.status = format!("Could not import: {error}"),
        }
    }

    pub fn request_delete(&mut self) {
        if !self.prepare_edits() {
            return;
        }
        if self.has_changes() {
            self.status = "Save or discard your changes before deleting this theme.".into();
            return;
        }
        if !matches!(self.registry.theme_def(&self.draft_theme().id), Some(theme) if theme.source == ThemeSource::User)
        {
            self.status = "Only a saved user theme can be deleted.".into();
            return;
        }
        self.delete_requested = true;
        self.status = format!("Delete {} from your library?", self.draft_theme().name);
    }

    pub fn cancel_delete(&mut self) {
        self.delete_requested = false;
        self.status = "Deletion cancelled".into();
    }

    pub fn confirm_delete(&mut self) {
        if !self.delete_requested || !self.prepare_edits() {
            return;
        }
        if self.has_changes() {
            self.status = "Save or discard your changes before deleting this theme.".into();
            return;
        }
        let mut candidate = registry_copy(&self.registry);
        if !candidate.remove_user_theme(&self.draft_theme().id) {
            self.status = "Only a saved user theme can be deleted.".into();
            self.delete_requested = false;
            return;
        }
        // Prepare the surviving editing state before publishing the deletion.
        let next = match Self::from_registry(candidate, None) {
            Ok(next) => next,
            Err(error) => {
                self.status = format!("Could not delete: {error}");
                return;
            },
        };
        let themes: Vec<_> = next
            .registry
            .list()
            .into_iter()
            .filter(|theme| theme.source == ThemeSource::User)
            .cloned()
            .collect();
        if let Some(library) = &mut self.library
            && let Err(error) = library.save(&themes)
        {
            self.status = format!("Could not delete: {error}");
            return;
        }
        self.registry = next.registry;
        self.draft = next.draft;
        self.opening = next.opening;
        self.mode = next.mode;
        self.unsaved_intake = false;
        self.delete_requested = false;
        self.refresh_controls();
        self.status = "Theme deleted".into();
        self.remember_editor_choice();
    }

    pub fn request_export(&mut self) {
        if !self.prepare_edits() {
            return;
        }
        self.pending_export = None;
        self.pending_replace = None;
        let format = self.export_format;
        let contents = match format {
            ExportFormat::ThemeJson => {
                theme_json(self.draft_theme()).map_err(|error| error.to_string())
            },
            ExportFormat::Css => self
                .draft_theme()
                .css_custom_properties_for_mode(self.mode())
                .map_err(|error| error.to_string()),
            ExportFormat::Dtcg => self
                .draft_theme()
                .design_tokens_json_for_mode(self.mode())
                .map_err(|error| error.to_string()),
        };
        match contents {
            Ok(contents) => {
                let mut name = safe_name(&self.draft_theme().id);
                if format != ExportFormat::ThemeJson {
                    name.push('-');
                    name.push_str(&safe_name(&self.mode_key()));
                }
                self.pending_export = Some(ExportArtifact {
                    suggested_name: format!("{name}.{}", format.extension()),
                    contents,
                    format,
                });
                self.pending_replace = None;
                self.status = format!("Choose where to export {}.", format.label());
            },
            Err(error) => {
                self.pending_export = None;
                self.status = format!("Could not export: {error}");
            },
        }
    }

    pub fn take_export(&mut self) -> Option<ExportArtifact> {
        self.pending_export.take()
    }

    /// Protect files owned by the embedding application, such as its settings
    /// or open documents. The host supplies its current authorities; the shared
    /// exporter checks their path identities before creation and replacement.
    /// This transient list never enters a theme or the authored library. The
    /// workshop's own library and preference files remain protected as well.
    pub fn set_protected_export_paths(&mut self, paths: Vec<PathBuf>) {
        self.protected_export_paths = paths;
    }

    /// Protect an application's storage directories, including future generated
    /// files. Directory boundaries are compared by path components, using the
    /// same alias handling as protected files. This transient policy does not
    /// enter the authored theme or library.
    pub fn set_protected_export_directories(&mut self, directories: Vec<PathBuf>) {
        self.protected_export_directories = directories;
    }

    pub fn complete_export(&mut self, artifact: ExportArtifact, path: Option<PathBuf>) {
        let Some(path) = path else {
            self.cancel_export();
            return;
        };
        if self.is_editor_path(&path) {
            self.status =
                "Choose an export path separate from your library and workshop settings or protected application files.".into();
            return;
        }
        match write_artifact(&path, &artifact.contents, WriteMode::CreateNew) {
            Ok(()) => {
                self.pending_replace = None;
                self.status = format!("Exported {} to {}", artifact.format.label(), path.display());
            },
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                self.pending_replace = Some((artifact, path));
                self.status = "This file already exists. Replace it or cancel the export.".into();
            },
            Err(error) => self.status = format!("Could not export: {error}"),
        }
    }

    pub fn replace_export(&mut self) {
        let Some((artifact, path)) = self.pending_replace.as_ref() else {
            return;
        };
        if self.is_editor_path(path) {
            self.status =
                "Choose an export path separate from your library and workshop settings or protected application files.".into();
            return;
        }
        match write_artifact(path, &artifact.contents, WriteMode::Replace) {
            Ok(()) => {
                self.status = format!("Exported {} to {}", artifact.format.label(), path.display());
                self.pending_replace = None;
            },
            Err(error) => self.status = format!("Could not replace export: {error}"),
        }
    }

    pub fn cancel_export(&mut self) {
        self.pending_export = None;
        self.pending_replace = None;
        self.status = "Export cancelled".into();
    }

    pub(crate) fn restore_editor_choice(&mut self) {
        let Some(library_path) = self.library_path() else {
            return;
        };
        let path = preference_path(library_path);
        let existed = path.exists();
        let preferences = match FileThemeChoiceStore::load(path) {
            Ok(preferences) => preferences,
            Err(error) => {
                self.status = format!("Could not restore workshop selection: {error}");
                return;
            },
        };
        let choice = preferences.choice().clone();
        self.preferences = Some(preferences);
        if !existed {
            return;
        }
        if matches!(self.registry.theme_def(&choice.theme_id), Some(theme) if theme.source == ThemeSource::User)
        {
            match ThemeDraft::open(&self.registry, &choice.theme_id) {
                Ok(draft) => {
                    self.mode = choice
                        .theme_mode
                        .filter(|mode| {
                            !matches!(mode, Mode::Custom(_))
                                || draft.theme().mode_sheet(mode).is_some()
                        })
                        .unwrap_or_else(|| default_mode_for_def(draft.theme()));
                    self.opening = draft.theme().clone();
                    self.draft = draft;
                    self.refresh_controls();
                },
                Err(error) => {
                    self.status = format!("Could not restore workshop selection: {error}")
                },
            }
        } else {
            self.status =
                "The previous workshop theme is unavailable; opened a remaining theme.".into();
        }
    }

    pub(crate) fn remember_editor_choice(&mut self) {
        if !matches!(self.registry.theme_def(&self.draft_theme().id), Some(theme) if theme.source == ThemeSource::User)
        {
            return;
        }
        let choice = ThemeChoice::new(&self.draft_theme().id, Some(self.mode.clone()));
        if self
            .preferences
            .as_ref()
            .is_some_and(|preferences| preferences.choice() == &choice)
        {
            return;
        }
        if let Some(preferences) = &mut self.preferences
            && let Err(error) = preferences.set_choice(choice)
        {
            self.status = format!(
                "{}; could not remember workshop selection: {error}",
                self.status
            );
        }
    }

    fn is_editor_path(&self, path: &Path) -> bool {
        if self
            .protected_export_paths
            .iter()
            .any(|reserved| same_path(path, reserved))
            || self
                .protected_export_directories
                .iter()
                .any(|directory| within_directory(path, directory))
        {
            return true;
        }
        let Some(library) = self.library_path() else {
            return false;
        };
        let preferences = preference_path(library);
        [
            library.to_path_buf(),
            preferences.clone(),
            suffixed(library, ".lock"),
            suffixed(&preferences, ".lock"),
        ]
        .iter()
        .any(|reserved| same_path(path, reserved))
    }
}

fn safe_name(id: &str) -> String {
    let name: String = id
        .chars()
        .take(100)
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect();
    let name = name.trim_matches(['-', '_']);
    if name.is_empty() {
        "theme".into()
    } else {
        name.into()
    }
}

fn preference_path(library: &Path) -> PathBuf {
    suffixed(library, ".workshop.json")
}

fn suffixed(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}

fn normalized_identity(path: &Path, canonical: bool) -> Option<PathBuf> {
    let mut ancestor = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(path)
    };
    if !canonical {
        let mut normalized = PathBuf::new();
        for component in ancestor.components() {
            match component {
                Component::CurDir => {},
                Component::ParentDir => {
                    normalized.pop();
                },
                component => normalized.push(component.as_os_str()),
            }
        }
        return Some(case_identity(normalized));
    }
    let mut suffix = Vec::new();
    // Missing output files still inherit the identity of their existing
    // parent directories. Resolve symlinks at the longest canonicalizable
    // ancestor before applying the missing suffix (including `..`).
    let mut normalized = loop {
        if let Ok(resolved) = ancestor.canonicalize() {
            break resolved;
        }
        suffix.push(
            ancestor
                .components()
                .next_back()?
                .as_os_str()
                .to_os_string(),
        );
        if !ancestor.pop() {
            return None;
        }
    };
    for segment in suffix.iter().rev() {
        for component in Path::new(segment).components() {
            match component {
                Component::CurDir => {},
                Component::ParentDir => {
                    normalized.pop();
                },
                component => normalized.push(component.as_os_str()),
            }
        }
    }
    Some(case_identity(normalized))
}

fn case_identity(path: PathBuf) -> PathBuf {
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        // Conservatively protect case aliases, including missing paths.
        PathBuf::from(path.as_os_str().to_string_lossy().to_lowercase())
    } else {
        path
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    matches!((normalized_identity(left, true), normalized_identity(right, true)),
        (Some(left), Some(right)) if left == right)
}

fn within_directory(path: &Path, directory: &Path) -> bool {
    // Canonical comparison recognizes aliases into an owned directory. The
    // lexical comparison also protects an owned symlink entry whose target
    // lies outside it. Both retain component boundaries, so adjacent directory
    // names never become accidental matches.
    [true, false].into_iter().any(|canonical| {
        matches!((normalized_identity(path, canonical), normalized_identity(directory, canonical)),
            (Some(path), Some(directory)) if path.starts_with(&directory))
    })
}
