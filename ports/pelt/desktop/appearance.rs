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
