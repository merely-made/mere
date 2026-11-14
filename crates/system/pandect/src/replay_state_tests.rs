// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The byte check agrees with the `Value` check it replaced, on every case
//! where the two could part: non-finite floats, a zero's sign, skipped
//! fields and map order; and the entry check of engine split D164 agrees
//! with the whole-graph bytes on the same cases.
use super::{replay_bytes, state_bytes};
use kernel::graph::Graph;
use kernel::graph::apply::add_node;
use kernel::persistence::PersistedNodeSessionState;
use kernel::types::{BadgeIcon, NodeTagPresentationState};
use portolan::{AcceptAll, FacetId, FacetStore};
use serde::Serialize;
use serde_json::{Value, json};
use uuid::Uuid;

/// Whether `a` and `b` compare equal, asserting the byte check and the old
/// `Value` check give the same answer.
fn agree(a: &impl Serialize, b: &impl Serialize) -> bool {
    let bytes = state_bytes(a).unwrap() == state_bytes(b).unwrap();
    let values = serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap();
    assert_eq!(bytes, values, "the byte and Value checks part");
    bytes
}

fn scroll(x: Option<f32>) -> PersistedNodeSessionState {
    PersistedNodeSessionState {
        scroll_x: x,
        scroll_y: None,
        form_draft: None,
        last_visited_ms: None,
    }
}

#[test]
fn non_finite_floats_read_as_null_and_a_zero_has_no_sign() {
    for (a, b) in [
        (Some(f32::NAN), None),
        (Some(f32::INFINITY), None),
        (Some(f32::NEG_INFINITY), Some(f32::NAN)),
        (Some(-0.0), Some(0.0)),
    ] {
        assert!(agree(&scroll(a), &scroll(b)), "{a:?} {b:?}");
    }
    assert!(agree(&(-0.0f64,), &(0.0f64,)));
    assert!(agree(&json!({ "z": -0.0 }), &json!({ "z": 0.0 })));
    // Controls: distinct finite values, and a number against null, differ.
    assert!(!agree(&scroll(Some(1.5)), &scroll(Some(2.5))));
    assert!(!agree(&scroll(Some(0.0)), &scroll(None)));
    assert!(!agree(&json!({ "z": 1 }), &json!({ "z": 1.0 })));
}

#[test]
fn skipped_fields_are_unseen_by_both() {
    let mut graph = Graph::new();
    add_node(
        &mut graph,
        Some(Uuid::from_u128(1)),
        "https://bytes.test/".into(),
        Default::default(),
    );
    let node = graph.to_snapshot_at(0).nodes[0].clone();
    let mut legacy = node.clone();
    legacy.legacy_thumbnail_png = Some(vec![1, 2, 3]);
    legacy.legacy_thumbnail_width = 7;
    legacy.legacy_thumbnail_height = 7;
    legacy.legacy_favicon_rgba = Some(vec![4]);
    legacy.legacy_favicon_width = 1;
    legacy.legacy_favicon_height = 1;
    assert!(agree(&node, &legacy));
    let mut retitled = node.clone();
    retitled.title = "other".into();
    assert!(!agree(&node, &retitled), "control: a written field differs");
}

#[test]
fn maps_compare_by_content_not_insertion() {
    // preserve_order is off: a Value object iterates sorted, not as inserted.
    let mut map = serde_json::Map::new();
    map.insert("b".into(), Value::Null);
    map.insert("a".into(), Value::Null);
    assert_eq!(map.keys().next().map(String::as_str), Some("a"));

    let (one, two) = (Uuid::from_u128(1), Uuid::from_u128(2));
    let facet = |name: &str| FacetId::new(name);
    let mut forward = FacetStore::<Uuid>::default();
    let mut backward = FacetStore::<Uuid>::default();
    for (store, order) in [(&mut forward, [one, two]), (&mut backward, [two, one])] {
        for id in order {
            for name in if id == one { ["x", "y"] } else { ["y", "x"] } {
                store
                    .set(id, facet(name), json!({ "b": 1, "a": [name] }), &AcceptAll)
                    .unwrap();
            }
        }
    }
    assert!(agree(&forward, &backward));

    // A HashMap-backed state is held as the Value its facet writer makes.
    let presentation = |tags: [&str; 3]| {
        let mut state = NodeTagPresentationState::default();
        for tag in tags {
            state
                .icon_overrides
                .insert(tag.into(), BadgeIcon::Lucide(tag.into()));
        }
        serde_json::to_value(state).unwrap()
    };
    assert!(agree(
        &presentation(["a", "b", "c"]),
        &presentation(["c", "a", "b"])
    ));
    assert!(!agree(
        &presentation(["a", "b", "c"]),
        &presentation(["a", "b", "d"])
    ));
}

#[test]
fn a_graph_writes_the_same_bytes_each_time_and_differs_when_it_does() {
    let mut graph = Graph::new();
    for i in 1..=3 {
        add_node(
            &mut graph,
            Some(Uuid::from_u128(i)),
            format!("https://bytes.test/{i}"),
            Default::default(),
        );
    }
    let old =
        |graph: &Graph| serde_json::to_value((graph.to_snapshot_at(0), graph.facets())).unwrap();
    let copy = graph.clone();
    assert_eq!(replay_bytes(&graph).unwrap(), replay_bytes(&graph).unwrap());
    assert_eq!(replay_bytes(&graph).unwrap(), replay_bytes(&copy).unwrap());
    assert_eq!(old(&graph), old(&copy));
    add_node(
        &mut graph,
        Some(Uuid::from_u128(4)),
        "https://bytes.test/4".into(),
        Default::default(),
    );
    assert_ne!(replay_bytes(&graph).unwrap(), replay_bytes(&copy).unwrap());
    assert_ne!(old(&graph), old(&copy));
}

// ---- The entry check (engine split D164) against the whole-graph bytes ----

use super::{Decided, replay_equal};
use kernel::graph::NodeKey;
use kernel::graph::fixtures::GraphFixtures;
use kernel::graph::snapshot::unshared::StateEntry;
use kernel::graph::{
    ContainmentSubKind, Coupling, CouplingId, CouplingResponse, EdgeAssertion, Field,
    FieldDefinition, FieldExtent, FieldId, NodeSelector, ScalarField, SemanticStatementSpec,
};
use kernel::graph::apply::{GraphDelta, apply_graph_delta};
use kernel::graph::{CapturedDelta, replay_captured_deltas_onto, with_isolated_capture};
use kernel::types::NodeProperty;
use std::sync::{Arc, Mutex};

/// The session edit's two sides: `edit` run on a recording clone of `graph`,
/// and another clone with the captured deltas replayed onto it.
fn sides(graph: &Graph, edit: impl FnOnce(&mut Graph)) -> (Graph, Graph, Vec<CapturedDelta>) {
    let mut candidate = graph.clone();
    let captured: Arc<Mutex<Vec<CapturedDelta>>> = Arc::default();
    let buffer = Arc::clone(&captured);
    candidate.set_recorder(Some(Arc::new(move |delta: &CapturedDelta| {
        buffer.lock().unwrap().push(delta.clone());
    })));
    with_isolated_capture(|| edit(&mut candidate));
    candidate.set_recorder(None);
    let deltas = std::mem::take(&mut *captured.lock().unwrap());
    let mut replay = graph.clone();
    replay_captured_deltas_onto(&mut replay, deltas.iter().cloned());
    (candidate, replay, deltas)
}

/// The entry check's answer, asserted equal to the whole-graph byte check's.
fn check(a: &Graph, b: &Graph) -> (bool, Decided) {
    let whole = replay_bytes(a).unwrap() == replay_bytes(b).unwrap();
    let (equal, decided) = replay_equal(a, b).unwrap();
    assert_eq!(equal, whole, "the entry check parts from the whole bytes ({decided:?})");
    (equal, decided)
}

/// Answered from the entries, with at least one compared.
fn by_entries(decided: Decided) -> bool {
    matches!(decided, Decided::Entries(n) if n > 0)
}

const ID: u128 = 0xf1e1d;

fn field(extent: FieldExtent) -> Field {
    Field::new(
        FieldId::from_uuid(Uuid::from_u128(ID)),
        FieldDefinition::Scalar(ScalarField::Const(1.0)),
    )
    .with_extent(extent)
}

fn coupling(strength: f32) -> Coupling {
    Coupling::new(
        CouplingId::from_uuid(Uuid::from_u128(ID + 1)),
        FieldId::from_uuid(Uuid::from_u128(ID)),
        NodeSelector::All,
        CouplingResponse::AttractToMin,
        strength,
    )
}

fn statement(predicate: &str) -> SemanticStatementSpec {
    SemanticStatementSpec {
        predicate: predicate.into(),
        ..Default::default()
    }
}

/// A durable Surface edge.
fn contain(graph: &mut Graph, from: NodeKey, to: NodeKey) {
    let folder = EdgeAssertion::Containment {
        sub_kind: ContainmentSubKind::UserFolder,
    };
    graph.assert_relation(from, to, folder).expect("Surface edge");
}

fn position(value: &str) -> Vec<NodeProperty> {
    vec![NodeProperty::new(
        "https://schema.org/position".into(),
        value.into(),
    )]
}

/// Three Surface nodes with a Surface edge, statements held by Resources,
/// a literal property, facets, a field and a coupling.
fn base() -> (Graph, Vec<NodeKey>) {
    let mut graph = Graph::new();
    let keys: Vec<NodeKey> = (1..=3)
        .map(|i| {
            add_node(
                &mut graph,
                Some(Uuid::from_u128(i)),
                format!("https://entries.test/{i}"),
                Default::default(),
            )
        })
        .collect();
    contain(&mut graph, keys[0], keys[1]);
    for (from, to) in [(0, 1), (1, 2)] {
        graph
            .assert_semantic_statement(keys[from], keys[to], statement("https://entries.test/v#next"))
            .expect("statement applies");
    }
    graph.append_node_properties(keys[0], position("1"));
    graph
        .facets_mut()
        .set(Uuid::from_u128(1), FacetId::new("x"), json!({ "a": 1 }), &AcceptAll)
        .unwrap();
    graph.add_field(field(FieldExtent::Global));
    graph.add_coupling(coupling(1.0));
    let snapshot = graph.to_snapshot_at(0);
    assert!(!snapshot.edges.is_empty(), "precondition: a Surface edge");
    assert!(!snapshot.resource_edges.is_empty(), "precondition: Resource edges");
    assert!(!snapshot.resources.is_empty(), "precondition: Resources");
    (graph, keys)
}

#[test]
fn untouched_clones_compare_equal_with_nothing_to_write() {
    let (graph, _) = base();
    let copy = graph.clone();
    assert_eq!(check(&graph, &copy), (true, Decided::Entries(0)));
}

#[test]
fn an_edit_and_its_replay_compare_equal_by_their_entries() {
    let (graph, keys) = base();
    let (candidate, replay, deltas) = sides(&graph, |g| {
        let title = GraphDelta::SetNodeTitle {
            key: keys[1],
            title: "retitled".into(),
        };
        apply_graph_delta(g, title);
        contain(g, keys[1], keys[2]);
        g.assert_semantic_statement(keys[2], keys[0], statement("https://entries.test/v#back"));
        g.append_node_properties(keys[1], position("2"));
        let renamed = field(FieldExtent::Global).with_name("renamed");
        apply_graph_delta(g, GraphDelta::AddField { field: renamed });
        add_node(g, Some(Uuid::from_u128(9)), "https://entries.test/9".into(), Default::default());
    });
    assert!(deltas.len() >= 5, "precondition: the edit recorded its writes");
    let (equal, decided) = check(&candidate, &replay);
    assert!(equal && by_entries(decided), "{decided:?}");
    // Control: the same edit replayed without its last delta differs.
    let mut short = graph.clone();
    replay_captured_deltas_onto(&mut short, deltas[..deltas.len() - 1].iter().cloned());
    assert!(!check(&candidate, &short).0);
}

#[test]
fn non_finite_floats_read_as_null_entry_by_entry() {
    let (graph, _) = base();
    let region = |x: f32| FieldExtent::Region {
        min_x: x,
        min_y: 0.0,
        max_x: 1.0,
        max_y: 1.0,
    };
    let (mut a, mut b) = (graph.clone(), graph.clone());
    a.add_field(field(region(f32::NAN)));
    b.add_field(field(region(f32::INFINITY)));
    let (equal, decided) = check(&a, &b);
    assert!(equal && by_entries(decided), "{decided:?}");
    // Control: two finite values differ, and the walk says so itself.
    a.add_field(field(region(2.0)));
    b.add_field(field(region(3.0)));
    assert_eq!(check(&a, &b), (false, Decided::Walk));
}

#[test]
fn a_zero_has_no_sign_entry_by_entry() {
    let (graph, _) = base();
    let (mut a, mut b) = (graph.clone(), graph.clone());
    a.add_coupling(coupling(-0.0));
    b.add_coupling(coupling(0.0));
    let (equal, decided) = check(&a, &b);
    assert!(equal && by_entries(decided), "{decided:?}");
    b.add_coupling(coupling(0.5));
    assert_eq!(check(&a, &b), (false, Decided::Walk), "control");
}

#[test]
fn fields_the_snapshot_skips_are_unseen_entry_by_entry() {
    let (graph, keys) = base();
    let (mut a, b) = (graph.clone(), graph.clone());
    // A second address is held but not written: the snapshot writes the
    // primary one only. Raw or typed equality would see it.
    a.get_node_mut(keys[0])
        .unwrap()
        .addresses
        .push(kernel::address::address_from_url("https://elsewhere.test/"));
    assert_ne!(
        format!("{:?}", a.get_node(keys[0])),
        format!("{:?}", b.get_node(keys[0])),
        "precondition: the raw entries differ"
    );
    let (equal, decided) = check(&a, &b);
    assert!(equal && by_entries(decided), "{decided:?}");
    // The legacy image columns are skipped by an entry's serde as well.
    let node = b.to_snapshot_at(0).nodes[0].clone();
    let mut legacy = node.clone();
    legacy.legacy_thumbnail_png = Some(vec![1]);
    legacy.legacy_favicon_rgba = Some(vec![2]);
    assert_eq!(
        state_bytes(&StateEntry::Node(node)).unwrap(),
        state_bytes(&StateEntry::Node(legacy)).unwrap()
    );
    a.set_node_title(keys[0], "control".into());
    assert_eq!(check(&a, &b), (false, Decided::Walk), "control: a written field differs");
}

#[test]
fn map_order_is_content_order_entry_by_entry() {
    let (graph, keys) = base();
    let (mut a, mut b) = (graph.clone(), graph.clone());
    let id = Uuid::from_u128(2);
    for (side, order) in [(&mut a, ["p", "q"]), (&mut b, ["q", "p"])] {
        for name in order {
            side.facets_mut()
                .set(id, FacetId::new(name), json!({ "z": name, "a": [1] }), &AcceptAll)
                .unwrap();
        }
    }
    let (equal, decided) = check(&a, &b);
    assert!(equal && by_entries(decided), "{decided:?}");
    // A node's tags are a set the snapshot sorts.
    for (side, order) in [(&mut a, ["one", "two", "three"]), (&mut b, ["three", "two", "one"])] {
        for tag in order {
            side.get_node_mut(keys[2]).unwrap().tags.insert(tag.into());
        }
    }
    let (equal, decided) = check(&a, &b);
    assert!(equal && by_entries(decided), "{decided:?}");
    b.facets_mut()
        .set(id, FacetId::new("p"), json!(0), &AcceptAll)
        .unwrap();
    assert_eq!(check(&a, &b), (false, Decided::Walk), "control: a facet differs");
}

#[test]
fn edges_written_in_another_order_compare_as_the_snapshot_sorts_them() {
    let (graph, keys) = base();
    let (a, _, deltas) = sides(&graph, |g| {
        contain(g, keys[2], keys[0]);
        contain(g, keys[2], keys[1]);
    });
    let mut b = graph.clone();
    replay_captured_deltas_onto(&mut b, deltas.iter().rev().cloned());
    // Surface edges are settled by their sort keys, with no pair listed.
    assert_eq!(check(&a, &b), (true, Decided::Entries(0)));
    let mut c = b.clone();
    contain(&mut c, keys[1], keys[0]);
    assert!(!check(&a, &c).0, "control: one more edge differs");
}

#[test]
fn resources_and_their_facets_compare_entry_by_entry() {
    let (graph, keys) = base();
    let (a, b, _) = sides(&graph, |g| {
        g.append_node_properties(keys[1], position("2"));
    });
    let (equal, decided) = check(&a, &b);
    assert!(equal && by_entries(decided), "{decided:?}");
    let mut c = b.clone();
    c.append_node_properties(keys[2], position("3"));
    assert!(!check(&a, &c).0, "control: a property on one side differs");
}
