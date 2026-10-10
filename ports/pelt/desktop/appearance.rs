// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Pelt's application-owned appearance setting.
//!
//! This is deliberately a small host seam. The theme choice and its stores
//! are tabard's (`tabard::theme::choice`); the store is injected so callers
//! can choose durable storage (or an in-memory store in tests) without making
//! the settings projection depend on a particular application or document
//! engine. Pelt keeps how it presents the choice: its Dark and Light options,
//! their labels, classes and action ids.

use mere_surface_api::settings::{
    SettingControl, SettingMovement, SettingMutability, SettingOption, SettingScope,
    SettingSecurity, SettingSpec, SettingValue, SettingsError, SettingsProvider,
};
use tabard::theme::choice::{ThemeChoice, ThemeChoiceStore};
use tabard::theme::registry::{Mode, THEME_ID_DARK, THEME_ID_LIGHT};
use workbench::SettingsRef;

pub const APPEARANCE_REFERENCE: &str = "pelt/appearance";
pub const CHROME_THEME_SETTING: &str = "chrome.theme";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppearanceTheme {
    Dark,
    Light,
}

impl AppearanceTheme {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
        }
    }

    pub(crate) const fn class(self) -> &'static str {
        match self {
            Self::Dark => "pelt-theme-dark",
            Self::Light => "pelt-theme-light",
        }
    }

    pub(crate) const fn action(self) -> &'static str {
        match self {
            Self::Dark => "appearance-dark",
            Self::Light => "appearance-light",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "dark" => Some(Self::Dark),
            "light" => Some(Self::Light),
            _ => None,
        }
    }

    /// The theme choice this option stores.
    pub fn choice(self) -> ThemeChoice {
        match self {
            Self::Dark => ThemeChoice::new(THEME_ID_DARK, Some(Mode::Dark)),
            Self::Light => ThemeChoice::new(THEME_ID_LIGHT, Some(Mode::Light)),
        }
    }

    /// How a stored choice presents: light modes as Light, other modes as
    /// Dark, and a choice without a mode by its theme.
    pub fn of(choice: &ThemeChoice) -> Self {
        match &choice.theme_mode {
            Some(Mode::Light | Mode::HcLight) => Self::Light,
            Some(_) => Self::Dark,
            None if choice.theme_id == THEME_ID_LIGHT => Self::Light,
            None => Self::Dark,
        }
    }
}

pub struct AppearanceSettingsProvider<S> {
    store: S,
}

impl<S> AppearanceSettingsProvider<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &S {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut S {
        &mut self.store
    }
}

impl<S: ThemeChoiceStore> SettingsProvider for AppearanceSettingsProvider<S> {
    fn describe(&self, reference: &SettingsRef) -> Result<Vec<SettingSpec>, SettingsError> {
        if reference.0 != APPEARANCE_REFERENCE {
            return Err(SettingsError::UnsupportedReference(reference.clone()));
        }
        Ok(vec![SettingSpec {
            id: CHROME_THEME_SETTING.into(),
            label: "Chrome theme".into(),
            scope: SettingScope::Application,
            movement: SettingMovement::LocalOnly,
            mutability: SettingMutability::Live,
            security: SettingSecurity::Ordinary,
            control: SettingControl::Choice {
                options: vec![
                    SettingOption {
                        value: "dark".into(),
                        label: "Dark".into(),
                    },
                    SettingOption {
                        value: "light".into(),
                        label: "Light".into(),
                    },
                ],
            },
            value: SettingValue::Text(AppearanceTheme::of(self.store.choice()).as_str().into()),
        }])
    }

    fn apply(
        &mut self,
        reference: &SettingsRef,
        setting_id: &str,
        value: SettingValue,
    ) -> Result<(), SettingsError> {
        if reference.0 != APPEARANCE_REFERENCE {
            return Err(SettingsError::UnsupportedReference(reference.clone()));
        }
        if setting_id != CHROME_THEME_SETTING {
            return Err(SettingsError::UnknownSetting(setting_id.into()));
        }
        let SettingValue::Text(value) = value else {
            return Err(SettingsError::InvalidValue {
                setting_id: setting_id.into(),
                message: "expected Text".into(),
            });
        };
        let Some(theme) = AppearanceTheme::parse(&value) else {
            return Err(SettingsError::InvalidValue {
                setting_id: setting_id.into(),
                message: "expected one of dark, light".into(),
            });
        };
        self.store
            .set_choice(theme.choice())
            .map_err(|error| SettingsError::Storage(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use tabard::theme::choice::InMemoryThemeChoiceStore;

    use super::*;

    fn reference() -> SettingsRef {
        SettingsRef(APPEARANCE_REFERENCE.into())
    }

    #[test]
    fn provider_describes_live_local_theme_choice() {
        let provider = AppearanceSettingsProvider::new(InMemoryThemeChoiceStore::default());
        let spec = &provider.describe(&reference()).unwrap()[0];
        assert_eq!(spec.id, CHROME_THEME_SETTING);
        assert_eq!(spec.scope, SettingScope::Application);
        assert_eq!(spec.movement, SettingMovement::LocalOnly);
        assert_eq!(spec.mutability, SettingMutability::Live);
        assert_eq!(spec.security, SettingSecurity::Ordinary);
        assert_eq!(spec.value, SettingValue::Text("dark".into()));
        assert_eq!(
            spec.control,
            SettingControl::Choice {
                options: vec![
                    SettingOption {
                        value: "dark".into(),
                        label: "Dark".into(),
                    },
                    SettingOption {
                        value: "light".into(),
                        label: "Light".into(),
                    },
                ],
            }
        );
    }

    #[test]
    fn provider_rejects_unknown_refs_keys_types_and_values() {
        let mut provider = AppearanceSettingsProvider::new(InMemoryThemeChoiceStore::default());
        assert!(matches!(
            provider.describe(&SettingsRef("other".into())),
            Err(SettingsError::UnsupportedReference(_))
        ));
        assert!(matches!(
            provider.apply(&reference(), "other", SettingValue::Text("dark".into())),
            Err(SettingsError::UnknownSetting(_))
        ));
        assert!(matches!(
            provider.apply(
                &SettingsRef("other".into()),
                CHROME_THEME_SETTING,
                SettingValue::Text("dark".into())
            ),
            Err(SettingsError::UnsupportedReference(_))
        ));
        assert!(matches!(
            provider.apply(
                &reference(),
                CHROME_THEME_SETTING,
                SettingValue::Boolean(true)
            ),
            Err(SettingsError::InvalidValue { .. })
        ));
        assert!(matches!(
            provider.apply(
                &reference(),
                CHROME_THEME_SETTING,
                SettingValue::Text("blue".into())
            ),
            Err(SettingsError::InvalidValue { .. })
        ));
    }

    #[test]
    fn in_memory_store_updates_live_value() {
        let mut provider = AppearanceSettingsProvider::new(InMemoryThemeChoiceStore::default());
        assert!(!provider.store().is_persistent());
        provider
            .apply(
                &reference(),
                CHROME_THEME_SETTING,
                SettingValue::Text("light".into()),
            )
            .unwrap();
        assert_eq!(provider.store().choice(), &AppearanceTheme::Light.choice());
        assert_eq!(
            AppearanceTheme::of(provider.store().choice()),
            AppearanceTheme::Light
        );
    }

    #[test]
    fn stored_choices_present_as_dark_or_light() {
        let of = |id: &str, mode| AppearanceTheme::of(&ThemeChoice::new(id, mode));
        assert_eq!(of("theme:default", Some(Mode::Dark)), AppearanceTheme::Dark);
        assert_eq!(of("user:mine", Some(Mode::HcLight)), AppearanceTheme::Light);
        assert_eq!(
            of("user:mine", Some(Mode::Custom("solar".into()))),
            AppearanceTheme::Dark
        );
        assert_eq!(of(THEME_ID_LIGHT, None), AppearanceTheme::Light);
        assert_eq!(of("theme:default", None), AppearanceTheme::Dark);
        for option in [AppearanceTheme::Dark, AppearanceTheme::Light] {
            assert_eq!(AppearanceTheme::of(&option.choice()), option);
        }
    }
}

/// Pelt keeps its application choice separately from the shared definitions.
#[cfg(feature = "livery")]
pub fn default_appearance_store_path() -> std::io::Result<std::path::PathBuf> {
    dirs::config_dir()
        .map(|directory| directory.join("mere").join("pelt").join("appearance.json"))
        .ok_or_else(|| {
            std::io::Error::other("No configuration directory; pass --appearance-store PATH")
        })
}

/// The standalone workshop and applications share this authored library.
#[cfg(feature = "livery")]
pub fn default_theme_library_path() -> std::io::Result<std::path::PathBuf> {
    dirs::data_local_dir()
        .map(|directory| directory.join("mere").join("tabard").join("themes.json"))
        .ok_or_else(|| {
            std::io::Error::other("No application data directory; pass --theme-library PATH")
        })
}

#[cfg(feature = "livery")]
pub(crate) struct ThemeCatalog {
    pub path: Option<std::path::PathBuf>,
    pub registry: tabard::theme::registry::ThemeRegistry,
    pub error: Option<String>,
}

#[cfg(feature = "livery")]
impl ThemeCatalog {
    pub fn new(path: Option<std::path::PathBuf>) -> Self {
        let mut catalog = Self {
            path,
            registry: tabard::theme::registry::ThemeRegistry::default(),
            error: None,
        };
        catalog.reload();
        catalog
    }

    pub fn reload(&mut self) -> bool {
        let Some(path) = self.path.as_ref() else {
            return true;
        };
        let result = tabard::library::ThemeLibraryStore::load(path.clone()).and_then(|library| {
            let mut registry = tabard::theme::registry::ThemeRegistry::default();
            for definition in library.themes() {
                registry
                    .add_user_theme(definition.clone())
                    .map_err(std::io::Error::other)?;
            }
            Ok(registry)
        });
        match result {
            Ok(registry) => {
                self.registry = registry;
                self.error = None;
                true
            },
            Err(error) => {
                self.error = Some(format!(
                    "Could not load theme library {}: {error}",
                    path.display()
                ));
                false
            },
        }
    }

    pub fn user_themes(&self) -> Vec<&tabard::Theme> {
        self.registry
            .list()
            .into_iter()
            .filter(|theme| theme.source == tabard::theme::registry::ThemeSource::User)
            .collect()
    }

    pub fn modes(&self, choice: &ThemeChoice) -> Vec<Mode> {
        let mut modes = vec![Mode::Light, Mode::Dark, Mode::HcLight, Mode::HcDark];
        if let Some(theme) = self.registry.theme_def(&choice.theme_id) {
            modes.extend(
                theme
                    .mode_sheets
                    .iter()
                    .filter(|(_, rules)| !rules.is_empty())
                    .filter_map(|(key, _)| Mode::from_key(key))
                    .filter(|mode| matches!(mode, Mode::Custom(_))),
            );
        }
        modes
    }

    /// Validation precedes writing the application choice. Preview drafts never
    /// enter this path: the ID must name a registered definition read from disk.
    pub fn apply_choice(
        &mut self,
        store: &mut dyn ThemeChoiceStore,
        choice: ThemeChoice,
    ) -> Result<(), String> {
        if !self.reload() {
            return Err(self.error.clone().unwrap());
        }
        let theme = self
            .registry
            .theme_def(&choice.theme_id)
            .ok_or_else(|| format!("Theme {} is not saved in the library", choice.theme_id))?;
        let mode = choice
            .theme_mode
            .clone()
            .unwrap_or_else(|| tabard::theme::seed::default_mode_for_def(theme));
        theme
            .presentation_for_mode(&mode)
            .map_err(|error| error.to_string())?;
        store
            .set_choice(choice)
            .map_err(|error| format!("Could not save Pelt appearance: {error}"))
    }
}

#[cfg(feature = "livery")]
pub fn load_appearance_store(
    path: impl Into<std::path::PathBuf>,
) -> std::io::Result<tabard::theme::choice::FileThemeChoiceStore> {
    let path = path.into();
    let store = tabard::theme::choice::FileThemeChoiceStore::load_strict(path.clone())?;
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    Ok(store)
}

#[cfg(all(test, feature = "livery"))]
mod catalog_tests {
    use super::*;
    use tabard::library::ThemeLibraryStore;
    use tabard::theme::choice::InMemoryThemeChoiceStore;
    use tabard::theme::registry::{THEME_ID_DEFAULT, ThemeRegistry};
    use tabard::{Theme, ThemePresentation, resolve_theme_choice};

    fn authored() -> Theme {
        let registry = ThemeRegistry::default();
        let mut theme = Theme::new(
            "theme:pelt-test",
            "Pelt test",
            registry.theme_def(THEME_ID_DEFAULT).unwrap().seeds,
        );
        theme.mode_sheets.insert(
            "custom:concert".into(),
            vec![".pelt-toolbar { background: #713f92; }".into()],
        );
        theme
    }

    #[test]
    fn saved_definition_and_exact_mode_survive_independent_store_recreation() {
        let temporary = tempfile::tempdir().unwrap();
        let library_path = temporary.path().join("shared/themes.json");
        let selection_path = temporary.path().join("pelt/appearance.json");
        let theme = authored();
        ThemeLibraryStore::load(&library_path)
            .unwrap()
            .save(&[theme.clone()])
            .unwrap();
        let library_bytes = std::fs::read(&library_path).unwrap();
        let mut catalog = ThemeCatalog::new(Some(library_path.clone()));
        let mut store = load_appearance_store(&selection_path).unwrap();
        let choice = ThemeChoice::new(&theme.id, Some(Mode::Custom("concert".into())));
        catalog.apply_choice(&mut store, choice.clone()).unwrap();
        drop(store);
        let restored = load_appearance_store(&selection_path).unwrap();
        let restored_catalog = ThemeCatalog::new(Some(library_path.clone()));
        assert_eq!(restored.choice(), &choice);
        assert_eq!(
            resolve_theme_choice(&restored_catalog.registry, restored.choice())
                .unwrap()
                .presentation,
            ThemePresentation::AuthoredStylesheet(theme.mode_sheets["custom:concert"].clone())
        );
        assert_eq!(std::fs::read(&library_path).unwrap(), library_bytes);
        assert_ne!(selection_path, library_path);
    }

    #[test]
    fn corrupt_library_and_invalid_modes_preserve_choice_and_original_bytes() {
        let temporary = tempfile::tempdir().unwrap();
        let path = temporary.path().join("themes.json");
        let theme = authored();
        ThemeLibraryStore::load(&path)
            .unwrap()
            .save(&[theme.clone()])
            .unwrap();
        let mut catalog = ThemeCatalog::new(Some(path.clone()));
        let mut store = InMemoryThemeChoiceStore::default();
        let before = store.choice().clone();
        assert!(
            catalog
                .apply_choice(
                    &mut store,
                    ThemeChoice::new(&theme.id, Some(Mode::Custom("missing".into())))
                )
                .is_err()
        );
        assert!(
            catalog
                .apply_choice(
                    &mut store,
                    ThemeChoice::new("theme:unsaved-preview", Some(Mode::Dark))
                )
                .is_err()
        );
        assert_eq!(store.choice(), &before);
        std::fs::write(&path, b"original malformed library").unwrap();
        assert!(
            catalog
                .apply_choice(&mut store, ThemeChoice::new(&theme.id, Some(Mode::Light)))
                .is_err()
        );
        assert!(
            catalog
                .error
                .as_deref()
                .unwrap()
                .contains("Could not load theme library")
        );
        assert_eq!(store.choice(), &before);
        assert_eq!(std::fs::read(&path).unwrap(), b"original malformed library");
    }

    #[test]
    fn failed_application_selection_write_keeps_current_choice_and_shared_definition() {
        let temporary = tempfile::tempdir().unwrap();
        let library = temporary.path().join("themes.json");
        let selection = temporary.path().join("appearance.json");
        let theme = authored();
        ThemeLibraryStore::load(&library)
            .unwrap()
            .save(&[theme.clone()])
            .unwrap();
        let library_bytes = std::fs::read(&library).unwrap();
        let mut catalog = ThemeCatalog::new(Some(library.clone()));
        let mut store = load_appearance_store(&selection).unwrap();
        let original = store.choice().clone();
        std::fs::create_dir(&selection).unwrap();
        std::fs::write(selection.join("preserved"), b"existing destination").unwrap();
        assert!(
            catalog
                .apply_choice(&mut store, ThemeChoice::new(&theme.id, Some(Mode::Dark)))
                .unwrap_err()
                .contains("Could not save Pelt appearance")
        );
        assert_eq!(store.choice(), &original);
        assert_eq!(
            std::fs::read(selection.join("preserved")).unwrap(),
            b"existing destination"
        );
        assert_eq!(std::fs::read(&library).unwrap(), library_bytes);
    }

    #[test]
    fn missing_definition_fallback_preserves_requested_durable_choice_and_corrupt_settings_are_rejected()
     {
        let temporary = tempfile::tempdir().unwrap();
        let path = temporary.path().join("appearance.json");
        let requested = ThemeChoice::new("theme:external-later", Some(Mode::HcLight));
        let mut store = load_appearance_store(&path).unwrap();
        store.set_choice(requested.clone()).unwrap();
        let before = std::fs::read(&path).unwrap();
        let catalog = ThemeCatalog::new(Some(temporary.path().join("absent.json")));
        let resolution = resolve_theme_choice(&catalog.registry, store.choice()).unwrap();
        assert!(!resolution.diagnostics.is_empty());
        assert_eq!(store.choice(), &requested);
        assert_eq!(std::fs::read(&path).unwrap(), before);
        std::fs::write(&path, b"invalid settings").unwrap();
        assert!(load_appearance_store(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"invalid settings");
    }
}
