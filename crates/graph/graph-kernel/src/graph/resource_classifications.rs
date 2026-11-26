// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Complete classification variants, retained origins and exact review edits.

use super::resource_content::{ContentError, RESOURCE_CLASSIFICATIONS};
use super::{Graph, NodeKey};
use crate::types::{ClassificationScheme, ClassificationStatus, NodeClassification};
pub use mere::model::resource_content::{ClassificationOrigin, ClassificationVariant};
pub use mere::model::resource_rules::merge_classification_variants;
use uuid::Uuid;

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
