// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Independent assertion identities and legacy attribution.

use super::super::*;
use crate::graph::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI;

fn claim(asserter: &str, time: u64) -> SemanticStatementSpec {
    SemanticStatementSpec {
        predicate: predicate_iri(SemanticSubKind::Cites).to_string(),
        recognized_sub_kind: Some(SemanticSubKind::Cites),
        graph_scope: GraphScope::User,
        provenance_iri: Some(asserter.to_string()),
        asserted_at_ms: Some(time),
        ..Default::default()
    }
}

#[test]
fn separate_asserters_keep_their_own_assertions_and_retract_alone() {
    let mut graph = Graph::new();
    let source = graph.add_node("https://source.test".to_string(), Point2D::new(0.0, 0.0));
    let target = graph.add_node("https://target.test".to_string(), Point2D::new(1.0, 0.0));
    let alice = claim("https://people.test/alice", 1_000);
    let bob = claim("https://people.test/bob", 2_000);
    let (edge, first) = graph
        .assert_semantic_statement(source, target, alice.clone())
        .unwrap();
    let (same_edge, second) = graph
        .assert_semantic_statement(source, target, bob.clone())
        .unwrap();
    assert_eq!(edge, same_edge, "both assertions share one pair bucket");
    assert_ne!(first.statement_id, second.statement_id);
    let statements = graph.get_edge(edge).unwrap().semantic_statements();
    assert_eq!(statements.len(), 2);
    assert_eq!(statements[0].provenance_iri, alice.provenance_iri);
    assert_eq!(statements[1].provenance_iri, bob.provenance_iri);
    assert_eq!(statements[0].asserted_at_ms, Some(1_000));
    assert_eq!(statements[1].asserted_at_ms, Some(2_000));
    let bob_before_update = statements[1].clone();

    let (_, unchanged) = graph
        .assert_semantic_statement(source, target, alice.clone())
        .unwrap();
    assert_eq!(unchanged.statement_id, first.statement_id);
    assert!(!unchanged.changed);
    let (_, updated) = graph
        .assert_semantic_statement(
            source,
            target,
            SemanticStatementSpec {
                label: Some("checked again".to_string()),
                asserted_at_ms: Some(3_000),
                ..alice
            },
        )
        .unwrap();
    assert!(updated.changed, "the same asserter can update its record");
    assert_eq!(updated.statement_id, first.statement_id);
    let statements = graph.get_edge(edge).unwrap().semantic_statements();
    assert_eq!(statements.len(), 2);
    assert_eq!(statements[0].label.as_deref(), Some("checked again"));
    assert_eq!(statements[0].asserted_at_ms, Some(3_000));
    assert_eq!(statements[1], bob_before_update);

    assert!(graph.retract_semantic_statement(source, target, &first.statement_id));
    assert!(!graph.retract_semantic_statement(source, target, &first.statement_id));
    let statements = graph.get_edge(edge).unwrap().semantic_statements();
    assert_eq!(statements, &[bob_before_update]);
    assert!(graph.retract_semantic_statement(source, target, &second.statement_id));
    assert!(graph.find_edge_key(source, target).is_none());
}

#[test]
fn legacy_insert_api_deduplicates_by_asserter_too() {
    let mut data = SemanticData::default();
    let insert = |data: &mut SemanticData, asserter: &str, time: u64| {
        data.insert_statement(
            Some(SemanticSubKind::Cites),
            predicate_iri(SemanticSubKind::Cites).to_string(),
            None,
            GraphScope::User,
            Some(asserter.to_string()),
            Some(time),
        )
    };
    assert!(insert(&mut data, "https://people.test/alice", 100));
    assert!(insert(&mut data, "https://people.test/bob", 200));
    assert_eq!(data.statements.len(), 2);
    let first_id = data.statements[0].statement_id.clone();
    let second = data.statements[1].clone();
    assert_ne!(first_id, second.statement_id);
    assert!(!insert(&mut data, "https://people.test/alice", 100));
    assert!(insert(&mut data, "https://people.test/alice", 300));
    assert_eq!(data.statements.len(), 2);
    assert_eq!(data.statements[0].statement_id, first_id);
    assert_eq!(data.statements[0].asserted_at_ms, Some(300));
    assert_eq!(data.statements[1], second);
}

#[test]
fn legacy_attribution_preserves_known_provenance_ids_and_times() {
    let mut unknown = SemanticStatement {
        statement_id: "urn:mere:statement:legacy".to_string(),
        predicate: predicate_iri(SemanticSubKind::Cites).to_string(),
        recognized_sub_kind: Some(SemanticSubKind::Cites),
        label: Some("historic claim".to_string()),
        graph_scope: GraphScope::Source,
        provenance_iri: None,
        asserted_at_ms: Some(77),
    };
    let mut expected = unknown.clone();
    expected.provenance_iri = Some(UNKNOWN_LEGACY_ASSERTER_IRI.to_string());
    assert!(unknown.normalize_legacy_asserter());
    assert_eq!(unknown, expected);
    assert!(
        !unknown.normalize_legacy_asserter(),
        "normalization is idempotent"
    );

    let mut known = SemanticStatement {
        provenance_iri: Some("https://people.test/alice".to_string()),
        ..expected
    };
    let known_before = known.clone();
    assert!(!known.normalize_legacy_asserter());
    assert_eq!(
        known, known_before,
        "known attribution is the positive control"
    );

    let mut data = SemanticData::default();
    assert!(data.push_persisted_statement(unknown.clone()));
    let asserted = data.assert_statement(SemanticStatementSpec {
        predicate: unknown.predicate,
        recognized_sub_kind: unknown.recognized_sub_kind,
        graph_scope: unknown.graph_scope,
        provenance_iri: known.provenance_iri,
        asserted_at_ms: Some(88),
        label: None,
    });
    assert_ne!(asserted.statement_id, unknown.statement_id);
    assert_eq!(
        data.statements.len(),
        2,
        "a known asserter keeps its own record"
    );
    assert_eq!(data.statements[0].asserted_at_ms, Some(77));
}
