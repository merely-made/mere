// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::super::*;
use crate::graph::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI;

#[test]
fn legacy_statement_load_preserves_ids_times_and_known_attribution() {
    let mut graph = Graph::new();
    let a = graph.add_node("https://a.test/".into(), Point2D::new(0.0, 0.0));
    let b = graph.add_node("https://b.test/".into(), Point2D::new(0.0, 0.0));
    for (asserter, time) in [
        ("https://author.test/first", 10),
        ("https://author.test/second", 20),
    ] {
        graph
            .assert_surface_semantic_statement(
                a,
                b,
                SemanticStatementSpec {
                    predicate: "https://schema.org/citation".into(),
                    recognized_sub_kind: Some(SemanticSubKind::Cites),
                    label: None,
                    graph_scope: GraphScope::Default,
                    provenance_iri: Some(asserter.into()),
                    asserted_at_ms: Some(time),
                },
            )
            .unwrap();
    }
    let mut snapshot = graph.to_snapshot();
    let statements = &mut snapshot.edges[0].semantic.as_mut().unwrap().statements;
    assert_eq!(statements.len(), 2);
    statements[0].provenance_iri = None;
    let mut another_legacy = statements[0].clone();
    another_legacy.statement_id = "another-legacy-handle".into();
    another_legacy.asserted_at_ms = Some(11);
    statements.push(another_legacy);
    let mut expected = statements.clone();
    for statement in &mut expected {
        if statement.provenance_iri.is_none() {
            statement.provenance_iri = Some(UNKNOWN_LEGACY_ASSERTER_IRI.into());
        }
    }
    let restored = Graph::from_snapshot(&snapshot).to_snapshot();
    assert_eq!(
        restored.edges[0].semantic.as_ref().unwrap().statements,
        expected
    );
    assert_eq!(
        Graph::from_snapshot(&restored).to_snapshot().edges,
        restored.edges
    );
}

#[test]
fn aggregate_only_snapshots_receive_unknown_attribution_for_both_predicate_forms() {
    for recognized in [false, true] {
        let mut graph = Graph::new();
        let a = graph.add_node("https://a.test/".into(), Point2D::new(0.0, 0.0));
        let b = graph.add_node("https://b.test/".into(), Point2D::new(0.0, 0.0));
        let mut snapshot = graph.to_snapshot();
        graph.assert_surface_semantic_statement(
            a,
            b,
            SemanticStatementSpec {
                predicate: if recognized {
                    predicate_iri(SemanticSubKind::Cites).into()
                } else {
                    "https://example.test/relates".into()
                },
                recognized_sub_kind: recognized.then_some(SemanticSubKind::Cites),
                label: Some("old citation".into()),
                ..Default::default()
            },
        );
        snapshot.edges = graph.to_snapshot().edges;
        snapshot.edges[0]
            .semantic
            .as_mut()
            .unwrap()
            .statements
            .clear();
        let restored = Graph::from_snapshot(&snapshot).to_snapshot();
        let statements = &restored.edges[0].semantic.as_ref().unwrap().statements;
        assert!(!statements.is_empty());
        assert!(
            statements
                .iter()
                .all(|s| s.provenance_iri.as_deref() == Some(UNKNOWN_LEGACY_ASSERTER_IRI))
        );
        assert!(statements.iter().all(|s| !s.statement_id.is_empty()));
    }
}
