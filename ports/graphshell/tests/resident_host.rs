// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
#![cfg(all(feature = "resident-storage", not(target_arch = "wasm32")))]
use graphshell::{
    access::AccessContext,
    handlers::HandlerRegistry,
    mere_host::{MereHost, SelectedPersonaRef},
    resident_host::{ResidentHost, ResidentHostError, ResidentPersistence},
};
use muniment::{DirectoryBackend, MemoryBackend};
use pandect::{
    Author, MereSessions,
    addressable_graph::GraphAddress,
    live_graph::{LiveCheck, LiveCommand, ObservedCommand},
};
use std::time::SystemTime;
use uuid::Uuid;
fn persona() -> SelectedPersonaRef {
    SelectedPersonaRef {
        persona: "native-person".into(),
        profile: "native".into(),
    }
}
fn access() -> AccessContext {
    AccessContext {
        persona: "native-person".into(),
        device: "native".into(),
        at_ms: 0,
    }
}
fn create(id: Uuid) -> Vec<ObservedCommand> {
    vec![ObservedCommand {
        command: LiveCommand::CreateAddress {
            id,
            address: "https://native-resident.test/".into(),
            title: "Resident".into(),
        },
        checks: vec![LiveCheck::Absent(GraphAddress::Surface(id))],
    }]
}
#[test]
fn directory_owner_reopen_retains_navigation_facets_author_and_undo() {
    pollster::block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let backend = DirectoryBackend::open(dir.path()).unwrap();
        let mut session = MereSessions::new(backend.clone())
            .begin_recorded(Author::person("native-person").via("turnstone"), None);
        // Seed actual navigation history, whose timestamps and transitions
        // must survive later combined checkpoints and owner recovery.
        session
            .edit_now(Author::person("native-person").via("turnstone"), |graph| {
                use mere::kernel::graph::apply::{GraphDelta, add_node, apply_graph_delta};
                let id = Uuid::from_u128(4999);
                add_node(
                    graph,
                    Some(id),
                    "https://navigation.test/first".into(),
                    Default::default(),
                );
                let key = graph.get_node_key_by_id(id).unwrap();
                for url in [
                    "https://navigation.test/first",
                    "https://navigation.test/second",
                ] {
                    apply_graph_delta(
                        graph,
                        GraphDelta::NavigateNode {
                            key,
                            url: url.into(),
                        },
                    );
                }
            })
            .unwrap();
        let navigation_key = session
            .graph()
            .get_node_key_by_id(Uuid::from_u128(4999))
            .unwrap();
        assert_eq!(
            session
                .graph()
                .node_history_projection(navigation_key)
                .entries,
            [
                "https://navigation.test/first",
                "https://navigation.test/second"
            ]
        );
        assert_eq!(
            session.graph().to_snapshot_at(0).navigation.visit_count(),
            2
        );
        session.flush(SystemTime::UNIX_EPOCH).await.unwrap();
        let sid = session.id();
        drop(session);
        drop(backend);
        let mut owner = ResidentHost::open_directory(
            dir.path(),
            sid,
            persona(),
            HandlerRegistry::default(),
            access(),
        )
        .await
        .unwrap();
        assert!(DirectoryBackend::open(dir.path()).is_err());
        let id = Uuid::from_u128(5000);
        owner
            .edit("turnstone", create(id), SystemTime::UNIX_EPOCH)
            .unwrap();
        assert!(matches!(
            owner.persist(SystemTime::UNIX_EPOCH).await.unwrap(),
            ResidentPersistence::Durable { .. }
        ));
        let digest = owner
            .reader()
            .unwrap()
            .record_digest(GraphAddress::Surface(id))
            .unwrap()
            .to_owned();
        owner
            .edit(
                "turnstone",
                vec![ObservedCommand {
                    command: LiveCommand::SetFacet {
                        surface: id,
                        facet: "test.unknown".into(),
                        value: serde_json::json!({"kept": [1,2,3]}),
                    },
                    checks: vec![LiveCheck::RecordUnchanged {
                        address: GraphAddress::Surface(id),
                        digest,
                    }],
                }],
                SystemTime::UNIX_EPOCH,
            )
            .unwrap();
        owner.persist(SystemTime::UNIX_EPOCH).await.unwrap();
        let snapshot = serde_json::to_value(owner.session().graph().to_snapshot_at(0)).unwrap();
        let facets = serde_json::to_value(owner.session().graph().facets()).unwrap();
        let cursor = owner.session().journal().live_cursor();
        let changes = owner.session().changes().to_vec();
        let mut owner = owner
            .reopen(persona(), HandlerRegistry::default(), access())
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(owner.session().graph().to_snapshot_at(0)).unwrap(),
            snapshot
        );
        assert_eq!(
            serde_json::to_value(owner.session().graph().facets()).unwrap(),
            facets
        );
        assert_eq!(owner.session().journal().live_cursor(), cursor);
        assert_eq!(
            serde_json::to_value(owner.session().changes()).unwrap(),
            serde_json::to_value(&changes).unwrap()
        );
        assert!(
            owner
                .session()
                .changes()
                .iter()
                .all(|c| c.author == Author::person("native-person").via("turnstone"))
        );
        owner.undo("turnstone", SystemTime::UNIX_EPOCH).unwrap();
        owner.persist(SystemTime::UNIX_EPOCH).await.unwrap();
        owner.redo("turnstone", SystemTime::UNIX_EPOCH).unwrap();
        owner.persist(SystemTime::UNIX_EPOCH).await.unwrap();
        assert_eq!(
            serde_json::to_value(owner.session().graph().facets()).unwrap(),
            facets
        );
    });
}
#[test]
fn generic_memory_host_cannot_claim_production_ownership() {
    let mut host = MereHost::empty(
        MemoryBackend::new(),
        persona(),
        HandlerRegistry::default(),
        access(),
    );
    assert!(matches!(
        host.prepare_resident_io(SystemTime::UNIX_EPOCH),
        Err(ResidentHostError::OwnershipUnproven)
    ));
    assert!(matches!(
        ResidentHost::try_from_host(host),
        Err(ResidentHostError::OwnershipUnproven)
    ));
}

#[test]
fn redb_factory_retains_exclusive_handle_and_reopens_committed_truth() {
    pollster::block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("resident.redb");
        let backend = muniment::RedbBackend::open(&path).unwrap();
        let mut session = MereSessions::new(backend.clone()).begin_recorded(Author::user(), None);
        session.flush(SystemTime::UNIX_EPOCH).await.unwrap();
        let sid = session.id();
        drop(session);
        drop(backend);
        let mut owner =
            ResidentHost::open_redb(&path, sid, persona(), HandlerRegistry::default(), access())
                .await
                .unwrap();
        assert!(muniment::RedbBackend::open(&path).is_err());
        owner
            .edit(
                "turnstone",
                create(Uuid::from_u128(5100)),
                SystemTime::UNIX_EPOCH,
            )
            .unwrap();
        assert!(matches!(
            owner.persist(SystemTime::UNIX_EPOCH).await.unwrap(),
            ResidentPersistence::Durable { .. }
        ));
        let cursor = owner.session().journal().live_cursor();
        drop(owner);
        let backend = muniment::RedbBackend::open(&path).unwrap();
        let session = MereSessions::new(backend).open(sid).await.unwrap();
        assert_eq!(session.journal().live_cursor(), cursor);
        assert_eq!(session.graph().node_count(), 1);
        assert_eq!(
            session.changes().last().unwrap().author,
            Author::person("native-person").via("turnstone")
        );
    });
}

/// Regression for the fixture's pre-storage replay refusal. Keep
/// raw array order, clocks, identities, and all facets exactly as admission does.
#[test]
fn native_navigation_candidate_replay_is_exact_across_edit_boundaries() {
    use mere::kernel::graph::{
        CapturedDelta, Graph,
        apply::{GraphDelta, add_node, apply_graph_delta},
        replay_captured_deltas_as_onto, with_isolated_capture,
    };
    use std::sync::{Arc, Mutex};

    fn differences(
        path: &str,
        live: &serde_json::Value,
        replay: &serde_json::Value,
        out: &mut Vec<String>,
    ) {
        if live == replay {
            return;
        }
        match (live, replay) {
            (serde_json::Value::Object(left), serde_json::Value::Object(right)) => {
                let keys: std::collections::BTreeSet<_> = left.keys().chain(right.keys()).collect();
                for key in keys {
                    differences(&format!("{path}/{key}"), &live[key], &replay[key], out);
                }
            },
            (serde_json::Value::Array(left), serde_json::Value::Array(right))
                if left.len() == right.len() =>
            {
                for (index, (left, right)) in left.iter().zip(right).enumerate() {
                    differences(&format!("{path}/{index}"), left, right, out);
                }
            },
            _ => out.push(format!("{path}: candidate={live} replay={replay}")),
        }
    }
    fn compare(
        base: &Graph,
        edit: impl FnOnce(&mut Graph),
    ) -> (Graph, Vec<String>, Vec<CapturedDelta>) {
        let author = Author::person("native-person").via("turnstone");
        let captured = Arc::new(Mutex::new(Vec::<CapturedDelta>::new()));
        let recorder = captured.clone();
        let mut candidate = base.clone();
        candidate.set_recorder(Some(Arc::new(move |delta| {
            recorder.lock().unwrap().push(delta.clone())
        })));
        with_isolated_capture(|| candidate.write_as(author.clone(), edit));
        let recorded = std::mem::take(&mut *captured.lock().unwrap());
        let mut replay = base.clone();
        replay_captured_deltas_as_onto(&mut replay, &author, recorded.iter().cloned());
        let live = serde_json::to_value((candidate.to_snapshot_at(0), candidate.facets())).unwrap();
        let replay = serde_json::to_value((replay.to_snapshot_at(0), replay.facets())).unwrap();
        let mut diff = Vec::new();
        differences("", &live, &replay, &mut diff);
        (candidate, diff, recorded)
    }
    fn add(graph: &mut Graph) {
        add_node(
            graph,
            Some(Uuid::from_u128(4999)),
            "https://navigation.test/first".into(),
            Default::default(),
        );
    }
    fn navigate(graph: &mut Graph, url: &str) {
        let key = graph.get_node_key_by_id(Uuid::from_u128(4999)).unwrap();
        apply_graph_delta(
            graph,
            GraphDelta::NavigateNode {
                key,
                url: url.into(),
            },
        );
    }
    let mut mismatches = [0; 4];
    let stages = [
        "combined add + two navigations",
        "separate add",
        "separate first navigation",
        "separate second navigation",
    ];
    for _ in 0..64 {
        let base = MereSessions::new(MemoryBackend::new()).begin_recorded(Author::user(), None);
        let (_, diff, captured) = compare(base.graph(), |graph| {
            add(graph);
            navigate(graph, "https://navigation.test/first");
            navigate(graph, "https://navigation.test/second");
        });
        if !diff.is_empty() {
            if mismatches[0] == 0 {
                eprintln!(
                    "{} raw differences: {diff:#?}\ncaptured: {captured:#?}",
                    stages[0]
                );
            }
            mismatches[0] += 1;
        }
        let (added, diff, captured) = compare(base.graph(), add);
        if !diff.is_empty() {
            if mismatches[1] == 0 {
                eprintln!(
                    "{} raw differences: {diff:#?}\ncaptured: {captured:#?}",
                    stages[1]
                );
            }
            mismatches[1] += 1;
        }
        let (first, diff, captured) = compare(&added, |graph| {
            navigate(graph, "https://navigation.test/first")
        });
        if !diff.is_empty() {
            if mismatches[2] == 0 {
                eprintln!(
                    "{} raw differences: {diff:#?}\ncaptured: {captured:#?}",
                    stages[2]
                );
            }
            mismatches[2] += 1;
        }
        let (_, diff, captured) = compare(&first, |graph| {
            navigate(graph, "https://navigation.test/second")
        });
        if !diff.is_empty() {
            if mismatches[3] == 0 {
                eprintln!(
                    "{} raw differences: {diff:#?}\ncaptured: {captured:#?}",
                    stages[3]
                );
            }
            mismatches[3] += 1;
        }
    }
    for (stage, count) in stages.into_iter().zip(mismatches) {
        eprintln!("{stage}: {count}/64 raw replay mismatches");
    }
    assert_eq!(
        mismatches, [0; 4],
        "raw replay must exactly preserve all snapshot and facet values across every edit boundary"
    );
}

/// The owned partial-display route must reach the product adapter, preserving
/// Complete product semantics and each operation's existing history boundary.
#[test]
fn owned_neighborhood_product_edits_match_complete_reopen_and_undo() {
    use graphshell::{
        product::EditableRelation,
        projection_compile::ProjectionSnapshot,
        projection_editor::{Channel, ProjectionDraft, Reading, SourceBinding},
        resident_host::{PaneRoots, ResidentSettings},
    };
    use mere::kernel::graph::{CapturedDelta, EdgeAssertion, Graph, SemanticSubKind};
    use muniment::Backend;
    use pandect::addressable_graph::{ResidencyPolicy, ResidencyStatus, ResidentGraph};
    use std::collections::BTreeSet;

    const VIA: &str = "task7.integration";
    const FACET: &str = "task7.integration.facet";

    fn observations<B: Backend + Clone>(reader: &ResidentGraph<B>, id: Uuid) -> Vec<LiveCheck> {
        let key = reader.graph().get_node_key_by_id(id).unwrap();
        let resource = reader.graph().shown_resource_id(key).unwrap();
        let mut checks = [GraphAddress::Surface(id), GraphAddress::Resource(resource)]
            .into_iter()
            .map(|address| LiveCheck::RecordUnchanged {
                address,
                digest: reader.record_digest(address).unwrap().to_owned(),
            })
            .collect::<Vec<_>>();
        checks.push(LiveCheck::ShownUnchanged {
            surface: id,
            resource,
        });
        checks.push(LiveCheck::ContentTagsUnchanged {
            surface: id,
            tags: reader
                .graph()
                .node_content_tags(key)
                .unwrap()
                .into_iter()
                .collect(),
        });
        checks
    }

    // Independent executions mint different history/statement identities and
    // clocks. Compare their product values and exact relation multiplicity;
    // reopen checks below compare the owner's entire raw state without this map.
    fn product_values(graph: &Graph) -> serde_json::Value {
        let nodes: std::collections::BTreeMap<_, _> = graph
            .nodes()
            .map(|(key, node)| {
                let tags: BTreeSet<_> = graph.node_content_tags(key).unwrap().into_iter().collect();
                (
                    node.id.to_string(),
                    serde_json::json!({
                        "resource": graph.shown_resource_id(key),
                        "title": graph.node_display_label(key),
                        "tags": tags,
                        "pinned": graph.node_is_pinned(key),
                        "facet": graph.facets().get(&node.id, &chartulary::FacetId::new(FACET)),
                    }),
                )
            })
            .collect();
        let mut relations: Vec<_> = graph
            .semantic_edges()
            .map(|edge| {
                serde_json::to_string(&(
                    graph.get_node(edge.from).unwrap().id,
                    graph.get_node(edge.to).unwrap().id,
                    edge.sub_kind,
                    edge.label,
                ))
                .unwrap()
            })
            .collect();
        relations.sort();
        serde_json::json!({ "nodes": nodes, "relations": relations })
    }

    fn raw(graph: &Graph) -> serde_json::Value {
        serde_json::json!({ "snapshot": graph.to_snapshot_at(0), "facets": graph.facets() })
    }

    pollster::block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let backend = DirectoryBackend::open(dir.path()).unwrap();
        let mut recorded = MereSessions::new(backend.clone()).begin_recorded(Author::user(), None);
        recorded.flush(SystemTime::UNIX_EPOCH).await.unwrap();
        let sid = recorded.id();
        drop(recorded);
        let mut seed = MereHost::open_session(
            backend.clone(),
            sid,
            persona(),
            HandlerRegistry::default(),
            access(),
        )
        .await
        .unwrap();
        let a = seed
            .create_address("https://partial-product.test/a", "Original A")
            .unwrap();
        let b = seed
            .create_address("https://partial-product.test/b", "Original B")
            .unwrap();
        let c = seed
            .create_address("https://partial-product.test/c", "Original C")
            .unwrap();
        let mut draft = ProjectionDraft::new();
        draft.id = "task7-one-member".into();
        draft.label = "One member".into();
        draft.source = SourceBinding {
            authority: "local".into(),
            domain: "notes".into(),
            resource: "graph:notes".into(),
        };
        draft.reading = Reading {
            kind: "nodes".into(),
            key: "id".into(),
            value: None,
        };
        draft.encoding.x = Channel::Field("x".into());
        draft.encoding.y = Channel::Field("y".into());
        draft.appearance.title = "One member".into();
        draft.appearance.realization = "canvas".into();
        draft.provenance.author = "native-person".into();
        draft.provenance.source_revision = Some("1".into());
        seed.save_projection(&ProjectionSnapshot {
            definition: draft.to_definition().unwrap(),
            selected_occurrence: None,
        })
        .unwrap();
        seed.persist(1).await.unwrap();
        let seed_values = product_values(seed.graph());
        // Copy the complete recorded seed into an independent test-only store.
        // The Complete baseline and owner therefore share every initial identity.
        let complete_backend = MemoryBackend::new();
        for key in backend.list("").await.unwrap() {
            complete_backend
                .put(&key, &backend.get(&key).await.unwrap().unwrap())
                .await
                .unwrap();
        }
        drop(seed);
        drop(backend);
        let mut complete = MereHost::open_session(
            complete_backend.clone(),
            sid,
            persona(),
            HandlerRegistry::default(),
            access(),
        )
        .await
        .unwrap();
        let mut owner = ResidentHost::open_directory(
            dir.path(),
            sid,
            persona(),
            HandlerRegistry::default(),
            access(),
        )
        .await
        .unwrap();
        assert!(
            DirectoryBackend::open(dir.path()).is_err(),
            "real lifetime admission"
        );
        assert!(owner.session().placement().is_some(), "recorded placement");
        assert_eq!(owner.settings().policy, ResidencyPolicy::Full);
        assert_eq!(
            owner.presentation_graph().node_count(),
            4,
            "three addresses plus authored projection"
        );
        assert!(matches!(
            owner.persist(SystemTime::UNIX_EPOCH).await.unwrap(),
            ResidentPersistence::Durable { .. }
        ));
        assert_eq!(product_values(owner.session().graph()), seed_values);
        assert_eq!(product_values(complete.graph()), seed_values);
        // This valid full-reader observation remains usable after B is evicted.
        // It is not synthesized from the complete private writer after eviction.
        let b_checks = observations(owner.reader().unwrap(), b);
        let full_presentation = raw(owner.presentation_graph());
        // The authored projection's pane supplies its exact one-member roster.
        let pane = pandect::ViewKey::new("graphshell", "task7-one-member").unwrap();
        owner
            .set_pane(
                pane.clone(),
                PaneRoots {
                    roster: BTreeSet::from([a]),
                    focus: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(owner.settings().policy, ResidencyPolicy::Full);
        assert_eq!(raw(owner.presentation_graph()), full_presentation);
        let reader_before_settings = raw(owner.reader().unwrap().graph());
        let generation_before_settings = owner.reader().unwrap().generation().to_owned();
        owner
            .set_settings(ResidentSettings {
                policy: ResidencyPolicy::Neighborhood { hops: 0 },
                projection: Some("task7-one-member".into()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(
            raw(owner.reader().unwrap().graph()),
            reader_before_settings,
            "staged settings preserve the published Full reader"
        );
        assert_eq!(
            owner.reader().unwrap().generation(),
            generation_before_settings,
            "staged settings do not advance the published generation"
        );
        assert_eq!(raw(owner.presentation_graph()), full_presentation);
        assert!(matches!(
            owner.persist(SystemTime::UNIX_EPOCH).await.unwrap(),
            ResidentPersistence::Durable { .. }
        ));
        let reader = owner.reader().unwrap();
        let resource = reader
            .graph()
            .shown_resource_id(reader.graph().get_node_key_by_id(a).unwrap())
            .unwrap();
        assert_eq!(
            reader.status(GraphAddress::Surface(a)),
            ResidencyStatus::Loaded
        );
        assert_eq!(
            reader.status(GraphAddress::Resource(resource)),
            ResidencyStatus::Loaded
        );
        for hidden in [b, c] {
            assert_eq!(
                reader.status(GraphAddress::Surface(hidden)),
                ResidencyStatus::NotLoaded
            );
        }
        assert_eq!(owner.presentation_graph().node_count(), 1);
        assert_eq!(owner.presentation_graph().resource_nodes().count(), 1);
        assert_eq!(owner.session().graph().node_count(), 4);
        let initial_changes = owner.session().changes().len();

        let tag_author = Author::person("native-person").via(VIA);
        let tag_concept = mere::kernel::graph::ResourceNode::for_term(
            &mere::kernel::graph::resource_tags::tag_concept_iri(
                &tag_author.asserter_iri(),
                "tagged",
            ),
        )
        .id();
        for step in 0..4 {
            let reader_before = raw(owner.reader().unwrap().graph());
            let generation_before = owner.reader().unwrap().generation().to_owned();
            let mut checks = observations(owner.reader().unwrap(), a);
            let command = match step {
                0 => LiveCommand::EditNode {
                    surface: a,
                    title: " Edited A ".into(),
                    tags: BTreeSet::from([" tagged ".into()]),
                },
                1 => LiveCommand::SetFacet {
                    surface: a,
                    facet: FACET.into(),
                    value: serde_json::json!({ "retained": [1, 2, 3] }),
                },
                2 => LiveCommand::SetPinned {
                    surface: a,
                    pinned: true,
                },
                _ => {
                    checks.extend(b_checks.clone());
                    LiveCommand::AssertRelation {
                        from: a,
                        to: b,
                        assertion: EdgeAssertion::Semantic {
                            sub_kind: SemanticSubKind::UserGrouped,
                            label: None,
                            decay_progress: None,
                        },
                    }
                },
            };
            let cursor = owner.session().journal().live_cursor();
            let status = owner
                .apply_observed_commands(
                    VIA,
                    vec![ObservedCommand { command, checks }],
                    SystemTime::UNIX_EPOCH,
                )
                .expect("owned partial edit must reach the product adapter");
            assert!(matches!(
                status,
                ResidentPersistence::AppliedButUnstored { .. }
            ));
            assert!(owner.is_busy(), "retained operation blocks a second edit");
            assert!(owner.session().journal().live_cursor() > cursor);
            assert_eq!(owner.session().changes().len(), initial_changes + step + 1);
            assert_eq!(
                owner.session().changes().last().unwrap().author,
                Author::person("native-person").via(VIA)
            );
            assert_eq!(
                raw(owner.reader().unwrap().graph()),
                reader_before,
                "reader stays at the published cut"
            );
            assert_eq!(raw(owner.presentation_graph()), reader_before);
            complete.through(VIA.into(), |host| match step {
                0 => host
                    .edit_node(a, " Edited A ", [" tagged ".into()])
                    .unwrap(),
                1 => host
                    .set_product_facet(a, FACET, r#"{"retained":[1,2,3]}"#)
                    .unwrap(),
                // Pin has no separate Complete product helper; use its existing
                // stable-ID captured-edit admission, not a raw graph mutation.
                2 => host
                    .apply_edits(vec![CapturedDelta::ReplaySetNodePinnedById {
                        node_id: a.to_string(),
                        is_pinned: true,
                    }])
                    .unwrap(),
                _ => host
                    .assert_product_relation(a, b, EditableRelation::UserGrouped)
                    .unwrap(),
            });
            assert_eq!(
                product_values(owner.session().graph()),
                product_values(complete.graph()),
                "Complete equivalence at step {step}"
            );
            assert_eq!(
                owner.session().changes().len(),
                complete.graph_session().changes().len()
            );
            assert!(matches!(
                owner.persist(SystemTime::UNIX_EPOCH).await.unwrap(),
                ResidentPersistence::Durable { .. }
            ));
            assert!(!owner.is_busy());
            assert_ne!(
                owner.reader().unwrap().generation(),
                generation_before,
                "publication advances the reader even when an edge endpoint stays unloaded"
            );
            let key = owner.presentation_graph().get_node_key_by_id(a).unwrap();
            assert_eq!(
                owner.presentation_graph().node_display_label(key),
                "Edited A"
            );
            if step == 0 {
                let reader = owner.reader().unwrap();
                assert_eq!(
                    reader.status(GraphAddress::Resource(tag_concept)),
                    ResidencyStatus::NotLoaded
                );
                assert_eq!(
                    reader.status(GraphAddress::Surface(a)),
                    ResidencyStatus::Loaded
                );
                assert_eq!(
                    reader.status(GraphAddress::Resource(resource)),
                    ResidencyStatus::Loaded
                );
                assert_eq!(owner.presentation_graph().node_count(), 1);
                assert!(
                    owner
                        .presentation_graph()
                        .node_content_tags(key)
                        .unwrap()
                        .is_empty()
                );
                let writer_key = owner.session().graph().get_node_key_by_id(a).unwrap();
                assert_eq!(
                    owner
                        .session()
                        .graph()
                        .node_content_tags(writer_key)
                        .unwrap(),
                    std::collections::HashSet::from(["tagged".into()])
                );
                let writer_before_focus = raw(owner.session().graph());
                let cursor_before_focus = owner.session().journal().live_cursor();
                let history_before_focus = owner.session().changes().to_vec();
                let generation_before_focus = reader.generation().to_owned();
                owner
                    .set_pane(
                        pane.clone(),
                        PaneRoots {
                            roster: BTreeSet::from([a]),
                            focus: Some(GraphAddress::Resource(tag_concept)),
                        },
                    )
                    .await
                    .unwrap();
                let reader = owner.reader().unwrap();
                assert_eq!(
                    reader.status(GraphAddress::Resource(tag_concept)),
                    ResidencyStatus::Loaded
                );
                for hidden in [b, c] {
                    assert_eq!(
                        reader.status(GraphAddress::Surface(hidden)),
                        ResidencyStatus::NotLoaded
                    );
                }
                assert_eq!(
                    owner.settings().policy,
                    ResidencyPolicy::Neighborhood { hops: 0 }
                );
                assert_eq!(owner.presentation_graph().node_count(), 1);
                assert_eq!(owner.presentation_graph().resource_nodes().count(), 2);
                assert_eq!(reader.generation(), generation_before_focus);
                assert_eq!(raw(owner.session().graph()), writer_before_focus);
                assert_eq!(owner.session().journal().live_cursor(), cursor_before_focus);
                assert_eq!(owner.session().changes(), history_before_focus);
            }
            // Hydration may rebuild reader-local keys; resolve the stable ID again.
            let key = owner.presentation_graph().get_node_key_by_id(a).unwrap();
            assert_eq!(
                owner.presentation_graph().node_content_tags(key).unwrap(),
                std::collections::HashSet::from(["tagged".into()])
            );
            if step >= 1 {
                assert_eq!(
                    owner
                        .presentation_graph()
                        .facets()
                        .get(&a, &chartulary::FacetId::new(FACET)),
                    Some(&serde_json::json!({ "retained": [1, 2, 3] })),
                );
            }
            assert_eq!(
                owner.presentation_graph().node_is_pinned(key),
                Some(step >= 2)
            );
            if step == 3 {
                let graph = owner.session().graph();
                let from = graph.get_node_key_by_id(a).unwrap();
                let to = graph.get_node_key_by_id(b).unwrap();
                assert_eq!(
                    graph
                        .semantic_edges()
                        .filter(|edge| edge.from == from
                            && edge.to == to
                            && edge.sub_kind == SemanticSubKind::UserGrouped)
                        .count(),
                    1
                );
            }
            assert_eq!(
                owner.presentation_graph().node_count(),
                1,
                "relation does not widen exact roster at zero hops"
            );
            complete.persist(2 + step as u64).await.unwrap();
        }
        let snapshot = raw(owner.session().graph());
        let history = serde_json::to_value(owner.session().changes()).unwrap();
        let cursor = owner.session().journal().live_cursor();
        let settings = owner.settings().clone();
        owner = owner
            .reopen(persona(), HandlerRegistry::default(), access())
            .await
            .unwrap();
        assert_eq!(
            raw(owner.session().graph()),
            snapshot,
            "exact raw reopen, including handles and clocks"
        );
        assert_eq!(
            serde_json::to_value(owner.session().changes()).unwrap(),
            history
        );
        assert_eq!(owner.session().journal().live_cursor(), cursor);
        assert_eq!(owner.settings(), &settings);
        assert!(DirectoryBackend::open(dir.path()).is_err());
        // Pane roots are attachment state, explicitly reattached after reopen.
        owner
            .set_pane(
                pane,
                PaneRoots {
                    roster: BTreeSet::from([a]),
                    focus: None,
                },
            )
            .await
            .unwrap();
        drop(complete);
        let mut complete = MereHost::open_session(
            complete_backend,
            sid,
            persona(),
            HandlerRegistry::default(),
            access(),
        )
        .await
        .unwrap();
        assert_eq!(
            product_values(owner.session().graph()),
            product_values(complete.graph())
        );
        for remaining in (0..4).rev() {
            let reader_before = raw(owner.reader().unwrap().graph());
            owner.undo(VIA, SystemTime::UNIX_EPOCH).unwrap();
            assert_eq!(raw(owner.reader().unwrap().graph()), reader_before);
            let reverted = complete
                .undo_now(VIA)
                .unwrap()
                .expect("matching Complete undo");
            assert!(reverted.kept.is_empty());
            assert_eq!(
                product_values(owner.session().graph()),
                product_values(complete.graph()),
                "undo leaves {remaining} product edits"
            );
            assert!(matches!(
                owner.persist(SystemTime::UNIX_EPOCH).await.unwrap(),
                ResidentPersistence::Durable { .. }
            ));
            complete.persist(10 + remaining).await.unwrap();
        }
        assert_eq!(product_values(owner.session().graph()), seed_values);
        assert_eq!(owner.presentation_graph().node_count(), 1);
        let raw_after_undo = raw(owner.session().graph());
        owner = owner
            .reopen(persona(), HandlerRegistry::default(), access())
            .await
            .unwrap();
        assert_eq!(
            raw(owner.session().graph()),
            raw_after_undo,
            "undo itself survives reopen"
        );
    });
}
