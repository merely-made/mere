// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use crate::canvas::{Canvas, DerivedFacePalette, Face};
use euclid::default::Point2D;
use kernel::graph::Graph;
use kernel::graph::fixtures::GraphFixtures;

fn graph_with_one_node(url: &str) -> (Graph, kernel::graph::NodeKey) {
    let mut graph = Graph::new();
    let key = graph.add_node_with_id(
        Graph::node_namespace_id(url),
        url.to_string(),
        Point2D::zero(),
    );
    (graph, key)
}

fn image_op_count(scene: &netrender::Scene) -> usize {
    scene
        .ops
        .iter()
        .filter(|op| matches!(op, netrender::SceneOp::Image(_)))
        .count()
}

fn has_shape_fill(scene: &netrender::Scene, fill: [f32; 4]) -> bool {
    scene
        .iter_shapes()
        .any(|shape| shape.fill_color == Some(fill))
}

/// A node carrying favicon RGBA emits an image op over its on-screen tile, so the
/// host rasterizes a real favicon on the square. (Favicon-on-tile.)
#[test]
fn favicon_node_emits_an_image_op() {
    let (mut graph, key) = graph_with_one_node("https://ex.test/");
    // The node carries a reference; the host registers the decoded pixels.
    let favicon = kernel::types::ImageRef::new([42u8; 32], 2, 2);
    assert!(graph.set_node_image(key, kernel::types::ImageRole::Favicon, favicon));
    let mut canvas = Canvas::with_graph(graph);
    canvas.register_resolved_image([42u8; 32], vec![255u8; 2 * 2 * 4], 2, 2);
    let (scene, _) = canvas.frame(800, 600);
    assert!(
        image_op_count(&scene) >= 1,
        "a favicon node emits at least one image op"
    );
}

#[test]
fn visible_cache_miss_requests_the_image_once() {
    let (mut graph, key) = graph_with_one_node("https://ex.test/");
    let favicon = kernel::types::ImageRef::new([42u8; 32], 2, 2);
    assert!(graph.set_node_image(key, kernel::types::ImageRole::Favicon, favicon));
    let mut canvas = Canvas::with_graph(graph);

    let (scene, _) = canvas.frame(800, 600);
    assert_eq!(
        image_op_count(&scene),
        0,
        "the cold frame has no pixels yet"
    );
    assert_eq!(canvas.take_image_requests(), vec![favicon]);

    let _ = canvas.frame(800, 600);
    assert!(
        canvas.take_image_requests().is_empty(),
        "an unresolved blob is not requested every frame"
    );
}

/// Without a favicon, the default is a vector face rather than an image.
#[test]
fn node_without_favicon_emits_a_derived_path() {
    let (graph, _key) = graph_with_one_node("mere://note/one");
    let mut canvas = Canvas::with_graph(graph);
    canvas.set_derived_face_palette(DerivedFacePalette::new([[255, 0, 0, 255]; 8]));
    let (scene, _) = canvas.frame(800, 600);
    assert_eq!(image_op_count(&scene), 0, "no favicon -> no image op");
    assert!(
        has_shape_fill(&scene, [1.0, 0.0, 0.0, 1.0]),
        "the favicon-less node emits palette-resolved vector paths"
    );
}

#[test]
fn explicit_face_override_suppresses_the_content_default() {
    let (graph, key) = graph_with_one_node("mere://note/bare");
    let mut canvas = Canvas::with_graph(graph);
    let id = canvas.graph().get_node(key).unwrap().id;
    canvas.set_derived_face_palette(DerivedFacePalette::new([[255, 0, 0, 255]; 8]));
    canvas.set_node_face(id, Face::Bare);

    let (scene, _) = canvas.frame(800, 600);
    assert!(!has_shape_fill(&scene, [1.0, 0.0, 0.0, 1.0]));
}

#[test]
fn palette_swap_recolors_the_cached_face_without_rederiving() {
    let (graph, _key) = graph_with_one_node("mere://note/recolor");
    let mut canvas = Canvas::with_graph(graph);
    canvas.set_derived_face_palette(DerivedFacePalette::new([[255, 0, 0, 255]; 8]));
    let (red_scene, _) = canvas.frame(800, 600);
    let cached = canvas.derived_face_cache.clone();

    canvas.set_derived_face_palette(DerivedFacePalette::new([[0, 0, 255, 255]; 8]));
    let (blue_scene, _) = canvas.frame(800, 600);

    assert!(has_shape_fill(&red_scene, [1.0, 0.0, 0.0, 1.0]));
    assert!(has_shape_fill(&blue_scene, [0.0, 0.0, 1.0, 1.0]));
    assert_eq!(
        canvas.derived_face_cache, cached,
        "theme swaps keep the bytes"
    );
}

/// A scene prop carrying a *registered* sprite handle paints as a textured billboard, so the
/// frame emits an image op over the prop. (Scene-prop sprites.)
#[test]
fn registered_sprite_scene_prop_emits_an_image_op() {
    let (graph, _key) = graph_with_one_node("https://ex.test/");
    let mut canvas = Canvas::with_graph(graph);
    canvas.register_scene_sprite("crate", vec![255u8; 4 * 4 * 4], 4, 4);
    let spec = seiche::SceneSpec {
        bodies: vec![
            seiche::SceneBodySpec::dynamic(seiche::NodeCollider::Square { half: 30.0 }, (0.0, 0.0))
                .sprite("crate"),
        ],
        gravity: (0.0, 0.0),
        default_tangible: false,
        perpetual: false,
        joints: Vec::new(),
    };
    canvas.load_scene(spec);
    let (scene, _) = canvas.frame(800, 600);
    assert!(
        image_op_count(&scene) >= 1,
        "a registered sprite prop emits an image op"
    );
}

/// A scene prop whose sprite handle is *not* registered falls back to the abstract polygon, so
/// no image op is emitted. (Scene-prop sprites.)
#[test]
fn unregistered_sprite_scene_prop_emits_no_image_op() {
    let (graph, _key) = graph_with_one_node("https://ex.test/");
    let mut canvas = Canvas::with_graph(graph);
    let spec = seiche::SceneSpec {
        bodies: vec![
            seiche::SceneBodySpec::dynamic(seiche::NodeCollider::Square { half: 30.0 }, (0.0, 0.0))
                .sprite("missing"),
        ],
        gravity: (0.0, 0.0),
        default_tangible: false,
        perpetual: false,
        joints: Vec::new(),
    };
    canvas.load_scene(spec);
    let (scene, _) = canvas.frame(800, 600);
    assert_eq!(
        image_op_count(&scene),
        0,
        "an unregistered handle falls back, no image op"
    );
}

/// The Game of Life ambient backdrop keeps the canvas redrawing (so it animates) while loaded,
/// and parks again once cleared. (Physics scenes P5.)
#[test]
fn game_of_life_backdrop_keeps_redrawing_until_cleared() {
    let (graph, _key) = graph_with_one_node("https://ex.test/");
    let mut canvas = Canvas::with_graph(graph);
    // Settle the graph so the only thing that could request a redraw is the ambient sim.
    for _ in 0..400 {
        let _ = canvas.frame(800, 600);
    }
    let (_, settled_redraw) = canvas.frame(800, 600);
    assert!(!settled_redraw, "a settled graph with no ambient sim parks");

    canvas.load_game_of_life();
    let (_, with_gol) = canvas.frame(800, 600);
    assert!(with_gol, "the ambient backdrop keeps the canvas redrawing");

    canvas.clear_ambient();
    for _ in 0..400 {
        let _ = canvas.frame(800, 600);
    }
    let (_, after_clear) = canvas.frame(800, 600);
    assert!(
        !after_clear,
        "clearing the ambient backdrop lets the canvas park again"
    );
}
