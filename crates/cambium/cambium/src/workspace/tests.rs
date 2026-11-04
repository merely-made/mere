/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use genet_scripted_dom::{NodeId, ScriptedDom};
use workbench::{
    ContentSource, DocumentRef, DropTarget, FloatEvent, SplitAxis, TabStack, TileBranch, TilePath,
    TileTree,
};

use super::*;
use crate::{
    AnyView, COMPONENT_PROBE_ATTR, ComponentView, DomHandle, GenetAppRunner, GenetMultiRunner,
    PointerClick, component, content_target, el, on_click,
};

type TestView = Box<dyn AnyView<State, (), GenetCtx, GenetElement>>;
/// A runner over one workspace view, plus the DOM it drew into.
type Harness = (
    DomHandle,
    GenetAppRunner<State, fn(&State) -> TestView, TestView, ()>,
);

struct State {
    workspace: Workspace,
    current: Option<TileId>,
    layer_visible: bool,
    events: Vec<WorkspaceEvent>,
}

fn tile(id: u64, title: &str) -> Tile {
    Tile {
        id: TileId(id),
        title: title.to_string(),
        content: ContentSource::Document(DocumentRef(format!("doc:{id}"))),
        accent: None,
    }
}

fn rect(x: f32, y: f32, w: f32, h: f32) -> RelativeRect {
    RelativeRect {
        x,
        y,
        width: w,
        height: h,
    }
}

/// A row split of two stacks (tiles 1+2 left, 3 right), with tiles 4 and 5
/// floated over it; 5 is pinned and on top.
fn sample() -> Workspace {
    let mut ws = Workspace::new(TileTree::Split {
        axis: SplitAxis::Row,
        children: vec![
            TileBranch {
                fraction: 0.5,
                tree: TileTree::Stack(TabStack {
                    tabs: vec![tile(1, "First"), tile(2, "Second")],
                    active: 0,
                }),
            },
            TileBranch {
                fraction: 0.5,
                tree: TileTree::Stack(TabStack {
                    tabs: vec![tile(3, "Third"), tile(4, "Fourth"), tile(5, "Fifth")],
                    active: 0,
                }),
            },
        ],
    });
    ws.apply(
        &FloatEvent::Float {
            tile: TileId(4),
            rect: rect(0.25, 0.5, 0.5, 0.25),
        }
        .into(),
    );
    ws.apply(
        &FloatEvent::Float {
            tile: TileId(5),
            rect: rect(0.1, 0.1, 0.3, 0.3),
        }
        .into(),
    );
    ws.apply(
        &FloatEvent::SetConstraints {
            tile: TileId(4),
            constraints: FloatSizeConstraints {
                min_width: 300.0,
                min_height: 120.0,
                max_width: None,
                max_height: Some(400.0),
            },
        }
        .into(),
    );
    ws.apply(
        &FloatEvent::SetPinned {
            tile: TileId(5),
            pinned: true,
        }
        .into(),
    );
    ws
}

/// Tile 2 gets a surface, tile 4 (a float) a hole, tile 5 (a float) a view.
fn fill_by_id(tile: &Tile) -> Slot<State, ()> {
    match tile.id.0 {
        2 | 3 => Slot::Surface,
        5 => Slot::View(Box::new(
            el::<_, State, ()>("p", "float child").attr("class", "float-child"),
        )),
        _ => Slot::Hole,
    }
}

fn view(state: &State) -> TestView {
    Box::new(workspace_view(
        &WorkspaceModel {
            workspace: &state.workspace,
            current: state.current,
            float_layer_visible: state.layer_visible,
        },
        |state: &mut State, event| state.events.push(event),
        fill_by_id,
    ))
}

fn harness(state: State) -> Harness {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let runner = GenetAppRunner::new(dom.clone(), view as fn(&State) -> TestView, state);
    (dom, runner)
}

fn sample_state() -> State {
    State {
        workspace: sample(),
        current: None,
        layer_visible: true,
        events: Vec::new(),
    }
}

fn attr_of<'a>(dom: &'a ScriptedDom, node: NodeId, name: &str) -> Option<&'a str> {
    dom.attribute(node, &Namespace::default(), &LocalName::from(name))
}

fn has_class(dom: &ScriptedDom, node: NodeId, class: &str) -> bool {
    attr_of(dom, node, "class")
        .is_some_and(|value| value.split_whitespace().any(|token| token == class))
}

fn all_with_class(dom: &ScriptedDom, root: NodeId, class: &str) -> Vec<NodeId> {
    let mut out = Vec::new();
    let mut walk = vec![root];
    while let Some(node) = walk.pop() {
        if has_class(dom, node, class) {
            out.push(node);
        }
        let children: Vec<_> = dom.dom_children(node).collect();
        walk.extend(children.into_iter().rev());
    }
    out
}

/// The floats in the order they were drawn (z order).
fn floats(dom: &ScriptedDom, root: NodeId) -> Vec<NodeId> {
    let layer = all_with_class(dom, root, "workspace-floats");
    assert_eq!(layer.len(), 1, "one float layer");
    dom.dom_children(layer[0]).collect()
}

fn float_for(dom: &ScriptedDom, root: NodeId, id: u64) -> NodeId {
    floats(dom, root)
        .into_iter()
        .find(|node| attr_of(dom, *node, FRISKET_TILE_ATTR) == Some(&id.to_string()))
        .unwrap_or_else(|| panic!("float for tile {id}"))
}

#[test]
fn the_frame_and_the_float_layer_both_render_with_the_floats_geometry() {
    let (dom, runner) = harness(sample_state());
    let root = runner.root();
    let dom = dom.borrow();

    assert!(has_class(&dom, root, "workspace"));
    assert_eq!(
        attr_of(&dom, root, "style"),
        Some("position: relative; width: 100%; height: 100%;")
    );
    // The frame is there, with its two stacks and their content elements.
    assert_eq!(all_with_class(&dom, root, "frisket-stack").len(), 2);
    // Two floats over it, in z order: 4 was floated first, 5 raised over it.
    let order: Vec<_> = floats(&dom, root)
        .iter()
        .map(|node| attr_of(&dom, *node, FRISKET_TILE_ATTR).unwrap().to_string())
        .collect();
    assert_eq!(order, ["4", "5"]);

    let four = float_for(&dom, root, 4);
    assert!(has_class(&dom, four, "workspace-float"));
    assert_eq!(attr_of(&dom, four, "role"), Some("group"));
    assert_eq!(attr_of(&dom, four, "aria-label"), Some("Fourth"));
    let style = attr_of(&dom, four, "style").expect("float style");
    // Proportion as percentages; constraints in px; z from the model.
    for fragment in [
        "position: absolute;",
        "left: 25%;",
        "top: 50%;",
        "width: 50%;",
        "height: 25%;",
        "min-width: 300px;",
        "min-height: 120px;",
        "max-height: 400px;",
        "z-index: 1;",
        "display: flex;",
        "flex-direction: column;",
    ] {
        assert!(
            style.contains(fragment),
            "{fragment:?} missing from {style:?}"
        );
    }
    assert!(
        !style.contains("max-width"),
        "an absent max emits nothing: {style:?}"
    );
    assert!(
        attr_of(&dom, float_for(&dom, root, 5), "style").is_some_and(|s| s.contains("z-index: 2;")),
        "the raised float stacks over the other"
    );

    // A float's own bar and content, wearing the frame's marks.
    assert_eq!(all_with_class(&dom, four, "frisket-tabbar").len(), 1);
    let content = all_with_class(&dom, four, "frisket-content");
    assert_eq!(content.len(), 1);
    assert_eq!(content_target(&*dom, content[0]), Some(TileId(4)));
    assert_eq!(slot_kind(&*dom, content[0]), Some(SlotKind::Hole));
    assert_eq!(dom.dom_children(content[0]).count(), 0, "a hole is empty");

    // Tile 5's float asked for a view, and got its child inline.
    let five_content = all_with_class(&dom, float_for(&dom, root, 5), "frisket-content");
    assert_eq!(slot_kind(&*dom, five_content[0]), Some(SlotKind::View));
    let child = dom.dom_children(five_content[0]).next().expect("child");
    assert!(has_class(&dom, child, "float-child"));
}

#[test]
fn a_hidden_float_layer_shows_only_the_pinned_floats() {
    let mut state = sample_state();
    state.layer_visible = false;
    let (dom, runner) = harness(state);
    let root = runner.root();
    let dom = dom.borrow();
    let ids: Vec<_> = floats(&dom, root)
        .iter()
        .map(|node| attr_of(&dom, *node, FRISKET_TILE_ATTR).unwrap().to_string())
        .collect();
    assert_eq!(
        ids,
        ["5"],
        "only the pinned float outlives the hidden layer"
    );
}

#[test]
fn composited_slots_lists_the_host_owned_content_of_both_stations() {
    let (dom, runner) = harness(sample_state());
    let root = runner.root();
    let dom = dom.borrow();

    let slots = composited_slots(&*dom, root);
    // Tile 1 (the left stack's active tab) is a hole, tile 3 a surface,
    // float 4 a hole. Float 5 is a view slot and is the frame's business.
    assert_eq!(
        slots
            .iter()
            .map(|(id, kind, _)| (id.0, *kind))
            .collect::<Vec<_>>(),
        [
            (1, SlotKind::Hole),
            (3, SlotKind::Surface),
            (4, SlotKind::Hole),
        ],
        "document order: the frame's slots, then the float layer's"
    );
    for (_, _, node) in &slots {
        assert!(has_class(&dom, *node, "frisket-content"));
    }
    assert!(
        !slots.iter().any(|(id, ..)| id.0 == 5),
        "a view slot is not the host's to composite"
    );
}

#[test]
fn an_empty_stack_does_not_alias_the_actual_floating_tile_zero() {
    let mut workspace = Workspace::new(TileTree::single(tile(0, "Real zero")));
    workspace.apply(
        &FloatEvent::Float {
            tile: TileId(0),
            rect: rect(0.1, 0.1, 0.5, 0.5),
        }
        .into(),
    );
    let (dom, runner) = harness(State {
        workspace,
        current: Some(TileId(0)),
        layer_visible: true,
        events: Vec::new(),
    });
    let dom = dom.borrow();
    let contents = all_with_class(&dom, runner.root(), "frisket-content");
    assert_eq!(contents.len(), 2, "empty stack and real float both render");
    assert_eq!(
        content_target(&*dom, contents[0]),
        None,
        "an empty stack has no owner"
    );
    assert_eq!(content_target(&*dom, contents[1]), Some(TileId(0)));
    let slots = composited_slots(&*dom, runner.root());
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0].0, TileId(0));
    assert_eq!(slots[0].2, contents[1]);
}

#[test]
fn the_current_tile_marks_its_stack_and_its_float() {
    // A tile in the tree: its stack's bar is current, the other is not.
    let mut state = sample_state();
    state.current = Some(TileId(2));
    let (dom, runner) = harness(state);
    let root = runner.root();
    {
        let dom = dom.borrow();
        let bars = all_with_class(&dom, root, "frisket-tabbar");
        let current: Vec<_> = bars
            .iter()
            .filter(|bar| attr_of(&dom, **bar, "aria-current") == Some("true"))
            .collect();
        assert_eq!(current.len(), 1, "exactly one current bar");
        assert_eq!(
            attr_of(&dom, *current[0], "data-stack"),
            Some("0"),
            "the stack that holds tile 2"
        );
    }

    // A floating tile: its float's bar is the current one instead.
    let mut state = sample_state();
    state.current = Some(TileId(4));
    let (dom, runner) = harness(state);
    let root = runner.root();
    let dom = dom.borrow();
    let four = float_for(&dom, root, 4);
    let bar = all_with_class(&dom, four, "frisket-tabbar")[0];
    assert_eq!(attr_of(&dom, bar, "aria-current"), Some("true"));
    assert!(has_class(&dom, bar, "current"));
    assert!(
        all_with_class(&dom, root, "frisket-tabbar")
            .iter()
            .filter(|b| attr_of(&dom, **b, "aria-current") == Some("true"))
            .count()
            == 1,
        "and no stack claims it"
    );
}

#[test]
fn a_floats_bar_reports_activation_and_close_in_the_tile_vocabulary() {
    let (dom, mut runner) = harness(sample_state());
    let root = runner.root();
    let before = runner.state().workspace.clone();

    let (tab, close) = {
        let dom = dom.borrow();
        let four = float_for(&dom, root, 4);
        let tab = all_with_class(&dom, four, "frisket-tab")[0];
        let close = all_with_class(&dom, four, "frisket-close")[0];
        (tab, close)
    };

    runner.dispatch_click(tab, PointerClick::at((2.0, 2.0)));
    assert_eq!(
        runner.state().events,
        [WorkspaceEvent::Tile(TileEvent::Activated(TileId(4)))]
    );

    runner.dispatch_click(close, PointerClick::at((2.0, 2.0)));
    assert_eq!(
        runner.state().events,
        [
            WorkspaceEvent::Tile(TileEvent::Activated(TileId(4))),
            WorkspaceEvent::Tile(TileEvent::Closed(TileId(4))),
        ],
        "the close reports one close and does not also activate"
    );
    assert_eq!(
        runner.state().workspace,
        before,
        "the view reports gestures; the caller owns the model"
    );
}

// ---- acceptance: what survives a move, and where ------------------------

/// The acceptance state: two workspaces (one per projection) over one
/// shared, `TileId`-keyed note store — the host's depth-1 pane state.
struct AcceptState {
    spaces: Vec<Workspace>,
    notes: HashMap<u64, String>,
}

type AcceptView = Box<dyn AnyView<AcceptState, (), GenetCtx, GenetElement>>;

/// A counter pane as a `Component`: its bump count is local view state, its
/// note comes in as a prop from the host's shared store.
#[derive(Clone, PartialEq)]
struct CounterProps {
    tile: u64,
    note: String,
}

struct CounterLocal {
    bumps: u32,
}

fn counter_body(props: &CounterProps, local: &CounterLocal) -> ComponentView<CounterLocal, ()> {
    Box::new(on_click(
        el::<_, CounterLocal, ()>("div", ())
            .attr("class", "counter")
            .attr("data-bumps", local.bumps.to_string())
            .attr("data-note", props.note.clone()),
        |local: &mut CounterLocal, _: PointerClick| local.bumps += 1,
    ))
}

fn counter(tile: u64, note: String) -> AcceptView {
    Box::new(
        component(
            CounterProps { tile, note },
            |_: &CounterProps| CounterLocal { bumps: 0 },
            |_: &CounterProps, _: &CounterProps, _: &mut CounterLocal| {},
            counter_body,
            |_: &mut AcceptState, _: ()| {},
        )
        .probe_id(format!("counter-{tile}")),
    )
}

fn accept_state() -> AcceptState {
    // Projection 0: two stacks so the drag has somewhere to land and
    // neither stack empties. Projection 1: a lone tile, the other window.
    let primary = Workspace::new(TileTree::Split {
        axis: SplitAxis::Row,
        children: vec![
            TileBranch {
                fraction: 0.5,
                tree: TileTree::Stack(TabStack {
                    tabs: vec![tile(1, "Counter"), tile(2, "Spare")],
                    active: 0,
                }),
            },
            TileBranch {
                fraction: 0.5,
                tree: TileTree::Stack(TabStack {
                    tabs: vec![tile(3, "Hole")],
                    active: 0,
                }),
            },
        ],
    });
    let secondary = Workspace::new(TileTree::single(tile(9, "Elsewhere")));
    AcceptState {
        spaces: vec![primary, secondary],
        notes: HashMap::from([(1u64, "kept by the host".to_string())]),
    }
}

/// One projection's view: its own workspace, filled from the shared state.
fn projection(index: usize) -> impl FnMut(&AcceptState) -> AcceptView {
    move |state: &AcceptState| {
        let notes = state.notes.clone();
        Box::new(workspace_view(
            &WorkspaceModel {
                workspace: &state.spaces[index],
                current: None,
                float_layer_visible: true,
            },
            |_: &mut AcceptState, _| {},
            move |tile: &Tile| match tile.id.0 {
                1 => Slot::View(counter(1, notes.get(&1).cloned().unwrap_or_default())),
                _ => Slot::Hole,
            },
        ))
    }
}

fn probe(dom: &ScriptedDom, root: NodeId, id: &str) -> Option<NodeId> {
    if attr_of(dom, root, COMPONENT_PROBE_ATTR) == Some(id) {
        return Some(root);
    }
    dom.dom_children(root)
        .find_map(|child| probe(dom, child, id))
}

#[path = "tests/lifetime.rs"]
mod lifetime;
