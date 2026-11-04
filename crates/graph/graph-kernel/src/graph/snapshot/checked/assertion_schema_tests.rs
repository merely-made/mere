// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::persistence::{PersistedEdge, PersistedSemanticEdgeData, PersistedSemanticStatement};
use crate::types::{AssertionBasis, GraphScope};
use euclid::default::Point2D;

fn base_snapshot() -> GraphSnapshot {
    let mut graph = Graph::new();
    graph.add_node("https://one.test/".into(), Point2D::new(0.0, 0.0));
    graph.add_node("https://two.test/".into(), Point2D::new(1.0, 0.0));
    graph.to_snapshot()
}

fn statement(id: &str, predicate: &str) -> PersistedSemanticStatement {
    PersistedSemanticStatement {
        statement_id: id.into(),
        predicate: predicate.into(),
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

fn canonical_edge(
    from: &str,
    to: &str,
    assertions: Vec<PersistedSemanticStatement>,
) -> PersistedEdge {
    PersistedEdge {
        from_node_id: from.into(),
        to_node_id: to.into(),
        families: Vec::new(),
        semantic: None,
        traversal: None,
        containment: None,
        arrangement: None,
        imported: None,
        provenance: None,
        assertions: Some(assertions),
    }
}

#[test]
fn schema_gate_rejects_unknown_and_mixed_formats_but_accepts_explicit_empty() {
    let mut snapshot = base_snapshot();
    snapshot.assertion_schema = 9;
    assert!(matches!(
        Graph::try_from_snapshot(&snapshot),
        Err(ResourceSnapshotError::UnsupportedAssertionSchema { schema: 9 })
    ));

    let mut snapshot = base_snapshot();
    let from = snapshot.nodes[0].node_id.clone();
    let to = snapshot.nodes[1].node_id.clone();
    snapshot.assertion_schema = 1;
    let mut mixed = canonical_edge(&from, &to, vec![statement("x", "urn:p")]);
    mixed.semantic = Some(PersistedSemanticEdgeData {
        statements: Vec::new(),
        sub_kinds: vec![crate::persistence::PersistedSemanticSubKind::UserGrouped],
        predicate: None,
        label: None,
        agent_decay_progress: None,
    });
    snapshot.edges.push(mixed);
    assert!(matches!(
        Graph::try_from_snapshot(&snapshot),
        Err(ResourceSnapshotError::MixedAssertionFormat {
            column: "edges",
            edge_index: 0
        })
    ));

    let mut resource_mixed = base_snapshot();
    resource_mixed.assertion_schema = 1;
    let mut legacy_resource_edge = canonical_edge("bad-from", "bad-to", vec![]);
    legacy_resource_edge.assertions = None;
    legacy_resource_edge.semantic = Some(PersistedSemanticEdgeData {
        statements: Vec::new(),
        sub_kinds: vec![crate::persistence::PersistedSemanticSubKind::UserGrouped],
        predicate: None,
        label: None,
        agent_decay_progress: None,
    });
    resource_mixed.resource_edges.push(legacy_resource_edge);
    assert!(matches!(
        Graph::try_from_snapshot(&resource_mixed),
        Err(ResourceSnapshotError::MissingCanonicalAssertions {
            column: "resource_edges",
            edge_index: 0
        })
    ));

    let mut empty = base_snapshot();
    empty.assertion_schema = 1;
    let from = empty.nodes[0].node_id.clone();
    let to = empty.nodes[1].node_id.clone();
    let edge = canonical_edge(&from, &to, Vec::new());
    let encoded = serde_json::to_value(&edge).unwrap();
    assert_eq!(encoded["assertions"], serde_json::json!([]));
    empty.edges.push(edge);
    assert!(Graph::try_from_snapshot(&empty).is_ok());
}

#[test]
fn canonical_schema_requires_assertions_and_valid_surface_endpoints() {
    let mut legacy = base_snapshot();
    legacy.assertion_schema = 1;
    let from = legacy.nodes[0].node_id.clone();
    let to = legacy.nodes[1].node_id.clone();
    let mut old = canonical_edge(&from, &to, Vec::new());
    old.assertions = None;
    old.semantic = Some(PersistedSemanticEdgeData {
        statements: Vec::new(),
        sub_kinds: vec![crate::persistence::PersistedSemanticSubKind::UserGrouped],
        predicate: None,
        label: None,
        agent_decay_progress: None,
    });
    legacy.edges.push(old);
    assert!(matches!(
        Graph::try_from_snapshot(&legacy),
        Err(ResourceSnapshotError::MissingCanonicalAssertions {
            column: "edges",
            edge_index: 0
        })
    ));

    let mut orphan = base_snapshot();
    orphan.assertion_schema = 1;
    orphan.edges.push(canonical_edge(
        &orphan.nodes[0].node_id,
        &Uuid::from_u128(99).to_string(),
        vec![statement("x", "urn:p")],
    ));
    assert!(matches!(
        Graph::try_from_snapshot(&orphan),
        Err(ResourceSnapshotError::MissingSurface { .. })
    ));

    // Schema zero intentionally keeps the historic permissive Surface load.
    let mut legacy_orphan = orphan;
    legacy_orphan.assertion_schema = 0;
    assert!(Graph::try_from_snapshot(&legacy_orphan).is_ok());
}

#[test]
fn checked_load_refuses_an_unparseable_or_repeated_surface_id() {
    for schema in [0, 1] {
        let mut unparseable = base_snapshot();
        unparseable.assertion_schema = schema;
        unparseable.nodes[1].node_id = "not-a-uuid".into();
        let mut repeated = base_snapshot();
        repeated.assertion_schema = schema;
        repeated.nodes[1].node_id = repeated.nodes[0].node_id.clone();
        let first = Uuid::parse_str(&repeated.nodes[0].node_id).unwrap();

        for load in [Graph::try_from_snapshot, Graph::try_from_recorded_snapshot] {
            assert_eq!(
                load(&unparseable).err(),
                Some(ResourceSnapshotError::InvalidSurfaceId {
                    index: 1,
                    value: "not-a-uuid".into()
                })
            );
            assert_eq!(
                load(&repeated).err(),
                Some(ResourceSnapshotError::RepeatedSurfaceId {
                    surface_id: first,
                    first_index: 0,
                    index: 1
                })
            );
        }
    }
}

#[test]
fn canonical_surface_assertion_handles_are_global_and_identical_copies_are_refused() {
    let mut snapshot = base_snapshot();
    snapshot.assertion_schema = 1;
    let from = snapshot.nodes[0].node_id.clone();
    let to = snapshot.nodes[1].node_id.clone();
    let same = statement("stable-id", "urn:p");
    snapshot
        .edges
        .push(canonical_edge(&from, &to, vec![same.clone()]));
    assert!(Graph::try_from_snapshot(&snapshot).is_ok());

    let mut identical = snapshot.clone();
    identical
        .edges
        .push(canonical_edge(&from, &to, vec![same.clone()]));
    let mut reversed = snapshot.clone();
    reversed
        .edges
        .push(canonical_edge(&to, &from, vec![same.clone()]));
    let mut changed = snapshot;
    let mut claim = same;
    claim.predicate = "urn:other".into();
    changed.edges.push(canonical_edge(&from, &to, vec![claim]));
    for repeated in [identical, reversed, changed] {
        assert!(matches!(
            Graph::try_from_snapshot(&repeated),
            Err(ResourceSnapshotError::RepeatedAssertionHandle(_))
        ));
    }
}
