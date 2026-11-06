// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use genet_scripted_dom::ScriptedDom;
use layout_dom_api::{LayoutDomMut, LocalName, Namespace, QualName, ShadowRootInit};

fn name(local: &str) -> QualName {
    QualName::new(None, Namespace::from(""), LocalName::from(local))
}

fn element(dom: &mut ScriptedDom, parent: NodeId, tag: &str, css: &str) -> NodeId {
    let node = dom.create_element(name(tag));
    dom.set_attribute(node, name("style"), css);
    dom.append_child(parent, node);
    node
}

fn layout(dom: &ScriptedDom) -> OwnedLayout {
    OwnedLayout::new(
        &WindowDom::document(dom),
        &[""],
        300.0,
        200.0,
        &[],
        &HashMap::new(),
    )
}

fn rebuild(layout: &mut OwnedLayout, dom: &mut ScriptedDom) {
    let mut mutations = Vec::new();
    dom.drain_mutations(&mut mutations);
    layout.rebuild(&WindowDom::document(dom), 300.0, 200.0, &mutations);
}

#[test]
fn pending_reorder_and_removal_keep_the_displayed_hit_identity() {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    let css = "position:absolute;left:10px;top:10px;width:80px;height:40px;";
    let first = element(&mut dom, root, "div", css);
    let second = element(&mut dom, root, "div", css);
    let mut layout = layout(&dom);
    let displayed = layout.presented_input(&WindowDom::document(&dom));
    assert_eq!(displayed.hit_at(20.0, 20.0), Some(second));
    dom.move_before(root, first, None);
    rebuild(&mut layout, &mut dom);
    assert_eq!(
        layout.hit_test(&WindowDom::document(&dom), 20.0, 20.0),
        Some(first)
    );
    assert_eq!(displayed.hit_at(20.0, 20.0), Some(second));
    assert!(!displayed.matches_dom(&WindowDom::document(&dom)));
    assert!(layout.generation > displayed.layout_generation());
    dom.remove(second);
    assert!(!WindowDom::document(&dom).contains(second));
    assert_eq!(displayed.hit_at(20.0, 20.0), Some(second));
    assert!(displayed.local_in(second, (20.0, 20.0)).is_some());
}

struct NeverDraw;
impl crate::producer::TextureProducer for NeverDraw {
    fn render(
        &mut self,
        _: &crate::producer::ProducerContext<'_>,
    ) -> Option<crate::producer::ProducedTexture> {
        panic!("input snapshot tests do not render")
    }
}

#[test]
fn transformed_producer_content_admission_uses_displayed_geometry() {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    let leaf = element(
        &mut dom,
        root,
        "custom-leaf",
        "position:absolute;left:20px;top:10px;width:40px;height:30px;border:4px solid black;padding:6px;transform:scale(2);transform-origin:0 0;",
    );
    dom.set_attribute(leaf, name("key"), "7");
    let mut registry = ProducerRegistry::new();
    registry.register(7, NeverDraw, &[]).unwrap();
    let mut layout = layout(&dom);
    let view = WindowDom::document(&dom);
    let displayed = layout.presented_input(&view);
    let point = (50.0, 40.0);
    let local = layout.local_coordinates(&view, leaf, point).unwrap();
    assert_eq!(displayed.local_in(leaf, point), Some(local));
    assert!(displayed.admits_producer_pointer(leaf, point, &registry));
    assert!(!displayed.admits_producer_pointer(leaf, (21.0, 11.0), &registry));
    assert_eq!(displayed.hit_at(point.0, point.1), Some(leaf));
    dom.set_attribute(
        leaf,
        name("style"),
        "position:absolute;left:180px;top:100px;width:40px;height:30px;transform:scale(.5);",
    );
    rebuild(&mut layout, &mut dom);
    assert_ne!(
        layout.hit_test(&WindowDom::document(&dom), point.0, point.1),
        Some(leaf)
    );
    assert_eq!(displayed.local_in(leaf, point), Some(local));
    assert!(displayed.admits_producer_pointer(leaf, point, &registry));
}

#[test]
fn element_and_document_scroll_changes_do_not_move_the_displayed_plane() {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    let scroller = element(
        &mut dom,
        root,
        "div",
        "width:100px;height:70px;overflow:auto;",
    );
    let first = element(&mut dom, scroller, "div", "width:100px;height:100px;");
    let second = element(&mut dom, scroller, "div", "width:100px;height:100px;");
    element(&mut dom, root, "div", "height:600px;");
    let mut layout = layout(&dom);
    let view = WindowDom::document(&dom);
    layout.set_element_scroll(&view, HashMap::from([(scroller, (0.0, 20.0))]));
    layout.set_viewport_scroll((0.0, 10.0));
    let displayed = layout.presented_input(&view);
    let before = layout.local_coordinates(&view, first, (10.0, 10.0));
    assert_eq!(displayed.hit_at(10.0, 10.0), Some(first));
    assert_eq!(displayed.local_in(first, (10.0, 10.0)), before);
    assert_eq!(
        displayed.hit_at(10.0, 90.0),
        layout.hit_test(&view, 10.0, 90.0),
        "ancestor clip stays with Genet"
    );
    layout.set_element_scroll(&view, HashMap::from([(scroller, (0.0, 120.0))]));
    layout.set_viewport_scroll((0.0, 20.0));
    assert_eq!(layout.hit_test(&view, 10.0, 10.0), Some(second));
    assert_eq!(displayed.hit_at(10.0, 10.0), Some(first));
    assert_eq!(displayed.local_in(first, (10.0, 10.0)), before);
    assert!(
        displayed.matches_dom(&view),
        "scroll has no text/DOM mutation"
    );
}

#[test]
fn frozen_dom_retains_scoped_light_shadow_slot_attribute_and_text_facts() {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    let window = element(&mut dom, root, "div", "width:100px;");
    let outside = element(&mut dom, root, "div", "height:20px;");
    let host = element(&mut dom, window, "div", "");
    let light = element(&mut dom, host, "span", "");
    let text = dom.create_text("displayed text");
    dom.append_child(light, text);
    let unslotted = element(&mut dom, host, "span", "");
    dom.set_attribute(unslotted, name("slot"), "missing");
    let shadow = dom.attach_shadow_unchecked(host, ShadowRootInit::default(), false);
    let slot = element(&mut dom, shadow, "slot", "");
    let fallback = element(&mut dom, slot, "i", "");
    let view = WindowDom::new(&dom, window);
    let frozen = FrozenDom::capture(&view);
    assert_eq!(frozen.document(), window);
    assert_eq!(frozen.parent(window), None);
    assert!(frozen.element_name(window).is_none());
    assert!(!frozen.is_live(outside));
    assert!(frozen.is_live(unslotted) && frozen.is_live(fallback));
    assert_eq!(
        frozen.dom_children(host).collect::<Vec<_>>(),
        vec![light, unslotted]
    );
    assert_eq!(frozen.flat_children(host).collect::<Vec<_>>(), vec![slot]);
    assert_eq!(frozen.flat_children(slot).collect::<Vec<_>>(), vec![light]);
    assert_eq!(frozen.assigned_slot(light), Some(slot));
    assert_eq!(frozen.assigned_nodes(slot), vec![light]);
    assert_eq!(frozen.shadow_roots(), vec![shadow]);
    assert_eq!(frozen.shadow_host(shadow), Some(host));
    assert_eq!(frozen.shadow_root(host), Some(shadow));
    assert_eq!(frozen.shadow_init(host), dom.shadow_init(host));
    dom.set_text(text, "pending replacement");
    dom.set_attribute(unslotted, name("slot"), "");
    dom.remove(host);
    assert_eq!(frozen.text(text), Some("displayed text"));
    assert_eq!(
        frozen.attribute(unslotted, &Namespace::from(""), &LocalName::from("slot")),
        Some("missing")
    );
    assert_eq!(frozen.flat_children(slot).collect::<Vec<_>>(), vec![light]);
}

#[test]
fn matching_rejects_fresh_documents_roots_and_mutations_but_not_log_drains() {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    let child = element(&mut dom, root, "div", "height:20px;");
    let layout = layout(&dom);
    let displayed = layout.presented_input(&WindowDom::document(&dom));
    let sequence = displayed.dom_sequence();
    assert!(displayed.matches_dom(&WindowDom::document(&dom)));
    dom.drain_mutations(&mut Vec::new());
    assert!(displayed.matches_dom(&WindowDom::document(&dom)));
    assert_eq!(dom_sequence(&WindowDom::document(&dom)), sequence);
    assert!(!displayed.matches_dom(&WindowDom::new(&dom, child)));
    let fresh = ScriptedDom::new();
    assert!(!displayed.matches_dom(&WindowDom::document(&fresh)));
    dom.set_attribute(child, name("title"), "new");
    assert!(!displayed.matches_dom(&WindowDom::document(&dom)));
    let foreign = fresh.document();
    assert!(displayed.local_in(foreign, (0.0, 0.0)).is_none());
    assert!(!displayed.admits_producer_pointer(foreign, (0.0, 0.0), &ProducerRegistry::new()));
}

#[test]
fn caret_uses_retained_text_frames_after_the_live_text_changes() {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    let field = element(
        &mut dom,
        root,
        "div",
        "width:200px;height:40px;font-size:16px;line-height:24px;",
    );
    let text = dom.create_text("displayed words");
    dom.append_child(field, text);
    let mut layout = layout(&dom);
    let view = WindowDom::document(&dom);
    let displayed = layout.presented_input(&view);
    let expected = layout.caret_position_at_point(&view, field, 30.0, 10.0);
    assert!(
        expected.is_some(),
        "the fixture must retain an actual text frame"
    );
    assert_eq!(displayed.caret_position(field, 30.0, 10.0), expected);
    dom.set_text(text, "");
    rebuild(&mut layout, &mut dom);
    assert!(!displayed.matches_dom(&WindowDom::document(&dom)));
    assert_eq!(displayed.caret_position(field, 30.0, 10.0), expected);
    assert!(
        layout
            .caret_position_at_point(&WindowDom::document(&dom), field, 30.0, 10.0)
            .is_none()
    );
    assert!(displayed.caret_position(field, f32::NAN, 0.0).is_none());
}

#[test]
fn current_layout_proof_rejects_scroll_rebuild_and_same_generation_recreation() {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    element(&mut dom, root, "div", "height:600px;");
    let mut retained = layout(&dom);
    let displayed = retained.presented_input(&WindowDom::document(&dom));
    assert!(displayed.matches_layout(&retained));
    assert!(!displayed.matches_layout(&layout(&dom)));
    retained.set_viewport_scroll((0.0, 10.0));
    assert!(!displayed.matches_layout(&retained));
    retained.set_viewport_scroll((0.0, 0.0));
    assert!(displayed.matches_layout(&retained));
    rebuild(&mut retained, &mut dom);
    assert!(!displayed.matches_layout(&retained));
}
