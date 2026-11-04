// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::graph::SemanticSubKind;
use euclid::default::Point2D;

#[test]
fn shown_joins_cost_zero_and_hidden_resources_carry_relation_hops() {
    use crate::graph::{EdgeFamily, NavigationTrigger, ResourceNode, SemanticStatement};
    use crate::persistence::PersistedResourceRecord;
    use crate::types::GraphScope;

    let mut graph = Graph::new();
    let [ra, hidden, rc] = ["a", "hidden", "c"].map(|name| {
        let iri = format!("https://resource.test/{name}");
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
    let [a, alias, c, d, isolated] = std::array::from_fn::<_, 5, _>(|i| {
        // Equal UUIDs in different strata must not join accidentally.
        let id = if i == 3 {
            hidden
        } else {
            Uuid::from_u128(100 + i as u128)
        };
        graph.add_node_with_id(id, format!("https://surface.test/{i}"), Default::default())
    });
    let ids = [a, alias, c, d, isolated].map(|key| graph.get_node(key).unwrap().id);
    for (key, resource) in [(a, ra), (alias, ra), (c, rc)] {
        assert!(graph.set_shown_resource(graph.get_node(key).unwrap().id, Some(resource)));
    }
    for (from, to, kinds) in [
        (
            ra,
            hidden,
            vec![SemanticSubKind::Cites, SemanticSubKind::Quotes],
        ),
        (rc, hidden, vec![SemanticSubKind::Cites]),
        (hidden, hidden, vec![SemanticSubKind::Cites]),
    ] {
        let records: Vec<_> = kinds
            .into_iter()
            .enumerate()
            .map(|(i, kind)| {
                let mut payload = EdgePayload::new();
                payload.push_persisted_semantic_statement(SemanticStatement {
                    statement_id: format!("walk:{from}:{to}:{i}"),
                    predicate: crate::graph::predicate_iri(kind).into(),
                    recognized_sub_kind: Some(kind),
                    label: Some("retained".into()),
                    graph_scope: GraphScope::Custom("https://scope.test/walk".into()),
                    provenance_iri: Some("https://author.test/walk".into()),
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
    assert!(graph.append_traversal(c, d, NavigationTrigger::LinkClick, Some(19)));
    let before = graph.to_snapshot();
    let revision = graph.revision();
    let [a_id, alias_id, c_id, d_id, isolated_id] = ids;
    let semantic = [RelationSelector::Family(EdgeFamily::Semantic)];
    let traversal = [RelationSelector::Family(EdgeFamily::Traversal)];
    assert_eq!(graph.ego_members(a_id, 0, &[]), vec![a_id, alias_id]);
    assert_eq!(graph.ego_members(alias_id, 0, &[]), vec![alias_id, a_id]);
    assert_eq!(graph.ego_members(a_id, 1, &[]), vec![a_id, alias_id]);
    assert_eq!(graph.ego_members(a_id, 2, &[]), vec![a_id, alias_id, c_id]);
    assert_eq!(
        graph.ego_members(a_id, 3, &[]),
        vec![a_id, alias_id, c_id, d_id]
    );
    assert_eq!(
        graph.component_members(a_id, &semantic),
        vec![a_id, alias_id, c_id]
    );
    assert_eq!(
        graph.component_members(a_id, &traversal),
        vec![a_id, alias_id]
    );
    assert_eq!(
        graph.component_members(a_id, &[RelationSelector::Semantic(SemanticSubKind::Quotes)]),
        vec![a_id, alias_id]
    );
    assert_eq!(
        graph.component_members(c_id, &semantic),
        vec![c_id, a_id, alias_id]
    );
    assert_eq!(
        graph.component_members(a_id, &[]),
        vec![a_id, alias_id, c_id, d_id]
    );
    assert_eq!(graph.component_members(isolated_id, &[]), vec![isolated_id]);
    assert!(graph.component_members(Uuid::nil(), &[]).is_empty());
    assert_eq!(graph.resource_relations().count(), 4);
    assert_eq!(
        serde_json::to_value(graph.to_snapshot()).unwrap(),
        serde_json::to_value(before).unwrap(),
        "walking retains all stored assertions"
    );
    assert_eq!(graph.revision(), revision, "walking is read-only");
}

/// An A–B–C chain plus an isolated D. Returns the graph and `[a, b, c, d]` uuids.
fn chain_plus_isolate() -> (Graph, [uuid::Uuid; 4]) {
    let mut g = Graph::new();
    let a = g.add_node("https://a".to_string(), Point2D::new(0.0, 0.0));
    let b = g.add_node("https://b".to_string(), Point2D::new(1.0, 0.0));
    let c = g.add_node("https://c".to_string(), Point2D::new(2.0, 0.0));
    let d = g.add_node("https://d".to_string(), Point2D::new(9.0, 9.0));
    let sem = || EdgeAssertion::Semantic {
        sub_kind: SemanticSubKind::Hyperlink,
        label: None,
    };
    g.assert_relation(a, b, sem());
    g.assert_relation(b, c, sem());
    let ids = [
        g.get_node(a).unwrap().id,
        g.get_node(b).unwrap().id,
        g.get_node(c).unwrap().id,
        g.get_node(d).unwrap().id,
    ];
    (g, ids)
}

#[test]
fn component_members_is_the_whole_connected_component() {
    let (g, [a, _b, _c, d]) = chain_plus_isolate();
    assert_eq!(g.component_members(a, &[]).len(), 3, "A reaches B and C");
    assert_eq!(g.component_members(d, &[]).len(), 1, "D is isolated");
    assert!(
        g.component_members(uuid::Uuid::nil(), &[]).is_empty(),
        "unknown seed: empty"
    );
}

#[test]
fn ego_members_is_radius_bounded() {
    let (g, [a, _b, _c, _d]) = chain_plus_isolate();
    assert_eq!(
        g.ego_members(a, 0, &[]).len(),
        1,
        "radius 0 is just the seed"
    );
    assert_eq!(
        g.ego_members(a, 1, &[]).len(),
        2,
        "radius 1 reaches B, not C"
    );
    assert_eq!(g.ego_members(a, 2, &[]).len(), 3, "radius 2 reaches C");
}

#[test]
fn the_selector_projection_changes_the_derived_shape() {
    use crate::graph::{ContainmentSubKind, EdgeFamily, RelationSelector};
    // A —Semantic→ B —Containment→ C. The same three nodes, two relation families.
    let mut g = Graph::new();
    let a = g.add_node("https://a".to_string(), Point2D::new(0.0, 0.0));
    let b = g.add_node("https://b".to_string(), Point2D::new(1.0, 0.0));
    let c = g.add_node("https://c".to_string(), Point2D::new(2.0, 0.0));
    g.assert_relation(
        a,
        b,
        EdgeAssertion::Semantic {
            sub_kind: SemanticSubKind::Hyperlink,
            label: None,
        },
    );
    g.assert_relation(
        b,
        c,
        EdgeAssertion::Containment {
            sub_kind: ContainmentSubKind::CollectionMember,
        },
    );
    let a_id = g.get_node(a).unwrap().id;

    let semantic = [RelationSelector::Family(EdgeFamily::Semantic)];
    let containment = [RelationSelector::Family(EdgeFamily::Containment)];
    // Whole graph under no projection; Semantic stops at B; Containment never leaves A.
    assert_eq!(
        g.component_members(a_id, &[]).len(),
        3,
        "all families: A, B, C"
    );
    assert_eq!(
        g.component_members(a_id, &semantic).len(),
        2,
        "Semantic projection: A, B (B→C is Containment, not followed)"
    );
    assert_eq!(
        g.component_members(a_id, &containment).len(),
        1,
        "Containment projection: just A (no Containment edge from A)"
    );
}

/// Q2 (ruled 2026-09-12): an antiparallel pair carrying different
/// families on each arc (n1->n2 Semantic hyperlink, n2->n1 Traversal,
/// the shape `tests/snapshot_basic.rs::test_snapshot_preserves_edge_types`
/// pins as two separate edges) is crossed by a selector-projected walk
/// from either end, because `edge_matches_selectors` scans the whole
/// pair rather than the first arc `find_edge_key` happens to return.
#[test]
fn antiparallel_pair_is_crossed_from_either_end() {
    use crate::graph::{EdgeFamily, NavigationTrigger, RelationSelector};
    let mut g = Graph::new();
    let n1 = g.add_node("https://a".to_string(), Point2D::new(0.0, 0.0));
    let n2 = g.add_node("https://b".to_string(), Point2D::new(1.0, 0.0));
    g.assert_relation(
        n1,
        n2,
        EdgeAssertion::Semantic {
            sub_kind: SemanticSubKind::Hyperlink,
            label: None,
        },
    );
    assert!(g.append_traversal(n2, n1, NavigationTrigger::LinkClick, Some(1_000_000)));
    assert_eq!(g.edge_count(), 1, "Traversal remains Surface");
    assert_eq!(
        g.resource_relations().count(),
        1,
        "Hyperlink belongs to Resource"
    );
    let pair: Vec<_> = g
        .projected_relations_between(n1, n2)
        .chain(g.projected_relations_between(n2, n1))
        .collect();
    assert_eq!(pair.len(), 2, "the pair primitive sees both arcs");
    assert_eq!(
        g.projected_relations_between(n2, n1)
            .chain(g.projected_relations_between(n1, n2))
            .count(),
        2,
        "and from the other end"
    );
    let id1 = g.get_node(n1).unwrap().id;
    let id2 = g.get_node(n2).unwrap().id;

    for family in [EdgeFamily::Traversal, EdgeFamily::Semantic] {
        let sel = [RelationSelector::Family(family)];
        for (seed, other) in [(id1, id2), (id2, id1)] {
            let members = g.component_members(seed, &sel);
            assert!(
                members.contains(&seed) && members.contains(&other),
                "{family:?} projection seeded at {seed} must cross the pair; got {members:?}"
            );
        }
    }
    // A family on neither arc still excludes.
    let sel = [RelationSelector::Family(EdgeFamily::Containment)];
    assert_eq!(g.component_members(id1, &sel), vec![id1]);
    assert_eq!(g.component_members(id2, &sel), vec![id2]);
}
