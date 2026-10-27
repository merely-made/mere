// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Typed, recorded content on shared resources. Unknown facets remain untouched.

use super::{CapturedDelta, Graph, NodeKey, ResourceNode};
use crate::persistence::{PersistedResourceFacet, PersistedResourceRecord};
use crate::types::{ClassificationScheme, ClassificationStatus, NodeClassification, NodeProperty};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

pub const RESOURCE_PROPERTIES: &str = "semantic.properties";
pub const RESOURCE_CLASSIFICATIONS: &str = "semantic.classification-variants/v1";
pub const TAG_CONCEPT: &str = "semantic.tag-concept/v1";

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

pub(crate) fn validate_content_record(
    record: &PersistedResourceRecord,
) -> Result<(), ContentError> {
    for facet in &record.facets {
        match facet.facet.as_str() {
            super::frozen_selection::FROZEN_SELECTION => {
                super::frozen_selection::selection_from_record(record)
                    .map_err(ContentError::InvalidFacet)?;
            },
            super::frozen_link::FROZEN_LINK => {
                super::frozen_link::validate_frozen_link_record(record)
                    .map_err(ContentError::InvalidFacet)?;
            },
            RESOURCE_PROPERTIES => {
                let properties: Vec<crate::types::NodeProperty> =
                    serde_json::from_str(&facet.value_json)
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
                let variants: Vec<super::resource_classifications::ClassificationVariant> =
                    serde_json::from_str(&facet.value_json)
                        .map_err(|e| ContentError::InvalidFacet(e.to_string()))?;
                super::resource_classifications::validate_variants(&variants)?;
            },
            TAG_CONCEPT => {
                let concept: super::resource_tags::TagConcept =
                    serde_json::from_str(&facet.value_json)
                        .map_err(|e| ContentError::InvalidFacet(e.to_string()))?;
                super::resource_tags::validate_tag_concept(&record.canonical_iri, &concept)?;
            },
            _ => {},
        }
    }
    Ok(())
}

#[derive(Clone, PartialEq, Eq)]
enum ResourceHandlePayload {
    Semantic(super::SemanticStatement),
    Literal(crate::types::NodeProperty),
}
#[derive(Clone, PartialEq, Eq)]
enum ResourceHandleOwner {
    Relation(Uuid, Uuid),
    Literal(Uuid),
}

/// Literals about to replace what one owner holds, checked before they are stored.
#[derive(Clone, Copy)]
pub(crate) enum LiteralWrite<'a> {
    Resource(Uuid, &'a [NodeProperty]),
    /// Retained Surface literals, by Surface node id.
    Surface(Uuid, &'a [NodeProperty]),
}

impl Graph {
    pub(crate) fn insert_node_tag(&mut self, key: NodeKey, tag: String) -> bool {
        self.insert_shown_tag(key, tag)
    }
    pub(crate) fn remove_node_tag(&mut self, key: NodeKey, tag: &str) -> bool {
        self.remove_shown_tag(key, tag)
    }
    pub(crate) fn append_node_property(&mut self, key: NodeKey, property: NodeProperty) -> bool {
        self.append_shown_properties(key, vec![property])
    }
    pub(crate) fn add_node_classification(
        &mut self,
        key: NodeKey,
        classification: NodeClassification,
    ) -> bool {
        self.add_shown_classifications(key, vec![classification])
    }
    pub(crate) fn remove_node_classification(
        &mut self,
        key: NodeKey,
        scheme: &ClassificationScheme,
        value: &str,
    ) -> bool {
        self.unique_shown_classification(key, scheme, value)
            .ok()
            .flatten()
            .is_some_and(|(id, variant)| {
                self.remove_resource_classification(id, &variant)
                    .unwrap_or(false)
            })
    }
    pub(crate) fn set_node_classification_status(
        &mut self,
        key: NodeKey,
        scheme: &ClassificationScheme,
        value: &str,
        status: ClassificationStatus,
    ) -> bool {
        self.unique_shown_classification(key, scheme, value)
            .ok()
            .flatten()
            .is_some_and(|(id, variant)| {
                self.edit_resource_classification(id, &variant, Some(status), None)
                    .unwrap_or(false)
            })
    }
    pub(crate) fn set_node_primary_classification(
        &mut self,
        key: NodeKey,
        scheme: &ClassificationScheme,
        value: &str,
    ) -> bool {
        self.unique_shown_classification(key, scheme, value)
            .ok()
            .flatten()
            .is_some_and(|(id, variant)| {
                self.edit_resource_classification(id, &variant, None, Some(true))
                    .unwrap_or(false)
            })
    }

    /// Check Resource handle ownership after a host overlays retained Surface facets.
    /// Historical Surface-only duplicate grammar remains unchanged. Checked open and
    /// replayed writes share this one rule.
    pub fn validate_active_resource_assertion_handles(&self) -> Result<(), ContentError> {
        self.validate_handles_after(None)
    }

    /// Whether storing `write` leaves the graph passing checked open. A write that
    /// claims no literal its owner does not already hold cannot collide, so it skips
    /// the scan.
    pub(crate) fn admit_literals(&self, write: LiteralWrite<'_>) -> Result<(), ContentError> {
        let (held, incoming) = match write {
            LiteralWrite::Resource(id, incoming) => (self.resource_properties(id), incoming),
            LiteralWrite::Surface(id, incoming) => (
                self.get_node_key_by_id(id)
                    .and_then(|key| self.legacy_node_properties(key))
                    .unwrap_or_default(),
                incoming,
            ),
        };
        if incoming.iter().all(|property| held.contains(property)) {
            return Ok(());
        }
        self.validate_handles_after(Some(write))
    }

    pub(crate) fn admit_record_literals(
        &self,
        id: Uuid,
        record: Option<&PersistedResourceRecord>,
    ) -> Result<(), ContentError> {
        let properties: Vec<NodeProperty> = record
            .and_then(|record| {
                record
                    .facets
                    .iter()
                    .find(|facet| facet.facet == RESOURCE_PROPERTIES)
            })
            .and_then(|facet| serde_json::from_str(&facet.value_json).ok())
            .unwrap_or_default();
        self.admit_literals(LiteralWrite::Resource(id, &properties))
    }

    pub(crate) fn admit_surface_facet(
        &self,
        node_id: Uuid,
        facet: &str,
        value: &serde_json::Value,
    ) -> Result<(), ContentError> {
        if facet != super::node_facets::SEMANTIC_PROPERTIES {
            return Ok(());
        }
        let properties = Vec::<NodeProperty>::deserialize(value).unwrap_or_default();
        self.admit_literals(LiteralWrite::Surface(node_id, &properties))
    }

    fn validate_handles_after(&self, write: Option<LiteralWrite<'_>>) -> Result<(), ContentError> {
        let mut handles = std::collections::BTreeMap::<
            String,
            (ResourceHandleOwner, ResourceHandlePayload),
        >::new();
        let mut insert = |id: String,
                          owner: ResourceHandleOwner,
                          payload: ResourceHandlePayload|
         -> Result<(), ContentError> {
            if let Some((held_owner, held_payload)) = handles.get(&id) {
                if *held_owner != owner || *held_payload != payload {
                    return Err(ContentError::HandleCollision(id));
                }
            } else {
                handles.insert(id, (owner, payload));
            }
            Ok(())
        };
        for (from, to, payload) in self.resource_edges() {
            for statement in payload.assertions() {
                insert(
                    statement.statement_id.clone(),
                    ResourceHandleOwner::Relation(from.id(), to.id()),
                    ResourceHandlePayload::Semantic(statement.clone()),
                )?;
            }
        }
        let unheld = match write {
            Some(LiteralWrite::Resource(id, _)) if self.resource(id).is_none() => Some(id),
            _ => None,
        };
        for id in self.resource_nodes().map(ResourceNode::id).chain(unheld) {
            let held;
            let properties = match write {
                Some(LiteralWrite::Resource(owner, properties)) if owner == id => properties,
                _ => {
                    held = self.resource_properties(id);
                    &held
                },
            };
            for property in properties {
                insert(
                    property.statement_id.clone(),
                    ResourceHandleOwner::Literal(id),
                    ResourceHandlePayload::Literal(property.clone()),
                )?;
            }
        }
        for payload in self.inner.inner().edge_weights() {
            for statement in payload.assertions() {
                if handles.contains_key(&statement.statement_id) {
                    return Err(ContentError::HandleCollision(
                        statement.statement_id.clone(),
                    ));
                }
            }
        }
        for (key, node) in self.nodes() {
            let held;
            let properties = match write {
                Some(LiteralWrite::Surface(owner, properties)) if owner == node.id => properties,
                _ => {
                    held = self.legacy_node_properties(key).unwrap_or_default();
                    &held
                },
            };
            for property in properties {
                if handles.contains_key(&property.statement_id) {
                    return Err(ContentError::HandleCollision(property.statement_id.clone()));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn resource_content<T: DeserializeOwned>(&self, id: Uuid, facet: &str) -> Option<T> {
        let value = self
            .resource_facets()
            .get(&id, &chartulary::FacetId::new(facet))?;
        T::deserialize(value).ok()
    }

    pub(crate) fn write_resource_content<T: Serialize>(
        &mut self,
        id: Uuid,
        facet: &str,
        value: &T,
    ) -> Result<bool, ContentError> {
        let mut record = self
            .resource_record(id)
            .ok_or(ContentError::MissingResource)?;
        let encoded =
            serde_json::to_string(value).map_err(|e| ContentError::InvalidFacet(e.to_string()))?;
        if let Some(stored) = record.facets.iter_mut().find(|entry| entry.facet == facet) {
            if serde_json::from_str::<serde_json::Value>(&stored.value_json).ok()
                == serde_json::from_str(&encoded).ok()
            {
                return Ok(false);
            }
            stored.value_json = encoded;
        } else {
            record.facets.push(PersistedResourceFacet {
                facet: facet.into(),
                value_json: encoded,
            });
        }
        validate_content_record(&record)?;
        if !self.set_resource_record(id, Some(record.clone())) {
            return Err(ContentError::InvalidFacet("resource record refused".into()));
        }
        self.record_delta(&CapturedDelta::ReplaySetResourceRecordById {
            resource_id: id.to_string(),
            record: Some(record),
        });
        Ok(true)
    }
}
