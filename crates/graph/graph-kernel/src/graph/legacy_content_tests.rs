// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::super::legacy_resource_migration::{
    migrate_legacy_prefix, migrate_legacy_prefix_with_content_context,
};
use super::super::node_facets::{SEMANTIC_CLASSIFICATIONS, SEMANTIC_PROPERTIES};
use super::super::{AttributedDelta, replay_captured_deltas_onto};
use super::*;
use crate::types::{
    ClassificationProvenance, ClassificationScheme, ClassificationStatus, NodeClassification,
};
use euclid::default::Point2D;
fn entry(delta: CapturedDelta) -> AttributedDelta {
    AttributedDelta {
        author: Author::person("historical"),
        delta,
    }
}
fn property(id: &str) -> NodeProperty {
    let mut property = NodeProperty::new("urn:predicate:literal".into(), "bonjour".into());
    property.statement_id = id.into();
    property.lang = Some("fr".into());
    property
}
fn classification(status: ClassificationStatus) -> NodeClassification {
    NodeClassification {
        scheme: ClassificationScheme::Custom("rdf:type".into()),
        value: "urn:type:Article".into(),
        label: Some("full original".into()),
        confidence: 0.5,
        provenance: ClassificationProvenance::AgentSuggested,
        status,
        primary: false,
    }
}

#[test]
fn content_migration_baseline_requires_original_mere_only_for_unknown_tags() {
    let mut baseline = Graph::new();
    let key = baseline.add_node("https://legacy.test/a".into(), Point2D::zero());
    baseline.legacy_insert_node_tag(key, "Cat".into());
    let presentation = serde_json::json!({
        "ordered_tags": ["Cat"], "icon_overrides": {"Cat": {"Lucide": "cat"}}
    });
    baseline.set_node_facet(
        key,
        super::super::node_facets::PRESENTATION_TAGS,
        &presentation,
    );
    baseline.legacy_append_node_property(key, property("literal"));
    baseline.legacy_add_node_classification(key, classification(ClassificationStatus::Rejected));
    let before = baseline.to_snapshot();
    assert!(migrate_legacy_prefix(&baseline, &[]).is_err());
    assert_eq!(
        serde_json::to_value(baseline.to_snapshot()).unwrap(),
        serde_json::to_value(before).unwrap()
    );
    let context = LegacyContentContext {
        original_mere_iri: "urn:mere:original".into(),
    };
    let migrated =
        migrate_legacy_prefix_with_content_context(&baseline, &[], &[], Some(&context)).unwrap();
    let id = ResourceNode::new("https://legacy.test/a").id();
    let concept = ResourceNode::for_term(&tag_concept_iri(&context.original_mere_iri, "Cat"));
    assert_eq!(
        migrated
            .graph
            .resource_tag_concept(concept.id())
            .unwrap()
            .owner_iri,
        context.original_mere_iri
    );
    assert_eq!(
        migrated.graph.resource_properties(id)[0].statement_id,
        "literal"
    );
    assert_eq!(
        migrated.graph.resource_properties(id)[0]
            .provenance_iri
            .as_deref(),
        Some(super::super::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI)
    );
    let variants = migrated.graph.resource_classification_variants(id);
    assert_eq!(
        variants[0].classification.status,
        ClassificationStatus::Rejected
    );
    assert!(variants[0].origins[0].author.is_none());
    assert!(variants[0].origins[0].uncertainty.is_some());
    let mut replayed = baseline.clone();
    replay_captured_deltas_onto(&mut replayed, migrated.baseline_effects.clone());
    assert!(replayed.get_node(key).unwrap().tags.is_empty());
    assert_eq!(replayed.facets(), migrated.graph.facets());
    assert_eq!(
        serde_json::to_value(replayed.to_snapshot()).unwrap(),
        serde_json::to_value(migrated.graph.to_snapshot()).unwrap()
    );
    assert_eq!(
        replayed.to_snapshot().resources,
        migrated.graph.to_snapshot().resources
    );
    assert_eq!(
        replayed.to_snapshot().resource_edges,
        migrated.graph.to_snapshot().resource_edges
    );
    let mut tag_free = baseline.clone();
    tag_free.legacy_remove_node_tag(key, "Cat");
    assert!(migrate_legacy_prefix(&tag_free, &[]).is_ok());
    let invalid = LegacyContentContext {
        original_mere_iri: "relative".into(),
    };
    assert!(
        migrate_legacy_prefix_with_content_context(&baseline, &[], &[], Some(&invalid)).is_err()
    );
}

#[test]
fn content_migration_historical_mints_do_not_follow_later_navigation() {
    let mut baseline = Graph::new();
    let key = baseline.add_node("https://legacy.test/first".into(), Point2D::zero());
    let surface = baseline.get_node(key).unwrap().id.to_string();
    let entries = vec![
        entry(CapturedDelta::ReplayAppendNodePropertyById {
            node_id: surface.clone(),
            property: property("first"),
        }),
        entry(CapturedDelta::ReplayInsertNodeTagById {
            node_id: surface.clone(),
            tag: "First".into(),
        }),
        entry(CapturedDelta::ReplaySetNodeUrlById {
            node_id: surface.clone(),
            new_url: "https://legacy.test/second".into(),
        }),
        entry(CapturedDelta::ReplayAppendNodePropertyById {
            node_id: surface.clone(),
            property: {
                let mut second = property("second");
                second.value = "salut".into();
                second
            },
        }),
        entry(CapturedDelta::ReplayInsertNodeTagById {
            node_id: surface.clone(),
            tag: "Second".into(),
        }),
    ];
    let migrated = migrate_legacy_prefix(&baseline, &entries).unwrap();
    let first = ResourceNode::new("https://legacy.test/first").id();
    let second = ResourceNode::new("https://legacy.test/second").id();
    assert_eq!(
        migrated.graph.resource_properties(first)[0].statement_id,
        "first"
    );
    assert_eq!(
        migrated.graph.resource_properties(second)[0].statement_id,
        "second"
    );
    assert!(migrated.graph.resource_tag_labels(first).contains("First"));
    assert!(!migrated.graph.resource_tag_labels(second).contains("First"));
    assert!(
        migrated
            .graph
            .resource_tag_labels(second)
            .contains("Second")
    );
    assert_eq!(
        migrated.graph.resource_properties(first)[0]
            .provenance_iri
            .as_deref(),
        Some(Author::person("historical").asserter_iri().as_str())
    );
    let mut replayed = baseline.clone();
    replay_captured_deltas_onto(&mut replayed, migrated.baseline_effects.clone());
    for effects in &migrated.entry_effects {
        replay_captured_deltas_onto(&mut replayed, effects.clone());
    }
    assert!(replayed.get_node(key).unwrap().tags.is_empty());
    assert_eq!(replayed.facets(), migrated.graph.facets());
    assert_eq!(
        serde_json::to_value(replayed.to_snapshot()).unwrap(),
        serde_json::to_value(migrated.graph.to_snapshot()).unwrap()
    );
    assert_eq!(
        replayed.to_snapshot().resources,
        migrated.graph.to_snapshot().resources
    );
    assert_eq!(
        replayed.to_snapshot().resource_edges,
        migrated.graph.to_snapshot().resource_edges
    );
}

#[test]
fn content_migration_aliases_keep_divergent_classifications_and_exact_literal_copies() {
    let mut baseline = Graph::new();
    let a = baseline.add_node("https://legacy.test/a#one".into(), Point2D::zero());
    let b = baseline.add_node("https://legacy.test/a#two".into(), Point2D::zero());
    baseline.set_node_facet(a, SEMANTIC_PROPERTIES, &vec![property("same")]);
    baseline.set_node_facet(b, SEMANTIC_PROPERTIES, &vec![property("same")]);
    baseline.set_node_facet(
        a,
        SEMANTIC_CLASSIFICATIONS,
        &vec![classification(ClassificationStatus::Accepted)],
    );
    baseline.set_node_facet(
        b,
        SEMANTIC_CLASSIFICATIONS,
        &vec![classification(ClassificationStatus::Rejected)],
    );
    let migrated = migrate_legacy_prefix(&baseline, &[]).unwrap();
    let id = ResourceNode::new("https://legacy.test/a").id();
    assert_eq!(migrated.graph.resource_properties(id).len(), 1);
    assert_eq!(migrated.graph.resource_classification_variants(id).len(), 2);
    assert!(migrated.graph.resource_classifications_conflicted(id));
    let mut invalid = property("same");
    invalid.value = "different".into();
    baseline.set_node_facet(b, SEMANTIC_PROPERTIES, &vec![invalid]);
    assert!(migrate_legacy_prefix(&baseline, &[]).is_err());
}
