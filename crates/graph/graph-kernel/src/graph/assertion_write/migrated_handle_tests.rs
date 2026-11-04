// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn ambiguous_content_requires_exact_id_and_updates_reach_the_one_row() {
    let mut graph = Graph::new();
    let a = graph.add_node_with_id(
        Uuid::from_u128(106),
        "https://example.org/a".into(),
        Default::default(),
    );
    let b = graph.add_node_with_id(
        Uuid::from_u128(107),
        "https://example.org/b".into(),
        Default::default(),
    );
    let spec = SemanticStatementSpec {
        predicate: "urn:predicate:ambiguous".into(),
        provenance_iri: Some("urn:author:held".into()),
        ..Default::default()
    };
    let (_, first) = graph
        .try_assert_semantic_statement(a, b, spec.clone())
        .unwrap();
    let pair = (
        graph.shown_resource_id(a).unwrap(),
        graph.shown_resource_id(b).unwrap(),
    );
    let mut edges = graph.persisted_resource_edges_between(pair.0, pair.1);
    let mut other = edges[0].retained_assertions()[0].clone();
    other.statement_id = "second-exact-handle".into();
    edges[0].assertions.as_mut().unwrap().push(other.clone());
    assert!(graph.set_resource_edges_between(pair.0, pair.1, &edges));
    assert!(Graph::try_from_snapshot(&graph.to_snapshot()).is_ok());
    let before = serde_json::to_value(graph.to_snapshot()).unwrap();
    assert!(
        matches!(graph.try_assert_semantic_statement(a, b, spec.clone()), Err(StatementWriteError::AmbiguousAssertion { statement_ids, .. }) if statement_ids.len() == 2)
    );
    assert_eq!(serde_json::to_value(graph.to_snapshot()).unwrap(), before);
    assert!(
        matches!(graph.try_assert_semantic_statement_by_resource_ids(pair.0, pair.1, spec.clone()),
        Err(StatementWriteError::AmbiguousAssertion { statement_ids, .. }) if statement_ids.len() == 2)
    );
    assert_eq!(serde_json::to_value(graph.to_snapshot()).unwrap(), before);
    let mut precise = graph
        .find_semantic_statement(&other.statement_id)
        .unwrap()
        .1
        .clone();
    precise.label = Some("precise edit".into());
    precise.basis.note = Some("precise edit".into());
    precise.asserted_at_ms = Some(99);
    assert!(
        graph
            .try_assert_persisted_semantic_statement(a, b, precise)
            .is_ok()
    );
    assert_eq!(
        graph
            .find_semantic_statement(&first.statement_id)
            .unwrap()
            .1
            .label,
        None
    );
    let copies: Vec<_> = graph
        .resource_relations()
        .flat_map(|(_, _, _, payload)| payload.semantic_statements().iter())
        .filter(|statement| statement.statement_id == other.statement_id)
        .collect();
    assert_eq!(copies.len(), 1);
    assert!(copies.iter().all(
        |statement| statement.label.as_deref() == Some("precise edit")
            && statement.asserted_at_ms == Some(99)
    ));
    assert!(Graph::try_from_snapshot(&graph.to_snapshot()).is_ok());
    assert!(!graph.update_assertion_metadata(
        &other.statement_id,
        Some("precise edit".into()),
        Some(99)
    ));
    assert!(!graph.update_assertion_metadata("absent", None, None));
    assert!(graph.retract_assertion(&first.statement_id));
    assert!(
        graph
            .find_semantic_statement(&first.statement_id)
            .unwrap()
            .1
            .withdrawn
            .is_some()
    );
    let withdrawn_copies: Vec<_> = graph
        .resource_relations()
        .flat_map(|(_, _, _, payload)| payload.semantic_statements().iter())
        .filter(|statement| statement.statement_id == first.statement_id)
        .collect();
    assert_eq!(withdrawn_copies.len(), 1);
    assert!(
        withdrawn_copies
            .iter()
            .all(|statement| statement.withdrawn.is_some())
    );
    assert!(graph.find_semantic_statement(&other.statement_id).is_some());
    assert!(Graph::try_from_snapshot(&graph.to_snapshot()).is_ok());
    assert!(
        graph.try_assert_semantic_statement(a, b, spec).is_ok(),
        "the remaining content key is unambiguous"
    );
    let copies: Vec<_> = graph
        .resource_relations()
        .flat_map(|(_, _, _, payload)| payload.semantic_statements().iter())
        .filter(|statement| statement.statement_id == other.statement_id)
        .collect();
    assert_eq!(copies.len(), 1);
    assert!(
        copies
            .iter()
            .all(|statement| statement.label.is_none() && statement.asserted_at_ms == Some(99))
    );
    assert!(Graph::try_from_snapshot(&graph.to_snapshot()).is_ok());
}
