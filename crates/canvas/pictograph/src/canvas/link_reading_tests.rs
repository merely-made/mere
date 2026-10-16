// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use kernel::graph::{SemanticStatement, SemanticStatementSpec, SemanticSubKind, predicate_iri};
use kernel::types::{AssertionBasis, AssertionMethod, GraphScope};

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
                basis: AssertionBasis::default(),
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
fn explicit_history_reads_first_last_withdrawal_and_basis_but_default_is_held_only() {
    let mut graph = Graph::new();
    let a = Uuid::from_u128(41);
    let b = Uuid::from_u128(42);
    kernel::graph::apply::add_node(
        &mut graph,
        Some(a),
        "https://history-a.test/".into(),
        Default::default(),
    );
    kernel::graph::apply::add_node(
        &mut graph,
        Some(b),
        "https://history-b.test/".into(),
        Default::default(),
    );
    let method = AssertionMethod {
        iri: "urn:method:extractor".into(),
        version: Some("v3".into()),
    };
    let mut add = |id: &str, first, last, withdrawn| {
        graph
            .try_assert_persisted_semantic_statement(
                graph.get_node_key_by_id(a).unwrap(),
                graph.get_node_key_by_id(b).unwrap(),
                SemanticStatement {
                    statement_id: id.into(),
                    predicate: predicate_iri(SemanticSubKind::Cites).into(),
                    recognized_sub_kind: Some(SemanticSubKind::Cites),
                    label: Some(format!("note:{id}")),
                    basis: AssertionBasis {
                        note: Some(format!("note:{id}")),
                        method: Some(method.clone()),
                        evidence: vec!["urn:evidence:b".into(), "urn:evidence:a".into()],
                    },
                    graph_scope: GraphScope::Default,
                    provenance_iri: Some("urn:asserter:history".into()),
                    first_asserted_ms: first,
                    asserted_at_ms: last,
                    withdrawn: None,
                },
            )
            .unwrap();
        if withdrawn {
            assert!(graph.apply_imported_semantic_withdrawal_for_pair(
                graph.get_node_key_by_id(a).unwrap(),
                graph.get_node_key_by_id(b).unwrap(),
                id,
                Some(25),
                Some("urn:withdrawer".into()),
            ));
        }
    };
    add("held", Some(5), Some(15), false);
    add("withdrawn", Some(10), Some(20), true);
    add("undated", None, None, false);

    let source = LinkSource::Unavailable {
        reason: "history fixture has no source identity".into(),
    };
    let held = read_link(&graph, a, b, source.clone()).unwrap();
    let history = read_link_history(&graph, a, b, source).unwrap();
    let collect = |reading: &LinkPairReading| {
        reading
            .links
            .iter()
            .flat_map(|link| &link.buckets)
            .flat_map(|bucket| &bucket.assertions)
            .cloned()
            .collect::<Vec<_>>()
    };
    let held = collect(&held);
    let history = collect(&history);
    assert_eq!(held.len(), 2);
    assert!(held.iter().all(|assertion| assertion.withdrawn.is_none()));
    assert_eq!(history.len(), 3);
    let removed = history
        .iter()
        .find(|assertion| assertion.id == "withdrawn")
        .unwrap();
    assert_eq!(removed.first_asserted_ms, Some(10));
    assert_eq!(removed.asserted_at_ms, Some(20));
    assert_eq!(
        removed.withdrawn,
        Some(LinkWithdrawal {
            at_ms: Some(25),
            by: Some("urn:withdrawer".into()),
        })
    );
    assert_eq!(removed.basis.note.as_deref(), Some("note:withdrawn"));
    assert_eq!(removed.basis.method.as_ref().unwrap().iri, method.iri);
    assert_eq!(
        removed.basis.method.as_ref().unwrap().version.as_deref(),
        Some("v3")
    );
    assert_eq!(
        removed.basis.evidence,
        vec!["urn:evidence:b", "urn:evidence:a"]
    );
    assert!(
        history
            .iter()
            .any(|assertion| assertion.first_asserted_ms.is_none())
    );
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

    let mut legacy_semantic = EdgePayload::new();
    legacy_semantic.semantic = Some(kernel::graph::edge_data::SemanticData {
        label: Some("legacy note".into()),
        predicate: Some("urn:predicate:legacy".into()),
        ..Default::default()
    });
    let (_, summaries) = legacy_data(&legacy_semantic, true);
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].family, "Semantic");
    assert_eq!(summaries[0].data["label"], "legacy note");

    let empty_semantic = EdgePayload {
        semantic: Some(Default::default()),
        ..EdgePayload::new()
    };
    let (_, summaries) = legacy_data(&empty_semantic, true);
    assert!(summaries.is_empty());
}
