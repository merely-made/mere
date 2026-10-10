// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::io;
use std::path::{Path, PathBuf};
use std::{cell::RefCell, rc::Rc};

use cambium::{Slider, TextInput};
use tabard::Theme;
use tabard::library::ThemeLibraryStore;
use tabard::theme::choice::{FileThemeChoiceStore, ThemeChoice};
use tabard::theme::registry::{
    Harmony, Mode, THEME_ID_DEFAULT, ThemeRegistry, ThemeSource, set_user_theme_harmony,
};
use tabard::theme::seed::default_mode_for_def;
use tabard::workshop::{Edit, ThemeDraft};
use tinct::{Palette, Srgb, SyntaxPalette};

use crate::{
    ExportArtifact, ExportFormat, ReaderSpecimen, StylesheetSpecimen, graph::GraphSpecimen,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeedRole {
    Primary,
    Secondary,
    Tertiary,
    Neutral,
}

impl SeedRole {
    pub const ALL: [Self; 4] = [
        Self::Primary,
        Self::Secondary,
        Self::Tertiary,
        Self::Neutral,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
            Self::Tertiary => "Tertiary",
            Self::Neutral => "Neutral",
        }
    }
    pub fn color(self, theme: &Theme) -> Srgb {
        match self {
            Self::Primary => theme.seeds.primary,
            Self::Secondary => theme.seeds.secondary,
            Self::Tertiary => theme.seeds.tertiary,
            Self::Neutral => theme.seeds.neutral,
        }
    }
    fn set(self, theme: &mut Theme, color: Srgb) {
        match self {
            Self::Primary => theme.seeds.primary = color,
            Self::Secondary => theme.seeds.secondary = color,
            Self::Tertiary => theme.seeds.tertiary = color,
            Self::Neutral => theme.seeds.neutral = color,
        }
    }
}

/// Retained workshop state. Preview selection and text-control focus are
/// local to the workshop; authored definitions are saved explicitly.
pub struct WorkshopState {
    pub(crate) registry: ThemeRegistry,
    pub(crate) draft: ThemeDraft,
    pub(crate) opening: Theme,
    pub(crate) library: Option<ThemeLibraryStore>,
    pub(crate) mode: Mode,
    seed: SeedRole,
    pub(crate) name: TextInput,
    pub(crate) seed_hex: TextInput,
    pub(crate) sheet_text: TextInput,
    pub(crate) hex_error: Option<String>,
    pub(crate) channels: [Slider; 3],
    pub(crate) status: String,
    pub(crate) import_requested: bool,
    pub(crate) delete_requested: bool,
    pub(crate) preferences: Option<FileThemeChoiceStore>,
    pub(crate) pending_export: Option<ExportArtifact>,
    pub(crate) pending_replace: Option<(ExportArtifact, PathBuf)>,
    pub(crate) protected_export_paths: Vec<PathBuf>,
    pub(crate) protected_export_directories: Vec<PathBuf>,
    pub(crate) export_format: ExportFormat,
    pub(crate) advanced_open: bool,
    pub(crate) unsaved_intake: bool,
    pub(crate) close_requested: bool,
    pub(crate) exit_requested: bool,
    pub(crate) graph: GraphSpecimen,
    reader: Rc<RefCell<ReaderSpecimen>>,
    stylesheet: Rc<RefCell<StylesheetSpecimen>>,
    appearance: RefCell<Option<(tinct::Seeds, Harmony, Mode, tabard::ModePalette)>>,
}

impl WorkshopState {
    pub fn in_memory() -> Self {
        Self::from_registry(ThemeRegistry::default(), None).expect("built-in theme is valid")
    }

    pub fn load(path: impl Into<PathBuf>) -> io::Result<Self> {
        let library = ThemeLibraryStore::load(path)?;
        let mut registry = ThemeRegistry::default();
        for theme in library.themes() {
            registry
                .add_user_theme(theme.clone())
                .map_err(io::Error::other)?;
        }
        let mut state = Self::from_registry(registry, Some(library))?;
        state.restore_editor_choice();
        Ok(state)
    }

    pub(crate) fn from_registry(
        registry: ThemeRegistry,
        library: Option<ThemeLibraryStore>,
    ) -> io::Result<Self> {
        let last_user = registry
            .list()
            .into_iter()
            .rev()
            .find(|t| t.source == ThemeSource::User);
        let (draft, mode) = if let Some(theme) = last_user {
            (
                ThemeDraft::open(&registry, &theme.id).map_err(io::Error::other)?,
                default_mode_for_def(theme),
            )
        } else {
            (
                ThemeDraft::fork(
                    &registry,
                    THEME_ID_DEFAULT,
                    "theme:untitled",
                    "Untitled theme",
                )
                .map_err(io::Error::other)?,
                Mode::Light,
            )
        };
        let opening = draft.theme().clone();
        let mut state = Self {
            registry,
            draft,
            opening,
            library,
            mode,
            seed: SeedRole::Primary,
            name: TextInput::new(""),
            seed_hex: TextInput::new(""),
            sheet_text: TextInput::new(""),
            hex_error: None,
            channels: [Slider::default(), Slider::default(), Slider::default()],
            status: "Choose a seed and shape your theme.".into(),
            import_requested: false,
            delete_requested: false,
            preferences: None,
            pending_export: None,
            pending_replace: None,
            protected_export_paths: Vec::new(),
            protected_export_directories: Vec::new(),
            export_format: ExportFormat::ThemeJson,
            advanced_open: false,
            unsaved_intake: false,
            close_requested: false,
            exit_requested: false,
            graph: GraphSpecimen::default(),
            reader: Rc::new(RefCell::new(ReaderSpecimen::default())),
            stylesheet: Rc::new(RefCell::new(StylesheetSpecimen::default())),
            appearance: RefCell::new(None),
        };
        state.refresh_controls();
        Ok(state)
    }

    pub fn draft_theme(&self) -> &Theme {
        self.draft.theme()
    }
    pub fn registry(&self) -> &ThemeRegistry {
        &self.registry
    }
    pub fn mode(&self) -> &Mode {
        &self.mode
    }
    pub fn mode_key(&self) -> String {
        self.mode.as_key()
    }
    pub fn seed_role(&self) -> SeedRole {
        self.seed
    }
    pub fn status(&self) -> &str {
        &self.status
    }
    pub fn is_dirty(&self) -> bool {
        self.draft.is_dirty() || self.has_pending_fields()
    }
    pub fn has_changes(&self) -> bool {
        self.unsaved_intake || self.draft_theme() != &self.opening || self.has_pending_fields()
    }
    pub fn library_path(&self) -> Option<&Path> {
        self.library.as_ref().map(ThemeLibraryStore::path)
    }

    /// The exact registered save point and preview mode a host may explicitly
    /// apply. Staged fields and unsaved intakes never produce an app choice.
    /// Library persistence (when configured) completed before registration.
    pub fn saved_choice(&self) -> Result<ThemeChoice, String> {
        if self.has_changes() || self.name.text() != self.draft_theme().name {
            return Err("Save the theme before applying it to the application.".into());
        }
        let theme = self.draft_theme();
        if theme.source != ThemeSource::User || self.registry.theme_def(&theme.id) != Some(theme) {
            return Err("Save an authored copy before applying it to the application.".into());
        }
        theme
            .presentation_for_mode(self.mode())
            .map_err(|error| error.to_string())?;
        Ok(ThemeChoice::new(theme.id.clone(), Some(self.mode.clone())))
    }

    /// Open a saved user definition or fork an external/built-in definition
    /// into a fresh user draft. All validation happens before replacing the
    /// editor state; failure preserves staged fields and the current draft.
    /// This never activates a registry theme or writes the authored library.
    pub fn edit_definition(&mut self, theme: &Theme, mode: Option<Mode>) -> Result<(), String> {
        if self.has_changes() || self.name.text() != self.draft_theme().name {
            return Err("Save or discard the current changes before editing another theme.".into());
        }
        let mode = mode.unwrap_or_else(|| default_mode_for_def(theme));
        theme
            .presentation_for_mode(&mode)
            .map_err(|error| error.to_string())?;
        let registered =
            theme.source == ThemeSource::User && self.registry.theme_def(&theme.id) == Some(theme);
        let draft = if registered {
            ThemeDraft::open(&self.registry, &theme.id)
        } else {
            let id = (1u64..)
                .map(|number| format!("theme:copy-{number}"))
                .find(|id| self.registry.theme_def(id).is_none() && self.draft_theme().id != *id)
                .ok_or_else(|| "No fresh theme identity is available.".to_string())?;
            ThemeDraft::fork_theme(&self.registry, theme, &id, &format!("{} copy", theme.name))
        }
        .map_err(|error| error.to_string())?;
        self.mode = mode;
        self.opening = draft.theme().clone();
        self.draft = draft;
        self.unsaved_intake = !registered;
        self.delete_requested = false;
        self.close_requested = false;
        self.exit_requested = false;
        self.refresh_controls();
        self.status = if registered {
            "Theme ready to edit"
        } else {
            "User copy ready to edit. Save to add it to your library."
        }
        .into();
        self.remember_editor_choice();
        Ok(())
    }
    pub fn hue_follows_primary(&self) -> bool {
        matches!(self.seed, SeedRole::Secondary | SeedRole::Tertiary)
            && matches!(
                self.draft_theme().harmony,
                tabard::theme::registry::Harmony::Locked { .. }
            )
    }

    pub fn text_field(&self, key: &str) -> Option<&TextInput> {
        match key {
            "name" => Some(&self.name),
            "seed-hex" => Some(&self.seed_hex),
            "mode-sheet" => Some(&self.sheet_text),
            _ => None,
        }
    }
    pub fn text_field_mut(&mut self, key: &str) -> Option<&mut TextInput> {
        match key {
            "name" => Some(&mut self.name),
            "seed-hex" => Some(&mut self.seed_hex),
            "mode-sheet" => Some(&mut self.sheet_text),
            _ => None,
        }
    }

    pub(crate) fn refresh_controls(&mut self) {
        if matches!(self.mode, Mode::Custom(_))
            && self.draft_theme().mode_sheet(&self.mode).is_none()
        {
            self.mode = default_mode_for_def(self.draft_theme());
        }
        self.name = TextInput::new(self.draft_theme().name.clone());
        self.seed_hex = TextInput::new(tinct::color_to_hex(self.seed.color(self.draft_theme())));
        self.sheet_text = TextInput::new(self.authored_sheet());
        self.hex_error = None;
        let (h, s, l) = tinct::color_to_hsl(self.seed.color(self.draft_theme()));
        self.channels = [
            Slider::new((h / 360.0) as f32)
                .with_steps(1.0 / 360.0, 15.0 / 360.0)
                .with_label(format!("{} hue", self.seed.label())),
            Slider::new(s as f32).with_label(format!("{} saturation", self.seed.label())),
            Slider::new(l as f32).with_label(format!("{} lightness", self.seed.label())),
        ];
    }

    /// Used after native IME updates; ordinary controls synchronize their own
    /// model action in the retained view's message path.
    pub fn sync_controls(&mut self) {
        self.sync_name();
    }

    pub(crate) fn sync_name(&mut self) {
        if self.name.text() != self.draft_theme().name {
            self.draft.edit(Edit::Name(self.name.text().to_owned()));
            self.status = "Unsaved changes".into();
        }
    }

    pub(crate) fn sync_channels(&mut self) {
        let mut theme = self.draft_theme().clone();
        let old = self.seed.color(&theme);
        let mut color = tinct::color_from_hsl(
            self.channels[0].value as f64 * 360.0,
            self.channels[1].value as f64,
            self.channels[2].value as f64,
        );
        color.a = old.a;
        self.seed.set(&mut theme, color);
        if self.draft.edit(Edit::Seeds(theme.seeds)) {
            self.seed_hex = TextInput::new(tinct::color_to_hex(color));
            self.hex_error = None;
            self.status = "Unsaved changes".into();
        }
    }

    pub fn set_seed(&mut self, seed: SeedRole) {
        if !self.prepare_edits() {
            return;
        }
        self.seed = seed;
        self.refresh_controls();
    }

    pub fn set_mode(&mut self, mode: Mode) {
        if !self.prepare_edits() {
            return;
        }
        if matches!(mode, Mode::Custom(_)) && self.draft_theme().mode_sheet(&mode).is_none() {
            self.status = "This custom mode needs an authored stylesheet.".into();
            return;
        }
        self.mode = mode;
        self.sheet_text = TextInput::new(self.authored_sheet());
        self.status = format!("Previewing {}", self.mode.label().to_lowercase());
        self.remember_editor_choice();
    }

    pub fn set_harmony(&mut self, key: &str) {
        if !self.prepare_edits() {
            return;
        }
        let mut theme = self.draft_theme().clone();
        if set_user_theme_harmony(&mut theme, key) && self.draft.edit(Edit::Harmony(theme.harmony))
        {
            self.status = "Unsaved changes".into();
        }
    }

    pub fn undo(&mut self) {
        if self.has_pending_fields() {
            self.refresh_controls();
            self.status = "Unapplied input discarded".into();
            return;
        }
        self.sync_name();
        if self.draft.undo() {
            self.refresh_controls();
            self.status = "Undid the last change".into();
        }
    }
    pub fn redo(&mut self) {
        if !self.prepare_edits() {
            return;
        }
        if self.draft.redo() {
            self.refresh_controls();
            self.status = "Restored the change".into();
        }
    }
    pub fn discard(&mut self) {
        self.delete_requested = false;
        self.draft.discard();
        self.unsaved_intake = false;
        self.refresh_controls();
        self.status = "Changes discarded".into();
    }

    /// Fork the current look into a fresh identity without altering its source.
    pub fn new_copy(&mut self) {
        self.delete_requested = false;
        if !self.prepare_edits() {
            return;
        }
        if self.has_changes() {
            self.status = "Save or discard your changes before starting a new copy.".into();
            return;
        }
        let mut number = 1;
        let id = loop {
            let id = format!("theme:untitled-{number}");
            if self.registry.theme_def(&id).is_none() && self.draft_theme().id != id {
                break id;
            }
            number += 1;
        };
        let source = self.draft_theme().clone();
        match ThemeDraft::fork_theme(
            &self.registry,
            &source,
            &id,
            &format!("{} copy", source.name),
        ) {
            Ok(draft) => {
                self.opening = draft.theme().clone();
                self.draft = draft;
                self.unsaved_intake = true;
                self.refresh_controls();
                self.status = "New copy ready to edit".into();
            },
            Err(error) => self.status = error.to_string(),
        }
    }

    pub fn select_theme(&mut self, id: &str) {
        self.delete_requested = false;
        if !self.prepare_edits() {
            return;
        }
        if self.has_changes() {
            self.status = "Save or discard your changes before choosing another theme.".into();
            return;
        }
        let Some(theme) = self.registry.theme_def(id) else {
            return;
        };
        let result = if theme.source == ThemeSource::User {
            ThemeDraft::open(&self.registry, id)
        } else {
            let mut number = 1;
            let new_id = loop {
                let candidate = format!("theme:copy-{number}");
                if self.registry.theme_def(&candidate).is_none() {
                    break candidate;
                }
                number += 1;
            };
            ThemeDraft::fork(&self.registry, id, &new_id, &format!("{} copy", theme.name))
        };
        match result {
            Ok(draft) => {
                self.mode = default_mode_for_def(draft.theme());
                self.opening = draft.theme().clone();
                self.draft = draft;
                self.unsaved_intake = false;
                self.refresh_controls();
                self.status = "Theme ready to edit".into();
                self.remember_editor_choice();
            },
            Err(error) => self.status = error.to_string(),
        }
    }

    pub fn save(&mut self) {
        if !self.prepare_edits() {
            return;
        }
        let mut candidate_registry = registry_copy(&self.registry);
        let mut candidate_draft = self.draft.clone();
        if let Err(error) = candidate_draft.commit(&mut candidate_registry) {
            self.status = error.to_string();
            return;
        }
        let themes: Vec<Theme> = candidate_registry
            .list()
            .into_iter()
            .filter(|t| t.source == ThemeSource::User)
            .cloned()
            .collect();
        if let Some(store) = &mut self.library
            && let Err(error) = store.save(&themes)
        {
            self.status = format!("Could not save: {error}");
            return;
        }
        self.registry = candidate_registry;
        self.draft = candidate_draft;
        self.opening = self.draft_theme().clone();
        self.unsaved_intake = false;
        self.status = if self.library.is_some() {
            "Theme saved".into()
        } else {
            "Theme saved for this session".into()
        };
        self.remember_editor_choice();
    }

    pub fn reopen(&mut self) {
        self.delete_requested = false;
        if !self.prepare_edits() {
            return;
        }
        if self.has_changes() {
            self.status = "Save or discard your changes before reopening the library.".into();
            return;
        }
        let Some(path) = self.library_path().map(Path::to_path_buf) else {
            self.status = "This library lasts for the current session.".into();
            return;
        };
        match Self::load(path) {
            Ok(state) => {
                *self = state;
                self.status = "Saved library reopened".into();
            },
            Err(error) => self.status = format!("Could not reopen: {error}"),
        }
    }

    pub fn preview_palette(&self) -> Palette {
        self.derived_appearance().palette
    }
    pub fn preview_syntax(&self) -> SyntaxPalette {
        self.derived_appearance().syntax
    }

    // These typed specimens deliberately show the seed-derived fallback when
    // the active mode has arbitrary CSS. The isolated application document
    // renders that exact stylesheet instead. All canonical math lives in Tabard.
    fn derived_appearance(&self) -> tabard::ModePalette {
        let mode = match &self.mode {
            Mode::Custom(_) => Mode::Dark,
            mode => mode.clone(),
        };
        let source = self.draft_theme();
        if let Some((seeds, harmony, cached_mode, appearance)) = &*self.appearance.borrow()
            && *seeds == source.seeds
            && *harmony == source.harmony
            && *cached_mode == mode
        {
            return appearance.clone();
        }
        let mut theme = source.clone();
        theme.mode_sheets.clear();
        // An incomplete name must not prevent editing or previewing colors.
        if theme.name.trim().is_empty() {
            theme.name = "Theme preview".into();
        }
        let appearance = theme
            .palette_for_mode(&mode)
            .expect("the draft's seed definition is validated");
        *self.appearance.borrow_mut() =
            Some((source.seeds, source.harmony, mode, appearance.clone()));
        appearance
    }

    pub fn authored_sheet(&self) -> String {
        self.draft_theme()
            .mode_sheet(&self.mode)
            .map(|rules| rules.join("\n"))
            .unwrap_or_default()
    }

    pub fn has_pending_fields(&self) -> bool {
        !self
            .seed_hex
            .text()
            .trim()
            .eq_ignore_ascii_case(&tinct::color_to_hex(self.seed.color(self.draft_theme())))
            || self.sheet_text.text() != self.authored_sheet()
    }

    /// Apply staged field edits before saving/exporting/navigation. Invalid
    /// RGB input stays visible and cannot silently disappear into a saved file.
    pub(crate) fn prepare_edits(&mut self) -> bool {
        self.sync_name();
        if !self.apply_hex() {
            return false;
        }
        self.apply_stylesheet();
        true
    }

    pub fn apply_hex(&mut self) -> bool {
        let Some(mut color) = tinct::color_from_hex(self.seed_hex.text()) else {
            self.hex_error = Some("Enter six hex digits, for example #3366C8.".into());
            self.status = "Fix the seed color before continuing.".into();
            return false;
        };
        let mut theme = self.draft_theme().clone();
        color.a = self.seed.color(&theme).a;
        self.seed.set(&mut theme, color);
        let changed = self.draft.edit(Edit::Seeds(theme.seeds));
        self.hex_error = None;
        if changed {
            // Refresh only seed controls; keep staged CSS and native name text.
            let sheet = self.sheet_text.clone();
            self.refresh_controls();
            self.sheet_text = sheet;
            self.status = "Seed color updated".into();
        } else if !self
            .seed_hex
            .text()
            .trim()
            .eq_ignore_ascii_case(&tinct::color_to_hex(color))
        {
            self.seed_hex = TextInput::new(tinct::color_to_hex(color));
        }
        true
    }

    pub fn apply_stylesheet(&mut self) {
        if self.sheet_text.text() == self.authored_sheet() {
            return;
        }
        let text = self.sheet_text.text().to_owned();
        let removed = text.trim().is_empty();
        self.draft.edit(Edit::ModeSheet {
            mode: self.mode.clone(),
            rules: if removed { vec![] } else { vec![text] },
        });
        if removed && matches!(self.mode, Mode::Custom(_)) {
            self.mode = default_mode_for_def(self.draft_theme());
            self.sheet_text = TextInput::new(self.authored_sheet());
            self.status = format!(
                "Custom stylesheet removed; previewing {}",
                self.mode.label()
            );
            self.remember_editor_choice();
            return;
        }
        self.status = if removed {
            "Derived appearance restored for this mode".into()
        } else {
            "Stylesheet applied to the application preview".into()
        };
    }

    pub fn clear_stylesheet(&mut self) {
        let previous_mode = self.mode.clone();
        self.sheet_text = TextInput::new("");
        self.apply_stylesheet();
        if self.mode == previous_mode {
            self.status = "Derived appearance restored for this mode".into();
        }
    }

    pub fn use_preview_as_default(&mut self) {
        if !self.prepare_edits() {
            return;
        }
        if matches!(self.mode, Mode::Custom(_)) {
            self.status = "Choose a standard default mode for the portable theme.".into();
            return;
        }
        self.draft.edit(Edit::DefaultMode(self.mode.clone()));
        self.status = format!("{} is the theme's default mode", self.mode.label());
    }

    /// Native close requests and embedding hosts share the same edit guard.
    /// An untouched initial blank draft can close immediately.
    pub fn request_close(&mut self) -> bool {
        self.sync_name();
        if self.exit_requested || (!self.has_changes() && !self.unsaved_intake) {
            return true;
        }
        self.close_requested = true;
        self.status = "Save your changes or close without saving.".into();
        false
    }

    pub fn close_requested(&self) -> bool {
        self.close_requested
    }
    pub fn exit_requested(&self) -> bool {
        self.exit_requested
    }
    pub fn cancel_close(&mut self) {
        self.close_requested = false;
        self.exit_requested = false;
        self.status = "Continue editing".into();
    }
    pub fn save_and_close(&mut self) {
        self.save();
        if !self.is_dirty() && !self.has_changes() {
            self.close_requested = false;
            self.exit_requested = true;
        }
    }
    pub fn discard_and_close(&mut self) {
        self.close_requested = false;
        self.exit_requested = true;
    }

    pub fn available_modes(&self) -> Vec<Mode> {
        let mut modes = vec![Mode::Light, Mode::Dark, Mode::HcLight, Mode::HcDark];
        modes.extend(
            self.draft_theme()
                .mode_sheets
                .iter()
                .filter(|(_, rules)| !rules.is_empty())
                .filter_map(|(key, _)| Mode::from_key(key))
                .filter(|mode| matches!(mode, Mode::Custom(_))),
        );
        modes
    }

    pub fn stylesheet_preview(&self) -> Rc<RefCell<StylesheetSpecimen>> {
        let rules = match self.draft.preview(&self.mode) {
            Ok(tabard::workshop::Preview::Stylesheet(rules)) => rules,
            _ => vec![self.derived_appearance().css_custom_properties()],
        };
        self.stylesheet
            .borrow_mut()
            .set_appearance(&self.mode, &rules);
        self.stylesheet.clone()
    }

    /// A shared renderer appearance of the held extracted article.
    pub fn reader_preview(&self) -> Rc<RefCell<ReaderSpecimen>> {
        self.reader
            .borrow_mut()
            .set_palette(ReaderSpecimen::palette_for_state(self));
        self.reader.clone()
    }

    /// The same graph leaf whose native targets the workshop view mounts.
    pub fn graph_leaf(&self) -> cambium::GraphCanvas {
        self.graph.paint_leaf(self.draft_theme(), self.mode())
    }

    pub fn selected_graph_node(&self) -> Option<u8> {
        self.graph.swatch.selected
    }
}

#[cfg(test)]
mod embedded_close_tests {
    use super::*;

    #[test]
    fn cancelling_discard_exit_restores_the_unsaved_guard_for_an_embedded_session() {
        let mut state = WorkshopState::in_memory();
        state.new_copy();
        assert!(!state.request_close());
        state.discard_and_close();
        assert!(state.exit_requested());
        state.cancel_close();
        assert!(!state.exit_requested());
        assert!(!state.close_requested());
        assert!(state.has_changes());
        assert!(!state.request_close());
        assert!(state.close_requested());
    }

    #[test]
    fn cancelling_saved_exit_does_not_skip_guards_after_the_next_edit() {
        let mut state = WorkshopState::in_memory();
        state.new_copy();
        state.save_and_close();
        assert!(state.exit_requested());
        assert!(!state.has_changes());
        state.cancel_close();
        assert!(!state.exit_requested());
        assert!(!state.close_requested());
        assert!(state.request_close());
        state.new_copy();
        assert!(!state.request_close());
        assert!(state.close_requested());
    }

    #[test]
    fn failed_save_of_initial_blank_draft_does_not_authorize_exit() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("themes.json");
        let mut state = WorkshopState::load(&path).unwrap();
        assert!(!state.has_changes());
        assert!(state.is_dirty());
        std::fs::write(&path, "external modification").unwrap();
        state.save_and_close();
        assert!(!state.exit_requested());
        assert!(state.status().contains("Could not save"));
        assert_eq!(
            std::fs::read_to_string(path).unwrap(),
            "external modification"
        );
    }

    #[test]
    fn failed_save_of_an_untouched_import_does_not_authorize_exit() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("themes.json");
        let mut state = WorkshopState::load(&path).unwrap();
        let mut theme = state.draft_theme().clone();
        theme.id = "theme:imported-close".into();
        theme.name = "Imported close".into();
        theme.source = ThemeSource::User;
        state.import_theme_json(&tabard::portable::theme_json(&theme).unwrap());
        assert!(state.has_changes());
        assert!(state.is_dirty());
        assert!(!state.request_close());
        std::fs::write(&path, "external modification").unwrap();
        state.save_and_close();
        assert!(!state.exit_requested());
        assert!(state.close_requested());
        assert!(state.has_changes());
        assert_eq!(state.draft_theme(), &theme);
        assert!(state.status().contains("Could not save"));
        assert_eq!(
            std::fs::read_to_string(path).unwrap(),
            "external modification"
        );
    }
}

pub(crate) fn registry_copy(registry: &ThemeRegistry) -> ThemeRegistry {
    let mut candidate = ThemeRegistry::default();
    for theme in registry
        .list()
        .into_iter()
        .filter(|t| t.source == ThemeSource::User)
    {
        candidate
            .add_user_theme(theme.clone())
            .expect("registered themes were validated");
    }
    candidate.set_active_theme(&registry.active_theme().resolved_id);
    candidate
}
