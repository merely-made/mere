// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use genet_scripted_dom::ScriptedDom;
use layout_dom_api::{LayoutDomMut, QualName};

fn fixture(sheet: &str) -> (ScriptedDom, OwnedLayout, NodeId, NodeId) {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    let a = dom.create_element(QualName::new(
        None,
        Namespace::from(""),
        LocalName::from("div"),
    ));
    let b = dom.create_element(QualName::new(
        None,
        Namespace::from(""),
        LocalName::from("div"),
    ));
    dom.append_child(root, a);
    dom.append_child(root, b);
    for node in [a, b] {
        let text = dom.create_text("Retained text with a second line of words.");
        dom.append_child(node, text);
    }
    let layout = OwnedLayout::new(&dom, &[sheet], 300.0, 100.0);
    (dom, layout, a, b)
}

#[test]
fn equal_hover_cascade_retains_geometry_text_generation_and_scroll() {
    let (dom, mut layout, a, b) = fixture(
        "div { width: 220px; height: 180px; color: black !important; } div:hover { color: red; }",
    );
    layout.set_viewport_scroll((0.0, 40.0));
    layout.set_element_scroll(HashMap::from([(a, (0.0, 12.0))]));
    let generation = layout.generation;
    let rect = layout.painted_rect(&dom, a);
    for hovered in [Some(a), Some(b), None, Some(a)] {
        assert!(!layout.set_interaction(&dom, hovered, None));
        assert_eq!(layout.generation, generation);
        assert_eq!(layout.painted_rect(&dom, a), rect);
        assert_eq!(layout.viewport_scroll(), (0.0, 40.0));
        assert_eq!(layout.element_scroll()[&a], (0.0, 12.0));
        assert_eq!(layout.layout_with_text_us, 0);
    }
}

#[test]
fn hover_geometry_changes_apply_and_revert_on_leave() {
    let (dom, mut layout, a, b) = fixture("div { height: 40px; } div:hover { height: 80px; }");
    let generation = layout.generation;
    let original_b = layout.painted_rect(&dom, b).unwrap();
    assert!(layout.set_interaction(&dom, Some(a), None));
    assert_eq!(layout.generation, generation + 1);
    assert_eq!(layout.painted_rect(&dom, a).unwrap().3, 80.0);
    assert_eq!(layout.painted_rect(&dom, b).unwrap().1, original_b.1 + 40.0);
    assert!(layout.set_interaction(&dom, None, None));
    assert_eq!(layout.painted_rect(&dom, b), Some(original_b));
}

#[test]
fn focus_changes_and_later_dom_rebuilds_keep_the_current_interaction() {
    let (mut dom, mut layout, a, b) = fixture("div { height: 40px; } div:focus { height: 90px; }");
    assert!(!layout.set_interaction(&dom, Some(a), None));
    assert!(layout.set_interaction(&dom, Some(a), Some(b)));
    assert_eq!(layout.painted_rect(&dom, b).unwrap().3, 90.0);
    dom.set_attribute(
        a,
        QualName::new(None, Namespace::from(""), LocalName::from("style")),
        "height: 60px",
    );
    layout.rebuild(&dom, 400.0, 100.0);
    assert_eq!(layout.painted_rect(&dom, a).unwrap().3, 60.0);
    assert_eq!(layout.painted_rect(&dom, b).unwrap().3, 90.0);
    let generation = layout.generation;
    assert!(!layout.set_interaction(&dom, None, Some(b)));
    assert_eq!(layout.generation, generation);
    assert!(layout.set_interaction(&dom, None, None));
    assert_eq!(layout.painted_rect(&dom, b).unwrap().3, 40.0);
}

#[test]
fn changed_color_is_not_mistaken_for_an_equal_cascade() {
    let (dom, mut layout, a, _) = fixture("div { color: black; } div:hover { color: red; }");
    let before = layout.styles.get(a).unwrap().clone();
    assert!(layout.set_interaction(&dom, Some(a), None));
    assert_ne!(layout.styles.get(a).unwrap(), &before);
    assert!(layout.set_interaction(&dom, None, None));
    assert_eq!(layout.styles.get(a).unwrap(), &before);
}

#[test]
fn hover_can_reflow_text_in_a_different_element() {
    let (dom, mut layout, a, b) =
        fixture("div { width: 120px; font-size: 12px; } div:hover + div { font-size: 24px; }");
    let height = layout.painted_rect(&dom, b).unwrap().3;
    assert!(layout.set_interaction(&dom, Some(a), None));
    assert!(layout.painted_rect(&dom, b).unwrap().3 > height);
    assert!(layout.set_interaction(&dom, None, None));
    assert_eq!(layout.painted_rect(&dom, b).unwrap().3, height);
}

#[test]
fn hover_without_dependent_rules_skips_cascade_even_when_focus_has_rules() {
    let (dom, mut layout, a, b) = fixture("div:focus { height: 90px; }");
    let generation = layout.generation;
    for hovered in [Some(a), Some(b), None] {
        assert!(!layout.set_interaction(&dom, hovered, None));
        assert_eq!(layout.stage_timings(), (0, 0, 0));
        assert_eq!(layout.generation, generation);
    }
    assert!(layout.set_interaction(&dom, None, Some(a)));
    assert_eq!(layout.painted_rect(&dom, a).unwrap().3, 90.0);
}

#[test]
fn escaped_commented_and_mixed_case_hover_rules_still_apply() {
    for selector in [r"div:\68 over", "div:/**/hover", "div:HoVeR"] {
        let sheet = format!("div {{ height: 40px; }} {selector} {{ height: 80px; }}");
        let (dom, mut layout, a, _) = fixture(&sheet);
        assert!(layout.set_interaction(&dom, Some(a), None), "{selector}");
        assert_eq!(layout.painted_rect(&dom, a).unwrap().3, 80.0, "{selector}");
    }
}
