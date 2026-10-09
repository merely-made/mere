// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Historical content placement alongside the legacy relation adapter.

use super::legacy_resource_migration::LegacyMigrationError;
use super::resource_classifications::{ClassificationOrigin, ClassificationVariant};
use super::resource_content::{RESOURCE_CLASSIFICATIONS, RESOURCE_PROPERTIES};
use super::resource_tags::{TagConcept, tag_concept_iri};
use super::{Author, CapturedDelta, Graph, ResourceNode, SemanticStatement};
use crate::persistence::PersistedResourceRecord;
use crate::types::{GraphScope, NodeProperty};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

/// Stable identity of the source mere, required only for unattributed tag concepts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyContentContext {
    pub original_mere_iri: String,
}

pub const LEGACY_CONTENT_ORIGINS: &str = "semantic.legacy-content-origins/v1";
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LegacyContentOrigin {
    pub kind: String,
    pub record_id: String,
    pub surface_id: Uuid,
    pub original_url: String,
    pub author: Option<Author>,
    pub uncertainty: Option<String>,
    pub original: serde_json::Value,
}

type TagKey = (Uuid, String);
type ClassKey = (Uuid, String, String);
#[derive(Clone)]
struct TagMember {
    resource: Uuid,
    concept: Uuid,
    handle: String,
}
#[derive(Clone)]
struct ClassMember {
    resource: Uuid,
    variant: String,
}
#[derive(Default)]
pub(crate) struct ContentReplay {
    properties: BTreeMap<String, Uuid>,
    active_properties: BTreeMap<(Uuid, String), Uuid>,
    tags: BTreeMap<TagKey, TagMember>,
    classes: BTreeMap<ClassKey, ClassMember>,
    retired: BTreeSet<(Uuid, String)>,
    retired_tag_handles: BTreeSet<String>,
}

impl ContentReplay {
    pub(crate) fn typed_record(&mut self, delta: &CapturedDelta) {
        if let CapturedDelta::ReplaySetResourceEdgesByIds {
            from_resource_id,
            to_resource_id,
            ..
        } = delta
            && let (Ok(from), Ok(to)) = (
                Uuid::parse_str(from_resource_id),
                Uuid::parse_str(to_resource_id),
            )
        {
            for member in self
                .tags
                .values()
                .filter(|member| member.resource == from && member.concept == to)
            {
                self.retired_tag_handles.insert(member.handle.clone());
            }
        }
        if let CapturedDelta::ReplaySetResourceRecordById {
            resource_id,
            record: Some(record),
        } = delta
            && let Ok(id) = Uuid::parse_str(resource_id)
        {
            for facet in &record.facets {
                if facet.facet == RESOURCE_PROPERTIES || facet.facet == RESOURCE_CLASSIFICATIONS {
                    self.retired.insert((id, facet.facet.clone()));
                }
            }
        }
    }

    pub(crate) fn sync(
        &mut self,
        source: &Graph,
        output: &mut Graph,
        context: Option<&LegacyContentContext>,
        author: Option<&Author>,
    ) -> Result<(), LegacyMigrationError> {
        // Remove the retained Surface assertions before admitting their exact
        // Resource handles. The original source and receipt remain unchanged.
        for (key, node) in source.nodes() {
            if let Some(output_key) = output.get_node_key_by_id(node.id) {
                if !source
                    .legacy_node_properties(key)
                    .unwrap_or_default()
                    .is_empty()
                {
                    output.set_node_facet(
                        output_key,
                        super::node_facets::SEMANTIC_PROPERTIES,
                        &Vec::<NodeProperty>::new(),
                    );
                }
                if !source
                    .legacy_node_classifications(key)
                    .unwrap_or_default()
                    .is_empty()
                {
                    output.set_node_facet(
                        output_key,
                        super::node_facets::SEMANTIC_CLASSIFICATIONS,
                        &Vec::<crate::types::NodeClassification>::new(),
                    );
                }
            }
        }
        let mut present_props = BTreeMap::new();
        let mut present_tags = BTreeSet::new();
        let mut present_classes = BTreeSet::new();
        for (key, node) in source.nodes() {
            let surface = node.id;
            let raw_url = node.primary_address().as_url_str();
            let props = source.legacy_node_properties(key).unwrap_or_default();
            let classes = source.legacy_node_classifications(key).unwrap_or_default();
            if node.tags.is_empty() && props.is_empty() && classes.is_empty() {
                continue;
            }
            let resource = ResourceNode::new(raw_url);
            if output.resource(resource.id()).is_none() {
                let record = PersistedResourceRecord {
                    canonical_iri: resource.canonical_iri().into(),
                    facets: vec![],
                };
                if !output.set_resource_record(resource.id(), Some(record)) {
                    return Err(error("content Resource insertion refused"));
                }
            }
            if let Some(output_key) = output.get_node_key_by_id(surface) {
                if output.shown_resource_id(output_key).is_none() {
                    output.set_shown_resource(surface, Some(resource.id()));
                }
            }
            for mut property in props {
                let original = serde_json::to_value(&property).expect("property serializes");
                let id = property.statement_id.clone();
                let owner = *self.properties.entry(id.clone()).or_insert(resource.id());
                if self.active_properties.contains_key(&(surface, id.clone())) == false
                    && owner != resource.id()
                    && self.active_properties.keys().any(|(_, held)| held == &id)
                {
                    return Err(error("legacy literal handle reused by another Resource"));
                }
                if self.retired.contains(&(owner, RESOURCE_PROPERTIES.into())) {
                    continue;
                }
                if let Some(existing_owner) = present_props.insert((surface, id.clone()), owner)
                    && existing_owner != owner
                {
                    return Err(error("literal ownership conflict"));
                }
                let mut held = output.resource_properties(owner);
                if property.provenance_iri.is_none() {
                    property.provenance_iri = held
                        .iter()
                        .find(|existing| existing.statement_id == id)
                        .and_then(|existing| existing.provenance_iri.clone())
                        .or_else(|| {
                            Some(author.map(Author::asserter_iri).unwrap_or_else(|| {
                                super::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI.into()
                            }))
                        });
                }
                if let Some(existing) = held.iter_mut().find(|entry| entry.statement_id == id) {
                    if !existing.content_eq(&property) {
                        return Err(error("legacy literal handle changed content"));
                    }
                    existing.asserted_at_ms = property.asserted_at_ms;
                } else {
                    held.push(property.clone());
                }
                output
                    .write_resource_content(owner, RESOURCE_PROPERTIES, &held)
                    .map_err(content_error)?;
                self.note(
                    output,
                    owner,
                    LegacyContentOrigin {
                        kind: "literal".into(),
                        record_id: id,
                        surface_id: surface,
                        original_url: raw_url.into(),
                        author: author.cloned(),
                        uncertainty: uncertainty(author),
                        original,
                    },
                )?;
            }
            for tag in &node.tags {
                let tag_key = (surface, tag.clone());
                present_tags.insert(tag_key.clone());
                if !self.tags.contains_key(&tag_key) {
                    let owner_iri = match author {
                        Some(author) => author.asserter_iri(),
                        None => context.map(|context| context.original_mere_iri.clone()).filter(|iri| !iri.is_empty()).ok_or_else(|| error("unattributed legacy tags require stable original-mere content context"))?,
                    };
                    let iri = tag_concept_iri(&owner_iri, tag);
                    let mut statement = SemanticStatement::new(
                        super::resource::TAGGED_WITH_IRI.into(),
                        None,
                        None,
                        GraphScope::Default,
                        Some(author.map(Author::asserter_iri).unwrap_or_else(|| {
                            super::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI.into()
                        })),
                        None,
                    );
                    let handle_key = serde_json::to_vec(&(surface, resource.id(), &iri))
                        .expect("key serializes");
                    statement.statement_id = format!(
                        "legacy-tag:{}",
                        Uuid::new_v5(&Uuid::NAMESPACE_URL, &handle_key)
                    );
                    let source_iri = statement.provenance_iri.clone();
                    let mut handle = statement.statement_id.clone();
                    output
                        .tag_resource_with_statement(
                            resource.id(),
                            &iri,
                            TagConcept {
                                owner_iri,
                                label: tag.clone(),
                            },
                            statement,
                        )
                        .map_err(content_error)?;
                    if let Some(existing) = output
                        .resource_edges()
                        .filter(|(from, to, _)| {
                            from.id() == resource.id()
                                && to.id() == ResourceNode::for_term(&iri).id()
                        })
                        .flat_map(|(_, _, payload)| payload.semantic_statements())
                        .find(|held| {
                            held.predicate == super::resource::TAGGED_WITH_IRI
                                && held.provenance_iri == source_iri
                        })
                    {
                        handle = existing.statement_id.clone();
                    }
                    self.note(
                        output,
                        resource.id(),
                        LegacyContentOrigin {
                            kind: "tag".into(),
                            record_id: handle.clone(),
                            surface_id: surface,
                            original_url: raw_url.into(),
                            author: author.cloned(),
                            uncertainty: uncertainty(author),
                            original: serde_json::json!(tag),
                        },
                    )?;
                    self.tags.insert(
                        tag_key,
                        TagMember {
                            resource: resource.id(),
                            concept: ResourceNode::for_term(&iri).id(),
                            handle,
                        },
                    );
                }
            }
            for classification in classes {
                let class_key = (
                    surface,
                    serde_json::to_string(&classification.scheme).expect("scheme serializes"),
                    classification.value.clone(),
                );
                present_classes.insert(class_key.clone());
                let member = self
                    .classes
                    .entry(class_key.clone())
                    .or_insert_with(|| {
                        let encoded = serde_json::to_vec(&(class_key, resource.id()))
                            .expect("key serializes");
                        ClassMember {
                            resource: resource.id(),
                            variant: format!(
                                "legacy-classification:{}",
                                Uuid::new_v5(&Uuid::NAMESPACE_URL, &encoded)
                            ),
                        }
                    })
                    .clone();
                if self
                    .retired
                    .contains(&(member.resource, RESOURCE_CLASSIFICATIONS.into()))
                {
                    continue;
                }
                let original =
                    serde_json::to_value(&classification).expect("classification serializes");
                let mut variants = output.resource_classification_variants(member.resource);
                if let Some(existing) = variants
                    .iter_mut()
                    .find(|entry| entry.variant_id == member.variant)
                {
                    existing.classification = classification;
                } else {
                    variants.push(ClassificationVariant {
                        variant_id: member.variant.clone(),
                        original_variant_id: None,
                        classification,
                        origins: vec![ClassificationOrigin {
                            surface_id: surface,
                            author: author.cloned(),
                            uncertainty: uncertainty(author),
                        }],
                    });
                }
                output
                    .write_resource_content(member.resource, RESOURCE_CLASSIFICATIONS, &variants)
                    .map_err(content_error)?;
                self.note(
                    output,
                    member.resource,
                    LegacyContentOrigin {
                        kind: "classification".into(),
                        record_id: member.variant,
                        surface_id: surface,
                        original_url: raw_url.into(),
                        author: author.cloned(),
                        uncertainty: uncertainty(author),
                        original,
                    },
                )?;
            }
            // Retained originals remain in the source receipt and content-origin ledger.
            if let Some(output_key) = output.get_node_key_by_id(surface) {
                if let Some(node) = output.inner.node_mut(output_key) {
                    node.tags.clear();
                }
                output.set_node_facet(
                    output_key,
                    super::node_facets::SEMANTIC_PROPERTIES,
                    &Vec::<NodeProperty>::new(),
                );
                output.set_node_facet(
                    output_key,
                    super::node_facets::SEMANTIC_CLASSIFICATIONS,
                    &Vec::<crate::types::NodeClassification>::new(),
                );
            }
        }
        let remaining_ids: BTreeSet<_> = present_props.keys().map(|(_, id)| id.clone()).collect();
        for ((_, id), owner) in &self.active_properties {
            if !remaining_ids.contains(id)
                && !self.retired.contains(&(*owner, RESOURCE_PROPERTIES.into()))
            {
                output
                    .retract_resource_property(*owner, id)
                    .map_err(content_error)?;
            }
        }
        self.active_properties = present_props;
        let removed: Vec<_> = self
            .tags
            .keys()
            .filter(|key| !present_tags.contains(*key))
            .cloned()
            .collect();
        for key in removed {
            let member = self.tags.remove(&key).expect("key exists");
            // Typed pair replacements may already have retired this raw assertion.
            if !self.retired_tag_handles.contains(&member.handle)
                && !self.tags.values().any(|held| held.handle == member.handle)
            {
                output.retract_assertion(&member.handle);
            }
        }
        let removed: Vec<_> = self
            .classes
            .keys()
            .filter(|key| !present_classes.contains(*key))
            .cloned()
            .collect();
        for key in removed {
            let member = self.classes.remove(&key).expect("key exists");
            if !self
                .retired
                .contains(&(member.resource, RESOURCE_CLASSIFICATIONS.into()))
            {
                output
                    .remove_resource_classification(member.resource, &member.variant)
                    .map_err(content_error)?;
            }
        }
        Ok(())
    }

    fn note(
        &self,
        output: &mut Graph,
        id: Uuid,
        note: LegacyContentOrigin,
    ) -> Result<(), LegacyMigrationError> {
        let mut notes: Vec<LegacyContentOrigin> = output
            .resource_content(id, LEGACY_CONTENT_ORIGINS)
            .unwrap_or_default();
        if !notes.iter().any(|existing| {
            existing.kind == note.kind
                && existing.record_id == note.record_id
                && existing.surface_id == note.surface_id
        }) {
            notes.push(note);
            output
                .write_resource_content(id, LEGACY_CONTENT_ORIGINS, &notes)
                .map_err(content_error)?;
        }
        Ok(())
    }
}
fn uncertainty(author: Option<&Author>) -> Option<String> {
    author
        .is_none()
        .then(|| "baseline minting author and historical URL are unavailable".into())
}
fn error(detail: &str) -> LegacyMigrationError {
    LegacyMigrationError(detail.into())
}
fn content_error(error: super::resource_content::ContentError) -> LegacyMigrationError {
    LegacyMigrationError(error.to_string())
}

#[cfg(test)]
#[path = "legacy_content_tests.rs"]
mod tests;
