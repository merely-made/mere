// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::graph::{
    ArrangementSubKind, EdgeFamily, ImportedSubKind, NavigationTrigger, ProvenanceSubKind,
    RelationKind, SemanticSubKind,
};
use euclid::default::Point2D;
use std::collections::BTreeSet;

fn row_families(
    g: &Graph,
) -> Vec<(
    RelationKey,
    NodeKey,
    NodeKey,
    BTreeSet<EdgeFamily>,
    BTreeSet<EdgeFamily>,
)> {
    g.nodes()
        .flat_map(|(from, _)| {
            g.projected_outgoing_relations(from)
                .map(move |(to, key, payload)| {
                    let rows = relation_rows(from, to, payload);
                    let from_rows: BTreeSet<EdgeFamily> =
                        rows.iter().map(|r| r.kind.family()).collect();
                    (key, from, to, payload.families(), from_rows)
                })
        })
        .collect()
}

#[test]
fn families_equal_row_families_for_every_edge_shape() {
    let mut g = Graph::new();
    let mut node =
        |g: &mut Graph, i: usize| g.add_node(format!("https://n{i}"), Point2D::new(i as f32, 0.0));
    let hub = node(&mut g, 0);
    let open = node(&mut g, 1);
    let hyper = node(&mut g, 2);
    let trav = node(&mut g, 3);
    let cont = node(&mut g, 4);
    let arr = node(&mut g, 5);
    let imp = node(&mut g, 6);
    let prov = node(&mut g, 7);
    let mixed = node(&mut g, 8);
    let sem = || EdgeAssertion::Semantic {
        sub_kind: SemanticSubKind::Hyperlink,
        label: None,
    };

    // Open predicate only: Semantic to `has_family`, formerly no rows.
    let open_key = g
        .assert_semantic_predicate(hub, open, "https://schema.org/citation".to_string())
        .expect("open-predicate edge");
    g.assert_relation(hub, hyper, sem());
    assert!(g.append_traversal(hub, trav, NavigationTrigger::LinkClick, Some(1)));
    g.assert_relation(
        hub,
        cont,
        EdgeAssertion::Containment {
            sub_kind: ContainmentSubKind::CollectionMember,
        },
    );
    g.assert_relation(
        hub,
        arr,
        EdgeAssertion::Arrangement {
            sub_kind: ArrangementSubKind::FrameMember,
        },
    );
    g.assert_relation(
        hub,
        imp,
        EdgeAssertion::Imported {
            sub_kind: ImportedSubKind::BookmarkFolder,
        },
    );
    g.assert_relation(
        hub,
        prov,
        EdgeAssertion::Provenance {
            sub_kind: ProvenanceSubKind::ClippedFrom,
        },
    );
    // Historical mixed Surface payload remains readable beside live Resource claims.
    g.assert_surface_relation_as(hub, mixed, sem(), g.write_author().asserter_iri(), None);
    g.assert_surface_semantic_statement(
        hub,
        mixed,
        crate::graph::SemanticStatementSpec {
            predicate: "https://example.org/related".into(),
            ..Default::default()
        },
    );
    assert!(g.append_traversal(hub, mixed, NavigationTrigger::Back, Some(2)));

    // The open-predicate edge yields exactly one OpenPredicate row.
    let open_rows = relation_rows(hub, open, g.get_relation(open_key).unwrap());
    assert_eq!(
        open_rows.iter().map(|r| r.kind).collect::<Vec<_>>(),
        vec![RelationKind::OpenPredicate]
    );

    // Live graph: every edge agrees.
    for (_, from, to, families, from_rows) in row_families(&g) {
        assert_eq!(families, from_rows, "live edge {from:?}->{to:?}");
    }

    // Event-free Traversal sidecar: unreachable from the writers
    // (`push_traversal` always records metrics), so reach it through
    // a snapshot whose traversal record has no events and zero
    // metrics. `has_family(Traversal)` is sidecar presence, and the
    // row must follow it.
    let mut snapshot = g.to_snapshot();
    let mut stripped = 0;
    for edge in &mut snapshot.edges {
        if let Some(t) = edge.traversal.as_mut() {
            t.traversals.clear();
            t.metrics = Default::default();
            stripped += 1;
        }
    }
    assert_eq!(stripped, 2, "the traversal-only edge and the mixed edge");
    let restored = Graph::from_snapshot(&snapshot);
    // Note: the restored graph is not asserted edge-for-edge equal to
    // the live one. `snapshot/from.rs` only restores `UrlPath` /
    // `Domain` containment sub-kinds, so the CollectionMember edge
    // does not come back; that gap predates this test and is not
    // what it pins. Parity is checked on every edge that survives.
    let mut saw_event_free_traversal = 0;
    let mut saw_open_predicate = 0;
    for (key, from, to, families, from_rows) in row_families(&restored) {
        assert_eq!(families, from_rows, "restored edge {from:?}->{to:?}");
        let payload = restored.get_relation(key).unwrap();
        if families.contains(&EdgeFamily::Traversal) {
            assert!(payload.traversals().is_empty());
            assert_eq!(payload.metrics().total_navigations, 0);
            saw_event_free_traversal += 1;
        }
        saw_open_predicate += relation_rows(from, to, payload)
            .iter()
            .filter(|r| r.kind == RelationKind::OpenPredicate)
            .count();
    }
    assert_eq!(
        saw_event_free_traversal, 2,
        "traversal-only and mixed edges"
    );
    assert_eq!(saw_open_predicate, 2, "open-predicate-only and mixed edges");
}

#[test]
fn open_predicate_tag_round_trips_and_keeps_the_semantic_family_byte() {
    let tag = RelationKind::OpenPredicate.tag();
    assert_eq!(
        tag >> 24,
        RelationKind::Semantic(SemanticSubKind::Hyperlink).tag() >> 24
    );
    assert_eq!(
        RelationKind::from_tag(tag),
        Some(RelationKind::OpenPredicate)
    );
    assert_eq!(RelationKind::OpenPredicate.family(), EdgeFamily::Semantic);
}
