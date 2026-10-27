// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Legacy migration dedupes whole identical rows to the first and says so; a handle
//! repeated on rows that differ is a conflict, and the migration refuses it.
use super::legacy_resource_migration::{
    LEGACY_RESOURCE_MIGRATION_FACET, LegacyMigrationNote, migrate_legacy_prefix,
};
use super::*;
use crate::persistence::{PersistedEdge, PersistedResourceRecord};
use crate::types::GraphScope;
use euclid::default::Point2D;
use std::collections::BTreeSet;
use uuid::Uuid;

fn cites() -> &'static str {
    predicate_iri(SemanticSubKind::Cites)
}

fn claim(ids: &[&str], predicate: &str) -> EdgePayload {
    let mut payload = EdgePayload::new();
    for id in ids {
        payload.push_persisted_semantic_statement(SemanticStatement {
            statement_id: (*id).into(),
            predicate: predicate.into(),
            recognized_sub_kind: sub_kind_from_iri(predicate),
            label: Some(format!("label-{id}")),
            graph_scope: GraphScope::User,
            provenance_iri: Some("https://author.test/".into()),
            asserted_at_ms: Some(100),
            first_asserted_ms: Some(100),
            withdrawn: None,
            basis: Default::default(),
        });
    }
    payload
}

fn relabeled(mut payload: EdgePayload) -> EdgePayload {
    let statement = &mut payload.assertions.statements[0];
    statement.label = Some("conflicting copy".into());
    statement.basis.note = Some("conflicting copy".into());
    payload
}

/// Surface nodes 1 and 2 at a.test and b.test.
fn surfaces() -> (Graph, NodeKey, NodeKey) {
    let mut graph = Graph::new();
    let a = graph.add_node_with_id(
        Uuid::from_u128(1),
        "https://a.test/".into(),
        Point2D::zero(),
    );
    let b = graph.add_node_with_id(
        Uuid::from_u128(2),
        "https://b.test/".into(),
        Point2D::zero(),
    );
    (graph, a, b)
}

/// Two Resources with no Surface behind them.
fn resources() -> (Graph, ResourceNode, ResourceNode) {
    let mut graph = Graph::new();
    let (from, to) = (
        ResourceNode::for_term("https://other.test/a"),
        ResourceNode::for_term("https://other.test/b"),
    );
    for resource in [&from, &to] {
        assert!(graph.set_resource_record(
            resource.id(),
            Some(PersistedResourceRecord {
                canonical_iri: resource.canonical_iri().into(),
                facets: vec![]
            })
        ));
    }
    (graph, from, to)
}

fn connect_resources(graph: &mut Graph, from: Uuid, to: Uuid, payloads: &[EdgePayload]) {
    let (from, to) = (
        graph.resources.key_of(&from).unwrap(),
        graph.resources.key_of(&to).unwrap(),
    );
    for payload in payloads {
        graph.resources.connect(from, to, payload.clone());
    }
}

fn ids(rows: &[PersistedEdge]) -> BTreeSet<String> {
    rows.iter()
        .flat_map(|row| row.retained_assertions())
        .map(|statement| statement.statement_id.clone())
        .collect()
}

fn notes(graph: &Graph, resource: Uuid) -> Vec<LegacyMigrationNote> {
    let record = graph.resource_record(resource).unwrap();
    let facet = record
        .facets
        .iter()
        .find(|facet| facet.facet == LEGACY_RESOURCE_MIGRATION_FACET)
        .expect("the owning Resource carries migration notes");
    serde_json::from_str(&facet.value_json).unwrap()
}

/// The migrated graph reopens, and the effects reproduce its rows and notes from the baseline.
fn assert_reopens_and_replays(
    baseline: &Graph,
    migrated: &legacy_resource_migration::MigratedReplay,
) {
    Graph::try_from_snapshot(&migrated.graph.to_snapshot()).expect("the output is legal state");
    let mut replayed = baseline.clone();
    replay_captured_deltas_onto(&mut replayed, migrated.baseline_effects.clone());
    for effects in &migrated.entry_effects {
        replay_captured_deltas_onto(&mut replayed, effects.clone());
    }
    let (replayed, output) = (replayed.to_snapshot(), migrated.graph.to_snapshot());
    assert_eq!(replayed.resources, output.resources);
    assert_eq!(replayed.edges, output.edges);
    assert_eq!(replayed.resource_edges, output.resource_edges);
}

#[test]
fn legacy_migration_dedupes_identical_copies_of_a_surface_placed_claim_and_says_so() {
    let (mut baseline, a, b) = surfaces();
    let grouped = claim(&["grouped"], predicate_iri(SemanticSubKind::UserGrouped));
    baseline.inner.connect(a, b, grouped.clone());
    baseline.inner.connect(a, b, grouped);
    assert!(
        Graph::try_from_snapshot(&baseline.to_snapshot()).is_err(),
        "the baseline holds a repeated handle"
    );

    let migrated = migrate_legacy_prefix(&baseline, &[]).unwrap();
    let rows = migrated.graph.persisted_edges_between(a, b);
    assert_eq!(rows.len(), 1, "the copy is dropped, the first row kept");
    assert_eq!(ids(&rows), BTreeSet::from(["grouped".into()]));
    assert!(migrated.graph.to_snapshot().resource_edges.is_empty());

    let owner = ResourceNode::new("https://a.test/").id();
    let [note] = notes(&migrated.graph, owner).try_into().unwrap();
    assert_eq!(
        note.uncertainty,
        "identical_parallel_surface_copy_deduped_to_first_row"
    );
    assert_eq!(note.statement.statement_id, "grouped");
    assert_eq!(note.from_surface_id, Uuid::from_u128(1).to_string());
    assert_eq!(note.to_surface_id, Uuid::from_u128(2).to_string());
    assert_eq!(note.from_resource_id, owner.to_string());
    assert_eq!(
        note.to_resource_id,
        ResourceNode::new("https://b.test/").id().to_string()
    );
    assert_reopens_and_replays(&baseline, &migrated);
}

#[test]
fn legacy_migration_dedupes_identical_surface_rows_that_a_journal_entry_sets() {
    let (baseline, a, b) = surfaces();
    let grouped = claim(&["grouped"], predicate_iri(SemanticSubKind::UserGrouped));
    let row = snapshot::persisted_edge_for_ids(Uuid::from_u128(1), Uuid::from_u128(2), &grouped);
    let entries = [AttributedDelta {
        author: Author::person("legacy-author"),
        delta: CapturedDelta::ReplaySetEdgesByIds {
            from_id: Uuid::from_u128(1).to_string(),
            to_id: Uuid::from_u128(2).to_string(),
            edges: vec![row.clone(), row],
        },
    }];
    let migrated = migrate_legacy_prefix(&baseline, &entries).unwrap();
    assert_eq!(migrated.graph.persisted_edges_between(a, b).len(), 1);
    let owner = ResourceNode::new("https://a.test/").id();
    assert_eq!(notes(&migrated.graph, owner).len(), 1);
    assert_reopens_and_replays(&baseline, &migrated);
}

#[test]
fn legacy_migration_dedupes_identical_copies_in_pre_existing_resource_rows_and_says_so() {
    let (mut baseline, from, to) = resources();
    let cited = claim(&["cited"], cites());
    connect_resources(&mut baseline, from.id(), to.id(), &[cited.clone(), cited]);
    assert!(
        Graph::try_from_snapshot(&baseline.to_snapshot()).is_err(),
        "the baseline holds a repeated handle"
    );

    let migrated = migrate_legacy_prefix(&baseline, &[]).unwrap();
    let rows = migrated
        .graph
        .persisted_resource_edges_between(from.id(), to.id());
    assert_eq!(rows.len(), 1, "the copy is dropped, the first row kept");
    assert_eq!(ids(&rows), BTreeSet::from(["cited".into()]));

    let [note] = notes(&migrated.graph, from.id()).try_into().unwrap();
    assert_eq!(
        note.uncertainty,
        "identical_parallel_resource_copy_deduped_to_first_row"
    );
    assert_eq!(note.statement.statement_id, "cited");
    assert_eq!(note.from_resource_id, from.id().to_string());
    assert_eq!(note.to_resource_id, to.id().to_string());
    assert_eq!(note.from_url, from.canonical_iri());
    assert!(
        note.from_surface_id.is_empty() && note.to_surface_id.is_empty(),
        "the rows never sat on a Surface"
    );
    assert_reopens_and_replays(&baseline, &migrated);
}

#[test]
fn legacy_migration_refuses_a_repeated_handle_on_rows_that_differ() {
    let cited = claim(&["cited"], cites());
    let grouped = claim(&["grouped"], predicate_iri(SemanticSubKind::UserGrouped));
    // Another claim of the same stratum on the second row: the rows differ as stored.
    let wider = |payload: &EdgePayload| {
        let mut wider = payload.clone();
        let predicate = &payload.assertions.statements[0].predicate;
        wider
            .assertions
            .statements
            .extend(claim(&["extra"], predicate).assertions.statements);
        wider
    };

    for surface in [true, false] {
        let own = if surface { &grouped } else { &cited };
        let id = &own.assertions.statements[0].statement_id;
        // The statement itself differs: a conflict between two claims.
        // The statement matches but the row also holds another: not a copy of the row.
        for (second, why) in [(relabeled(own.clone()), "statement"), (wider(own), "row")] {
            let refused = if surface {
                let (mut baseline, a, b) = surfaces();
                baseline.inner.connect(a, b, own.clone());
                baseline.inner.connect(a, b, second);
                migrate_legacy_prefix(&baseline, &[])
            } else {
                let (mut baseline, from, to) = resources();
                connect_resources(&mut baseline, from.id(), to.id(), &[own.clone(), second]);
                migrate_legacy_prefix(&baseline, &[])
            };
            let error = refused.err().unwrap_or_else(|| {
                panic!("surface={surface}: rows that differ in {why} are not copies")
            });
            assert!(
                error.0.contains(id.as_str()),
                "surface={surface} {why}: {error}"
            );
        }
    }
}
