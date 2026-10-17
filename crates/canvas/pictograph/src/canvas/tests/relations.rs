// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Link-key hit testing and compatibility behavior for exact directed filters.

use super::*;

#[test]
fn parallel_assertions_share_one_stroke_and_one_link_key_hit_target() {
    let mut canvas = Canvas::new();
    let a = canvas.open_member_as_new_node(None, "https://fan-a.test");
    let b = canvas.open_member_as_new_node(None, "https://fan-b.test");
    let ak = canvas.graph().get_node_by_id(a).unwrap().0;
    let bk = canvas.graph().get_node_by_id(b).unwrap().0;
    assert!(canvas.assert_relation_between_members(a, b, SemanticSubKind::Cites));
    assert!(canvas.assert_relation_between_members(a, b, SemanticSubKind::Quotes));
    canvas
        .view
        .set_position(ak, euclid::default::Point2D::new(0.0, 0.0));
    canvas
        .view
        .set_position(bk, euclid::default::Point2D::new(100.0, 0.0));

    let key = scenograph::link::LinkKey::new(
        scenograph::link::LinkStratum::Resource,
        canvas
            .graph()
            .resource(canvas.graph().shown_resource_id(ak).unwrap())
            .unwrap()
            .canonical_iri(),
        canvas
            .graph()
            .resource(canvas.graph().shown_resource_id(bk).unwrap())
            .unwrap()
            .canonical_iri(),
    );
    let segments: Vec<_> = crate::canvas::edge_cells::visible_link_appearances(
        canvas.graph(),
        &canvas.view,
        &canvas.hidden_links,
        &canvas.hidden_edges,
        |_| 36.0,
    )
    .into_iter()
    .filter(|segment| segment.appearance.key == key)
    .collect();
    assert_eq!(
        segments.len(),
        1,
        "parallel assertions produce one resting stroke"
    );

    let before = serde_json::to_value(canvas.graph().to_snapshot_at(0)).unwrap();
    let segment = &segments[0];
    let midpoint = euclid::default::Point2D::new(
        (segment.from.x + segment.to.x) * 0.5,
        (segment.from.y + segment.to.y) * 0.5,
    );
    assert_eq!(
        crate::canvas::edge_cells::link_hit_test(
            canvas.graph(),
            &canvas.view,
            &canvas.hidden_links,
            &canvas.hidden_edges,
            midpoint,
            1.0,
            |_| true,
            |_| 36.0,
        )
        .map(|appearance| appearance.key),
        Some(key.clone()),
        "stroke hit resolves the canonical link identity"
    );
    canvas.selected_links.insert(key.clone());
    assert_eq!(
        canvas.selected_link_keys(),
        vec![key.clone()],
        "parallel assertions produce one open target"
    );
    assert_eq!(
        canvas.link_appearance_pair(&key),
        Some((a, b)),
        "a Resource key resolves its actual displayed appearance"
    );
    assert_eq!(
        serde_json::to_value(canvas.graph().to_snapshot_at(0)).unwrap(),
        before,
        "opening a lifted link leaves its source assertions unchanged"
    );
}

#[test]
fn exact_assertion_hides_filter_before_aggregation_and_preserve_other_hide_sets() {
    let mut canvas = Canvas::new();
    let a = canvas.open_member_as_new_node(None, "https://hide-assertion-a.test");
    let b = canvas.open_member_as_new_node(None, "https://hide-assertion-b.test");
    let ak = canvas.graph().get_node_by_id(a).unwrap().0;
    let bk = canvas.graph().get_node_by_id(b).unwrap().0;
    assert!(canvas.assert_relation_between_members(a, b, SemanticSubKind::Cites));
    assert!(canvas.assert_relation_between_members(a, b, SemanticSubKind::Quotes));
    canvas
        .view
        .set_position(ak, euclid::default::Point2D::new(0.0, 0.0));
    canvas
        .view
        .set_position(bk, euclid::default::Point2D::new(100.0, 0.0));
    let before = serde_json::to_value(canvas.graph().to_snapshot_at(0)).unwrap();
    let pair = crate::canvas::link_reading::read_link(
        canvas.graph(),
        a,
        b,
        scenograph::link::LinkSource::Unavailable {
            reason: "test source omitted".into(),
        },
    )
    .unwrap();
    let ids = pair
        .links
        .iter()
        .flat_map(|link| &link.buckets)
        .flat_map(|bucket| &bucket.assertions)
        .map(|assertion| assertion.id.clone())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), 2);
    let hidden_link = scenograph::link::LinkKey::new(
        scenograph::link::LinkStratum::Surface,
        a.to_string(),
        b.to_string(),
    );
    canvas.set_hidden_link_keys([hidden_link.clone()]);
    canvas.set_hidden_assertion_ids([ids.iter().next().unwrap().clone()]);
    assert_eq!(canvas.hidden_link_keys(), vec![hidden_link]);
    assert_eq!(canvas.hidden_assertion_ids().len(), 1);
    assert!(
        canvas
            .visible_link_appearances()
            .iter()
            .all(|appearance| appearance.key.stratum != scenograph::link::LinkStratum::Surface)
    );
    assert!(
        canvas.visible_link_appearances().iter().any(|appearance| {
            appearance.key.stratum == scenograph::link::LinkStratum::Resource
        }),
        "the unhidden exact assertion still contributes a visible resource appearance"
    );

    canvas.set_hidden_assertion_ids(ids.clone());
    assert!(
        !canvas.visible_link_appearances().iter().any(|appearance| {
            appearance.key.stratum == scenograph::link::LinkStratum::Resource
        })
    );
    assert_eq!(canvas.hidden_link_keys().len(), 1);
    assert_eq!(canvas.show_all_assertions(), 2);
    assert_eq!(canvas.hidden_link_keys().len(), 1);
    assert_eq!(
        serde_json::to_value(canvas.graph().to_snapshot_at(0)).unwrap(),
        before
    );
}

#[test]
fn selector_hides_classify_exact_nonsemantic_assertions_by_canonical_predicate() {
    let mut canvas = Canvas::new();
    let a = canvas.open_member_as_new_node(None, "https://containment-hide-a.test");
    let b = canvas.open_member_as_new_node(None, "https://containment-hide-b.test");
    let ak = canvas.graph().get_node_by_id(a).unwrap().0;
    let bk = canvas.graph().get_node_by_id(b).unwrap().0;
    let predicate = RelationKind::Containment(kernel::graph::ContainmentSubKind::CollectionMember)
        .predicate_iri()
        .unwrap();
    canvas.ingest_graph(|graph| {
        graph
            .try_assert_persisted_relation_statement(
                ak,
                bk,
                kernel::graph::SemanticStatement {
                    statement_id: "urn:containment:hide-test".into(),
                    predicate: predicate.into(),
                    recognized_sub_kind: None,
                    label: None,
                    basis: kernel::types::AssertionBasis::default(),
                    graph_scope: kernel::types::GraphScope::Default,
                    provenance_iri: Some("urn:author:test".into()),
                    first_asserted_ms: Some(1),
                    asserted_at_ms: Some(1),
                    withdrawn: None,
                },
            )
            .unwrap();
        true
    });
    let before = canvas.visible_link_appearances();
    assert!(!before.is_empty());
    let cell = first_edge_cell_between(&canvas, ak, bk);
    assert_eq!(
        cell.selector,
        RelationSelector::Containment(kernel::graph::ContainmentSubKind::CollectionMember)
    );
    canvas.hidden_edges.insert(cell);
    assert!(canvas.visible_link_appearances().is_empty());
}

#[test]
fn self_links_draw_separate_strata_loops_and_select_through_node_pick_area() {
    use scenograph::link::LinkStratum;

    let mut canvas = Canvas::new();
    let member = canvas.open_member_as_new_node(None, "https://self-link.test");
    let node = canvas.graph().get_node_by_id(member).unwrap().0;
    assert!(canvas.assert_relation_between_members(member, member, SemanticSubKind::Cites));
    assert!(canvas.assert_relation_between_members(member, member, SemanticSubKind::UserGrouped));
    canvas
        .view
        .set_position(node, euclid::default::Point2D::new(200.0, 200.0));
    canvas.set_camera(CameraView {
        offset: (0.0, 0.0),
        zoom: 1.0,
    });

    let segments = crate::canvas::edge_cells::visible_link_appearances(
        canvas.graph(),
        &canvas.view,
        &canvas.hidden_links,
        &canvas.hidden_edges,
        |key| canvas.node_size(key),
    );
    let self_links: Vec<_> = segments
        .iter()
        .filter(|segment| segment.appearance.display_from == node)
        .collect();
    assert_eq!(
        self_links.len(),
        2,
        "Surface and Resource self-links remain distinct"
    );
    assert!(
        self_links
            .iter()
            .any(|link| link.appearance.key.stratum == LinkStratum::Surface)
    );
    assert!(
        self_links
            .iter()
            .any(|link| link.appearance.key.stratum == LinkStratum::Resource)
    );
    assert!(self_links.iter().all(|link| link.points.len() > 2));
    assert_ne!(self_links[0].points[4], self_links[1].points[4]);
    let node_size = canvas.node_size(node);
    let boundary_y = canvas.view.position_of(node).unwrap().y - node_size * 0.5;
    let min_x = canvas.view.position_of(node).unwrap().x - node_size * 0.5;
    let max_x = canvas.view.position_of(node).unwrap().x + node_size * 0.5;
    for segment in &self_links {
        for endpoint in [
            segment.points.first().unwrap(),
            segment.points.last().unwrap(),
        ] {
            assert!(
                (endpoint.y - boundary_y).abs() < 0.001,
                "self-link endpoint must meet the top boundary: {endpoint:?} vs {boundary_y}"
            );
            assert!(
                endpoint.x >= min_x - 0.001 && endpoint.x <= max_x + 0.001,
                "self-link endpoint must stay within the node's horizontal bounds: {endpoint:?} not in [{min_x}, {max_x}]"
            );
        }
    }

    for segment in self_links {
        let pick = segment.points[5];
        let selected = crate::canvas::edge_cells::link_hit_test(
            canvas.graph(),
            &canvas.view,
            &canvas.hidden_links,
            &canvas.hidden_edges,
            pick,
            1.0,
            |_| true,
            |key| canvas.node_size(key),
        )
        .expect("loop stroke has a geometric hit target");
        assert_eq!(selected.key, segment.appearance.key);

        canvas.pointer_down(PointerButton::Left, pick.x, pick.y);
        assert!(canvas.pointer_up(PointerButton::Left, pick.x, pick.y));
        assert_eq!(
            canvas.selected_link_keys(),
            vec![segment.appearance.key.clone()]
        );
        assert!(
            canvas.selected_members().is_empty(),
            "the loop beats its node's pick area"
        );
    }
}

#[test]
fn closing_a_link_hides_all_parallel_assertions_under_one_key() {
    let mut canvas = Canvas::new();
    let a = canvas.open_member_as_new_node(None, "https://closed-a.test");
    let b = canvas.open_member_as_new_node(None, "https://closed-b.test");
    let ak = canvas.graph().get_node_by_id(a).unwrap().0;
    let bk = canvas.graph().get_node_by_id(b).unwrap().0;
    assert!(canvas.assert_relation_between_members(a, b, SemanticSubKind::Cites));
    assert!(canvas.assert_relation_between_members(a, b, SemanticSubKind::Quotes));
    canvas
        .view
        .set_position(ak, euclid::default::Point2D::new(0.0, 0.0));
    canvas
        .view
        .set_position(bk, euclid::default::Point2D::new(100.0, 0.0));
    let key = scenograph::link::LinkKey::new(
        scenograph::link::LinkStratum::Resource,
        canvas
            .graph()
            .resource(canvas.graph().shown_resource_id(ak).unwrap())
            .unwrap()
            .canonical_iri(),
        canvas
            .graph()
            .resource(canvas.graph().shown_resource_id(bk).unwrap())
            .unwrap()
            .canonical_iri(),
    );
    canvas.selected_links.insert(key.clone());
    assert_eq!(canvas.hide_selected_links(), 1);
    assert!(
        crate::canvas::edge_cells::visible_link_appearances(
            canvas.graph(),
            &canvas.view,
            &canvas.hidden_links,
            &canvas.hidden_edges,
            |_| 36.0,
        )
        .into_iter()
        .all(|segment| segment.appearance.key != key)
    );
    assert_eq!(canvas.graph().projected_relations().count(), 2);
}

#[test]
fn legacy_member_filters_reduce_one_link_summary_without_becoming_link_identity() {
    let mut canvas = Canvas::new();
    let a = canvas.open_member_as_new_node(None, "https://legacy-filter-a.test");
    let b = canvas.open_member_as_new_node(None, "https://legacy-filter-b.test");
    let ak = canvas.graph().get_node_by_id(a).unwrap().0;
    let bk = canvas.graph().get_node_by_id(b).unwrap().0;
    assert!(canvas.assert_relation_between_members(a, b, SemanticSubKind::Cites));
    assert!(canvas.assert_relation_between_members(a, b, SemanticSubKind::Quotes));
    canvas
        .view
        .set_position(ak, euclid::default::Point2D::new(0.0, 0.0));
    canvas
        .view
        .set_position(bk, euclid::default::Point2D::new(100.0, 0.0));
    let key = scenograph::link::LinkKey::new(
        scenograph::link::LinkStratum::Resource,
        canvas
            .graph()
            .resource(canvas.graph().shown_resource_id(ak).unwrap())
            .unwrap()
            .canonical_iri(),
        canvas
            .graph()
            .resource(canvas.graph().shown_resource_id(bk).unwrap())
            .unwrap()
            .canonical_iri(),
    );
    let (from, to) = canvas
        .graph()
        .projected_relations()
        .find_map(|(_, row)| {
            (row.kind == RelationKind::Semantic(SemanticSubKind::Cites))
                .then_some((row.from, row.to))
        })
        .unwrap();
    let hidden = HashSet::from([EdgeCell {
        from,
        to,
        selector: RelationSelector::Semantic(SemanticSubKind::Cites),
    }]);
    let segments = crate::canvas::edge_cells::visible_link_appearances(
        canvas.graph(),
        &canvas.view,
        &HashSet::new(),
        &hidden,
        |_| 36.0,
    );
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].appearance.key, key);
    assert_eq!(segments[0].appearance.strength, 1.0);
}

#[test]
fn antiparallel_and_opposed_assertions_become_marks_on_one_resting_stroke() {
    let mut canvas = Canvas::new();
    let a = canvas.open_member_as_new_node(None, "https://marks-a.test");
    let b = canvas.open_member_as_new_node(None, "https://marks-b.test");
    let ak = canvas.graph().get_node_by_id(a).unwrap().0;
    let bk = canvas.graph().get_node_by_id(b).unwrap().0;
    canvas.assert_relation_between_members(a, b, SemanticSubKind::Supports);
    canvas.assert_relation_between_members(b, a, SemanticSubKind::Contradicts);
    canvas
        .view
        .set_position(ak, euclid::default::Point2D::new(0.0, 0.0));
    canvas
        .view
        .set_position(bk, euclid::default::Point2D::new(100.0, 0.0));
    let segments = crate::canvas::edge_cells::visible_link_appearances(
        canvas.graph(),
        &canvas.view,
        &HashSet::new(),
        &HashSet::new(),
        |_| 36.0,
    );
    assert_eq!(segments.len(), 1, "two directions share one link stroke");
    assert!(segments[0].appearance.marks.forward);
    assert!(segments[0].appearance.marks.reverse);
    assert!(segments[0].appearance.marks.opposed);
}

#[test]
fn retract_selected_relation_removes_the_user_relation() {
    let mut canvas = Canvas::new();
    let a = canvas.open_member_as_new_node(None, "https://a.test");
    let b = canvas.open_member_as_new_node(None, "https://b.test");
    let ak = canvas.graph().get_node_by_id(a).unwrap().0;
    let bk = canvas.graph().get_node_by_id(b).unwrap().0;
    canvas.selected.clear();
    canvas.selected.insert(ak);
    canvas.selected.insert(bk);
    canvas.assert_selected_relation(SemanticSubKind::UserGrouped);
    let with_edge = canvas.graph().relations().count();

    assert_eq!(
        canvas.retract_selected_relation(),
        1,
        "the user relation is retracted"
    );
    assert_eq!(
        canvas.graph().relations().count(),
        with_edge - 1,
        "the edge is gone (no families left → garbage-collected)",
    );
    assert!(
        canvas.selected.len() == 2,
        "pairwise retraction leaves the node selection intact"
    );
}

#[test]
fn explicit_member_retraction_removes_only_that_relation() {
    let mut canvas = Canvas::new();
    let a = canvas.open_member_as_new_node(None, "https://cell-a.test");
    let b = canvas.open_member_as_new_node(None, "https://cell-b.test");
    assert!(canvas.assert_relation_between_members(a, b, SemanticSubKind::Cites));
    assert!(canvas.assert_relation_between_members(a, b, SemanticSubKind::Quotes));
    assert_eq!(
        canvas.retract_relation_between_members(
            a,
            b,
            RelationSelector::Semantic(SemanticSubKind::Cites),
        ),
        1
    );
    let remaining: Vec<_> = canvas
        .graph()
        .projected_relations()
        .map(|(_, r)| r.kind)
        .collect();
    assert_eq!(
        remaining,
        vec![RelationKind::Semantic(SemanticSubKind::Quotes)],
        "the other semantic cell survives the canvas-cell delete"
    );
}

#[test]
fn legacy_directed_filter_hides_only_that_member_under_one_link() {
    let mut canvas = Canvas::new();
    let a = canvas.open_member_as_new_node(None, "https://hide-cell-a.test");
    let b = canvas.open_member_as_new_node(None, "https://hide-cell-b.test");
    assert!(canvas.assert_relation_between_members(a, b, SemanticSubKind::Cites));
    assert!(canvas.assert_relation_between_members(a, b, SemanticSubKind::Quotes));
    assert!(canvas.hide_relation_between_members(
        a,
        b,
        RelationSelector::Semantic(SemanticSubKind::Cites)
    ));
    assert!(canvas.relation_between_members_hidden(
        a,
        b,
        RelationSelector::Semantic(SemanticSubKind::Cites)
    ));
    assert!(!canvas.relation_between_members_hidden(
        a,
        b,
        RelationSelector::Semantic(SemanticSubKind::Quotes)
    ));
    assert!(
        !canvas.edge_between_members_hidden(a, b),
        "the endpoint bundle is not fully hidden while another cell remains visible"
    );
    let visible: Vec<_> = crate::canvas::edge_cells::visible_link_appearances(
        canvas.graph(),
        &canvas.view,
        &canvas.hidden_links,
        &canvas.hidden_edges,
        |_| 36.0,
    )
    .into_iter()
    .filter(|segment| {
        let from = canvas.graph().get_node_by_id(a).unwrap().0;
        let to = canvas.graph().get_node_by_id(b).unwrap().0;
        (segment.appearance.display_from == from && segment.appearance.display_to == to)
            || (segment.appearance.display_from == to && segment.appearance.display_to == from)
    })
    .collect();
    assert_eq!(
        visible.len(),
        1,
        "the remaining assertion keeps one link visible"
    );
    assert_eq!(visible[0].appearance.strength, 1.0);
}

#[test]
fn unrelate_is_symmetric_with_relate_on_a_two_node_selection() {
    // `>unrelate` mirrors `>relate`: the same two-node selection that related a
    // pair also unrelates it, without having to click the edge.
    let mut canvas = Canvas::new();
    let a = canvas.open_member_as_new_node(None, "https://a.test");
    let b = canvas.open_member_as_new_node(None, "https://b.test");
    let ak = canvas.graph().get_node_by_id(a).unwrap().0;
    let bk = canvas.graph().get_node_by_id(b).unwrap().0;
    let base = canvas.graph().relations().count();
    canvas.selected.clear();
    canvas.selected.insert(ak);
    canvas.selected.insert(bk);

    canvas.assert_selected_relation(SemanticSubKind::UserGrouped);
    assert_eq!(canvas.graph().relations().count(), base + 1, "related");

    // No edge selected — the two-node selection alone retracts the relation.
    assert!(canvas.selected_link_keys().is_empty());
    assert_eq!(
        canvas.retract_selected_relation(),
        1,
        "the pair selection unrelates"
    );
    assert_eq!(
        canvas.graph().relations().count(),
        base,
        "back to no relation"
    );
}

#[test]
fn visit_dedups_by_url() {
    let mut canvas = Canvas::new();
    let a = canvas.visit("https://example.com");
    let _ = canvas.visit("https://other.com");
    let again = canvas.visit("https://example.com");
    assert_eq!(
        a, again,
        "revisiting a URL selects the existing node, not a duplicate"
    );
    assert_eq!(
        canvas.graph.nodes().count(),
        2,
        "no duplicate node is added"
    );
    assert!(
        canvas.selected.contains(&a),
        "selection returns to the revisited node"
    );
}

#[test]
fn focused_url_is_the_single_selected_nodes_url() {
    let mut canvas = Canvas::new();
    assert_eq!(
        canvas.focused_url(),
        None,
        "nothing selected yet → no focus"
    );
    canvas.visit("https://example.com");
    assert_eq!(
        canvas.focused_url(),
        Some("https://example.com"),
        "visit focuses the node"
    );
    canvas.visit("https://second.com");
    assert_eq!(
        canvas.focused_url(),
        Some("https://second.com"),
        "focus follows the new node"
    );
    canvas.visit("https://example.com");
    assert_eq!(
        canvas.focused_url(),
        Some("https://example.com"),
        "revisit re-focuses"
    );
}
