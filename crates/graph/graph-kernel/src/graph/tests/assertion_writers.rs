// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Attribution at statement writer entry points.

use super::super::*;

fn pair(graph: &mut Graph) -> (NodeKey, NodeKey) {
    let from = graph.add_node("https://from.test/".into(), Point2D::new(0.0, 0.0));
    let to = graph.add_node("https://to.test/".into(), Point2D::new(1.0, 0.0));
    (from, to)
}

fn spec() -> SemanticStatementSpec {
    SemanticStatementSpec {
        predicate: predicate_iri(SemanticSubKind::Cites).into(),
        recognized_sub_kind: Some(SemanticSubKind::Cites),
        graph_scope: GraphScope::User,
        asserted_at_ms: Some(100),
        ..Default::default()
    }
}

fn assertion() -> EdgeAssertion {
    EdgeAssertion::Semantic {
        sub_kind: SemanticSubKind::Cites,
        label: None,
        decay_progress: None,
    }
}

fn persisted() -> SemanticStatement {
    SemanticStatement::new(
        predicate_iri(SemanticSubKind::Cites).into(),
        Some(SemanticSubKind::Cites),
        None,
        GraphScope::Source,
        None,
        Some(100),
    )
}

#[test]
fn statement_writers_all_record_the_current_author() {
    type Writer = fn(&mut Graph, NodeKey, NodeKey);
    let writers: [(&str, Writer); 9] = [
        ("Graph::assert_semantic_statement", |graph, from, to| {
            graph.assert_semantic_statement(from, to, spec()).unwrap();
        }),
        (
            "Graph::assert_persisted_semantic_statement",
            |graph, from, to| {
                graph
                    .assert_persisted_semantic_statement(from, to, persisted())
                    .unwrap();
            },
        ),
        ("Graph::assert_relation", |graph, from, to| {
            graph.assert_relation(from, to, assertion()).unwrap();
        }),
        (
            "Graph::assert_semantic_relation_in_scope",
            |graph, from, to| {
                graph
                    .assert_semantic_relation_in_scope(
                        from,
                        to,
                        SemanticSubKind::Cites,
                        None,
                        GraphScope::User,
                    )
                    .unwrap();
            },
        ),
        ("Graph::assert_semantic_predicate", |graph, from, to| {
            graph
                .assert_semantic_predicate(from, to, "https://vocab.test/mentions".into())
                .unwrap();
        }),
        (
            "Graph::assert_semantic_predicate_in_scope",
            |graph, from, to| {
                graph
                    .assert_semantic_predicate_in_scope(
                        from,
                        to,
                        "https://vocab.test/mentions".into(),
                        GraphScope::Source,
                    )
                    .unwrap();
            },
        ),
        ("apply::assert_relation", |graph, from, to| {
            apply::assert_relation(graph, from, to, assertion()).unwrap();
        }),
        (
            "apply::assert_semantic_relation_in_scope",
            |graph, from, to| {
                apply::assert_semantic_relation_in_scope(
                    graph,
                    from,
                    to,
                    SemanticSubKind::Cites,
                    None,
                    GraphScope::User,
                )
                .unwrap();
            },
        ),
        (
            "apply::assert_semantic_predicate_in_scope",
            |graph, from, to| {
                apply::assert_semantic_predicate_in_scope(
                    graph,
                    from,
                    to,
                    "https://vocab.test/mentions".into(),
                    GraphScope::Source,
                )
                .unwrap();
            },
        ),
    ];
    let author = Author::engine("writer-audit", "1");
    let expected = author.asserter_iri();
    assert_ne!(expected, Author::user().asserter_iri());
    for (name, write) in writers {
        let mut graph = Graph::new();
        let (from, to) = pair(&mut graph);
        graph.write_as(author.clone(), |graph| write(graph, from, to));
        let edge = graph.find_edge_key(from, to).expect(name);
        let statements = graph.get_edge(edge).unwrap().semantic_statements();
        assert_eq!(statements.len(), 1, "{name} must produce a real assertion");
        assert_eq!(
            statements[0].provenance_iri.as_deref(),
            Some(expected.as_str()),
            "{name}"
        );
    }
}

#[test]
fn new_engine_version_updates_its_assertion_and_other_author_stays_separate() {
    let mut graph = Graph::new();
    let (from, to) = pair(&mut graph);
    let (_, first) = graph.write_as(Author::engine("extractor", "1"), |graph| {
        graph.assert_semantic_statement(from, to, spec()).unwrap()
    });
    let (edge, second) = graph.write_as(Author::engine("other-extractor", "1"), |graph| {
        graph.assert_semantic_statement(from, to, spec()).unwrap()
    });
    assert_ne!(first.statement_id, second.statement_id);
    let other_before = graph.get_edge(edge).unwrap().semantic_statements()[1].clone();
    let (_, updated) = graph.write_as(Author::engine("extractor", "2"), |graph| {
        graph
            .assert_semantic_statement(
                from,
                to,
                SemanticStatementSpec {
                    label: Some("new extraction".into()),
                    asserted_at_ms: Some(200),
                    ..spec()
                },
            )
            .unwrap()
    });
    assert_eq!(first.statement_id, updated.statement_id);
    assert!(updated.changed);
    let statements = graph.get_edge(edge).unwrap().semantic_statements();
    assert_eq!(statements.len(), 2);
    assert_eq!(statements[0].asserted_at_ms, Some(200));
    assert_eq!(statements[1], other_before);
}

#[test]
fn explicit_source_overrides_writer_fallback_on_both_statement_apis() {
    let mut graph = Graph::new();
    let (from, to) = pair(&mut graph);
    let source = "https://source.test/page";
    let writer = Author::engine("extractor", "1");
    let writer_iri = writer.asserter_iri();
    graph.write_as(writer, |graph| {
        let explicit = SemanticStatementSpec {
            provenance_iri: Some(source.into()),
            ..spec()
        };
        graph.assert_semantic_statement(from, to, explicit).unwrap();
        let mut statement = persisted();
        statement.provenance_iri = Some(source.into());
        graph
            .assert_persisted_semantic_statement(from, to, statement)
            .unwrap();
        graph.assert_semantic_statement(from, to, spec()).unwrap();
    });
    let edge = graph.find_edge_key(from, to).unwrap();
    let statements = graph.get_edge(edge).unwrap().semantic_statements();
    assert_eq!(statements.len(), 3);
    assert_eq!(statements[0].provenance_iri.as_deref(), Some(source));
    assert_eq!(statements[1].provenance_iri.as_deref(), Some(source));
    assert_eq!(
        statements[2].provenance_iri.as_deref(),
        Some(writer_iri.as_str())
    );
}

#[test]
fn writer_scope_restores_after_nested_returns_and_panic() {
    let mut graph = Graph::new();
    let original = graph.write_author().clone();
    let outer = Author::rule("outer", "1");
    let inner = Author::script("inner", "1");
    let returned = graph.write_as(outer.clone(), |graph| {
        assert_eq!(graph.write_author(), &outer);
        graph.write_as(inner.clone(), |graph| {
            assert_eq!(graph.write_author(), &inner);
        });
        assert_eq!(graph.write_author(), &outer);
        42
    });
    assert_eq!(returned, 42);
    assert_eq!(graph.write_author(), &original);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        graph.write_as(inner.clone(), |graph| {
            assert_eq!(graph.write_author(), &inner);
            panic!("scope restoration control");
        });
    }));
    assert!(
        result.is_err(),
        "the panic occurred inside the writer scope"
    );
    assert_eq!(graph.write_author(), &original);
    let (from, to) = pair(&mut graph);
    let (edge, _) = graph.assert_semantic_statement(from, to, spec()).unwrap();
    assert_eq!(
        graph.get_edge(edge).unwrap().semantic_statements()[0].provenance_iri,
        Some(original.asserter_iri()),
        "a real write after the panic uses the restored author"
    );
}
