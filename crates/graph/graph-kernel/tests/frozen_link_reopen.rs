// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
#![cfg(all(feature = "store", not(target_arch = "wasm32")))]

use kernel::graph::apply::add_node;
use kernel::graph::{
    FrozenLinkSource, FrozenSourceBinding, FrozenSourceRevision, Graph, GraphStratum,
    SemanticStatementSpec,
};
use kernel::types::GraphScope;
use std::path::Path;
use std::process::Command;
use uuid::Uuid;

const DIRECTORY: &str = "MERE_P7_REOPEN_DIRECTORY";
const MODE: &str = "MERE_P7_REOPEN_MODE";

#[test]
fn frozen_link_and_receiving_coverage_survive_fresh_processes() {
    let directory = tempfile::tempdir().unwrap();
    for mode in ["write", "read", "receiver"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--ignored", "--exact", "frozen_link_child", "--nocapture"])
            .env(DIRECTORY, directory.path())
            .env(MODE, mode)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{mode} failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
}

#[test]
#[ignore = "separate process helper for promoted link persistence"]
fn frozen_link_child() {
    let Some(directory) = std::env::var_os(DIRECTORY) else {
        return;
    };
    let directory = Path::new(&directory);
    match std::env::var(MODE).as_deref() {
        Ok("write") => write(directory),
        Ok("read") => read(directory),
        Ok("receiver") => receiver(directory),
        _ => panic!("unknown mode"),
    }
}

fn write(directory: &Path) {
    std::fs::create_dir_all(directory.join("author")).unwrap();
    std::fs::create_dir_all(directory.join("receiver")).unwrap();
    let mut graph = Graph::new();
    let first = add_node(
        &mut graph,
        Some(Uuid::from_u128(701)),
        "https://p7-first.test/".into(),
        Default::default(),
    );
    let second = add_node(
        &mut graph,
        Some(Uuid::from_u128(702)),
        "https://p7-second.test/".into(),
        Default::default(),
    );
    let predicate = "urn:p7:personal-surface-claim";
    graph
        .declare_predicate(predicate, GraphStratum::Surface)
        .unwrap();
    let (_, claim) = graph
        .try_assert_semantic_statement(
            first,
            second,
            SemanticStatementSpec {
                predicate: predicate.into(),
                graph_scope: GraphScope::User,
                provenance_iri: Some("urn:p7:person".into()),
                asserted_at_ms: Some(10),
                ..Default::default()
            },
        )
        .unwrap();
    let owner = graph
        .freeze_link_with_nonce(
            GraphStratum::Surface,
            Uuid::from_u128(701),
            Uuid::from_u128(702),
            FrozenLinkSource::Available {
                binding: FrozenSourceBinding {
                    authority: "test-authority".into(),
                    domain: "graph".into(),
                    resource: "test-source".into(),
                },
                revision: FrozenSourceRevision::Unavailable {
                    reason: "This fixture has no public immutable source revision".into(),
                },
            },
            Uuid::from_u128(703),
        )
        .unwrap();
    let frozen = serde_json::to_value(graph.open_frozen_link(owner).unwrap().link).unwrap();
    assert!(graph.withdraw_semantic_assertion(
        &claim.statement_id,
        Some(20),
        Some("urn:p7:person".into())
    ));
    let property = kernel::types::NodeProperty::new(
        "https://mere.computer/ns/rel#reviewComment".into(),
        "A frozen review comment".into(),
    )
    .with_graph_scope(GraphScope::User)
    .with_metadata(Some("urn:p7:reviewer".into()), Some(30));
    graph
        .append_resource_properties(owner, vec![property])
        .unwrap();
    let record = graph.resource_record(owner).unwrap();
    let mut carried = Graph::new().to_snapshot();
    carried.resources = vec![record.clone()];
    let receiver = Graph::try_from_snapshot(&carried).unwrap();
    kernel::store::save_graph(&directory.join("receiver"), &receiver).unwrap();
    kernel::store::save_graph(&directory.join("author"), &graph).unwrap();
    std::fs::write(directory.join("expected.json"), serde_json::to_vec(&serde_json::json!({"owner":owner,"frozen":frozen,"claim":claim.statement_id,"record":record})).unwrap()).unwrap();
    println!(
        "P7 writer PID {} saved source withdrawal, frozen copy, comment and reference-only receiver",
        std::process::id()
    );
}

fn expected(directory: &Path) -> serde_json::Value {
    serde_json::from_slice(&std::fs::read(directory.join("expected.json")).unwrap()).unwrap()
}

fn read(directory: &Path) {
    let expected = expected(directory);
    let graph = kernel::store::load_graph(&directory.join("author"))
        .unwrap()
        .unwrap();
    let owner = serde_json::from_value(expected["owner"].clone()).unwrap();
    assert_eq!(
        serde_json::to_value(graph.open_frozen_link(owner).unwrap().link).unwrap(),
        expected["frozen"]
    );
    assert_eq!(
        serde_json::to_value(graph.resource_record(owner).unwrap()).unwrap(),
        expected["record"]
    );
    let id = expected["claim"].as_str().unwrap();
    assert_eq!(
        graph
            .find_assertion(id)
            .unwrap()
            .1
            .withdrawn
            .as_ref()
            .unwrap()
            .at_ms,
        Some(20)
    );
    assert!(graph.find_held_assertion(id).is_none());
    assert_eq!(
        graph.resource_properties(owner)[0].value,
        "A frozen review comment"
    );
    println!(
        "P7 author reader PID {} retained exact frozen JSON, source tombstone and Resource review assertion",
        std::process::id()
    );
}

fn receiver(directory: &Path) {
    let expected = expected(directory);
    let graph = kernel::store::load_graph(&directory.join("receiver"))
        .unwrap()
        .unwrap();
    let owner = serde_json::from_value(expected["owner"].clone()).unwrap();
    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.resource_nodes().count(), 1);
    let opened = graph.open_frozen_link(owner).unwrap();
    assert_eq!(
        serde_json::to_value(opened.link).unwrap(),
        expected["frozen"]
    );
    assert!(
        opened
            .coverage
            .limits
            .iter()
            .any(|limit| limit.surfaces.len() == 2)
    );
    assert!(
        opened
            .coverage
            .limits
            .iter()
            .any(|limit| limit.resources.len() == 2)
    );
    assert_eq!(
        graph.resource_properties(owner)[0].value,
        "A frozen review comment"
    );
    println!(
        "P7 receiver reader PID {} kept immutable copies and missing-reference coverage without inventing endpoints",
        std::process::id()
    );
}
