/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

#[test]
fn paint_leaf_and_hit_targets_share_projection() {
    let swatch = model(Some(1), Some(2));
    let leaf = swatch.paint_leaf(|kind| match *kind {
        "document" => ColorF {
            r: 0.2,
            g: 0.4,
            b: 0.8,
            a: 1.0,
        },
        _ => ColorF {
            r: 0.7,
            g: 0.4,
            b: 0.3,
            a: 1.0,
        },
    });
    let projected = swatch.projected_positions();
    assert_eq!(
        leaf.node_local_position(
            0,
            Size {
                width: 240.0,
                height: 112.0
            }
        ),
        Some(projected[0].1)
    );
}

#[test]
fn node_regions_share_projection_and_clamp_inside_the_canvas() {
    let swatch = model(None, None);
    let projected = swatch.projected_positions();
    let region = swatch
        .projected_node_region(&1, 120.0, 80.0)
        .expect("node region");
    let center = projected[0].1;
    assert!(center.0 >= region.left && center.0 <= region.left + region.width);
    assert!(center.1 >= region.top && center.1 <= region.top + region.height);
    assert!(region.left >= 0.0 && region.top >= 0.0);
    assert!(region.left + region.width <= swatch.width as f32);
    assert!(region.top + region.height <= swatch.height as f32);

    let full = swatch
        .projected_node_region(&2, 10_000.0, 10_000.0)
        .expect("oversized region");
    assert_eq!(
        full,
        GraphCanvasNodeRegion {
            left: 0.0,
            top: 0.0,
            width: swatch.width as f32,
            height: swatch.height as f32,
        }
    );
    assert!(swatch.projected_node_region(&99, 10.0, 10.0).is_none());
}

#[test]
fn node_footprints_clip_incident_relation_routes_to_the_card_perimeter() {
    let relations = vec![
        GraphCanvasRelation {
            link_key: None,
            marks: sceno::LinkMarks::default(),
            strength: 1.0,
            id: "outgoing".into(),
            from: 1,
            to: 2,
            kind: "First".into(),
            label: "Outgoing relation".into(),
            route: Vec::new(),
            visible: true,
            emphasized: false,
        },
        GraphCanvasRelation {
            link_key: None,
            marks: sceno::LinkMarks::default(),
            strength: 1.0,
            id: "incoming".into(),
            from: 2,
            to: 1,
            kind: "Second".into(),
            label: "Incoming relation".into(),
            route: Vec::new(),
            visible: true,
            emphasized: false,
        },
    ];
    let compact = model(None, None).with_relations(relations.clone());
    let compact_routes = compact.projected_relations();
    let swatch = model(None, None)
        .with_relations(relations)
        .with_node_footprint(1, 80.0, 60.0);
    let region = swatch
        .projected_node_footprint(&1)
        .expect("declared footprint");
    let routes = swatch.projected_relations();
    let on_perimeter = |point: (f32, f32)| {
        let right = region.left + region.width;
        let bottom = region.top + region.height;
        let on_vertical = (point.0 - region.left).abs() < 0.01 || (point.0 - right).abs() < 0.01;
        let on_horizontal = (point.1 - region.top).abs() < 0.01 || (point.1 - bottom).abs() < 0.01;
        (on_vertical && point.1 >= region.top && point.1 <= bottom)
            || (on_horizontal && point.0 >= region.left && point.0 <= right)
    };

    assert_eq!(routes.len(), 2);
    assert!(on_perimeter(routes[0].1[0]), "outgoing source is clipped");
    assert!(
        on_perimeter(*routes[1].1.last().expect("incoming endpoint")),
        "incoming target is clipped"
    );
    assert_eq!(
        *routes[0].1.last().expect("unoccupied target"),
        *compact_routes[0].1.last().expect("compact target")
    );
    assert_eq!(routes[1].1[0], compact_routes[1].1[0]);
}

#[test]
fn graph_position_inverts_the_viewport_projection() {
    let mut swatch = model(None, None);
    swatch.viewport = GraphViewport {
        pan: (0.12, -0.08),
        zoom: 1.35,
    };
    let point = (0.73, 0.21);
    let size = Size {
        width: swatch.width as f32,
        height: swatch.height as f32,
    };
    let leaf = swatch
        .viewport
        .project(point, size, swatch.node_radius + swatch.edge_width);
    let recovered = swatch.graph_position_at(leaf);
    assert!((recovered.0 - point.0).abs() < 1e-5, "{recovered:?}");
    assert!((recovered.1 - point.1).abs() < 1e-5, "{recovered:?}");
}

#[test]
fn unbound_routes_keep_independent_targets_and_visibility() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner =
        GenetAppRunner::<_, _, _, ()>::new(dom.clone(), relation_view, State::default());
    let root = runner.root();
    let cites = find_attr(&dom.borrow(), root, "data-relation-id", "cites")
        .expect("citation relation target");
    let quotes = find_attr(&dom.borrow(), root, "data-relation-id", "quotes")
        .expect("quotation relation target");
    assert_ne!(cites, quotes, "parallel cells must get separate targets");
    assert!(
        find_attr(&dom.borrow(), root, "data-relation-id", "hidden").is_none(),
        "hidden cells must not remain hittable"
    );
    assert_eq!(attr(&dom.borrow(), cites, "aria-current"), Some("true"));
    assert_eq!(attr(&dom.borrow(), cites, "data-kind"), Some("Citation"));
    assert_eq!(attr(&dom.borrow(), quotes, "data-kind"), Some("Quotation"));

    runner.dispatch_hover(
        quotes,
        HoverEvent::new(HoverPhase::Enter, (2.0, 2.0), (20.0, 20.0)),
    );
    runner.dispatch_click(cites, PointerClick::at((2.0, 2.0)));
    assert_eq!(runner.state().relation_hovered.as_deref(), Some("quotes"));
    assert_eq!(runner.state().relation_clicked, ["cites"]);

    let swatch = model(None, None).with_relations(vec![GraphCanvasRelation {
        link_key: None,
        marks: sceno::LinkMarks::default(),
        strength: 1.0,
        id: "hidden".into(),
        from: 1,
        to: 2,
        kind: "Hidden".into(),
        label: "This relation is hidden".into(),
        route: Vec::new(),
        visible: false,
        emphasized: false,
    }]);
    assert!(swatch.projected_relations().is_empty());
}

#[test]
fn one_link_target_keeps_both_directions_opposition_and_strength_without_fanning() {
    let key = sceno::LinkKey::new(sceno::LinkStratum::Surface, "1", "2");
    let relation = |id: &str, from, to, marks| GraphCanvasRelation {
        id: id.into(),
        link_key: Some(key.clone()),
        marks,
        strength: 1.0,
        from,
        to,
        kind: "reason".into(),
        label: id.into(),
        route: Vec::new(),
        visible: true,
        emphasized: false,
    };
    let swatch = model(None, None).with_relations(vec![
        relation(
            "one",
            1,
            2,
            sceno::LinkMarks {
                forward: true,
                reverse: false,
                opposed: true,
            },
        ),
        relation(
            "two",
            2,
            1,
            sceno::LinkMarks {
                forward: true,
                reverse: false,
                opposed: false,
            },
        ),
    ]);
    assert_eq!(swatch.relations.len(), 1);
    let (link, route) = &swatch.projected_relations()[0];
    assert_eq!(link.id, key.stable_id());
    assert_eq!(
        link.marks,
        sceno::LinkMarks {
            forward: true,
            reverse: true,
            opposed: true
        }
    );
    assert_eq!(link.strength, 2.0);
    assert_eq!(
        route.len(),
        2,
        "assertion count does not bend the resting path"
    );
}

#[test]
fn stratum_strokes_remain_separate_without_assertion_lanes() {
    let relation = |stratum| GraphCanvasRelation {
        id: "ignored".into(),
        link_key: Some(sceno::LinkKey::new(stratum, "1", "2")),
        marks: sceno::LinkMarks {
            forward: true,
            ..Default::default()
        },
        strength: 1.0,
        from: 1,
        to: 2,
        kind: "link".into(),
        label: "source pair".into(),
        route: Vec::new(),
        visible: true,
        emphasized: false,
    };
    let swatch = model(None, None).with_relations(vec![
        relation(sceno::LinkStratum::Surface),
        relation(sceno::LinkStratum::Resource),
    ]);
    let routes = swatch.projected_relations();
    assert_eq!(routes.len(), 2);
    assert_ne!(routes[0].0.id, routes[1].0.id);
    assert!(
        (routes[0].1[1].1 - routes[1].1[1].1).abs() > 5.9,
        "both stratum strokes have distinct paint and pick paths"
    );
    let mut hidden = swatch.clone();
    hidden.relations[1].visible = false;
    assert_eq!(hidden.projected_relations().len(), 1);
    assert_eq!(hidden.projected_relations()[0].1.len(), 2);
}
