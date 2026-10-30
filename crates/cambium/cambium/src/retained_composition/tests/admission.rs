// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::cell::RefCell;
use std::rc::Rc;

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{DomMutation, LayoutDom, LayoutDomMut, LocalName, Namespace, NodeKind};

use super::*;
use crate::{
    ComponentView, CompositionEvent, Key, PointerEvent, PointerPhase, TextInput, ValueEvent,
    on_pointer, on_value, text_field_typed,
};

fn drain(dom: &DomHandle) -> Vec<DomMutation<NodeId>> {
    let mut mutations = Vec::new();
    dom.borrow_mut().drain_mutations(&mut mutations);
    mutations
}

fn node_with_attr(dom: &ScriptedDom, root: NodeId, name: &str, value: &str) -> Option<NodeId> {
    if dom.kind(root) == NodeKind::Element
        && dom.attribute(root, &Namespace::from(""), &LocalName::from(name)) == Some(value)
    {
        return Some(root);
    }
    dom.dom_children(root)
        .find_map(|child| node_with_attr(dom, child, name, value))
}

fn child_group(dom: &ScriptedDom, root: NodeId, key: &str) -> Option<NodeId> {
    if dom.kind(root) == NodeKind::Element
        && dom.attribute(
            root,
            &Namespace::from(""),
            &LocalName::from("data-composition-key"),
        ) == Some(key)
    {
        return Some(root);
    }
    dom.dom_children(root)
        .find_map(|child| child_group(dom, child, key))
}

struct ImeState {
    transaction: CompositionTransaction,
    factory: RetainedComponentFactory<ImeState, ()>,
    collapsed: BTreeSet<ContainerKey>,
    sink: LifecycleEventSink,
}

fn ime_factory() -> RetainedComponentFactory<ImeState, ()> {
    Rc::new(|descriptor| {
        let row_id = descriptor.key.0.clone();
        let prevent_commit = descriptor.kind == ComponentKind::from("prevent-commit");
        Box::new(
            component(
                row_id,
                |_: &String| TextInput::new(""),
                |_: &String, _: &String, _: &mut TextInput| {},
                move |id: &String, input: &TextInput| -> ComponentView<TextInput, ()> {
                    let field = text_field_typed(input).attr("data-row", id.clone());
                    if prevent_commit {
                        Box::new(
                            on_key(field, |_: &mut TextInput, event: crate::KeyEvent| {
                                if matches!(
                                    &event.key,
                                    Key::Composition(CompositionEvent::Commit(_))
                                ) {
                                    event.prevent_default();
                                    event.stop_immediate_propagation();
                                }
                            })
                            .capture(true),
                        )
                    } else {
                        Box::new(field)
                    }
                },
                |_: &mut ImeState, _: ()| {},
            )
            .memo(),
        )
    })
}

fn ime_state(source: Composition) -> ImeState {
    ImeState {
        transaction: transaction(source),
        factory: ime_factory(),
        collapsed: BTreeSet::new(),
        sink: LifecycleEventSink::default(),
    }
}

fn ime_logic(state: &ImeState) -> RetainedComposition<ImeState, ()> {
    RetainedComposition::new(&state.transaction, state.factory.clone())
        .with_collapsed(state.collapsed.clone())
        .with_lifecycle_sink(state.sink.clone())
}

struct DragState {
    transaction: CompositionTransaction,
    factory: RetainedComponentFactory<DragState, ()>,
    collapsed: BTreeSet<ContainerKey>,
    sink: LifecycleEventSink,
    phases: Vec<PointerPhase>,
}

fn drag_factory() -> RetainedComponentFactory<DragState, ()> {
    Rc::new(|descriptor| {
        let row_id = descriptor.key.0.clone();
        Box::new(on_pointer(
            el::<_, DragState, ()>("div", row_id.clone()).attr("data-row", row_id),
            |state: &mut DragState, event: PointerEvent| state.phases.push(event.phase),
        ))
    })
}

fn drag_state(source: Composition) -> DragState {
    DragState {
        transaction: transaction(source),
        factory: drag_factory(),
        collapsed: BTreeSet::new(),
        sink: LifecycleEventSink::default(),
        phases: Vec::new(),
    }
}

fn drag_logic(state: &DragState) -> RetainedComposition<DragState, ()> {
    RetainedComposition::new(&state.transaction, state.factory.clone())
        .with_collapsed(state.collapsed.clone())
        .with_lifecycle_sink(state.sink.clone())
}

fn move_a_right() -> Composition {
    model(
        vec![group("left", vec![]), group("right", vec![row("a")])],
        vec![descriptor("a", "row")],
    )
}

fn move_a_right_kind(kind: &str) -> Composition {
    model(
        vec![group("left", vec![]), group("right", vec![row("a")])],
        vec![descriptor("a", kind)],
    )
}

struct ValueState {
    transaction: CompositionTransaction,
    factory: RetainedComponentFactory<ValueState, ()>,
    collapsed: BTreeSet<ContainerKey>,
    sink: LifecycleEventSink,
    hits: usize,
}

fn value_factory() -> RetainedComponentFactory<ValueState, ()> {
    Rc::new(|descriptor| {
        let row_id = descriptor.key.0.clone();
        let button = crate::button("value row", |state: &mut ValueState, _| state.hits += 1)
            .attr("data-row", row_id);
        Box::new(on_value(
            on_key(button, |_: &mut ValueState, _: crate::KeyEvent| {}),
            |state: &mut ValueState, event: ValueEvent| state.hits += event.value as usize,
        ))
    })
}

fn value_state(source: Composition) -> ValueState {
    ValueState {
        transaction: transaction(source),
        factory: value_factory(),
        collapsed: BTreeSet::new(),
        sink: LifecycleEventSink::default(),
        hits: 0,
    }
}

fn value_logic(state: &ValueState) -> RetainedComposition<ValueState, ()> {
    RetainedComposition::new(&state.transaction, state.factory.clone())
        .with_collapsed(state.collapsed.clone())
        .with_lifecycle_sink(state.sink.clone())
}

#[test]
fn active_preedit_rejects_move_without_dom_mutation_then_prepares_through_old_route() {
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let source = model(
        vec![group("left", vec![row("a")]), group("right", vec![])],
        vec![descriptor("a", "row")],
    );
    let mut runner = GenetAppRunner::new(dom.clone(), ime_logic, ime_state(source));
    let root = runner.root();
    let node = node_with_attr(&dom.borrow(), root, "data-row", "a").unwrap();
    runner.set_focus(Some(node));
    runner.dispatch_key(crate::KeyEvent::new(Key::Composition(
        CompositionEvent::Preedit {
            text: "漢".into(),
            selection: None,
        },
    )));
    assert_eq!(
        dom.borrow().attribute(
            node,
            &Namespace::from(""),
            &LocalName::from("data-cambium-text-value")
        ),
        Some("")
    );
    assert!(drain(&dom).len() > 0, "preedit is visible DOM content");
    let old_route = runner.state().sink.route(&ComponentKey::from("a")).unwrap();

    runner.update(|state| {
        state
            .transaction
            .apply(move_a_right(), &AllowBindings)
            .unwrap();
    });

    assert_eq!(
        runner.state().sink.last_refusal(),
        Some(LifecycleRefusal::ActiveInteraction)
    );
    assert_eq!(
        runner.state().sink.route(&ComponentKey::from("a")),
        Some(old_route.clone())
    );
    assert!(
        drain(&dom).is_empty(),
        "preflight refusal precedes DOM mutation"
    );
    assert_eq!(
        dom.borrow().parent(node),
        child_group(&dom.borrow(), root, "left")
    );

    let actions = runner.prepare_subtree_move(node);
    assert!(actions.is_empty());
    assert_eq!(
        dom.borrow().attribute(
            node,
            &Namespace::from(""),
            &LocalName::from("data-cambium-text-value")
        ),
        Some("漢"),
        "the retained TextInput committed before the move was applied"
    );
    assert_eq!(runner.state().sink.last_refusal(), None);
    assert_eq!(
        dom.borrow().parent(node),
        child_group(&dom.borrow(), root, "right")
    );
    let new_route = runner.state().sink.route(&ComponentKey::from("a")).unwrap();
    assert_eq!(new_route.route.slot, old_route.route.slot);
    assert_eq!(
        new_route.route.route_generation,
        old_route.route.route_generation
    );
    assert!(new_route.route.placement_generation > old_route.route.placement_generation);
    assert_eq!(
        node_with_attr(&dom.borrow(), root, "data-row", "a"),
        Some(node)
    );
}

#[test]
fn prevented_synthetic_commit_keeps_move_refused_without_transient_dom_change() {
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let source = model(
        vec![group("left", vec![row("a")]), group("right", vec![])],
        vec![descriptor("a", "prevent-commit")],
    );
    let mut runner = GenetAppRunner::new(dom.clone(), ime_logic, ime_state(source));
    let root = runner.root();
    let node = node_with_attr(&dom.borrow(), root, "data-row", "a").unwrap();
    runner.set_focus(Some(node));
    runner.dispatch_key(crate::KeyEvent::new(Key::Composition(
        CompositionEvent::Preedit {
            text: "漢".into(),
            selection: None,
        },
    )));
    drain(&dom);
    let old_parent = dom.borrow().parent(node);
    let old_route = runner.state().sink.route(&ComponentKey::from("a")).unwrap();

    runner.update(|state| {
        state
            .transaction
            .apply(move_a_right_kind("prevent-commit"), &AllowBindings)
            .unwrap();
    });
    assert_eq!(
        runner.state().sink.last_refusal(),
        Some(LifecycleRefusal::ActiveInteraction)
    );
    assert!(drain(&dom).is_empty());

    runner.prepare_subtree_move(node);

    assert_eq!(
        runner.state().sink.last_refusal(),
        Some(LifecycleRefusal::ActiveInteraction)
    );
    assert_eq!(
        runner.state().sink.route(&ComponentKey::from("a")).unwrap(),
        old_route
    );
    assert_eq!(dom.borrow().parent(node), old_parent);
    assert_eq!(
        node_with_attr(&dom.borrow(), root, "data-row", "a"),
        Some(node)
    );
    assert!(
        drain(&dom).is_empty(),
        "prevented commit cannot transiently move the row"
    );
}

#[test]
fn prevented_native_commit_still_ends_tracked_preedit() {
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let source = model(
        vec![group("left", vec![row("a")]), group("right", vec![])],
        vec![descriptor("a", "prevent-commit")],
    );
    let mut runner = GenetAppRunner::new(dom.clone(), ime_logic, ime_state(source));
    let root = runner.root();
    let node = node_with_attr(&dom.borrow(), root, "data-row", "a").unwrap();
    runner.set_focus(Some(node));
    runner.dispatch_key(crate::KeyEvent::new(Key::Composition(
        CompositionEvent::Preedit {
            text: "漢".into(),
            selection: None,
        },
    )));
    drain(&dom);

    // This is a native Commit: the custom handler prevents default but has
    // consumed the event. It must not be treated like a refused synthetic
    // move-preparation commit.
    runner.dispatch_key(crate::KeyEvent::new(Key::Composition(
        CompositionEvent::Commit("漢".into()),
    )));
    drain(&dom);
    runner.update(|state| {
        state
            .transaction
            .apply(move_a_right_kind("prevent-commit"), &AllowBindings)
            .unwrap();
    });

    assert_eq!(runner.state().sink.last_refusal(), None);
    assert_eq!(
        dom.borrow().parent(node),
        child_group(&dom.borrow(), root, "right")
    );
}

#[test]
fn active_pointer_capture_refuses_move_then_sends_cancel_before_reconcile() {
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let source = model(
        vec![group("left", vec![row("a")]), group("right", vec![])],
        vec![descriptor("a", "row")],
    );
    let mut runner = GenetAppRunner::new(dom.clone(), drag_logic, drag_state(source));
    let root = runner.root();
    let node = node_with_attr(&dom.borrow(), root, "data-row", "a").unwrap();
    let old_route = runner.state().sink.route(&ComponentKey::from("a")).unwrap();
    runner.dispatch_pointer_down(
        node,
        PointerEvent::new(PointerPhase::Down, (1.0, 1.0), (10.0, 10.0)),
    );
    assert_eq!(runner.pointer_capture(), Some(node));
    drain(&dom);

    runner.update(|state| {
        state
            .transaction
            .apply(move_a_right(), &AllowBindings)
            .unwrap();
    });

    assert_eq!(
        runner.state().sink.last_refusal(),
        Some(LifecycleRefusal::ActiveInteraction)
    );
    assert_eq!(
        runner.state().sink.route(&ComponentKey::from("a")).unwrap(),
        old_route
    );
    assert!(
        drain(&dom).is_empty(),
        "capture refusal leaves presentation untouched"
    );
    assert_eq!(
        dom.borrow().parent(node),
        child_group(&dom.borrow(), root, "left")
    );

    runner.prepare_subtree_move(node);

    assert_eq!(
        runner.state().phases,
        [PointerPhase::Down, PointerPhase::Cancel]
    );
    assert_eq!(runner.pointer_capture(), None);
    assert_eq!(
        dom.borrow().parent(node),
        child_group(&dom.borrow(), root, "right")
    );
    assert_eq!(
        node_with_attr(&dom.borrow(), root, "data-row", "a"),
        Some(node)
    );
    assert!(runner.state().sink.last_refusal().is_none());
}

#[test]
fn same_parent_reorder_updates_visible_tab_order_and_parked_row_rejects_input() {
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let initial = model(
        vec![group("g", vec![row("a"), row("b")])],
        vec![descriptor("a", "keyed"), descriptor("b", "keyed")],
    );
    let mut runner = GenetAppRunner::new(dom.clone(), value_logic, value_state(initial));
    let root = runner.root();
    let a = find_row(&dom.borrow(), root, "a").unwrap();
    let b = find_row(&dom.borrow(), root, "b").unwrap();
    assert_eq!(runner.focusables(), [a, b]);
    runner.dispatch_value(a, ValueEvent { value: 3.0 });
    assert_eq!(
        runner.state().hits,
        3,
        "visible accessibility input reaches the row"
    );

    runner.update(|state| {
        state
            .transaction
            .apply(
                model(
                    vec![group("g", vec![row("b"), row("a")])],
                    vec![descriptor("a", "keyed"), descriptor("b", "keyed")],
                ),
                &AllowBindings,
            )
            .unwrap();
    });
    assert_eq!(runner.focusables(), [b, a]);

    let collapsed = ContainerKey {
        kind: ContainerKind::Group,
        key: "g".into(),
    };
    runner.update(|state| {
        state.collapsed.insert(collapsed);
    });
    assert!(runner.focusables().is_empty());
    let before = runner.state().hits;
    runner.dispatch_click(a, PointerClick::at((1.0, 1.0)));
    runner.dispatch_value(a, ValueEvent { value: 99.0 });
    assert_eq!(runner.state().hits, before);
    assert_eq!(
        node_with_attr(&dom.borrow(), root, "data-row", "a"),
        Some(a)
    );
}
