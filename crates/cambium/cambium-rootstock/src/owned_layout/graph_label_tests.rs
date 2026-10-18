// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Resolve the actual graph label DOM through the host's retained text layout.
//! Box separation is independent of whether a clipped name is readable in full.

use std::{cell::RefCell, rc::Rc};

use cambium::{
    AnyView, DomHandle, GRAPH_CANVAS_SWATCH_CSS, GenetAppRunner, GenetCtx, GenetElement,
    GraphCanvasEdge, GraphCanvasNode, GraphCanvasSubgraph, GraphCanvasSwatch, HoverEvent,
    HoverPhase, graph_canvas_swatch_with_focus,
};
use genet_scripted_dom::ScriptedDom;
use layout_dom_api::{LayoutDomMut, QualName};

use super::*;

const SIZE: (f32, f32) = (300.0, 210.0);
const LABELS: [&str; 12] = [
    "Autumn rehearsal",
    "Dorian study",
    "Evening set",
    "Walking bass",
    "Bridge variation",
    "Quiet chorus",
    "Modal interchange",
    "Sunday session",
    "Turnaround study",
    "New arrangement",
    "Minor blues",
    "Final cadence",
];

#[derive(Default)]
struct State {
    focus: Option<usize>,
    hover: Option<usize>,
}

type TestView = Box<dyn AnyView<State, (), GenetCtx, GenetElement>>;
type Rect = (f32, f32, f32, f32);

fn radial_view(state: &State) -> TestView {
    let nodes = LABELS
        .iter()
        .enumerate()
        .map(|(id, label)| {
            let angle = id as f32 * std::f32::consts::TAU / LABELS.len() as f32;
            GraphCanvasNode {
                id,
                kind: (),
                position: (0.5 + 0.38 * angle.cos(), 0.5 + 0.38 * angle.sin()),
                label: (*label).into(),
                key: Some(format!("related-{id}")),
            }
        })
        .collect();
    let mut swatch = GraphCanvasSwatch::new(
        77,
        GraphCanvasSubgraph {
            nodes,
            edges: (0..LABELS.len())
                .map(|from| GraphCanvasEdge {
                    from,
                    to: (from + 1) % LABELS.len(),
                })
                .collect(),
        },
    )
    .with_size(SIZE.0 as u32, SIZE.1 as u32)
    .with_expand(false)
    .with_node_labels(true);
    // Exercise the default collision policy, including mutual emphasis.
    swatch.selected = Some(0);
    swatch.focus = state.focus;
    swatch.hovered = state.hover;
    Box::new(graph_canvas_swatch_with_focus(
        &swatch,
        |_: &mut State, _: usize| {},
        |state: &mut State, id| state.hover = id,
        |state: &mut State, id| state.focus = id,
        |_: &mut State| {},
    ))
}

fn attr<'a>(dom: &'a ScriptedDom, node: NodeId, name: &str) -> Option<&'a str> {
    dom.attribute(node, &Namespace::from(""), &LocalName::from(name))
}

fn by_class(dom: &ScriptedDom, class: &str) -> Vec<NodeId> {
    let mut found = Vec::new();
    walk(dom, dom.document(), &mut |node| {
        if attr(dom, node, "class")
            .is_some_and(|value| value.split_whitespace().any(|c| c == class))
        {
            found.push(node);
        }
    });
    found
}

fn target(dom: &ScriptedDom, id: usize) -> NodeId {
    by_class(dom, "graph-canvas-swatch-node")
        .into_iter()
        .find(|&node| attr(dom, node, "data-key") == Some(format!("related-{id}").as_str()))
        .expect("full accessible node target")
}

fn overlaps(a: Rect, b: Rect) -> bool {
    // Permit subpixel rounding at a shared edge, but no positive-area overlap.
    const EPS: f32 = 0.01;
    a.0 < b.0 + b.2 - EPS && b.0 < a.0 + a.2 - EPS && a.1 < b.1 + b.3 - EPS && b.1 < a.1 + a.3 - EPS
}

fn assert_radial_geometry(dom: &ScriptedDom, layout: &OwnedLayout) -> Vec<String> {
    let roots = by_class(dom, "graph-canvas-swatch");
    assert_eq!(roots.len(), 1);
    // The swatch has a border outside its declared 300x210 content area.
    // Labels are positioned within this actual retained overlay rectangle.
    let layers = by_class(dom, "graph-canvas-swatch-labels");
    assert_eq!(layers.len(), 1);
    let canvas = layout.painted_rect(dom, layers[0]).unwrap();
    assert_eq!((canvas.2, canvas.3), SIZE);
    let labels = by_class(dom, "graph-canvas-swatch-label");
    assert!(
        labels.len() >= 3,
        "nonvacuous retained label fixture: {labels:?}"
    );
    let targets = by_class(dom, "graph-canvas-swatch-node");
    assert_eq!(targets.len(), LABELS.len());
    for (id, label) in LABELS.iter().enumerate() {
        let node = target(dom, id);
        assert_eq!(attr(dom, node, "aria-label"), Some(*label));
        assert_eq!(
            targets
                .iter()
                .filter(|&&node| attr(dom, node, "aria-label") == Some(*label))
                .count(),
            1,
            "exactly one full accessible target for {label}"
        );
        assert!(layout.painted_rect(dom, node).is_some());
    }
    let rects: Vec<_> = labels
        .iter()
        .map(|&node| {
            assert_eq!(attr(dom, node, "aria-hidden"), Some("true"));
            let rect = layout.painted_rect(dom, node).expect("retained label box");
            assert!(rect.2 > 0.0 && (rect.3 - 14.0).abs() < 0.01, "{rect:?}");
            assert!(
                rect.0 >= canvas.0 - 0.01
                    && rect.1 >= canvas.1 - 0.01
                    && rect.0 + rect.2 <= canvas.0 + canvas.2 + 0.01
                    && rect.1 + rect.3 <= canvas.1 + canvas.3 + 0.01,
                "label {} outside {canvas:?}: {rect:?}",
                node_text(dom, node)
            );
            let visible = layout
                .visible_rect(dom, node)
                .expect("visible retained label");
            for (painted, clipped) in [rect.0, rect.1, rect.2, rect.3]
                .into_iter()
                .zip([visible.0, visible.1, visible.2, visible.3])
            {
                assert!(
                    (painted - clipped).abs() < 0.01,
                    "{rect:?} clipped to {visible:?}"
                );
            }
            rect
        })
        .collect();
    for (index, &a) in rects.iter().enumerate() {
        for (other, &b) in rects.iter().enumerate().skip(index + 1) {
            assert!(
                !overlaps(a, b),
                "retained labels {:?} {a:?} and {:?} {b:?} overlap",
                node_text(dom, labels[index]),
                node_text(dom, labels[other])
            );
        }
    }
    labels
        .iter()
        .map(|&node| attr(dom, node, "class").unwrap().to_owned())
        .collect()
}

#[test]
fn radial_graph_labels_stay_disjoint_inside_retained_layout_after_emphasis() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(dom.clone(), radial_view, State::default());
    let fonts = [crate::HostFont {
        family: Some("sans-serif".into()),
        bytes: include_bytes!("../../../examples/genet_web_smoke/assets/Roboto-Regular.ttf")
            .to_vec(),
    }];
    let mut layout = OwnedLayout::new(
        &*dom.borrow(),
        &[
            GRAPH_CANVAS_SWATCH_CSS,
            ".graph-canvas-swatch-label { font-family:sans-serif; }",
        ],
        SIZE.0,
        SIZE.1,
        &fonts,
        &Default::default(),
    );
    assert!(layout.font_registration_errors().is_empty());
    let initial = assert_radial_geometry(&dom.borrow(), &layout);
    assert!(
        initial
            .iter()
            .any(|class| class.split_whitespace().any(|c| c == "selected"))
    );
    // Discard initial construction mutations before testing the retained update.
    dom.borrow_mut().drain_mutations(&mut Vec::new());
    let focused = target(&dom.borrow(), 6);
    runner.set_focus(Some(focused));
    let hovered = target(&dom.borrow(), 9);
    runner.dispatch_hover(
        hovered,
        HoverEvent::new(HoverPhase::Enter, (1.0, 1.0), SIZE),
    );
    assert_eq!(runner.state().focus, Some(6));
    assert_eq!(runner.state().hover, Some(9));
    let mut mutations = Vec::new();
    dom.borrow_mut().drain_mutations(&mut mutations);
    assert!(
        !mutations.is_empty(),
        "emphasis must update the actual graph DOM"
    );
    layout.rebuild(&*dom.borrow(), SIZE.0, SIZE.1, &mutations);
    let emphasized = assert_radial_geometry(&dom.borrow(), &layout);
    for modifier in ["selected", "focused", "hovered"] {
        assert!(
            emphasized
                .iter()
                .any(|class| class.split_whitespace().any(|c| c == modifier)),
            "fixture must retain a {modifier} label: {emphasized:?}"
        );
    }
}

#[test]
fn oversized_ahem_graph_label_has_a_retained_clip_boundary() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let _runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |_: &State| -> TestView {
            let swatch = GraphCanvasSwatch::new(
                78,
                GraphCanvasSubgraph {
                    nodes: vec![GraphCanvasNode {
                        id: 0,
                        kind: (),
                        position: (0.5, 0.5),
                        label: "XXXXX".into(),
                        key: None,
                    }],
                    edges: Vec::new(),
                },
            )
            .with_size(300, 210)
            .with_expand(false)
            .with_node_labels(true);
            Box::new(graph_canvas_swatch_with_focus(
                &swatch,
                |_: &mut State, _: usize| {},
                |_: &mut State, _: Option<usize>| {},
                |_: &mut State, _: Option<usize>| {},
                |_: &mut State| {},
            ))
        },
        State::default(),
    );
    let mut dom = dom.borrow_mut();
    let probe = dom.create_element(QualName::new(
        None,
        Namespace::from(""),
        LocalName::from("span"),
    ));
    dom.set_attribute(probe, QualName::new(None, Namespace::from(""), LocalName::from("style")),
        "position:absolute;left:0;top:0;display:block;font-family:GraphLabelAhem;font-size:20px;white-space:nowrap;");
    let text = dom.create_text("XXXXX");
    dom.append_child(probe, text);
    let root = dom.document();
    dom.append_child(root, probe);
    let fonts = [crate::HostFont {
        family: Some("GraphLabelAhem".into()),
        bytes: include_bytes!("../../../../../ports/pelt/examples/Ahem.ttf").to_vec(),
    }];
    let layout = OwnedLayout::new(
        &*dom,
        &[
            GRAPH_CANVAS_SWATCH_CSS,
            ".graph-canvas-swatch-label { font-family:GraphLabelAhem;font-size:20px; }",
        ],
        SIZE.0,
        SIZE.1,
        &fonts,
        &Default::default(),
    );
    assert!(layout.font_registration_errors().is_empty());
    let intrinsic = layout.painted_rect(&*dom, probe).unwrap();
    assert_eq!(
        intrinsic.2, 100.0,
        "positive control: five Ahem glyphs at 20px"
    );
    let labels = by_class(&dom, "graph-canvas-swatch-label");
    assert_eq!(labels.len(), 1);
    assert_eq!(computed_px(&layout.styles, labels[0], "font-size"), 20.0);
    let rect = layout.painted_rect(&*dom, labels[0]).unwrap();
    assert!(
        rect.2 > 0.0 && rect.2 < intrinsic.2,
        "{rect:?} vs {intrinsic:?}"
    );
    assert_eq!(rect.3, 14.0);
    assert_eq!(layout.content_clip(&*dom, labels[0]), Some(rect));
    assert_eq!(node_text(&*dom, labels[0]), "XXXXX");
    // This proves the retained clipping boundary, not full-name visibility or
    // the particular glyphs an ellipsis implementation chooses to paint.
}
