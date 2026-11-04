// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn exact_containment_assertions_keep_direction_ids_and_held_history_projection() {
    let mut graph = Graph::new();
    let a = Uuid::from_u128(9101);
    let b = Uuid::from_u128(9102);
    for (id, address) in [(a, "https://member.test/"), (b, "https://collection.test/")] {
        kernel::graph::apply::add_node(&mut graph, Some(id), address.into(), Default::default());
    }
    let make_record = |id: &str, kind: RelationKind, author: &str| SemanticStatement {
        statement_id: id.to_owned(),
        predicate: kind.predicate_iri().expect("canonical predicate").into(),
        recognized_sub_kind: None,
        label: Some(format!("record {id}")),
        basis: AssertionBasis {
            note: Some(format!("independent basis {id}")),
            ..Default::default()
        },
        graph_scope: GraphScope::Custom("urn:collection:fixture".into()),
        provenance_iri: Some(author.into()),
        first_asserted_ms: Some(100),
        asserted_at_ms: Some(110),
        withdrawn: None,
    };
    for (id, kind, author) in [
        (
            "containment-one",
            RelationKind::Containment(ContainmentSubKind::CollectionMember),
            "urn:author:one",
        ),
        (
            "containment-two",
            RelationKind::Containment(ContainmentSubKind::CollectionMember),
            "urn:author:two",
        ),
        (
            "arrangement-one",
            RelationKind::Arrangement(ArrangementSubKind::FrameMember),
            "urn:author:arrangement",
        ),
        (
            "imported-one",
            RelationKind::Imported(ImportedSubKind::BookmarkFolder),
            "urn:author:imported",
        ),
        (
            "provenance-one",
            RelationKind::Provenance(ProvenanceSubKind::ClippedFrom),
            "urn:author:provenance",
        ),
    ] {
        let record = make_record(id, kind, author);
        graph
            .try_assert_persisted_relation_statement(
                graph.get_node_key_by_id(a).unwrap(),
                graph.get_node_key_by_id(b).unwrap(),
                record,
            )
            .unwrap();
    }
    let withdrawal_actor = graph.write_author().asserter_iri();
    assert!(graph.retract_assertion("containment-two"));

    let held = read_link(
        &graph,
        a,
        b,
        LinkSource::Unavailable {
            reason: "test".into(),
        },
    )
    .unwrap();
    let held_assertions = held
        .links
        .iter()
        .flat_map(|link| &link.buckets)
        .flat_map(|bucket| &bucket.assertions)
        .collect::<Vec<_>>();
    let held_ids = held_assertions
        .iter()
        .map(|assertion| assertion.id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        held_ids,
        BTreeSet::from([
            "arrangement-one",
            "containment-one",
            "imported-one",
            "provenance-one",
        ])
    );
    assert!(
        held_assertions
            .iter()
            .any(|assertion| assertion.id == "containment-one")
    );
    assert!(
        !held_assertions
            .iter()
            .any(|assertion| assertion.id == "containment-two")
    );
    let containment = held_assertions
        .iter()
        .find(|assertion| assertion.id == "containment-one")
        .unwrap();
    assert_eq!(
        containment.predicate,
        RelationKind::Containment(ContainmentSubKind::CollectionMember)
            .predicate_iri()
            .unwrap()
    );
    assert_eq!(containment.recognized_kind, None);
    assert_eq!(containment.asserter.as_deref(), Some("urn:author:one"));
    assert!(
        held.links
            .iter()
            .flat_map(|link| &link.buckets)
            .all(|bucket| {
                bucket.assertions.is_empty()
                    || (bucket.displayed_from == a.to_string()
                        && bucket.displayed_to == b.to_string())
            })
    );

    let history = read_link_history(
        &graph,
        a,
        b,
        LinkSource::Unavailable {
            reason: "test".into(),
        },
    )
    .unwrap();
    let retained = history
        .links
        .iter()
        .flat_map(|link| &link.buckets)
        .flat_map(|bucket| &bucket.assertions)
        .collect::<Vec<_>>();
    assert_eq!(
        retained
            .iter()
            .map(|assertion| assertion.id.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "arrangement-one",
            "containment-one",
            "containment-two",
            "imported-one",
            "provenance-one",
        ])
    );
    assert!(retained.iter().any(|assertion| {
        assertion.id == "containment-two"
            && assertion.withdrawn.as_ref().is_some_and(|withdrawal| {
                withdrawal.by.as_deref() == Some(withdrawal_actor.as_str())
            })
    }));
    assert!(history.links.iter().any(|link| {
        link.buckets.iter().any(|bucket| {
            bucket.displayed_from == a.to_string() && bucket.displayed_to == b.to_string()
        })
    }));
}
