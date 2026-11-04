/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{cell::RefCell, rc::Rc};

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LocalName, Namespace};

use super::*;
use crate::{AnyView, DomHandle, GenetAppRunner, KeyEvent, Modifiers, lens, map_action};

#[derive(Default)]
struct App {
    order: Vec<&'static str>,
    reorder: ReorderState<&'static str>,
    moves: Vec<ReorderMove<&'static str>>,
}

type AppView = Box<dyn AnyView<App, (), GenetCtx, GenetElement>>;

fn items(order: &[&'static str]) -> Vec<ReorderItem<&'static str>> {
    order
        .iter()
        .map(|id| {
            let label = match *id {
                "a" => "Alpha",
                "b" => "Bravo",
                _ => "Charlie",
            };
            ReorderItem::new(*id, label)
        })
        .collect()
}

fn app_view(state: &App) -> AppView {
    let items = items(&state.order);
    Box::new(map_action(
        lens(
            move |reorder: &mut ReorderState<&'static str>| reorderable_list(reorder, &items),
            |app: &mut App| &mut app.reorder,
        ),
        |app: &mut App, movement: ReorderMove<&'static str>| {
            let from = app
                .order
                .iter()
                .position(|id| *id == movement.id)
                .expect("move identity remains in application order");
            let item = app.order.remove(from);
            let to = movement.to.min(app.order.len());
            app.order.insert(to, item);
            app.moves.push(movement);
        },
    ))
}

fn runner() -> GenetAppRunner<App, fn(&App) -> AppView, AppView> {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    GenetAppRunner::new(
        dom,
        app_view,
        App {
            order: vec!["a", "b", "c"],
            reorder: ReorderState::new()
                .with_id("test-reorder")
                .with_label("Test order"),
            moves: Vec::new(),
        },
    )
}

fn attr<'a>(dom: &'a ScriptedDom, node: NodeId, name: &str) -> Option<&'a str> {
    dom.attribute(node, &Namespace::from(""), &LocalName::from(name))
}

fn find_attr(dom: &ScriptedDom, node: NodeId, name: &str, value: &str) -> Option<NodeId> {
    if attr(dom, node, name) == Some(value) {
        return Some(node);
    }
    dom.dom_children(node)
        .find_map(|child| find_attr(dom, child, name, value))
}

fn pointer(phase: PointerPhase, y: f32) -> PointerEvent {
    PointerEvent::new(phase, (8.0, y), (180.0, 20.0))
}

#[test]
fn keyboard_cancel_and_drop_preserve_focus_and_emit_one_move() {
    let mut runner = runner();
    let first = find_attr(
        &runner.dom().borrow(),
        runner.root(),
        "id",
        "test-reorder-item-a",
    )
    .expect("first row");
    runner.set_focus(Some(first));

    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Space)));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::End)));
    assert_eq!(runner.state().reorder.destination(), Some(2));
    assert!(
        find_attr(
            &runner.dom().borrow(),
            runner.root(),
            "data-position",
            "after"
        )
        .is_some()
    );
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Escape)));
    assert_eq!(runner.state().order, ["a", "b", "c"]);
    assert_eq!(runner.state().reorder.destination(), None);

    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Space)));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::End)));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Space)));
    assert_eq!(runner.state().order, ["b", "c", "a"]);
    assert_eq!(
        runner.state().moves,
        [ReorderMove {
            id: "a",
            from: 0,
            to: 2,
        }]
    );
    assert_eq!(runner.focus(), Some(first), "the keyed row keeps focus");
}

#[test]
fn pointer_and_keyboard_emit_the_same_move() {
    let mut pointer_runner = runner();
    let first = find_attr(
        &pointer_runner.dom().borrow(),
        pointer_runner.root(),
        "id",
        "test-reorder-item-a",
    )
    .expect("first row");
    pointer_runner.dispatch_pointer_down(first, pointer(PointerPhase::Down, 10.0));
    assert_eq!(pointer_runner.pointer_capture(), Some(first));
    pointer_runner.dispatch_pointer_move(pointer(PointerPhase::Move, 50.0));
    pointer_runner.dispatch_pointer_up(pointer(PointerPhase::Up, 50.0));
    assert_eq!(pointer_runner.pointer_capture(), None);

    let mut keyboard_runner = runner();
    let first = find_attr(
        &keyboard_runner.dom().borrow(),
        keyboard_runner.root(),
        "id",
        "test-reorder-item-a",
    )
    .expect("first row");
    keyboard_runner.set_focus(Some(first));
    keyboard_runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Space)));
    keyboard_runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::End)));
    keyboard_runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Space)));

    assert_eq!(pointer_runner.state().moves, keyboard_runner.state().moves);
    assert_eq!(pointer_runner.state().order, keyboard_runner.state().order);
}

#[test]
fn alt_arrow_moves_directly_without_entering_drag_mode() {
    let mut runner = runner();
    let first = find_attr(
        &runner.dom().borrow(),
        runner.root(),
        "id",
        "test-reorder-item-a",
    )
    .expect("first row");
    runner.set_focus(Some(first));
    runner.dispatch_key(KeyEvent::with_mods(
        Key::Named(NamedKey::ArrowDown),
        Modifiers {
            alt: true,
            ..Modifiers::default()
        },
    ));

    assert_eq!(runner.state().order, ["b", "a", "c"]);
    assert_eq!(runner.state().reorder.dragged(), None);
    assert_eq!(
        runner.state().moves,
        [ReorderMove {
            id: "a",
            from: 0,
            to: 1,
        }]
    );
    assert_eq!(runner.focus(), Some(first));
}

#[test]
fn removing_the_captured_item_clears_drag_capture_and_indicator() {
    let mut runner = runner();
    let first = find_attr(
        &runner.dom().borrow(),
        runner.root(),
        "id",
        "test-reorder-item-a",
    )
    .expect("first row");
    runner.dispatch_pointer_down(first, pointer(PointerPhase::Down, 10.0));
    runner.dispatch_pointer_move(pointer(PointerPhase::Move, 30.0));
    assert!(runner.state().reorder.dragged().is_some());
    assert!(runner.pointer_capture().is_some());

    runner.update(|app| {
        app.order.remove(0);
    });

    assert_eq!(runner.pointer_capture(), None);
    assert_eq!(runner.state().reorder.dragged(), None);
    assert!(
        find_attr(
            &runner.dom().borrow(),
            runner.root(),
            "class",
            "reorder-drop-indicator"
        )
        .is_none()
    );
}
