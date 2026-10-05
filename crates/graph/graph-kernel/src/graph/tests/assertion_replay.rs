// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::super::*;
use crate::graph::apply::{GraphDelta, apply_graph_delta};
use crate::graph::capture::{CapturedDelta, replay_captured_deltas_onto};
use std::sync::{Arc, Mutex};

#[test]
fn assertion_updates_and_precise_retractions_replay_exactly() {
    let mut graph = Graph::new();
    let from = graph.add_node("https://a.test/".into(), Default::default());
    let to = graph.add_node("https://b.test/".into(), Default::default());
    let mut replayed = graph.clone();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let sink = captured.clone();
    graph.set_recorder(Some(Arc::new(move |delta| {
        sink.lock().unwrap().push(delta.clone())
    })));
    let spec = |asserter: &str, time| SemanticStatementSpec {
        predicate: predicate_iri(SemanticSubKind::Cites).into(),
        recognized_sub_kind: Some(SemanticSubKind::Cites),
        graph_scope: crate::types::GraphScope::User,
        provenance_iri: Some(asserter.into()),
        asserted_at_ms: Some(time),
        ..Default::default()
    };
    let (_, alice) = graph
        .assert_semantic_statement(from, to, spec("https://people.test/alice", 10))
        .unwrap();
    let (_, bob) = graph
        .assert_semantic_statement(from, to, spec("https://people.test/bob", 20))
        .unwrap();
    assert_eq!(captured.lock().unwrap().len(), 2);
    graph.assert_semantic_statement(from, to, spec("https://people.test/alice", 10));
    assert_eq!(
        captured.lock().unwrap().len(),
        2,
        "identical reassertion records nothing"
    );
    let (_, updated) = graph
        .assert_semantic_statement(from, to, spec("https://people.test/alice", 30))
        .unwrap();
    assert_eq!(updated.statement_id, alice.statement_id);
    assert!(updated.changed);
    assert!(graph.retract_semantic_statement(from, to, &alice.statement_id));
    assert!(!graph.retract_semantic_statement(from, to, &alice.statement_id));
    let captured = captured.lock().unwrap();
    assert_eq!(
        captured.len(),
        4,
        "same-id update and precise retract are recorded"
    );
    assert!(
        captured
            .iter()
            .all(|delta| matches!(delta, CapturedDelta::ReplaySetEdgesByIds { .. }))
    );
    replay_captured_deltas_onto(&mut replayed, captured.iter().cloned());
    assert_eq!(
        serde_json::to_value(graph.to_snapshot()).unwrap(),
        serde_json::to_value(replayed.to_snapshot()).unwrap()
    );
    let remaining = replayed
        .get_edge(replayed.find_edge_key(from, to).unwrap())
        .unwrap()
        .semantic_statements();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].statement_id, bob.statement_id);
    assert_eq!(remaining[0].asserted_at_ms, Some(20));
}

#[test]
fn predicate_replacement_mints_an_id_and_preserves_the_other_asserter_on_replay() {
    let mut graph = Graph::new();
    let from = graph.add_node("https://a.test/".into(), Default::default());
    let to = graph.add_node("https://b.test/".into(), Default::default());
    let mut replayed = graph.clone();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let sink = captured.clone();
    graph.set_recorder(Some(Arc::new(move |delta| {
        sink.lock().unwrap().push(delta.clone())
    })));
    let spec = |predicate: &str, asserter: &str, time| SemanticStatementSpec {
        predicate: predicate.into(),
        graph_scope: crate::types::GraphScope::User,
        provenance_iri: Some(asserter.into()),
        asserted_at_ms: Some(time),
        ..Default::default()
    };
    let (_, original) = graph
        .assert_semantic_statement(
            from,
            to,
            spec("https://vocab.test/old", "https://people.test/alice", 10),
        )
        .unwrap();
    let (_, other) = graph
        .assert_semantic_statement(
            from,
            to,
            spec("https://vocab.test/old", "https://people.test/bob", 20),
        )
        .unwrap();
    let other_before = graph
        .get_edge(graph.find_edge_key(from, to).unwrap())
        .unwrap()
        .semantic_statements()
        .iter()
        .find(|statement| statement.statement_id == other.statement_id)
        .unwrap()
        .clone();
    assert!(graph.retract_semantic_statement(from, to, &original.statement_id));
    assert!(!graph.retract_semantic_statement(from, to, &original.statement_id));
    let (_, replacement) = graph
        .assert_semantic_statement(
            from,
            to,
            spec("https://vocab.test/new", "https://people.test/alice", 30),
        )
        .unwrap();
    assert_ne!(replacement.statement_id, original.statement_id);
    assert_ne!(replacement.statement_id, other.statement_id);
    let captured = captured.lock().unwrap();
    assert_eq!(
        captured.len(),
        4,
        "two assertions, one retract, one replacement"
    );
    replay_captured_deltas_onto(&mut replayed, captured.iter().cloned());
    assert_eq!(
        serde_json::to_value(graph.to_snapshot()).unwrap(),
        serde_json::to_value(replayed.to_snapshot()).unwrap()
    );
    let statements = replayed
        .get_edge(replayed.find_edge_key(from, to).unwrap())
        .unwrap()
        .semantic_statements();
    assert_eq!(statements.len(), 2);
    assert!(
        !statements
            .iter()
            .any(|statement| statement.statement_id == original.statement_id)
    );
    assert_eq!(
        statements
            .iter()
            .find(|statement| statement.statement_id == other.statement_id),
        Some(&other_before)
    );
    let replaced = statements
        .iter()
        .find(|statement| statement.statement_id == replacement.statement_id)
        .unwrap();
    assert_eq!(replaced.predicate, "https://vocab.test/new");
    assert_eq!(
        replaced.provenance_iri.as_deref(),
        Some("https://people.test/alice")
    );
    assert_eq!(replaced.asserted_at_ms, Some(30));
}

#[test]
fn exact_replay_normalizes_legacy_buckets_and_aggregate_predicates() {
    let mut source = Graph::new();
    let from = source.add_node("https://a.test/".into(), Default::default());
    let to = source.add_node("https://b.test/".into(), Default::default());
    let empty = source.clone();
    for (id, asserter, time) in [
        ("unknown-one", "https://people.test/one", 10),
        ("unknown-two", "https://people.test/two", 20),
        ("known", "https://people.test/known", 30),
    ] {
        source.assert_persisted_semantic_statement(
            from,
            to,
            SemanticStatement {
                statement_id: id.into(),
                predicate: "https://vocab.test/rel".into(),
                recognized_sub_kind: None,
                label: None,
                graph_scope: crate::types::GraphScope::User,
                provenance_iri: Some(asserter.into()),
                asserted_at_ms: Some(time),
            },
        );
    }
    let mut edges = source.persisted_edges_between(from, to);
    let statements = &mut edges[0].semantic.as_mut().unwrap().statements;
    statements[0].provenance_iri = None;
    statements[1].provenance_iri = None;
    let capture = CapturedDelta::ReplaySetEdgesByIds {
        from_id: source.get_node(from).unwrap().id.to_string(),
        to_id: source.get_node(to).unwrap().id.to_string(),
        edges: edges.clone(),
    };
    let mut replayed = empty.clone();
    replay_captured_deltas_onto(&mut replayed, [capture]);
    let statements = replayed
        .get_edge(replayed.find_edge_key(from, to).unwrap())
        .unwrap()
        .semantic_statements();
    assert_eq!(
        statements.len(),
        3,
        "unknown attribution must not merge exact handles"
    );
    for (index, id, time) in [
        (0, "unknown-one", 10),
        (1, "unknown-two", 20),
        (2, "known", 30),
    ] {
        assert_eq!(statements[index].statement_id, id);
        assert_eq!(statements[index].asserted_at_ms, Some(time));
    }
    let unknown = super::super::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI;
    assert_eq!(statements[0].provenance_iri.as_deref(), Some(unknown));
    assert_eq!(statements[1].provenance_iri.as_deref(), Some(unknown));
    assert_eq!(
        statements[2].provenance_iri.as_deref(),
        Some("https://people.test/known")
    );

    let parallel_edges = edges[0]
        .semantic
        .as_ref()
        .unwrap()
        .statements
        .iter()
        .map(|statement| {
            let mut edge = edges[0].clone();
            edge.semantic.as_mut().unwrap().statements = vec![statement.clone()];
            edge
        })
        .collect();
    let mut parallel = empty.clone();
    replay_captured_deltas_onto(
        &mut parallel,
        [CapturedDelta::ReplaySetEdgesByIds {
            from_id: source.get_node(from).unwrap().id.to_string(),
            to_id: source.get_node(to).unwrap().id.to_string(),
            edges: parallel_edges,
        }],
    );
    let restored: Vec<_> = parallel
        .edges_between_undirected(from, to)
        .flat_map(|(_, edge)| edge.semantic_statements())
        .collect();
    assert_eq!(restored.len(), 3);
    assert_eq!(
        restored
            .iter()
            .filter(|s| s.provenance_iri.as_deref() == Some(unknown))
            .count(),
        2
    );
    assert_eq!(
        restored
            .iter()
            .filter(|s| s.provenance_iri.as_deref() == Some("https://people.test/known"))
            .count(),
        1
    );

    let semantic = edges[0].semantic.as_mut().unwrap();
    semantic.statements.clear();
    semantic.sub_kinds.clear();
    semantic.predicate = Some("https://vocab.test/aggregate".into());
    let mut aggregate_replayed = empty;
    replay_captured_deltas_onto(
        &mut aggregate_replayed,
        [CapturedDelta::ReplaySetEdgesByIds {
            from_id: source.get_node(from).unwrap().id.to_string(),
            to_id: source.get_node(to).unwrap().id.to_string(),
            edges,
        }],
    );
    let statements = aggregate_replayed
        .get_edge(aggregate_replayed.find_edge_key(from, to).unwrap())
        .unwrap()
        .semantic_statements();
    assert_eq!(statements.len(), 1);
    assert_eq!(statements[0].predicate, "https://vocab.test/aggregate");
    assert_eq!(statements[0].provenance_iri.as_deref(), Some(unknown));
}

#[test]
fn asserting_deltas_carry_source_attribution() {
    let mut graph = Graph::new();
    let from = graph.add_node("https://a.test/".into(), Default::default());
    let to = graph.add_node("https://b.test/".into(), Default::default());
    apply_graph_delta(
        &mut graph,
        GraphDelta::AssertRelation {
            from,
            to,
            asserter_iri: "https://peer.test/alice".into(),
            assertion: EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Cites,
                label: None,
                decay_progress: None,
            },
        },
    );
    apply_graph_delta(
        &mut graph,
        GraphDelta::AssertSemanticPredicate {
            from,
            to,
            asserter_iri: "https://page.test/".into(),
            predicate: "https://example.test/rel".into(),
        },
    );
    let statements = graph
        .get_edge(graph.find_edge_key(from, to).unwrap())
        .unwrap()
        .semantic_statements();
    assert_eq!(statements.len(), 2);
    assert_eq!(
        statements[0].provenance_iri.as_deref(),
        Some("https://peer.test/alice")
    );
    assert_eq!(
        statements[1].provenance_iri.as_deref(),
        Some("https://page.test/")
    );
    assert!(
        statements
            .iter()
            .all(|statement| statement.provenance_iri.as_deref()
                != Some(&graph.write_author().asserter_iri()))
    );
}

#[test]
fn reingest_updates_the_same_asserter_without_replacing_its_id() {
    let mut graph = Graph::new();
    let from = graph.add_node("https://a.test/".into(), Default::default());
    let to = graph.add_node("https://b.test/".into(), Default::default());
    let statement = |id: &str, asserter: &str, time| SemanticStatement {
        statement_id: id.into(),
        predicate: predicate_iri(SemanticSubKind::Cites).into(),
        recognized_sub_kind: Some(SemanticSubKind::Cites),
        label: None,
        graph_scope: crate::types::GraphScope::Source,
        provenance_iri: Some(asserter.into()),
        asserted_at_ms: Some(time),
    };
    graph.assert_persisted_semantic_statement(
        from,
        to,
        statement("first", "https://page.test/", 10),
    );
    graph.assert_persisted_semantic_statement(
        from,
        to,
        statement("peer", "https://peer.test/", 20),
    );
    graph.assert_persisted_semantic_statement(
        from,
        to,
        statement("new-id", "https://page.test/", 30),
    );
    let statements = graph
        .get_edge(graph.find_edge_key(from, to).unwrap())
        .unwrap()
        .semantic_statements();
    assert_eq!(statements.len(), 2);
    assert_eq!(statements[0].statement_id, "first");
    assert_eq!(statements[0].asserted_at_ms, Some(30));
    assert_eq!(statements[1].statement_id, "peer");
    assert_eq!(statements[1].asserted_at_ms, Some(20));
}

#[test]
fn open_predicate_assertion_supplies_attribution_on_an_existing_pair() {
    let mut graph = Graph::new();
    let from = graph.add_node("https://a.test/".into(), Default::default());
    let to = graph.add_node("https://b.test/".into(), Default::default());
    let edge = graph
        .assert_relation(
            from,
            to,
            EdgeAssertion::Containment {
                sub_kind: ContainmentSubKind::UserFolder,
            },
        )
        .unwrap();
    assert!(
        graph
            .get_edge(edge)
            .unwrap()
            .semantic_statements()
            .is_empty()
    );
    apply_graph_delta(
        &mut graph,
        GraphDelta::AssertSemanticPredicate {
            from,
            to,
            predicate: "https://example.test/rel".into(),
            asserter_iri: "https://source.test/".into(),
        },
    );
    let statements = graph.get_edge(edge).unwrap().semantic_statements();
    assert_eq!(statements.len(), 1);
    assert_eq!(statements[0].predicate, "https://example.test/rel");
    assert_eq!(
        statements[0].provenance_iri.as_deref(),
        Some("https://source.test/")
    );
}
