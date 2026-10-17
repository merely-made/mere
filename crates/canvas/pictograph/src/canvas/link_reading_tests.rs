// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use kernel::graph::{
    ArrangementSubKind, ContainmentSubKind, ImportedSubKind, ProvenanceSubKind, RelationKind,
    SemanticStatement, SemanticStatementSpec, SemanticSubKind, predicate_iri,
};
use kernel::types::{AssertionBasis, AssertionMethod, GraphScope};

#[test]
fn reading_keys_each_stratum_separately_and_preserves_assertion_source_facts() {
    let mut graph = Graph::new();
    let a = Uuid::from_u128(1);
    let b = Uuid::from_u128(2);
    kernel::graph::apply::add_node(
        &mut graph,
        Some(a),
        "https://a.test/".into(),
        Default::default(),
    );
    kernel::graph::apply::add_node(
        &mut graph,
        Some(b),
        "https://b.test/".into(),
        Default::default(),
    );
    let resource_assertion = graph
        .try_assert_semantic_statement(
            graph.get_node_key_by_id(a).unwrap(),
            graph.get_node_key_by_id(b).unwrap(),
            SemanticStatementSpec {
                predicate: predicate_iri(SemanticSubKind::Cites).into(),
                recognized_sub_kind: Some(SemanticSubKind::Cites),
                label: Some("source claim".into()),
                basis: AssertionBasis::default(),
                graph_scope: GraphScope::Custom("urn:scope".into()),
                provenance_iri: Some("urn:asserter".into()),
                asserted_at_ms: Some(42),
            },
        )
        .unwrap()
        .1
        .statement_id;
    graph
        .try_assert_semantic_statement(
            graph.get_node_key_by_id(b).unwrap(),
            graph.get_node_key_by_id(a).unwrap(),
            SemanticStatementSpec {
                predicate: predicate_iri(SemanticSubKind::UserGrouped).into(),
                recognized_sub_kind: Some(SemanticSubKind::UserGrouped),
                ..Default::default()
            },
        )
        .unwrap();

    let before_revision = graph.revision();
    let before_rows = graph.projected_relations().count();
    let pair = read_link(
        &graph,
        a,
        b,
        LinkSource::Unavailable {
            reason: "test source omitted".into(),
        },
    )
    .unwrap();
    assert!(matches!(pair.source, LinkSource::Unavailable { .. }));
    assert_eq!(graph.revision(), before_revision);
    assert_eq!(graph.projected_relations().count(), before_rows);
    let resource_a = graph
        .resource(
            graph
                .shown_resource_id(graph.get_node_key_by_id(a).unwrap())
                .unwrap(),
        )
        .unwrap();
    let resource_b = graph
        .resource(
            graph
                .shown_resource_id(graph.get_node_key_by_id(b).unwrap())
                .unwrap(),
        )
        .unwrap();
    let resource_key = LinkKey::new(
        LinkStratum::Resource,
        resource_a.canonical_iri(),
        resource_b.canonical_iri(),
    );
    let surface_key = LinkKey::new(LinkStratum::Surface, a.to_string(), b.to_string());
    assert!(pair.links.iter().any(|link| link.key == resource_key));
    assert!(pair.links.iter().any(|link| link.key == surface_key));
    assert!(
        pair.links
            .iter()
            .all(|link| link.key.stratum == LinkStratum::Resource
                || link.key.stratum == LinkStratum::Surface)
    );

    let member = pair
        .links
        .iter()
        .find(|link| link.key == resource_key)
        .unwrap()
        .buckets
        .iter()
        .flat_map(|bucket| &bucket.assertions)
        .find(|assertion| assertion.id == resource_assertion)
        .unwrap();
    assert_eq!(member.predicate, predicate_iri(SemanticSubKind::Cites));
    assert_eq!(member.recognized_kind.as_deref(), Some("Cites"));
    assert_eq!(member.asserter.as_deref(), Some("urn:asserter"));
    assert_eq!(member.asserted_at_ms, Some(42));
    assert_eq!(member.scope, LinkGraphScope::Custom("urn:scope".into()));

    let known_source = LinkSource::Available {
        binding: scenograph::SourceBinding {
            authority: "mere:test".into(),
            domain: "graph".into(),
            resource: "fixture".into(),
        },
        revision: scenograph::link::LinkSourceRevision::Available {
            revision: scenograph::PublicSourceRevision::new("fixture-r1"),
        },
    };
    let known = read_link(&graph, a, b, known_source.clone()).unwrap();
    assert_eq!(known.source, known_source);
}

#[test]
fn explicit_history_reads_first_last_withdrawal_and_basis_but_default_is_held_only() {
    let mut graph = Graph::new();
    let a = Uuid::from_u128(41);
    let b = Uuid::from_u128(42);
    kernel::graph::apply::add_node(
        &mut graph,
        Some(a),
        "https://history-a.test/".into(),
        Default::default(),
    );
    kernel::graph::apply::add_node(
        &mut graph,
        Some(b),
        "https://history-b.test/".into(),
        Default::default(),
    );
    let method = AssertionMethod {
        iri: "urn:method:extractor".into(),
        version: Some("v3".into()),
    };
    let mut add = |id: &str, first, last, withdrawn| {
        graph
            .try_assert_persisted_semantic_statement(
                graph.get_node_key_by_id(a).unwrap(),
                graph.get_node_key_by_id(b).unwrap(),
                SemanticStatement {
                    statement_id: id.into(),
                    predicate: predicate_iri(SemanticSubKind::Cites).into(),
                    recognized_sub_kind: Some(SemanticSubKind::Cites),
                    label: Some(format!("note:{id}")),
                    basis: AssertionBasis {
                        note: Some(format!("note:{id}")),
                        method: Some(method.clone()),
                        evidence: vec!["urn:evidence:b".into(), "urn:evidence:a".into()],
                    },
                    graph_scope: GraphScope::Default,
                    provenance_iri: Some("urn:asserter:history".into()),
                    first_asserted_ms: first,
                    asserted_at_ms: last,
                    withdrawn: None,
                },
            )
            .unwrap();
        if withdrawn {
            assert!(graph.apply_imported_semantic_withdrawal_for_pair(
                graph.get_node_key_by_id(a).unwrap(),
                graph.get_node_key_by_id(b).unwrap(),
                id,
                Some(25),
                Some("urn:withdrawer".into()),
            ));
        }
    };
    add("held", Some(5), Some(15), false);
    add("withdrawn", Some(10), Some(20), true);
    add("undated", None, None, false);

    let source = LinkSource::Unavailable {
        reason: "history fixture has no source identity".into(),
    };
    let held = read_link(&graph, a, b, source.clone()).unwrap();
    let history = read_link_history(&graph, a, b, source).unwrap();
    let collect = |reading: &LinkPairReading| {
        reading
            .links
            .iter()
            .flat_map(|link| &link.buckets)
            .flat_map(|bucket| &bucket.assertions)
            .cloned()
            .collect::<Vec<_>>()
    };
    let held = collect(&held);
    let history = collect(&history);
    assert_eq!(held.len(), 2);
    assert!(held.iter().all(|assertion| assertion.withdrawn.is_none()));
    assert_eq!(history.len(), 3);
    let removed = history
        .iter()
        .find(|assertion| assertion.id == "withdrawn")
        .unwrap();
    assert_eq!(removed.first_asserted_ms, Some(10));
    assert_eq!(removed.asserted_at_ms, Some(20));
    assert_eq!(
        removed.withdrawn,
        Some(LinkWithdrawal {
            at_ms: Some(25),
            by: Some("urn:withdrawer".into()),
        })
    );
    assert_eq!(removed.basis.note.as_deref(), Some("note:withdrawn"));
    assert_eq!(removed.basis.method.as_ref().unwrap().iri, method.iri);
    assert_eq!(
        removed.basis.method.as_ref().unwrap().version.as_deref(),
        Some("v3")
    );
    assert_eq!(
        removed.basis.evidence,
        vec!["urn:evidence:b", "urn:evidence:a"]
    );
    assert!(
        history
            .iter()
            .any(|assertion| assertion.first_asserted_ms.is_none())
    );
}

#[test]
fn exact_containment_assertions_keep_direction_ids_and_held_history_projection() {
    let mut graph = Graph::new();
    let a = Uuid::from_u128(9101);
    let b = Uuid::from_u128(9102);
    for (id, address) in [(a, "https://member.test/"), (b, "https://collection.test/")] {
        kernel::graph::apply::add_node(&mut graph, Some(id), address.into(), Default::default());
    }
    let make_record = |id: &str, kind: RelationKind, author: &str| SemanticStatement {
        statement_id: id.to_owned(),
        predicate: kind.predicate_iri().expect("canonical predicate").into(),
        recognized_sub_kind: None,
        label: Some(format!("record {id}")),
        basis: AssertionBasis {
            note: Some(format!("independent basis {id}")),
            ..Default::default()
        },
        graph_scope: GraphScope::Custom("urn:collection:fixture".into()),
        provenance_iri: Some(author.into()),
        first_asserted_ms: Some(100),
        asserted_at_ms: Some(110),
        withdrawn: None,
    };
    for (id, kind, author) in [
        (
            "containment-one",
            RelationKind::Containment(ContainmentSubKind::CollectionMember),
            "urn:author:one",
        ),
        (
            "containment-two",
            RelationKind::Containment(ContainmentSubKind::CollectionMember),
            "urn:author:two",
        ),
        (
            "arrangement-one",
            RelationKind::Arrangement(ArrangementSubKind::FrameMember),
            "urn:author:arrangement",
        ),
        (
            "imported-one",
            RelationKind::Imported(ImportedSubKind::BookmarkFolder),
            "urn:author:imported",
        ),
        (
            "provenance-one",
            RelationKind::Provenance(ProvenanceSubKind::ClippedFrom),
            "urn:author:provenance",
        ),
    ] {
        let record = make_record(id, kind, author);
        graph
            .try_assert_persisted_relation_statement(
                graph.get_node_key_by_id(a).unwrap(),
                graph.get_node_key_by_id(b).unwrap(),
                record,
            )
            .unwrap();
    }
    let withdrawal_actor = graph.write_author().asserter_iri();
    assert!(graph.retract_assertion("containment-two"));

    let held = read_link(
        &graph,
        a,
        b,
        LinkSource::Unavailable {
            reason: "test".into(),
        },
    )
    .unwrap();
    let held_assertions = held
        .links
        .iter()
        .flat_map(|link| &link.buckets)
        .flat_map(|bucket| &bucket.assertions)
        .collect::<Vec<_>>();
    let held_ids = held_assertions
        .iter()
        .map(|assertion| assertion.id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        held_ids,
        BTreeSet::from([
            "arrangement-one",
            "containment-one",
            "imported-one",
            "provenance-one",
        ])
    );
    assert!(
        held_assertions
            .iter()
            .any(|assertion| assertion.id == "containment-one")
    );
    assert!(
        !held_assertions
            .iter()
            .any(|assertion| assertion.id == "containment-two")
    );
    let containment = held_assertions
        .iter()
        .find(|assertion| assertion.id == "containment-one")
        .unwrap();
    assert_eq!(
        containment.predicate,
        RelationKind::Containment(ContainmentSubKind::CollectionMember)
            .predicate_iri()
            .unwrap()
    );
    assert_eq!(containment.recognized_kind, None);
    assert_eq!(containment.asserter.as_deref(), Some("urn:author:one"));
    assert!(
        held.links
            .iter()
            .flat_map(|link| &link.buckets)
            .all(|bucket| {
                bucket.assertions.is_empty()
                    || (bucket.displayed_from == a.to_string()
                        && bucket.displayed_to == b.to_string())
            })
    );

    let history = read_link_history(
        &graph,
        a,
        b,
        LinkSource::Unavailable {
            reason: "test".into(),
        },
    )
    .unwrap();
    let retained = history
        .links
        .iter()
        .flat_map(|link| &link.buckets)
        .flat_map(|bucket| &bucket.assertions)
        .collect::<Vec<_>>();
    assert_eq!(
        retained
            .iter()
            .map(|assertion| assertion.id.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "arrangement-one",
            "containment-one",
            "containment-two",
            "imported-one",
            "provenance-one",
        ])
    );
    assert!(retained.iter().any(|assertion| {
        assertion.id == "containment-two"
            && assertion.withdrawn.as_ref().is_some_and(|withdrawal| {
                withdrawal.by.as_deref() == Some(withdrawal_actor.as_str())
            })
    }));
    assert!(history.links.iter().any(|link| {
        link.buckets.iter().any(|bucket| {
            bucket.displayed_from == a.to_string() && bucket.displayed_to == b.to_string()
        })
    }));
}

#[test]
fn portable_link_reads_bound_held_work_and_materialize_substantial_history() {
    use std::time::{Duration, Instant};

    use kernel::persistence::{PersistedEdge, PersistedEdgeFamily, PersistedSemanticStatement};
    use kernel::types::{AssertionBasis, AssertionMethod, AssertionWithdrawal};

    // Declared acceptance budgets for this debug portable-reader probe:
    // 16 held reads, nearest-rank p95 <100 ms; one explicit history materialization <2 s;
    // serialized portable history <64 MiB. These do not make a UI frame-rate claim.
    const WITHDRAWN: usize = 50_000;
    const HELD_READS: usize = 16;
    const HELD_P95_BUDGET: Duration = Duration::from_millis(100);
    const HISTORY_BUDGET: Duration = Duration::from_secs(2);
    const PORTABLE_BUDGET: usize = 64 * 1024 * 1024;

    let mut graph = Graph::new();
    let from = Uuid::from_u128(50_001);
    let to = Uuid::from_u128(50_002);
    kernel::graph::apply::add_node(
        &mut graph,
        Some(from),
        "https://history-budget.test/from".into(),
        Default::default(),
    );
    kernel::graph::apply::add_node(
        &mut graph,
        Some(to),
        "https://history-budget.test/to".into(),
        Default::default(),
    );
    let predicate = RelationKind::Containment(ContainmentSubKind::CollectionMember)
        .predicate_iri()
        .unwrap()
        .to_owned();
    let actor = "urn:history-budget:author";
    let method = AssertionMethod {
        iri: "urn:history-budget:method".into(),
        version: Some("v1".into()),
    };
    let make_record = |id: String,
                       note: String,
                       first: u64,
                       last: u64,
                       evidence: String,
                       withdrawn: Option<AssertionWithdrawal>| {
        PersistedSemanticStatement {
            statement_id: id,
            predicate: predicate.clone(),
            recognized_sub_kind: None,
            label: Some(note.clone()),
            basis: AssertionBasis {
                note: Some(note),
                method: Some(method.clone()),
                evidence: vec![evidence],
            },
            graph_scope: GraphScope::Custom("urn:history-budget:scope".into()),
            provenance_iri: Some(actor.into()),
            first_asserted_ms: Some(first),
            asserted_at_ms: Some(last),
            withdrawn,
        }
    };
    let mut assertions = (0..WITHDRAWN)
        .map(|index| {
            make_record(
                format!("history-{index:05}"),
                format!("retained history reason {index:05}"),
                index as u64,
                index as u64 + 10,
                "urn:history-budget:evidence".into(),
                Some(AssertionWithdrawal {
                    at_ms: Some(index as u64 + 20),
                    by: Some("urn:history-budget:withdrawer".into()),
                }),
            )
        })
        .collect::<Vec<_>>();
    assertions.extend(
        [
            ("held-reason-a", "independent held reason A", 60_001),
            ("held-reason-b", "independent held reason B", 60_002),
        ]
        .map(|(id, note, at)| {
            make_record(
                id.into(),
                note.into(),
                at,
                at,
                format!("urn:history-budget:{id}"),
                None,
            )
        }),
    );
    let mut snapshot = graph.to_snapshot_at(123);
    snapshot.edges.push(PersistedEdge {
        from_node_id: from.to_string(),
        to_node_id: to.to_string(),
        families: vec![PersistedEdgeFamily::Containment],
        assertions: Some(assertions),
        semantic: None,
        traversal: None,
        containment: None,
        arrangement: None,
        imported: None,
        provenance: None,
    });
    snapshot.assertion_schema = 1;
    let graph = Graph::try_from_recorded_snapshot(&snapshot)
        .expect("canonical checked snapshot retains exact family history");
    let source_before = serde_json::to_vec(&graph.to_snapshot_at(123)).unwrap();

    let mut held_durations = Vec::with_capacity(HELD_READS);
    for _ in 0..HELD_READS {
        let start = Instant::now();
        let held = read_link(
            &graph,
            from,
            to,
            LinkSource::Unavailable {
                reason: "fixed history-budget fixture".into(),
            },
        )
        .unwrap();
        held_durations.push(start.elapsed());
        let records = held
            .links
            .iter()
            .flat_map(|link| &link.buckets)
            .flat_map(|bucket| &bucket.assertions)
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 2);
        assert_eq!(
            records
                .iter()
                .map(|record| record.id.as_str())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(["held-reason-a", "held-reason-b"])
        );
    }
    held_durations.sort_unstable();
    let held_p95 = held_durations[(HELD_READS * 95).div_ceil(100) - 1];

    let history_start = Instant::now();
    let history = read_link_history(
        &graph,
        from,
        to,
        LinkSource::Unavailable {
            reason: "fixed history-budget fixture".into(),
        },
    )
    .unwrap();
    let history_elapsed = history_start.elapsed();
    let records = history
        .links
        .iter()
        .flat_map(|link| &link.buckets)
        .flat_map(|bucket| &bucket.assertions)
        .collect::<Vec<_>>();
    assert_eq!(records.len(), WITHDRAWN + 2);
    let record_ids = records
        .iter()
        .map(|record| record.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut expected_ids = (0..WITHDRAWN)
        .map(|index| format!("history-{index:05}"))
        .collect::<BTreeSet<_>>();
    expected_ids.extend(["held-reason-a".into(), "held-reason-b".into()]);
    assert_eq!(
        record_ids,
        expected_ids.iter().map(String::as_str).collect()
    );
    for record in &records {
        assert_eq!(record.predicate, predicate);
        assert_eq!(
            record.scope,
            scenograph::link::LinkGraphScope::Custom("urn:history-budget:scope".into())
        );
        assert_eq!(record.asserter.as_deref(), Some(actor));
        assert_eq!(
            record
                .basis
                .method
                .as_ref()
                .map(|value| (&value.iri, &value.version)),
            Some((&method.iri, &method.version))
        );
        assert_eq!(record.basis.evidence.len(), 1);
        if let Some(index) = record.id.strip_prefix("history-") {
            let index = index.parse::<u64>().unwrap();
            assert_eq!(
                record.basis.note.as_deref(),
                Some(format!("retained history reason {index:05}").as_str())
            );
            assert_eq!(record.first_asserted_ms, Some(index));
            assert_eq!(record.asserted_at_ms, Some(index + 10));
            assert_eq!(record.basis.evidence, vec!["urn:history-budget:evidence"]);
            let withdrawal = record
                .withdrawn
                .as_ref()
                .expect("history tombstone retained");
            assert_eq!(withdrawal.at_ms, Some(index + 20));
            assert_eq!(
                withdrawal.by.as_deref(),
                Some("urn:history-budget:withdrawer")
            );
        } else {
            assert!(record.withdrawn.is_none());
            let held = match record.id.as_str() {
                "held-reason-a" => (
                    "independent held reason A",
                    "urn:history-budget:held-reason-a",
                ),
                "held-reason-b" => (
                    "independent held reason B",
                    "urn:history-budget:held-reason-b",
                ),
                other => panic!("unexpected held assertion {other}"),
            };
            assert_eq!(record.basis.note.as_deref(), Some(held.0));
            assert_eq!(record.basis.evidence, vec![held.1]);
        }
    }
    let portable_bytes = serde_json::to_vec(&history).unwrap().len();
    assert_eq!(
        serde_json::to_vec(&graph.to_snapshot_at(123)).unwrap(),
        source_before
    );
    println!(
        "portable link history probe: held_reads={HELD_READS} held_p95_ms={} history_records={} history_ms={} serialized_bytes={portable_bytes}",
        held_p95.as_millis(),
        records.len(),
        history_elapsed.as_millis(),
    );
    assert!(
        held_p95 < HELD_P95_BUDGET,
        "held p95 {held_p95:?} exceeds {HELD_P95_BUDGET:?}"
    );
    assert!(
        history_elapsed < HISTORY_BUDGET,
        "history read {history_elapsed:?} exceeds {HISTORY_BUDGET:?}"
    );
    assert!(
        portable_bytes < PORTABLE_BUDGET,
        "portable history {portable_bytes} bytes exceeds {PORTABLE_BUDGET}"
    );
}

#[test]
fn summary_free_read_keeps_traversal_members_without_copying_event_history() {
    let mut payload = EdgePayload::new();
    payload.push_traversal(kernel::graph::edge_data::Traversal {
        timestamp_ms: 9,
        trigger: kernel::graph::edge_taxonomy::NavigationTrigger::LinkClick,
    });

    let (members, summaries) = legacy_data(&payload, false);
    assert!(summaries.is_empty());
    assert!(members.iter().any(|member| member.family == "Traversal"));

    let (_, summaries) = legacy_data(&payload, true);
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].family, "Traversal");
    assert_eq!(summaries[0].data["traversals"].as_array().unwrap().len(), 1);

    // Every relation family is represented by exact records, including
    // inputs normalized from old snapshots. These records never duplicate
    // themselves as compatibility members or family summaries.
    for assertion in [
        kernel::graph::EdgeAssertion::Semantic {
            sub_kind: kernel::graph::SemanticSubKind::Cites,
            label: Some("exact note".into()),
            decay_progress: None,
        },
        kernel::graph::EdgeAssertion::Containment {
            sub_kind: ContainmentSubKind::CollectionMember,
        },
    ] {
        let mut exact = EdgePayload::new();
        assert!(exact.assert_relation(assertion));
        assert_eq!(exact.assertions().len(), 1);
        let (members, summaries) = legacy_data(&exact, true);
        assert!(members.is_empty());
        assert!(summaries.is_empty());
    }
    let (members, summaries) = legacy_data(&EdgePayload::new(), true);
    assert!(members.is_empty());
    assert!(summaries.is_empty());
}
