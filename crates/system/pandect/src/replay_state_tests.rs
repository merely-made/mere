// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The byte check agrees with the `Value` check it replaced, on every case
//! where the two could part: non-finite floats, a zero's sign, skipped
//! fields and map order.
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
