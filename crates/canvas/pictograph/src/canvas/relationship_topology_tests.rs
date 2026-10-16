// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
use super::*;
use kernel::graph::apply::{GraphDelta, add_node, apply_graph_delta};
use kernel::graph::{
    ContainmentSubKind, RelationKind, SemanticStatement, SemanticStatementSpec, SemanticSubKind,
    predicate_iri,
};
use kernel::types::GraphScope;
fn fixture() -> (Graph, [Uuid; 4]) {
    let mut graph = Graph::new();
    let ids = [1, 2, 3, 4].map(Uuid::from_u128);
    for (id, url) in ids.into_iter().zip([
        "https://source.test/",
        "https://target.test/",
        "https://source.test/",
        "https://target.test/",
    ]) {
        add_node(&mut graph, Some(id), url.into(), Default::default());
    }
    (graph, ids)
}
fn assertion(
    graph: &mut Graph,
    a: Uuid,
    b: Uuid,
    kind: Option<SemanticSubKind>,
    author: &str,
) -> String {
    graph
        .try_assert_semantic_statement(
            graph.get_node_key_by_id(a).unwrap(),
            graph.get_node_key_by_id(b).unwrap(),
            SemanticStatementSpec {
                predicate: kind
                    .map(predicate_iri)
                    .unwrap_or("urn:open:relationship")
                    .into(),
                recognized_sub_kind: kind,
                label: None,
                basis: Default::default(),
                graph_scope: GraphScope::Custom("urn:graph:review".into()),
                provenance_iri: Some(author.into()),
                asserted_at_ms: Some(17),
            },
        )
        .unwrap()
        .1
        .statement_id
}
#[test]
fn assertions_preserve_current_projected_row_multiplicity() {
    let (mut graph, [a, b, _, _]) = fixture();
    let first = assertion(&mut graph, a, b, Some(SemanticSubKind::Cites), "urn:alice");
    let second = assertion(&mut graph, a, b, Some(SemanticSubKind::Cites), "urn:bob");
    let before = serde_json::to_value(graph.to_snapshot()).unwrap();
    let pair = inspect_pair(&graph, a, b, |_, _, _| false).unwrap();
    assert_eq!((pair.held_rows, pair.visible_rows), (2, 2));
    assert_eq!(pair.cells[0].assertions, BTreeSet::from([first, second]));
    assert_eq!(serde_json::to_value(graph.to_snapshot()).unwrap(), before);
}

#[test]
fn closed_surface_link_hide_filters_all_rows_without_merging_their_controller_weight() {
    let (mut graph, [a, b, _, _]) = fixture();
    assertion(
        &mut graph,
        a,
        b,
        Some(SemanticSubKind::UserGrouped),
        "urn:alice",
    );
    assertion(
        &mut graph,
        a,
        b,
        Some(SemanticSubKind::UserGrouped),
        "urn:bob",
    );
    let key = LinkKey::new(
        scenograph::link::LinkStratum::Surface,
        a.to_string(),
        b.to_string(),
    );
    let hidden = BTreeSet::from([key]);
    let pair = inspect_pair_with_link_hides(&graph, a, b, &hidden, |_, _, _| false).unwrap();
    assert_eq!((pair.held_rows, pair.visible_rows), (2, 0));
    let ak = graph.get_node_key_by_id(a).unwrap();
    let bk = graph.get_node_key_by_id(b).unwrap();
    assert_eq!(
        crate::canvas::seiche_bridge::visible_relation_edges_with_link_hides(
            &graph,
            &std::collections::HashSet::new(),
            &hidden.iter().cloned().collect(),
        )
        .into_iter()
        .filter(|(from, to)| (*from == ak && *to == bk) || (*from == bk && *to == ak))
        .count(),
        0,
        "a closed link hide removes every spring row in its own source stratum"
    );
}
#[test]
fn visibility_and_controller_share_exact_cell_filter() {
    let (mut graph, [a, b, _, _]) = fixture();
    assertion(&mut graph, a, b, Some(SemanticSubKind::Cites), "urn:alice");
    assertion(
        &mut graph,
        a,
        b,
        Some(SemanticSubKind::UserGrouped),
        "urn:bob",
    );
    let hidden = |_, _, selector| selector == RelationSelector::Semantic(SemanticSubKind::Cites);
    let pair = inspect_pair(&graph, a, b, hidden).unwrap();
    assert_eq!((pair.held_rows, pair.visible_rows), (2, 1));
    let ak = graph.get_node_key_by_id(a).unwrap();
    let bk = graph.get_node_key_by_id(b).unwrap();
    assert_eq!(
        visible_edges(&graph, hidden)
            .iter()
            .filter(|(from, to)| (*from == ak && *to == bk) || (*from == bk && *to == ak))
            .count(),
        pair.visible_rows
    );
    assert_eq!(pair.cells.iter().filter(|cell| cell.hidden).count(), 1);
}
#[test]
fn repeated_accesses_share_owner_but_keep_displayed_pair() {
    let (mut graph, [a, b, c, d]) = fixture();
    assertion(&mut graph, a, b, Some(SemanticSubKind::Cites), "urn:alice");
    let ab = inspect_pair(&graph, a, b, |_, _, _| false).unwrap();
    let cd = inspect_pair(&graph, c, d, |_, _, _| false).unwrap();
    assert_ne!(ab.cells[0].owner.from, cd.cells[0].owner.from);
    assert_eq!(ab.cells[0].owner.source_from, cd.cells[0].owner.source_from);
    assert_eq!(ab.cells[0].assertions, cd.cells[0].assertions);
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplaySetShownResourceById {
            surface_id: c,
            resource_id: None,
        },
    );
    assert_eq!(
        inspect_pair(&graph, c, d, |_, _, _| false)
            .unwrap()
            .held_rows,
        0
    );
    assert_eq!(
        inspect_pair(&graph, a, b, |_, _, _| false)
            .unwrap()
            .held_rows,
        1
    );
}
#[test]
fn parallel_buckets_preserve_input_multiplicity_and_assertion_handles() {
    let (mut graph, [a, b, _, _]) = fixture();
    let id = assertion(&mut graph, a, b, Some(SemanticSubKind::Cites), "urn:alice");
    let pair = inspect_pair(&graph, a, b, |_, _, _| false).unwrap();
    let owner = &pair.cells[0].owner;
    let edge = graph.to_snapshot().resource_edges.remove(0);
    let mut second = edge.clone();
    second.assertions.as_mut().unwrap()[0].statement_id = "urn:parallel".into();
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplaySetResourceEdgesByIds {
            from_resource_id: owner.source_from,
            to_resource_id: owner.source_to,
            edges: vec![edge, second],
        },
    );
    let pair = inspect_pair(&graph, a, b, |_, _, _| false).unwrap();
    assert_eq!(pair.cells.len(), 1);
    assert_eq!(pair.visible_rows, 2);
    assert_eq!(
        pair.cells[0].assertions,
        BTreeSet::from([id, "urn:parallel".into()])
    );
    assert_eq!(
        inspect_pair(&graph, a, b, |_, _, _| true)
            .unwrap()
            .visible_rows,
        0
    );
}
#[test]
fn open_predicates_keep_handles_without_recognized_kind_invention() {
    let (mut graph, [a, b, _, _]) = fixture();
    let id = assertion(&mut graph, a, b, None, "urn:alice");
    let pair = inspect_pair(&graph, a, b, |_, _, _| false).unwrap();
    assert_eq!(
        pair.cells[0].owner.selector,
        RelationSelector::Family(EdgeFamily::Semantic)
    );
    assert_eq!(pair.cells[0].assertions, BTreeSet::from([id]));
}

#[test]
fn exact_same_kind_containment_keeps_independent_hide_handles() {
    let (mut graph, [a, b, _, _]) = fixture();
    let predicate = RelationKind::Containment(ContainmentSubKind::CollectionMember)
        .predicate_iri()
        .expect("canonical containment predicate")
        .to_owned();
    for (id, author) in [
        ("member-claim-one", "urn:author:one"),
        ("member-claim-two", "urn:author:two"),
    ] {
        graph
            .try_assert_persisted_relation_statement(
                graph.get_node_key_by_id(a).unwrap(),
                graph.get_node_key_by_id(b).unwrap(),
                SemanticStatement {
                    statement_id: id.into(),
                    predicate: predicate.clone(),
                    recognized_sub_kind: None,
                    label: None,
                    basis: Default::default(),
                    graph_scope: GraphScope::Default,
                    provenance_iri: Some(author.into()),
                    first_asserted_ms: Some(1),
                    asserted_at_ms: Some(2),
                    withdrawn: None,
                },
            )
            .unwrap();
    }
    let pair = inspect_pair(&graph, a, b, |_, _, _| false).unwrap();
    assert!(pair.held_rows >= 2);
    assert!(pair.cells.iter().all(|cell| cell.owner.selector
        == RelationSelector::Containment(ContainmentSubKind::CollectionMember)));
    let ids = pair
        .cells
        .iter()
        .flat_map(|cell| cell.assertions.iter().cloned())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        ids,
        BTreeSet::from(["member-claim-one".into(), "member-claim-two".into()])
    );
    let exact_id_hide_candidate = BTreeSet::from(["member-claim-one".to_owned()]);
    assert_eq!(exact_id_hide_candidate.len(), 1);
    assert!(ids.is_superset(&exact_id_hide_candidate));

    graph.retract_assertion("member-claim-one");
    let held = inspect_pair(&graph, a, b, |_, _, _| false).unwrap();
    assert!(held.held_rows >= 1);
    assert_eq!(
        held.cells
            .iter()
            .flat_map(|cell| cell.assertions.iter().cloned())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["member-claim-two".into()])
    );
}
#[test]
fn directions_strata_and_self_links_do_not_collapse() {
    let (mut graph, [a, b, _, _]) = fixture();
    assertion(&mut graph, a, b, Some(SemanticSubKind::Cites), "urn:alice");
    assertion(
        &mut graph,
        b,
        a,
        Some(SemanticSubKind::UserGrouped),
        "urn:bob",
    );
    let pair = inspect_pair(&graph, b, a, |_, _, _| false).unwrap();
    assert_eq!(pair.held_rows, 2);
    assert!(
        pair.cells
            .iter()
            .any(|cell| cell.owner.stratum == GraphStratum::Surface)
    );
    assert!(
        pair.cells
            .iter()
            .any(|cell| cell.owner.stratum == GraphStratum::Resource)
    );
    assertion(&mut graph, a, a, Some(SemanticSubKind::Cites), "urn:self");
    assert_eq!(
        inspect_pair(&graph, a, a, |_, _, _| false)
            .unwrap()
            .held_rows,
        1
    );
}
#[test]
fn absent_access_refuses_instead_of_retargeting_equal_address() {
    let (mut graph, [a, b, c, _]) = fixture();
    let key = graph.get_node_key_by_id(a).unwrap();
    apply_graph_delta(&mut graph, GraphDelta::RemoveNode { key });
    assert!(inspect_pair(&graph, a, b, |_, _, _| false).is_err());
    assert!(inspect_pair(&graph, c, b, |_, _, _| false).is_ok());
}

#[test]
fn shared_bucket_keeps_every_kind_and_repeated_access_direction() {
    let (mut graph, [a, _, c, _]) = fixture();
    // a and c show the same Resource. Its self-link lifts to a->c and c->a.
    let cites: BTreeSet<_> = (0..32)
        .map(|i| {
            assertion(
                &mut graph,
                a,
                a,
                Some(SemanticSubKind::Cites),
                &format!("urn:author:{i}"),
            )
        })
        .collect();
    let open = assertion(&mut graph, a, a, None, "urn:open-author");
    let grouped = assertion(
        &mut graph,
        a,
        c,
        Some(SemanticSubKind::UserGrouped),
        "urn:group-author",
    );
    let before = serde_json::to_value(graph.to_snapshot()).unwrap();
    let pair = inspect_pair(&graph, a, c, |_, _, _| false).unwrap();
    assert_eq!((pair.held_rows, pair.visible_rows), (67, 67));
    for (from, to) in [(a, c), (c, a)] {
        for (selector, handles) in [
            (
                RelationSelector::Semantic(SemanticSubKind::Cites),
                cites.clone(),
            ),
            (
                RelationSelector::Family(EdgeFamily::Semantic),
                BTreeSet::from([open.clone()]),
            ),
        ] {
            let cell = pair
                .cells
                .iter()
                .find(|cell| {
                    cell.owner.from == from
                        && cell.owner.to == to
                        && cell.owner.selector == selector
                })
                .unwrap();
            assert_eq!(cell.owner.stratum, GraphStratum::Resource);
            assert_eq!(cell.assertions, handles);
            assert_eq!(cell.rows, handles.len());
        }
    }
    let cell = pair
        .cells
        .iter()
        .find(|cell| cell.owner.stratum == GraphStratum::Surface)
        .unwrap();
    assert_eq!(cell.assertions, BTreeSet::from([grouped]));
    assert_eq!(serde_json::to_value(graph.to_snapshot()).unwrap(), before);
}
