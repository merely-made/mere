// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Portable font preferences. Hosts supply installed-family discovery and
//! semantic selectors; their existing text engine still resolves and measures
//! faces. An absent preference leaves the host's existing pattern untouched.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FontRole {
    Ui,
    Reading,
    Heading,
    Label,
    Code,
}

impl FontRole {
    pub const ALL: [Self; 5] = [
        Self::Ui,
        Self::Reading,
        Self::Heading,
        Self::Label,
        Self::Code,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Self::Ui => "ui",
            Self::Reading => "reading",
            Self::Heading => "heading",
            Self::Label => "label",
            Self::Code => "code",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Ui => "Interface",
            Self::Reading => "Reading",
            Self::Heading => "Headings",
            Self::Label => "Labels",
            Self::Code => "Code",
        }
    }
    pub fn fallback(self) -> &'static str {
        match self {
            Self::Reading => "serif",
            Self::Code => "monospace",
            _ => "sans-serif",
        }
    }
}

/// A requested family retains its spelling even on a host where it is absent.
/// The generic fallback is explicit in emitted CSS. Font bytes are not embedded
/// or fetched by this model. Sizes are CSS pixels; line height is unitless.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FontPreferences {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_px: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_height: Option<f32>,
}

impl FontPreferences {
    pub fn is_empty(&self) -> bool {
        self.family.is_none() && self.size_px.is_none() && self.line_height.is_none()
    }
    pub fn validate(&self) -> Result<(), String> {
        if let Some(family) = &self.family {
            if family.trim().is_empty()
                || family.len() > 256
                || family.chars().any(char::is_control)
            {
                return Err(
                    "A font family must contain 1 to 256 bytes without control characters.".into(),
                );
            }
        }
        if self
            .size_px
            .is_some_and(|v| !v.is_finite() || !(6.0..=128.0).contains(&v))
        {
            return Err("Font size must be between 6 and 128 CSS pixels.".into());
        }
        if self
            .line_height
            .is_some_and(|v| !v.is_finite() || !(0.8..=3.0).contains(&v))
        {
            return Err("Line height must be between 0.8 and 3.".into());
        }
        Ok(())
    }
    pub fn css_family(&self, role: FontRole) -> Option<String> {
        self.family.as_ref().map(|family| {
            let family = family.trim();
            if is_generic_family(family) {
                family.to_ascii_lowercase()
            } else {
                format!(
                    "\"{}\", {}",
                    family.replace('\\', "\\\\").replace('"', "\\\""),
                    role.fallback()
                )
            }
        })
    }
}

pub fn is_generic_family(family: &str) -> bool {
    matches!(
        family.to_ascii_lowercase().as_str(),
        "system-ui" | "sans-serif" | "serif" | "monospace"
    )
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Typography(pub BTreeMap<FontRole, FontPreferences>);

impl Typography {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn validate(&self) -> Result<(), String> {
        for (role, preferences) in &self.0 {
            preferences
                .validate()
                .map_err(|error| format!("{}: {error}", role.label()))?;
        }
        Ok(())
    }
    pub fn set(&mut self, role: FontRole, preferences: FontPreferences) -> Result<(), String> {
        preferences.validate()?;
        if preferences.is_empty() {
            self.0.remove(&role);
        } else {
            self.0.insert(role, preferences);
        }
        Ok(())
    }
    pub fn css_custom_properties(&self) -> String {
        if self.0.values().all(FontPreferences::is_empty) {
            return String::new();
        }
        let mut css = String::from(":root {\n");
        for (role, preferences) in &self.0 {
            if let Some(family) = preferences.css_family(*role) {
                css.push_str(&format!(
                    "  --tabard-font-{}-family: {family};\n",
                    role.key()
                ));
            }
            if let Some(size) = preferences.size_px {
                css.push_str(&format!("  --tabard-font-{}-size: {size}px;\n", role.key()));
            }
            if let Some(height) = preferences.line_height {
                css.push_str(&format!(
                    "  --tabard-font-{}-line-height: {height};\n",
                    role.key()
                ));
            }
        }
        css.push_str("}\n");
        css
    }
    /// Only authored properties are projected. Missing size/family/leading
    /// preserves the consumer's existing hierarchy and control geometry.
    pub fn role_css(&self, role: FontRole, selector: &str) -> String {
        let Some(p) = self.0.get(&role) else {
            return String::new();
        };
        let mut declarations = String::new();
        for (property, suffix, present) in [
            ("font-family", "family", p.family.is_some()),
            ("font-size", "size", p.size_px.is_some()),
            ("line-height", "line-height", p.line_height.is_some()),
        ] {
            if present {
                declarations.push_str(&format!(
                    "{property}:var(--tabard-font-{}-{suffix});",
                    role.key()
                ));
            }
        }
        if declarations.is_empty() {
            String::new()
        } else {
            format!("{selector}{{{declarations}}}\n")
        }
    }
    /// Read-only diagnostics retain the authored family rather than rewriting
    /// a theme when a portable definition travels to a different machine.
    pub fn missing_families(&self, installed: &[String]) -> Vec<String> {
        self.0
            .iter()
            .filter_map(|(role, p)| {
                let name = p.family.as_ref()?.trim();
                (!is_generic_family(name)
                    && !installed.iter().any(|f| f.eq_ignore_ascii_case(name)))
                .then(|| {
                    format!(
                        "{}: {name} is not installed on this device; {} is the fallback.",
                        role.label(),
                        role.fallback()
                    )
                })
            })
            .collect()
    }
}
