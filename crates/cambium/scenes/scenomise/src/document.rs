// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! A bounded portable document binds source occurrences to supplied readings.
//! Each reading uses the existing scene contract; no domain resolver is needed.
use crate::backdrops::{MAX_SCENE_INPUT_BYTES, SceneInput, strict_core_records};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const SCENE_DOCUMENT_FORMAT: &str = "mere.scene-document";
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkedScene {
    pub occurrence: String,
    pub label: String,
    pub scene: SceneInput,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SceneDocument {
    pub format: String,
    pub version: u16,
    pub primary: SceneInput,
    pub linked: Vec<LinkedScene>,
}
impl SceneDocument {
    pub fn new(primary: SceneInput, linked: Vec<LinkedScene>) -> Self {
        Self {
            format: SCENE_DOCUMENT_FORMAT.into(),
            version: 1,
            primary,
            linked,
        }
    }
    /// Bare recipes remain supported and acquire no inferred linked reading.
    pub fn parse(json: &str) -> Result<Self, String> {
        if json.len() > MAX_SCENE_INPUT_BYTES {
            return Err("Document exceeds one MiB".into());
        }
        let value: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let document = if value.get("format").is_some() {
            strict_core_records(&value, "document").map_err(|e| e.to_string())?;
            serde_json::from_value(value).map_err(|e| e.to_string())?
        } else {
            Self::new(SceneInput::parse(json).map_err(|e| e.to_string())?, vec![])
        };
        document.validate()?;
        Ok(document)
    }
    /// Admit every linked reading before a host replaces any active surface.
    pub fn validate(&self) -> Result<(), String> {
        if self.format != SCENE_DOCUMENT_FORMAT || self.version != 1 || self.linked.len() > 32 {
            return Err("Unsupported scene document or linked-reading budget".into());
        }
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > MAX_SCENE_INPUT_BYTES {
            return Err("Document exceeds one MiB".into());
        }
        let primary = self.primary.compile().map_err(|e| e.to_string())?;
        let mut used = HashSet::new();
        for view in &self.linked {
            if view.label.trim().is_empty()
                || !used.insert(&view.occurrence)
                || !(primary
                    .marks
                    .iter()
                    .any(|m| m.selectable && m.key == view.occurrence)
                    || primary.unplaced.iter().any(|m| m.key == view.occurrence))
            {
                return Err("Linked reading needs a unique source occurrence and label".into());
            }
            view.scene.compile().map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub fn for_occurrence(&self, key: Option<&str>) -> Option<&LinkedScene> {
        self.linked
            .iter()
            .find(|view| Some(view.occurrence.as_str()) == key)
    }
}
