// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Portable family requests, separate from host-local availability and metrics.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Semantic roles. Hosts decide which surfaces use a role; explicit writer
/// preferences, point sizes, line heights and measures remain host-owned.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FontRole {
    Interface,
    Reading,
    Heading,
    Label,
    Code,
}
impl FontRole {
    pub const ALL: [Self; 5] = [
        Self::Interface,
        Self::Reading,
        Self::Heading,
        Self::Label,
        Self::Code,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Self::Interface => "interface",
            Self::Reading => "reading",
            Self::Heading => "heading",
            Self::Label => "label",
            Self::Code => "code",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Interface => "Interface",
            Self::Reading => "Reading",
            Self::Heading => "Headings",
            Self::Label => "Labels",
            Self::Code => "Code",
        }
    }
}

/// One family name or CSS generic. Constructed and deserialized through the
/// same validation; CSS quoting is applied only on output, never to saved intent.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct FontFamily(String);
impl FontFamily {
    pub fn new(name: impl Into<String>) -> Result<Self, String> {
        let name = name.into();
        if name.trim().is_empty() || name.len() > 256 || name.chars().any(char::is_control) {
            return Err("Font family needs a nonempty name of at most 256 bytes without control characters.".into());
        }
        Ok(Self(name))
    }
    pub fn name(&self) -> &str {
        &self.0
    }
    pub fn is_generic(&self) -> bool {
        matches!(
            self.0.as_str(),
            "serif"
                | "sans-serif"
                | "monospace"
                | "system-ui"
                | "cursive"
                | "fantasy"
                | "emoji"
                | "math"
                | "fangsong"
        )
    }
    pub fn css(&self) -> String {
        if self.is_generic() {
            self.0.clone()
        } else {
            format!("\"{}\"", self.0.replace('\\', "\\\\").replace('"', "\\\""))
        }
    }
}
impl TryFrom<String> for FontFamily {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
impl From<FontFamily> for String {
    fn from(value: FontFamily) -> Self {
        value.0
    }
}

/// Ordered family requests. An empty map inherits the host/theme defaults.
/// Empty role stacks also inherit, permitting older preference writers.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FontPreferences(pub BTreeMap<FontRole, Vec<FontFamily>>);
impl FontPreferences {
    pub fn is_empty(&self) -> bool {
        self.0.values().all(Vec::is_empty)
    }
    /// Does not mutate requests. Generics delegate realization to the engine;
    /// named-family availability does not prove glyph coverage or face identity.
    pub fn resolve(
        &self,
        available: &[String],
        defaults: &BTreeMap<FontRole, FontFamily>,
    ) -> Vec<ResolvedFontRole> {
        self.0
            .iter()
            .filter(|(_, stack)| !stack.is_empty())
            .map(|(&role, requested)| {
                let chosen = requested.iter().find(|family| {
                    family.is_generic()
                        || available
                            .iter()
                            .any(|name| name.eq_ignore_ascii_case(family.name()))
                });
                ResolvedFontRole {
                    role,
                    requested: requested.clone(),
                    family: chosen.cloned().unwrap_or_else(|| defaults[&role].clone()),
                    used_default: chosen.is_none(),
                }
            })
            .collect()
    }
    /// Emits only requested roles, after availability resolution. Ordinary
    /// cascade specificity still applies; explicit authored font-family wins
    /// over a host declaration that consumes these custom properties.
    pub fn css_custom_properties(
        &self,
        available: &[String],
        defaults: &BTreeMap<FontRole, FontFamily>,
    ) -> String {
        let roles = self.resolve(available, defaults);
        if roles.is_empty() {
            return String::new();
        }
        let mut css = String::from(":root {\n");
        for role in roles {
            let mut stack: Vec<_> = role
                .requested
                .iter()
                .filter(|family| {
                    family.is_generic()
                        || available
                            .iter()
                            .any(|name| name.eq_ignore_ascii_case(family.name()))
                })
                .cloned()
                .collect();
            if !stack.iter().any(FontFamily::is_generic) {
                stack.push(defaults[&role.role].clone());
            }
            css.push_str(&format!(
                "  --tabard-font-{}: {};\n",
                role.role.key(),
                stack
                    .iter()
                    .map(FontFamily::css)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        css.push_str("}\n");
        css
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedFontRole {
    pub role: FontRole,
    pub requested: Vec<FontFamily>,
    pub family: FontFamily,
    pub used_default: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn defaults() -> BTreeMap<FontRole, FontFamily> {
        FontRole::ALL
            .into_iter()
            .map(|role| (role, FontFamily::new("sans-serif").unwrap()))
            .collect()
    }
    #[test]
    fn requested_order_and_missing_names_survive_resolution_and_serialization() {
        let prefs: FontPreferences =
            serde_json::from_str(r#"{"reading":["Missing face","Source Serif 4","serif"]}"#)
                .unwrap();
        let receipt = prefs.resolve(&["Source Serif 4".into()], &defaults());
        assert_eq!(receipt[0].family.name(), "Source Serif 4");
        assert!(!receipt[0].used_default);
        let elsewhere = prefs.resolve(&[], &defaults());
        assert_eq!(elsewhere[0].family.name(), "serif");
        assert_eq!(receipt[0].requested, elsewhere[0].requested);
        assert_eq!(
            serde_json::to_string(&prefs).unwrap(),
            r#"{"reading":["Missing face","Source Serif 4","serif"]}"#
        );
    }
    #[test]
    fn unavailable_request_uses_host_default_without_rewriting_intent() {
        let prefs: FontPreferences = serde_json::from_str(r#"{"label":["Absent"]}"#).unwrap();
        let receipt = prefs.resolve(&[], &defaults());
        assert!(receipt[0].used_default);
        assert_eq!(receipt[0].requested[0].name(), "Absent");
        assert_eq!(receipt[0].family.name(), "sans-serif");
        assert!(
            prefs
                .css_custom_properties(&[], &defaults())
                .contains("--tabard-font-label: sans-serif;")
        );
        assert!(
            FontPreferences::default()
                .css_custom_properties(&[], &defaults())
                .is_empty()
        );
    }
    #[test]
    fn names_cannot_escape_a_css_string_and_invalid_json_is_refused() {
        let family = FontFamily::new("Face\"; } :root { color:red; /*\\").unwrap();
        assert_eq!(family.css(), "\"Face\\\"; } :root { color:red; /*\\\\\"");
        for json in [
            r#"{"reading":[""]}"#,
            r#"{"reading":["bad\nname"]}"#,
            r#"{"unknown":["serif"]}"#,
        ] {
            assert!(serde_json::from_str::<FontPreferences>(json).is_err());
        }
    }
}
