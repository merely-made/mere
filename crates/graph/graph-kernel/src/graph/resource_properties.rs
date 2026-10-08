// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shared literal assertions with exact handles and per-asserter deduplication.

use super::resource_content::{ContentError, RESOURCE_PROPERTIES};
use super::{Graph, NodeKey};
use crate::types::NodeProperty;
use uuid::Uuid;

/// Merge copied assertions by their exact handles, preserving distinct carried IDs.
pub fn merge_resource_properties(
    left: &[NodeProperty],
    right: &[NodeProperty],
) -> Result<Vec<NodeProperty>, ContentError> {
    let mut merged = left.to_vec();
    for property in right {
        if let Some(existing) = merged
            .iter()
            .find(|held| held.statement_id == property.statement_id)
        {
            if existing != property {
                return Err(ContentError::HandleCollision(property.statement_id.clone()));
            }
        } else {
            merged.push(property.clone());
        }
    }
    merged.sort_by(|a, b| a.statement_id.cmp(&b.statement_id));
    Ok(merged)
}

impl Graph {
    pub fn resource_properties(&self, id: Uuid) -> Vec<NodeProperty> {
        self.resource_content(id, RESOURCE_PROPERTIES)
            .unwrap_or_default()
    }

    pub(crate) fn literal_handle_available(&self, resource: Uuid, property: &NodeProperty) -> bool {
        if self
            .find_semantic_statement(&property.statement_id)
            .is_some()
        {
            return false;
        }
        for node in self.resource_nodes() {
            for held in self.resource_properties(node.id()) {
                if held.statement_id == property.statement_id
                    && (node.id() != resource || !held.content_eq(property))
                {
                    return false;
                }
            }
        }
        for (key, _) in self.nodes() {
            if self
                .legacy_node_properties(key)
                .into_iter()
                .flatten()
                .any(|held| held.statement_id == property.statement_id)
            {
                return false;
            }
        }
        true
    }

    pub fn append_resource_properties(
        &mut self,
        id: Uuid,
        incoming: Vec<NodeProperty>,
    ) -> Result<bool, ContentError> {
        if self.resource(id).is_none() {
            return Err(ContentError::MissingResource);
        }
        let mut properties = self.resource_properties(id);
        for mut property in incoming {
            if property.provenance_iri.is_none() {
                property.provenance_iri = Some(self.write_author().asserter_iri());
            }
            if !self.literal_handle_available(id, &property) {
                return Err(ContentError::HandleCollision(property.statement_id));
            }
            if let Some(existing) = properties
                .iter_mut()
                .find(|held| held.content_eq(&property))
            {
                existing.asserted_at_ms = property.asserted_at_ms;
            } else {
                if properties
                    .iter()
                    .any(|held| held.statement_id == property.statement_id)
                {
                    return Err(ContentError::HandleCollision(property.statement_id));
                }
                properties.push(property);
            }
        }
        self.write_resource_content(id, RESOURCE_PROPERTIES, &properties)
    }

    pub(crate) fn append_shown_properties(
        &mut self,
        key: NodeKey,
        incoming: Vec<NodeProperty>,
    ) -> bool {
        if incoming.is_empty() {
            return false;
        }
        let Some(node) = self.get_node(key) else {
            return false;
        };
        let id = self
            .shown_resource_id(key)
            .unwrap_or_else(|| super::ResourceNode::new(node.primary_address().as_url_str()).id());
        let mut incoming = incoming;
        for property in &mut incoming {
            if property.provenance_iri.is_none() {
                property.provenance_iri = Some(self.write_author().asserter_iri());
            }
            if !self.literal_handle_available(id, property) {
                return false;
            }
        }
        if merge_resource_properties(&[], &incoming).is_err() {
            return false;
        }
        let Some(id) = self.ensure_surface_resource(key) else {
            return false;
        };
        self.append_resource_properties(id, incoming)
            .unwrap_or(false)
    }

    pub fn retract_resource_property(
        &mut self,
        id: Uuid,
        statement_id: &str,
    ) -> Result<bool, ContentError> {
        let mut properties = self.resource_properties(id);
        let before = properties.len();
        properties.retain(|property| property.statement_id != statement_id);
        if before == properties.len() {
            return Ok(false);
        }
        self.write_resource_content(id, RESOURCE_PROPERTIES, &properties)
    }
}
