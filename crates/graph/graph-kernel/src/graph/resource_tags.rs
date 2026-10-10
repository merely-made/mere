// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Identity-scoped tag concepts and separately attributable tagging assertions.

use super::resource::TAGGED_WITH_IRI;
use super::resource_content::{ContentError, TAG_CONCEPT};
use super::{CapturedDelta, EdgePayload, Graph, NodeKey, ResourceNode, SemanticStatement};
use crate::persistence::PersistedResourceRecord;
use crate::types::GraphScope;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

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

pub(crate) fn validate_tag_concept(iri: &str, concept: &TagConcept) -> Result<(), ContentError> {
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

impl Graph {
    pub fn resource_tag_concept(&self, id: Uuid) -> Option<TagConcept> {
        self.resource_content(id, TAG_CONCEPT)
    }

    pub fn resource_tag_labels(&self, id: Uuid) -> HashSet<String> {
        self.resource_edges()
            .filter(|(from, _, payload)| {
                from.id() == id
                    && payload
                        .semantic_statements()
                        .iter()
                        .any(|statement| statement.predicate == TAGGED_WITH_IRI)
            })
            .filter_map(|(_, to, _)| {
                self.resource_tag_concept(to.id())
                    .map(|concept| concept.label)
            })
            .collect()
    }

    /// Content tags of the shown resource; raw Surface tags remain legacy fallback.
    pub fn node_content_tags(&self, key: NodeKey) -> Option<HashSet<String>> {
        let node = self.get_node(key)?;
        Some(
            self.shown_resource_id(key)
                .map(|id| self.resource_tag_labels(id))
                .unwrap_or_else(|| node.tags.clone()),
        )
    }

    /// Preserve an explicit SKOS concept's prepared IRI and declared owner.
    pub fn define_tag_concept(
        &mut self,
        id: Uuid,
        concept: TagConcept,
    ) -> Result<bool, ContentError> {
        let resource = self.resource(id).ok_or(ContentError::MissingResource)?;
        validate_tag_concept(resource.canonical_iri(), &concept)?;
        if let Some(existing) = self.resource_tag_concept(id)
            && existing != concept
        {
            return Err(ContentError::InvalidFacet(
                "concept identity has different label or owner".into(),
            ));
        }
        self.write_resource_content(id, TAG_CONCEPT, &concept)
    }

    /// Reuse an explicitly named concept, including one in another owner's vocabulary.
    pub fn tag_resource_with_concept(
        &mut self,
        resource_id: Uuid,
        concept_iri: &str,
        concept: TagConcept,
    ) -> Result<bool, ContentError> {
        let statement = SemanticStatement::new(
            TAGGED_WITH_IRI.into(),
            None,
            None,
            GraphScope::Default,
            Some(self.write_author().asserter_iri()),
            Some(Self::epoch_ms()),
        );
        self.tag_resource_with_statement(resource_id, concept_iri, concept, statement)
    }

    /// Tag with an explicit imported source rather than the host's Author fallback.
    pub fn tag_resource_with_asserter(
        &mut self,
        resource_id: Uuid,
        concept_iri: &str,
        concept: TagConcept,
        asserter_iri: String,
    ) -> Result<bool, ContentError> {
        let statement = SemanticStatement::new(
            TAGGED_WITH_IRI.into(),
            None,
            None,
            GraphScope::Default,
            Some(asserter_iri),
            None,
        );
        self.tag_resource_with_statement(resource_id, concept_iri, concept, statement)
    }

    pub(crate) fn tag_resource_with_statement(
        &mut self,
        resource_id: Uuid,
        concept_iri: &str,
        concept: TagConcept,
        statement: SemanticStatement,
    ) -> Result<bool, ContentError> {
        validate_tag_concept(concept_iri, &concept)?;
        if self.resource(resource_id).is_none() {
            return Err(ContentError::MissingResource);
        }
        if self.literal_statement_exists(&statement.statement_id) {
            return Err(ContentError::HandleCollision(statement.statement_id));
        }
        let target = ResourceNode::for_term(concept_iri);
        if let Some(existing) = self.resource_tag_concept(target.id())
            && existing != concept
        {
            return Err(ContentError::InvalidFacet(
                "concept identity has different label or owner".into(),
            ));
        }
        if let Some((key, existing)) = self.find_semantic_statement(&statement.statement_id) {
            let same_pair = matches!(key, super::RelationKey::Resource(_))
                && self.resource_edges().any(|(from, to, payload)| {
                    from.id() == resource_id
                        && to.id() == target.id()
                        && payload
                            .semantic_statements()
                            .iter()
                            .any(|held| held.statement_id == statement.statement_id)
                });
            if !same_pair || existing != &statement {
                return Err(ContentError::HandleCollision(statement.statement_id));
            }
        }
        let mut payloads = self.persisted_resource_edges_between(resource_id, target.id());
        if payloads
            .iter()
            .filter_map(|edge| edge.semantic.as_ref())
            .flat_map(|semantic| &semantic.statements)
            .any(|held| {
                held.predicate == TAGGED_WITH_IRI
                    && held.graph_scope == statement.graph_scope
                    && held.provenance_iri == statement.provenance_iri
            })
        {
            return Ok(false);
        }
        if self.resource(target.id()).is_none() {
            let record = PersistedResourceRecord {
                canonical_iri: concept_iri.into(),
                facets: vec![],
            };
            if !self.set_resource_record(target.id(), Some(record.clone())) {
                return Err(ContentError::InvalidFacet(
                    "concept insertion refused".into(),
                ));
            }
            self.record_delta(&CapturedDelta::ReplaySetResourceRecordById {
                resource_id: target.id().to_string(),
                record: Some(record),
            });
        }
        self.write_resource_content(target.id(), TAG_CONCEPT, &concept)?;
        let mut payload = EdgePayload::new();
        payload.push_persisted_semantic_statement(statement);
        payloads.push(super::snapshot::persisted_edge_for_ids(
            resource_id,
            target.id(),
            &payload,
        ));
        if !self.set_resource_edges_between(resource_id, target.id(), &payloads) {
            return Err(ContentError::InvalidFacet(
                "tag assertion insertion refused".into(),
            ));
        }
        self.capture_resource_pair(resource_id, target.id());
        Ok(true)
    }

    pub(crate) fn insert_shown_tag(&mut self, key: NodeKey, tag: String) -> bool {
        let Some(resource_id) = self.ensure_surface_resource(key) else {
            return false;
        };
        let owner_iri = self.write_author().asserter_iri();
        let concept_iri = tag_concept_iri(&owner_iri, &tag);
        self.tag_resource_with_concept(
            resource_id,
            &concept_iri,
            TagConcept {
                owner_iri,
                label: tag,
            },
        )
        .unwrap_or(false)
    }

    pub(crate) fn remove_shown_tag(&mut self, key: NodeKey, tag: &str) -> bool {
        let Some(id) = self.shown_resource_id(key) else {
            return false;
        };
        let source = self.write_author().asserter_iri();
        let handles: Vec<String> = self
            .resource_edges()
            .filter(|(from, to, _)| {
                from.id() == id
                    && self
                        .resource_tag_concept(to.id())
                        .is_some_and(|concept| concept.label == tag)
            })
            .flat_map(|(_, _, payload)| {
                payload
                    .semantic_statements()
                    .iter()
                    .filter(|statement| {
                        statement.predicate == TAGGED_WITH_IRI
                            && statement.provenance_iri.as_deref() == Some(source.as_str())
                    })
                    .map(|statement| statement.statement_id.clone())
            })
            .collect();
        let mut changed = false;
        for handle in handles {
            changed |= self.retract_assertion(&handle);
        }
        if changed
            && !self.resource_tag_labels(id).contains(tag)
            && let Some(mut presentation) = self
                .node_facet::<crate::types::NodeTagPresentationState>(
                    key,
                    super::node_facets::PRESENTATION_TAGS,
                )
        {
            presentation.ordered_tags.retain(|entry| entry != tag);
            presentation.icon_overrides.remove(tag);
            if self.set_node_facet(key, super::node_facets::PRESENTATION_TAGS, &presentation) {
                self.record_delta(&CapturedDelta::ReplaySetNodeFacetById {
                    node_id: self
                        .get_node(key)
                        .expect("shown Surface exists")
                        .id
                        .to_string(),
                    facet: super::node_facets::PRESENTATION_TAGS.into(),
                    value_json: serde_json::to_string(&presentation)
                        .expect("tag presentation serializes"),
                });
            }
        }
        changed
    }
}
