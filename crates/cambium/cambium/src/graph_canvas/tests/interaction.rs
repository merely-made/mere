/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

#[test]
fn node_drag_captures_and_reports_normalized_graph_positions() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(dom.clone(), drag_view, State::default());
    let root = runner.root();
    let first =
        find_attr(&dom.borrow(), root, "aria-label", "First node").expect("first node target");

    runner.dispatch_pointer_down(
        first,
        PointerEvent::new(PointerPhase::Down, (10.0, 10.0), (20.0, 20.0)),
    );
    assert_eq!(runner.pointer_capture(), Some(first));
    runner.dispatch_pointer_move(PointerEvent::new(
        PointerPhase::Move,
        (120.0, 10.0),
        (20.0, 20.0),
    ));
    runner.dispatch_pointer_up(PointerEvent::new(
        PointerPhase::Up,
        (120.0, 10.0),
        (20.0, 20.0),
    ));

    let dragged = &runner.state().dragged;
    assert_eq!(
        dragged.iter().map(|event| event.phase).collect::<Vec<_>>(),
        vec![PointerPhase::Down, PointerPhase::Move, PointerPhase::Up]
    );
    assert!(
        dragged[1].position.0 > dragged[0].position.0,
        "a rightward drag advances the normalized x position: {dragged:?}"
    );
    assert!(
        dragged.iter().all(|event| event.id == 1
            && (0.0..=1.0).contains(&event.position.0)
            && (0.0..=1.0).contains(&event.position.1)),
        "every emitted event keeps the node id and clamps graph coordinates: {dragged:?}"
    );
    assert_eq!(runner.pointer_capture(), None, "Up releases capture");
}

#[test]
fn component_owns_emphasis_and_reports_only_events() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        component_view,
        AppState {
            selected: Some(1),
            events: Vec::new(),
        },
    );
    let root = runner.root();
    let second = find_attr(&dom.borrow(), root, "aria-label", "Second node").expect("second node");

    // Hover renders emphasis without the application storing anything.
    runner.dispatch_hover(
        second,
        HoverEvent::new(HoverPhase::Enter, (2.0, 2.0), (20.0, 20.0)),
    );
    assert!(
        runner.state().events.is_empty(),
        "emphasis is not an application event"
    );
    assert_eq!(
        attr(&dom.borrow(), second, "class"),
        Some("graph-canvas-swatch-node hovered"),
        "the component rendered its own hover emphasis"
    );

    runner.dispatch_hover(
        second,
        HoverEvent::new(HoverPhase::Leave, (2.0, 2.0), (20.0, 20.0)),
    );
    assert_eq!(
        attr(&dom.borrow(), second, "class"),
        Some("graph-canvas-swatch-node"),
        "leaving clears the component's emphasis"
    );

    // Activation is an event, and the selection it drives is parent truth
    // that comes back through props.
    runner.dispatch_click(second, PointerClick::at((2.0, 2.0)));
    assert_eq!(runner.state().events, [GraphCanvasEvent::Activate(2)]);
    assert_eq!(runner.state().selected, Some(2));
    assert_eq!(
        attr(&dom.borrow(), second, "class"),
        Some("graph-canvas-swatch-node selected focused"),
        "parent-owned selection renders beside the component's own focus emphasis"
    );

    let expand =
        find_attr(&dom.borrow(), root, "aria-label", "Expand graph").expect("expand target");
    runner.dispatch_click(expand, PointerClick::at((2.0, 2.0)));
    assert_eq!(
        runner.state().events,
        [GraphCanvasEvent::Activate(2), GraphCanvasEvent::Expand]
    );
}

#[test]
fn atlas_pointer_uses_polygon_not_semantic_button_bounds_and_captures_drag() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner =
        GenetAppRunner::<_, _, _, ()>::new(dom.clone(), atlas_view, AtlasState::default());
    let root = find_attr(
        &dom.borrow(),
        runner.root(),
        "class",
        "graph-canvas-swatch graph-canvas-atlas-swatch",
    )
    .expect("atlas root");
    let size = (200.0, 100.0);
    // (115, 30) lies in the triangle's bounding rectangle but outside its
    // actual edge, proving the full-surface handler does not accept it.
    runner.dispatch_pointer_down(
        root,
        PointerEvent::new(PointerPhase::Down, (115.0, 30.0), size),
    );
    runner.dispatch_pointer_up(PointerEvent::new(PointerPhase::Up, (115.0, 30.0), size));
    assert!(runner.state().events.is_empty());

    runner.dispatch_pointer_down(
        root,
        PointerEvent::new(PointerPhase::Down, (100.0, 50.0), size),
    );
    runner.dispatch_pointer_move(PointerEvent::new(PointerPhase::Move, (180.0, 50.0), size));
    runner.dispatch_pointer_up(PointerEvent::new(PointerPhase::Up, (180.0, 50.0), size));
    let drags: Vec<_> = runner
        .state()
        .events
        .iter()
        .filter_map(|event| match event {
            GraphAtlasEvent::Drag(drag) => Some(drag),
            _ => None,
        })
        .collect();
    assert_eq!(drags.len(), 3, "down/move/up retain the geographic field");
    assert!(drags.iter().all(|drag| drag.id == 1));
    assert!(
        drags[1].position.0 > 0.5,
        "inverse atlas view remains normalized"
    );
}

#[test]
fn deferred_atlas_hover_and_move_do_not_rebuild_the_logic_view() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let builds = Rc::new(Cell::new(0));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        atlas_view,
        AtlasState {
            events: Vec::new(),
            deferred: true,
            builds: builds.clone(),
        },
    );
    let root = find_attr(
        &dom.borrow(),
        runner.root(),
        "class",
        "graph-canvas-swatch graph-canvas-atlas-swatch",
    )
    .expect("atlas root");
    let size = (200.0, 100.0);
    assert_eq!(builds.get(), 1);
    for _ in 0..3 {
        runner.dispatch_hover(root, HoverEvent::new(HoverPhase::Move, (100.0, 50.0), size));
    }
    runner.dispatch_pointer_down(
        root,
        PointerEvent::new(PointerPhase::Down, (100.0, 50.0), size),
    );
    runner.dispatch_pointer_move(PointerEvent::new(PointerPhase::Move, (120.0, 50.0), size));
    runner.dispatch_pointer_move(PointerEvent::new(PointerPhase::Move, (140.0, 50.0), size));
    assert_eq!(
        builds.get(),
        2,
        "Down rebuilds once; deferred hover and move do not"
    );
    runner.dispatch_pointer_up(PointerEvent::new(PointerPhase::Up, (140.0, 50.0), size));
    assert_eq!(
        builds.get(),
        3,
        "release reconciles the retained surface once"
    );
}

#[test]
fn atlas_field_focus_and_enter_emit_focus_then_activation() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner =
        GenetAppRunner::<_, _, _, ()>::new(dom.clone(), atlas_view, AtlasState::default());
    let field = find_attr(&dom.borrow(), runner.root(), "aria-label", "Tower field")
        .expect("semantic geographic field button");
    runner.set_focus(Some(field));
    runner.dispatch_key(crate::KeyEvent::new(crate::Key::Named(
        crate::NamedKey::Enter,
    )));
    assert_eq!(
        runner.state().events,
        [
            GraphAtlasEvent::Focus(Some(1)),
            GraphAtlasEvent::Activate(1)
        ]
    );
}
