// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! V3's durable receipt: an owner saves, exits, and a real child process opens
//! the reservoir/session/archive stores, checks exact state and makes an edit fork.

use muniment::Backend;
use pandect::graph_codicil::{
    archive_timestamp, load_graph_codicil, parse_codicil_id, save_session_codicil_checked,
};
use pandect::manifest::PersonaId;
use pandect::{
    Author, CapturedDelta, DomainId, MereId, MereSessions, ReservoirStore, ViewIntent, ViewKey,
    open_mere_archive_backend, open_mere_backend, open_reservoir_backend,
};
use std::process::Command;
use std::time::SystemTime;
use uuid::Uuid;

const CHILD: &str = "archive_child_reopens_and_forks";
const ROOT: &str = "MERE_V3_ARCHIVE_RECEIPT_ROOT";
const ID: &str = "MERE_V3_ARCHIVE_RECEIPT_CODICIL";
const MERE: &str = "MERE_V3_ARCHIVE_RECEIPT_MERE";
const CONTEND: &str = "MERE_V3_ARCHIVE_RECEIPT_CONTEND";
fn persona() -> PersonaId {
    PersonaId::from_uuid(Uuid::from_u128(0x7633))
}
fn author() -> Author {
    Author::person(persona().as_uuid().to_string()).via("cleromancy")
}
fn state(graph: &pandect::graph_codicil::GraphCandidate) -> Vec<u8> {
    serde_json::to_vec(&serde_json::to_value((graph.to_snapshot(), graph.facets())).unwrap())
        .unwrap()
}

#[test]
fn archive_survives_owner_exit_and_a_fresh_process_forks_the_thaw() {
    let root = tempfile::tempdir().unwrap();
    let (mere, codicil, expected, contention) = pollster::block_on(async {
        let mut reservoir = ReservoirStore::open(
            open_reservoir_backend(root.path(), persona()).unwrap(),
            persona(),
        )
        .await
        .unwrap();
        let mere = reservoir
            .ensure(DomainId::new("divination").unwrap(), 42)
            .await
            .unwrap()
            .0
            .id;
        let backend = open_mere_backend(root.path(), persona(), mere).unwrap();
        let sessions = MereSessions::new(backend.clone());
        let mut live = sessions.begin(author(), None);
        live.apply(
            author(),
            vec![CapturedDelta::ReplayAddNodeWithIdIfMissing {
                id: Uuid::from_u128(1).to_string(),
                url: "mere://v3/kept-reading".into(),
                position: [4.0, 9.0],
            }],
        )
        .await
        .unwrap();
        live.set_view(
            author(),
            ViewKey::new("cleromancy", "scene").unwrap(),
            ViewIntent {
                focus: Some("mere://v3/kept-reading".into()),
                strategy: Some("spectral.default".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let expected = state(live.graph());
        let mut archive = open_mere_archive_backend(root.path(), persona(), mere).unwrap();
        assert!(
            open_mere_archive_backend(root.path(), persona(), mere).is_err(),
            "second archive owner is clearly refused"
        );
        assert!(
            open_mere_backend(root.path(), persona(), mere).is_err(),
            "second session owner is clearly refused"
        );
        let contender = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD, "--ignored", "--nocapture"])
            .env(ROOT, root.path())
            .env(MERE, mere.to_string())
            .env(CONTEND, "1")
            .output()
            .unwrap();
        assert!(
            contender.status.success(),
            "real second owner failed its refusal check: {} {}",
            String::from_utf8_lossy(&contender.stdout),
            String::from_utf8_lossy(&contender.stderr)
        );
        let contention = String::from_utf8_lossy(&contender.stdout).into_owned();
        assert!(contention.contains("V3_RECEIPT real second process refused"));
        let id = save_session_codicil_checked(
            &mut archive,
            live.codicil_payload(),
            archive_timestamp(),
            |_| Ok(()),
        )
        .await
        .unwrap();
        (mere, id, expected, contention)
    });
    let expected_path = root.path().join("expected.json");
    std::fs::write(&expected_path, &expected).unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD, "--ignored", "--nocapture"])
        .env(ROOT, root.path())
        .env(ID, codicil.to_string())
        .env(MERE, mere.to_string())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "child failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("V3_RECEIPT child reopened exact state and independently forked")
    );
    pollster::block_on(async {
        let backend = open_mere_backend(root.path(), persona(), mere).unwrap();
        let sessions = MereSessions::new(backend);
        let manifests = sessions.list().await.unwrap();
        assert_eq!(manifests.len(), 3);
        let thaw = manifests.iter().find(|m| m.codicil_read_only).unwrap();
        assert_eq!(
            state(sessions.open(thaw.session_id).await.unwrap().graph()),
            expected
        );
        let fork = manifests
            .iter()
            .find(|m| m.parent_session == Some(thaw.session_id))
            .unwrap();
        assert_eq!(fork.source_codicil, thaw.source_codicil);
        assert!(!fork.codicil_read_only);
    });
    let receipt = serde_json::json!({"schema":"mere.reservoir-v3-receipt/v1","result":"passed",
        "scope":"durable owner exit, real child process reopen, exact canonical graph/facet bytes, views, read-only direct APIs, edit-fork lineage, second-owner refusals",
        "parent_pid":std::process::id(),"contention_stdout":contention,
        "source_revision":std::env::var("MERE_V3_SOURCE_REVISION").unwrap_or_else(|_| "uncommitted-source-hash-below".into()),"source_hash":blake3::hash(concat!(include_str!("../src/graph_session.rs"),include_str!("../src/graph_codicil.rs"),include_str!("../src/reservoir.rs")).as_bytes()).to_hex().to_string(),
        "codicil":codicil.to_string(),"state_hash":blake3::hash(&expected).to_hex().to_string(),"child_stdout":String::from_utf8_lossy(&output.stdout)});
    println!("{receipt}");
    if let Some(path) = std::env::var_os("MERE_V3_RECEIPT_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
    }
}

#[test]
#[ignore = "spawned by the receipt parent with isolated fixture stores"]
fn archive_child_reopens_and_forks() {
    let root = std::path::PathBuf::from(std::env::var_os(ROOT).unwrap());
    let mere = MereId(Uuid::parse_str(&std::env::var(MERE).unwrap()).unwrap());
    if std::env::var_os(CONTEND).is_some() {
        let archive_error = open_mere_archive_backend(&root, persona(), mere)
            .err()
            .expect("real second archive owner must be refused");
        let session_error = open_mere_backend(&root, persona(), mere)
            .err()
            .expect("real second session owner must be refused");
        println!(
            "V3_RECEIPT real second process refused; pid={}; archive={archive_error}; session={session_error}",
            std::process::id()
        );
        return;
    }
    let id = parse_codicil_id(&std::env::var(ID).unwrap()).unwrap();
    pollster::block_on(async {
        let reservoir =
            ReservoirStore::open(open_reservoir_backend(&root, persona()).unwrap(), persona())
                .await
                .unwrap();
        assert_eq!(reservoir.meres().count(), 1);
        let backend = open_mere_backend(&root, persona(), mere).unwrap();
        let sessions = MereSessions::new(backend);
        let mut archive = open_mere_archive_backend(&root, persona(), mere).unwrap();
        let mut keys = archive.list("").await.unwrap();
        keys.sort();
        let mut archive_bytes = Vec::new();
        for key in keys {
            archive_bytes.push((key.clone(), archive.get(&key).await.unwrap().unwrap()));
        }
        let kept = load_graph_codicil(&mut archive, id).await.unwrap().unwrap();
        assert_eq!(kept.sessions[0].journal.len(), 2);
        let thaw = sessions
            .open_codicil_from_checked(&mut archive, id, author(), |_| Ok(()))
            .await
            .unwrap();
        let mut thaw = sessions.open(thaw.session_id).await.unwrap();
        let expected = std::fs::read(root.join("expected.json")).unwrap();
        assert_eq!(state(thaw.graph()), expected);
        let key = ViewKey::new("cleromancy", "scene").unwrap();
        assert_eq!(
            thaw.view(&key).unwrap().strategy.as_deref(),
            Some("spectral.default")
        );
        let edit = CapturedDelta::ReplaySetNodeTitleById {
            node_id: Uuid::from_u128(1).to_string(),
            title: "Edited child fork".into(),
        };
        assert!(matches!(
            thaw.apply(author(), vec![edit.clone()]).await,
            Err(pandect::SessionError::ReadOnly(_))
        ));
        let mut fork = sessions
            .begin_fork_at(&thaw, thaw.journal().live_cursor(), author())
            .unwrap();
        fork.apply_now(author(), vec![edit]).unwrap();
        fork.flush(SystemTime::now()).await.unwrap();
        assert_ne!(state(fork.graph()), expected);
        assert_eq!(state(thaw.graph()), expected);
        assert_eq!(fork.view(&key), thaw.view(&key));
        for (key, bytes) in archive_bytes {
            assert_eq!(
                archive.get(&key).await.unwrap().unwrap(),
                bytes,
                "immutable archive unchanged"
            );
        }
        println!(
            "V3_RECEIPT child reopened exact state and independently forked; pid={}",
            std::process::id()
        );
    });
}
