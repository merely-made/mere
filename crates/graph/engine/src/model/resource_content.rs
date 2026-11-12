// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Typed content on shared resources: the facet names and the records they
//! hold. Moved from the kernel (engine split P5 L3); admission and its checks
//! stay with the kernel's recorded write path.

use super::{Author, NodeClassification};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const RESOURCE_PROPERTIES: &str = "semantic.properties";
pub const RESOURCE_CLASSIFICATIONS: &str = "semantic.classification-variants/v1";
pub const TAG_CONCEPT: &str = "semantic.tag-concept/v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagConcept {
    pub owner_iri: String,
    pub label: String,
}

/// The exact label and explicit owner form one unambiguous identity key.
pub fn tag_concept_iri(owner_iri: &str, label: &str) -> String {
    let key = serde_json::to_vec(&(owner_iri, label)).expect("string tuple serializes");
    format!(
        "https://mere.computer/ns/tag#{}",
        Uuid::new_v5(&Uuid::NAMESPACE_URL, &key)
    )
}

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
