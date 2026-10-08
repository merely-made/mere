// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The pointer gestures of Scenograph editor plan track C1: a left-drag on
//! empty canvas pans (SE23), a right-drag selects, and a right click asks the
//! host for its context menu (SE26).

use super::*;

/// A canvas with one node held at world (100, 120), and that point on screen.
fn one_node() -> (Canvas, NodeKey, (f32, f32)) {
    let mut graph = Graph::new();
    graph.add_node(
        "https://gesture.example".to_string(),
        PortablePoint::new(0.0, 0.0),
    );
    let mut canvas = Canvas::with_graph(graph);
    let key = canvas
        .graph()
        .get_node_by_url("https://gesture.example")
        .unwrap()
        .0;
    canvas.set_layout_strategy(Some("test.grid".to_string()));
    canvas.apply_strategy_positions(&[(key, PortablePoint::new(100.0, 120.0))]);
    canvas.apply_strategy_to_view();
    let at = canvas.camera.to_screen(PortablePoint::new(100.0, 120.0));
    (canvas, key, at)
}

#[test]
fn a_left_drag_on_empty_canvas_pans_and_glides() {
    let (mut canvas, _, node) = one_node();
    let empty = (node.0 + 400.0, node.1 + 300.0);
    let offset = canvas.camera.offset;

    canvas.pointer_down(PointerButton::Left, empty.0, empty.1);
    canvas.cursor_moved(empty.0 + 2.0, empty.1 + 1.0);
    assert_eq!(
        canvas.camera.offset, offset,
        "within the slop it is still a click"
    );
    assert!(!canvas.left_panning());

    canvas.cursor_moved(empty.0 + 60.0, empty.1 + 30.0);
    assert!(canvas.left_panning());
    canvas.cursor_moved(empty.0 + 80.0, empty.1 + 40.0);
    assert_eq!(
        (
            canvas.camera.offset.0 - offset.0,
            canvas.camera.offset.1 - offset.1
        ),
        (80.0, 40.0),
        "the canvas stays under the hand from the press, slop included"
    );
    canvas.pointer_up(PointerButton::Left, empty.0 + 80.0, empty.1 + 40.0);
    assert!(!canvas.left_panning());
    assert!(
        canvas.selected_members().is_empty(),
        "a pan selects nothing"
    );
    let released = canvas.camera.offset;
    canvas.frame(1024, 768);
    assert_ne!(
        canvas.camera.offset, released,
        "the pan's momentum glides on"
    );
}

#[test]
fn a_bare_left_click_on_empty_canvas_still_clears_the_selection() {
    let (mut canvas, key, node) = one_node();
    canvas.pointer_down(PointerButton::Left, node.0, node.1);
    canvas.pointer_up(PointerButton::Left, node.0, node.1);
    assert!(canvas.selected.contains(&key));

    let empty = (node.0 + 400.0, node.1 + 300.0);
    let offset = canvas.camera.offset;
    canvas.pointer_down(PointerButton::Left, empty.0, empty.1);
    canvas.pointer_up(PointerButton::Left, empty.0, empty.1);
    assert!(canvas.selected.is_empty());
    assert_eq!(canvas.camera.offset, offset);
}

#[test]
fn a_left_drag_on_a_node_still_moves_the_node() {
    let (mut canvas, key, node) = one_node();
    let offset = canvas.camera.offset;
    canvas.pointer_down(PointerButton::Left, node.0, node.1);
    canvas.cursor_moved(node.0 + 80.0, node.1 + 40.0);
    canvas.pointer_up(PointerButton::Left, node.0 + 80.0, node.1 + 40.0);
    canvas.apply_strategy_to_view();
    let moved = canvas.view.position_of(key).expect("moved node position");
    assert!((moved.x - 180.0).abs() < 0.01);
    assert_eq!(canvas.camera.offset, offset, "a node drag does not pan");
}

#[test]
fn a_right_drag_selects_what_it_covers() {
    let (mut canvas, key, node) = one_node();
    canvas.pointer_down(PointerButton::Right, node.0 - 50.0, node.1 - 50.0);
    canvas.cursor_moved(node.0 + 50.0, node.1 + 50.0);
    canvas.pointer_up(PointerButton::Right, node.0 + 50.0, node.1 + 50.0);
    assert!(canvas.selected.contains(&key));
    assert_eq!(
        canvas.take_context_request(),
        None,
        "a drag asks for no menu"
    );
}

#[test]
fn a_right_click_asks_for_the_context_menu_naming_the_node_under_it() {
    let (mut canvas, key, node) = one_node();
    let member = canvas.graph().get_node(key).unwrap().id;
    canvas.pointer_down(PointerButton::Right, node.0, node.1);
    canvas.pointer_up(PointerButton::Right, node.0 + 1.0, node.1);
    assert_eq!(
        canvas.take_context_request(),
        Some(ContextRequest {
            at: (node.0 + 1.0, node.1),
            node: Some(member),
        })
    );
    assert_eq!(canvas.take_context_request(), None, "taken once");

    let empty = (node.0 + 400.0, node.1 + 300.0);
    canvas.pointer_down(PointerButton::Right, empty.0, empty.1);
    canvas.pointer_up(PointerButton::Right, empty.0, empty.1);
    assert_eq!(
        canvas.take_context_request().map(|request| request.node),
        Some(None),
        "on empty canvas the menu names no node"
    );
}
