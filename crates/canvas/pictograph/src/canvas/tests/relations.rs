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
