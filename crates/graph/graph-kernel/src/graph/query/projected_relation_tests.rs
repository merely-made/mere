// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::graph::{ResourceNode, SemanticStatement, SemanticSubKind};
use crate::persistence::PersistedResourceRecord;
use crate::types::GraphScope;

#[test]
fn projected_rows_keep_owner_direction_multiplicity_and_all_shown_pairs() {
    let mut graph = Graph::new();
    let [a, alias_a, b, alias_b, unbound] = std::array::from_fn::<_, 5, _>(|i| {
        graph.add_node(format!("https://surface.test/{i}"), Default::default())
    });
    let surface_key = graph
        .assert_relation(
            a,
            b,
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::UserGrouped,
                label: None,
            },
        )
        .unwrap();
    let [ra, rb, unshown] = ["a", "b", "unshown"].map(|suffix| {
        let iri = format!("https://resource.test/{suffix}");
        let resource = ResourceNode::for_term(&iri);
        assert!(graph.set_resource_record(
            resource.id(),
            Some(PersistedResourceRecord {
                canonical_iri: iri,
                facets: vec![],
            })
        ));
        resource.id()
    });
    for (from, to, ids) in [
        (ra, rb, vec!["forward-1", "forward-2"]),
        (rb, ra, vec!["backward"]),
        (ra, ra, vec!["self"]),
        (ra, unshown, vec!["not-shown"]),
    ] {
        let records: Vec<_> = ids
            .into_iter()
            .map(|id| {
                let mut payload = EdgePayload::new();
                payload.push_persisted_semantic_statement(SemanticStatement {
                    statement_id: id.into(),
                    predicate: crate::graph::predicate_iri(SemanticSubKind::Cites).into(),
                    recognized_sub_kind: Some(SemanticSubKind::Cites),
                    label: Some(id.into()),
                    graph_scope: GraphScope::Default,
                    provenance_iri: Some("https://author.test/".into()),
                    asserted_at_ms: Some(17),
                    first_asserted_ms: Some(17),
                    withdrawn: None,
                    basis: Default::default(),
                });
                super::super::snapshot::persisted_edge_for_ids(from, to, &payload)
            })
            .collect();
        assert!(graph.set_resource_edges_between(from, to, &records));
    }
    assert_eq!(
        graph.projected_relations().count(),
        1,
        "unshown truth stays stored"
    );
    for (surface, resource) in [(a, ra), (alias_a, ra), (b, rb), (alias_b, rb)] {
        assert!(graph.set_shown_resource(graph.get_node(surface).unwrap().id, Some(resource)));
    }
    let rows: Vec<_> = graph.projected_relations().collect();
    assert_eq!(rows.len(), 17); // surface + 8 forward + 4 backward + 4 self
    assert_eq!(rows[0].0, surface_key, "surface prefix keeps its owner");
    for source in [a, alias_a] {
        for target in [b, alias_b] {
            assert_eq!(
                rows.iter()
                    .filter(|(key, row)| matches!(key, RelationKey::Resource(_))
                        && row.from == source
                        && row.to == target)
                    .count(),
                2
            );
            assert_eq!(
                rows.iter()
                    .filter(|(key, row)| matches!(key, RelationKey::Resource(_))
                        && row.from == target
                        && row.to == source)
                    .count(),
                1
            );
        }
        for target in [a, alias_a] {
            assert_eq!(
                rows.iter()
                    .filter(|(_, row)| row.from == source && row.to == target)
                    .count(),
                1,
                "resource self-loop lifts every alias pair"
            );
        }
    }
    assert!(
        rows.iter()
            .all(|(key, _)| graph.get_relation(*key).is_some())
    );
    assert!(
        rows.iter()
            .all(|(_, row)| row.from != unbound && row.to != unbound)
    );
    assert_eq!(
        graph.resource_relations().count(),
        5,
        "unshown assertion was retained"
    );
    assert!(graph.set_shown_resource(graph.get_node(alias_b).unwrap().id, None));
    assert_eq!(graph.projected_relations().count(), 11);
    assert_eq!(graph.resource_relations().count(), 5);
}
