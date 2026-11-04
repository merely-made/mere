// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn legacy_journal_assertions_use_each_entry_author_on_full_and_prefix_replay() {
    let mut journal = GraphJournal::new();
    journal.record(add(1, "https://a.test/"));
    journal.record(add(2, "https://b.test/"));
    let recognized = CapturedDelta::ReplayAssertRelationByIds {
        from_id: Uuid::from_u128(1).to_string(),
        to_id: Uuid::from_u128(2).to_string(),
        assertion: EdgeAssertion::Semantic {
            sub_kind: SemanticSubKind::Cites,
            label: None,
        },
    };
    let open = CapturedDelta::ReplayAssertSemanticPredicateByIds {
        from_id: Uuid::from_u128(1).to_string(),
        to_id: Uuid::from_u128(2).to_string(),
        predicate: "https://vocab.test/mentions".into(),
    };
    let alice = Author::person("https://people.test/alice");
    let bob = Author::person("https://people.test/bob");
    for author in [&alice, &bob] {
        journal.record_as(author.clone(), recognized.clone());
        journal.record_as(author.clone(), open.clone());
    }
    let attributions = |graph: &Graph| {
        let a = graph.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
        let b = graph.get_node_key_by_id(Uuid::from_u128(2)).unwrap();
        let edge = graph.find_edge_key(a, b).unwrap();
        let mut values: Vec<_> = graph
            .get_edge(edge)
            .unwrap()
            .semantic_statements()
            .iter()
            .map(|statement| {
                (
                    statement.predicate.clone(),
                    statement.provenance_iri.clone().unwrap(),
                )
            })
            .collect();
        values.sort();
        values
    };
    let full = journal.replay();
    let full_attributions = attributions(&full);
    assert_eq!(full_attributions.len(), 4);
    for author in [&alice, &bob] {
        assert_eq!(
            full_attributions
                .iter()
                .filter(|(_, iri)| iri == &author.asserter_iri())
                .count(),
            2
        );
    }
    let mut prefix = journal.snapshot_at(Seq(4)).unwrap().unwrap();
    let prefix_attributions = attributions(&prefix);
    assert_eq!(prefix_attributions.len(), 2);
    assert!(
        prefix_attributions
            .iter()
            .all(|(_, iri)| iri == &alice.asserter_iri())
    );
    journal.replay_from(Seq(4), &mut prefix).unwrap();
    assert_eq!(attributions(&prefix), full_attributions);
    let unattributed =
        replay_captured_deltas(journal.entries().iter().map(|entry| entry.delta.clone()));
    let legacy_attributions = attributions(&unattributed);
    assert_eq!(
        legacy_attributions.len(),
        2,
        "without envelopes attribution remains unknown"
    );
    assert!(
        legacy_attributions
            .iter()
            .all(|(_, iri)| iri == crate::graph::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI)
    );
}

#[test]
fn exact_statement_capture_preserves_source_over_journal_recorder() {
    let mut source = replay_captured_deltas([add(1, "https://a.test/"), add(2, "https://b.test/")]);
    let a = source.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
    let b = source.get_node_key_by_id(Uuid::from_u128(2)).unwrap();
    let source_iri = "https://source.test/page";
    source
        .assert_surface_semantic_statement(
            a,
            b,
            super::super::super::SemanticStatementSpec {
                predicate: super::super::super::predicate_iri(SemanticSubKind::Cites).into(),
                recognized_sub_kind: Some(SemanticSubKind::Cites),
                provenance_iri: Some(source_iri.into()),
                asserted_at_ms: Some(100),
                ..Default::default()
            },
        )
        .unwrap();
    let expected = source.persisted_edges_between(a, b);
    let exact = CapturedDelta::ReplaySetEdgesByIds {
        from_id: Uuid::from_u128(1).to_string(),
        to_id: Uuid::from_u128(2).to_string(),
        edges: expected.clone(),
    };
    let recorder = Author::engine("extractor", "9");
    assert_ne!(source_iri, recorder.asserter_iri());
    let mut journal = GraphJournal::new();
    journal.record(add(1, "https://a.test/"));
    journal.record(add(2, "https://b.test/"));
    journal.record_as(recorder.clone(), exact.clone());
    let restored = journal.replay();
    let restored_a = restored.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
    let restored_b = restored.get_node_key_by_id(Uuid::from_u128(2)).unwrap();
    assert_eq!(
        restored.persisted_edges_between(restored_a, restored_b),
        expected
    );
    let mut direct = Graph::new();
    crate::graph::capture::replay_captured_deltas_as_onto(
        &mut direct,
        &recorder,
        [add(1, "https://a.test/"), add(2, "https://b.test/"), exact],
    );
    let direct_a = direct.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
    let direct_b = direct.get_node_key_by_id(Uuid::from_u128(2)).unwrap();
    assert_eq!(direct.persisted_edges_between(direct_a, direct_b), expected);
}

#[test]
fn legacy_exact_capture_attribution_matches_checkpoint_replay() {
    use crate::graph::SemanticStatementSpec;

    let recorder = Author::engine("legacy-extractor", "1");
    for source in [Some("https://source.test/page"), None] {
        let mut template =
            replay_captured_deltas([add(1, "https://a.test/"), add(2, "https://b.test/")]);
        let from = template.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
        let to = template.get_node_key_by_id(Uuid::from_u128(2)).unwrap();
        template
            .assert_surface_semantic_statement(
                from,
                to,
                SemanticStatementSpec {
                    predicate: super::super::super::predicate_iri(SemanticSubKind::Cites).into(),
                    recognized_sub_kind: Some(SemanticSubKind::Cites),
                    provenance_iri: Some("https://template.test/".into()),
                    asserted_at_ms: Some(100),
                    ..Default::default()
                },
            )
            .unwrap();
        let mut old_edges = template.persisted_edges_between(from, to);
        let old_statement = &mut old_edges[0].retained_assertions_mut()[0];
        old_statement.statement_id = "legacy-exact-handle".into();
        old_statement.provenance_iri = source.map(str::to_owned);
        if source.is_none() {
            let exact = CapturedDelta::ReplaySetEdgesByIds {
                from_id: Uuid::from_u128(1).to_string(),
                to_id: Uuid::from_u128(2).to_string(),
                edges: old_edges.clone(),
            };
            let mut baseline = template.clone();
            super::super::super::capture::replay_captured_deltas_onto(
                &mut baseline,
                [exact.clone()],
            );
            let mut baseline_journal = GraphJournal::new();
            baseline_journal.record_as(recorder.clone(), exact);
            baseline_journal.replay_from(Seq(0), &mut baseline).unwrap();
            assert_eq!(
                baseline.persisted_edges_between(from, to)[0].retained_assertions()[0]
                    .provenance_iri
                    .as_deref(),
                Some(super::super::super::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI),
                "an unattributed baseline handle stays unknown"
            );
        }
        let mut journal = GraphJournal::new();
        journal.record(add(1, "https://a.test/"));
        journal.record(add(2, "https://b.test/"));
        journal.record_as(
            recorder.clone(),
            CapturedDelta::ReplayAssertRelationByIds {
                from_id: Uuid::from_u128(1).to_string(),
                to_id: Uuid::from_u128(2).to_string(),
                assertion: EdgeAssertion::Semantic {
                    sub_kind: SemanticSubKind::Cites,
                    label: None,
                },
            },
        );
        let prefix = journal.snapshot_at(Seq(3)).unwrap().unwrap();
        let prefix_edges = prefix.persisted_edges_between(from, to);
        assert_eq!(
            prefix_edges[0].retained_assertions()[0].provenance_iri,
            Some(recorder.asserter_iri()),
            "the raw historical assertion first recovers its recorder"
        );
        journal.record_as(
            recorder.clone(),
            CapturedDelta::ReplaySetEdgesByIds {
                from_id: Uuid::from_u128(1).to_string(),
                to_id: Uuid::from_u128(2).to_string(),
                edges: old_edges,
            },
        );

        let full = journal.replay();
        let expected = full.persisted_edges_between(from, to);
        let statements = &expected[0].retained_assertions();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].statement_id, "legacy-exact-handle");
        assert_eq!(statements[0].asserted_at_ms, Some(100));
        assert_eq!(
            statements[0].provenance_iri.as_deref(),
            Some(
                source
                    .map(str::to_owned)
                    .unwrap_or_else(|| recorder.asserter_iri())
                    .as_str()
            ),
            "a minted handle recovers its Author; a known source remains unchanged"
        );

        let mut checkpoint = Graph::from_snapshot(&prefix.to_snapshot());
        journal
            .replay_from_with_baseline(Seq(3), &mut checkpoint, &Graph::new())
            .unwrap();
        assert_eq!(checkpoint.persisted_edges_between(from, to), expected);
        let reopened = Graph::from_snapshot(&full.to_snapshot());
        assert_eq!(reopened.persisted_edges_between(from, to), expected);
    }
}

#[test]
fn legacy_checkpoint_empty_tail_recovers_proven_minting_attribution() {
    use crate::graph::{SemanticStatementSpec, edge_data::UNKNOWN_LEGACY_ASSERTER_IRI};

    let mut template =
        replay_captured_deltas([add(1, "https://a.test/"), add(2, "https://b.test/")]);
    let from = template.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
    let to = template.get_node_key_by_id(Uuid::from_u128(2)).unwrap();
    template.assert_surface_semantic_statement(
        from,
        to,
        SemanticStatementSpec {
            predicate: super::super::super::predicate_iri(SemanticSubKind::Cites).into(),
            recognized_sub_kind: Some(SemanticSubKind::Cites),
            provenance_iri: Some("https://template.test/".into()),
            asserted_at_ms: Some(100),
            ..Default::default()
        },
    );
    let mut edges = template.persisted_edges_between(from, to);
    let mut unknown = edges[0].retained_assertions()[0].clone();
    unknown.statement_id = "baseline-unknown".into();
    unknown.provenance_iri = None;
    let mut known = unknown.clone();
    known.statement_id = "baseline-source".into();
    known.provenance_iri = Some("https://source.test/page".into());
    known.asserted_at_ms = Some(101);
    *edges[0].assertions.as_mut().unwrap() = vec![unknown.clone(), known];
    let capture = |edges| CapturedDelta::ReplaySetEdgesByIds {
        from_id: Uuid::from_u128(1).to_string(),
        to_id: Uuid::from_u128(2).to_string(),
        edges,
    };
    let mut baseline = template;
    super::super::super::capture::replay_captured_deltas_onto(
        &mut baseline,
        [capture(edges.clone())],
    );
    let mut minted = unknown;
    minted.statement_id = "minted-handle".into();
    minted.asserted_at_ms = Some(200);
    edges[0].assertions.as_mut().unwrap().push(minted);
    let minter = Author::engine("first-extractor", "1");
    let mut journal = GraphJournal::new();
    journal.record_as(minter.clone(), capture(edges));
    let mut checkpoint = baseline.clone();
    super::super::super::capture::replay_captured_deltas_onto(
        &mut checkpoint,
        journal.entries().iter().map(|entry| entry.delta.clone()),
    );
    let old = checkpoint.persisted_edges_between(from, to);
    let old_statements = &old[0].retained_assertions();
    assert_eq!(old_statements.len(), 3);
    assert_eq!(
        old_statements[0].provenance_iri.as_deref(),
        Some(UNKNOWN_LEGACY_ASSERTER_IRI)
    );
    assert_eq!(
        old_statements[1].provenance_iri.as_deref(),
        Some("https://source.test/page")
    );
    assert_eq!(
        old_statements[2].provenance_iri.as_deref(),
        Some(UNKNOWN_LEGACY_ASSERTER_IRI)
    );
    checkpoint = Graph::from_snapshot(&checkpoint.to_snapshot());
    let before_revision = checkpoint.revision();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let sink = captured.clone();
    checkpoint.set_recorder(Some(Arc::new(move |delta| {
        sink.lock().unwrap().push(delta.clone());
    })));
    journal
        .replay_from_with_baseline(Seq(1), &mut checkpoint, &baseline)
        .unwrap();
    let repaired = checkpoint.persisted_edges_between(from, to);
    let statements = &repaired[0].retained_assertions();
    assert_eq!(statements[2].provenance_iri, Some(minter.asserter_iri()));
    assert_eq!(statements[2].statement_id, "minted-handle");
    assert_eq!(statements[2].asserted_at_ms, Some(200));
    assert_eq!(statements[0], old_statements[0]);
    assert_eq!(statements[1], old_statements[1]);
    assert!(checkpoint.revision() > before_revision);
    assert!(captured.lock().unwrap().is_empty());
    let full = journal
        .snapshot_at_from(&baseline, Seq(1))
        .unwrap()
        .unwrap();
    assert_eq!(repaired, full.persisted_edges_between(from, to));
}

#[test]
fn exact_capture_attribution_survives_later_authors_and_absent_checkpoint_handles() {
    use crate::graph::{SemanticStatementSpec, edge_data::UNKNOWN_LEGACY_ASSERTER_IRI};

    let mut template =
        replay_captured_deltas([add(1, "https://a.test/"), add(2, "https://b.test/")]);
    let from = template.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
    let to = template.get_node_key_by_id(Uuid::from_u128(2)).unwrap();
    template.assert_surface_semantic_statement(
        from,
        to,
        SemanticStatementSpec {
            predicate: super::super::super::predicate_iri(SemanticSubKind::Cites).into(),
            recognized_sub_kind: Some(SemanticSubKind::Cites),
            provenance_iri: Some("https://template.test/".into()),
            asserted_at_ms: Some(100),
            ..Default::default()
        },
    );
    let mut edges = template.persisted_edges_between(from, to);
    let mut unknown = edges[0].retained_assertions()[0].clone();
    unknown.statement_id = "baseline-unknown".into();
    unknown.provenance_iri = None;
    let mut known = unknown.clone();
    known.statement_id = "baseline-source".into();
    known.provenance_iri = Some("https://source.test/page".into());
    *edges[0].assertions.as_mut().unwrap() = vec![unknown.clone(), known.clone()];
    let capture = |edges| CapturedDelta::ReplaySetEdgesByIds {
        from_id: Uuid::from_u128(1).to_string(),
        to_id: Uuid::from_u128(2).to_string(),
        edges,
    };
    let mut baseline = template;
    super::super::super::capture::replay_captured_deltas_onto(
        &mut baseline,
        [capture(edges.clone())],
    );

    let minter = Author::engine("first-extractor", "1");
    let updater = Author::script("later-update", "1");
    let undoer = Author::rule("undo", "1");
    let mut minted = unknown.clone();
    minted.statement_id = "minted-handle".into();
    minted.asserted_at_ms = Some(200);
    edges[0].assertions.as_mut().unwrap().push(minted);
    let mut journal = GraphJournal::new();
    journal.record_as(minter.clone(), capture(edges.clone()));
    edges[0].retained_assertions_mut()[2].asserted_at_ms = Some(201);
    journal.record_as(updater.clone(), capture(edges.clone()));
    let mut withdrawn = edges.clone();
    *withdrawn[0].assertions.as_mut().unwrap() = vec![known];
    journal.record_as(updater, capture(withdrawn));
    edges[0].retained_assertions_mut()[2].asserted_at_ms = Some(202);
    edges[0].retained_assertions_mut()[1].provenance_iri = None;
    journal.record_as(undoer, capture(edges));

    let full = journal
        .snapshot_at_from(&baseline, Seq(4))
        .unwrap()
        .unwrap();
    let expected = full.persisted_edges_between(from, to);
    let statements = &expected[0].retained_assertions();
    assert_eq!(statements.len(), 3);
    assert_eq!(statements[0].statement_id, "baseline-unknown");
    assert_eq!(statements[0].asserted_at_ms, Some(100));
    assert_eq!(
        statements[0].provenance_iri.as_deref(),
        Some(UNKNOWN_LEGACY_ASSERTER_IRI)
    );
    assert_eq!(statements[1].statement_id, "baseline-source");
    assert_eq!(
        statements[1].provenance_iri.as_deref(),
        Some("https://source.test/page")
    );
    assert_eq!(statements[2].statement_id, "minted-handle");
    assert_eq!(statements[2].asserted_at_ms, Some(202));
    assert_eq!(statements[2].provenance_iri, Some(minter.asserter_iri()));
    for cursor in 0..=4 {
        let prefix = journal
            .snapshot_at_from(&baseline, Seq(cursor))
            .unwrap()
            .unwrap();
        let mut checkpoint = Graph::from_snapshot(&prefix.to_snapshot());
        journal
            .replay_from_with_baseline(Seq(cursor), &mut checkpoint, &baseline)
            .unwrap();
        assert_eq!(checkpoint.persisted_edges_between(from, to), expected);
    }

    let withdrawn = journal
        .snapshot_at_from(&baseline, Seq(3))
        .unwrap()
        .unwrap();
    assert_eq!(
        withdrawn.persisted_edges_between(from, to)[0]
            .retained_assertions()
            .len(),
        1
    );
    let mut without_baseline = Graph::from_snapshot(&withdrawn.to_snapshot());
    journal.replay_from(Seq(3), &mut without_baseline).unwrap();
    let conservative = without_baseline.persisted_edges_between(from, to);
    assert_eq!(
        conservative[0].retained_assertions()[2]
            .provenance_iri
            .as_deref(),
        Some(UNKNOWN_LEGACY_ASSERTER_IRI)
    );
}
