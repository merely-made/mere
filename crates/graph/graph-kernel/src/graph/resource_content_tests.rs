// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shared Resource content and legacy compatibility controls.
use super::apply::{GraphDelta, GraphDeltaResult, apply_graph_delta};
use super::resource_classifications::{
    ClassificationOrigin, ClassificationVariant, merge_classification_variants,
};
use super::resource_content::ContentError;
use super::resource_tags::{TagConcept, tag_concept_iri};
use super::*;
use crate::types::{
    ClassificationProvenance, ClassificationScheme, ClassificationStatus, GraphScope,
    NodeClassification, NodeProperty,
};
use euclid::default::Point2D;
use uuid::Uuid;

fn surfaces() -> (Graph, NodeKey, NodeKey) {
    let mut graph = Graph::new();
    let a = graph.add_node("https://EXAMPLE.test/page#one".into(), Point2D::zero());
    let b = graph.add_node("https://example.test/page#two".into(), Point2D::zero());
    graph.refresh_surface_resource(a);
    graph.refresh_surface_resource(b);
    (graph, a, b)
}
fn classification(status: ClassificationStatus) -> NodeClassification {
    NodeClassification {
        scheme: ClassificationScheme::Custom("rdf:type".into()),
        value: "https://types.test/Article".into(),
        label: Some("Article".into()),
        confidence: 0.75,
        provenance: ClassificationProvenance::Imported,
        status,
        primary: false,
    }
}
fn property(id: &str, author: &str, time: u64) -> NodeProperty {
    NodeProperty {
        statement_id: id.into(),
        predicate: "https://schema.org/datePublished".into(),
        value: "2026-10-08".into(),
        datatype: Some("http://www.w3.org/2001/XMLSchema#date".into()),
        lang: None,
        graph_scope: GraphScope::User,
        provenance_iri: Some(author.into()),
        asserted_at_ms: Some(time),
    }
}

#[test]
fn resource_content_aliases_literals_navigation_and_exact_retraction() {
    let (mut graph, a, b) = surfaces();
    let id = graph.shown_resource_id(a).unwrap();
    assert_eq!(graph.shown_resource_id(b), Some(id));
    assert!(graph.append_node_properties(
        a,
        vec![
            property("literal-a", "urn:author:a", 0),
            property("literal-b", "urn:author:b", 1)
        ]
    ));
    assert_eq!(graph.node_properties(b).unwrap().len(), 2);
    assert!(graph.legacy_node_properties(a).unwrap().is_empty());
    let revision = graph.revision();
    assert!(
        !graph.append_node_properties(b, vec![property("duplicate-new-id", "urn:author:a", 0)])
    );
    assert_eq!(graph.revision(), revision);
    assert!(graph.append_node_properties(b, vec![property("duplicate-new-id", "urn:author:a", 7)]));
    assert_eq!(graph.resource_properties(id)[0].statement_id, "literal-a");
    assert_eq!(graph.resource_properties(id)[0].asserted_at_ms, Some(7));
    graph.update_node_url(a, "https://example.test/next".into());
    graph.refresh_surface_resource(a);
    assert!(graph.node_properties(a).unwrap().is_empty());
    assert_eq!(graph.node_properties(b).unwrap().len(), 2);
    assert!(graph.retract_resource_property(id, "literal-a").unwrap());
    assert!(!graph.retract_resource_property(id, "literal-a").unwrap());
    assert_eq!(graph.resource_properties(id)[0].statement_id, "literal-b");
    let snapshot = graph.to_snapshot();
    let loaded = Graph::try_from_snapshot(&snapshot).unwrap();
    assert_eq!(
        loaded.resource_properties(id),
        graph.resource_properties(id)
    );
}

#[test]
fn resource_content_literal_collision_is_atomic_with_distinct_id_control() {
    let (mut graph, a, b) = surfaces();
    assert!(graph.append_node_properties(a, vec![property("held", "urn:author:a", 0)]));
    let id = graph.shown_resource_id(b).unwrap();
    let before = graph.to_snapshot();
    let revision = graph.revision();
    let mut conflicting = property("held", "urn:author:a", 0);
    conflicting.value = "different".into();
    assert!(matches!(
        graph.append_resource_properties(id, vec![conflicting]),
        Err(ContentError::HandleCollision(_))
    ));
    assert_eq!(graph.to_snapshot().resources, before.resources);
    assert_eq!(graph.revision(), revision);
    assert!(
        graph
            .append_resource_properties(id, vec![property("distinct", "urn:author:b", 0)])
            .unwrap()
    );
    assert_eq!(graph.resource_properties(id).len(), 2);
}

#[test]
fn resource_content_tag_owners_exact_labels_and_explicit_reuse() {
    let (mut graph, a, b) = surfaces();
    graph.write_as(Author::person("alice"), |graph| {
        assert!(graph.insert_node_tags(a, vec!["Cat".into()]))
    });
    graph.write_as(Author::person("bob"), |graph| {
        assert!(graph.insert_node_tags(b, vec!["Cat".into(), "cat".into(), "ÃƒÂ§Ã…â€™Ã‚Â«".into()]))
    });
    let id = graph.shown_resource_id(a).unwrap();
    assert_eq!(graph.node_content_tags(a).unwrap().len(), 3);
    let alice = Author::person("alice").asserter_iri();
    let bob = Author::person("bob").asserter_iri();
    assert_ne!(tag_concept_iri(&alice, "Cat"), tag_concept_iri(&bob, "Cat"));
    let reused = tag_concept_iri(&alice, "Cat");
    graph.write_as(Author::person("bob"), |graph| {
        assert!(
            graph
                .tag_resource_with_concept(
                    id,
                    &reused,
                    TagConcept {
                        owner_iri: alice.clone(),
                        label: "Cat".into()
                    }
                )
                .unwrap()
        )
    });
    let target = ResourceNode::for_term(&reused).id();
    assert_eq!(
        graph
            .resource_edges()
            .filter(|(from, to, _)| from.id() == id && to.id() == target)
            .flat_map(|(_, _, p)| p.semantic_statements())
            .count(),
        2
    );
    graph.write_as(Author::person("alice"), |graph| {
        assert!(graph.remove_shown_tag(a, "Cat"))
    });
    assert!(graph.node_content_tags(b).unwrap().contains("Cat"));
    let revision = graph.revision();
    graph.write_as(Author::person("alice"), |graph| {
        assert!(!graph.remove_shown_tag(a, "Cat"))
    });
    assert_eq!(graph.revision(), revision);
}

#[test]
fn resource_content_classification_conflicts_require_precise_edits() {
    let (mut graph, a, b) = surfaces();
    graph.write_as(Author::person("alice"), |graph| {
        assert!(
            graph.add_node_classifications(a, vec![classification(ClassificationStatus::Accepted)])
        )
    });
    graph.write_as(Author::engine("classifier", "2"), |graph| {
        assert!(
            graph.add_node_classifications(b, vec![classification(ClassificationStatus::Rejected)])
        )
    });
    let id = graph.shown_resource_id(a).unwrap();
    let variants = graph.resource_classification_variants(id);
    assert_eq!(variants.len(), 2);
    assert!(graph.resource_classifications_conflicted(id));
    assert_eq!(
        variants[0].origins[0].surface_id,
        graph.get_node(a).unwrap().id
    );
    assert_eq!(
        variants[1].origins[0]
            .author
            .as_ref()
            .unwrap()
            .version
            .as_deref(),
        Some("2")
    );
    assert!(matches!(
        graph.unique_shown_classification(
            a,
            &variants[0].classification.scheme,
            &variants[0].classification.value
        ),
        Err(ContentError::AmbiguousClassification(_))
    ));
    let before = graph.to_snapshot();
    let result = apply_graph_delta(
        &mut graph,
        GraphDelta::SetNodeClassificationStatus {
            key: b,
            scheme: variants[0].classification.scheme.clone(),
            value: variants[0].classification.value.clone(),
            status: ClassificationStatus::Verified,
        },
    );
    assert!(matches!(
        result,
        GraphDeltaResult::NodeMetadataUpdated(false)
    ));
    assert_eq!(graph.to_snapshot().resources, before.resources);
    assert!(
        graph
            .edit_resource_classification(
                id,
                &variants[1].variant_id,
                Some(ClassificationStatus::Verified),
                None
            )
            .unwrap()
    );
    let updated = graph.resource_classification_variants(id);
    assert_eq!(updated[0], variants[0]);
    assert_eq!(updated[1].origins, variants[1].origins);
    assert!(
        !graph
            .edit_resource_classification(
                id,
                &variants[1].variant_id,
                Some(ClassificationStatus::Verified),
                None
            )
            .unwrap()
    );
    assert!(
        graph
            .edit_resource_classification(id, "absent", None, Some(true))
            .is_err()
    );
    assert!(
        graph
            .remove_resource_classification(id, &variants[1].variant_id)
            .unwrap()
    );
    assert!(!graph.resource_classifications_conflicted(id));
}

#[test]
fn resource_content_variant_union_preserves_records_and_origin_controls() {
    let left = ClassificationVariant {
        variant_id: "left".into(),
        original_variant_id: None,
        classification: classification(ClassificationStatus::Accepted),
        origins: vec![ClassificationOrigin {
            surface_id: Uuid::from_u128(1),
            author: Some(Author::person("alice")),
            uncertainty: None,
        }],
    };
    let mut equal = left.clone();
    equal.origins[0].surface_id = Uuid::from_u128(2);
    let merged = merge_classification_variants(&[left.clone()], &[equal]).unwrap();
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].origins.len(), 2);
    let mut rejected = left.clone();
    rejected.variant_id = "right".into();
    rejected.classification.status = ClassificationStatus::Rejected;
    assert_eq!(
        merge_classification_variants(&[left.clone()], &[rejected.clone()])
            .unwrap()
            .len(),
        2
    );
    rejected.variant_id = left.variant_id.clone();
    let divergent = merge_classification_variants(&[left.clone()], &[rejected.clone()]).unwrap();
    assert_eq!(divergent.len(), 2);
    assert!(
        divergent
            .iter()
            .all(|version| version.original_variant_id.as_deref() == Some("left"))
    );
    assert_eq!(
        divergent,
        merge_classification_variants(&[rejected.clone()], &[left.clone()]).unwrap()
    );
    assert_eq!(
        divergent,
        merge_classification_variants(&divergent, &divergent).unwrap()
    );
    assert_eq!(
        divergent,
        merge_classification_variants(&divergent, &[left.clone(), rejected]).unwrap()
    );
    assert_eq!(
        merge_classification_variants(&[left.clone()], &[left.clone()]).unwrap(),
        vec![left]
    );
}

#[test]
fn resource_content_legacy_replay_remains_surface_only() {
    let mut graph = Graph::new();
    let key = graph.add_node("https://legacy.test/page".into(), Point2D::zero());
    let id = graph.get_node(key).unwrap().id;
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplayAppendNodePropertyById {
            node_id: id,
            property: property("legacy", "urn:author:a", 0),
        },
    );
    assert!(graph.shown_resource_id(key).is_none());
    assert_eq!(graph.legacy_node_properties(key).unwrap().len(), 1);
    assert!(graph.append_node_properties(key, vec![property("live", "urn:author:b", 1)]));
    assert_eq!(graph.node_properties(key).unwrap()[0].statement_id, "live");
    assert_eq!(
        graph.legacy_node_properties(key).unwrap()[0].statement_id,
        "legacy"
    );
}

#[test]
fn resource_content_concept_identity_validation_is_atomic_and_exact() {
    let (mut graph, a, _) = surfaces();
    let id = graph.shown_resource_id(a).unwrap();
    let before = serde_json::to_value(graph.to_snapshot()).unwrap();
    let revision = graph.revision();
    for (iri, owner) in [
        ("relative", "urn:owner:valid"),
        ("urn:tag:valid", "relative"),
        ("urn:tag:valid", ""),
    ] {
        assert!(
            graph
                .tag_resource_with_concept(
                    id,
                    iri,
                    TagConcept {
                        owner_iri: owner.into(),
                        label: "猫 Exact".into()
                    }
                )
                .is_err()
        );
        assert_eq!(serde_json::to_value(graph.to_snapshot()).unwrap(), before);
        assert_eq!(graph.revision(), revision);
    }
    assert!(
        graph
            .tag_resource_with_concept(
                id,
                "urn:tag:Valid#Cat",
                TagConcept {
                    owner_iri: "https://owner.test/#Alice".into(),
                    label: "猫 Exact".into()
                }
            )
            .unwrap()
    );
    assert_eq!(
        graph
            .resource_tag_concept(ResourceNode::for_term("urn:tag:Valid#Cat").id())
            .unwrap()
            .label,
        "猫 Exact"
    );
    let mut snapshot = graph.to_snapshot();
    let record = snapshot
        .resources
        .iter_mut()
        .find(|record| record.canonical_iri == "urn:tag:Valid#Cat")
        .unwrap();
    record
        .facets
        .iter_mut()
        .find(|facet| facet.facet == super::resource_content::TAG_CONCEPT)
        .unwrap()
        .value_json = serde_json::to_string(&TagConcept {
        owner_iri: "relative".into(),
        label: "猫 Exact".into(),
    })
    .unwrap();
    assert!(Graph::try_from_snapshot(&snapshot).is_err());
    assert!(Graph::try_from_snapshot(&graph.to_snapshot()).is_ok());
}

#[test]
fn resource_content_checked_resource_handles_keep_equal_copies_and_reject_bad_owners() {
    let (mut graph, a, _) = surfaces();
    let id = graph.shown_resource_id(a).unwrap();
    let held = property("opaque\nhandle", "urn:author:a", 0);
    assert!(
        graph
            .append_resource_properties(id, vec![held.clone()])
            .unwrap()
    );
    let original = graph.to_snapshot();
    let mut copied = original.clone();
    let facet = copied
        .resources
        .iter_mut()
        .find(|record| record.canonical_iri == "https://example.test/page")
        .unwrap()
        .facets
        .iter_mut()
        .find(|facet| facet.facet == super::resource_content::RESOURCE_PROPERTIES)
        .unwrap();
    facet.value_json = serde_json::to_string(&vec![held.clone(), held.clone()]).unwrap();
    let loaded = Graph::try_from_snapshot(&copied).unwrap();
    assert!(loaded.validate_active_resource_assertion_handles().is_ok());
    assert_eq!(loaded.resource_properties(id).len(), 2);
    let mut changed = held.clone();
    changed.value = "different".into();
    let mut bad = copied.clone();
    bad.resources
        .iter_mut()
        .find(|record| record.canonical_iri == "https://example.test/page")
        .unwrap()
        .facets
        .iter_mut()
        .find(|facet| facet.facet == super::resource_content::RESOURCE_PROPERTIES)
        .unwrap()
        .value_json = serde_json::to_string(&vec![held.clone(), changed]).unwrap();
    let untouched = serde_json::to_value(&bad).unwrap();
    assert!(Graph::try_from_snapshot(&bad).is_err());
    assert_eq!(serde_json::to_value(&bad).unwrap(), untouched);
    let other = ResourceNode::for_term("urn:resource:other");
    let mut bad = original.clone();
    bad.resources
        .push(crate::persistence::PersistedResourceRecord {
            canonical_iri: other.canonical_iri().into(),
            facets: vec![crate::persistence::PersistedResourceFacet {
                facet: super::resource_content::RESOURCE_PROPERTIES.into(),
                value_json: serde_json::to_string(&vec![held.clone()]).unwrap(),
            }],
        });
    assert!(Graph::try_from_snapshot(&bad).is_err());
    let last = bad.resources.last_mut().unwrap();
    let mut distinct = held.clone();
    distinct.statement_id = "distinct-owner".into();
    last.facets[0].value_json = serde_json::to_string(&vec![distinct]).unwrap();
    assert!(
        Graph::try_from_snapshot(&bad)
            .unwrap()
            .validate_active_resource_assertion_handles()
            .is_ok()
    );
    let mut bad = original.clone();
    bad.nodes[0].properties.push(held.clone());
    assert!(Graph::try_from_snapshot(&bad).is_err());
    let mut restored = Graph::try_from_snapshot(&original).unwrap();
    let surface = restored
        .get_node_key_by_id(graph.get_node(a).unwrap().id)
        .unwrap();
    restored.set_node_facet(
        surface,
        super::node_facets::SEMANTIC_PROPERTIES,
        &vec![held],
    );
    assert!(
        restored
            .validate_active_resource_assertion_handles()
            .is_err()
    );
    restored.set_node_facet(
        surface,
        super::node_facets::SEMANTIC_PROPERTIES,
        &Vec::<NodeProperty>::new(),
    );
    assert!(
        restored
            .validate_active_resource_assertion_handles()
            .is_ok()
    );
}

#[test]
fn resource_content_empty_batches_preserve_unbound_and_unknown_state() {
    let (mut graph, bound, _) = surfaces();
    let resource = graph.shown_resource_id(bound).unwrap();
    let mut record = graph.resource_record(resource).unwrap();
    let unknown = crate::persistence::PersistedResourceFacet {
        facet: "extension.opaque".into(),
        value_json: "{\"retained\":[1,\"exact\",null]}".into(),
    };
    record.facets.push(unknown.clone());
    assert!(graph.set_resource_record(resource, Some(record)));
    let unbound = graph.add_node("https://unbound.test/page".into(), Point2D::zero());
    let before = graph.to_snapshot();
    let facets = graph.facets().clone();
    let revision = graph.revision();
    let captures = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let sink = captures.clone();
    graph.set_recorder(Some(std::sync::Arc::new(move |delta| {
        sink.lock().unwrap().push(delta.clone());
    })));
    for key in [unbound, bound] {
        assert!(!graph.append_node_properties(key, vec![]));
        assert!(!graph.add_node_classifications(key, vec![]));
    }
    assert_eq!(graph.revision(), revision);
    assert!(captures.lock().unwrap().is_empty());
    assert_eq!(graph.shown_resource_id(unbound), None);
    let mut after = graph.to_snapshot();
    after.timestamp_secs = before.timestamp_secs;
    assert_eq!(
        serde_json::to_value(after).unwrap(),
        serde_json::to_value(before).unwrap()
    );
    assert_eq!(graph.facets(), &facets);

    assert!(graph.append_node_properties(
        unbound,
        vec![property("nonempty", "urn:author:positive", 10)]
    ));
    let new_resource = graph.shown_resource_id(unbound).unwrap();
    assert_eq!(graph.resource_properties(new_resource).len(), 1);
    assert!(
        graph.add_node_classifications(bound, vec![classification(ClassificationStatus::Accepted)])
    );
    assert_eq!(graph.resource_classification_variants(resource).len(), 1);
    assert!(
        graph
            .resource_record(resource)
            .unwrap()
            .facets
            .contains(&unknown)
    );
    assert!(graph.revision() > revision);
    assert!(!captures.lock().unwrap().is_empty());
}
