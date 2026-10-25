// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
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

impl OwnedLayout {
    /// `rebuild` over whatever `dom` has logged, as the host's relayout drains it.
    pub(super) fn rebuild_drained(&mut self, dom: &mut ScriptedDom, width: f32, height: f32) {
        let mut mutations = Vec::new();
        dom.drain_mutations(&mut mutations);
        self.rebuild(dom, width, height, &mutations);
    }
}

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

#[test]
fn element_reveal_notifies_each_two_axis_plane_once_and_keeps_start_vertical() {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    let scroller = div(
        &mut dom,
        root,
        "position:relative;margin-left:400px;margin-top:500px;width:200px;height:100px;overflow:auto;",
    );
    let content = div(
        &mut dom,
        scroller,
        "position:relative;width:900px;height:600px;",
    );
    let target = div(
        &mut dom,
        content,
        "position:absolute;left:820px;top:550px;width:80px;height:40px;overflow:auto;",
    );
    div(&mut dom, target, "width:160px;height:80px;");
    let mut layout = OwnedLayout::new(
        &dom,
        &[""],
        VIEWPORT.0,
        VIEWPORT.1,
        &[],
        &Default::default(),
    );

    assert_eq!(
        layout.scroll_into_view(&dom, target, ScrollAlign::Nearest),
        vec![ScrollTarget::Element(scroller), ScrollTarget::Document]
    );
    assert_eq!(layout.element_scroll()[&scroller], (700.0, 490.0));
    assert!(
        !layout.element_scroll().contains_key(&target),
        "reveal scrolls ancestors, not the target's contents"
    );
    assert!(layout.visible_rect(&dom, target).is_some());
    assert!(
        layout
            .scroll_into_view(&dom, target, ScrollAlign::Nearest)
            .is_empty()
    );

    assert_eq!(
        layout.scroll_into_view(&dom, target, ScrollAlign::Start),
        vec![ScrollTarget::Element(scroller)]
    );
    assert_eq!(
        layout.element_scroll()[&scroller],
        (700.0, 500.0),
        "Start changes vertical alignment only; horizontal remains nearest and clamped"
    );
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
    layout.rebuild_drained(&mut dom, VIEWPORT.0, VIEWPORT.1);
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
    layout.rebuild_drained(&mut dom, VIEWPORT.0, VIEWPORT.1);
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
    layout.rebuild_drained(&mut dom, VIEWPORT.0, VIEWPORT.1);
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

/// A bounded workspace containing the recorder's three rows and transport.
/// Both horizontal content and the transport extend beyond the recorder, but
/// neither may enlarge the document after a wheel reaches the recorder end.
#[test]
fn document_extent_nested_recorder_repeated_wheel_stays_in_child() {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    let workspace = div(&mut dom, root, "width:280px;height:320px;overflow:auto;");
    let recorder = div(
        &mut dom,
        workspace,
        "width:260px;height:200px;overflow:auto;",
    );
    let content = div(&mut dom, recorder, "width:900px;height:400px;");
    div(&mut dom, content, "width:900px;height:120px;");
    div(&mut dom, content, "width:900px;height:120px;");
    let third_card = div(&mut dom, content, "width:900px;height:120px;");
    let transport = div(&mut dom, content, "width:900px;height:40px;");
    let mut layout = OwnedLayout::new(
        &dom,
        &[""],
        VIEWPORT.0,
        VIEWPORT.1,
        &[],
        &Default::default(),
    );
    assert_eq!(
        layout.scroll_at_target(&dom, 20.0, 20.0, 10_000.0, 10_000.0),
        Some(ScrollTarget::Element(recorder)),
    );
    assert_eq!(layout.element_scroll()[&recorder], (640.0, 200.0));
    assert_eq!(layout.viewport_scroll(), (0.0, 0.0));
    assert!(layout.visible_rect(&dom, third_card).is_some());
    assert!(layout.visible_rect(&dom, transport).is_some());
    for _ in 0..3 {
        assert_eq!(
            layout.scroll_at_target(&dom, 20.0, 180.0, 10_000.0, 10_000.0),
            None,
        );
        assert_eq!(layout.viewport_scroll(), (0.0, 0.0));
        assert_eq!(layout.element_scroll()[&recorder], (640.0, 200.0));
        assert!(!layout.element_scroll().contains_key(&workspace));
    }
    assert_eq!(
        layout.scroll_at_target(&dom, 20.0, 180.0, -10_000.0, -10_000.0),
        Some(ScrollTarget::Element(recorder)),
    );
    assert_eq!(layout.element_scroll()[&recorder], (0.0, 0.0));
    assert_eq!(layout.viewport_scroll(), (0.0, 0.0));
}

#[test]
fn document_extent_excludes_hidden_clip_auto_and_scroll_descendants() {
    for overflow in ["hidden", "clip", "auto", "scroll"] {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let boundary = div(
            &mut dom,
            root,
            &format!("width:100px;height:100px;overflow:{overflow};"),
        );
        div(&mut dom, boundary, "width:900px;height:1000px;");
        let mut layout = OwnedLayout::new(
            &dom,
            &[""],
            VIEWPORT.0,
            VIEWPORT.1,
            &[],
            &Default::default(),
        );
        layout.set_viewport_scroll((10_000.0, 10_000.0));
        assert_eq!(layout.viewport_scroll(), (0.0, 0.0), "overflow:{overflow}");
    }
}

#[test]
fn document_extent_clips_only_the_bounded_axis() {
    for (overflow, expected) in [
        ("overflow-x:clip;overflow-y:visible;", (0.0, 600.0)),
        ("overflow-x:visible;overflow-y:clip;", (600.0, 0.0)),
    ] {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let boundary = div(
            &mut dom,
            root,
            &format!("width:100px;height:100px;{overflow}"),
        );
        div(&mut dom, boundary, "width:900px;height:1000px;");
        let mut layout = OwnedLayout::new(
            &dom,
            &[""],
            VIEWPORT.0,
            VIEWPORT.1,
            &[],
            &Default::default(),
        );
        assert_eq!(
            layout.scroll_at_target(&dom, 200.0, 200.0, 10_000.0, 10_000.0),
            Some(ScrollTarget::Document),
        );
        assert_eq!(layout.viewport_scroll(), expected, "{overflow}");
    }
}

#[test]
fn document_extent_preserves_visible_outer_overflow_and_boundary_border_box() {
    for bounded in [false, true] {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let outer_style = if bounded {
            "margin-left:100px;margin-top:100px;width:500px;height:600px;overflow:auto;"
        } else {
            "margin-left:100px;margin-top:100px;width:100px;height:100px;overflow:visible;"
        };
        let outer = div(&mut dom, root, outer_style);
        let inner_style = if bounded {
            "width:900px;height:1000px;"
        } else {
            "width:500px;height:600px;"
        };
        div(&mut dom, outer, inner_style);
        let mut layout = OwnedLayout::new(
            &dom,
            &[""],
            VIEWPORT.0,
            VIEWPORT.1,
            &[],
            &Default::default(),
        );
        // Outside the element: real document overflow must still accept wheel.
        assert_eq!(
            layout.scroll_at_target(&dom, 20.0, 20.0, 10_000.0, 10_000.0),
            Some(ScrollTarget::Document),
        );
        assert_eq!(
            layout.viewport_scroll(),
            (300.0, 300.0),
            "bounded:{bounded}"
        );
        assert_eq!(
            layout.scroll_at_target(&dom, 20.0, 20.0, -10_000.0, -10_000.0),
            Some(ScrollTarget::Document),
        );
        assert_eq!(layout.viewport_scroll(), (0.0, 0.0));
    }
}

#[test]
fn document_extent_preserves_inline_bounds_and_rebuild_reclamps_after_clipping() {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    let owner = div(
        &mut dom,
        root,
        "width:220px;height:40px;line-height:600px;overflow:visible;",
    );
    let text = dom.create_text("Hg");
    dom.append_child(owner, text);
    let mut layout = OwnedLayout::new(
        &dom,
        &[""],
        VIEWPORT.0,
        VIEWPORT.1,
        &[],
        &Default::default(),
    );
    let lines = layout.fragments.inline_scroll_bounds(owner).unwrap();
    assert_eq!(lines.height, 600.0);
    layout.set_viewport_scroll((0.0, 10_000.0));
    assert!(
        layout.viewport_scroll().1 >= 200.0,
        "visible line boxes must contribute",
    );
    style(
        &mut dom,
        owner,
        "width:220px;height:40px;line-height:600px;overflow:hidden;",
    );
    layout.rebuild_drained(&mut dom, VIEWPORT.0, VIEWPORT.1);
    assert_eq!(layout.viewport_scroll(), (0.0, 0.0));
}

#[test]
fn a_caret_paints_in_its_fields_text_colour() {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    let field = div(&mut dom, root, "color:rgb(255, 0, 0);");
    let layout = OwnedLayout::new(
        &dom,
        &[""],
        VIEWPORT.0,
        VIEWPORT.1,
        &[],
        &Default::default(),
    );
    assert_eq!(layout.caret_color(&dom, field), Some([1.0, 0.0, 0.0, 1.0]));
}

#[test]
fn formatting_lines_supply_own_and_visible_descendant_scroll_range() {
    for descendant in [false, true] {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let scroller = div(
            &mut dom,
            root,
            "width:220px;height:180px;overflow:auto;line-height:200px;",
        );
        let owner = if descendant {
            div(&mut dom, scroller, "height:180px;overflow:visible;")
        } else {
            scroller
        };
        let text = dom.create_text("Hg");
        dom.append_child(owner, text);
        let mut layout = OwnedLayout::new(
            &dom,
            &[""],
            VIEWPORT.0,
            VIEWPORT.1,
            &[],
            &Default::default(),
        );
        assert_eq!(
            element_scroll_range(&dom, &layout.styles, &layout.fragments, scroller).1,
            20.0
        );
        layout.set_element_scroll(&dom, HashMap::from([(scroller, (0.0, 12.0))]));
        assert_eq!(layout.element_scroll()[&scroller], (0.0, 12.0));
    }
}

#[test]
fn clipped_descendant_lines_do_not_expand_the_outer_scroll_range() {
    for overflow in ["hidden", "clip", "auto", "scroll"] {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let scroller = div(&mut dom, root, "width:220px;height:180px;overflow:auto;");
        let child = div(
            &mut dom,
            scroller,
            &format!("height:180px;line-height:200px;overflow:{overflow};"),
        );
        let text = dom.create_text("Hg");
        dom.append_child(child, text);
        let mut layout = OwnedLayout::new(
            &dom,
            &[""],
            VIEWPORT.0,
            VIEWPORT.1,
            &[],
            &Default::default(),
        );
        assert_eq!(
            layout.fragments.inline_scroll_bounds(child).unwrap().height,
            200.0
        );
        assert_eq!(
            element_scroll_range(&dom, &layout.styles, &layout.fragments, scroller).1,
            0.0,
            "overflow:{overflow}"
        );
        layout.set_element_scroll(&dom, HashMap::from([(scroller, (0.0, 12.0))]));
        assert_eq!(layout.element_scroll()[&scroller], (0.0, 0.0));
    }
}

#[test]
fn later_text_fragments_contribute_beyond_short_formatting_lines() {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    let scroller = div(
        &mut dom,
        root,
        "width:150px;height:20px;overflow:auto;font-size:80px;line-height:10px;",
    );
    let text = dom.create_text("Hg Hg Hg");
    dom.append_child(scroller, text);
    let mut layout = OwnedLayout::new(
        &dom,
        &[""],
        VIEWPORT.0,
        VIEWPORT.1,
        &[],
        &Default::default(),
    );
    let container = layout.fragments.get(scroller).unwrap();
    let first = layout.fragments.get(text).unwrap();
    let all: Vec<_> = layout.fragments.fragments_for_node(text).collect();
    assert!(all.len() > 1, "the text must span retained fragments");
    let bottom = all
        .iter()
        .map(|fragment| fragment.y + fragment.height)
        .fold(f32::NEG_INFINITY, f32::max);
    let lines = layout.fragments.inline_scroll_bounds(scroller).unwrap();
    assert!(bottom > first.y + first.height);
    assert!(
        bottom > lines.y + lines.height,
        "font content must exceed the short line boxes"
    );
    let expected = bottom - container.y - container.height;
    assert!(expected > 0.0);
    assert_eq!(
        element_scroll_range(&dom, &layout.styles, &layout.fragments, scroller).1,
        expected
    );
    layout.set_element_scroll(&dom, HashMap::from([(scroller, (0.0, 10_000.0))]));
    assert_eq!(layout.element_scroll()[&scroller], (0.0, expected));
}

#[test]
fn host_generic_faces_reach_actual_paint_and_refuse_invalid_font_bytes() {
    use paint_list_api::PaintList;
    let bytes =
        include_bytes!("../../../examples/genet_web_smoke/assets/Roboto-Regular.ttf").to_vec();
    let mut dom = ScriptedDom::from_serialized_document(
        "<html><body><p style='font-family:sans-serif'>Sans</p><p style='font-family:serif'>Serif</p><p style='font-family:monospace'>Mono</p><p style='font-family:system-ui'>System</p><p style=\"font-family:'sans-serif'\">Quoted alias</p></body></html>",
    );
    let fonts: Vec<_> = ["sans-serif", "serif", "monospace", "system-ui"]
        .into_iter()
        .map(|family| crate::HostFont {
            family: Some(family.into()),
            bytes: bytes.clone(),
        })
        .collect();
    let mut layout = OwnedLayout::new(
        &dom,
        &["p {font-size:16px}"],
        300.0,
        240.0,
        &fonts,
        &Default::default(),
    );
    assert!(layout.font_registration_errors().is_empty());
    let list = layout.emit_paint_list_with_leaves(
        &dom,
        DeviceIntSize::new(300, 240),
        None,
        |_| None,
        |_| None,
    );
    assert!(
        list.commands()
            .iter()
            .filter(|op| matches!(op,PaintCmd::DrawText(run) if !run.glyphs.is_empty()))
            .count()
            >= 5
    );
    assert!(!list.fonts().is_empty());
    assert!(
        list.fonts()
            .iter()
            .all(|face| face.data.as_slice() == bytes)
    );
    layout.rebuild_drained(&mut dom, 280.0, 240.0);
    let list = layout.emit_paint_list_with_leaves(
        &dom,
        DeviceIntSize::new(280, 240),
        None,
        |_| None,
        |_| None,
    );
    assert!(
        list.fonts()
            .iter()
            .all(|face| face.data.as_slice() == bytes)
    );
    let invalid = OwnedLayout::new(
        &dom,
        &[""],
        300.0,
        240.0,
        &[crate::HostFont {
            family: Some("sans-serif".into()),
            bytes: vec![0],
        }],
        &Default::default(),
    );
    assert_eq!(
        invalid.font_registration_errors(),
        &[genet_livery::GenericFontError::InvalidFont]
    );
}
