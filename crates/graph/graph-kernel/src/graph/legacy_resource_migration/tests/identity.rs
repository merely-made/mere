// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn legacy_resource_migration_typed_pair_owns_only_its_pair_and_is_not_repopulated() {
    let baseline = baseline(&[
        "https://a.test/",
        "https://b.test/",
        "https://c.test/",
        "https://d.test/",
    ]);
    let mut journal = GraphJournal::new();
    let predicate = predicate_iri(SemanticSubKind::Cites);
    journal.record(exact(
        1,
        2,
        &payload("old-a", predicate, Some("https://author.test/")),
    ));
    journal.record(exact(
        3,
        4,
        &payload("old-c", predicate, Some("https://author.test/")),
    ));
    let first = resource_pair("https://a.test/", "https://b.test/");
    let second = resource_pair("https://c.test/", "https://d.test/");
    let typed = persisted_edge_for_ids(
        first.0,
        first.1,
        &payload("typed", predicate, Some("https://typed-author.test/")),
    );
    journal.record(CapturedDelta::ReplaySetResourceEdgesByIds {
        from_resource_id: first.0.to_string(),
        to_resource_id: first.1.to_string(),
        edges: vec![typed.clone()],
    });
    let mut update = payload("old-c", predicate, Some("https://author.test/"));
    update.assertions.statements[0].label = Some("later unrelated edit".into());
    journal.record(exact(3, 4, &update));
    let replay = journal
        .migrated_prefix_at_from(&baseline, journal.live_cursor())
        .unwrap()
        .unwrap();
    assert_eq!(
        replay
            .graph
            .persisted_resource_edges_between(first.0, first.1),
        vec![typed]
    );
    assert_eq!(
        statement_ids(
            &replay
                .graph
                .persisted_resource_edges_between(second.0, second.1)
        ),
        BTreeSet::from(["old-c".into()])
    );
    assert!(
        replay
            .graph
            .to_snapshot()
            .edges
            .iter()
            .all(|edge| edge.semantic.is_none() && edge.assertions.is_some())
    );
    journal.record(CapturedDelta::ReplaySetResourceEdgesByIds {
        from_resource_id: first.0.to_string(),
        to_resource_id: first.1.to_string(),
        edges: vec![],
    });
    journal.record(exact(3, 4, &update));
    let cleared = journal
        .migrated_snapshot_at_from(&baseline, journal.live_cursor())
        .unwrap()
        .unwrap();
    assert!(
        cleared
            .persisted_resource_edges_between(first.0, first.1)
            .is_empty(),
        "unrelated old membership cannot undo explicit typed clear"
    );
    assert_eq!(
        statement_ids(&cleared.persisted_resource_edges_between(second.0, second.1)),
        BTreeSet::from(["old-c".into()])
    );
}

#[test]
fn legacy_resource_migration_authored_overrides_apply_at_mint_and_conflicts_keep_typed_history() {
    let baseline = baseline(&["https://a.test/", "https://b.test/"]);
    let predicate = "https://vocab.test/p#custom";
    let mut declarations = Graph::new();
    declarations
        .declare_predicate(predicate, GraphStratum::Surface)
        .unwrap();
    let id = ResourceNode::for_term(predicate).id();
    let surface = declarations.resource_record(id).unwrap();
    declarations
        .declare_predicate(predicate, GraphStratum::Resource)
        .unwrap();
    let resource = declarations.resource_record(id).unwrap();
    let mut journal = GraphJournal::new();
    journal.record(CapturedDelta::ReplaySetResourceRecordById {
        resource_id: id.to_string(),
        record: Some(surface),
    });
    journal.record(exact(
        1,
        2,
        &payload("surface-mint", predicate, Some("https://author.test/")),
    ));
    journal.record(CapturedDelta::ReplaySetResourceRecordById {
        resource_id: id.to_string(),
        record: Some(resource),
    });
    let mut both = payload("surface-mint", predicate, Some("https://author.test/"));
    both.push_persisted_semantic_statement(SemanticStatement {
        statement_id: "resource-mint".into(),
        predicate: predicate.into(),
        recognized_sub_kind: None,
        label: None,
        graph_scope: GraphScope::User,
        provenance_iri: Some("https://other.test/".into()),
        asserted_at_ms: Some(200),
        first_asserted_ms: Some(200),
        withdrawn: None,
        basis: Default::default(),
    });
    journal.record(exact(1, 2, &both));
    let migrated = journal
        .migrated_snapshot_at_from(&baseline, journal.live_cursor())
        .unwrap()
        .unwrap();
    assert_eq!(
        statement_ids(&migrated.to_snapshot().edges),
        BTreeSet::from(["surface-mint".into()])
    );
    assert_eq!(
        statement_ids(&migrated.to_snapshot().resource_edges),
        BTreeSet::from(["resource-mint".into()])
    );
    let mut conflicted = migrated.clone();
    let mut record = conflicted.resource_record(id).unwrap();
    let facet = record
        .facets
        .iter_mut()
        .find(|facet| facet.facet == PREDICATE_DECLARATIONS_FACET)
        .unwrap();
    let mut declarations: super::super::super::PredicateDeclarations =
        serde_json::from_str(&facet.value_json).unwrap();
    declarations.selected = None;
    facet.value_json = serde_json::to_string(&declarations).unwrap();
    assert!(conflicted.set_resource_record(id, Some(record)));
    assert!(conflicted.effective_predicate_stratum(predicate).is_err());
    // Only caller-qualified raw legacy bytes are adapted; typed claims keep their store.
    let mut typed_only = baseline.clone();
    for record in &conflicted.to_snapshot().resources {
        let resource = ResourceNode::for_term(&record.canonical_iri).id();
        typed_only.set_resource_record(resource, Some(record.clone()));
    }
    let pair = resource_pair("https://a.test/", "https://b.test/");
    let typed = conflicted.persisted_resource_edges_between(pair.0, pair.1);
    assert!(typed_only.set_resource_edges_between(pair.0, pair.1, &typed));
    let replay = migrate_legacy_prefix(&typed_only, &[]).unwrap();
    assert_eq!(
        replay
            .graph
            .persisted_resource_edges_between(pair.0, pair.1),
        typed
    );
    assert!(replay.graph.effective_predicate_stratum(predicate).is_err());
}

#[test]
fn legacy_resource_migration_handle_collision_is_atomic_with_distinct_handle_control() {
    let mut baseline = baseline(&["https://a.test/", "https://b.test/"]);
    let a = baseline.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
    let b = baseline.get_node_key_by_id(Uuid::from_u128(2)).unwrap();
    let predicate = predicate_iri(SemanticSubKind::Cites);
    baseline.inner.connect(
        a,
        b,
        payload("collision", predicate, Some("https://legacy.test/")),
    );
    let resources = [
        ResourceNode::for_term("https://other.test/a"),
        ResourceNode::for_term("https://other.test/b"),
    ];
    for resource in &resources {
        assert!(baseline.set_resource_record(
            resource.id(),
            Some(PersistedResourceRecord {
                canonical_iri: resource.canonical_iri().into(),
                facets: vec![]
            })
        ));
    }
    let pair = (resources[0].id(), resources[1].id());
    // Deliberately malformed in-memory input bypasses the checked setter.
    let from = baseline.resources.key_of(&pair.0).unwrap();
    let to = baseline.resources.key_of(&pair.1).unwrap();
    baseline.resources.connect(
        from,
        to,
        payload("collision", predicate, Some("https://typed.test/")),
    );
    let before = baseline.to_snapshot();
    assert!(migrate_legacy_prefix(&baseline, &[]).is_err());
    assert_eq!(baseline.to_snapshot().edges, before.edges);
    assert_eq!(baseline.to_snapshot().resource_edges, before.resource_edges);
    let keys: Vec<_> = baseline.resources.inner().edge_indices().collect();
    for key in keys {
        baseline.resources.disconnect(key);
    }
    assert!(baseline.set_resource_edges_between(
        pair.0,
        pair.1,
        &[persisted_edge_for_ids(
            pair.0,
            pair.1,
            &payload("distinct", predicate, Some("https://typed.test/"))
        )]
    ));
    let valid = migrate_legacy_prefix(&baseline, &[]).unwrap();
    assert_eq!(
        statement_ids(&valid.graph.to_snapshot().resource_edges),
        BTreeSet::from(["collision".into(), "distinct".into()])
    );
}

#[test]
fn legacy_resource_migration_claim_identity_survives_absence_and_allows_metadata_updates() {
    let baseline = baseline(&["https://a.test/", "https://b.test/"]);
    let predicate = predicate_iri(SemanticSubKind::Cites);
    let held = payload("held", predicate, Some("https://author.test/"));
    let mut journal = GraphJournal::new();
    journal.record(exact(1, 2, &held));
    journal.record(clear(1, 2));
    let mut metadata = held.clone();
    let statement = &mut metadata.assertions.statements[0];
    statement.label = Some("new label".into());
    statement.basis.note = Some("new label".into());
    statement.asserted_at_ms = Some(300);
    journal.record(exact(1, 2, &metadata));
    let valid = journal
        .migrated_snapshot_at_from(&baseline, journal.live_cursor())
        .unwrap()
        .unwrap();
    let valid_snapshot = valid.to_snapshot();
    let statement = &valid_snapshot.resource_edges[0].retained_assertions()[0];
    assert_eq!(statement.statement_id, "held");
    assert_eq!(statement.label.as_deref(), Some("new label"));
    assert_eq!(statement.asserted_at_ms, Some(300));
    for change in 0..4 {
        let mut conflict = held.clone();
        let statement = &mut conflict.assertions.statements[0];
        match change {
            0 => statement.predicate = "https://vocab.test/changed".into(),
            1 => statement.recognized_sub_kind = Some(SemanticSubKind::Quotes),
            2 => statement.graph_scope = GraphScope::Default,
            _ => statement.provenance_iri = Some("https://changed-author.test/".into()),
        }
        let mut divergent = journal.clone();
        divergent.record(clear(1, 2));
        divergent.record(exact(1, 2, &conflict));
        let source_before = serde_json::to_vec(divergent.entries()).unwrap();
        let mut checkpoint = valid.clone();
        let before = checkpoint.to_snapshot();
        assert!(
            divergent
                .migrated_replay_from_with_baseline(Seq(0), &mut checkpoint, &baseline)
                .is_err()
        );
        assert_eq!(
            checkpoint.to_snapshot().resource_edges,
            before.resource_edges
        );
        assert_eq!(
            serde_json::to_vec(divergent.entries()).unwrap(),
            source_before
        );
        conflict.assertions.statements[0].statement_id = format!("distinct-{change}");
        let mut distinct = journal.clone();
        distinct.record(exact(1, 2, &conflict));
        assert!(
            distinct
                .migrated_snapshot_at_from(&baseline, distinct.live_cursor())
                .unwrap()
                .is_some(),
            "distinct-handle control"
        );
    }
}

#[test]
fn legacy_resource_migration_dedupes_identical_parallel_handle_copies_and_says_so() {
    let mut baseline = baseline(&["https://a.test/", "https://b.test/"]);
    let a = baseline.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
    let b = baseline.get_node_key_by_id(Uuid::from_u128(2)).unwrap();
    let claim = payload(
        "copied",
        predicate_iri(SemanticSubKind::Cites),
        Some("https://author.test/"),
    );
    baseline.inner.connect(a, b, claim.clone());
    baseline.inner.connect(a, b, claim);
    let migrated = migrate_legacy_prefix(&baseline, &[]).unwrap();
    let pair = resource_pair("https://a.test/", "https://b.test/");
    let rows = migrated
        .graph
        .persisted_resource_edges_between(pair.0, pair.1);
    assert_eq!(rows.len(), 1, "the copy is dropped, the first row kept");
    assert_eq!(statement_ids(&rows), BTreeSet::from(["copied".into()]));
    assert!(migrated.graph.persisted_edges_between(a, b).is_empty());
    let record = migrated.graph.resource_record(pair.0).unwrap();
    let notes: Vec<LegacyMigrationNote> = serde_json::from_str(
        &record
            .facets
            .iter()
            .find(|facet| facet.facet == LEGACY_RESOURCE_MIGRATION_FACET)
            .unwrap()
            .value_json,
    )
    .unwrap();
    assert!(
        notes
            .iter()
            .any(|note| note.uncertainty == copies::DEDUPED_COPY
                && note.statement.statement_id == "copied")
    );
    Graph::try_from_snapshot(&migrated.graph.to_snapshot())
        .expect("the migrated graph is legal state");
    let mut replayed = baseline.clone();
    replay_captured_deltas_onto(&mut replayed, migrated.baseline_effects.clone());
    assert_eq!(
        replayed.to_snapshot().resources,
        migrated.graph.to_snapshot().resources
    );
    assert_eq!(
        replayed.to_snapshot().resource_edges,
        migrated.graph.to_snapshot().resource_edges
    );
    let mut invalid = baseline.clone();
    let key = invalid.find_edge_key(a, b).unwrap();
    invalid.get_edge_mut(key).unwrap().assertions.statements[0].label =
        Some("conflicting copy".into());
    invalid.get_edge_mut(key).unwrap().assertions.statements[0]
        .basis
        .note = Some("conflicting copy".into());
    assert!(migrate_legacy_prefix(&invalid, &[]).is_err());
    assert_eq!(baseline.persisted_edges_between(a, b).len(), 2);
}

#[test]
fn legacy_resource_migration_new_handles_honor_selected_baseline_declarations() {
    let mut baseline = baseline(&["https://a.test/", "https://b.test/"]);
    let surface_predicate = "https://vocab.test/surface";
    baseline
        .declare_predicate(surface_predicate, GraphStratum::Surface)
        .unwrap();
    let mut journal = GraphJournal::new();
    journal.record(exact(
        1,
        2,
        &payload(
            "new-surface",
            surface_predicate,
            Some("https://author.test/"),
        ),
    ));
    let mut combined = payload(
        "new-surface",
        surface_predicate,
        Some("https://author.test/"),
    );
    combined.push_persisted_semantic_statement(SemanticStatement {
        statement_id: "resource-control".into(),
        predicate: "https://vocab.test/default".into(),
        recognized_sub_kind: None,
        label: None,
        graph_scope: GraphScope::User,
        provenance_iri: Some("https://author.test/".into()),
        asserted_at_ms: Some(200),
        first_asserted_ms: Some(200),
        withdrawn: None,
        basis: Default::default(),
    });
    journal.record(exact(1, 2, &combined));
    let graph = journal
        .migrated_snapshot_at_from(&baseline, journal.live_cursor())
        .unwrap()
        .unwrap();
    assert_eq!(
        statement_ids(&graph.to_snapshot().edges),
        BTreeSet::from(["new-surface".into()])
    );
    assert_eq!(
        statement_ids(&graph.to_snapshot().resource_edges),
        BTreeSet::from(["resource-control".into()])
    );
    assert_eq!(
        graph.effective_predicate_stratum(surface_predicate),
        Ok(GraphStratum::Surface)
    );
}
