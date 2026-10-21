// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The graph grammar's bounded scenes picked through the canvas: Partition,
//! Sets and Provenance.

use super::*;
use crate::canvas::physics_controls::{FREE_ARRANGEMENT, apply_arrangement};
use crate::canvas::scene_axes::SOURCES_COLUMN;
use kernel::graph::{EdgeAssertion, ProvenanceSubKind};

const VIEW: (u32, u32) = (800, 600);

fn scene_graph() -> (Graph, Vec<NodeKey>) {
    let mut graph = Graph::new();
    let keys: Vec<NodeKey> = [
        "https://woodshed.example/progression",
        "https://woodshed.example/take",
        "https://knot.example/lyrics",
        "https://search.example/chords",
    ]
    .iter()
    .map(|url| graph.add_node(url.to_string(), PortablePoint::zero()))
    .collect();
    graph.assert_relation(
        keys[0],
        keys[3],
        EdgeAssertion::Provenance {
            sub_kind: ProvenanceSubKind::ExtractedFrom,
        },
    );
    graph.assert_relation(
        keys[2],
        keys[0],
        EdgeAssertion::Provenance {
            sub_kind: ProvenanceSubKind::RewrittenFrom,
        },
    );
    (graph, keys)
}

fn canvas() -> (Canvas, Vec<NodeKey>) {
    let (graph, keys) = scene_graph();
    let mut canvas = Canvas::with_graph(graph);
    for (key, tags) in [
        (keys[0], ["song", "practice"]),
        (keys[2], ["song", "publishing"]),
    ] {
        let member = canvas.graph().get_node(key).unwrap().id;
        for tag in tags {
            assert!(canvas.tag_node(member, tag));
        }
    }
    canvas.resize(VIEW.0, VIEW.1);
    (canvas, keys)
}

fn positions(canvas: &mut Canvas, id: &str) -> HashMap<NodeKey, PortablePoint> {
    canvas
        .project_arrangement_for_view(id, VIEW.0, VIEW.1, None, true, 1.0, None)
        .positions
        .into_iter()
        .collect()
}

#[test]
fn partition_subdivides_the_viewport_by_site() {
    let (mut canvas, keys) = canvas();
    let placed = positions(&mut canvas, "partition.default");
    for key in &keys {
        let p = placed[key];
        assert!(
            p.x.abs() <= 400.0 && p.y.abs() <= 300.0,
            "inside the viewport: {p:?}"
        );
    }
    apply_arrangement(&mut canvas, "partition.default", VIEW).unwrap();
    let mut labels: Vec<_> = canvas
        .strategy_regions()
        .iter()
        .filter_map(|r| r.label.clone())
        .collect();
    labels.sort();
    assert_eq!(
        labels,
        ["knot.example", "search.example", "woodshed.example"]
    );
    let woodshed = canvas
        .strategy_regions()
        .iter()
        .find(|r| r.label.as_deref() == Some("woodshed.example"))
        .unwrap();
    assert_eq!(woodshed.members.len(), 2);
    assert_eq!(canvas.strategy_region_commands().len(), 3);

    apply_arrangement(&mut canvas, FREE_ARRANGEMENT, VIEW).unwrap();
    assert!(
        canvas.strategy_regions().is_empty(),
        "regions leave with the arrangement"
    );
}

#[test]
fn sets_draw_the_common_tags_and_place_by_membership() {
    let (mut canvas, keys) = canvas();
    apply_arrangement(&mut canvas, "sets.default", VIEW).unwrap();
    let sets: Vec<_> = canvas
        .strategy_regions()
        .iter()
        .filter_map(|r| r.label.clone())
        .filter(|label| !label.ends_with("· core"))
        .collect();
    assert_eq!(
        sets,
        ["song", "practice", "publishing"],
        "most common first"
    );
    let song = &canvas.strategy_regions()[0];
    assert!(song.members.contains(&keys[0]) && song.members.contains(&keys[2]));
    assert!(!song.members.contains(&keys[1]), "untagged sits outside");
}

#[test]
fn sets_read_public_resource_tags_without_legacy_surface_tags() {
    let mut canvas = Canvas::new();
    let members = [
        canvas.open_member_as_new_node(None, "https://songs.example/shared"),
        canvas.open_member_as_new_node(None, "https://songs.example/shared"),
        canvas.open_member_as_new_node(None, "https://songs.example/other"),
    ];
    let keys: Vec<_> = members
        .iter()
        .map(|member| canvas.graph().get_node_by_id(*member).unwrap().0)
        .collect();
    assert_ne!(members[0], members[1]);
    assert_eq!(
        canvas.graph().shown_resource_id(keys[0]),
        canvas.graph().shown_resource_id(keys[1])
    );
    assert!(canvas.tag_node(members[0], "song"));
    assert!(canvas.tag_node(members[0], "practice"));
    assert!(canvas.tag_node(members[2], "song"));
    for key in &keys {
        assert!(canvas.graph().node_tags(*key).unwrap().is_empty());
        assert!(
            canvas
                .graph()
                .node_content_tags(*key)
                .unwrap()
                .contains("song")
        );
    }
    canvas.resize(VIEW.0, VIEW.1);
    apply_arrangement(&mut canvas, "sets.default", VIEW).unwrap();
    let song = canvas
        .strategy_regions()
        .iter()
        .find(|region| region.label.as_deref() == Some("song"))
        .expect("Sets must read canonical Resource tags written through Canvas");
    assert!(keys.iter().all(|key| song.members.contains(key)));
    let practice = canvas
        .strategy_regions()
        .iter()
        .find(|region| region.label.as_deref() == Some("practice"))
        .expect("each canonical membership produces its bounded region");
    assert_eq!(practice.members.len(), 2);
    assert!(practice.members.contains(&keys[0]) && practice.members.contains(&keys[1]));
    assert!(!practice.members.contains(&keys[2]));
}

#[test]
fn provenance_stands_sources_left() {
    let (mut canvas, keys) = canvas();
    let placed = positions(&mut canvas, "provenance.default");
    let x = |k: usize| placed[&keys[k]].x;
    assert!(
        x(3) < x(0) && x(0) < x(2),
        "source, then each derivation step"
    );
    assert!(x(1) > x(2), "an unrelated node stands past the derivations");
    assert_eq!(crate::canvas::scene_axes::derived_column(0), SOURCES_COLUMN);
}

#[test]
fn the_territory_field_claims_ground_by_site_beneath_the_graph() {
    let (mut canvas, keys) = canvas();
    assert!(
        canvas.territory_commands().is_empty(),
        "off until asked for"
    );
    let spread = [(-220.0, 0.0), (-170.0, 60.0), (40.0, 220.0), (220.0, -40.0)];
    for (key, (x, y)) in keys.iter().zip(spread) {
        canvas.view.set_position(*key, PortablePoint::new(x, y));
    }
    canvas.set_physics_paused(true);
    canvas.set_territory_field(true);
    canvas.set_camera(crate::canvas::CameraView { offset: (400.0, 300.0), zoom: 1.0 });
    canvas.frame(VIEW.0, VIEW.1);
    let runs = canvas.territory_runs();
    let mut domains: Vec<usize> = runs.iter().map(|r| r.domain).collect();
    domains.sort_unstable();
    domains.dedup();
    assert_eq!(
        domains.len(),
        3,
        "woodshed, knot and search each claim ground"
    );
    assert!(runs.iter().any(|r| r.tier == 1) && runs.iter().any(|r| r.tier == 3));
    assert_eq!(canvas.territory_commands().len(), runs.len());
    crate::canvas::controls::CanvasCommand::ToggleTerritory.apply(&mut canvas, VIEW);
    assert!(!canvas.territory_field());
}

#[test]
fn territory_fills_the_current_view_after_pan_zoom_rotation_and_resize() {
    let (mut canvas, _) = canvas();
    canvas.set_physics_paused(true);
    let snapshot = serde_json::to_value(canvas.graph().to_snapshot_at(0)).unwrap();
    canvas.set_territory_field(true);
    for (w, h, yaw, zoom) in [(1200, 300, 0.0, 0.4), (300, 900, 0.7, 2.0)] {
        canvas.resize(w, h);
        let mut view = canvas.viewport();
        view.offset = (-700.0, 400.0);
        view.zoom = zoom;
        view.yaw = yaw;
        view.tilt = 0.6;
        canvas.set_viewport(view);
        let runs = canvas.territory_runs();
        let area: f64 = runs
            .iter()
            .map(|r| f64::from(r.max.x - r.min.x) * f64::from(r.max.y - r.min.y))
            .sum();
        assert!(
            (area - f64::from(w) * f64::from(h)).abs() < 0.1,
            "viewport must have no uncovered strip"
        );
        assert_eq!(canvas.territory_commands().len(), runs.len());
    }
    assert_eq!(
        snapshot,
        serde_json::to_value(canvas.graph().to_snapshot_at(0)).unwrap()
    );
}

#[test]
fn a_saved_scene_carries_the_seed_and_the_regions() {
    let (mut canvas, keys) = canvas();
    apply_arrangement(&mut canvas, "partition.default", VIEW).unwrap();
    canvas
        .view
        .set_position(keys[1], PortablePoint::new(-33.0, 44.0));
    canvas.set_seed();
    let regions = canvas.strategy_regions().to_vec();
    let json = serde_json::to_string(&canvas.cartography_geometry()).unwrap();

    let geometry: crate::canvas::geometry::CartographyGeometry =
        serde_json::from_str(&json).unwrap();
    let mut restored = Canvas::with_graph(canvas.graph().clone());
    restored.resize(VIEW.0, VIEW.1);
    restored.set_physics_paused(true);
    assert!(!restored.has_saved_seed());
    restored.apply_cartography_scene_state(&geometry);
    assert!(restored.has_saved_seed());
    assert_eq!(restored.strategy_regions(), regions.as_slice());
    restored.reset_to_seed();
    assert_eq!(
        restored.view.position_of(keys[1]),
        Some(PortablePoint::new(-33.0, 44.0)),
        "reset lands on the saved seed"
    );

    let mut lone = geometry.clone();
    let kept = canvas.graph().get_node(keys[0]).unwrap().id;
    lone.retain_members(|id| *id == kept);
    assert_eq!(lone.seed().unwrap().len(), 1);
    assert!(
        lone.regions()
            .iter()
            .all(|r| r.members.iter().all(|m| *m == kept))
    );

    let old: crate::canvas::geometry::CartographyGeometry =
        serde_json::from_str(r#"{"positions": []}"#).unwrap();
    assert!(
        old.seed().is_none() && old.regions().is_empty(),
        "an older scene still loads"
    );
}
