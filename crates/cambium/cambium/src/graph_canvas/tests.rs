/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;
use crate::{AnyView, DomHandle, GenetAppRunner};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LocalName, Namespace};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Default)]
struct State {
    clicked: Vec<u8>,
    hovered: Option<u8>,
    focused: Option<u8>,
    dragged: Vec<GraphCanvasNodeDrag<u8>>,
    relation_clicked: Vec<String>,
    relation_hovered: Option<String>,
    expanded: bool,
    focus_request: Option<u8>,
}

type TestView = Box<dyn AnyView<State, (), GenetCtx, GenetElement>>;

fn model(hovered: Option<u8>, focused: Option<u8>) -> GraphCanvasSwatch<u8, &'static str> {
    let mut swatch = GraphCanvasSwatch::new(
        77,
        GraphCanvasSubgraph {
            nodes: vec![
                GraphCanvasNode {
                    id: 1,
                    kind: "document",
                    position: (0.1, 0.5),
                    label: "First node".into(),
                    key: None,
                },
                GraphCanvasNode {
                    id: 2,
                    kind: "person",
                    position: (0.9, 0.5),
                    label: "Second node".into(),
                    key: None,
                },
            ],
            edges: vec![GraphCanvasEdge { from: 1, to: 2 }],
        },
    )
    .with_size(240, 112);
    swatch.selected = Some(1);
    swatch.focus = focused;
    swatch.hovered = hovered;
    swatch
}

fn view(state: &State) -> TestView {
    let swatch = model(state.hovered, state.focused);
    Box::new(graph_canvas_swatch_with_focus(
        &swatch,
        |state: &mut State, id| state.clicked.push(id),
        |state: &mut State, id| state.hovered = id,
        |state: &mut State, id| state.focused = id,
        |state: &mut State| state.expanded = true,
    ))
}

#[derive(Default)]
struct ViewportState {
    pans: Vec<(f32, f32)>,
    zooms: Vec<f32>,
}

type ViewportView = Box<dyn AnyView<ViewportState, (), GenetCtx, GenetElement>>;

fn viewport_view(_state: &ViewportState) -> ViewportView {
    let swatch = model(None, None);
    Box::new(graph_canvas(
        &swatch,
        |state: &mut ViewportState, event: GraphCanvasEvent<u8>| match event {
            GraphCanvasEvent::Pan { delta } => state.pans.push(delta),
            GraphCanvasEvent::Zoom { factor } => state.zooms.push(factor),
            _ => {},
        },
    ))
}

/// The viewport stays consumer state: a background drag reports pan deltas
/// in canvas fractions, a wheel notch reports a zoom factor, and neither
/// mutates the swatch.
#[test]
fn background_drag_pans_and_wheel_zooms() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner =
        GenetAppRunner::<_, _, _, ()>::new(dom.clone(), viewport_view, ViewportState::default());
    let root = runner.root();
    let canvas = find_attr(&dom.borrow(), root, "class", "graph-canvas-swatch")
        .expect("canvas root element");

    let size = (240.0, 112.0);
    runner.dispatch_pointer_down(
        canvas,
        PointerEvent::new(PointerPhase::Down, (10.0, 10.0), size),
    );
    runner.dispatch_pointer_move(PointerEvent::new(PointerPhase::Move, (34.0, 24.0), size));
    runner.dispatch_pointer_up(PointerEvent::new(PointerPhase::Up, (34.0, 24.0), size));
    assert_eq!(runner.state().pans, vec![(24.0 / 240.0, 14.0 / 112.0)]);

    runner.dispatch_wheel(canvas, WheelEvent::new((0.0, 3.0), (5.0, 5.0), size));
    runner.dispatch_wheel(canvas, WheelEvent::new((0.0, -3.0), (5.0, 5.0), size));
    assert_eq!(runner.state().zooms, vec![1.2, 1.0 / 1.2]);
}

#[derive(Default)]
struct WheelState {
    wheels: Vec<((f32, f32), (f32, f32), (f32, f32), bool, bool)>,
    others: usize,
    take: bool,
}

type WheelView = Box<dyn AnyView<WheelState, (), GenetCtx, GenetElement>>;

fn wheel_view(state: &WheelState) -> WheelView {
    let swatch = model(None, None);
    let _ = state;
    Box::new(graph_canvas_with_wheel(
        &swatch,
        |state: &mut WheelState, input: GraphCanvasInput<u8>| match input {
            GraphCanvasInput::Wheel(event) => {
                state.wheels.push((
                    event.delta,
                    event.local,
                    event.size,
                    event.modifiers.ctrl,
                    event.modifiers.meta,
                ));
                if state.take {
                    event.prevent_default();
                }
            },
            GraphCanvasInput::Event(_) => state.others += 1,
        },
    ))
}

/// A consumer that opts in sees the whole wheel notch (both axes, where the
/// pointer was, the modifiers) and decides whether the host's scroll default
/// runs. A wheel that is not asked for as a zoom is not turned into one.
#[test]
fn the_wheel_reaches_an_opting_in_consumer_whole() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner =
        GenetAppRunner::<_, _, _, ()>::new(dom.clone(), wheel_view, WheelState::default());
    let root = runner.root();
    let canvas = find_attr(&dom.borrow(), root, "class", "graph-canvas-swatch")
        .expect("canvas root element");
    let size = (240.0, 112.0);

    let mut event = WheelEvent::new((7.0, -3.0), (100.0, 40.0), size);
    event.modifiers.ctrl = true;
    runner.dispatch_wheel(canvas, event);
    assert_eq!(
        runner.state().wheels,
        vec![((7.0, -3.0), (100.0, 40.0), size, true, false)]
    );
    assert!(!runner.default_prevented(), "the consumer did not take it");

    runner.update(|state| state.take = true);
    runner.dispatch_wheel(canvas, WheelEvent::new((0.0, 0.0), (1.0, 2.0), size));
    assert_eq!(
        runner.state().wheels.len(),
        2,
        "a zero notch is still reported"
    );
    assert!(runner.default_prevented(), "the consumer took it");
    assert_eq!(runner.state().others, 0);
}

/// The established form is unchanged: a notch is a fixed zoom factor, the
/// host's scroll default is suppressed, and a zero notch reports nothing.
#[test]
fn the_established_wheel_zoom_is_unchanged() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner =
        GenetAppRunner::<_, _, _, ()>::new(dom.clone(), viewport_view, ViewportState::default());
    let root = runner.root();
    let canvas = find_attr(&dom.borrow(), root, "class", "graph-canvas-swatch")
        .expect("canvas root element");
    let size = (240.0, 112.0);
    runner.dispatch_wheel(canvas, WheelEvent::new((9.0, 3.0), (5.0, 5.0), size));
    assert!(runner.default_prevented());
    runner.dispatch_wheel(canvas, WheelEvent::new((9.0, 0.0), (5.0, 5.0), size));
    assert!(
        !runner.default_prevented(),
        "a zero notch is left to the host"
    );
    assert_eq!(runner.state().zooms, vec![1.2]);
}

fn attr<'a>(dom: &'a ScriptedDom, node: NodeId, name: &str) -> Option<&'a str> {
    dom.attribute(node, &Namespace::from(""), &LocalName::from(name))
}

fn find_attr(dom: &ScriptedDom, root: NodeId, name: &str, value: &str) -> Option<NodeId> {
    if attr(dom, root, name) == Some(value) {
        return Some(root);
    }
    dom.dom_children(root)
        .find_map(|child| find_attr(dom, child, name, value))
}

#[test]
fn node_targets_route_click_hover_and_expand() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(dom.clone(), view, State::default());
    let root = runner.root();
    let first =
        find_attr(&dom.borrow(), root, "aria-label", "First node").expect("first node target");
    runner.dispatch_hover(
        first,
        HoverEvent::new(HoverPhase::Enter, (2.0, 2.0), (20.0, 20.0)),
    );
    assert_eq!(runner.state().hovered, Some(1));
    runner.dispatch_click(first, PointerClick::at((2.0, 2.0)));
    assert_eq!(runner.state().clicked, [1]);
    assert_eq!(runner.focus(), Some(first));
    assert_eq!(runner.state().focused, Some(1));

    let expand =
        find_attr(&dom.borrow(), root, "aria-label", "Expand graph").expect("expand target");
    runner.dispatch_click(expand, PointerClick::at((2.0, 2.0)));
    assert!(runner.state().expanded);
}

fn attributed_view(state: &State) -> TestView {
    let swatch = model(state.hovered, state.focused)
        .with_node_attr(2, "aria-expanded", "false")
        .with_node_attr(2, "aria-expanded", "true")
        .with_node_attr(2, "data-tier", "session")
        .with_node_attr(2, "aria-label", "Not the canvas's name")
        .with_focus_request(state.focus_request);
    Box::new(graph_canvas_swatch_with_focus(
        &swatch,
        |state: &mut State, id| state.clicked.push(id),
        |state: &mut State, id| state.hovered = id,
        |state: &mut State, id| state.focused = id,
        |state: &mut State| state.expanded = true,
    ))
}

/// A consumer's node kind rides on the shared target as attributes, and a
/// requested node takes focus once, without the canvas's own attributes
/// being overridden.
#[test]
fn node_attrs_and_a_focus_request_reach_one_nodes_target() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner =
        GenetAppRunner::<_, _, _, ()>::new(dom.clone(), attributed_view, State::default());
    let root = runner.root();
    let second = find_attr(&dom.borrow(), root, "aria-label", "Second node")
        .expect("the canvas keeps its own accessible name");
    let first = find_attr(&dom.borrow(), root, "aria-label", "First node").unwrap();
    {
        let dom = dom.borrow();
        assert_eq!(attr(&dom, second, "aria-expanded"), Some("true"));
        assert_eq!(attr(&dom, second, "data-tier"), Some("session"));
        assert_eq!(attr(&dom, first, "aria-expanded"), None);
    }
    assert_eq!(runner.focus(), None);
    runner.update(|state| state.focus_request = Some(2));
    let second = find_attr(&dom.borrow(), root, "aria-label", "Second node").unwrap();
    assert_eq!(runner.focus(), Some(second), "the request takes focus");
    let first = find_attr(&dom.borrow(), root, "aria-label", "First node").unwrap();
    runner.set_focus(Some(first));
    runner.update(|state| state.hovered = Some(1));
    assert_eq!(
        runner.focus(),
        Some(first),
        "a held request does not steal focus again"
    );
}

fn labelled_view(state: &State) -> TestView {
    let swatch = model(state.hovered, state.focused).with_node_labels(true);
    Box::new(graph_canvas_swatch_with_focus(
        &swatch,
        |state: &mut State, id| state.clicked.push(id),
        |state: &mut State, id| state.hovered = id,
        |state: &mut State, id| state.focused = id,
        |state: &mut State| state.expanded = true,
    ))
}

/// Collect the class attribute of every element whose *base* class is
/// `base`. Matching the first token rather than a prefix keeps the
/// `-labels` container out of a search for `-label` elements.
fn classes_of(dom: &ScriptedDom, root: NodeId, base: &str, out: &mut Vec<String>) {
    if let Some(class) = attr(dom, root, "class") {
        if class.split_whitespace().next() == Some(base) {
            out.push(class.to_string());
        }
    }
    for child in dom.dom_children(root) {
        classes_of(dom, child, base, out);
    }
}

fn short_labelled_view(_state: &State) -> TestView {
    let swatch = model(None, None)
        .with_node_labels(true)
        .with_visible_label(2, "Second")
        .with_label_placement(1, GraphLabelPlacement::Left);
    Box::new(graph_canvas_swatch_with_focus(
        &swatch,
        |state: &mut State, id| state.clicked.push(id),
        |state: &mut State, id| state.hovered = id,
        |state: &mut State, id| state.focused = id,
        |state: &mut State| state.expanded = true,
    ))
}

fn drag_view(state: &State) -> TestView {
    let swatch = model(state.hovered, state.focused);
    Box::new(graph_canvas_swatch_with_drag(
        &swatch,
        |state: &mut State, id| state.clicked.push(id),
        |state: &mut State, id| state.hovered = id,
        |state: &mut State, event| state.dragged.push(event),
        |state: &mut State| state.expanded = true,
    ))
}

fn relation_view(state: &State) -> TestView {
    let swatch = model(state.hovered, state.focused).with_relations(vec![
        GraphCanvasRelation {
            link_key: None,
            marks: sceno::LinkMarks::default(),
            strength: 1.0,
            id: "cites".into(),
            from: 1,
            to: 2,
            kind: "Citation".into(),
            label: "First node cites Second node".into(),
            route: Vec::new(),
            visible: true,
            emphasized: true,
        },
        GraphCanvasRelation {
            link_key: None,
            marks: sceno::LinkMarks::default(),
            strength: 1.0,
            id: "quotes".into(),
            from: 1,
            to: 2,
            kind: "Quotation".into(),
            label: "First node quotes Second node".into(),
            route: Vec::new(),
            visible: true,
            emphasized: false,
        },
        GraphCanvasRelation {
            link_key: None,
            marks: sceno::LinkMarks::default(),
            strength: 1.0,
            id: "hidden".into(),
            from: 1,
            to: 2,
            kind: "Hidden".into(),
            label: "This relation is hidden".into(),
            route: Vec::new(),
            visible: false,
            emphasized: false,
        },
    ]);
    Box::new(graph_canvas_swatch_with_drag_and_relations(
        &swatch,
        |state: &mut State, id| state.clicked.push(id),
        |state: &mut State, id| state.hovered = id,
        |state: &mut State, event| state.dragged.push(event),
        |state: &mut State, id| state.relation_clicked.push(id),
        |state: &mut State, id| state.relation_hovered = id,
        |state: &mut State| state.expanded = true,
    ))
}

/// The component form's app state: selection truth in, events out. There
/// is deliberately no hover or focus field here — that is the whole point.
#[derive(Default)]
struct AppState {
    selected: Option<u8>,
    events: Vec<GraphCanvasEvent<u8>>,
}

type AppView = Box<dyn AnyView<AppState, (), GenetCtx, GenetElement>>;

fn component_view(state: &AppState) -> AppView {
    let mut swatch = model(None, None);
    swatch.selected = state.selected;
    Box::new(graph_canvas(
        &swatch,
        |state: &mut AppState, event: GraphCanvasEvent<u8>| {
            if let GraphCanvasEvent::Activate(id) = &event {
                state.selected = Some(*id);
            }
            state.events.push(event);
        },
    ))
}

struct AtlasState {
    events: Vec<GraphAtlasEvent<u8>>,
    deferred: bool,
    builds: Rc<Cell<u32>>,
}

impl Default for AtlasState {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            deferred: false,
            builds: Rc::new(Cell::new(0)),
        }
    }
}

type AtlasView = Box<dyn AnyView<AtlasState, (), GenetCtx, GenetElement>>;

fn atlas_view(state: &AtlasState) -> AtlasView {
    use sceno::{Footprint, Rect, Size2, Vec2};

    state.builds.set(state.builds.get() + 1);

    let swatch = GraphCanvasSwatch::new(
        77,
        GraphCanvasSubgraph {
            nodes: vec![GraphCanvasNode {
                id: 1,
                kind: (),
                position: (0.5, 0.5),
                label: "Tower".into(),
                key: None,
            }],
            edges: Vec::new(),
        },
    )
    .with_size(200, 100)
    .with_expand(false)
    .with_atlas(
        GraphCanvasAtlas::new(Rect::new(Vec2::ZERO, Size2::new(100.0, 50.0))).with_fields(vec![
            GraphCanvasAtlasField {
                id: 1,
                anchor: Vec2::new(50.0, 25.0),
                footprint: Footprint::Polygon {
                    points: vec![
                        Vec2::new(-10.0, -10.0),
                        Vec2::new(10.0, 0.0),
                        Vec2::new(-10.0, 10.0),
                    ],
                },
                priority: 1,
                label: "Tower field".into(),
            },
        ]),
    );
    let swatch = if state.deferred {
        swatch.with_deferred_drag_rebuild(true)
    } else {
        swatch
    };
    Box::new(graph_atlas_swatch(
        &swatch,
        |state: &mut AtlasState, event| {
            state.events.push(event);
        },
    ))
}

#[path = "tests/labels.rs"]
mod labels;

#[path = "tests/relations.rs"]
mod relations;

#[path = "tests/interaction.rs"]
mod interaction;
