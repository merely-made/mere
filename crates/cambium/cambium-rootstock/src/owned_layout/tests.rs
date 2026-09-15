// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Nested scroll offsets across a layout rebuild.
//!
//! The boxes are sized in pixels and hold no text, so every figure below is
//! arithmetic rather than a font metric: a 50px-tall `overflow: auto` box over
//! a 200px child scrolls exactly 150.

use super::*;
use genet_scripted_dom::ScriptedDom;
use layout_dom_api::{LayoutDomMut, QualName};

const VIEWPORT: (f32, f32) = (300.0, 400.0);

fn style(dom: &mut ScriptedDom, node: NodeId, declarations: &str) {
    dom.set_attribute(
        node,
        QualName::new(None, Namespace::from(""), LocalName::from("style")),
        declarations,
    );
}

fn div(dom: &mut ScriptedDom, parent: NodeId, declarations: &str) -> NodeId {
    let node = dom.create_element(QualName::new(
        None,
        Namespace::from(""),
        LocalName::from("div"),
    ));
    dom.append_child(parent, node);
    style(dom, node, declarations);
    node
}

/// Two independent scroll containers, each 50px over a 200px child.
/// Returns the DOM, the layout, and `(box, content, other)`.
fn fixture() -> (ScriptedDom, OwnedLayout, NodeId, NodeId, NodeId) {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    let scroller = div(&mut dom, root, "width:100px;height:50px;overflow:auto;");
    let content = div(&mut dom, scroller, "width:100px;height:200px;");
    let other = div(&mut dom, root, "width:100px;height:50px;overflow:auto;");
    div(&mut dom, other, "width:100px;height:200px;");
    let layout = OwnedLayout::new(
        &dom,
        &[""],
        VIEWPORT.0,
        VIEWPORT.1,
        &[],
        &Default::default(),
    );
    (dom, layout, scroller, content, other)
}

/// Wheel a container to its end, whatever its range is.
fn scroll_to_end(dom: &ScriptedDom, layout: &mut OwnedLayout, x: f32, y: f32) {
    layout.scroll_at_target(dom, x, y, 0.0, 10_000.0);
}

#[test]
fn a_shrinking_container_pulls_its_offset_back_to_the_new_end() {
    let (mut dom, mut layout, scroller, content, _) = fixture();
    scroll_to_end(&dom, &mut layout, 50.0, 25.0);
    assert_eq!(layout.element_scroll()[&scroller], (0.0, 150.0));

    style(&mut dom, content, "width:100px;height:60px;");
    layout.rebuild(&dom, VIEWPORT.0, VIEWPORT.1);
    assert_eq!(
        layout.element_scroll()[&scroller],
        (0.0, 10.0),
        "the offset follows the content down, rather than staying at 150",
    );
}

#[test]
fn a_box_that_stops_scrolling_or_leaves_the_dom_loses_its_offset() {
    let (mut dom, mut layout, scroller, _, other) = fixture();
    scroll_to_end(&dom, &mut layout, 50.0, 25.0);
    scroll_to_end(&dom, &mut layout, 50.0, 75.0);
    assert_eq!(layout.element_scroll().len(), 2);

    style(&mut dom, scroller, "width:100px;height:50px;");
    dom.remove(other);
    layout.rebuild(&dom, VIEWPORT.0, VIEWPORT.1);
    assert!(
        layout.element_scroll().is_empty(),
        "neither a non-scrolling box nor a departed one keeps an entry: {:?}",
        layout.element_scroll(),
    );
}

#[test]
fn an_unrelated_container_keeps_the_offset_it_had() {
    let (mut dom, mut layout, _, content, other) = fixture();
    scroll_to_end(&dom, &mut layout, 50.0, 75.0);
    assert_eq!(layout.element_scroll()[&other], (0.0, 150.0));

    style(&mut dom, content, "width:100px;height:60px;");
    layout.rebuild(&dom, VIEWPORT.0, VIEWPORT.1);
    assert_eq!(layout.element_scroll()[&other], (0.0, 150.0));
}

/// The rebuild path that builds a *fresh* session carries the plane across
/// after that session has already laid out, so the clamp has to happen as the
/// plane is set.
#[test]
fn a_plane_carried_onto_a_fresh_session_is_clamped_as_it_is_set() {
    let (mut dom, mut layout, scroller, content, other) = fixture();
    scroll_to_end(&dom, &mut layout, 50.0, 25.0);
    scroll_to_end(&dom, &mut layout, 50.0, 75.0);
    let carried = layout.element_scroll().clone();

    style(&mut dom, content, "width:100px;height:60px;");
    dom.remove(other);
    let mut fresh = OwnedLayout::new(
        &dom,
        &[""],
        VIEWPORT.0,
        VIEWPORT.1,
        &[],
        &Default::default(),
    );
    fresh.set_element_scroll(&dom, carried);
    assert_eq!(fresh.element_scroll()[&scroller], (0.0, 10.0));
    assert_eq!(fresh.element_scroll().len(), 1);
}
