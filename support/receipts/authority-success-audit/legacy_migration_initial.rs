// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
// Diagnostic assertions describe observed discrepancies, not desired behavior.
use kernel::graph::{
    Author, CapturedDelta, Graph, GraphJournal, SemanticStatement, SemanticStatementSpec,
    SemanticSubKind, predicate_iri, replay_captured_deltas_onto,
    legacy_resource_migration::migrate_legacy_prefix,
};
use kernel::graph::apply::add_node;
use kernel::persistence::PersistedEdge;
use kernel::types::{AssertionBasis, AssertionMethod};
use uuid::Uuid;

fn baseline() -> Graph {
    let mut graph = Graph::new();
    for (id, url) in [(1, "https://audit.test/a"), (2, "https://audit.test/b")] {
        add_node(&mut graph, Some(Uuid::from_u128(id)), url.into(), Default::default());
    }
    graph
}

fn rows(graph: &mut Graph) -> Vec<PersistedEdge> {
    let a = graph.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
    let b = graph.get_node_key_by_id(Uuid::from_u128(2)).unwrap();
    for author in ["urn:audit:first", "urn:audit:second"] {
        graph.try_assert_semantic_statement(a, b, SemanticStatementSpec {
            predicate: predicate_iri(SemanticSubKind::UserGrouped).into(),
            recognized_sub_kind: Some(SemanticSubKind::UserGrouped),
            provenance_iri: Some(author.into()),
            asserted_at_ms: Some(10),
            ..Default::default()
        }).unwrap();
    }
    let edge = graph.to_snapshot().edges[0].clone();
    edge.retained_assertions().iter().map(|statement| {
        let mut row = edge.clone();
        row.assertions = Some(vec![statement.clone()]);
        row
    }).collect()
}

fn capture(edges: Vec<PersistedEdge>) -> CapturedDelta {
    CapturedDelta::ReplaySetEdgesByIds {
        from_id: Uuid::from_u128(1).to_string(),
        to_id: Uuid::from_u128(2).to_string(),
        edges,
    }
}

#[test]
fn parallel_surface_row_order_disagrees_with_the_migrations_own_replay_effects() {
    let baseline = baseline();
    for count in [1, 2] {
        let mut journal = GraphJournal::new();
        let mut template = baseline.clone();
        let mut edges = rows(&mut template);
        edges.truncate(count);
        journal.record(capture(edges));
        let migration = migrate_legacy_prefix(&baseline, journal.entries()).unwrap();
        let mut replayed = baseline.clone();
        replay_captured_deltas_onto(&mut replayed, migration.baseline_effects);
        for effects in migration.entry_effects {
            replay_captured_deltas_onto(&mut replayed, effects);
        }
        let expected = migration.graph.to_snapshot().edges;
        let actual = replayed.to_snapshot().edges;
        if count == 1 {
            assert_eq!(actual, expected, "single row is the existing happy-path control");
        } else {
            assert_ne!(actual, expected, "the emitted reconstruction is not exact for parallel rows");
            assert_eq!(actual, expected.into_iter().rev().collect::<Vec<_>>(), "the discrepancy is row order, not missing assertions");
        }
    }
}

#[test]
fn migrated_handle_can_change_reason_identity_that_the_checked_writer_refuses() {
    let baseline = baseline();
    let mut template = baseline.clone();
    let mut edge = rows(&mut template).remove(0);
    edge.assertions.as_mut().unwrap()[0].basis = AssertionBasis {
        method: Some(AssertionMethod { iri: "urn:audit:method:first".into(), version: Some("1".into()) }),
        evidence: vec!["urn:audit:evidence:first".into()],
        ..Default::default()
    };
    let mut journal = GraphJournal::new();
    journal.record_as(Author::user(), capture(vec![edge.clone()]));
    let mut changed = edge.clone();
    changed.assertions.as_mut().unwrap()[0].basis.method.as_mut().unwrap().iri = "urn:audit:method:other".into();
    changed.assertions.as_mut().unwrap()[0].basis.evidence = vec!["urn:audit:evidence:other".into()];

    let mut checked = baseline.clone();
    let a = checked.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
    let b = checked.get_node_key_by_id(Uuid::from_u128(2)).unwrap();
    let to_statement = |edge: &PersistedEdge| -> SemanticStatement {
        serde_json::from_value(serde_json::to_value(&edge.retained_assertions()[0]).unwrap()).unwrap()
    };
    checked.try_assert_persisted_semantic_statement(a, b, to_statement(&edge)).unwrap();
    assert!(checked.try_assert_persisted_semantic_statement(a, b, to_statement(&changed)).is_err());
    journal.record(capture(vec![changed.clone()]));
    let migrated = migrate_legacy_prefix(&baseline, journal.entries()).unwrap();
    let stored = migrated.graph.to_snapshot().edges;
    assert_eq!(stored[0].retained_assertions()[0].statement_id, edge.retained_assertions()[0].statement_id);
    assert_eq!(stored[0].retained_assertions()[0].basis, changed.retained_assertions()[0].basis);
}
