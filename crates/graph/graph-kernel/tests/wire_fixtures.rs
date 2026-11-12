// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Wire fixtures written by the kernel before the engine split's model seam
//! (P5 L3): the journal's `AttributedDelta` stream (JSON, postcard, rkyv), the
//! authority-guest candidate `mere.graph-snapshot-and-facets/json`, and the
//! rkyv graph archive. Each decodes and re-encodes to identical bytes, so a
//! type that moves between crates is proved shape-neutral.
//!
//! Regenerate only on a deliberate format change:
//! `MERE_WRITE_WIRE_FIXTURES=1 cargo test -p mere-kernel --test wire_fixtures -- --ignored write_wire_fixtures`

#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use kernel::graph::apply::{GraphDelta, add_node, apply_graph_delta};
use kernel::graph::{
    AttributedDelta, Author, CapturedDelta, Graph, GraphStratum, NodeFacetStore,
    SemanticStatementSpec, SemanticSubKind,
};
use kernel::persistence::GraphSnapshot;
use kernel::types::{
    AssertionBasis, AssertionMethod, FrameLayoutHint, GraphScope, NodeProperty, SplitOrientation,
};
use uuid::Uuid;

const WRITE_ENV: &str = "MERE_WRITE_WIRE_FIXTURES";

/// The authority-guest candidate shape (Cleromancy `authority-guest`).
#[derive(serde::Serialize, serde::Deserialize)]
struct Candidate {
    snapshot: GraphSnapshot,
    facets: NodeFacetStore,
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/wire")
}

fn read(name: &str) -> Vec<u8> {
    std::fs::read(fixture_dir().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn aligned(bytes: &[u8]) -> rkyv::util::AlignedVec<16> {
    let mut out = rkyv::util::AlignedVec::<16>::new();
    out.extend_from_slice(bytes);
    out
}

fn spec(predicate: &str, asserter: &str, at: u64, note: &str) -> SemanticStatementSpec {
    SemanticStatementSpec {
        predicate: predicate.into(),
        label: Some(format!("{note} label")),
        provenance_iri: Some(asserter.into()),
        graph_scope: GraphScope::User,
        asserted_at_ms: Some(at),
        basis: AssertionBasis {
            note: Some(note.into()),
            method: Some(AssertionMethod {
                iri: "urn:fixture:method".into(),
                version: Some("2".into()),
            }),
            evidence: vec![
                "urn:fixture:evidence-b".into(),
                "urn:fixture:evidence-a".into(),
            ],
        },
        ..Default::default()
    }
}

/// A graph exercising both strata, and the attributed deltas it recorded.
fn scenario() -> (Graph, Vec<AttributedDelta>) {
    let mut graph = Graph::new();
    let captured: Arc<Mutex<Vec<CapturedDelta>>> = Arc::default();
    let sink = captured.clone();
    graph.set_recorder(Some(Arc::new(move |delta| {
        sink.lock().unwrap().push(delta.clone());
    })));

    let a = add_node(
        &mut graph,
        Some(Uuid::from_u128(0xA1)),
        "https://a.test/page".into(),
        Default::default(),
    );
    let b = add_node(
        &mut graph,
        Some(Uuid::from_u128(0xB2)),
        "https://b.test/other".into(),
        Default::default(),
    );
    let c = add_node(
        &mut graph,
        Some(Uuid::from_u128(0xC3)),
        "https://c.test/third".into(),
        Default::default(),
    );
    apply_graph_delta(
        &mut graph,
        GraphDelta::SetNodeTitle {
            key: a,
            title: "Page A".into(),
        },
    );
    apply_graph_delta(
        &mut graph,
        GraphDelta::SetNodePinned {
            key: b,
            is_pinned: true,
        },
    );
    apply_graph_delta(
        &mut graph,
        GraphDelta::AppendFrameLayoutHint {
            key: a,
            hint: FrameLayoutHint::SplitHalf {
                first: Uuid::from_u128(0xA1).to_string(),
                second: Uuid::from_u128(0xB2).to_string(),
                orientation: SplitOrientation::Vertical,
            },
        },
    );

    // Surface reason between surfaces.
    let surface_predicate = "urn:fixture:surface-reason";
    graph
        .declare_predicate(surface_predicate, GraphStratum::Surface)
        .unwrap();
    graph
        .try_assert_semantic_reason(
            a,
            c,
            spec(surface_predicate, "urn:fixture:alice", 11, "surface"),
        )
        .unwrap();

    // Resource statements: two asserters on one pair, one withdrawn, one reconfirmed.
    let ra = graph.shown_resource_id(a).expect("a shows a resource");
    let rb = graph.shown_resource_id(b).expect("b shows a resource");
    let rc = graph.shown_resource_id(c).expect("c shows a resource");
    let cites = kernel::graph::predicate_iri(SemanticSubKind::Cites);
    let (_, alice) = graph
        .try_assert_semantic_statement_by_resource_ids(
            ra,
            rb,
            spec(cites, "urn:fixture:alice", 20, "alice"),
        )
        .unwrap();
    let (_, bob) = graph
        .try_assert_semantic_statement_by_resource_ids(
            ra,
            rb,
            spec(cites, "urn:fixture:bob", 21, "bob"),
        )
        .unwrap();
    assert_ne!(alice.statement_id, bob.statement_id);
    assert!(graph.withdraw_semantic_assertion(
        &bob.statement_id,
        Some(30),
        Some("urn:fixture:bob".into())
    ));
    let open = "urn:fixture:open-predicate";
    graph
        .declare_predicate(open, GraphStratum::Resource)
        .unwrap();
    graph
        .try_assert_semantic_statement_by_resource_ids(
            rb,
            rc,
            spec(open, "urn:fixture:carol", 40, "carol"),
        )
        .unwrap();

    graph
        .append_resource_properties(
            rc,
            vec![NodeProperty {
                statement_id: "fixture:property".into(),
                predicate: "https://schema.org/datePublished".into(),
                value: "2026-11-12".into(),
                datatype: Some("http://www.w3.org/2001/XMLSchema#date".into()),
                lang: None,
                graph_scope: GraphScope::Source,
                provenance_iri: Some("urn:fixture:alice".into()),
                asserted_at_ms: Some(50),
            }],
        )
        .unwrap();
    graph
        .freeze_resource_selection_with_nonce(
            serde_json::json!({"query": "fixture"}),
            vec![ra, rb],
            Uuid::from_u128(0xF1),
        )
        .unwrap();

    let deltas = captured.lock().unwrap().clone();
    let agent = Author::engine("fixture-engine", "1");
    let entries = deltas
        .into_iter()
        .enumerate()
        .map(|(i, delta)| AttributedDelta {
            author: if i % 3 == 2 {
                agent.clone()
            } else {
                Author::user()
            },
            delta,
        })
        .collect();
    (graph, entries)
}

fn variant(delta: &CapturedDelta) -> String {
    let value = serde_json::to_value(delta).unwrap();
    match value {
        serde_json::Value::Object(map) => map.keys().next().unwrap().clone(),
        serde_json::Value::String(name) => name,
        other => panic!("unexpected variant shape {other}"),
    }
}

#[test]
#[ignore = "writes the wire fixtures; run only on a deliberate format change"]
fn write_wire_fixtures() {
    if std::env::var_os(WRITE_ENV).is_none() {
        return;
    }
    let (graph, journal) = scenario();
    let dir = fixture_dir();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("journal.json"),
        serde_json::to_vec(&journal).unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("journal.postcard"),
        postcard::to_allocvec(&journal).unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("journal.rkyv"),
        rkyv::to_bytes::<rkyv::rancor::Error>(&journal)
            .unwrap()
            .as_slice(),
    )
    .unwrap();
    let candidate = Candidate {
        snapshot: graph.to_snapshot_at(1_700_000_000),
        facets: graph.facets().clone(),
    };
    std::fs::write(
        dir.join("candidate.json"),
        serde_json::to_vec(&candidate).unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("snapshot.rkyv"),
        rkyv::to_bytes::<rkyv::rancor::Error>(&candidate.snapshot)
            .unwrap()
            .as_slice(),
    )
    .unwrap();
}

#[test]
fn journal_fixture_covers_resource_and_surface_variants() {
    let journal: Vec<AttributedDelta> = serde_json::from_slice(&read("journal.json")).unwrap();
    let names: std::collections::BTreeSet<String> =
        journal.iter().map(|entry| variant(&entry.delta)).collect();
    for expected in [
        "ReplaySetResourceRecordById",
        "ReplaySetResourceEdgesByIds",
        "ReplaySetShownResourceById",
        "ReplayAddNodeWithIdIfMissing",
        "ReplaySetNodeTitleById",
        "ReplaySetNodePinnedById",
        "ReplayAppendFrameLayoutHintById",
    ] {
        assert!(
            names.contains(expected),
            "journal fixture lacks {expected}: {names:?}"
        );
    }
    assert!(journal.iter().any(|entry| entry.author != Author::user()));
}

#[test]
fn journal_json_postcard_and_rkyv_reencode_byte_identical() {
    let json = read("journal.json");
    let decoded: Vec<AttributedDelta> = serde_json::from_slice(&json).unwrap();
    assert_eq!(serde_json::to_vec(&decoded).unwrap(), json);

    let postcard_bytes = read("journal.postcard");
    let from_postcard: Vec<AttributedDelta> = postcard::from_bytes(&postcard_bytes).unwrap();
    assert_eq!(from_postcard, decoded);
    assert_eq!(
        postcard::to_allocvec(&from_postcard).unwrap(),
        postcard_bytes
    );

    let rkyv_bytes = read("journal.rkyv");
    let from_rkyv =
        rkyv::from_bytes::<Vec<AttributedDelta>, rkyv::rancor::Error>(&aligned(&rkyv_bytes))
            .unwrap();
    assert_eq!(from_rkyv, decoded);
    assert_eq!(
        rkyv::to_bytes::<rkyv::rancor::Error>(&from_rkyv)
            .unwrap()
            .as_slice(),
        rkyv_bytes.as_slice()
    );
}

#[test]
fn journal_fixture_replays_to_the_candidate_graph() {
    let journal: Vec<AttributedDelta> = serde_json::from_slice(&read("journal.json")).unwrap();
    let candidate: Candidate = serde_json::from_slice(&read("candidate.json")).unwrap();
    let mut graph = Graph::new();
    kernel::graph::replay_captured_deltas_onto(&mut graph, journal.into_iter().map(|e| e.delta));
    assert_eq!(
        serde_json::to_value(graph.to_snapshot_at(1_700_000_000)).unwrap(),
        serde_json::to_value(&candidate.snapshot).unwrap()
    );
    assert_eq!(
        serde_json::to_value(graph.facets()).unwrap(),
        serde_json::to_value(&candidate.facets).unwrap()
    );
}

#[test]
fn authority_candidate_json_reencodes_byte_identical_through_the_graph() {
    let json = read("candidate.json");
    let candidate: Candidate = serde_json::from_slice(&json).unwrap();
    assert!(!candidate.snapshot.resources.is_empty());
    assert!(!candidate.snapshot.resource_edges.is_empty());
    assert!(!candidate.snapshot.shown_resources.is_empty());
    assert_eq!(
        serde_json::to_vec(&candidate).unwrap(),
        json,
        "type-level round trip"
    );

    let mut graph = Graph::try_from_snapshot(&candidate.snapshot).unwrap();
    graph.install_facets(candidate.facets.clone()).unwrap();
    let again = Candidate {
        snapshot: graph.to_snapshot_at(candidate.snapshot.timestamp_secs),
        facets: graph.facets().clone(),
    };
    assert_eq!(
        serde_json::to_vec(&again).unwrap(),
        json,
        "graph round trip"
    );
}

#[test]
fn snapshot_rkyv_reencodes_byte_identical_and_restores_the_graph() {
    let bytes = read("snapshot.rkyv");
    let snapshot =
        rkyv::from_bytes::<GraphSnapshot, rkyv::rancor::Error>(&aligned(&bytes)).unwrap();
    assert_eq!(
        rkyv::to_bytes::<rkyv::rancor::Error>(&snapshot)
            .unwrap()
            .as_slice(),
        bytes.as_slice()
    );
    let candidate: Candidate = serde_json::from_slice(&read("candidate.json")).unwrap();
    assert_eq!(
        serde_json::to_value(&snapshot).unwrap(),
        serde_json::to_value(&candidate.snapshot).unwrap()
    );
    let graph = rkyv::from_bytes::<Graph, rkyv::rancor::Error>(&aligned(&bytes)).unwrap();
    assert_eq!(
        rkyv::to_bytes::<rkyv::rancor::Error>(&graph.to_snapshot_at(snapshot.timestamp_secs))
            .unwrap()
            .as_slice(),
        bytes.as_slice(),
        "graph round trip"
    );
}
