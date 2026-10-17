// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mosaic on the infinite canvas: tiles by site and weight, lifted by
//! selection without reflowing, organised and resized by the person.

use super::*;
use crate::canvas::mosaic::{MOSAIC_ARRANGEMENT, MosaicTarget};
use crate::canvas::physics_controls::{advance_arrangement, apply_arrangement};

const VIEW: (u32, u32) = (800, 600);

fn mosaic_canvas(urls: &[&str]) -> (Canvas, Vec<NodeKey>) {
    let mut graph = Graph::new();
    let keys = urls
        .iter()
        .map(|url| graph.add_node(url.to_string(), PortablePoint::zero()))
        .collect();
    let mut canvas = Canvas::with_graph(graph);
    canvas.resize(VIEW.0, VIEW.1);
    let applied = apply_arrangement(&mut canvas, MOSAIC_ARRANGEMENT, VIEW).unwrap();
    let mut transition = applied.transition;
    for at in [0.0, 1.0e6] {
        advance_arrangement(&mut canvas, &mut transition, MOSAIC_ARRANGEMENT, at);
    }
    (canvas, keys)
}

fn songs() -> (Canvas, Vec<NodeKey>) {
    mosaic_canvas(&[
        "https://woodshed.example/progression",
        "https://woodshed.example/take",
        "https://knot.example/lyrics",
        "https://search.example/chords",
    ])
}

fn id(canvas: &Canvas, key: NodeKey) -> uuid::Uuid {
    canvas.graph().get_node(key).unwrap().id
}

fn group_members(canvas: &Canvas, group: &str) -> Vec<NodeKey> {
    canvas
        .strategy_regions()
        .iter()
        .find(|r| r.label.as_deref() == Some(group))
        .map(|r| r.members.clone())
        .unwrap_or_default()
}

fn tile_area(canvas: &Canvas, key: NodeKey) -> f32 {
    let tile = canvas
        .strategy_regions()
        .iter()
        .find(|r| r.label.is_none() && r.members == [key])
        .unwrap();
    let (x0, y0, x1, y1) = tile.contour.iter().fold(
        (
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ),
        |(a, b, c, d), p| (a.min(p.x), b.min(p.y), c.max(p.x), d.max(p.y)),
    );
    (x1 - x0) * (y1 - y0)
}

#[test]
fn tiles_group_by_site_and_the_mosaic_outgrows_the_viewport() {
    let (canvas, keys) = songs();
    assert_eq!(group_members(&canvas, "woodshed.example"), keys[..2]);
    let tiles = canvas
        .strategy_regions()
        .iter()
        .filter(|r| r.label.is_none())
        .count();
    assert_eq!(tiles, 4);

    let urls: Vec<String> = (0..120)
        .map(|i| format!("https://s{}.example/{i}", i % 6))
        .collect();
    let refs: Vec<&str> = urls.iter().map(String::as_str).collect();
    let (large, keys) = mosaic_canvas(&refs);
    let reach = keys
        .iter()
        .filter_map(|k| large.view.position_of(*k))
        .map(|p| p.x.abs().max(p.y.abs()))
        .fold(0.0, f32::max);
    assert!(
        reach > 400.0,
        "infinite canvas, not the viewport: reach {reach}"
    );
}

#[test]
fn selection_lifts_without_reflowing() {
    // Two identical mosaics framed alike; only one has a selection.
    let (mut plain, keys) = songs();
    let (mut chosen, _) = songs();
    chosen.select_member(id(&chosen, keys[0]));
    chosen.toggle_select_member(id(&chosen, keys[2]));
    for _ in 0..3 {
        plain.frame(VIEW.0, VIEW.1);
        chosen.frame(VIEW.0, VIEW.1);
    }
    let at = |c: &Canvas| {
        keys.iter()
            .map(|k| c.view.position_of(*k))
            .collect::<Vec<_>>()
    };
    assert_eq!(at(&plain), at(&chosen));
    assert_eq!(plain.strategy_regions(), chosen.strategy_regions());
    assert_eq!(
        chosen.strategy_region_commands().len(),
        plain.strategy_region_commands().len(),
        "lifting restyles two tiles, adds nothing"
    );
}

#[test]
fn a_tile_moves_into_another_group_and_groups_reorder() {
    let (mut canvas, keys) = songs();
    let take = id(&canvas, keys[1]);
    let lyrics = id(&canvas, keys[2]);
    assert!(canvas.mosaic_move_member(take, MosaicTarget::Tile(lyrics)));
    assert_eq!(
        group_members(&canvas, "knot.example"),
        vec![keys[1], keys[2]]
    );
    assert_eq!(group_members(&canvas, "woodshed.example"), vec![keys[0]]);

    assert!(canvas.mosaic_move_group("search.example", "knot.example"));
    let order: Vec<_> = canvas
        .strategy_regions()
        .iter()
        .filter_map(|r| r.label.clone())
        .collect();
    let at = |g: &str| order.iter().position(|o| o == g).unwrap();
    assert!(at("search.example") < at("knot.example"));
    assert!(!canvas.mosaic_move_group("knot.example", "nowhere"));
}

#[test]
fn dragging_a_tile_onto_another_files_it_there() {
    let (mut canvas, keys) = songs();
    let from = canvas.screen_position_of(keys[3]).unwrap();
    let onto = canvas.screen_position_of(keys[0]).unwrap();
    canvas.pointer_down(PointerButton::Left, from.0, from.1);
    canvas.cursor_moved((from.0 + onto.0) / 2.0, (from.1 + onto.1) / 2.0);
    canvas.cursor_moved(onto.0, onto.1);
    canvas.pointer_up(PointerButton::Left, onto.0, onto.1);
    assert_eq!(
        group_members(&canvas, "woodshed.example"),
        vec![keys[3], keys[0], keys[1]]
    );
    assert!(group_members(&canvas, "search.example").is_empty());
}

#[test]
fn dragging_a_group_label_moves_the_group() {
    let (mut canvas, _) = songs();
    let band = |canvas: &Canvas, group: &str| {
        let region = canvas
            .strategy_regions()
            .iter()
            .find(|r| r.label.as_deref() == Some(group))
            .unwrap();
        let x = region.contour.iter().map(|p| p.x).sum::<f32>() / region.contour.len() as f32;
        let y = region
            .contour
            .iter()
            .map(|p| p.y)
            .fold(f32::INFINITY, f32::min)
            + 4.0;
        canvas.camera.to_screen(PortablePoint::new(x, y))
    };
    let labels = |canvas: &Canvas| -> Vec<String> {
        canvas
            .strategy_regions()
            .iter()
            .filter_map(|r| r.label.clone())
            .collect()
    };
    let first = labels(&canvas)[0].clone();
    let last = labels(&canvas).last().unwrap().clone();
    let (from, onto) = (band(&canvas, &last), band(&canvas, &first));
    canvas.pointer_down(PointerButton::Left, from.0, from.1);
    canvas.cursor_moved(onto.0, onto.1);
    canvas.pointer_up(PointerButton::Left, onto.0, onto.1);
    assert_eq!(labels(&canvas)[0], last, "the dragged group now leads");
}

#[test]
fn a_selected_tile_grows_and_shrinks() {
    let (mut canvas, keys) = songs();
    let before = tile_area(&canvas, keys[2]);
    canvas.select_member(id(&canvas, keys[2]));
    assert!(canvas.resize_selected_tiles(2.0));
    let grown = tile_area(&canvas, keys[2]);
    assert!(grown > before * 1.5, "{before} -> {grown}");
    crate::canvas::controls::CanvasCommand::ShrinkTiles.apply(&mut canvas, VIEW);
    assert!(tile_area(&canvas, keys[2]) < grown);
    canvas.clear_selection();
    assert!(!canvas.resize_selected_tiles(2.0));
}

#[test]
fn the_persons_mosaic_survives_a_saved_scene() {
    let (mut canvas, keys) = songs();
    let (take, lyrics) = (id(&canvas, keys[1]), id(&canvas, keys[2]));
    canvas.mosaic_move_member(take, MosaicTarget::Tile(lyrics));
    canvas.mosaic_move_group("search.example", "knot.example");
    canvas.select_member(lyrics);
    canvas.resize_selected_tiles(2.0);
    canvas.clear_selection();
    let arranged = canvas.strategy_regions().to_vec();
    let json = serde_json::to_string(&canvas.cartography_geometry()).unwrap();

    let geometry: crate::canvas::geometry::CartographyGeometry =
        serde_json::from_str(&json).unwrap();
    let mut restored = Canvas::with_graph(canvas.graph().clone());
    restored.resize(VIEW.0, VIEW.1);
    restored.apply_cartography_scene_state(&geometry);
    assert_eq!(
        restored.mosaic_state().group_order,
        canvas.mosaic_state().group_order
    );
    let applied = apply_arrangement(&mut restored, MOSAIC_ARRANGEMENT, VIEW).unwrap();
    let mut transition = applied.transition;
    for at in [0.0, 1.0e6] {
        advance_arrangement(&mut restored, &mut transition, MOSAIC_ARRANGEMENT, at);
    }
    assert_eq!(
        restored.strategy_regions(),
        arranged.as_slice(),
        "the same mosaic, re-projected"
    );

    let mut plain = geometry.clone();
    plain.retain_members(|member| *member != take);
    assert!(
        plain.mosaic().unwrap().group_of.is_empty(),
        "the moved tile was filtered out"
    );
    let (fresh, _) = songs();
    assert!(
        fresh.cartography_geometry().mosaic().is_none(),
        "nothing arranged, nothing saved"
    );
}
