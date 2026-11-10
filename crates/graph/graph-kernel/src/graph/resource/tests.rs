// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::graph::{Graph, SurfaceNode};

fn record(iri: &str, value_json: &str) -> PersistedResourceRecord {
    PersistedResourceRecord {
        canonical_iri: chartulary::canonical_url(iri),
        facets: vec![PersistedResourceFacet {
            facet: "test.content".into(),
            value_json: value_json.into(),
        }],
    }
}

#[test]
fn resource_record_validation_is_atomic_and_replacement_clears_old_facets() {
    let mut graph = Graph::new();
    let iri = "https://example.com/resource";
    let id = chartulary::resource_id(iri);
    let original = record(iri, "{\"title\":\"original\"}");
    assert!(graph.set_resource_record(id, Some(original.clone())));
    assert_eq!(graph.resource_record(id), Some(original.clone()));
    assert!(!graph.set_resource_record(id, Some(original.clone())));
    assert!(!graph.set_resource_record(Uuid::nil(), Some(original.clone())));
    assert!(!graph.set_resource_record(id, Some(record(iri, "invalid JSON"))));
    let mut duplicate = original.clone();
    duplicate.facets.push(duplicate.facets[0].clone());
    assert!(!graph.set_resource_record(id, Some(duplicate)));
    assert_eq!(graph.resource_record(id), Some(original));
    assert!(graph.set_resource_record(
        id,
        Some(PersistedResourceRecord {
            canonical_iri: iri.into(),
            facets: vec![]
        })
    ));
    assert!(graph.resource_record(id).unwrap().facets.is_empty());
    assert!(graph.set_resource_record(id, None));
    assert!(graph.resource(id).is_none());
    assert!(graph.resource_facets.facets_of(&id).is_none());
}

#[test]
fn shown_resource_is_explicit_and_references_guard_inverse_creation() {
    let mut graph = Graph::new();
    let first = graph.add_node(
        "https://example.com/page".into(),
        euclid::default::Point2D::new(0.0, 0.0),
    );
    let second = graph.add_node(
        "https://example.com/page#part".into(),
        euclid::default::Point2D::new(1.0, 0.0),
    );
    let surface = graph.get_node(first).unwrap().id;
    let alias = graph.get_node(second).unwrap().id;
    let id = chartulary::resource_id("https://example.com/page");
    assert!(graph.shown_resource_id(first).is_none());
    assert!(!graph.set_shown_resource(surface, Some(id)));
    assert!(graph.set_resource_record(id, Some(record("https://example.com/page", "null"))));
    assert!(graph.set_shown_resource(surface, Some(id)));
    assert!(graph.set_shown_resource(alias, Some(id)));
    let mut expected = vec![surface, alias];
    expected.sort();
    assert_eq!(graph.surface_ids_showing_resource(id), expected);
    assert!(!graph.set_shown_resource(surface, Some(id)));
    assert!(!graph.set_resource_record(id, None));
    assert!(graph.remove_node(first));
    assert_eq!(graph.surface_ids_showing_resource(id), vec![alias]);
    assert!(graph.set_shown_resource(alias, None));
    assert!(graph.set_resource_record(id, None));
}

#[test]
fn resource_pair_rejects_conflicting_handles_before_replacing_truth() {
    use crate::graph::{EdgePayload, SemanticStatement, SemanticSubKind};
    let mut graph = Graph::new();
    let ids: Vec<_> = ["https://a.test/", "https://b.test/", "https://c.test/"]
        .into_iter()
        .map(|iri| {
            let id = chartulary::resource_id(iri);
            assert!(graph.set_resource_record(id, Some(record(iri, "null"))));
            id
        })
        .collect();
    let mut payload = EdgePayload::new();
    payload.push_persisted_semantic_statement(SemanticStatement {
        statement_id: "stable-handle".into(),
        predicate: crate::graph::predicate_iri(SemanticSubKind::Cites).into(),
        recognized_sub_kind: Some(SemanticSubKind::Cites),
        label: Some("original".into()),
        graph_scope: crate::types::GraphScope::Default,
        provenance_iri: Some("https://alice.test/".into()),
        asserted_at_ms: Some(17),
        first_asserted_ms: Some(17),
        withdrawn: None,
        basis: Default::default(),
    });
    let original = super::super::snapshot::persisted_edge_for_ids(ids[0], ids[1], &payload);
    let mut aggregate_only = original.clone();
    aggregate_only.assertions = None;
    aggregate_only.semantic = Some(Default::default());
    aggregate_only.semantic.as_mut().unwrap().sub_kinds =
        vec![crate::persistence::PersistedSemanticSubKind::Cites];
    assert!(!graph.set_resource_edges_between(ids[0], ids[1], &[aggregate_only.clone()]));
    assert!(!graph.set_resource_edges_between(ids[0], ids[1], &[aggregate_only]));
    assert!(
        graph
            .persisted_resource_edges_between(ids[0], ids[1])
            .is_empty()
    );
    assert!(graph.set_resource_edges_between(ids[0], ids[1], &[original.clone()]));
    assert!(!graph.set_resource_edges_between(ids[0], ids[1], &[original.clone()]));
    let mut conflicting = original.clone();
    conflicting.retained_assertions_mut()[0].label = Some("conflict".into());
    assert!(!graph.set_resource_edges_between(ids[0], ids[1], &[original.clone(), conflicting]));
    assert_eq!(
        graph.persisted_resource_edges_between(ids[0], ids[1]),
        vec![original.clone()]
    );
    let mut other_pair = original.clone();
    other_pair.to_node_id = ids[2].to_string();
    assert!(!graph.set_resource_edges_between(ids[0], ids[2], &[other_pair.clone()]));
    assert!(
        graph
            .persisted_resource_edges_between(ids[0], ids[2])
            .is_empty()
    );
    other_pair.retained_assertions_mut()[0].statement_id = "other-handle".into();
    assert!(graph.set_resource_edges_between(ids[0], ids[2], &[other_pair.clone()]));
    assert_eq!(
        graph.persisted_resource_edges_between(ids[0], ids[2]),
        vec![other_pair.clone()]
    );
    let first = graph.add_node(
        "https://surface.test/a".into(),
        euclid::default::Point2D::new(0.0, 0.0),
    );
    let second = graph.add_node(
        "https://surface.test/b".into(),
        euclid::default::Point2D::new(1.0, 0.0),
    );
    let mut collision = other_pair.clone();
    collision.retained_assertions_mut()[0].statement_id = "surface-handle".into();
    graph.connect_surface(
        first,
        second,
        super::super::snapshot::payload_from_persisted(&collision),
    );
    assert!(!graph.set_resource_edges_between(ids[0], ids[2], &[collision.clone()]));
    assert_eq!(
        graph.persisted_resource_edges_between(ids[0], ids[2]),
        vec![other_pair]
    );
    collision.retained_assertions_mut()[0].statement_id = "fresh-resource-handle".into();
    assert!(graph.set_resource_edges_between(ids[0], ids[2], &[collision.clone()]));
    assert_eq!(
        graph.persisted_resource_edges_between(ids[0], ids[2]),
        vec![collision]
    );
    assert!(!graph.set_resource_record(ids[0], None));
    assert!(graph.set_resource_edges_between(ids[0], ids[1], &[]));
    assert!(graph.set_resource_edges_between(ids[0], ids[2], &[]));
    assert!(graph.set_resource_record(ids[0], None));
}

#[test]
fn resource_identity_is_common_and_canonical_without_renumbering_surfaces() {
    let iri = "https://example.com/page?id=7";
    let alias = "https://EXAMPLE.COM:443/page?utm_source=news&id=7#part";
    let resource = ResourceNode::new(alias);
    assert_eq!(resource.canonical_iri(), iri);
    assert_eq!(resource.id(), chartulary::resource_id(iri));
    assert_eq!(*Identified::id(&resource), resource.id());
    assert_eq!(resource.primary_address(), Some(Address::new(iri)));
    assert_eq!(resource, ResourceNode::new(iri));
    assert_ne!(
        resource.id(),
        ResourceNode::new("https://example.com/page?id=8").id()
    );
    assert_ne!(resource.id(), Graph::node_namespace_id(iri));

    let first = SurfaceNode::test_stub(iri);
    let second = SurfaceNode::test_stub(alias);
    assert_ne!(first.id, second.id);
    assert_eq!(
        ResourceNode::new(first.url()),
        ResourceNode::new(second.url())
    );
}

#[test]
fn vocabulary_term_identity_preserves_fragments_query_and_case() {
    let cat = ResourceNode::for_term("https://vocab.test/concepts#Cat");
    for iri in [
        "https://vocab.test/concepts#Dog",
        "https://vocab.test/concepts#cat",
        "https://VOCAB.test/concepts#Cat",
        "https://vocab.test/concepts?edition=1#Cat",
    ] {
        let term = ResourceNode::for_term(iri);
        assert_eq!(term.canonical_iri(), iri);
        assert_eq!(term.id(), chartulary::resource_id_from_canonical_iri(iri));
        assert_ne!(term.id(), cat.id());
    }
    assert_eq!(cat, ResourceNode::for_term(cat.canonical_iri()));
    assert_eq!(
        ResourceNode::new("https://vocab.test/concepts#Cat"),
        ResourceNode::new("https://VOCAB.test:443/concepts#Dog")
    );
    assert_ne!(cat.id(), ResourceNode::new(cat.canonical_iri()).id());
}

#[test]
fn exact_term_records_and_resource_edge_endpoints_survive_checked_load() {
    use crate::graph::{EdgePayload, SemanticStatement, SemanticSubKind};
    let mut graph = Graph::new();
    let terms = [
        ResourceNode::for_term("https://vocab.test/concepts#Cat"),
        ResourceNode::for_term("https://vocab.test/concepts#Dog"),
    ];
    for term in &terms {
        assert!(graph.set_resource_record(
            term.id(),
            Some(PersistedResourceRecord {
                canonical_iri: term.canonical_iri().into(),
                facets: vec![],
            })
        ));
    }
    let mut payload = EdgePayload::new();
    payload.push_persisted_semantic_statement(SemanticStatement {
        statement_id: "exact-vocabulary-assertion".into(),
        predicate: crate::graph::predicate_iri(SemanticSubKind::SameEntityAs).into(),
        recognized_sub_kind: Some(SemanticSubKind::SameEntityAs),
        label: None,
        graph_scope: crate::types::GraphScope::Default,
        provenance_iri: Some("https://author.test/".into()),
        asserted_at_ms: Some(19),
        first_asserted_ms: Some(19),
        withdrawn: None,
        basis: Default::default(),
    });
    let edge =
        super::super::snapshot::persisted_edge_for_ids(terms[0].id(), terms[1].id(), &payload);
    assert!(graph.set_resource_edges_between(terms[0].id(), terms[1].id(), &[edge]));
    let restored = Graph::try_from_snapshot(&graph.to_snapshot()).unwrap();
    for term in &terms {
        assert_eq!(restored.resource(term.id()), Some(term));
    }
    let edges: Vec<_> = restored.resource_edges().collect();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].0, &terms[0]);
    assert_eq!(edges[0].1, &terms[1]);
    assert_eq!(edges[0].2, &payload);
    assert_eq!(
        restored.edge_count(),
        0,
        "resource edges do not create surface edges"
    );
}

#[test]
fn chartulary_indexes_resources_by_the_shared_identity() {
    let mut graph = chartulary::Graph::<ResourceNode, ()>::new();
    let first = graph.insert(ResourceNode::new("https://example.com/page"));
    let alias = graph.insert(ResourceNode::new("https://EXAMPLE.COM/page#part"));
    assert_eq!(first, alias);
    assert_eq!(graph.node_count(), 1);
    let other = graph.insert(ResourceNode::new("https://example.com/other"));
    assert_ne!(first, other);
    assert_eq!(graph.node_count(), 2);
    let clone = graph.clone();
    assert_eq!(
        clone.get(&chartulary::resource_id("https://example.com/page")),
        graph.node(first)
    );
    assert_eq!(
        clone.get(&chartulary::resource_id("https://example.com/other")),
        graph.node(other)
    );
}
