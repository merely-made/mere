// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Load and both freezes ask one rule about assertion handles.
use super::apply::add_node;
use super::assertion_handles::{AssertionHandles, validate_assertion_handles};
use super::*;
use uuid::Uuid;

const REASON: &str = "one handle means one row";

/// A Resource pair holding one claim: legal state.
pub(super) fn claimed_pair() -> (Graph, Uuid, Uuid) {
    let mut graph = Graph::new();
    let mut shown = |id: u128, url: &str| {
        let surface = add_node(
            &mut graph,
            Some(Uuid::from_u128(id)),
            url.into(),
            Default::default(),
        );
        graph.shown_resource_id(surface).unwrap()
    };
    let (a, b) = (
        shown(1, "https://handles.test/a"),
        shown(2, "https://handles.test/b"),
    );
    graph
        .try_assert_semantic_statement_by_resource_ids(
            a,
            b,
            SemanticStatementSpec {
                predicate: "urn:test:claim".into(),
                provenance_iri: Some("urn:test:author".into()),
                asserted_at_ms: Some(1),
                ..Default::default()
            },
        )
        .unwrap();
    (graph, a, b)
}

/// Give the pair a second identical row, bypassing the checked writers as
/// stored data that predates the rule did.
pub(super) fn copy_resource_row(graph: &mut Graph, from: Uuid, to: Uuid) {
    let row = graph
        .resource_edges()
        .find(|(f, t, _)| f.id() == from && t.id() == to)
        .map(|(_, _, payload)| payload.clone())
        .unwrap();
    let (from, to) = (
        graph.resources.key_of(&from).unwrap(),
        graph.resources.key_of(&to).unwrap(),
    );
    graph.resources.connect(from, to, row);
}

fn source() -> FrozenLinkSource {
    FrozenLinkSource::Unavailable {
        reason: "no public source".into(),
    }
}

fn load_reasons(graph: &Graph) -> [String; 2] {
    let snapshot = graph.to_snapshot();
    [
        Graph::try_from_snapshot(&snapshot).err(),
        Graph::try_from_recorded_snapshot(&snapshot).err(),
    ]
    .map(|error| error.expect("load refuses the copy").to_string())
}

#[test]
fn identical_copies_on_parallel_rows_are_refused_by_load_and_both_freezes_with_one_reason() {
    let (mut graph, a, b) = claimed_pair();
    copy_resource_row(&mut graph, a, b);
    assert_eq!(graph.resource_edges().count(), 2);
    let before = graph.to_snapshot();

    let [checked, recorded] = load_reasons(&graph);
    let link = graph
        .freeze_link_with_nonce(GraphStratum::Resource, a, b, source(), Uuid::from_u128(3))
        .unwrap_err();
    let selection = graph
        .freeze_resource_selection_with_nonce(serde_json::json!({}), vec![a, b], Uuid::from_u128(4))
        .unwrap_err();

    assert!(checked.contains(REASON), "{checked}");
    assert_eq!([&recorded, &link, &selection], [&checked; 3]);
    assert_eq!(
        graph.to_snapshot().resources,
        before.resources,
        "a refused freeze mints nothing"
    );
}

#[test]
fn a_graph_without_copies_loads_and_freezes_both_ways() {
    let (mut graph, a, b) = claimed_pair();
    let snapshot = graph.to_snapshot();
    assert!(Graph::try_from_snapshot(&snapshot).is_ok());
    assert!(Graph::try_from_recorded_snapshot(&snapshot).is_ok());
    let link = graph
        .freeze_link_with_nonce(GraphStratum::Resource, a, b, source(), Uuid::from_u128(3))
        .unwrap();
    let selection = graph
        .freeze_resource_selection_with_nonce(serde_json::json!({}), vec![a, b], Uuid::from_u128(4))
        .unwrap();
    assert_eq!(
        graph
            .open_frozen_link(link)
            .unwrap()
            .link
            .statements()
            .len(),
        1
    );
    assert_eq!(
        graph
            .open_frozen_selection(selection)
            .unwrap()
            .selection
            .statements()
            .len(),
        1
    );
    assert!(Graph::try_from_snapshot(&graph.to_snapshot()).is_ok());
}

#[test]
fn a_repeat_is_refused_across_strata_and_within_one_row_whether_or_not_identical() {
    let (graph, ..) = claimed_pair();
    let row = graph.to_snapshot().resource_edges.remove(0);
    let mut changed = row.clone();
    changed.assertions.as_mut().unwrap()[0].predicate = "urn:test:other".into();
    let mut twice = row.clone();
    let again = twice.assertions.as_ref().unwrap()[0].clone();
    twice.assertions.as_mut().unwrap().push(again);

    assert!(validate_assertion_handles([(GraphStratum::Resource, &row)]).is_ok());
    for (first, second) in [
        (GraphStratum::Resource, GraphStratum::Resource),
        (GraphStratum::Surface, GraphStratum::Resource),
    ] {
        for copy in [&row, &changed] {
            let error = validate_assertion_handles([(first, &row), (second, copy)]).unwrap_err();
            assert_eq!((error.first.stratum, error.again.stratum), (first, second));
            assert_eq!(
                error.statement_id,
                row.retained_assertions()[0].statement_id
            );
        }
    }
    assert!(validate_assertion_handles([(GraphStratum::Resource, &twice)]).is_err());

    let mut held = AssertionHandles::new();
    held.hold_legacy_surface(&row);
    held.hold_legacy_surface(&row);
    assert!(held.holds(&row.retained_assertions()[0].statement_id));
    assert!(
        held.admit(GraphStratum::Resource, &row).is_err(),
        "legacy rows still bar later copies"
    );
}
