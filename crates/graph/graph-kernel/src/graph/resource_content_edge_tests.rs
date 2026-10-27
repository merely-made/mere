// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Replayed edge sets meet the rule checked open applies; a legacy migration stages it.
use super::apply::{GraphDelta, GraphDeltaResult, add_node, apply_graph_delta};
use super::legacy_resource_migration::migrate_legacy_prefix;
use super::node_facets::SEMANTIC_PROPERTIES;
use super::resource_content::{ContentError, RESOURCE_PROPERTIES};
use super::snapshot::persisted_edge_for_ids;
use super::*;
use crate::persistence::{PersistedEdge, PersistedResourceFacet, PersistedResourceRecord};
use crate::types::{GraphScope, NodeProperty};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

type Seen = Arc<Mutex<Vec<CapturedDelta>>>;

fn recording(graph: &mut Graph) -> Seen {
    let seen = Seen::default();
    let sink = seen.clone();
    graph.set_recorder(Some(Arc::new(move |delta| {
        sink.lock().unwrap().push(delta.clone())
    })));
    seen
}

fn literal(handle: &str) -> NodeProperty {
    let mut property = NodeProperty::new("urn:edge:predicate".into(), "value".into());
    property.statement_id = handle.into();
    property
}

fn record(iri: &str, properties: &[NodeProperty]) -> PersistedResourceRecord {
    PersistedResourceRecord {
        canonical_iri: iri.into(),
        facets: vec![PersistedResourceFacet {
            facet: RESOURCE_PROPERTIES.into(),
            value_json: serde_json::to_string(properties).unwrap(),
        }],
    }
}

fn resource(iri: &str) -> Uuid {
    chartulary::resource_id_from_canonical_iri(iri)
}

fn set_record(graph: &mut Graph, iri: &str, properties: &[NodeProperty]) {
    apply_graph_delta(
        graph,
        GraphDelta::ReplaySetResourceRecordById {
            resource_id: resource(iri),
            record: Some(record(iri, properties)),
        },
    );
}

/// One row from `from` to `to` carrying the statement `handle`.
fn row(from: Uuid, to: Uuid, handle: &str) -> PersistedEdge {
    claim(from, to, handle, "urn:edge:predicate", None)
}

fn claim(
    from: Uuid,
    to: Uuid,
    handle: &str,
    predicate: &str,
    recognized_sub_kind: Option<SemanticSubKind>,
) -> PersistedEdge {
    let mut payload = EdgePayload::new();
    payload.push_persisted_semantic_statement(SemanticStatement {
        statement_id: handle.into(),
        predicate: predicate.into(),
        recognized_sub_kind,
        label: None,
        graph_scope: GraphScope::User,
        provenance_iri: Some("urn:edge:author".into()),
        asserted_at_ms: Some(1),
        first_asserted_ms: Some(1),
        withdrawn: None,
        basis: Default::default(),
    });
    persisted_edge_for_ids(from, to, &payload)
}

fn replace_resource_edges(
    graph: &mut Graph,
    from: Uuid,
    to: Uuid,
    handles: &[&str],
) -> GraphDeltaResult {
    let edges = handles.iter().map(|h| row(from, to, h)).collect();
    apply_graph_delta(
        graph,
        GraphDelta::ReplaySetResourceEdgesByIds {
            from_resource_id: from,
            to_resource_id: to,
            edges,
        },
    )
}

fn replace_surface_edges(
    graph: &mut Graph,
    from: Uuid,
    to: Uuid,
    handles: &[&str],
) -> GraphDeltaResult {
    let edges = handles.iter().map(|h| row(from, to, h)).collect();
    apply_graph_delta(
        graph,
        GraphDelta::ReplaySetEdgesByIds {
            from_id: from,
            to_id: to,
            edges,
        },
    )
}

fn reopens(graph: &Graph) -> bool {
    graph.validate_active_resource_assertion_handles().is_ok()
        && Graph::try_from_recorded_snapshot(&graph.to_snapshot()).is_ok()
}

fn refused_for(result: &GraphDeltaResult, handle: &str) -> bool {
    matches!(result, GraphDeltaResult::Refused(ContentError::HandleCollision(held)) if held == handle)
}

fn state(graph: &Graph) -> serde_json::Value {
    serde_json::to_value(graph.to_snapshot()).unwrap()
}

#[test]
fn replayed_resource_edges_are_refused_where_checked_open_refuses_them() {
    let mut graph = Graph::new();
    let seen = recording(&mut graph);
    let (a, b, c) = (
        resource("urn:edge:a"),
        resource("urn:edge:b"),
        resource("urn:edge:c"),
    );
    set_record(&mut graph, "urn:edge:a", &[literal("owned-handle")]);
    set_record(&mut graph, "urn:edge:b", &[]);
    set_record(&mut graph, "urn:edge:c", &[]);
    let surface = Uuid::from_u128(7);
    add_node(
        &mut graph,
        Some(surface),
        "https://surface.test/".into(),
        Default::default(),
    );
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplaySetNodeFacetById {
            node_id: surface,
            facet: SEMANTIC_PROPERTIES.into(),
            value: serde_json::to_value(vec![literal("surface-handle")]).unwrap(),
        },
    );
    let (before, recorded) = (state(&graph), seen.lock().unwrap().len());

    // A Resource literal's handle, a Surface literal's handle, and a handle on two
    // rows of the replayed set are refused, changing and recording nothing.
    for (handles, owned) in [
        (&["owned-handle"][..], "owned-handle"),
        (&["surface-handle"], "surface-handle"),
        (&["twice", "twice"], "twice"),
    ] {
        let result = replace_resource_edges(&mut graph, b, c, handles);
        assert!(refused_for(&result, owned), "{owned}");
        assert_eq!(state(&graph), before);
        assert_eq!(seen.lock().unwrap().len(), recorded);
        assert!(reopens(&graph));
    }

    // A handle another pair's row holds is refused, even as an identical copy.
    assert!(matches!(
        replace_resource_edges(&mut graph, b, c, &["held-edge"]),
        GraphDeltaResult::Applied
    ));
    let (held, recorded) = (state(&graph), seen.lock().unwrap().len());
    let result = replace_resource_edges(&mut graph, a, c, &["held-edge"]);
    assert!(refused_for(&result, "held-edge"));
    assert_eq!(state(&graph), held);
    assert_eq!(seen.lock().unwrap().len(), recorded);
    assert!(reopens(&graph));

    // A distinct handle, a restated set and a grown set are stored.
    for handles in [
        &["distinct-handle"][..],
        &["distinct-handle"],
        &["distinct-handle", "grown"],
    ] {
        let result = replace_resource_edges(&mut graph, a, c, handles);
        assert!(matches!(result, GraphDeltaResult::Applied), "{handles:?}");
        assert!(reopens(&graph));
    }
    assert_eq!(graph.persisted_resource_edges_between(a, c).len(), 2);
    assert_eq!(seen.lock().unwrap().len(), recorded + 2);
}

#[test]
fn replayed_surface_edges_are_refused_where_checked_open_refuses_them() {
    let mut graph = Graph::new();
    let seen = recording(&mut graph);
    let node = |graph: &mut Graph, id: u128, url: &str| {
        add_node(
            graph,
            Some(Uuid::from_u128(id)),
            url.into(),
            Default::default(),
        );
        Uuid::from_u128(id)
    };
    let (s, t, u) = (
        node(&mut graph, 11, "https://s.test/"),
        node(&mut graph, 12, "https://t.test/"),
        node(&mut graph, 13, "https://u.test/"),
    );
    let (b, c) = (resource("urn:edge:b"), resource("urn:edge:c"));
    set_record(&mut graph, "urn:edge:a", &[literal("owned-handle")]);
    set_record(&mut graph, "urn:edge:b", &[]);
    set_record(&mut graph, "urn:edge:c", &[]);
    replace_resource_edges(&mut graph, b, c, &["held-edge"]);
    let (before, recorded) = (state(&graph), seen.lock().unwrap().len());

    // A Resource literal's handle, a Resource row's handle (an identical copy
    // included), and a handle on two rows of the replayed set are refused.
    for (handles, owned) in [
        (&["owned-handle"][..], "owned-handle"),
        (&["held-edge"], "held-edge"),
        (&["twice", "twice"], "twice"),
    ] {
        let result = replace_surface_edges(&mut graph, s, t, handles);
        assert!(refused_for(&result, owned), "{owned}");
        assert_eq!(state(&graph), before);
        assert_eq!(seen.lock().unwrap().len(), recorded);
        assert!(reopens(&graph));
    }

    // A handle another Surface pair holds is refused; its own pair may restate it.
    for _ in 0..2 {
        let result = replace_surface_edges(&mut graph, s, t, &["surface-edge"]);
        assert!(matches!(result, GraphDeltaResult::Applied));
    }
    let (held, recorded) = (state(&graph), seen.lock().unwrap().len());
    let result = replace_surface_edges(&mut graph, t, u, &["surface-edge"]);
    assert!(refused_for(&result, "surface-edge"));
    assert_eq!(state(&graph), held);
    assert_eq!(seen.lock().unwrap().len(), recorded);
    assert!(reopens(&graph));

    // A distinct handle, a restated set and a grown set are stored.
    for handles in [
        &["distinct-handle"][..],
        &["distinct-handle"],
        &["distinct-handle", "grown"],
    ] {
        let result = replace_surface_edges(&mut graph, t, u, handles);
        assert!(matches!(result, GraphDeltaResult::Applied), "{handles:?}");
        assert!(reopens(&graph));
    }
    assert_eq!(
        graph
            .persisted_edges_between(
                graph.get_node_key_by_id(t).unwrap(),
                graph.get_node_key_by_id(u).unwrap()
            )
            .len(),
        2
    );
}

/// A refused edge set is reported where the journal and the graph disagree,
/// with the graph holding only the entries before it.
#[test]
fn checked_replay_reports_a_refused_edge_set_instead_of_dropping_it() {
    let (b, c) = (resource("urn:edge:b"), resource("urn:edge:c"));
    let mut graph = Graph::new();
    set_record(&mut graph, "urn:edge:a", &[literal("shared replay handle")]);
    set_record(&mut graph, "urn:edge:b", &[]);
    set_record(&mut graph, "urn:edge:c", &[]);
    let before = state(&graph);
    let colliding = CapturedDelta::ReplaySetResourceEdgesByIds {
        from_resource_id: b.to_string(),
        to_resource_id: c.to_string(),
        edges: vec![row(b, c, "shared replay handle")],
    };
    let refused = replay_captured_deltas_onto_checked(&mut graph, [colliding.clone()])
        .expect_err("a colliding edge handle is refused on replay");
    assert_eq!(refused.position, 0);
    assert!(
        refused.to_string().contains("shared replay handle"),
        "{refused}"
    );
    assert_eq!(state(&graph), before);
    assert!(reopens(&graph));
    let mut journal = journal::GraphJournal::default();
    journal.record_as(Author::person("test"), colliding);
    assert!(journal.snapshot_at_from(&graph, Seq(1)).is_err());
}

/// The journal a pre-invariant graph could write: an edge claims a handle a
/// literal still holds, and a later entry drops the literal.
fn legacy_journal(drop_literal: bool) -> GraphJournal {
    let (b, c) = (resource("urn:edge:b"), resource("urn:edge:c"));
    let mut journal = GraphJournal::new();
    let mut entry = |delta| journal.record_as(Author::user(), delta);
    for (iri, properties) in [
        ("urn:edge:b", vec![literal("staged-handle")]),
        ("urn:edge:c", vec![]),
    ] {
        entry(CapturedDelta::ReplaySetResourceRecordById {
            resource_id: resource(iri).to_string(),
            record: Some(record(iri, &properties)),
        });
    }
    entry(CapturedDelta::ReplaySetResourceEdgesByIds {
        from_resource_id: b.to_string(),
        to_resource_id: c.to_string(),
        edges: vec![row(b, c, "staged-handle")],
    });
    if drop_literal {
        entry(CapturedDelta::ReplaySetResourceRecordById {
            resource_id: b.to_string(),
            record: Some(record("urn:edge:b", &[])),
        });
    }
    journal
}

#[test]
fn legacy_migration_stages_edge_admission_and_refuses_a_result_that_would_not_reopen() {
    let (b, c) = (resource("urn:edge:b"), resource("urn:edge:c"));
    let finishing = legacy_journal(true);
    let stuck = legacy_journal(false);

    // Ordinary replay refuses the moment both strata hold the handle.
    for (journal, position) in [(&finishing, 2), (&stuck, 2)] {
        let deltas = journal.entries().iter().map(|entry| entry.delta.clone());
        let refused = replay_captured_deltas_onto_checked(&mut Graph::new(), deltas)
            .expect_err("ordinary replay checks every edge set");
        assert_eq!(refused.position, position);
    }

    // The migration stages the check: the handle may sit on both until the end.
    let migrated = migrate_legacy_prefix(&Graph::new(), finishing.entries())
        .expect("a transient collision completes through the staged state");
    assert_eq!(
        migrated.graph.persisted_resource_edges_between(b, c).len(),
        1
    );
    assert!(migrated.graph.resource_properties(b).is_empty());
    assert!(reopens(&migrated.graph));

    // The finished graph is checked once and the migration refuses what would not reopen.
    let error = migrate_legacy_prefix(&Graph::new(), stuck.entries())
        .err()
        .expect("a final collision is refused");
    assert!(error.0.contains("staged-handle"), "{error}");
    assert!(!error.0.contains("replacement was refused"), "{error}");
}

/// Legacy journals repeat rows; the migration reads them as written.
#[test]
fn legacy_migration_reads_its_source_through_the_staged_state_repeated_rows_included() {
    let (a, b) = (Uuid::from_u128(1), Uuid::from_u128(2));
    let mut baseline = Graph::new();
    for (id, url) in [(a, "https://a.test/"), (b, "https://b.test/")] {
        add_node(&mut baseline, Some(id), url.into(), Default::default());
    }
    let cites = SemanticSubKind::Cites;
    let copy = claim(a, b, "legacy-copy", predicate_iri(cites), Some(cites));
    let mut journal = GraphJournal::new();
    journal.record_as(
        Author::user(),
        CapturedDelta::ReplaySetEdgesByIds {
            from_id: a.to_string(),
            to_id: b.to_string(),
            edges: vec![copy.clone(), copy],
        },
    );

    let deltas = journal.entries().iter().map(|entry| entry.delta.clone());
    let refused = replay_captured_deltas_onto_checked(&mut baseline.clone(), deltas)
        .expect_err("ordinary replay keeps a handle to one row");
    assert!(refused.to_string().contains("legacy-copy"), "{refused}");

    let migrated = migrate_legacy_prefix(&baseline, journal.entries()).unwrap();
    let pair = (
        ResourceNode::new("https://a.test/").id(),
        ResourceNode::new("https://b.test/").id(),
    );
    let held = migrated
        .graph
        .persisted_resource_edges_between(pair.0, pair.1);
    assert!(!held.is_empty(), "the legacy claim survives the migration");
    assert!(held.iter().all(|edge| {
        edge.retained_assertions()
            .iter()
            .all(|statement| statement.statement_id == "legacy-copy")
    }));
    assert!(reopens(&migrated.graph));
}
