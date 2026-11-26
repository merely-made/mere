// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The rules a resource's content facets must meet. Moved from the kernel
//! so the resource store checks records itself (engine split D243).
#![allow(clippy::wildcard_imports)]

use super::*;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentError {
    MissingResource,
    HandleCollision(String),
    AmbiguousClassification(Vec<String>),
    MissingClassification(String),
    InvalidFacet(String),
}
impl std::fmt::Display for ContentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ContentError {}

pub fn validate_content_record(record: &PersistedResourceRecord) -> Result<(), ContentError> {
    for facet in &record.facets {
        match facet.facet.as_str() {
            FROZEN_SELECTION => {
                super::frozen_rules::selection_from_record(record)
                    .map_err(ContentError::InvalidFacet)?;
            },
            FROZEN_LINK => {
                super::frozen_rules::validate_frozen_link_record(record)
                    .map_err(ContentError::InvalidFacet)?;
            },
            RESOURCE_PROPERTIES => {
                let properties: Vec<super::NodeProperty> = serde_json::from_str(&facet.value_json)
                    .map_err(|e| ContentError::InvalidFacet(e.to_string()))?;
                let mut handles = std::collections::BTreeMap::new();
                for property in &properties {
                    if let Some(previous) = handles.insert(&property.statement_id, property)
                        && previous != property
                    {
                        return Err(ContentError::HandleCollision(property.statement_id.clone()));
                    }
                }
            },
            RESOURCE_CLASSIFICATIONS => {
                let variants: Vec<ClassificationVariant> = serde_json::from_str(&facet.value_json)
                    .map_err(|e| ContentError::InvalidFacet(e.to_string()))?;
                validate_variants(&variants)?;
            },
            TAG_CONCEPT => {
                let concept: TagConcept = serde_json::from_str(&facet.value_json)
                    .map_err(|e| ContentError::InvalidFacet(e.to_string()))?;
                validate_tag_concept(&record.canonical_iri, &concept)?;
            },
            _ => {},
        }
    }
    Ok(())
}

pub fn validate_variants(variants: &[ClassificationVariant]) -> Result<(), ContentError> {
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

pub fn validate_tag_concept(iri: &str, concept: &TagConcept) -> Result<(), ContentError> {
    for (role, value) in [("concept", iri), ("owner", concept.owner_iri.as_str())] {
        if value.chars().any(|c| c.is_whitespace() || c.is_control())
            || url::Url::parse(value).is_err()
        {
            return Err(ContentError::InvalidFacet(format!(
                "{role} must be an absolute IRI"
            )));
        }
    }
    Ok(())
}
