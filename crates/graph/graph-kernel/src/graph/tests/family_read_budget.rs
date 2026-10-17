// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use crate::graph::{ContainmentSubKind, Graph, RelationKind, SemanticStatementSpec};
use crate::types::{AssertionBasis, AssertionWithdrawal};
use std::time::{Duration, Instant};

/// A debug-build interaction budget, not a device frame-rate claim. Large
/// history must neither re-enter held reads nor make a pair query unbounded.
#[test]
fn family_read_budget_keeps_fifty_thousand_withdrawals_out_of_held_rows() {
    let mut seed = Graph::new();
    let from = seed.add_node("urn:budget:from".into(), Default::default());
    let to = seed.add_node("urn:budget:to".into(), Default::default());
    let predicate = RelationKind::Containment(ContainmentSubKind::CollectionMember)
        .predicate_iri()
        .unwrap();
    for evidence in ["urn:reason:first", "urn:reason:second"] {
        seed.try_assert_relation_statement(
            from,
            to,
            SemanticStatementSpec {
                predicate: predicate.into(),
                provenance_iri: Some("urn:agent:budget".into()),
                basis: AssertionBasis {
                    evidence: vec![evidence.into()],
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();
    }
    let mut snapshot = seed.to_snapshot_at(0);
    let records = snapshot.edges[0].assertions.as_mut().unwrap();
    let held = records.clone();
    for index in 0..50_000 {
        let mut dead = held[0].clone();
        dead.statement_id = format!("urn:budget:withdrawn:{index}");
        dead.withdrawn = Some(AssertionWithdrawal {
            at_ms: None,
            by: Some("urn:agent:budget".into()),
        });
        records.push(dead);
    }
    // Declared wire-memory budget for this substantial-history fixture.
    // Runtime peak memory is measured separately against the test binary.
    let encoded_bytes = serde_json::to_vec(&snapshot).unwrap().len();
    assert!(
        encoded_bytes < 64 * 1024 * 1024,
        "history snapshot exceeds 64 MiB fixture budget"
    );
    let start = Instant::now();
    let graph = Graph::try_from_recorded_snapshot(&snapshot).unwrap();
    let load = start.elapsed();
    let mut reads = Vec::new();
    for _ in 0..64 {
        let start = Instant::now();
        assert_eq!(
            std::hint::black_box(graph.projected_assertions().count()),
            2
        );
        assert_eq!(std::hint::black_box(graph.projected_relations().count()), 2);
        reads.push(start.elapsed());
    }
    reads.sort_unstable();
    let p95 = reads[60];
    eprintln!(
        "P6 debug read budget: retained=50002 held=2 snapshot-bytes={encoded_bytes} wire budget=64MiB load={load:?} pair-query p50={:?} p95={p95:?} max={:?}; p95 budget=100ms",
        reads[32], reads[63]
    );
    assert!(
        p95 < Duration::from_millis(100),
        "held pair query exceeded declared debug interaction budget: {p95:?}"
    );
    let actual = graph.to_snapshot_at(0);
    assert_eq!(
        actual.edges[0].retained_assertions(),
        snapshot.edges[0].retained_assertions()
    );
}
