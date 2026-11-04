// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn legacy_resource_migration_exact_facet_effects_freeze_birth_clock_and_unknown_values() {
    let baseline = baseline(&["https://a.test/", "https://b.test/"]);
    let born = Uuid::from_u128(3);
    let mut journal = GraphJournal::new();
    let author = Author::person("legacy-author");
    journal.record_as(
        author.clone(),
        CapturedDelta::ReplayAddNodeWithIdIfMissing {
            id: born.to_string(),
            url: "https://c.test/".into(),
            position: [0.0, 0.0],
        },
    );
    journal.record_as(
        author.clone(),
        CapturedDelta::ReplayAssertRelationByIds {
            from_id: Uuid::from_u128(1).to_string(),
            to_id: born.to_string(),
            assertion: EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::UserGrouped,
                label: None,
            },
        },
    );
    journal.record_as(
        author.clone(),
        CapturedDelta::ReplaySetNodeFacetById {
            node_id: born.to_string(),
            facet: "foreign.opaque".into(),
            value_json: r#"{"raw":"KeepCase#Exact","nested":[null,17,{"x":true}]}"#.into(),
        },
    );
    journal.record_as(
        author.clone(),
        CapturedDelta::ReplaySetNodeFacetById {
            node_id: born.to_string(),
            facet: "foreign.removed".into(),
            value_json: "[1,2]".into(),
        },
    );
    journal.record_as(
        author,
        CapturedDelta::ReplayRemoveNodeFacetById {
            node_id: born.to_string(),
            facet: "foreign.removed".into(),
        },
    );
    let input = serde_json::to_vec(journal.entries()).unwrap();
    let translated = migrate_legacy_prefix(&baseline, journal.entries()).unwrap();
    assert!(translated.entry_effects[0].iter().any(|delta| matches!(delta,
        CapturedDelta::ReplaySetNodeFacetById { node_id, facet, .. }
            if node_id == &born.to_string() && facet == super::super::super::node_facets::VISIT_HISTORY)));
    assert!(
        translated.entry_effects[4]
            .iter()
            .any(|delta| matches!(delta,
        CapturedDelta::ReplayRemoveNodeFacetById { node_id, facet }
            if node_id == &born.to_string() && facet == "foreign.removed"))
    );
    let snapshot_value = |graph: &Graph| {
        let mut value = serde_json::to_value(graph.to_snapshot()).unwrap();
        value["timestamp_secs"] = serde_json::json!(0);
        value
    };
    let mut recreated = baseline.clone();
    replay_captured_deltas_onto(&mut recreated, translated.baseline_effects.clone());
    for effects in &translated.entry_effects {
        replay_captured_deltas_onto(&mut recreated, effects.iter().cloned());
    }
    assert_eq!(
        snapshot_value(&recreated),
        snapshot_value(&translated.graph)
    );
    assert_eq!(recreated.facets(), translated.graph.facets());
    let mut reordered_baseline = translated.baseline_effects.clone();
    let records: Vec<_> = reordered_baseline
        .iter()
        .enumerate()
        .filter_map(|(index, delta)| {
            matches!(
                delta,
                CapturedDelta::ReplaySetResourceRecordById {
                    record: Some(_),
                    ..
                }
            )
            .then_some(index)
        })
        .collect();
    assert_eq!(records.len(), 2);
    for (destination, source) in records.iter().zip(records.iter().rev()) {
        reordered_baseline[*destination] = translated.baseline_effects[*source].clone();
    }
    let mut reordered = baseline.clone();
    replay_captured_deltas_onto(&mut reordered, reordered_baseline);
    for effects in &translated.entry_effects {
        replay_captured_deltas_onto(&mut reordered, effects.iter().cloned());
    }
    assert_eq!(
        reordered.to_snapshot().edges,
        translated.graph.to_snapshot().edges,
        "changing record creation order does not change the carried statement handle"
    );
    assert_ne!(
        reordered.to_snapshot().resources,
        translated.graph.to_snapshot().resources,
        "exact frozen reconstruction also preserves resource record order"
    );
    let mut fresh_clock = baseline.clone();
    replay_captured_deltas_onto(&mut fresh_clock, translated.baseline_effects.clone());
    for effects in &translated.entry_effects {
        replay_captured_deltas_onto(&mut fresh_clock, effects.iter().filter(|delta| !matches!(delta,
            CapturedDelta::ReplaySetNodeFacetById { node_id, facet, .. }
                if node_id == &born.to_string() && facet == super::super::super::node_facets::VISIT_HISTORY)).cloned());
    }
    let key = translated.graph.get_node_key_by_id(born).unwrap();
    let frozen_ms = translated
        .graph
        .node_last_visited(key)
        .unwrap()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    // A controlled later replay clock makes the negative independent of clock resolution.
    replay_captured_deltas_onto(
        &mut fresh_clock,
        [CapturedDelta::ReplayTouchNodeLastVisitedById {
            node_id: born.to_string(),
            timestamp_ms: frozen_ms.saturating_add(1),
        }],
    );
    assert_eq!(
        snapshot_value(&fresh_clock),
        snapshot_value(&translated.graph),
        "the snapshot alone does not include the authoritative visit facet"
    );
    assert_ne!(
        fresh_clock.facets(),
        translated.graph.facets(),
        "an unfrozen replay clock is observably different"
    );
    assert_eq!(serde_json::to_vec(journal.entries()).unwrap(), input);
    assert_eq!(
        statement_ids(&recreated.to_snapshot().edges),
        statement_ids(&translated.graph.to_snapshot().edges)
    );
}

#[test]
fn legacy_resource_migration_lifetime_and_pair_membership_survive_navigation_and_restore() {
    let baseline = baseline(&[
        "https://a.test/old",
        "https://a.test/old",
        "https://b.test/",
    ]);
    let alice = Author::person("alice");
    let bob = Author::person("bob");
    let mut journal = GraphJournal::new();
    let predicate = predicate_iri(SemanticSubKind::Cites);
    journal.record_as(alice.clone(), exact(1, 3, &payload("one", predicate, None)));
    journal.record_as(bob.clone(), exact(2, 3, &payload("two", predicate, None)));
    journal.record(CapturedDelta::ReplaySetNodeUrlById {
        node_id: Uuid::from_u128(1).to_string(),
        new_url: "https://a.test/new".into(),
    });
    journal.record(clear(1, 3));
    let pair = resource_pair("https://a.test/old", "https://b.test/");
    let withdrawn = journal
        .migrated_prefix_at_from(&baseline, journal.live_cursor())
        .unwrap()
        .unwrap();
    assert_eq!(
        statement_ids(
            &withdrawn
                .graph
                .persisted_resource_edges_between(pair.0, pair.1)
        ),
        BTreeSet::from(["two".into()]),
        "other original surface pair survives withdrawal"
    );
    let checkpoint_cursor = journal.live_cursor();
    let mut checkpoint = journal
        .snapshot_at_from(&baseline, checkpoint_cursor)
        .unwrap()
        .unwrap();
    journal.record_as(
        Author::person("restorer"),
        exact(1, 3, &payload("one", predicate, None)),
    );
    let replay = journal
        .migrated_prefix_at_from(&baseline, journal.live_cursor())
        .unwrap()
        .unwrap();
    let claims = replay
        .graph
        .persisted_resource_edges_between(pair.0, pair.1);
    assert_eq!(
        statement_ids(&claims),
        BTreeSet::from(["one".into(), "two".into()])
    );
    for (id, author) in [("one", alice), ("two", bob)] {
        let statement = claims
            .iter()
            .flat_map(|edge| edge.retained_assertions())
            .find(|statement| statement.statement_id == id)
            .unwrap();
        assert_eq!(statement.asserted_at_ms, Some(100));
        assert_eq!(statement.provenance_iri, Some(author.asserter_iri()));
    }
    let new = ResourceNode::new("https://a.test/new").id();
    assert!(
        replay
            .graph
            .persisted_resource_edges_between(new, pair.1)
            .is_empty(),
        "restored handle does not follow navigation"
    );
    assert_eq!(
        replay
            .graph
            .shown_resource_id(replay.graph.get_node_key_by_id(Uuid::from_u128(1)).unwrap()),
        Some(new)
    );
    assert!(
        replay
            .graph
            .to_snapshot()
            .edges
            .iter()
            .all(|edge| edge.semantic.is_none() && edge.assertions.is_some())
    );
    journal
        .migrated_replay_from_with_baseline(checkpoint_cursor, &mut checkpoint, &baseline)
        .unwrap();
    assert_eq!(
        checkpoint.to_snapshot().resource_edges,
        replay.graph.to_snapshot().resource_edges
    );
    let mut effective = baseline.clone();
    replay_captured_deltas_onto(
        &mut effective,
        replay
            .baseline_effects
            .into_iter()
            .chain(replay.entry_effects.into_iter().flatten()),
    );
    assert_eq!(
        effective.to_snapshot().resource_edges,
        replay.graph.to_snapshot().resource_edges
    );
    assert!(
        journal
            .migrated_snapshot_at_from(&baseline, Seq(journal.live_cursor().0 + 1))
            .unwrap()
            .is_none()
    );
}

#[test]
fn legacy_resource_migration_effective_captures_undo_claim_after_later_navigation() {
    let baseline = baseline(&[
        "https://a.test/old",
        "https://a.test/old",
        "https://b.test/",
    ]);
    let predicate = predicate_iri(SemanticSubKind::Cites);
    let mut journal = GraphJournal::new();
    journal.record(exact(
        2,
        3,
        &payload("other", predicate, Some("https://other.test/")),
    ));
    let before = journal
        .migrated_snapshot_at_from(&baseline, journal.live_cursor())
        .unwrap()
        .unwrap();
    journal.record(exact(
        1,
        3,
        &payload("mine", predicate, Some("https://author.test/")),
    ));
    let after = journal
        .migrated_prefix_at_from(&baseline, journal.live_cursor())
        .unwrap()
        .unwrap();
    let change = after.entry_effects.last().unwrap().clone();
    assert!(
        change
            .iter()
            .any(|delta| matches!(delta, CapturedDelta::ReplaySetResourceEdgesByIds { .. }))
    );
    journal.record(CapturedDelta::ReplaySetNodeUrlById {
        node_id: Uuid::from_u128(1).to_string(),
        new_url: "https://a.test/new".into(),
    });
    let mut live = journal
        .migrated_snapshot_at_from(&baseline, journal.live_cursor())
        .unwrap()
        .unwrap();
    let undo = super::super::super::revert_change(&change, &before, &after.graph, &live);
    assert!(undo.kept.is_empty());
    replay_captured_deltas_onto(&mut live, undo.edits);
    let pair = resource_pair("https://a.test/old", "https://b.test/");
    assert_eq!(
        statement_ids(&live.persisted_resource_edges_between(pair.0, pair.1)),
        BTreeSet::from(["other".into()])
    );
    let key = live.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
    assert_eq!(
        live.get_node(key).unwrap().primary_address().as_url_str(),
        "https://a.test/new"
    );
    assert_eq!(
        live.shown_resource_id(key),
        Some(ResourceNode::new("https://a.test/new").id())
    );
}

#[test]
fn legacy_resource_migration_raw_assertion_retains_author_and_historical_endpoints() {
    let baseline = baseline(&["https://a.test/old", "https://b.test/"]);
    let mut journal = GraphJournal::new();
    let author = Author::person("raw-author");
    journal.record_as(
        author.clone(),
        CapturedDelta::ReplayAssertRelationByIds {
            from_id: Uuid::from_u128(1).to_string(),
            to_id: Uuid::from_u128(2).to_string(),
            assertion: EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Cites,
                label: None,
            },
        },
    );
    journal.record(CapturedDelta::ReplaySetNodeUrlById {
        node_id: Uuid::from_u128(1).to_string(),
        new_url: "https://a.test/new".into(),
    });
    for _ in 0..2 {
        let graph = journal
            .migrated_snapshot_at_from(&baseline, journal.live_cursor())
            .unwrap()
            .unwrap();
        let pair = resource_pair("https://a.test/old", "https://b.test/");
        let claims = graph.persisted_resource_edges_between(pair.0, pair.1);
        assert_eq!(claims.len(), 1);
        assert_eq!(
            claims[0].retained_assertions()[0].provenance_iri,
            Some(author.asserter_iri())
        );
        assert!(
            graph
                .persisted_resource_edges_between(
                    ResourceNode::new("https://a.test/new").id(),
                    pair.1
                )
                .is_empty()
        );
        assert!(
            graph
                .to_snapshot()
                .edges
                .iter()
                .all(|edge| edge.semantic.is_none() && edge.assertions.is_some())
        );
    }
    // Raw records did not carry stable handles; repeated replay deliberately
    // makes no identity-equality promise absent from the retained history.
    assert!(
        probe_legacy_mint_links(&baseline, journal.entries())
            .unwrap()
            .is_empty()
    );
}
