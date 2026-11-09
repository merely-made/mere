// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A chosen theme as stored values: an id, a mode key and per-role font
//! stacks. Pandect keeps them without reading them; the host converts to its
//! theming type (Tabard's `ThemeChoice`) at its edge (engine split D60, D79).
//!
//! The serialized form is byte-identical to `ThemeChoice`'s: `theme_id`, then
//! `theme_mode` (a key, or null), then `fonts` (a role-to-families map, in the
//! order it was written, left out when every stack is empty). An unknown mode
//! key is kept as written; the host decides what it means.

use std::fmt;

use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A chosen theme, as stored.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredTheme {
    /// The theme's id.
    pub theme_id: String,
    /// The mode's key (`"dark"`, `"hc_light"`, `"custom:…"`); `None` is the
    /// theme's own default mode.
    #[serde(default)]
    pub theme_mode: Option<String>,
    /// Requested font families per role.
    #[serde(default, skip_serializing_if = "FontStacks::is_empty")]
    pub fonts: FontStacks,
}

impl StoredTheme {
    pub fn new(theme_id: impl Into<String>, theme_mode: Option<String>) -> Self {
        Self {
            theme_id: theme_id.into(),
            theme_mode,
            fonts: FontStacks::default(),
        }
    }
}

/// Font family stacks by role name, kept in the order they were written.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FontStacks(pub Vec<(String, Vec<String>)>);

impl FontStacks {
    /// True when no role requests a family; such stacks are not written.
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|(_, stack)| stack.is_empty())
    }
}

impl Serialize for FontStacks {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (role, stack) in &self.0 {
            map.serialize_entry(role, stack)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for FontStacks {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Stacks;
        impl<'de> Visitor<'de> for Stacks {
            type Value = FontStacks;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a map from font role to family names")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut access: M) -> Result<FontStacks, M::Error> {
                let mut stacks = Vec::new();
                while let Some(entry) = access.next_entry::<String, Vec<String>>()? {
                    stacks.push(entry);
                }
                Ok(FontStacks(stacks))
            }
        }
        deserializer.deserialize_map(Stacks)
    }
}
