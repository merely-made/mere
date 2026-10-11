// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The theme a lens carries: a background, an accent, a font scale and a
//! stroke width. Built-in ids resolve to the built-in token sets' derived
//! `theme_data`, so a lens and the chrome come from the same seeds.

use std::sync::LazyLock;

use serde::Deserialize;

use crate::theme::registry::{
    THEME_ID_DARK, THEME_ID_DEFAULT, THEME_ID_HIGH_CONTRAST, THEME_ID_LIGHT,
};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ThemeData {
    pub background_rgb: (u8, u8, u8),
    pub accent_rgb: (u8, u8, u8),
    pub font_scale: f32,
    pub stroke_width: f32,
}

#[derive(Debug, Clone)]
pub struct ThemeResolution {
    pub requested_id: String,
    pub resolved_id: String,
    pub matched: bool,
    pub fallback_used: bool,
    pub theme_id: String,
    pub theme: ThemeData,
}

/// The built-in token sets' lens theme data, derived once from their seeds.
static BUILTINS: LazyLock<Vec<(String, ThemeData)>> = LazyLock::new(|| {
    crate::theme::seed::builtin_token_sets()
        .into_iter()
        .map(|tokens| (tokens.theme_id, tokens.theme_data))
        .collect()
});

fn builtin(id: &str) -> Option<ThemeData> {
    BUILTINS
        .iter()
        .find(|(builtin_id, _)| builtin_id == id)
        .map(|(_, theme)| theme.clone())
}

pub fn resolve_theme_data(theme_id: &str) -> ThemeResolution {
    let requested = theme_id.trim().to_ascii_lowercase();
    if let Some(theme) = builtin(&requested) {
        return ThemeResolution {
            requested_id: requested.clone(),
            resolved_id: requested.clone(),
            matched: true,
            fallback_used: false,
            theme_id: requested,
            theme,
        };
    }
    ThemeResolution {
        requested_id: requested,
        resolved_id: THEME_ID_DEFAULT.to_string(),
        matched: false,
        fallback_used: true,
        theme_id: THEME_ID_DEFAULT.to_string(),
        theme: builtin(THEME_ID_DEFAULT).expect("the default theme is built in"),
    }
}

pub fn theme_data_id(theme: &ThemeData) -> &'static str {
    [THEME_ID_DARK, THEME_ID_LIGHT, THEME_ID_HIGH_CONTRAST]
        .into_iter()
        .find(|id| builtin(id).as_ref() == Some(theme))
        .unwrap_or(THEME_ID_DEFAULT)
}

#[derive(Deserialize)]
#[serde(untagged)]
enum PersistedThemeValue {
    Theme(ThemeData),
    ThemeId(String),
}

pub fn deserialize_optional_theme_data<'de, D>(
    deserializer: D,
) -> Result<Option<ThemeData>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let persisted = Option::<PersistedThemeValue>::deserialize(deserializer)?;
    Ok(persisted.map(|value| match value {
        PersistedThemeValue::Theme(theme) => theme,
        PersistedThemeValue::ThemeId(theme_id) => resolve_theme_data(&theme_id).theme,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token_set_theme(id: &str) -> ThemeData {
        crate::theme::seed::builtin_token_sets()
            .into_iter()
            .find(|tokens| tokens.theme_id == id)
            .expect("built-in token set")
            .theme_data
    }

    #[test]
    fn theme_lookup_falls_back_for_unknown_id() {
        let resolution = resolve_theme_data("theme:unknown");

        assert!(!resolution.matched);
        assert!(resolution.fallback_used);
        assert_eq!(resolution.resolved_id, THEME_ID_DEFAULT);
        assert_eq!(resolution.theme_id, THEME_ID_DEFAULT);
        assert_eq!(resolution.theme, token_set_theme(THEME_ID_DEFAULT));
    }

    #[test]
    fn built_in_ids_resolve_to_their_token_sets() {
        for id in [
            THEME_ID_DEFAULT,
            THEME_ID_DARK,
            THEME_ID_LIGHT,
            THEME_ID_HIGH_CONTRAST,
        ] {
            let resolution = resolve_theme_data(id);
            assert!(resolution.matched, "{id}");
            assert_eq!(resolution.theme, token_set_theme(id), "{id}");
            assert_eq!(theme_data_id(&resolution.theme), id, "{id} round-trips");
        }
        let light = resolve_theme_data(THEME_ID_LIGHT).theme;
        assert!(
            light.background_rgb.0 > 200,
            "a light background: {light:?}"
        );
    }

    #[test]
    fn saved_lens_themes_read_by_id_and_by_value() {
        #[derive(Deserialize)]
        struct SavedLens {
            #[serde(default, deserialize_with = "deserialize_optional_theme_data")]
            theme: Option<ThemeData>,
        }
        let by_id: SavedLens = serde_json::from_str(r#"{"theme":"theme:dark"}"#).unwrap();
        assert_eq!(by_id.theme, Some(token_set_theme(THEME_ID_DARK)));
        let by_value: SavedLens = serde_json::from_str(
            r#"{"theme":{"background_rgb":[1,2,3],"accent_rgb":[4,5,6],"font_scale":1.0,"stroke_width":2.0}}"#,
        )
        .unwrap();
        let theme = by_value.theme.expect("a stored value");
        assert_eq!(
            (theme.background_rgb, theme.accent_rgb),
            ((1, 2, 3), (4, 5, 6))
        );
        let absent: SavedLens = serde_json::from_str("{}").unwrap();
        assert_eq!(absent.theme, None);
    }
}
