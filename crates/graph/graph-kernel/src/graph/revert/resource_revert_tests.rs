// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::graph::SemanticStatement;

fn record(iri: &str, facets: &[(&str, Value)]) -> CapturedDelta {
    CapturedDelta::ReplaySetResourceRecordById {
        resource_id: chartulary::resource_id(iri).to_string(),
        record: Some(PersistedResourceRecord {
            canonical_iri: chartulary::canonical_url(iri),
            facets: facets
                .iter()
                .map(|(facet, value)| PersistedResourceFacet {
                    facet: (*facet).into(),
                    value_json: value.to_string(),
                })
                .collect(),
        }),
    }
}

fn shown(surface: Uuid, resource: Uuid) -> CapturedDelta {
    CapturedDelta::ReplaySetShownResourceById {
        surface_id: surface.to_string(),
        resource_id: Some(resource.to_string()),
    }
}

fn pair(from: Uuid, to: Uuid, handle: &str) -> CapturedDelta {
    let mut source = Graph::new();
    let a = source.add_node_with_id(from, "https://a.test".into(), Default::default());
    let b = source.add_node_with_id(to, "https://b.test".into(), Default::default());
    source.assert_surface_persisted_semantic_statement(
        a,
        b,
        SemanticStatement {
            statement_id: handle.into(),
            predicate: "https://vocab.test/relation".into(),
            recognized_sub_kind: None,
            label: None,
            graph_scope: crate::types::GraphScope::User,
            provenance_iri: Some("https://people.test/source".into()),
            asserted_at_ms: Some(100),
            first_asserted_ms: Some(100),
            withdrawn: None,
            basis: Default::default(),
        },
    );
    CapturedDelta::ReplaySetResourceEdgesByIds {
        from_resource_id: from.to_string(),
        to_resource_id: to.to_string(),
        edges: source.persisted_edges_between(a, b),
    }
}

fn apply(graph: &mut Graph, edits: &[CapturedDelta]) {
    super::tests::apply_all(graph, edits);
}

#[test]
fn resource_undo_keeps_later_facet_keys_and_reports_same_key_conflicts() {
    let iri = "https://resource.test/a";
    let resource = chartulary::resource_id(iri);
    let mut before = Graph::new();
    apply(
        &mut before,
        &[record(iri, &[("a", Value::from(1)), ("b", Value::from(1))])],
    );
    let change = [record(iri, &[("a", Value::from(2)), ("b", Value::from(1))])];
    let mut after = before.clone();
    apply(&mut after, &change);
    let mut live = after.clone();
    apply(
        &mut live,
        &[record(
            iri,
            &[
                ("a", Value::from(2)),
                ("b", Value::from(3)),
                ("foreign", serde_json::json!({"kept":true})),
            ],
        )],
    );
    let undo = revert_change(&change, &before, &after, &live);
    assert!(undo.kept.is_empty());
    apply(&mut live, &undo.edits);
    let facets = resource_facets(&live.resource_record(resource).unwrap());
    assert_eq!(facets["a"], Value::from(1));
    assert_eq!(facets["b"], Value::from(3));
    assert_eq!(facets["foreign"], serde_json::json!({"kept":true}));

    let mut conflict = after.clone();
    apply(
        &mut conflict,
        &[record(iri, &[("a", Value::from(4)), ("b", Value::from(3))])],
    );
    let undo = revert_change(&change, &before, &after, &conflict);
    assert_eq!(undo.kept, [Part::ResourceFacet(resource, "a".into())]);
    assert!(undo.edits.is_empty());
    assert_eq!(
        resource_facets(&conflict.resource_record(resource).unwrap())["a"],
        Value::from(4)
    );
}

fn creation_fixture() -> (Graph, Graph, Vec<CapturedDelta>, Uuid, Uuid, Uuid, Uuid) {
    let surface_a = Uuid::from_u128(1);
    let surface_b = Uuid::from_u128(2);
    let resource_a = chartulary::resource_id("https://resource.test/a");
    let resource_b = chartulary::resource_id("https://resource.test/b");
    let mut before = Graph::new();
    before.add_node_with_id(
        surface_a,
        "https://surface.test/a".into(),
        Default::default(),
    );
    before.add_node_with_id(
        surface_b,
        "https://surface.test/b".into(),
        Default::default(),
    );
    apply(
        &mut before,
        &[
            record("https://resource.test/b", &[]),
            shown(surface_b, resource_b),
        ],
    );
    let change = vec![
        record("https://resource.test/a", &[]),
        pair(resource_a, resource_b, "owned-handle"),
        shown(surface_a, resource_a),
    ];
    let mut after = before.clone();
    apply(&mut after, &change);
    (
        before, after, change, surface_a, surface_b, resource_a, resource_b,
    )
}

#[test]
fn resource_undo_creation_clears_pairs_and_bindings_before_record() {
    let (before, after, change, surface_a, surface_b, resource_a, resource_b) = creation_fixture();
    let undo = revert_change(&change, &before, &after, &after);
    assert!(undo.kept.is_empty());
    let removal = undo
        .edits
        .iter()
        .position(|edit| {
            matches!(
                edit,
                CapturedDelta::ReplaySetResourceRecordById { record: None, .. }
            )
        })
        .unwrap();
    let pair = undo.edits.iter().position(|edit| matches!(edit, CapturedDelta::ReplaySetResourceEdgesByIds { edges, .. } if edges.is_empty())).unwrap();
    let binding = undo
        .edits
        .iter()
        .position(|edit| {
            matches!(
                edit,
                CapturedDelta::ReplaySetShownResourceById {
                    resource_id: None,
                    ..
                }
            )
        })
        .unwrap();
    assert!(pair < removal && binding < removal);
    let mut restored = after.clone();
    apply(&mut restored, &undo.edits);
    assert!(restored.resource_record(resource_a).is_none());
    assert!(
        restored
            .persisted_resource_edges_between(resource_a, resource_b)
            .is_empty()
    );
    assert_eq!(shown_resource(&restored, surface_a), None);
    assert_eq!(shown_resource(&restored, surface_b), Some(resource_b));
    assert_eq!(
        restored.resource_record(resource_b),
        before.resource_record(resource_b)
    );
    assert!(restored.get_node_by_id(surface_a).is_some());
}

#[test]
fn resource_undo_creation_keeps_later_surface_and_pair_references() {
    let (before, after, change, surface_a, surface_b, resource_a, resource_b) = creation_fixture();
    for later_surface in [true, false] {
        let mut live = after.clone();
        let resource_c = chartulary::resource_id("https://resource.test/c");
        if later_surface {
            apply(&mut live, &[shown(surface_b, resource_a)]);
        } else {
            apply(
                &mut live,
                &[
                    record("https://resource.test/c", &[]),
                    pair(resource_a, resource_c, "later-handle"),
                ],
            );
        }
        let undo = revert_change(&change, &before, &after, &live);
        assert!(undo.kept.contains(&Part::Resource(resource_a)));
        apply(&mut live, &undo.edits);
        assert!(live.resource_record(resource_a).is_some());
        assert!(
            live.persisted_resource_edges_between(resource_a, resource_b)
                .is_empty(),
            "owned pair still reverts"
        );
        assert_eq!(
            shown_resource(&live, surface_a),
            None,
            "owned binding still reverts"
        );
        assert!(live.resource_record(resource_b).is_some());
        if later_surface {
            assert_eq!(shown_resource(&live, surface_b), Some(resource_a));
        } else {
            let surviving = live.persisted_resource_edges_between(resource_a, resource_c);
            assert_eq!(
                surviving[0].retained_assertions()[0].statement_id,
                "later-handle"
            );
            assert_eq!(shown_resource(&live, surface_b), Some(resource_b));
        }
    }
}

#[test]
fn surface_creation_undo_preserves_later_shown_binding() {
    let resource_a = chartulary::resource_id("https://resource.test/a");
    let resource_b = chartulary::resource_id("https://resource.test/b");
    let surface = Uuid::from_u128(99);
    let mut before = Graph::new();
    apply(
        &mut before,
        &[
            record("https://resource.test/a", &[]),
            record("https://resource.test/b", &[]),
        ],
    );
    for initial_binding in [None, Some(resource_a)] {
        let mut change = vec![CapturedDelta::ReplayAddNodeWithIdIfMissing {
            id: surface.to_string(),
            url: "https://surface.test".into(),
            position: [0.0, 0.0],
        }];
        if let Some(resource) = initial_binding {
            change.push(shown(surface, resource));
        }
        let mut after = before.clone();
        apply(&mut after, &change);
        let undo = revert_change(&change, &before, &after, &after);
        assert!(undo.kept.is_empty());
        let mut original = after.clone();
        apply(&mut original, &undo.edits);
        assert!(original.get_node_by_id(surface).is_none());
        assert!(original.surface_ids_showing_resource(resource_a).is_empty());
        assert!(original.resource_record(resource_a).is_some());
        assert!(original.resource_record(resource_b).is_some());

        let mut later = after.clone();
        apply(&mut later, &[shown(surface, resource_b)]);
        let undo = revert_change(&change, &before, &after, &later);
        assert!(undo.kept.contains(&Part::Node(surface)));
        if initial_binding.is_some() {
            assert!(undo.kept.contains(&Part::ShownResource(surface)));
        }
        apply(&mut later, &undo.edits);
        assert!(later.get_node_by_id(surface).is_some());
        assert_eq!(shown_resource(&later, surface), Some(resource_b));
        assert_eq!(later.surface_ids_showing_resource(resource_b), [surface]);
        assert!(later.resource_record(resource_a).is_some());
    }
}

#[test]
fn surface_removal_undo_restores_shown_binding_and_keeps_later_recreation() {
    let (_, before, _, surface_a, surface_b, resource_a, resource_b) = creation_fixture();
    let change = [CapturedDelta::ReplayRemoveNodeById {
        node_id: surface_a.to_string(),
    }];
    let mut after = before.clone();
    apply(&mut after, &change);
    assert!(after.get_node_by_id(surface_a).is_none());
    assert!(after.surface_ids_showing_resource(resource_a).is_empty());
    let undo = revert_change(&change, &before, &after, &after);
    assert!(undo.kept.is_empty());
    let mut restored = after.clone();
    apply(&mut restored, &undo.edits);
    assert_eq!(shown_resource(&restored, surface_a), Some(resource_a));
    assert_eq!(shown_resource(&restored, surface_b), Some(resource_b));
    assert_eq!(
        restored.resource_record(resource_a),
        before.resource_record(resource_a)
    );

    let mut later = after.clone();
    later.add_node_with_id(surface_a, "https://later.test".into(), Default::default());
    apply(&mut later, &[shown(surface_a, resource_b)]);
    let undo = revert_change(&change, &before, &after, &later);
    assert!(undo.kept.contains(&Part::Node(surface_a)));
    assert!(undo.kept.contains(&Part::ShownResource(surface_a)));
    apply(&mut later, &undo.edits);
    assert_eq!(shown_resource(&later, surface_a), Some(resource_b));
    assert_eq!(
        later.get_node_by_id(surface_a).unwrap().1.url(),
        "https://later.test"
    );
    assert_eq!(shown_resource(&later, surface_b), Some(resource_b));
}

#[test]
fn resource_touched_parts_keep_equal_surface_ids_separate() {
    let from = Uuid::from_u128(1);
    let to = Uuid::from_u128(2);
    let touched = Touched::of(&[CapturedDelta::ReplaySetResourceEdgesByIds {
        from_resource_id: from.to_string(),
        to_resource_id: to.to_string(),
        edges: Vec::new(),
    }]);
    assert!(touched.reaches(&Part::ResourceEdges(from, to)));
    assert!(touched.reaches(&Part::Resource(from)));
    assert!(!touched.reaches(&Part::Edges(from, to)));
    assert!(!touched.reaches(&Part::Node(from)));
    let touched = Touched::of(&[shown(from, to)]);
    assert!(touched.reaches(&Part::ShownResource(from)));
    assert!(touched.reaches(&Part::Resource(to)));
    assert!(!touched.reaches(&Part::Resource(from)));
}
