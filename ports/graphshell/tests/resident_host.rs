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
