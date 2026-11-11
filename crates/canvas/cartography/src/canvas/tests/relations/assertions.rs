// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

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
