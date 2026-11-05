// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::graph::resource_tags::tag_concept_iri;
use crate::graph::{Graph, SemanticStatement};
use euclid::default::Point2D;

const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const ALICE: &str = "https://alice.test/me";
const BOB: &str = "https://bob.test/me";

fn graph_with_resource() -> (Graph, Uuid) {
    let mut graph = Graph::new();
    let node = graph.add_node("https://page.test/".into(), Point2D::new(0.0, 0.0));
    let resource = graph.ensure_surface_resource(node).unwrap();
    (graph, resource)
}

fn accepted(owner: &str, label: &str, asserter: &str, hash: &str) -> AcceptedTagSuggestion {
    AcceptedTagSuggestion {
        concept_owner_iri: owner.into(),
        concept_iri: tag_concept_iri(owner, label),
        concept_label: Some(label.into()),
        asserter_iri: asserter.into(),
        suggestion_hash_hex: hash.into(),
    }
}

fn tag_statements(graph: &Graph, resource: Uuid, concept_iri: &str) -> Vec<SemanticStatement> {
    let target = ResourceNode::for_term(concept_iri).id();
    graph
        .resource_edges()
        .filter(|(from, to, _)| from.id() == resource && to.id() == target)
        .flat_map(|(_, _, payload)| {
            payload
                .held_semantic_statements()
                .cloned()
                .collect::<Vec<_>>()
        })
        .filter(|statement| statement.predicate == TAGGED_WITH_IRI)
        .collect()
}

#[test]
fn admit_records_a_personal_statement_citing_the_suggestion() {
    let (mut graph, resource) = graph_with_resource();
    let draft = accepted("https://owner.test/", "Rust", ALICE, HASH_A);
    assert_eq!(
        graph.admit_accepted_tag_suggestion(resource, &draft),
        Ok(TagAdmission::Added)
    );
    let statements = tag_statements(&graph, resource, &draft.concept_iri);
    assert_eq!(statements.len(), 1);
    let statement = &statements[0];
    assert_eq!(statement.provenance_iri.as_deref(), Some(ALICE));
    let method = statement.basis.method.as_ref().unwrap();
    assert_eq!(method.iri, ACCEPTED_TAG_SUGGESTION_METHOD);
    assert_eq!(
        statement.basis.evidence,
        vec![format!("urn:eidetic:tag-suggestion:{HASH_A}")]
    );
    assert!(graph.resource_tag_labels(resource).contains("Rust"));
}

#[test]
fn readmitting_the_same_draft_is_idempotent() {
    let (mut graph, resource) = graph_with_resource();
    let draft = accepted("https://owner.test/", "Rust", ALICE, HASH_A);
    graph
        .admit_accepted_tag_suggestion(resource, &draft)
        .unwrap();
    let before = tag_statements(&graph, resource, &draft.concept_iri);
    assert_eq!(
        graph.admit_accepted_tag_suggestion(resource, &draft),
        Ok(TagAdmission::AlreadyHeld)
    );
    assert_eq!(tag_statements(&graph, resource, &draft.concept_iri), before);
}

#[test]
fn a_tag_held_by_another_asserter_remains() {
    let (mut graph, resource) = graph_with_resource();
    let draft = accepted("https://owner.test/", "Rust", ALICE, HASH_A);
    graph
        .tag_resource_with_asserter(
            resource,
            &draft.concept_iri,
            TagConcept {
                owner_iri: draft.concept_owner_iri.clone(),
                label: "Rust".into(),
            },
            BOB.into(),
        )
        .unwrap();
    let bobs = tag_statements(&graph, resource, &draft.concept_iri);
    assert_eq!(
        graph.admit_accepted_tag_suggestion(resource, &draft),
        Ok(TagAdmission::Added)
    );
    let all = tag_statements(&graph, resource, &draft.concept_iri);
    assert_eq!(all.len(), 2);
    assert!(all.contains(&bobs[0]));
    assert!(
        all.iter()
            .any(|s| s.provenance_iri.as_deref() == Some(ALICE))
    );
}

#[test]
fn admitting_leaves_other_personal_tags_alone() {
    let (mut graph, resource) = graph_with_resource();
    let other = accepted("https://owner.test/", "Other", ALICE, HASH_B);
    graph
        .admit_accepted_tag_suggestion(resource, &other)
        .unwrap();
    let before = tag_statements(&graph, resource, &other.concept_iri);
    let draft = accepted("https://owner.test/", "Rust", ALICE, HASH_A);
    graph
        .admit_accepted_tag_suggestion(resource, &draft)
        .unwrap();
    assert_eq!(tag_statements(&graph, resource, &other.concept_iri), before);
    let labels = graph.resource_tag_labels(resource);
    assert!(labels.contains("Other") && labels.contains("Rust"));
}

#[test]
fn equal_labels_with_different_owners_stay_distinct() {
    let (mut graph, resource) = graph_with_resource();
    let one = accepted("https://one.test/", "Rust", ALICE, HASH_A);
    let two = accepted("https://two.test/", "Rust", ALICE, HASH_A);
    assert_ne!(one.concept_iri, two.concept_iri);
    graph.admit_accepted_tag_suggestion(resource, &one).unwrap();
    assert_eq!(
        graph.admit_accepted_tag_suggestion(resource, &two),
        Ok(TagAdmission::Added)
    );
    assert_eq!(tag_statements(&graph, resource, &one.concept_iri).len(), 1);
    assert_eq!(tag_statements(&graph, resource, &two.concept_iri).len(), 1);
    assert_eq!(
        graph
            .resource_tag_concept(ResourceNode::for_term(&one.concept_iri).id())
            .unwrap()
            .owner_iri,
        "https://one.test/"
    );
}

#[test]
fn invalid_inputs_are_refused_without_writing() {
    let (mut graph, resource) = graph_with_resource();
    let good = accepted("https://owner.test/", "Rust", ALICE, HASH_A);
    let mut bad_concept = good.clone();
    bad_concept.concept_iri = "not an iri".into();
    let mut bad_owner = good.clone();
    bad_owner.concept_owner_iri = "owner".into();
    let mut bad_asserter = good.clone();
    bad_asserter.asserter_iri = "alice".into();
    let mut bad_hash = good.clone();
    bad_hash.suggestion_hash_hex = "ABC".into();
    let mut no_label = good.clone();
    no_label.concept_label = None;
    for bad in [bad_concept, bad_owner, bad_asserter, bad_hash, no_label] {
        assert!(matches!(
            graph.admit_accepted_tag_suggestion(resource, &bad),
            Err(ContentError::InvalidFacet(_))
        ));
    }
    assert_eq!(
        graph.admit_accepted_tag_suggestion(Uuid::nil(), &good),
        Err(ContentError::MissingResource)
    );
    assert!(graph.resource_tag_labels(resource).is_empty());
}

#[test]
fn a_known_concept_supplies_its_own_label() {
    let (mut graph, resource) = graph_with_resource();
    let draft = accepted("https://owner.test/", "Rust", ALICE, HASH_A);
    graph
        .admit_accepted_tag_suggestion(resource, &draft)
        .unwrap();
    let mut later = draft.clone();
    later.concept_label = None;
    later.asserter_iri = BOB.into();
    assert_eq!(
        graph.admit_accepted_tag_suggestion(resource, &later),
        Ok(TagAdmission::Added)
    );
    later.concept_label = Some("Different".into());
    later.asserter_iri = "https://carol.test/me".into();
    assert!(
        graph
            .admit_accepted_tag_suggestion(resource, &later)
            .is_err()
    );
}
