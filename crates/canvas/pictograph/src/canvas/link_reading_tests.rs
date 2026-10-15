// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use kernel::graph::{SemanticStatementSpec, SemanticSubKind, predicate_iri};
use kernel::types::GraphScope;

#[test]
fn reading_keys_each_stratum_separately_and_preserves_assertion_source_facts() {
    let mut graph = Graph::new();
    let a = Uuid::from_u128(1);
    let b = Uuid::from_u128(2);
    kernel::graph::apply::add_node(
        &mut graph,
        Some(a),
        "https://a.test/".into(),
        Default::default(),
    );
    kernel::graph::apply::add_node(
        &mut graph,
        Some(b),
        "https://b.test/".into(),
        Default::default(),
    );
    let resource_assertion = graph
        .try_assert_semantic_statement(
            graph.get_node_key_by_id(a).unwrap(),
            graph.get_node_key_by_id(b).unwrap(),
            SemanticStatementSpec {
                predicate: predicate_iri(SemanticSubKind::Cites).into(),
                recognized_sub_kind: Some(SemanticSubKind::Cites),
                label: Some("source claim".into()),
                graph_scope: GraphScope::Custom("urn:scope".into()),
                provenance_iri: Some("urn:asserter".into()),
                asserted_at_ms: Some(42),
            },
        )
        .unwrap()
        .1
        .statement_id;
    graph
        .try_assert_semantic_statement(
            graph.get_node_key_by_id(b).unwrap(),
            graph.get_node_key_by_id(a).unwrap(),
            SemanticStatementSpec {
                predicate: predicate_iri(SemanticSubKind::UserGrouped).into(),
                recognized_sub_kind: Some(SemanticSubKind::UserGrouped),
                ..Default::default()
            },
        )
        .unwrap();

    let before_revision = graph.revision();
    let before_rows = graph.projected_relations().count();
    let pair = read_link(
        &graph,
        a,
        b,
        LinkSource::Unavailable {
            reason: "test source omitted".into(),
        },
    )
    .unwrap();
    assert!(matches!(pair.source, LinkSource::Unavailable { .. }));
    assert_eq!(graph.revision(), before_revision);
    assert_eq!(graph.projected_relations().count(), before_rows);
    let resource_a = graph
        .resource(
            graph
                .shown_resource_id(graph.get_node_key_by_id(a).unwrap())
                .unwrap(),
        )
        .unwrap();
    let resource_b = graph
        .resource(
            graph
                .shown_resource_id(graph.get_node_key_by_id(b).unwrap())
                .unwrap(),
        )
        .unwrap();
    let resource_key = LinkKey::new(
        LinkStratum::Resource,
        resource_a.canonical_iri(),
        resource_b.canonical_iri(),
    );
    let surface_key = LinkKey::new(LinkStratum::Surface, a.to_string(), b.to_string());
    assert!(pair.links.iter().any(|link| link.key == resource_key));
    assert!(pair.links.iter().any(|link| link.key == surface_key));
    assert!(
        pair.links
            .iter()
            .all(|link| link.key.stratum == LinkStratum::Resource
                || link.key.stratum == LinkStratum::Surface)
    );

    let member = pair
        .links
        .iter()
        .find(|link| link.key == resource_key)
        .unwrap()
        .buckets
        .iter()
        .flat_map(|bucket| &bucket.assertions)
        .find(|assertion| assertion.id == resource_assertion)
        .unwrap();
    assert_eq!(member.predicate, predicate_iri(SemanticSubKind::Cites));
    assert_eq!(member.recognized_kind.as_deref(), Some("Cites"));
    assert_eq!(member.asserter.as_deref(), Some("urn:asserter"));
    assert_eq!(member.asserted_at_ms, Some(42));
    assert_eq!(member.scope, LinkGraphScope::Custom("urn:scope".into()));

    let known_source = LinkSource::Available {
        binding: scenograph::SourceBinding {
            authority: "mere:test".into(),
            domain: "graph".into(),
            resource: "fixture".into(),
        },
        revision: scenograph::link::LinkSourceRevision::Available {
            revision: scenograph::PublicSourceRevision::new("fixture-r1"),
        },
    };
    let known = read_link(&graph, a, b, known_source.clone()).unwrap();
    assert_eq!(known.source, known_source);
}

#[test]
fn summary_free_read_keeps_traversal_members_without_copying_event_history() {
    let mut payload = EdgePayload::new();
    payload.push_traversal(kernel::graph::edge_data::Traversal {
        timestamp_ms: 9,
        trigger: kernel::graph::edge_taxonomy::NavigationTrigger::LinkClick,
    });

    let (members, summaries) = legacy_data(&payload, false);
    assert!(summaries.is_empty());
    assert!(members.iter().any(|member| member.family == "Traversal"));

    let (_, summaries) = legacy_data(&payload, true);
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].family, "Traversal");
    assert_eq!(summaries[0].data["traversals"].as_array().unwrap().len(), 1);
}
