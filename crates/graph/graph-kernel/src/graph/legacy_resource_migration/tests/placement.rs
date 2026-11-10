// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn legacy_resource_migration_surface_placed_claims_need_no_resource_origin_resolution() {
    let baseline = baseline(&["https://a.test/old", "https://b.test/"]);
    for predicate in [
        predicate_iri(SemanticSubKind::UserGrouped),
        "https://vocab.test/custom",
    ] {
        let mut entries = Vec::new();
        let author = Author::person("mint-author");
        entries.push(AttributedDelta {
            author: author.clone(),
            delta: CapturedDelta::ReplayAssertSemanticPredicateByIds {
                from_id: Uuid::from_u128(1).to_string(),
                to_id: Uuid::from_u128(2).to_string(),
                predicate: predicate.into(),
            },
        });
        entries.push(AttributedDelta {
            author: author.clone(),
            delta: CapturedDelta::ReplaySetNodeUrlById {
                node_id: Uuid::from_u128(1).to_string(),
                new_url: "https://a.test/new".into(),
            },
        });
        if predicate.starts_with("https://vocab.test/") {
            let mut declarations = Graph::new();
            declarations
                .declare_predicate(predicate, GraphStratum::Surface)
                .unwrap();
            let id = ResourceNode::for_term(predicate).id();
            entries.push(AttributedDelta {
                author: author.clone(),
                delta: CapturedDelta::ReplaySetResourceRecordById {
                    resource_id: id.to_string(),
                    record: declarations.resource_record(id),
                },
            });
        }
        let mut carried = payload("surface-carried", predicate, Some(&author.asserter_iri()));
        carried.assertions.statements[0].graph_scope = GraphScope::Default;
        entries.push(AttributedDelta {
            author,
            delta: exact(1, 2, &carried),
        });
        assert!(
            probe_legacy_mint_links(&baseline, &entries)
                .unwrap()
                .is_empty()
        );
        let result = migrate_legacy_prefix(&baseline, &entries).unwrap();
        assert_eq!(
            statement_ids(&result.graph.to_snapshot().edges),
            BTreeSet::from(["surface-carried".into()])
        );
        assert!(statement_ids(&result.graph.to_snapshot().resource_edges).is_empty());
    }
}

#[test]
fn legacy_resource_migration_new_carried_conflicted_placement_refuses_with_known_handle_controls() {
    let baseline = baseline(&["https://a.test/old", "https://b.test/"]);
    let predicate = "https://vocab.test/conflicted";
    let author = Author::person("mint");
    let mut declarations = Graph::new();
    let resource_choice = declarations
        .declare_predicate(predicate, GraphStratum::Resource)
        .unwrap();
    let surface_choice = declarations
        .declare_predicate(predicate, GraphStratum::Surface)
        .unwrap();
    let resource_id = ResourceNode::for_term(predicate).id();
    let mut conflict = declarations.resource_record(resource_id).unwrap();
    let facet = conflict
        .facets
        .iter_mut()
        .find(|facet| facet.facet == PREDICATE_DECLARATIONS_FACET)
        .unwrap();
    let mut choices: super::super::super::PredicateDeclarations =
        serde_json::from_str(&facet.value_json).unwrap();
    choices.selected = None;
    facet.value_json = serde_json::to_string(&choices).unwrap();
    let mut journal = GraphJournal::new();
    journal.record_as(
        author.clone(),
        CapturedDelta::ReplayAssertSemanticPredicateByIds {
            from_id: Uuid::from_u128(1).to_string(),
            to_id: Uuid::from_u128(2).to_string(),
            predicate: predicate.into(),
        },
    );
    journal.record(CapturedDelta::ReplaySetNodeUrlById {
        node_id: Uuid::from_u128(1).to_string(),
        new_url: "https://a.test/new".into(),
    });
    journal.record(CapturedDelta::ReplaySetResourceRecordById {
        resource_id: resource_id.to_string(),
        record: Some(conflict.clone()),
    });
    let mut carried = payload("new-carried", predicate, Some(&author.asserter_iri()));
    carried.assertions.statements[0].graph_scope = GraphScope::Default;
    journal.record(exact(1, 2, &carried));
    let before = serde_json::to_vec(journal.entries()).unwrap();
    let error = migrate_legacy_prefix(&baseline, journal.entries())
        .err()
        .unwrap();
    assert!(error.to_string().contains("checkpoint C22"));
    let mut checkpoint = baseline.clone();
    let revision = checkpoint.revision();
    assert!(
        journal
            .migrated_replay_from_with_baseline(Seq(0), &mut checkpoint, &baseline)
            .is_err()
    );
    assert_eq!(checkpoint.to_snapshot().edges, baseline.to_snapshot().edges);
    assert_eq!(checkpoint.revision(), revision);
    assert_eq!(serde_json::to_vec(journal.entries()).unwrap(), before);
    for (selected, stratum) in [
        (resource_choice, GraphStratum::Resource),
        (surface_choice, GraphStratum::Surface),
    ] {
        choices.selected = Some(selected);
        let mut record = conflict.clone();
        record
            .facets
            .iter_mut()
            .find(|facet| facet.facet == PREDICATE_DECLARATIONS_FACET)
            .unwrap()
            .value_json = serde_json::to_string(&choices).unwrap();
        let mut entries = journal.entries().to_vec();
        entries[2].delta = CapturedDelta::ReplaySetResourceRecordById {
            resource_id: resource_id.to_string(),
            record: Some(record),
        };
        let diagnostics = probe_legacy_mint_links(&baseline, &entries).unwrap();
        let resolutions = if stratum == GraphStratum::Resource {
            assert_eq!(diagnostics.len(), 1);
            vec![LegacyResourceOriginResolution {
                statement_id: "new-carried".into(),
                from_resource_id: parse_id(&diagnostics[0].earlier_resource_pair.0).unwrap(),
                to_resource_id: parse_id(&diagnostics[0].earlier_resource_pair.1).unwrap(),
            }]
        } else {
            assert!(diagnostics.is_empty());
            vec![]
        };
        let result =
            migrate_legacy_prefix_with_resolutions(&baseline, &entries, &resolutions).unwrap();
        assert_eq!(
            statement_ids(&result.graph.to_snapshot().resource_edges).contains("new-carried"),
            stratum == GraphStratum::Resource
        );
        assert_eq!(
            statement_ids(&result.graph.to_snapshot().edges).contains("new-carried"),
            stratum == GraphStratum::Surface
        );
    }
    // An already observed handle keeps its owning Resource store across the conflict.
    let mut known = GraphJournal::new();
    known.record(exact(1, 2, &carried));
    known.record(CapturedDelta::ReplaySetResourceRecordById {
        resource_id: resource_id.to_string(),
        record: Some(conflict),
    });
    known.record(exact(1, 2, &carried));
    let result = migrate_legacy_prefix(&baseline, known.entries()).unwrap();
    assert_eq!(
        statement_ids(&result.graph.to_snapshot().resource_edges),
        BTreeSet::from(["new-carried".into()])
    );
    assert!(result.graph.effective_predicate_stratum(predicate).is_err());
}

#[test]
fn legacy_resource_migration_canonical_family_records_keep_exact_ids_without_aggregate_copies() {
    let baseline = baseline(&["https://a.test/old", "https://b.test/"]);
    let kinds: Vec<_> = [(2u32, 7), (4, 7), (5, 9)]
        .into_iter()
        .flat_map(|(family, count)| (0..count).map(move |sub| (family << 24) | sub))
        .map(|tag| super::super::super::RelationKind::from_tag(tag).unwrap())
        .collect();
    assert_eq!(kinds.len(), 23);
    let mut all = EdgePayload::new();
    for kind in &kinds {
        all.assert_relation(aggregate_assertion(*kind));
    }
    let exact_ids: BTreeSet<_> = all
        .assertions()
        .iter()
        .map(|statement| statement.statement_id.clone())
        .collect();
    let mut journal = GraphJournal::new();
    journal.record_as(
        Author::engine("legacy-import", "v7").via("reader"),
        exact(1, 2, &all),
    );
    let result = migrate_legacy_prefix(&baseline, journal.entries()).unwrap();
    let snapshot = result.graph.to_snapshot();
    // These are already canonical assertion records, not anonymous old
    // sidecar memberships. Preserve their exact records without emitting
    // compatibility aggregate copies alongside them.
    let resource_tags = persisted_aggregate_tags(&snapshot.resource_edges);
    let surface_tags = persisted_aggregate_tags(&snapshot.edges);
    assert!(resource_tags.is_empty());
    assert!(surface_tags.is_empty());
    let resource_ids = statement_ids(&snapshot.resource_edges);
    let surface_ids = statement_ids(&snapshot.edges);
    assert_eq!(resource_ids.len(), 16);
    assert_eq!(surface_ids.len(), 7);
    assert_eq!(
        resource_ids
            .union(&surface_ids)
            .cloned()
            .collect::<BTreeSet<_>>(),
        exact_ids
    );
    for edge in &snapshot.resource_edges {
        for statement in payload_from_persisted(edge).assertions() {
            assert_eq!(
                super::super::super::built_in_relation_stratum(statement.relation_kind()),
                GraphStratum::Resource
            );
        }
    }
    for edge in &snapshot.edges {
        for statement in payload_from_persisted(edge).assertions() {
            assert_eq!(
                super::super::super::built_in_relation_stratum(statement.relation_kind()),
                GraphStratum::Surface
            );
        }
    }
    let from = ResourceNode::new("https://a.test/old").id();
    let record = result.graph.resource_record(from).unwrap();
    assert!(
        !record
            .facets
            .iter()
            .any(|facet| facet.facet == LEGACY_AGGREGATE_MIGRATION_FACET)
    );
    let mut effective = baseline.clone();
    replay_captured_deltas_onto(&mut effective, result.baseline_effects);
    for effects in result.entry_effects {
        replay_captured_deltas_onto(&mut effective, effects);
    }
    assert_eq!(
        effective.to_snapshot().resource_edges,
        result.graph.to_snapshot().resource_edges
    );
    assert_eq!(
        effective.to_snapshot().edges,
        result.graph.to_snapshot().edges
    );
}

#[test]
fn legacy_resource_migration_aggregate_original_pair_survives_withdrawal_and_typed_takeover() {
    let baseline = baseline(&[
        "https://a.test/old",
        "https://a.test/old",
        "https://b.test/",
    ]);
    let mut claim = EdgePayload::new();
    claim.assert_relation(EdgeAssertion::Provenance {
        sub_kind: ProvenanceSubKind::ClippedFrom,
    });
    let legacy_kind = super::super::super::RelationKind::Provenance(ProvenanceSubKind::ClippedFrom);
    let mint = Author::person("mint");
    let mut journal = GraphJournal::new();
    journal.record_as(mint, exact_legacy(1, 3, legacy_kind));
    journal.record_as(Author::person("alias"), exact_legacy(2, 3, legacy_kind));
    journal.record(CapturedDelta::ReplaySetNodeUrlById {
        node_id: Uuid::from_u128(1).to_string(),
        new_url: "https://a.test/new".into(),
    });
    journal.record(clear(1, 3));
    let pair = resource_pair("https://a.test/old", "https://b.test/");
    let withdrawn = migrate_legacy_prefix(&baseline, journal.entries()).unwrap();
    assert_eq!(
        persisted_aggregate_tags(
            &withdrawn
                .graph
                .persisted_resource_edges_between(pair.0, pair.1)
        )
        .len(),
        1,
        "withdrawal preserves the other original surface pair"
    );
    // Restore the old family membership through its legacy sidecar form.
    // `exact` carries canonical assertion IDs and belongs to the typed
    // takeover path below; it must not be reinterpreted as an aggregate.
    journal.record_as(Author::person("restorer"), exact_legacy(1, 3, legacy_kind));
    let restored = migrate_legacy_prefix(&baseline, journal.entries()).unwrap();
    assert_eq!(
        persisted_aggregate_tags(
            &restored
                .graph
                .persisted_resource_edges_between(pair.0, pair.1)
        )
        .len(),
        2
    );
    assert!(
        restored
            .graph
            .persisted_resource_edges_between(ResourceNode::new("https://a.test/new").id(), pair.1)
            .is_empty()
    );
    let notes: Vec<LegacyAggregateMigrationNote> = serde_json::from_str(
        &restored
            .graph
            .resource_record(pair.0)
            .unwrap()
            .facets
            .iter()
            .find(|facet| facet.facet == LEGACY_AGGREGATE_MIGRATION_FACET)
            .unwrap()
            .value_json,
    )
    .unwrap();
    assert_eq!(notes.len(), 2);
    assert_eq!(
        notes
            .iter()
            .find(|note| note.from_surface_id == Uuid::from_u128(1).to_string())
            .unwrap()
            .author,
        None,
        "an aggregate snapshot has no exact mint event author"
    );
    let typed = vec![persisted_edge_for_ids(pair.0, pair.1, &claim)];
    journal.record(CapturedDelta::ReplaySetResourceEdgesByIds {
        from_resource_id: pair.0.to_string(),
        to_resource_id: pair.1.to_string(),
        edges: typed.clone(),
    });
    journal.record(CapturedDelta::ReplaySetNodeTitleById {
        node_id: Uuid::from_u128(2).to_string(),
        title: "unrelated".into(),
    });
    let taken = migrate_legacy_prefix(&baseline, journal.entries()).unwrap();
    assert_eq!(
        taken.graph.persisted_resource_edges_between(pair.0, pair.1),
        typed,
        "explicit pair ownership is not repopulated by an unrelated edit"
    );
    journal.record(clear(1, 3));
    journal.record(exact_legacy(1, 3, legacy_kind));
    let new_member = migrate_legacy_prefix(&baseline, journal.entries()).unwrap();
    let new_member_edges = new_member
        .graph
        .persisted_resource_edges_between(pair.0, pair.1);
    assert_eq!(persisted_aggregate_tags(&new_member_edges).len(), 1);
    let records: Vec<_> = new_member_edges
        .iter()
        .flat_map(|edge| edge.retained_assertions())
        .collect();
    assert_eq!(
        records.len(),
        2,
        "a later legacy reappearance adds its own copy without erasing the typed copy"
    );
    assert!(
        records.contains(&&typed[0].retained_assertions()[0]),
        "the explicit typed record remains exact alongside the legacy copy"
    );
    journal.record(clear(1, 3));
    let final_state = migrate_legacy_prefix(&baseline, journal.entries()).unwrap();
    assert_eq!(
        final_state
            .graph
            .persisted_resource_edges_between(pair.0, pair.1),
        typed
    );
}

#[test]
fn legacy_resource_migration_preserves_baseline_handles_and_uncertainty_with_surface_controls() {
    let mut baseline = baseline(&["https://a.test/current", "https://b.test/current"]);
    let mut held = payload("unknown", predicate_iri(SemanticSubKind::Cites), None);
    held.push_persisted_semantic_statement(SemanticStatement {
        statement_id: "group".into(),
        predicate: predicate_iri(SemanticSubKind::UserGrouped).into(),
        recognized_sub_kind: Some(SemanticSubKind::UserGrouped),
        label: Some("surface control".into()),
        graph_scope: GraphScope::User,
        provenance_iri: Some("https://group-author.test/".into()),
        asserted_at_ms: Some(9),
        first_asserted_ms: Some(9),
        withdrawn: None,
        basis: Default::default(),
    });
    held.assert_relation(EdgeAssertion::Imported {
        sub_kind: ImportedSubKind::BookmarkFolder,
    });
    held.assert_relation(EdgeAssertion::Provenance {
        sub_kind: ProvenanceSubKind::ClippedFrom,
    });
    let a = baseline.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
    let b = baseline.get_node_key_by_id(Uuid::from_u128(2)).unwrap();
    baseline.connect_surface(a, b, held.clone());
    let before = baseline.to_snapshot();
    let replay = migrate_legacy_prefix(&baseline, &[]).unwrap();
    assert_eq!(
        baseline.to_snapshot().edges,
        before.edges,
        "input remains exact"
    );
    let pair = resource_pair("https://a.test/current", "https://b.test/current");
    let claims = replay
        .graph
        .persisted_resource_edges_between(pair.0, pair.1);
    let source_payload = payload_from_persisted(&before.edges[0]);
    let expected_resource_ids: BTreeSet<_> = source_payload
        .assertions()
        .iter()
        .filter(|statement| {
            historical_statement_stratum(&baseline, statement) == GraphStratum::Resource
        })
        .map(|statement| statement.statement_id.clone())
        .collect();
    let expected_surface_ids: BTreeSet<_> = source_payload
        .assertions()
        .iter()
        .filter(|statement| {
            historical_statement_stratum(&baseline, statement) == GraphStratum::Surface
        })
        .map(|statement| statement.statement_id.clone())
        .collect();
    assert_eq!(statement_ids(&claims), expected_resource_ids);
    assert!(statement_ids(&claims).contains("unknown"));
    let claim = claims
        .iter()
        .flat_map(|edge| edge.retained_assertions())
        .find(|statement| statement.statement_id == "unknown")
        .expect("unknown legacy statement remains available");
    assert_eq!(claim.asserted_at_ms, Some(100));
    assert_eq!(
        claim.provenance_iri.as_deref(),
        Some(super::super::super::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI)
    );
    let held_surface = replay.graph.persisted_edges_between(a, b);
    assert_eq!(
        statement_ids(&held_surface),
        expected_surface_ids,
        "canonical surface-family records remain on the original Surface pair"
    );
    assert_eq!(held_surface[0].imported, before.edges[0].imported);
    assert!(held_surface[0].provenance.is_none());
    assert!(
        claims
            .iter()
            .any(|edge| edge.provenance == before.edges[0].provenance)
    );
    assert_eq!(
        held_surface[0]
            .retained_assertions()
            .iter()
            .find(|statement| statement.statement_id == "group")
            .expect("the held user-grouped claim remains on Surface")
            .provenance_iri
            .as_deref(),
        Some("https://group-author.test/")
    );
    let record = replay.graph.resource_record(pair.0).unwrap();
    let notes: Vec<LegacyMigrationNote> = serde_json::from_str(
        &record
            .facets
            .iter()
            .find(|facet| facet.facet == LEGACY_RESOURCE_MIGRATION_FACET)
            .unwrap()
            .value_json,
    )
    .unwrap();
    assert_eq!(notes.len(), expected_resource_ids.len());
    let claim_note = notes
        .iter()
        .find(|note| note.statement.statement_id == claim.statement_id)
        .expect("each exact carried record keeps its own migration note");
    assert_eq!(claim_note.from_surface_id, Uuid::from_u128(1).to_string());
    assert_eq!(claim_note.to_surface_id, Uuid::from_u128(2).to_string());
    assert_eq!(
        claim_note.uncertainty,
        "baseline_current_resources_without_mint_event"
    );
    assert_eq!(replay.graph.shown_resource_id(a), Some(pair.0));
    let mut effective = baseline.clone();
    replay_captured_deltas_onto(&mut effective, replay.baseline_effects);
    assert_eq!(
        effective.to_snapshot().resource_edges,
        replay.graph.to_snapshot().resource_edges
    );
    assert_eq!(
        effective.to_snapshot().edges,
        replay.graph.to_snapshot().edges
    );
    assert!(
        !record
            .facets
            .iter()
            .any(|facet| facet.facet == LEGACY_AGGREGATE_MIGRATION_FACET)
    );
    let mut noted_baseline = baseline.clone();
    for record in &replay.graph.to_snapshot().resources {
        assert!(noted_baseline.set_resource_record(
            ResourceNode::for_term(&record.canonical_iri).id(),
            Some(record.clone())
        ));
    }
    let repeated = migrate_legacy_prefix(&noted_baseline, &[]).unwrap();
    assert_eq!(
        repeated.graph.resource_record(pair.0),
        replay.graph.resource_record(pair.0),
        "an existing identical uncertainty note is a valid no-op"
    );
}
