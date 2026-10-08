// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Complete classification variants, retained origins and exact review edits.

use super::resource_content::{ContentError, RESOURCE_CLASSIFICATIONS};
use super::{Author, Graph, NodeKey};
use crate::types::{ClassificationScheme, ClassificationStatus, NodeClassification};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassificationOrigin {
    pub surface_id: Uuid,
    pub author: Option<Author>,
    pub uncertainty: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassificationVariant {
    pub variant_id: String,
    /// Original record identity shared by divergent review versions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_variant_id: Option<String>,
    pub classification: NodeClassification,
    pub origins: Vec<ClassificationOrigin>,
}

pub(crate) fn validate_variants(variants: &[ClassificationVariant]) -> Result<(), ContentError> {
    let mut ids = std::collections::BTreeMap::new();
    for variant in variants {
        if let Some(previous) = ids.insert(&variant.variant_id, variant)
            && previous != variant
        {
            return Err(ContentError::InvalidFacet(format!(
                "conflicting classification ID {:?}",
                variant.variant_id
            )));
        }
    }
    Ok(())
}

/// Union exact variants and origins. Differing records for one logical key remain separate.
pub fn merge_classification_variants(
    left: &[ClassificationVariant],
    right: &[ClassificationVariant],
) -> Result<Vec<ClassificationVariant>, ContentError> {
    // Two replicas may carry divergent review versions under the original ID.
    // Admission requires normalized unique IDs; composition expands the versions.
    let mut lineage = std::collections::BTreeMap::new();
    for variant in left.iter().chain(right) {
        let root = variant
            .original_variant_id
            .as_ref()
            .unwrap_or(&variant.variant_id);
        if let Some(previous) = lineage.insert(&variant.variant_id, root)
            && previous != root
        {
            return Err(ContentError::InvalidFacet(
                "conflicting classification lineage".into(),
            ));
        }
    }
    let mut groups = std::collections::BTreeMap::<
        String,
        std::collections::BTreeMap<String, ClassificationVariant>,
    >::new();
    for variant in left.iter().chain(right) {
        let original = variant
            .original_variant_id
            .as_ref()
            .unwrap_or(&variant.variant_id)
            .clone();
        let payload = serde_json::to_string(&variant.classification)
            .map_err(|e| ContentError::InvalidFacet(e.to_string()))?;
        let versions = groups.entry(original).or_default();
        if let Some(existing) = versions.get_mut(&payload) {
            for origin in &variant.origins {
                if !existing.origins.contains(origin) {
                    existing.origins.push(origin.clone());
                }
            }
            if variant.original_variant_id.is_some() {
                existing.original_variant_id = variant.original_variant_id.clone();
            }
        } else {
            versions.insert(payload, variant.clone());
        }
    }
    let mut merged = Vec::new();
    for (original, versions) in groups {
        let diverged = versions.len() > 1;
        for (payload, mut version) in versions {
            if diverged || version.original_variant_id.is_some() {
                let key =
                    serde_json::to_vec(&(&original, &payload)).expect("version key serializes");
                version.variant_id = format!(
                    "classification-version:{}",
                    Uuid::new_v5(&Uuid::NAMESPACE_URL, &key)
                );
                version.original_variant_id = Some(original.clone());
            }
            version
                .origins
                .sort_by_key(|origin| serde_json::to_string(origin).expect("origin serializes"));
            version.origins.dedup();
            merged.push(version);
        }
    }
    merged.sort_by(|a, b| a.variant_id.cmp(&b.variant_id));
    Ok(merged)
}

impl Graph {
    pub fn resource_classification_variants(&self, id: Uuid) -> Vec<ClassificationVariant> {
        self.resource_content(id, RESOURCE_CLASSIFICATIONS)
            .unwrap_or_default()
    }

    pub fn resource_classifications_conflicted(&self, id: Uuid) -> bool {
        let variants = self.resource_classification_variants(id);
        variants.iter().enumerate().any(|(index, left)| {
            variants[index + 1..].iter().any(|right| {
                left.classification.scheme == right.classification.scheme
                    && left.classification.value == right.classification.value
                    && left.classification != right.classification
            })
        })
    }

    pub(crate) fn add_shown_classifications(
        &mut self,
        key: NodeKey,
        incoming: Vec<NodeClassification>,
    ) -> bool {
        if incoming.is_empty() {
            return false;
        }
        let Some(surface_id) = self.get_node(key).map(|node| node.id) else {
            return false;
        };
        let Some(id) = self.ensure_surface_resource(key) else {
            return false;
        };
        let origin = ClassificationOrigin {
            surface_id,
            author: Some(self.write_author().clone()),
            uncertainty: None,
        };
        let mut variants = self.resource_classification_variants(id);
        for classification in incoming {
            if let Some(existing) = variants
                .iter_mut()
                .find(|entry| entry.classification == classification)
            {
                if !existing.origins.contains(&origin) {
                    existing.origins.push(origin.clone());
                }
            } else {
                variants.push(ClassificationVariant {
                    variant_id: Uuid::new_v4().to_string(),
                    original_variant_id: None,
                    classification,
                    origins: vec![origin.clone()],
                });
            }
        }
        self.write_resource_content(id, RESOURCE_CLASSIFICATIONS, &variants)
            .unwrap_or(false)
    }

    pub fn edit_resource_classification(
        &mut self,
        id: Uuid,
        variant_id: &str,
        status: Option<ClassificationStatus>,
        primary: Option<bool>,
    ) -> Result<bool, ContentError> {
        let mut variants = self.resource_classification_variants(id);
        let index = variants
            .iter()
            .position(|entry| entry.variant_id == variant_id)
            .ok_or_else(|| ContentError::MissingClassification(variant_id.into()))?;
        if let Some(status) = status {
            variants[index].classification.status = status;
        }
        if let Some(primary) = primary {
            let scheme = variants[index].classification.scheme.clone();
            if primary {
                for entry in &mut variants {
                    if entry.classification.scheme == scheme {
                        entry.classification.primary = false;
                    }
                }
            }
            variants[index].classification.primary = primary;
        }
        self.write_resource_content(id, RESOURCE_CLASSIFICATIONS, &variants)
    }

    pub fn remove_resource_classification(
        &mut self,
        id: Uuid,
        variant_id: &str,
    ) -> Result<bool, ContentError> {
        let mut variants = self.resource_classification_variants(id);
        let before = variants.len();
        variants.retain(|entry| entry.variant_id != variant_id);
        if before == variants.len() {
            return Ok(false);
        }
        self.write_resource_content(id, RESOURCE_CLASSIFICATIONS, &variants)
    }

    pub(crate) fn unique_shown_classification(
        &self,
        key: NodeKey,
        scheme: &ClassificationScheme,
        value: &str,
    ) -> Result<Option<(Uuid, String)>, ContentError> {
        let Some(id) = self.shown_resource_id(key) else {
            return Ok(None);
        };
        let matches: Vec<_> = self
            .resource_classification_variants(id)
            .into_iter()
            .filter(|entry| {
                entry.classification.scheme == *scheme && entry.classification.value == value
            })
            .collect();
        match matches.len() {
            0 => Ok(None),
            1 => Ok(Some((id, matches[0].variant_id.clone()))),
            _ => Err(ContentError::AmbiguousClassification(
                matches.into_iter().map(|entry| entry.variant_id).collect(),
            )),
        }
    }
}
