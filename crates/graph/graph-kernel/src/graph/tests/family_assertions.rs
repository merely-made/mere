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
fn legacy_surface_identity_uses_pair_local_ordinal_in_snapshot_and_replay() {
    use crate::graph::capture::{CapturedDelta, replay_captured_deltas_onto};
    use crate::persistence::{
        PersistedContainmentEdgeData, PersistedContainmentSubKind, PersistedEdge,
        PersistedEdgeFamily,
    };

    let mut graph = Graph::new();
    let unrelated_from = graph.add_node("https://unrelated.test/a".into(), Default::default());
    let unrelated_to = graph.add_node("https://unrelated.test/b".into(), Default::default());
    let target_from = graph.add_node("https://target.test/a".into(), Default::default());
    let target_to = graph.add_node("https://target.test/b".into(), Default::default());
    let snapshot = graph.to_snapshot();
    let id = |key| graph.get_node(key).unwrap().id.to_string();
    let make_legacy = |from: String, to: String| PersistedEdge {
        from_node_id: from,
        to_node_id: to,
        families: vec![PersistedEdgeFamily::Containment],
        semantic: None,
        traversal: None,
        containment: Some(PersistedContainmentEdgeData {
            sub_kinds: vec![PersistedContainmentSubKind::CollectionMember],
        }),
        arrangement: None,
        imported: None,
        provenance: None,
        assertions: None,
    };
    let unrelated = make_legacy(id(unrelated_from), id(unrelated_to));
    let target_a = make_legacy(id(target_from), id(target_to));
    let target_b = make_legacy(id(target_from), id(target_to));
    let mut legacy = snapshot.clone();
    legacy.assertion_schema = 0;
    legacy.edges = vec![unrelated, target_a.clone(), target_b.clone()];

    let loaded = Graph::try_from_recorded_snapshot(&legacy)
        .unwrap()
        .to_snapshot();
    let target_pair = (id(target_from), id(target_to));
    let mut loaded_ids: Vec<_> = loaded
        .edges
        .iter()
        .filter(|edge| (edge.from_node_id.clone(), edge.to_node_id.clone()) == target_pair)
        .flat_map(|edge| {
            edge.retained_assertions()
                .iter()
                .map(|row| row.statement_id.clone())
        })
        .collect();
    loaded_ids.sort();

    let mut base = legacy.clone();
    base.edges.clear();
    let mut replayed = Graph::try_from_recorded_snapshot(&base).unwrap();
    replay_captured_deltas_onto(
        &mut replayed,
        [CapturedDelta::ReplaySetEdgesByIds {
            from_id: target_pair.0.clone(),
            to_id: target_pair.1.clone(),
            edges: vec![target_a, target_b],
        }],
    );
    let replayed = replayed.to_snapshot();
    let mut replayed_ids: Vec<_> = replayed
        .edges
        .iter()
        .filter(|edge| (edge.from_node_id.clone(), edge.to_node_id.clone()) == target_pair)
        .flat_map(|edge| {
            edge.retained_assertions()
                .iter()
                .map(|row| row.statement_id.clone())
        })
        .collect();
    replayed_ids.sort();
    assert_eq!(loaded_ids.len(), 2);
    assert_ne!(loaded_ids[0], loaded_ids[1]);
    assert!(
        loaded_ids
            .iter()
            .all(|id| id.starts_with("urn:mere:legacy-assertion:v1:"))
    );
    assert_eq!(loaded_ids, replayed_ids);
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
fn legacy_four_sidecar_json_migrates_to_stable_unknown_records() {
    let mut graph = Graph::new();
    let from = graph.add_node("https://legacy-from.test/item".into(), Default::default());
    let to = graph.add_node("https://legacy-to.other/item".into(), Default::default());
    let base = serde_json::to_value(graph.to_snapshot_at(0)).unwrap();
    let from_id = graph.get_node(from).unwrap().id.to_string();
    let to_id = graph.get_node(to).unwrap().id.to_string();
    let mut json = base;
    // Schema zero is the pre-canonical edge-sidecar representation. Keeping
    // this as JSON exercises the actual backward-compatible wire shape.
    json["assertion_schema"] = serde_json::json!(0);
    json["edges"] = serde_json::json!([{
        "from_node_id": from_id,
        "to_node_id": to_id,
        "families": ["Containment", "Arrangement", "Imported", "Provenance"],
        "semantic": null,
        "traversal": null,
        "containment": { "sub_kinds": ["CollectionMember"] },
        "arrangement": { "sub_kinds": ["FrameMember"] },
        "imported": { "sub_kinds": ["BookmarkFolder"] },
        "provenance": { "sub_kinds": ["CopiedFrom"] }
    }]);
    // `assertions` is intentionally absent: None selects the old sidecar reader.
    let legacy: GraphSnapshot = serde_json::from_value(json).unwrap();

    let first = Graph::from_snapshot(&legacy);
    let second = Graph::from_snapshot(&legacy);
    let first_key = first
        .find_edge_key(from, to)
        .expect("legacy edge remains directed");
    let second_from = second
        .get_node_key_by_id(graph.get_node(from).unwrap().id)
        .unwrap();
    let second_to = second
        .get_node_key_by_id(graph.get_node(to).unwrap().id)
        .unwrap();
    let second_key = second.find_edge_key(second_from, second_to).unwrap();
    let first_records = first.get_edge(first_key).unwrap().assertions();
    let second_records = second.get_edge(second_key).unwrap().assertions();
    assert_eq!(first_records.len(), 4);
    assert_eq!(second_records.len(), 4);

    let mut first_by_kind: Vec<_> = first_records
        .iter()
        .map(|record| {
            (
                RelationKind::from_predicate_iri(&record.predicate),
                record.statement_id.clone(),
            )
        })
        .collect();
    let mut second_by_kind: Vec<_> = second_records
        .iter()
        .map(|record| {
            (
                RelationKind::from_predicate_iri(&record.predicate),
                record.statement_id.clone(),
            )
        })
        .collect();
    first_by_kind.sort_by_key(|(kind, _)| kind.map(RelationKind::tag));
    second_by_kind.sort_by_key(|(kind, _)| kind.map(RelationKind::tag));
    assert_eq!(
        first_by_kind, second_by_kind,
        "legacy IDs do not depend on process randomness"
    );

    for record in first_records {
        assert_eq!(
            record.provenance_iri.as_deref(),
            Some(crate::graph::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI)
        );
        assert_eq!(record.first_asserted_ms, None);
        assert_eq!(record.asserted_at_ms, None);
        assert!(record.basis.method.is_none());
        assert!(record.basis.evidence.is_empty());
    }
    let canonical = first.to_snapshot_at(0);
    assert!(canonical.edges[0].assertions.is_some());
    assert!(canonical.edges[0].has_legacy_assertion_data() == false);
}

#[test]
fn derived_url_and_domain_records_are_rebuilt_without_stale_assertion_metadata() {
    let mut graph = Graph::new();
    let parent = graph.add_node("https://derived.test/root/".into(), Default::default());
    let child = graph.add_node("https://derived.test/root/item".into(), Default::default());
    // The compatibility snapshot reader rebuilds the old Surface rule
    // outputs. Recorded Resource assertions are a separate placement boundary.
    for sub_kind in [ContainmentSubKind::UrlPath, ContainmentSubKind::Domain] {
        graph.assert_surface_relation_as(
            child,
            parent,
            EdgeAssertion::Containment { sub_kind },
            "urn:old:rule".into(),
            None,
        );
    }
    let pair_keys: Vec<_> = graph
        .retained_relations_between(child, parent)
        .map(|(key, _)| key)
        .collect();
    assert!(!pair_keys.is_empty(), "derived child-to-parent rows exist");
    for (kind, id) in [
        (ContainmentSubKind::UrlPath, "old-url-path-derived-id"),
        (ContainmentSubKind::Domain, "old-domain-derived-id"),
    ] {
        let mut found = false;
        for key in &pair_keys {
            for record in &mut graph.get_relation_mut(*key).unwrap().assertions.statements {
                if record.relation_kind() != RelationKind::Containment(kind) {
                    continue;
                }
                found = true;
                record.statement_id = id.into();
                record.provenance_iri = Some("urn:mere:rule:derived-containment:v1".into());
                record.first_asserted_ms = Some(10);
                record.asserted_at_ms = Some(20);
                record.basis = AssertionBasis {
                    note: Some("must not survive derivation".into()),
                    method: Some(AssertionMethod {
                        iri: "urn:mere:method:derived-containment".into(),
                        version: Some("1".into()),
                    }),
                    evidence: vec!["urn:evidence:stale".into()],
                };
            }
        }
        assert!(found, "fixture includes {kind:?} before rebuilding");
    }

    let restored = Graph::from_snapshot(&graph.to_snapshot_at(0));
    let restored_parent = restored
        .get_node_by_url("https://derived.test/root/")
        .unwrap()
        .0;
    let restored_child = restored
        .get_node_by_url("https://derived.test/root/item")
        .unwrap()
        .0;
    let restored_pairs: Vec<_> = restored
        .retained_relations_between(restored_child, restored_parent)
        .collect();
    assert!(
        !restored_pairs.is_empty(),
        "derived child-to-parent relation is rebuilt"
    );
    let records: Vec<_> = restored_pairs
        .iter()
        .flat_map(|(_, payload)| payload.assertions())
        .collect();
    for kind in [ContainmentSubKind::UrlPath, ContainmentSubKind::Domain] {
        let derived: Vec<_> = records
            .iter()
            .filter(|record| {
                record.predicate
                    == RelationKind::Containment(kind)
                        .predicate_iri()
                        .expect("derived containment has a canonical predicate")
            })
            .collect();
        assert_eq!(derived.len(), 1, "one current derived member for {kind:?}");
        assert!(
            !["old-url-path-derived-id", "old-domain-derived-id",]
                .contains(&derived[0].statement_id.as_str())
        );
        assert_eq!(derived[0].first_asserted_ms, None);
        assert_eq!(derived[0].asserted_at_ms, None);
        assert_eq!(
            derived[0].provenance_iri.as_deref(),
            Some("urn:mere:rule:derived-containment:v1")
        );
        assert_eq!(
            derived[0]
                .basis
                .method
                .as_ref()
                .map(|method| method.iri.as_str()),
            Some("urn:mere:method:derived-containment")
        );
        assert_eq!(
            derived[0]
                .basis
                .method
                .as_ref()
                .and_then(|method| method.version.as_deref()),
            Some("1")
        );
        assert!(derived[0].basis.evidence.is_empty());
    }
}

#[test]
fn snapshot_rebuild_preserves_unmarked_canonical_urlpath_and_domain_claims() {
    let mut graph = Graph::new();
    let parent = graph.add_node("https://canonical.test/root/".into(), Default::default());
    let child = graph.add_node(
        "https://canonical.test/root/item".into(),
        Default::default(),
    );
    let mut expected = Vec::new();

    for (kind, name) in [
        (ContainmentSubKind::UrlPath, "url-path"),
        (ContainmentSubKind::Domain, "domain"),
    ] {
        for (surface, stratum) in [(true, "surface"), (false, "resource")] {
            let edge = if surface {
                graph.assert_surface_relation_as(
                    child,
                    parent,
                    EdgeAssertion::Containment { sub_kind: kind },
                    "urn:initial:rule".into(),
                    None,
                )
            } else {
                graph.assert_relation_as(
                    child,
                    parent,
                    EdgeAssertion::Containment { sub_kind: kind },
                    "urn:initial:rule".into(),
                    None,
                )
            }
            .expect("fixture relation is inserted");
            let payload = graph.get_relation_mut(edge).expect("inserted edge exists");
            let record = payload
                .assertions
                .statements
                .iter_mut()
                .find(|record| record.relation_kind() == RelationKind::Containment(kind))
                .expect("inserted exact assertion exists");
            record.statement_id = format!("canonical-{stratum}-{name}");
            record.provenance_iri = Some(format!("urn:author:{stratum}"));
            record.first_asserted_ms = Some(101);
            record.asserted_at_ms = Some(205);
            record.label = Some(format!("authored {name} {stratum}"));
            record.basis = AssertionBasis {
                note: Some(format!("authored {name} {stratum}")),
                method: Some(AssertionMethod {
                    iri: format!("urn:method:authored:{stratum}"),
                    version: Some("v2".into()),
                }),
                evidence: vec![format!("urn:evidence:{name}:{stratum}")],
            };
            expected.push(record.clone());
        }
    }

    let restored = Graph::from_snapshot(&graph.to_snapshot_at(0));
    for expected_record in expected {
        let (_, actual) = restored
            .find_assertion(&expected_record.statement_id)
            .expect("canonical non-rule assertion survives the ordinary loader");
        assert_eq!(actual, &expected_record);
    }
}

#[test]
fn ordinary_snapshot_rebuild_keeps_withdrawn_rule_identity_terminal() {
    let mut graph = Graph::new();
    let parent = graph.add_node("https://tombstone.test/root/".into(), Default::default());
    let child = graph.add_node(
        "https://tombstone.test/root/item".into(),
        Default::default(),
    );
    graph.assert_surface_relation_as(
        child,
        parent,
        EdgeAssertion::Containment {
            sub_kind: ContainmentSubKind::UrlPath,
        },
        "urn:mere:rule:derived-containment:v1".into(),
        None,
    );
    let held = graph
        .retained_relations_between(child, parent)
        .flat_map(|(_, payload)| payload.held_assertions())
        .find(|record| {
            record.relation_kind() == RelationKind::Containment(ContainmentSubKind::UrlPath)
        })
        .expect("live URL path derivation exists")
        .clone();
    assert!(graph.retract_assertion(&held.statement_id));

    let restored = Graph::from_snapshot(&graph.to_snapshot_at(0));
    let rows: Vec<_> = restored
        .retained_relations_between(child, parent)
        .flat_map(|(_, payload)| payload.assertions())
        .filter(|record| record.statement_id == held.statement_id)
        .collect();
    assert_eq!(
        rows.len(),
        1,
        "the stable derived ID remains globally unique"
    );
    assert!(
        rows[0].withdrawn.is_some(),
        "a retained rule tombstone is terminal"
    );
    assert!(restored.to_snapshot_at(0).assertion_schema == 1);
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
fn ordinary_legacy_sidecar_rebuild_replaces_only_surface_rule_rows() {
    let mut graph = Graph::new();
    let stale_parent = graph.add_node("https://legacy.test/stale/".into(), Default::default());
    let current_parent = graph.add_node("https://legacy.test/current/".into(), Default::default());
    let child = graph.add_node(
        "https://legacy.test/current/item".into(),
        Default::default(),
    );
    graph.assert_surface_relation_as(
        child,
        stale_parent,
        EdgeAssertion::Containment {
            sub_kind: ContainmentSubKind::UrlPath,
        },
        "urn:legacy:producer".into(),
        None,
    );

    let mut legacy = graph.to_snapshot_at(0);
    legacy.assertion_schema = 0;
    let stale_parent_id = graph.get_node(stale_parent).unwrap().id.to_string();
    let child_id = graph.get_node(child).unwrap().id.to_string();
    let edge = legacy
        .edges
        .iter_mut()
        .find(|edge| edge.from_node_id == child_id && edge.to_node_id == stale_parent_id)
        .expect("stale Surface row is serialized");
    edge.assertions = None;
    edge.semantic = None;
    edge.containment = Some(PersistedContainmentEdgeData {
        sub_kinds: vec![PersistedContainmentSubKind::UrlPath],
    });

    let restored = Graph::from_snapshot(&legacy);
    let restored_child = restored
        .get_node_key_by_id(graph.get_node(child).unwrap().id)
        .unwrap();
    let restored_current = restored
        .get_node_key_by_id(graph.get_node(current_parent).unwrap().id)
        .unwrap();
    let restored_stale = restored
        .get_node_key_by_id(graph.get_node(stale_parent).unwrap().id)
        .unwrap();
    assert!(
        restored
            .retained_relations_between(restored_child, restored_stale)
            .flat_map(|(_, payload)| payload.held_assertions())
            .all(|record| record.relation_kind()
                != RelationKind::Containment(ContainmentSubKind::UrlPath)),
        "the old Surface sidecar is identified as rebuildable legacy rule output"
    );
    assert!(
        restored
            .retained_relations_between(restored_child, restored_current)
            .flat_map(|(_, payload)| payload.held_assertions())
            .any(|record| record.relation_kind()
                == RelationKind::Containment(ContainmentSubKind::UrlPath)),
        "the current URL-derived Surface relation is rebuilt"
    );
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
