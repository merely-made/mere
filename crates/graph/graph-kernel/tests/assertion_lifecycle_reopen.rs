// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

#![cfg(all(feature = "store", not(target_arch = "wasm32")))]

use std::path::Path;
use std::process::Command;

use kernel::graph::apply::add_node;
use kernel::graph::{
    Graph, GraphStratum, SemanticStatement, SemanticStatementSpec, SemanticSubKind,
};
use kernel::types::{AssertionBasis, AssertionMethod, GraphScope};
use uuid::Uuid;

const PROBE_DIR: &str = "MERE_LINK_MODEL_REOPEN_DIR";
const PROBE_MODE: &str = "MERE_LINK_MODEL_REOPEN_MODE";
const ACTOR: &str = "urn:fixture:author";

#[test]
fn assertion_lifecycle_survives_a_fresh_process() {
    let directory = tempfile::tempdir().unwrap();
    let executable = std::env::current_exe().unwrap();
    for mode in ["write", "read"] {
        let result = Command::new(&executable)
            .args([
                "--ignored",
                "--exact",
                "lifecycle_reopen_child",
                "--nocapture",
            ])
            .env(PROBE_DIR, directory.path())
            .env(PROBE_MODE, mode)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{mode} child failed:\n{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        print!("{}", String::from_utf8_lossy(&result.stdout));
    }
}

// Invoked only by the parent with a fresh process for each side of the store boundary.
#[test]
#[ignore = "subprocess helper for assertion_lifecycle_survives_a_fresh_process"]
fn lifecycle_reopen_child() {
    let Some(directory) = std::env::var_os(PROBE_DIR) else {
        return;
    };
    let directory = Path::new(&directory);
    match std::env::var(PROBE_MODE).as_deref() {
        Ok("write") => write_fixture(directory),
        Ok("read") => read_fixture(directory),
        _ => panic!("unknown lifecycle reopen mode"),
    }
}

fn write_fixture(directory: &Path) {
    let mut graph = Graph::new();
    // Separate hosts isolate lifecycle storage from legacy derived Domain repair.
    let from = add_node(
        &mut graph,
        Some(Uuid::from_u128(0x5201)),
        "https://lifecycle-from.test/".into(),
        Default::default(),
    );
    let to = add_node(
        &mut graph,
        Some(Uuid::from_u128(0x5202)),
        "https://lifecycle-to.test/".into(),
        Default::default(),
    );
    let predicate = "urn:fixture:surface-reason";
    graph
        .declare_predicate(predicate, GraphStratum::Surface)
        .unwrap();
    let reason = SemanticStatementSpec {
        predicate: predicate.into(),
        provenance_iri: Some(ACTOR.into()),
        graph_scope: GraphScope::User,
        asserted_at_ms: Some(10),
        basis: AssertionBasis {
            note: Some("retained reason".into()),
            method: Some(AssertionMethod {
                iri: "urn:fixture:method".into(),
                version: Some("1".into()),
            }),
            evidence: vec![
                "urn:fixture:evidence-b".into(),
                "urn:fixture:evidence-a".into(),
            ],
        },
        ..Default::default()
    };
    let (_, old) = graph
        .try_assert_semantic_reason(from, to, reason.clone())
        .unwrap();
    assert!(graph.reconfirm_semantic_assertion(
        &old.statement_id,
        Some("confirmed reason".into()),
        Some(20),
    ));
    assert!(graph.withdraw_semantic_assertion(&old.statement_id, Some(30), Some(ACTOR.into())));
    let (_, held) = graph
        .try_assert_semantic_reason(
            from,
            to,
            SemanticStatementSpec {
                asserted_at_ms: Some(40),
                ..reason
            },
        )
        .unwrap();
    assert_ne!(old.statement_id, held.statement_id);
    graph
        .try_assert_persisted_semantic_statement(
            from,
            to,
            SemanticStatement {
                statement_id: "fixture:unknown-first".into(),
                predicate: kernel::graph::predicate_iri(SemanticSubKind::Cites).into(),
                recognized_sub_kind: Some(SemanticSubKind::Cites),
                label: None,
                basis: Default::default(),
                graph_scope: GraphScope::Source,
                provenance_iri: Some(ACTOR.into()),
                first_asserted_ms: None,
                asserted_at_ms: Some(7),
                withdrawn: None,
            },
        )
        .unwrap();
    kernel::store::save_graph(directory, &graph).unwrap();
    std::fs::write(
        directory.join("expected.json"),
        serde_json::to_vec(&graph.to_snapshot_at(0)).unwrap(),
    )
    .unwrap();
    std::fs::write(
        directory.join("handles.json"),
        serde_json::to_vec(&[old.statement_id, held.statement_id]).unwrap(),
    )
    .unwrap();
    println!("wrote lifecycle fixture and exited before reopen");
}

fn read_fixture(directory: &Path) {
    let graph = kernel::store::load_graph(directory).unwrap().unwrap();
    let expected: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("expected.json")).unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(graph.to_snapshot_at(0)).unwrap(),
        expected
    );
    let handles: Vec<String> =
        serde_json::from_slice(&std::fs::read(directory.join("handles.json")).unwrap()).unwrap();
    let old = graph.find_semantic_statement(&handles[0]).unwrap().1;
    assert_eq!(
        (old.first_asserted_ms, old.asserted_at_ms),
        (Some(10), Some(20))
    );
    assert_eq!(old.withdrawn.as_ref().unwrap().at_ms, Some(30));
    assert_eq!(old.withdrawn.as_ref().unwrap().by.as_deref(), Some(ACTOR));
    assert_eq!(old.basis.note.as_deref(), Some("confirmed reason"));
    assert_eq!(
        old.basis.method.as_ref().unwrap().version.as_deref(),
        Some("1")
    );
    assert_eq!(old.basis.evidence.len(), 2);
    assert!(graph.find_held_semantic_statement(&handles[0]).is_none());
    let held = graph.find_held_semantic_statement(&handles[1]).unwrap().1;
    assert_eq!(
        (held.first_asserted_ms, held.asserted_at_ms),
        (Some(40), Some(40))
    );
    let unknown = graph
        .find_held_semantic_statement("fixture:unknown-first")
        .unwrap()
        .1;
    assert_eq!(
        (unknown.first_asserted_ms, unknown.asserted_at_ms),
        (None, Some(7))
    );
    println!("fresh process retained IDs, basis, withdrawal and unknown first time");
}
