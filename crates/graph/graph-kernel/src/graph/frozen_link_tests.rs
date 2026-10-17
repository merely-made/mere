// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::graph::apply::add_node;
use crate::persistence::PersistedResourceFacet;
use crate::types::{AssertionBasis, GraphScope};
use std::sync::{Arc, Mutex};

fn available_source() -> FrozenLinkSource {
    FrozenLinkSource::Available {
        binding: FrozenSourceBinding {
            authority: "mere".into(),
            domain: "graph-session".into(),
            resource: "session:11111111-1111-4111-8111-111111111111".into(),
        },
        revision: FrozenSourceRevision::Available {
            revision: "sha256:source-snapshot".into(),
        },
    }
}

fn add_surface_pair(graph: &mut Graph) -> (NodeKey, NodeKey) {
    let first = add_node(
        graph,
        None,
        "https://frozen.test/first".into(),
        Default::default(),
    );
    let second = add_node(
        graph,
        None,
        "https://frozen.test/second".into(),
        Default::default(),
    );
    (first, second)
}

fn claim(graph: &mut Graph, from: NodeKey, to: NodeKey, label: &str) -> String {
    graph
        .try_assert_semantic_statement(
            from,
            to,
            SemanticStatementSpec {
                predicate: predicate_iri(SemanticSubKind::UserGrouped).into(),
                recognized_sub_kind: Some(SemanticSubKind::UserGrouped),
                label: Some(label.into()),
                graph_scope: GraphScope::User,
                provenance_iri: Some("urn:author:test".into()),
                asserted_at_ms: Some(123),
                basis: AssertionBasis {
                    note: Some(format!("reason: {label}")),
                    evidence: vec![format!("urn:evidence:{label}")],
                    ..Default::default()
                },
            },
        )
        .unwrap()
        .1
        .statement_id
}

#[test]
fn freezes_surface_pair_buckets_as_independent_immutable_resource_facets() {
    let mut graph = Graph::new();
    let (first, second) = add_surface_pair(&mut graph);
    let first_id = graph.get_node(first).unwrap().id;
    let second_id = graph.get_node(second).unwrap().id;
    let forward = claim(&mut graph, first, second, "forward");
    let reverse = claim(&mut graph, second, first, "reverse");
    let mut parallel = EdgePayload::new();
    parallel.push_traversal(Traversal {
        timestamp_ms: 456,
        trigger: NavigationTrigger::Programmatic,
    });
    graph.inner.connect(first, second, parallel);
    let source = available_source();
    let source_revision = source.clone();

    let mut replay = graph.clone();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let captured_sink = captured.clone();
    graph.set_recorder(Some(Arc::new(move |delta| {
        captured_sink.lock().unwrap().push(delta.clone());
    })));
    let owner = graph
        .freeze_link_with_nonce(
            GraphStratum::Surface,
            second_id,
            first_id,
            source,
            Uuid::from_u128(101),
        )
        .unwrap();
    let frozen = graph.open_frozen_link(owner).unwrap().link.clone();
    assert_eq!(frozen.stratum(), GraphStratum::Surface);
    assert_eq!(frozen.first().id, first_id.min(second_id));
    assert_eq!(frozen.second().id, first_id.max(second_id));
    assert_eq!(frozen.source(), &source_revision);
    assert_eq!(
        frozen.statements().len(),
        3,
        "parallel and antiparallel pair buckets are retained"
    );
    assert!(
        frozen
            .statements()
            .iter()
            .any(|edge| edge.traversal.is_some()),
        "traversal data is copied too"
    );
    let ids = frozen
        .statements()
        .iter()
        .flat_map(PersistedEdge::retained_assertions)
        .map(|statement| statement.statement_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        ids,
        [forward.as_str(), reverse.as_str()].into_iter().collect()
    );

    assert!(graph.retract_assertion(&forward));
    assert_eq!(graph.resource(owner).unwrap().nested_link(), Some(&frozen));
    assert_eq!(
        Graph::try_from_snapshot(&graph.to_snapshot())
            .unwrap()
            .resource(owner)
            .unwrap()
            .nested_link(),
        Some(&frozen),
        "snapshot reopen retains the immutable copy"
    );
    capture::replay_captured_deltas_onto(&mut replay, captured.lock().unwrap().clone());
    assert_eq!(replay.resource(owner).unwrap().nested_link(), Some(&frozen));
    assert_eq!(
        replay
            .open_frozen_link(owner)
            .unwrap()
            .link
            .statements()
            .len(),
        3
    );
    let current = graph
        .read_link_snapshot(
            GraphStratum::Surface,
            first_id,
            second_id,
            available_source(),
        )
        .unwrap();
    let independent = graph
        .freeze_link_with_nonce(
            GraphStratum::Surface,
            first_id,
            second_id,
            available_source(),
            Uuid::from_u128(107),
        )
        .unwrap();
    assert_ne!(owner, independent);
    assert_eq!(
        graph.resource(independent).unwrap().nested_link(),
        Some(&current),
        "freezing the same pair again mints an independent immutable Resource"
    );
    assert!(
        graph
            .freeze_link_with_nonce(
                GraphStratum::Surface,
                first_id,
                second_id,
                available_source(),
                Uuid::from_u128(101),
            )
            .is_err(),
        "a reused nonce cannot replace the first frozen reading"
    );
}

#[test]
fn resource_freeze_is_separate_and_receiver_reports_missing_references() {
    let mut graph = Graph::new();
    let (first_surface, second_surface) = add_surface_pair(&mut graph);
    let first = graph.shown_resource_id(first_surface).unwrap();
    let second = graph.shown_resource_id(second_surface).unwrap();
    let (_, resource_claim) = graph
        .try_assert_semantic_statement_by_resource_ids(
            first,
            second,
            SemanticStatementSpec {
                predicate: predicate_iri(SemanticSubKind::Cites).into(),
                recognized_sub_kind: Some(SemanticSubKind::Cites),
                label: Some("resource claim".into()),
                graph_scope: GraphScope::User,
                provenance_iri: Some("urn:author:test".into()),
                asserted_at_ms: Some(88),
                ..Default::default()
            },
        )
        .unwrap();
    let owner = graph
        .freeze_link_with_nonce(
            GraphStratum::Resource,
            first,
            second,
            FrozenLinkSource::Unavailable {
                reason: "public source revision is not available".into(),
            },
            Uuid::from_u128(102),
        )
        .unwrap();
    let link = graph.open_frozen_link(owner).unwrap().link.clone();
    assert_eq!(link.stratum(), GraphStratum::Resource);
    assert_eq!(link.first().resource, Some(link.first().id));
    assert_eq!(link.second().resource, Some(link.second().id));
    assert!(matches!(
        link.source(),
        FrozenLinkSource::Unavailable { reason } if reason.contains("revision")
    ));
    assert_eq!(
        link.statements()[0].retained_assertions()[0].statement_id,
        resource_claim.statement_id
    );

    let record = graph.resource_record(owner).unwrap();
    let mut receiver = Graph::new();
    assert!(receiver.set_resource_record(owner, Some(record)));
    assert_eq!(
        receiver.resource_nodes().count(),
        1,
        "opening does not invent endpoints"
    );
    let opened = receiver.open_frozen_link(owner).unwrap();
    assert_eq!(opened.link, &link);
    assert!(
        opened
            .coverage
            .limits
            .iter()
            .any(|limit| { limit.layer == CoverageLayer::Possession && limit.count == Some(2) })
    );
    assert_eq!(
        receiver.freeze_link_with_nonce(
            GraphStratum::Resource,
            first,
            second,
            available_source(),
            Uuid::from_u128(103),
        ),
        Err("missing first Resource endpoint".into())
    );
}

#[test]
fn refuses_empty_pairs_invalid_source_and_mutation_or_removal_of_frozen_owner() {
    let mut graph = Graph::new();
    let (first, second) = add_surface_pair(&mut graph);
    let first_id = graph.get_node(first).unwrap().id;
    let second_id = graph.get_node(second).unwrap().id;
    let empty = graph
        .read_link_snapshot(
            GraphStratum::Surface,
            first_id,
            second_id,
            available_source(),
        )
        .unwrap();
    assert!(empty.statements().is_empty());
    assert!(
        graph
            .freeze_link_with_nonce(
                GraphStratum::Surface,
                first_id,
                second_id,
                available_source(),
                Uuid::from_u128(104),
            )
            .is_err()
    );
    assert!(
        graph
            .freeze_link_with_nonce(
                GraphStratum::Surface,
                first_id,
                second_id,
                FrozenLinkSource::Unavailable { reason: " ".into() },
                Uuid::from_u128(105),
            )
            .is_err()
    );
    assert!(
        graph
            .freeze_link_with_nonce(
                GraphStratum::Surface,
                first_id,
                second_id,
                available_source(),
                Uuid::nil(),
            )
            .is_err()
    );

    claim(&mut graph, first, second, "held");
    let owner = graph
        .freeze_link_with_nonce(
            GraphStratum::Surface,
            first_id,
            second_id,
            available_source(),
            Uuid::from_u128(106),
        )
        .unwrap();
    let record = graph.resource_record(owner).unwrap();
    let revision = graph.revision();
    let mut duplicate_id = record.clone();
    let facet = duplicate_id
        .facets
        .iter_mut()
        .find(|f| f.facet == FROZEN_LINK)
        .unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&facet.value_json).unwrap();
    let assertions = value["statements"][0]["assertions"].as_array_mut().unwrap();
    let duplicate = assertions[0].clone();
    assertions.push(duplicate);
    facet.value_json = value.to_string();
    assert!(
        frozen_link::validate_frozen_link_record(&duplicate_id)
            .unwrap_err()
            .contains("duplicate frozen assertion handle")
    );

    let mut changed = record.clone();
    let facet = changed
        .facets
        .iter_mut()
        .find(|f| f.facet == FROZEN_LINK)
        .unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&facet.value_json).unwrap();
    value["source"]["binding"]["authority"] = serde_json::json!("other");
    facet.value_json = value.to_string();
    assert!(!graph.set_resource_record(owner, Some(changed)));
    assert_eq!(graph.resource_record(owner), Some(record));
    assert_eq!(graph.revision(), revision);

    let mut unknown = graph.resource_record(owner).unwrap();
    let facet = unknown
        .facets
        .iter_mut()
        .find(|f| f.facet == FROZEN_LINK)
        .unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&facet.value_json).unwrap();
    value["coverage"]["unrecognized"] = serde_json::json!(true);
    facet.value_json = value.to_string();
    assert!(!graph.set_resource_record(owner, Some(unknown)));

    let mut duplicate = graph.resource_record(owner).unwrap();
    duplicate.facets.push(PersistedResourceFacet {
        facet: FROZEN_LINK.into(),
        value_json: "{}".into(),
    });
    assert!(!graph.set_resource_record(owner, Some(duplicate)));

    assert!(graph.set_resource_record(owner, None));
    assert!(
        graph.resource(owner).is_none(),
        "whole Resource deletion remains distinct from facet replacement"
    );
}
