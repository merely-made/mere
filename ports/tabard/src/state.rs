// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::io;
use std::path::{Path, PathBuf};

use cambium::{Slider, TextInput};
use tabard::Theme;
use tabard::library::ThemeLibraryStore;
use tabard::theme::registry::{
    Mode, THEME_ID_DEFAULT, ThemeRegistry, ThemeSource, set_user_theme_harmony,
};
use tabard::theme::seed::{default_mode_for_def, harmonized_seeds};
use tabard::workshop::{Edit, ThemeDraft};
use tinct::{Palette, Srgb, SyntaxPalette};

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
    registry: ThemeRegistry,
    draft: ThemeDraft,
    opening: Theme,
    library: Option<ThemeLibraryStore>,
    mode: Mode,
    seed: SeedRole,
    pub(crate) name: TextInput,
    pub(crate) channels: [Slider; 3],
    status: String,
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
        Self::from_registry(registry, Some(library))
    }

    fn from_registry(
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
            channels: [Slider::default(), Slider::default(), Slider::default()],
            status: "Choose a seed and shape your theme.".into(),
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
        self.draft.is_dirty()
    }
    pub fn has_changes(&self) -> bool {
        self.draft_theme() != &self.opening
    }
    pub fn library_path(&self) -> Option<&Path> {
        self.library.as_ref().map(ThemeLibraryStore::path)
    }
    pub fn hue_follows_primary(&self) -> bool {
        matches!(self.seed, SeedRole::Secondary | SeedRole::Tertiary)
            && matches!(
                self.draft_theme().harmony,
                tabard::theme::registry::Harmony::Locked { .. }
            )
    }

    pub fn text_field(&self, key: &str) -> Option<&TextInput> {
        (key == "name").then_some(&self.name)
    }
    pub fn text_field_mut(&mut self, key: &str) -> Option<&mut TextInput> {
        (key == "name").then_some(&mut self.name)
    }

    fn refresh_controls(&mut self) {
        self.name = TextInput::new(self.draft_theme().name.clone());
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
            self.status = "Unsaved changes".into();
        }
    }

    pub fn set_seed(&mut self, seed: SeedRole) {
        self.sync_name();
        self.seed = seed;
        self.refresh_controls();
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.sync_name();
        if matches!(mode, Mode::Custom(_)) {
            self.status = "Custom mode previews need their own stylesheet renderer. Choose a built-in preview mode.".into();
            return;
        }
        self.mode = mode;
        self.status = format!("Previewing {}", self.mode.label().to_lowercase());
    }

    pub fn set_harmony(&mut self, key: &str) {
        self.sync_name();
        let mut theme = self.draft_theme().clone();
        if set_user_theme_harmony(&mut theme, key) && self.draft.edit(Edit::Harmony(theme.harmony))
        {
            self.status = "Unsaved changes".into();
        }
    }

    pub fn undo(&mut self) {
        self.sync_name();
        if self.draft.undo() {
            self.refresh_controls();
            self.status = "Undid the last change".into();
        }
    }
    pub fn redo(&mut self) {
        self.sync_name();
        if self.draft.redo() {
            self.refresh_controls();
            self.status = "Restored the change".into();
        }
    }
    pub fn discard(&mut self) {
        self.draft.discard();
        self.refresh_controls();
        self.status = "Changes discarded".into();
    }

    /// Fork the current look into a fresh identity without altering its source.
    pub fn new_copy(&mut self) {
        self.sync_name();
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
        let mut candidate = registry_copy(&self.registry);
        let source = self.draft_theme().clone();
        candidate
            .add_user_theme(source.clone())
            .expect("current draft is valid");
        match ThemeDraft::fork(&candidate, &source.id, &id, "Untitled theme") {
            Ok(draft) => {
                self.opening = draft.theme().clone();
                self.draft = draft;
                self.refresh_controls();
                self.status = "New copy ready to edit".into();
            },
            Err(error) => self.status = error.to_string(),
        }
    }

    pub fn select_theme(&mut self, id: &str) {
        self.sync_name();
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
                self.refresh_controls();
                self.status = "Theme ready to edit".into();
            },
            Err(error) => self.status = error.to_string(),
        }
    }

    pub fn save(&mut self) {
        self.sync_name();
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
        self.status = if self.library.is_some() {
            "Theme saved".into()
        } else {
            "Theme saved for this session".into()
        };
    }

    pub fn reopen(&mut self) {
        self.sync_name();
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
        tinct::derive_palette_with(
            &harmonized_seeds(self.draft_theme()),
            tinct::ModeProfile {
                dark: self.mode.dark(),
                high_contrast: self.mode.high_contrast(),
            },
        )
    }
    pub fn preview_syntax(&self) -> SyntaxPalette {
        tinct::derive_syntax_palette_with(
            &harmonized_seeds(self.draft_theme()),
            tinct::ModeProfile {
                dark: self.mode.dark(),
                high_contrast: self.mode.high_contrast(),
            },
        )
    }
}

fn registry_copy(registry: &ThemeRegistry) -> ThemeRegistry {
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
