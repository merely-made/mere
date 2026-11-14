// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Replay admits through the handle index and refuses what the live path and
//! the whole-graph rule refuse, warm or cold, with a fresh handle admitted.
use super::apply::{GraphDelta, GraphDeltaResult, apply_graph_delta};
use super::node_facets::SEMANTIC_PROPERTIES;
use super::resource_content::{ContentError, LiteralWrite, PendingWrite, RESOURCE_PROPERTIES};
use super::snapshot::persisted_edge_for_ids;
use super::*;
use crate::persistence::{PersistedEdge, PersistedResourceFacet, PersistedResourceRecord};
use crate::types::{GraphScope, NodeProperty};
use euclid::default::Point2D;

fn property(id: &str, value: &str) -> NodeProperty {
    NodeProperty {
        statement_id: id.into(),
        predicate: "https://schema.org/position".into(),
        value: value.into(),
        datatype: None,
        lang: None,
        graph_scope: GraphScope::User,
        provenance_iri: Some("urn:author:a".into()),
        asserted_at_ms: Some(0),
    }
}

fn row(from: Uuid, to: Uuid, handle: &str) -> PersistedEdge {
    let mut payload = EdgePayload::new();
    payload.push_persisted_semantic_statement(SemanticStatement {
        statement_id: handle.into(),
        predicate: "urn:replay:predicate".into(),
        recognized_sub_kind: None,
        label: None,
        graph_scope: GraphScope::User,
        provenance_iri: Some("urn:replay:author".into()),
        asserted_at_ms: Some(1),
        first_asserted_ms: Some(1),
        withdrawn: None,
        basis: Default::default(),
    });
    persisted_edge_for_ids(from, to, &payload)
}

struct Live {
    graph: Graph,
    nodes: [Uuid; 4],
    resources: [Uuid; 4],
    /// The handle the live write put on the Resource relation a -> b.
    relation: String,
}

/// A graph written through the live path: a relation a -> b, a Resource
/// literal on a, a retained Surface literal on d, and the live refusals.
fn live() -> Live {
    let mut graph = Graph::new();
    let keys = ["a", "b", "c", "d"].map(|name| {
        let key = graph.add_node(format!("https://replay.test/{name}"), Point2D::zero());
        graph.refresh_surface_resource(key);
        key
    });
    let spec = SemanticStatementSpec {
        predicate: "https://replay.test/vocab#next".into(),
        ..Default::default()
    };
    let (key, _) = graph
        .assert_semantic_statement(keys[0], keys[1], spec)
        .unwrap();
    let relation = graph.get_relation(key).unwrap().assertions()[0]
        .statement_id
        .clone();
    assert!(matches!(key, RelationKey::Resource(_)));
    assert!(graph.append_node_properties(keys[0], vec![property("literal", "1")]));
    assert!(graph.set_node_facet(keys[3], SEMANTIC_PROPERTIES, &vec![property("legacy", "1")]));

    // The live path refuses each held handle on another holder.
    assert!(!graph.append_node_properties(keys[2], vec![property("literal", "1")]));
    assert!(!graph.append_node_properties(keys[2], vec![property("legacy", "1")]));
    assert!(!graph.append_node_properties(keys[2], vec![property(&relation, "1")]));
    assert!(graph.append_node_properties(keys[2], vec![property("fresh", "1")]));
    assert!(
        graph
            .retract_resource_property(graph.shown_resource_id(keys[2]).unwrap(), "fresh")
            .unwrap()
    );

    let nodes = keys.map(|key| graph.get_node(key).unwrap().id);
    let resources = keys.map(|key| graph.shown_resource_id(key).unwrap());
    Live {
        graph,
        nodes,
        resources,
        relation,
    }
}

fn record(graph: &Graph, id: Uuid, properties: &[NodeProperty]) -> PersistedResourceRecord {
    let mut record = graph.resource_record(id).unwrap();
    record
        .facets
        .retain(|facet| facet.facet != RESOURCE_PROPERTIES);
    record.facets.push(PersistedResourceFacet {
        facet: RESOURCE_PROPERTIES.into(),
        value_json: serde_json::to_string(properties).unwrap(),
    });
    record
}

/// The write a replayed delta checks, as the whole-graph rule reads it.
enum Write {
    Literals(Uuid, Vec<NodeProperty>),
    Surface(Uuid, Vec<NodeProperty>),
    ResourceEdges(Uuid, Uuid, Vec<String>),
    SurfaceEdges(Uuid, Uuid, Vec<String>),
}

impl Write {
    fn pending(&self) -> PendingWrite<'_> {
        match self {
            Write::Literals(id, p) => PendingWrite::Literals(LiteralWrite::Resource(*id, p)),
            Write::Surface(id, p) => PendingWrite::Literals(LiteralWrite::Surface(*id, p)),
            Write::ResourceEdges(a, b, rows) => PendingWrite::ResourceEdges(*a, *b, rows),
            Write::SurfaceEdges(a, b, rows) => PendingWrite::SurfaceEdges(*a, *b, rows),
        }
    }

    fn delta(&self, graph: &Graph) -> GraphDelta {
        let rows = |a, b, handles: &[String]| handles.iter().map(|h| row(a, b, h)).collect();
        match self {
            Write::Literals(id, p) => GraphDelta::ReplaySetResourceRecordById {
                resource_id: *id,
                record: Some(record(graph, *id, p)),
            },
            Write::Surface(id, p) => GraphDelta::ReplaySetNodeFacetById {
                node_id: *id,
                facet: SEMANTIC_PROPERTIES.into(),
                value: serde_json::to_value(p).unwrap(),
            },
            Write::ResourceEdges(a, b, handles) => GraphDelta::ReplaySetResourceEdgesByIds {
                from_resource_id: *a,
                to_resource_id: *b,
                edges: rows(*a, *b, handles),
            },
            Write::SurfaceEdges(a, b, handles) => GraphDelta::ReplaySetEdgesByIds {
                from_id: *a,
                to_id: *b,
                edges: rows(*a, *b, handles),
            },
        }
    }
}

fn state(graph: &Graph) -> serde_json::Value {
    serde_json::to_value((graph.to_snapshot_at(0), graph.facets())).unwrap()
}

/// Apply `write` as a replayed delta to a warm and a cold copy of `graph`:
/// both answer as the whole-graph rule does, and a refusal changes nothing.
fn replayed(graph: &Graph, write: &Write) -> Result<(), ContentError> {
    let rule = graph.validate_handles_after(Some(write.pending()));
    let mut cold = graph.clone();
    cold.clear_handle_index();
    for mut copy in [graph.clone(), cold] {
        assert_eq!(copy.write_collides(write.pending()), rule.is_err());
        let before = state(&copy);
        match apply_graph_delta(&mut copy, write.delta(graph)) {
            GraphDeltaResult::Refused(refused) => {
                assert_eq!(Err(refused), rule);
                assert_eq!(state(&copy), before, "a refusal stores nothing");
            },
            _ => {
                assert_eq!(rule, Ok(()));
                assert!(copy.validate_active_resource_assertion_handles().is_ok());
            },
        }
    }
    rule
}

#[test]
fn replay_refuses_the_handles_the_live_path_refuses() {
    let Live {
        graph,
        nodes: [na, nb, nc, nd],
        resources: [ra, rb, rc, _],
        relation,
    } = live();
    // Warm the index, as a session's live graph is.
    let _ = graph.find_semantic_statement(&relation);
    let refused = |id: &str| Err(ContentError::HandleCollision(id.into()));
    let held = [relation.as_str(), "literal", "legacy"];

    // A Resource literal may not take a relation's, another Resource's or a
    // Surface literal's handle; a fresh handle is admitted.
    for id in held {
        let write = Write::Literals(rc, vec![property(id, "2")]);
        assert_eq!(replayed(&graph, &write), refused(id), "{id}");
    }
    assert_eq!(
        replayed(&graph, &Write::Literals(rc, vec![property("fresh", "2")])),
        Ok(())
    );
    // Its own literal restated or edited is no collision; two contents under
    // one handle in one write are.
    assert_eq!(
        replayed(&graph, &Write::Literals(ra, vec![property("literal", "9")])),
        Ok(())
    );
    let twice = vec![property("twin", "1"), property("twin", "2")];
    assert_eq!(
        replayed(&graph, &Write::Literals(rc, twice)),
        refused("twin")
    );

    // A Resource row may not take another pair's, a literal's or a Surface
    // literal's handle, nor sit twice in one set.
    for id in held {
        let write = Write::ResourceEdges(rb, rc, vec![id.into()]);
        assert_eq!(replayed(&graph, &write), refused(id), "{id}");
    }
    let dup = Write::ResourceEdges(rb, rc, vec!["dup".into(), "dup".into()]);
    assert_eq!(replayed(&graph, &dup), refused("dup"));
    let fresh = Write::ResourceEdges(rb, rc, vec!["fresh-row".into()]);
    assert_eq!(replayed(&graph, &fresh), Ok(()));
    // The pair that holds a handle may restate it beside a new row.
    let grown = Write::ResourceEdges(ra, rb, vec![relation.clone(), "grown".into()]);
    assert_eq!(replayed(&graph, &grown), Ok(()));

    // A Surface row may not take a Resource holder's handle; a retained
    // Surface literal keeps its historical grammar and does not refuse it.
    for id in [relation.as_str(), "literal"] {
        let write = Write::SurfaceEdges(nb, nc, vec![id.into()]);
        assert_eq!(replayed(&graph, &write), refused(id), "{id}");
    }
    let legacy = Write::SurfaceEdges(nb, nc, vec!["legacy".into()]);
    assert_eq!(replayed(&graph, &legacy), Ok(()));

    // A retained Surface literal may not take a Resource holder's handle.
    for id in [relation.as_str(), "literal"] {
        let write = Write::Surface(nd, vec![property(id, "1")]);
        assert_eq!(replayed(&graph, &write), refused(id), "{id}");
    }
    assert_eq!(
        replayed(&graph, &Write::Surface(na, vec![property("legacy", "1")])),
        Ok(())
    );
}

#[test]
fn replay_admission_reads_handed_out_relations_and_stale_holders_live() {
    let Live {
        mut graph,
        resources: [ra, rb, rc, _],
        relation,
        ..
    } = live();
    let _ = graph.find_semantic_statement(&relation);
    let key = graph.find_semantic_statement(&relation).unwrap().0;
    // A payload handed out mutably and renamed: the index still names the old
    // handle, and the renamed one is read live from the handout.
    graph.get_relation_mut(key).unwrap().assertions.statements[0].statement_id = "renamed".into();
    let renamed = Write::ResourceEdges(rb, rc, vec!["renamed".into()]);
    assert!(replayed(&graph, &renamed).is_err());
    assert_eq!(
        replayed(&graph, &Write::ResourceEdges(rb, rc, vec![relation])),
        Ok(())
    );
    // A retracted literal leaves a stale holder; its handle is free again.
    assert!(graph.retract_resource_property(ra, "literal").unwrap());
    assert_eq!(
        replayed(&graph, &Write::Literals(rc, vec![property("literal", "2")])),
        Ok(())
    );
}

#[test]
fn a_journal_replays_to_the_live_graph_and_stops_at_a_colliding_entry() {
    let Live {
        mut graph,
        nodes: [na, ..],
        resources: [_, rb, rc, _],
        ..
    } = live();
    let journal = std::sync::Arc::new(std::sync::Mutex::new(journal::GraphJournal::new()));
    let sink = journal.clone();
    graph.set_recorder(Some(std::sync::Arc::new(move |delta: &CapturedDelta| {
        sink.lock()
            .unwrap()
            .record_as(Author::user(), delta.clone());
    })));
    let baseline = graph.clone();
    let a = graph.get_node_key_by_id(na).unwrap();
    assert!(graph.write_as(Author::user(), |graph| {
        graph.append_node_properties(a, vec![property("later", "later")])
    }));
    let mut journal = std::mem::take(&mut *journal.lock().unwrap());
    let live = journal.live_cursor();
    let replayed = journal.snapshot_at_from(&baseline, live).unwrap().unwrap();
    assert_eq!(state(&replayed), state(&graph));

    // The control: a fresh row replays. Then a row taking the new literal's
    // handle is refused at its position, as the live path refuses it.
    journal.record_as(
        Author::user(),
        CapturedDelta::ReplaySetResourceEdgesByIds {
            from_resource_id: rb.to_string(),
            to_resource_id: rc.to_string(),
            edges: vec![row(rb, rc, "fresh-row")],
        },
    );
    assert!(
        journal
            .snapshot_at_from(&baseline, journal.live_cursor())
            .is_ok()
    );
    journal.record_as(
        Author::user(),
        CapturedDelta::ReplaySetResourceEdgesByIds {
            from_resource_id: rc.to_string(),
            to_resource_id: rb.to_string(),
            edges: vec![row(rc, rb, "later")],
        },
    );
    let Err(refused) = journal.snapshot_at_from(&baseline, journal.live_cursor()) else {
        panic!("the colliding row is refused on replay");
    };
    assert_eq!(refused.position, journal.live_cursor().index() - 1);
    assert!(refused.to_string().contains("later"), "{refused}");
}

/// Merge (D156, F3): a codicil import's edits pass the same admission, so an
/// import whose relation takes a held handle is refused at that edit and the
/// edit stores nothing, while a fresh handle imports.
#[test]
fn an_import_taking_a_held_handle_is_refused_at_its_edit() {
    let Live {
        graph: live,
        nodes: [na, nb, ..],
        relation,
        ..
    } = live();
    let incoming = |handle: &str| {
        let mut incoming = Graph::new();
        for (id, name) in [(na, "a"), (nb, "b")] {
            incoming.add_node_with_id(id, format!("https://replay.test/{name}"), Point2D::zero());
        }
        let edges = vec![row(na, nb, handle)];
        let result = apply_graph_delta(
            &mut incoming,
            GraphDelta::ReplaySetEdgesByIds {
                from_id: na,
                to_id: nb,
                edges,
            },
        );
        assert!(matches!(result, GraphDeltaResult::Applied));
        incoming
    };
    let import = |handle: &str| {
        let edits = super::merge::import_edits(&live, &incoming(handle));
        let at = edits
            .iter()
            .position(|edit| matches!(edit, CapturedDelta::ReplaySetEdgesByIds { .. }))
            .expect("the import writes the relation");
        (edits, at)
    };

    let (edits, at) = import(&relation);
    let mut before = live.clone();
    replay_captured_deltas_onto_checked(&mut before, edits[..at].iter().cloned()).unwrap();
    let mut merged = live.clone();
    let Err(refused) = replay_captured_deltas_onto_checked(&mut merged, edits) else {
        panic!("an import taking a held handle is refused");
    };
    assert_eq!(refused.position, at);
    assert!(refused.to_string().contains(&relation), "{refused}");
    assert_eq!(
        state(&merged),
        state(&before),
        "the refused edit stores nothing"
    );

    // The control: the same import with a fresh handle is admitted whole.
    let (edits, _) = import("fresh-import");
    let mut merged = live.clone();
    replay_captured_deltas_onto_checked(&mut merged, edits).unwrap();
    assert!(merged.find_semantic_statement("fresh-import").is_some());
}
