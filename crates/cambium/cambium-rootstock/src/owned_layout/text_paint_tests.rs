// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Exercise the emission helper used by the actual frame pipeline. Geometry
//! alone cannot prove that a later DOM surface covers source selection/caret.

use super::*;
use genet_scripted_dom::ScriptedDom;
use layout_dom_api::{LayoutDomMut, QualName};
use paint_list_api::PaintList;

const SELECTION: ColorF = ColorF {
    r: 0.20,
    g: 0.45,
    b: 0.90,
    a: 0.35,
};
const RED: ColorF = ColorF {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};
const BLUE: ColorF = ColorF {
    r: 0.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};

fn name(local: &str) -> QualName {
    QualName::new(None, Namespace::from(""), LocalName::from(local))
}

fn element(dom: &mut ScriptedDom, parent: NodeId, tag: &str, css: &str) -> NodeId {
    let node = dom.create_element(name(tag));
    dom.set_attribute(node, name("style"), css);
    dom.append_child(parent, node);
    node
}

fn fixture(overlay: bool) -> (ScriptedDom, OwnedLayout, NodeId, NodeId) {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    let outer = element(
        &mut dom,
        root,
        "div",
        "position:absolute;left:10px;top:10px;width:200px;height:90px;overflow:auto;opacity:.6;transform:translate(7px,5px);",
    );
    let field = element(
        &mut dom,
        outer,
        "div",
        "width:180px;height:48px;overflow:auto;white-space:pre;font-size:16px;line-height:24px;color:rgb(255,0,0);background:rgb(0,255,0);",
    );
    dom.set_attribute(field, name("contenteditable"), "true");
    // Highlighted fields split their bytes across inline spans. Their selection
    // must remain visible over opaque token backgrounds as well as text.
    for text in ["Alpha\n", "Beta\nGamma\nDelta\nEpsilon\n"] {
        let span = element(
            &mut dom,
            field,
            "span",
            "color:inherit;background:rgb(255,255,0);",
        );
        let text = dom.create_text(text);
        dom.append_child(span, text);
    }
    element(&mut dom, outer, "div", "height:200px;");
    if overlay {
        element(
            &mut dom,
            root,
            "div",
            "position:absolute;left:10px;top:10px;width:220px;height:100px;z-index:20;background:rgb(0,0,255);",
        );
    }
    let layout = OwnedLayout::new(&dom, &[""], 300.0, 220.0, &[], &Default::default());
    (dom, layout, field, outer)
}

fn focused(field: NodeId) -> Option<FocusedTextPaint> {
    Some((
        field,
        VisualCaret {
            byte: 10,
            affinity: VisualAffinity::Downstream,
        },
        Some((6, 10)),
    ))
}

fn emitted(dom: &ScriptedDom, layout: &mut OwnedLayout, field: NodeId) -> LiveryPaintList {
    layout.emit_paint_list_with_leaves(
        dom,
        DeviceIntSize::new(300, 220),
        focused(field),
        |_| None,
        |_| None,
    )
}

fn rect_index(list: &LiveryPaintList, color: ColorF) -> usize {
    let indices = list
        .commands()
        .iter()
        .enumerate()
        .filter_map(|(index, command)| {
            matches!(command, PaintCmd::DrawRect(rect) if rect.color == color).then_some(index)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        indices.len(),
        1,
        "one nonempty decoration/overlay: {color:?}"
    );
    indices[0]
}

fn glyph_indices(list: &LiveryPaintList) -> Vec<usize> {
    let indices = list.commands().iter().enumerate().filter_map(|(index, command)| {
        matches!(command, PaintCmd::DrawText(text) if text.color == RED && !text.glyphs.is_empty()).then_some(index)
    }).collect::<Vec<_>>();
    assert!(
        !indices.is_empty(),
        "the fixture must paint real shaped source text"
    );
    indices
}

#[test]
fn normal_highlighted_text_keeps_its_selection_visible_over_opaque_tokens() {
    let (dom, mut layout, field, _) = fixture(false);
    let list = emitted(&dom, &mut layout, field);
    let selection = rect_index(&list, SELECTION);
    let caret = rect_index(&list, RED);
    for glyph in glyph_indices(&list) {
        assert!(
            glyph < selection && selection < caret,
            "selection {selection}, glyph {glyph}, caret {caret}"
        );
    }
    let tokens = list.commands().iter().enumerate().filter_map(|(index, command)| {
        matches!(command, PaintCmd::DrawRect(rect) if rect.color == ColorF { r: 1.0, g: 1.0, b: 0.0, a: 1.0 }).then_some(index)
    }).collect::<Vec<_>>();
    assert!(
        !tokens.is_empty(),
        "the fixture must paint opaque syntax backgrounds"
    );
    assert!(tokens.into_iter().all(|index| index < selection));
    let PaintCmd::DrawRect(selection) = &list.commands()[selection] else {
        unreachable!()
    };
    assert!(selection.placement.bounds.width() > 0.0 && selection.placement.bounds.height() > 0.0);
}

#[test]
fn a_later_dom_overlay_covers_both_source_selection_and_caret() {
    let (dom, mut layout, field, _) = fixture(true);
    let list = emitted(&dom, &mut layout, field);
    let overlay = rect_index(&list, BLUE);
    assert!(rect_index(&list, SELECTION) < overlay);
    assert!(rect_index(&list, RED) < overlay);
    for glyph in glyph_indices(&list) {
        assert!(glyph < overlay);
    }
}

#[test]
fn the_original_window_overlay_order_is_a_failing_control() {
    let (dom, mut layout, field, _) = fixture(true);
    let mut legacy = layout.emit_paint_list_with_leaves(
        &dom,
        DeviceIntSize::new(300, 220),
        None,
        |_| None,
        |_| None,
    );
    // The previous frame pipeline appended viewport rectangles after every DOM
    // command. Keep that control to prove the ordering check detects the bug.
    for rect in layout.selection_rects(&dom, field, 6, 10) {
        legacy.push_overlay_rect(
            LayoutRect::from_origin_and_size(
                LayoutPoint::new(rect.x, rect.y),
                LayoutSize::new(rect.width, rect.height),
            ),
            SELECTION,
        );
    }
    let caret = layout
        .caret_rect_for_position(
            &dom,
            field,
            VisualCaret {
                byte: 10,
                affinity: VisualAffinity::Downstream,
            },
            2.0,
        )
        .unwrap();
    legacy.push_overlay_rect(
        LayoutRect::from_origin_and_size(
            LayoutPoint::new(caret.x, caret.y),
            LayoutSize::new(caret.width, caret.height),
        ),
        RED,
    );
    assert!(rect_index(&legacy, BLUE) < rect_index(&legacy, SELECTION));
    assert!(rect_index(&legacy, BLUE) < rect_index(&legacy, RED));
    let corrected = emitted(&dom, &mut layout, field);
    assert!(rect_index(&corrected, SELECTION) < rect_index(&corrected, BLUE));
    assert!(rect_index(&corrected, RED) < rect_index(&corrected, BLUE));
}

#[test]
fn a_positive_z_overlay_inside_a_field_also_covers_its_decorations() {
    let (mut dom, mut layout, field, _) = fixture(false);
    dom.set_attribute(field, name("style"),
        "position:relative;z-index:0;width:180px;height:48px;overflow:auto;white-space:pre;font-size:16px;line-height:24px;color:rgb(255,0,0);background:rgb(0,255,0);");
    element(
        &mut dom,
        field,
        "div",
        "position:absolute;left:0;top:0;width:220px;height:100px;z-index:20;background:rgb(0,0,255);",
    );
    layout.rebuild(&dom, 300.0, 220.0);
    let list = emitted(&dom, &mut layout, field);
    let overlay = rect_index(&list, BLUE);
    assert!(rect_index(&list, SELECTION) < overlay);
    assert!(rect_index(&list, RED) < overlay);
}

#[test]
fn raw_decoration_geometry_inherits_nested_scroll_clip_opacity_and_transform_once() {
    let (dom, mut layout, field, outer) = fixture(true);
    layout.set_element_scroll(
        &dom,
        HashMap::from([(field, (0.0, 12.0)), (outer, (0.0, 8.0))]),
    );
    assert_eq!(layout.element_scroll()[&field], (0.0, 12.0));
    assert_eq!(layout.element_scroll()[&outer], (0.0, 8.0));
    layout.viewport_scroll = (3.0, 4.0);
    let raw = layout.caret_rect_at(&dom, field, 10).unwrap();
    let viewport = layout
        .caret_rect_for_position(
            &dom,
            field,
            VisualCaret {
                byte: 10,
                affinity: VisualAffinity::Downstream,
            },
            2.0,
        )
        .unwrap();
    assert!((raw.y - viewport.y - 24.0).abs() < 0.01);
    let list = emitted(&dom, &mut layout, field);
    let caret_index = rect_index(&list, RED);
    let selection_index = rect_index(&list, SELECTION);
    let PaintCmd::DrawRect(caret) = &list.commands()[caret_index] else {
        unreachable!()
    };
    assert!((caret.placement.bounds.min.x - raw.x).abs() < 0.01);
    assert!(
        (caret.placement.bounds.min.y - raw.y).abs() < 0.01,
        "the field's paint context, not the rectangle, applies its scroll"
    );
    let mut clips = 0;
    let mut layers = 0;
    let mut transforms = 0;
    let mut scrolls = Vec::new();
    for (index, command) in list.commands().iter().enumerate() {
        match command {
            PaintCmd::PushClip(_) => clips += 1,
            PaintCmd::PopClip => clips -= 1,
            PaintCmd::PushLayer(_) => layers += 1,
            PaintCmd::PopLayer => layers -= 1,
            PaintCmd::PushTransform(spec) => {
                transforms += 1;
                scrolls.push(spec.transform.m42);
            },
            PaintCmd::PopTransform => {
                transforms -= 1;
                scrolls.pop();
            },
            _ => {},
        }
        if index == selection_index || index == caret_index {
            assert!(
                clips >= 2 && layers >= 1 && transforms >= 4,
                "decoration escaped its CSS context: clips={clips}, layers={layers}, transforms={transforms}"
            );
            assert!(scrolls.contains(&-12.0) && scrolls.contains(&-8.0));
        }
    }
    assert_eq!((clips, layers, transforms), (0, 0, 0));
}

#[test]
fn an_empty_field_still_paints_a_caret_inside_its_scope() {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    let field = element(
        &mut dom,
        root,
        "div",
        "width:100px;height:30px;overflow:hidden;color:rgb(255,0,0);",
    );
    let mut layout = OwnedLayout::new(&dom, &[""], 300.0, 220.0, &[], &Default::default());
    let list = layout.emit_paint_list_with_leaves(
        &dom,
        DeviceIntSize::new(300, 220),
        Some((
            field,
            VisualCaret {
                byte: 0,
                affinity: VisualAffinity::Downstream,
            },
            None,
        )),
        |_| None,
        |_| None,
    );
    let caret = rect_index(&list, RED);
    assert!(
        list.commands()[..caret]
            .iter()
            .any(|command| matches!(command, PaintCmd::PushClip(_)))
    );
    assert!(
        list.commands()[caret..]
            .iter()
            .any(|command| matches!(command, PaintCmd::PopClip))
    );
}
