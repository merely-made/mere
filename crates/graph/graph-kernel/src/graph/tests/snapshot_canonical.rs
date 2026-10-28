// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The snapshot is canonical at the source: one graph serializes to one byte
//! string whatever order its fields, couplings and edges were added in, and
//! after a save and load.

use super::super::apply::{GraphDelta, GraphDeltaResult, add_node, apply_graph_delta};
use super::super::*;
use crate::persistence::{PersistedEdge, PersistedSemanticStatement};
use crate::types::{AssertionBasis, GraphScope};

const NODES: u128 = 4;

fn claim(handle: &str) -> PersistedSemanticStatement {
    PersistedSemanticStatement {
        statement_id: handle.into(),
        predicate: "urn:canonical:p".into(),
        recognized_sub_kind: None,
        label: None,
        graph_scope: GraphScope::User,
        provenance_iri: None,
        asserted_at_ms: None,
        first_asserted_ms: None,
        withdrawn: None,
        basis: AssertionBasis::default(),
    }
}

/// One edge row per claim, as `ReplaySetEdgesByIds` stores them.
fn edges(from: u128, to: u128, handles: &[&str]) -> GraphDelta {
    let row = |handle: &&str| PersistedEdge {
        from_node_id: Uuid::from_u128(from).to_string(),
        to_node_id: Uuid::from_u128(to).to_string(),
        families: Vec::new(),
        semantic: None,
        traversal: None,
        containment: None,
        arrangement: None,
        imported: None,
        provenance: None,
        assertions: Some(vec![claim(handle)]),
    };
    GraphDelta::ReplaySetEdgesByIds {
        from_id: Uuid::from_u128(from),
        to_id: Uuid::from_u128(to),
        edges: handles.iter().map(row).collect(),
    }
}

/// Eight fields, three couplings and six edge sets, each applied in the order
/// `fields`, `couplings` and `edge_sets` index them. Nodes keep one order:
/// they are not among the things being reordered.
fn graph_built(fields: &[u128], couplings: &[u128], edge_sets: &[usize]) -> Graph {
    let mut graph = Graph::new();
    for node in 1..=NODES {
        add_node(
            &mut graph,
            Some(Uuid::from_u128(node)),
            format!("https://n{node}.test/"),
            Default::default(),
        );
    }
    for &n in fields {
        let field = Field::new(
            FieldId(Uuid::from_u128(n)),
            FieldDefinition::Scalar(ScalarField::Const(n as f32)),
        )
        .with_name(format!("field {n}"));
        apply_graph_delta(&mut graph, GraphDelta::AddField { field });
    }
    for &n in couplings {
        let coupling = Coupling::new(
            CouplingId(Uuid::from_u128(100 + n)),
            FieldId(Uuid::from_u128(1 + n)),
            NodeSelector::All,
            CouplingResponse::AttractToMin,
            1.0 + n as f32,
        );
        apply_graph_delta(&mut graph, GraphDelta::AddCoupling { coupling });
    }
    let sets = [
        edges(1, 2, &["a", "b"]),
        edges(2, 3, &["c"]),
        edges(3, 4, &["d"]),
        edges(4, 1, &["e"]),
        edges(1, 3, &["f"]),
        edges(1, 2, &["b", "a"]),
    ];
    for &set in edge_sets {
        let result = apply_graph_delta(&mut graph, sets[set].clone());
        assert!(matches!(result, GraphDeltaResult::Applied), "{result:?}");
    }
    graph
}

fn bytes(graph: &Graph) -> Vec<u8> {
    serde_json::to_vec(&graph.to_snapshot_at(0)).unwrap()
}

#[test]
fn a_snapshot_is_canonical_whatever_the_insertion_or_hash_order() {
    // The forward build stores 1 -> 2 as rows a, b and the shuffled build as
    // b, a, so the two also differ in the order of one pair's rows.
    let forward = graph_built(&[1, 2, 3, 4, 5, 6, 7, 8], &[0, 1, 2], &[0, 1, 2, 3, 4]);
    let shuffled = graph_built(&[6, 2, 8, 1, 4, 7, 3, 5], &[2, 0, 1], &[5, 4, 3, 2, 1]);
    assert_eq!(forward.fields().count(), 8);
    assert_eq!(forward.couplings().count(), 3);

    let expected = bytes(&forward);
    assert_eq!(bytes(&shuffled), expected, "insertion order");
    let reloaded = Graph::from_snapshot(&serde_json::from_slice(&expected).unwrap());
    assert_eq!(bytes(&reloaded), expected, "a save and load");

    let snapshot = forward.to_snapshot_at(0);
    let ids: Vec<_> = snapshot
        .fields
        .iter()
        .map(|field| field.id.clone())
        .collect();
    assert!(ids.is_sorted(), "fields in id order");
    assert_eq!(snapshot.edges.len(), 6, "one row per claim");
}
