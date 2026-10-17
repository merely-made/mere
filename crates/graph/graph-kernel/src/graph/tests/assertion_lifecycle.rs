// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::super::*;

fn same_reason(at_ms: Option<u64>) -> SemanticStatementSpec {
    SemanticStatementSpec {
        predicate: predicate_iri(SemanticSubKind::Cites).into(),
        recognized_sub_kind: Some(SemanticSubKind::Cites),
        graph_scope: GraphScope::User,
        provenance_iri: Some("urn:author:one".into()),
        label: Some("same reason".into()),
        asserted_at_ms: at_ms,
        basis: crate::types::AssertionBasis {
            note: Some("same reason".into()),
            evidence: vec!["urn:evidence:a".into(), "urn:evidence:b".into()],
            ..Default::default()
        },
    }
}

#[test]
fn reason_key_reuses_method_version_and_evidence_order_but_splits_changed_reason() {
    let mut graph = Graph::new();
    let from = graph.add_node("https://a.test/".into(), Default::default());
    let to = graph.add_node("https://b.test/".into(), Default::default());

    let (_, ensured) = graph
        .try_assert_semantic_statement(from, to, same_reason(Some(10)))
        .unwrap();
    let (_, unchanged) = graph
        .try_assert_semantic_statement(from, to, same_reason(Some(10)))
        .unwrap();
    assert_eq!(unchanged.statement_id, ensured.statement_id);
    assert!(!unchanged.changed, "renderer-style ensure is a no-op");

    let mut same_key = same_reason(Some(11));
    same_key.basis.evidence.reverse();
    let (_, same_reason_new_order) = graph
        .try_assert_semantic_reason(from, to, same_key)
        .unwrap();
    assert_eq!(same_reason_new_order.statement_id, ensured.statement_id);
    assert!(
        same_reason_new_order.changed,
        "explicit confirmation can advance last time"
    );

    let mut duplicate_evidence = same_reason(Some(11));
    duplicate_evidence
        .basis
        .evidence
        .push("urn:evidence:a".into());
    let (_, duplicate_ref) = graph
        .try_assert_semantic_reason(from, to, duplicate_evidence)
        .unwrap();
    assert_eq!(duplicate_ref.statement_id, ensured.statement_id);

    let mut method_version = same_reason(Some(12));
    method_version.basis.method = Some(crate::types::AssertionMethod {
        iri: "urn:method:extractor".into(),
        version: Some("v1".into()),
    });
    let (_, method_one) = graph
        .try_assert_semantic_reason(from, to, method_version)
        .unwrap();
    let mut method_version = same_reason(Some(13));
    method_version.basis.method = Some(crate::types::AssertionMethod {
        iri: "urn:method:extractor".into(),
        version: Some("v2".into()),
    });
    let (_, method_two) = graph
        .try_assert_semantic_reason(from, to, method_version)
        .unwrap();
    assert_eq!(method_one.statement_id, method_two.statement_id);
    assert_eq!(
        graph
            .find_semantic_statement(&method_two.statement_id)
            .unwrap()
            .1
            .basis
            .method
            .as_ref()
            .unwrap()
            .version
            .as_deref(),
        Some("v2")
    );

    let mut changed_evidence = same_reason(Some(14));
    changed_evidence.basis.evidence = vec!["urn:evidence:c".into()];
    let (_, evidence_reason) = graph
        .try_assert_semantic_reason(from, to, changed_evidence)
        .unwrap();
    assert_ne!(evidence_reason.statement_id, method_two.statement_id);
    let mut changed_method = same_reason(Some(15));
    changed_method.basis.method = Some(crate::types::AssertionMethod {
        iri: "urn:method:other".into(),
        version: None,
    });
    let (_, other_method) = graph
        .try_assert_semantic_reason(from, to, changed_method)
        .unwrap();
    assert_ne!(other_method.statement_id, method_two.statement_id);
    assert_eq!(
        graph
            .find_semantic_statement(&ensured.statement_id)
            .unwrap()
            .1
            .first_asserted_ms,
        Some(10)
    );
    assert_eq!(
        graph
            .find_semantic_statement(&ensured.statement_id)
            .unwrap()
            .1
            .asserted_at_ms,
        Some(11),
        "the explicit exact-reason reassertion advances last time"
    );
    assert_eq!(
        graph
            .find_semantic_statement(&method_one.statement_id)
            .unwrap()
            .1
            .first_asserted_ms,
        Some(12)
    );
}

#[test]
fn exact_reconfirm_withdraw_and_stale_import_preserve_lifecycle_history() {
    let mut graph = Graph::new();
    let from = graph.add_node("https://a.test/".into(), Default::default());
    let to = graph.add_node("https://b.test/".into(), Default::default());
    let (_, first) = graph
        .try_assert_semantic_statement(from, to, same_reason(Some(10)))
        .unwrap();
    let original = graph
        .find_semantic_statement(&first.statement_id)
        .unwrap()
        .1
        .clone();
    let mut other_reason = same_reason(Some(12));
    other_reason.basis.evidence = vec!["urn:evidence:other".into()];
    let (_, other) = graph
        .try_assert_semantic_reason(from, to, other_reason)
        .unwrap();

    assert!(graph.reconfirm_semantic_assertion(
        &first.statement_id,
        original.label.clone(),
        Some(15)
    ));
    let confirmed = graph
        .find_semantic_statement(&first.statement_id)
        .unwrap()
        .1;
    assert_eq!(confirmed.first_asserted_ms, Some(10));
    assert_eq!(confirmed.asserted_at_ms, Some(15));

    assert!(graph.withdraw_semantic_assertion(
        &first.statement_id,
        Some(20),
        Some("urn:author:one".into())
    ));
    let tombstone = graph
        .find_semantic_statement(&first.statement_id)
        .unwrap()
        .1;
    assert_eq!(tombstone.first_asserted_ms, Some(10));
    assert_eq!(tombstone.asserted_at_ms, Some(15));
    assert_eq!(tombstone.withdrawn.as_ref().unwrap().at_ms, Some(20));
    assert_eq!(
        tombstone.withdrawn.as_ref().unwrap().by.as_deref(),
        Some("urn:author:one")
    );
    assert!(
        graph
            .find_held_semantic_statement(&first.statement_id)
            .is_none()
    );
    assert_eq!(
        graph.projected_relations().count(),
        1,
        "only the other held reason projects"
    );
    assert!(!graph.reconfirm_semantic_assertion(&first.statement_id, None, Some(30)));

    // An older carried assertion with the withdrawn ID is stale, even though
    // its claim is otherwise valid.
    assert!(
        graph
            .try_assert_persisted_semantic_statement(from, to, original)
            .is_ok()
    );
    assert_eq!(
        graph
            .find_semantic_statement(&first.statement_id)
            .unwrap()
            .1
            .withdrawn
            .as_ref()
            .unwrap()
            .at_ms,
        Some(20)
    );
    assert!(!graph.apply_imported_semantic_withdrawal("urn:unknown", None, None));
    assert!(graph.find_semantic_statement("urn:unknown").is_none());

    let persisted = graph.to_snapshot().resource_edges;
    assert!(
        persisted
            .iter()
            .flat_map(|edge| edge.retained_assertions())
            .any(|s| { s.statement_id == first.statement_id && s.withdrawn.is_some() })
    );
    assert!(
        persisted
            .iter()
            .flat_map(|edge| edge.retained_assertions())
            .any(|s| { s.statement_id == other.statement_id && s.withdrawn.is_none() })
    );

    let (_, fresh_reason) = graph
        .try_assert_semantic_reason(from, to, same_reason(Some(31)))
        .unwrap();
    assert_ne!(fresh_reason.statement_id, first.statement_id);
    assert!(
        graph
            .find_held_semantic_statement(&fresh_reason.statement_id)
            .is_some()
    );
}

#[test]
fn authored_reason_reconfirm_stamps_missing_time_but_membership_ensure_does_not() {
    let mut graph = Graph::new();
    let from = graph.add_node("https://a.test/".into(), Default::default());
    let to = graph.add_node("https://b.test/".into(), Default::default());
    let (_, initial) = graph
        .try_assert_semantic_statement(from, to, same_reason(Some(0)))
        .unwrap();
    let (_, confirmed) = graph
        .try_assert_semantic_reason(from, to, same_reason(None))
        .unwrap();
    assert_eq!(confirmed.statement_id, initial.statement_id);
    assert!(confirmed.changed);
    let last = graph
        .find_semantic_statement(&initial.statement_id)
        .unwrap()
        .1
        .asserted_at_ms
        .unwrap();
    assert!(last > 0);
    let (_, ensured) = graph
        .try_assert_semantic_statement(from, to, same_reason(None))
        .unwrap();
    assert_eq!(ensured.statement_id, initial.statement_id);
    assert!(!ensured.changed);
    let statement = graph
        .find_semantic_statement(&initial.statement_id)
        .unwrap()
        .1;
    assert_eq!(statement.first_asserted_ms, Some(0));
    assert_eq!(statement.asserted_at_ms, Some(last));
}

#[test]
fn plain_claim_is_an_empty_reason_key_and_tombstone_only_buckets_leave_topology() {
    let mut graph = Graph::new();
    let a = graph.add_node("https://a.test/".into(), Default::default());
    let b = graph.add_node("https://b.test/".into(), Default::default());
    let c = graph.add_node("https://c.test/".into(), Default::default());
    let d = graph.add_node("https://d.test/".into(), Default::default());

    let mut plain = same_reason(Some(1));
    plain.basis = Default::default();
    plain.label = Some("plain claim note".into());
    let (_, first) = graph
        .try_assert_semantic_reason(a, b, plain.clone())
        .unwrap();
    plain.asserted_at_ms = None;
    plain.label = Some("updated note".into());
    let (_, ensured) = graph
        .try_assert_semantic_reason(a, b, plain.clone())
        .unwrap();
    assert_eq!(ensured.statement_id, first.statement_id);

    assert!(graph.withdraw_semantic_assertion(&first.statement_id, Some(2), None));
    let (_, withdrawn_again) = graph.try_assert_semantic_reason(a, b, plain).unwrap();
    assert_ne!(withdrawn_again.statement_id, first.statement_id);
    assert!(
        graph
            .find_semantic_statement(&first.statement_id)
            .unwrap()
            .1
            .withdrawn
            .is_some()
    );
    assert!(graph.append_traversal(a, b, NavigationTrigger::LinkClick, Some(2)));
    assert!(graph.in_neighbors(b).any(|neighbor| neighbor == a));
    assert!(
        graph.neighbors_undirected(b).any(|neighbor| neighbor == a),
        "incoming-only arcs are undirected neighbors"
    );
    assert!(graph.shortest_path(b, a).is_some());

    let (_, dead) = graph
        .try_assert_semantic_reason(c, d, same_reason(Some(3)))
        .unwrap();
    assert!(matches!(
        graph.find_semantic_statement(&dead.statement_id),
        Some((RelationKey::Resource(_), _))
    ));
    assert!(graph.withdraw_semantic_assertion(&dead.statement_id, Some(4), None));
    assert!(!graph.has_edge_between(c, d));
    assert!(!graph.is_reachable(c, d));
    assert!(graph.out_neighbors(c).next().is_none());
    assert!(graph.in_neighbors(d).next().is_none());
    assert!(graph.neighbors_undirected(c).next().is_none());
    assert!(graph.orphan_node_keys().contains(&c));
    assert!(
        graph
            .weakly_connected_components()
            .iter()
            .any(|part| part == &vec![c])
    );
    assert!(
        graph
            .strongly_connected_components()
            .iter()
            .any(|part| part == &vec![d])
    );
    assert!(graph.shortest_path(c, d).is_none());
    assert!(!graph.hop_distances_from(c).contains_key(&d));
    let c_id = graph.get_node(c).unwrap().id;
    let d_id = graph.get_node(d).unwrap().id;
    assert_eq!(graph.component_members(c_id, &[]), vec![c_id]);
    assert_eq!(graph.ego_members(c_id, 1, &[]), vec![c_id]);
    assert_eq!(graph.component_members(d_id, &[]), vec![d_id]);
    assert!(
        graph.resource_relations().any(|(_, _, _, payload)| payload
            .semantic_statements()
            .iter()
            .any(|s| s.statement_id == dead.statement_id)),
        "raw Resource bucket access retains the tombstone history"
    );
    assert!(
        graph
            .projected_relations()
            .all(|(_, row)| row.from != c || row.to != d)
    );
    assert!(graph.projected_relations_between(c, d).next().is_none());
    assert!(graph.retained_relations_between(c, d).any(|(_, payload)| {
        payload
            .semantic_statements()
            .iter()
            .any(|statement| statement.statement_id == dead.statement_id)
    }));
    assert!(
        graph
            .to_snapshot()
            .resource_edges
            .iter()
            .flat_map(|edge| edge.retained_assertions())
            .any(|s| s.statement_id == dead.statement_id && s.withdrawn.is_some())
    );

    assert!(graph.append_traversal(c, d, NavigationTrigger::LinkClick, Some(5)));
    assert!(
        graph.has_edge_between(c, d),
        "another held family on the bucket remains topological"
    );
    assert!(graph.is_reachable(c, d));
}

#[test]
fn exact_metadata_note_edits_and_clears_basis_note_through_snapshot_roundtrip() {
    let mut graph = Graph::new();
    let from = graph.add_node("https://a.test/".into(), Default::default());
    let to = graph.add_node("https://b.test/".into(), Default::default());
    let mut spec = same_reason(Some(10));
    spec.basis.method = Some(crate::types::AssertionMethod {
        iri: "urn:method:extractor".into(),
        version: Some("v1".into()),
    });
    let (_, assertion) = graph.try_assert_semantic_reason(from, to, spec).unwrap();

    assert!(graph.update_assertion_metadata(
        &assertion.statement_id,
        Some("edited".into()),
        Some(11)
    ));
    let edited = graph
        .find_semantic_statement(&assertion.statement_id)
        .unwrap()
        .1;
    assert_eq!(edited.label.as_deref(), Some("edited"));
    assert_eq!(edited.basis.note.as_deref(), Some("edited"));
    assert_eq!(edited.basis.evidence, ["urn:evidence:a", "urn:evidence:b"]);

    assert!(graph.update_assertion_metadata(&assertion.statement_id, None, Some(12)));
    let json = serde_json::to_vec(&graph.to_snapshot()).unwrap();
    let snapshot = serde_json::from_slice(&json).unwrap();
    let restored = Graph::from_snapshot(&snapshot);
    let restored = restored
        .find_semantic_statement(&assertion.statement_id)
        .unwrap()
        .1;
    assert_eq!(restored.label, None);
    assert_eq!(restored.basis.note, None);
    assert_eq!(
        restored.basis.method.as_ref().unwrap().version.as_deref(),
        Some("v1")
    );
    assert_eq!(
        restored.basis.evidence,
        ["urn:evidence:a", "urn:evidence:b"]
    );
}

#[test]
fn authored_missing_time_is_stamped_but_imported_missing_time_stays_undated() {
    let mut graph = Graph::new();
    let from = graph.add_node("https://a.test/".into(), Default::default());
    let to = graph.add_node("https://b.test/".into(), Default::default());
    let (_, authored) = graph
        .try_assert_semantic_reason(from, to, same_reason(None))
        .unwrap();
    let authored = graph
        .find_semantic_statement(&authored.statement_id)
        .unwrap()
        .1;
    assert!(authored.first_asserted_ms.is_some());
    assert_eq!(authored.first_asserted_ms, authored.asserted_at_ms);

    let imported = SemanticStatement::new(
        "urn:imported".into(),
        None,
        None,
        GraphScope::Default,
        Some("urn:source".into()),
        None,
    );
    let imported_id = imported.statement_id.clone();
    assert!(
        graph
            .try_assert_persisted_semantic_statement(from, to, imported)
            .is_ok()
    );
    let imported = graph.find_semantic_statement(&imported_id).unwrap().1;
    assert_eq!(imported.first_asserted_ms, None);
    assert_eq!(imported.asserted_at_ms, None);
}

#[test]
fn imported_withdrawal_checks_resource_bucket_pair_ownership() {
    let mut graph = Graph::new();
    let from = graph.add_node("https://a.test/".into(), Default::default());
    let to = graph.add_node("https://b.test/".into(), Default::default());
    let other_from = graph.add_node("https://c.test/".into(), Default::default());
    let other_to = graph.add_node("https://d.test/".into(), Default::default());
    let mut spec = same_reason(Some(10));
    spec.predicate = "urn:open:resource-predicate".into();
    spec.recognized_sub_kind = None;
    let (owner, assertion) = graph
        .try_assert_semantic_reason(from, to, spec.clone())
        .unwrap();
    assert!(matches!(owner, RelationKey::Resource(_)));

    // Compatibility exact-ID withdrawal follows the recorded owner even when
    // the caller's current surface pair no longer projects to that resource.
    let statement_id = assertion.statement_id;
    assert!(graph.retract_semantic_statement(other_from, other_to, &statement_id));
    assert!(graph.find_held_semantic_statement(&statement_id).is_none());

    // The explicit current-pair import API still rejects wrong-pair delivery.
    let (_, next) = graph.try_assert_semantic_reason(from, to, spec).unwrap();
    assert!(!graph.apply_imported_semantic_withdrawal_for_pair(
        other_from,
        other_to,
        &next.statement_id,
        None,
        Some("urn:source".into()),
    ));
    assert!(
        graph
            .find_held_semantic_statement(&next.statement_id)
            .is_some()
    );
    assert!(graph.apply_imported_semantic_withdrawal_for_pair(
        from,
        to,
        &next.statement_id,
        None,
        Some("urn:source".into()),
    ));
    assert!(
        graph
            .find_held_semantic_statement(&next.statement_id)
            .is_none()
    );
}

#[test]
fn carried_tombstone_withdraws_exact_held_id_and_stale_assert_cannot_revive_it() {
    let mut graph = Graph::new();
    let from = graph.add_node("https://a.test/".into(), Default::default());
    let to = graph.add_node("https://b.test/".into(), Default::default());
    let (_, assertion) = graph
        .try_assert_semantic_statement(from, to, same_reason(Some(10)))
        .unwrap();
    let held = graph
        .find_semantic_statement(&assertion.statement_id)
        .unwrap()
        .1
        .clone();
    let mut tombstone = held.clone();
    tombstone.first_asserted_ms = None;
    tombstone.withdrawn = Some(crate::types::AssertionWithdrawal {
        at_ms: None,
        by: Some("urn:source".into()),
    });

    assert!(
        graph
            .try_assert_persisted_semantic_statement(from, to, tombstone)
            .is_ok()
    );
    let stored = graph
        .find_semantic_statement(&assertion.statement_id)
        .unwrap()
        .1;
    assert_eq!(
        stored.first_asserted_ms,
        Some(10),
        "local first time is stable"
    );
    assert_eq!(stored.withdrawn.as_ref().unwrap().at_ms, None);
    assert!(
        graph
            .find_held_semantic_statement(&assertion.statement_id)
            .is_none()
    );

    assert!(
        graph
            .try_assert_persisted_semantic_statement(from, to, held)
            .is_ok()
    );
    assert!(
        graph
            .find_held_semantic_statement(&assertion.statement_id)
            .is_none()
    );
    assert_eq!(
        graph
            .find_semantic_statement(&assertion.statement_id)
            .unwrap()
            .1
            .withdrawn
            .as_ref()
            .unwrap()
            .by
            .as_deref(),
        Some("urn:source")
    );
}
