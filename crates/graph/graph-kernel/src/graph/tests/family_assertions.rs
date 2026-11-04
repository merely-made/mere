// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Exact-record controls for the former non-semantic edge families.

use super::super::*;
use crate::graph::{
    ArrangementSubKind, ContainmentSubKind, EdgeAssertion, ImportedSubKind, ProvenanceSubKind,
    RelationKey, RelationKind, SemanticStatement, StatementWriteError,
};
use crate::persistence::{
    GraphSnapshot, PersistedContainmentEdgeData, PersistedContainmentSubKind,
};
use crate::types::{AssertionBasis, AssertionMethod, GraphScope};

fn reason_spec(
    kind: RelationKind,
    actor: &str,
    time_ms: u64,
    method_iri: &str,
    evidence: &str,
    note: &str,
) -> SemanticStatementSpec {
    SemanticStatementSpec {
        predicate: kind
            .predicate_iri()
            .expect("assertion kind has a predicate")
            .into(),
        recognized_sub_kind: match kind {
            RelationKind::Semantic(sub_kind) => Some(sub_kind),
            _ => None,
        },
        label: Some(note.into()),
        graph_scope: GraphScope::User,
        provenance_iri: Some(actor.into()),
        asserted_at_ms: Some(time_ms),
        basis: AssertionBasis {
            note: Some(note.into()),
            method: Some(AssertionMethod {
                iri: method_iri.into(),
                version: Some("v1".into()),
            }),
            evidence: vec![evidence.into()],
        },
    }
}

fn saved_record(graph: &Graph, id: &str) -> SemanticStatement {
    graph
        .find_assertion(id)
        .expect("exact record remains in the graph")
        .1
        .clone()
}

fn persisted_ids(snapshot: &GraphSnapshot) -> Vec<String> {
    snapshot
        .edges
        .iter()
        .chain(&snapshot.resource_edges)
        .flat_map(|edge| edge.retained_assertions())
        .map(|record| record.statement_id.clone())
        .collect()
}

#[test]
fn collection_member_reasons_keep_exact_handles_and_stale_replay_cannot_revive() {
    let actor = "https://people.test/alex";
    let mut graph = Graph::new();
    let from = graph.add_node("https://source.test/item".into(), Default::default());
    let to = graph.add_node("https://target.other/item".into(), Default::default());
    let other = graph.add_node("https://other.test/item".into(), Default::default());
    let kind = RelationKind::Containment(ContainmentSubKind::CollectionMember);

    let (owner, first) = graph
        .try_assert_relation_statement(
            from,
            to,
            reason_spec(
                kind,
                actor,
                100,
                "urn:method:collection-import",
                "urn:evidence:collection-a",
                "first note",
            ),
        )
        .unwrap();
    let carried_before_withdrawal = saved_record(&graph, &first.statement_id);
    let (second_owner, second) = graph
        .try_assert_relation_statement(
            from,
            to,
            reason_spec(
                kind,
                actor,
                110,
                "urn:method:collection-review",
                "urn:evidence:collection-b",
                "second note",
            ),
        )
        .unwrap();

    assert_eq!(owner, second_owner);
    assert!(matches!(owner, RelationKey::Surface(_)));
    assert_eq!(
        graph.find_edge_key(from, to),
        match owner {
            RelationKey::Surface(key) => Some(key),
            RelationKey::Resource(_) => None,
        }
    );
    assert!(
        graph.find_edge_key(to, from).is_none(),
        "the original directed pair is retained"
    );
    assert_ne!(first.statement_id, second.statement_id);
    assert_eq!(graph.get_relation(owner).unwrap().assertions().len(), 2);
    assert_eq!(
        graph
            .find_held_assertion(&first.statement_id)
            .unwrap()
            .1
            .relation_kind(),
        kind
    );

    // The L13 reason key excludes the note and method version. A new note for
    // the same method IRI and evidence confirms the existing exact assertion.
    let mut note_only = reason_spec(
        kind,
        actor,
        120,
        "urn:method:collection-import",
        "urn:evidence:collection-a",
        "updated display note",
    );
    note_only.basis.method.as_mut().unwrap().version = Some("v2".into());
    let (_, ensured_again) = graph
        .try_assert_relation_statement(from, to, note_only)
        .unwrap();
    assert_eq!(ensured_again.statement_id, first.statement_id);
    assert_eq!(graph.get_relation(owner).unwrap().assertions().len(), 2);
    let confirmed = saved_record(&graph, &first.statement_id);
    assert_eq!(confirmed.first_asserted_ms, Some(100));
    assert_eq!(confirmed.asserted_at_ms, Some(120));
    assert_eq!(
        confirmed.basis.note.as_deref(),
        Some("updated display note")
    );
    assert_eq!(
        confirmed.basis.method.as_ref().unwrap().version.as_deref(),
        Some("v2")
    );

    assert!(graph.withdraw_semantic_assertion(&first.statement_id, Some(130), Some(actor.into())));
    let tombstone = saved_record(&graph, &first.statement_id);
    assert_eq!(tombstone.withdrawn.as_ref().unwrap().at_ms, Some(130));
    assert_eq!(
        tombstone.withdrawn.as_ref().unwrap().by.as_deref(),
        Some(actor)
    );
    assert!(graph.find_held_assertion(&first.statement_id).is_none());
    assert!(graph.find_held_assertion(&second.statement_id).is_some());
    let payload = graph.get_relation(owner).unwrap();
    assert_eq!(payload.assertions().len(), 2);
    assert_eq!(payload.held_assertions().count(), 1);
    assert_eq!(payload.semantic_statements().len(), 0);

    let revision = graph.revision();
    assert_eq!(
        graph
            .try_assert_persisted_relation_statement(from, to, carried_before_withdrawal.clone())
            .unwrap(),
        owner,
        "a carried stale held copy is recognized at its original owner"
    );
    assert_eq!(graph.revision(), revision);
    assert_eq!(saved_record(&graph, &first.statement_id), tombstone);
    assert!(matches!(
        graph.try_assert_persisted_relation_statement(other, to, carried_before_withdrawal),
        Err(StatementWriteError::HandleCollision(id)) if id == first.statement_id
    ));
    assert_eq!(saved_record(&graph, &first.statement_id), tombstone);

    let first_snapshot = serde_json::to_value(graph.to_snapshot_at(0)).unwrap();
    let first_loaded: GraphSnapshot = serde_json::from_value(first_snapshot.clone()).unwrap();
    let loaded = Graph::from_snapshot(&first_loaded);
    assert_eq!(saved_record(&loaded, &first.statement_id), tombstone);
    assert_eq!(
        saved_record(&loaded, &second.statement_id),
        saved_record(&graph, &second.statement_id)
    );
    let second_snapshot = serde_json::to_value(loaded.to_snapshot_at(0)).unwrap();
    let second_loaded = Graph::from_snapshot(&serde_json::from_value(second_snapshot).unwrap());
    assert_eq!(saved_record(&second_loaded, &first.statement_id), tombstone);
    assert_eq!(
        saved_record(&second_loaded, &second.statement_id),
        saved_record(&graph, &second.statement_id)
    );
}

#[test]
fn all_nonsemantic_families_preserve_metadata_and_arrangement_durability() {
    let mut graph = Graph::new();
    let from = graph.add_node("https://from.test/item".into(), Default::default());
    let to = graph.add_node("https://to.other/item".into(), Default::default());
    let actor = "urn:agent:family-fixture";
    let cases = [
        (
            RelationKind::Containment(ContainmentSubKind::CollectionMember),
            "containment",
            "urn:evidence:containment",
        ),
        (
            RelationKind::Arrangement(ArrangementSubKind::FrameMember),
            "arrangement-frame",
            "urn:evidence:arrangement-frame",
        ),
        (
            RelationKind::Arrangement(ArrangementSubKind::TileGroup),
            "arrangement-session",
            "urn:evidence:arrangement-session",
        ),
        (
            RelationKind::Imported(ImportedSubKind::BookmarkFolder),
            "imported",
            "urn:evidence:imported",
        ),
        (
            RelationKind::Provenance(ProvenanceSubKind::CopiedFrom),
            "provenance",
            "urn:evidence:provenance",
        ),
    ];
    let mut ids = Vec::new();
    let mut owner = None;
    for (index, (kind, method, evidence)) in cases.into_iter().enumerate() {
        let (next_owner, asserted) = graph
            .try_assert_relation_statement(
                from,
                to,
                reason_spec(
                    kind,
                    actor,
                    1_000 + index as u64,
                    &format!("urn:method:{method}"),
                    evidence,
                    method,
                ),
            )
            .unwrap();
        if let Some(owner) = owner {
            assert_eq!(
                owner, next_owner,
                "these canonical predicates share the Surface bucket"
            );
        } else {
            owner = Some(next_owner);
        }
        ids.push((
            asserted.statement_id,
            kind,
            evidence.to_string(),
            method.to_string(),
            1_000 + index as u64,
        ));
    }
    let owner = owner.unwrap();
    assert!(matches!(owner, RelationKey::Surface(_)));
    let payload = graph.get_relation(owner).unwrap();
    assert_eq!(payload.assertions().len(), cases_len());
    assert_eq!(payload.held_assertions().count(), cases_len());
    assert!(payload.semantic_statements().is_empty());
    assert_eq!(payload.held_semantic_statements().count(), 0);

    for (id, kind, evidence, method, time) in &ids {
        let (actual_owner, record) = graph.find_held_assertion(id).unwrap();
        assert_eq!(actual_owner, owner);
        assert_eq!(record.relation_kind(), *kind);
        assert_eq!(record.provenance_iri.as_deref(), Some(actor));
        assert_eq!(record.first_asserted_ms, Some(*time));
        assert_eq!(record.asserted_at_ms, Some(*time));
        assert_eq!(record.basis.evidence, [evidence.as_str()]);
        assert_eq!(
            record.basis.method.as_ref().unwrap().iri,
            format!("urn:method:{method}")
        );
    }

    let pair_capture = graph.persisted_edges_between(from, to);
    let pair_ids: Vec<_> = pair_capture
        .iter()
        .flat_map(|edge| edge.retained_assertions())
        .map(|record| record.statement_id.as_str())
        .collect();
    assert_eq!(pair_ids.len(), cases_len());
    assert!(ids.iter().all(|(id, ..)| pair_ids.contains(&id.as_str())));

    let snapshot = graph.to_snapshot_at(0);
    let durable_ids = persisted_ids(&snapshot);
    assert_eq!(durable_ids.len(), cases_len() - 1);
    assert!(ids.iter().all(|(id, kind, ..)| {
        let is_session_arrangement =
            *kind == RelationKind::Arrangement(ArrangementSubKind::TileGroup);
        durable_ids.contains(id) != is_session_arrangement
    }));
    let restored = Graph::from_snapshot(&snapshot);
    assert!(
        restored.find_assertion(&ids[2].0).is_none(),
        "session arrangement is excluded from durable snapshot"
    );
    for (id, kind, evidence, method, time) in ids
        .iter()
        .filter(|(_, kind, ..)| *kind != RelationKind::Arrangement(ArrangementSubKind::TileGroup))
    {
        let (restored_owner, record) = restored.find_assertion(id).unwrap();
        assert_eq!(record.relation_kind(), *kind);
        assert_eq!(record.provenance_iri.as_deref(), Some(actor));
        assert_eq!(record.first_asserted_ms, Some(*time));
        assert_eq!(record.asserted_at_ms, Some(*time));
        assert_eq!(record.basis.evidence, [evidence.as_str()]);
        assert_eq!(
            record.basis.method.as_ref().unwrap().iri,
            format!("urn:method:{method}")
        );
        assert_eq!(
            restored
                .get_relation(restored_owner)
                .unwrap()
                .held_assertions()
                .count(),
            cases_len() - 1
        );
    }
}

fn cases_len() -> usize {
    5
}

#[test]
fn derived_admission_refuses_a_retained_stable_id_on_another_pair() {
    let parent_id = uuid::Uuid::from_u128(0x100);
    let child_id = uuid::Uuid::from_u128(0x101);
    let parent_url = "https://collision.test/root/";
    let child_url = "https://collision.test/root/item";

    let mut seed = Graph::new();
    let parent = seed.add_node_with_id(parent_id, parent_url.into(), Default::default());
    let child = seed.add_node_with_id(child_id, child_url.into(), Default::default());
    seed.derive_containment_for(child);
    let deterministic_id = seed
        .retained_relations_between(child, parent)
        .flat_map(|(_, payload)| payload.held_assertions())
        .find(|record| {
            record.relation_kind() == RelationKind::Containment(ContainmentSubKind::UrlPath)
        })
        .expect("seed derives its stable Resource identity")
        .statement_id
        .clone();

    let mut graph = Graph::new();
    let parent = graph.add_node_with_id(parent_id, parent_url.into(), Default::default());
    let child = graph.add_node_with_id(child_id, child_url.into(), Default::default());
    let foreign_from = graph.add_node_with_id(
        uuid::Uuid::from_u128(0x200),
        "https://foreign.test/from".into(),
        Default::default(),
    );
    let foreign_to = graph.add_node_with_id(
        uuid::Uuid::from_u128(0x201),
        "https://foreign.test/to".into(),
        Default::default(),
    );
    let mut carried = SemanticStatement::new(
        RelationKind::Containment(ContainmentSubKind::UrlPath)
            .predicate_iri()
            .unwrap()
            .into(),
        None,
        Some("foreign carried row".into()),
        GraphScope::User,
        Some("urn:actor:foreign".into()),
        Some(80),
    );
    carried.statement_id = deterministic_id.clone();
    carried.basis.note = Some("independent foreign reason".into());
    let foreign_owner = graph
        .try_assert_persisted_relation_statement(foreign_from, foreign_to, carried)
        .expect("foreign carried assertion is admitted");
    let before = graph.to_snapshot_at(0);
    let revision = graph.revision();

    graph.derive_containment_for(child);

    assert_eq!(graph.revision(), revision, "collision refusal is atomic");
    assert_eq!(
        serde_json::to_value(graph.to_snapshot_at(0)).unwrap(),
        serde_json::to_value(before).unwrap(),
        "collision refusal leaves the graph unchanged"
    );
    let (_, retained) = graph
        .find_assertion(&deterministic_id)
        .expect("the existing foreign row remains the unique owner");
    assert_eq!(
        retained.provenance_iri.as_deref(),
        Some("urn:actor:foreign")
    );
    let after = graph.to_snapshot_at(0);
    assert_eq!(
        after
            .edges
            .iter()
            .chain(&after.resource_edges)
            .flat_map(|edge| edge.retained_assertions())
            .filter(|record| record.statement_id == deterministic_id)
            .count(),
        1,
        "no second pair acquires the carried stable ID"
    );
    assert!(Graph::try_from_snapshot(&after).is_ok());
    assert!(matches!(foreign_owner, RelationKey::Resource(_)));
    assert!(
        after
            .resource_edges
            .iter()
            .filter(|edge| {
                edge.from_node_id == ResourceNode::new(child_url).id().to_string()
                    && edge.to_node_id == ResourceNode::new(parent_url).id().to_string()
            })
            .flat_map(|edge| edge.retained_assertions())
            .all(|record| record.statement_id != deterministic_id),
        "original derived pair does not acquire the colliding ID"
    );
    let _ = parent;
}

#[test]
fn live_resource_rule_output_keeps_recorded_owner_during_compatibility_rebuild() {
    let mut graph = Graph::new();
    let parent = graph.add_node("https://rule.test/root/".into(), Default::default());
    let child = graph.add_node("https://rule.test/root/item".into(), Default::default());
    graph.derive_containment_for(child);
    let before = graph
        .retained_relations_between(child, parent)
        .filter(|(key, _)| matches!(key, RelationKey::Resource(_)))
        .flat_map(|(_, payload)| payload.held_assertions())
        .find(|record| {
            record.relation_kind() == RelationKind::Containment(ContainmentSubKind::UrlPath)
        })
        .unwrap()
        .clone();
    let restored = Graph::from_snapshot(&graph.to_snapshot_at(0));
    let (owner, after) = restored.find_assertion(&before.statement_id).unwrap();
    assert!(matches!(owner, RelationKey::Resource(_)));
    assert_eq!(after, &before);
    // Ordinary compatibility loading keeps its established Surface rule
    // projection, separate from the carried Resource-owned source assertion.
    let surface: Vec<_> = restored
        .retained_relations_between(child, parent)
        .filter(|(key, _)| matches!(key, RelationKey::Surface(_)))
        .flat_map(|(_, payload)| payload.held_assertions())
        .filter(|record| {
            record.relation_kind() == RelationKind::Containment(ContainmentSubKind::UrlPath)
        })
        .collect();
    assert_eq!(surface.len(), 1);
    assert_ne!(surface[0].statement_id, before.statement_id);
    assert!(Graph::try_from_snapshot(&restored.to_snapshot_at(0)).is_ok());
}

#[path = "family_assertions/compatibility.rs"]
mod compatibility;
