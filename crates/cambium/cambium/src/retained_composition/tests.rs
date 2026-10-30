// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::cell::Cell;
use std::collections::BTreeSet;
use std::rc::Rc;

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::NodeKind;
use layout_dom_api::{LayoutDom, LayoutDomMut, LocalName, Namespace};
use meristem::{DynMessage, MessageCtx, MessageResult, ViewId};

use super::*;
use crate::composition::{
    BindingAdmission, ComponentKind, CompositionLimits, CompositionValidator, DrawerKey, GroupKey,
    PlacementNode,
};
use crate::{
    DomHandle, GenetAppRunner, GenetCtx, KeyEvent, PointerClick, View, component, el, on_click,
    on_key,
};

struct AllowBindings;

impl CompositionValidator for AllowBindings {
    fn validate_component(
        &self,
        _candidate: &ComponentDescriptor,
        _previous: Option<&ComponentDescriptor>,
    ) -> Result<BindingAdmission, String> {
        Ok(BindingAdmission::Available)
    }
}

fn descriptor(key: &str, kind: &str) -> ComponentDescriptor {
    ComponentDescriptor {
        key: ComponentKey::from(key),
        kind: ComponentKind::from(kind),
        config: serde_json::Value::Null,
        binding: serde_json::Value::Null,
    }
}

fn transaction(model: Composition) -> CompositionTransaction {
    CompositionTransaction::new(model, CompositionLimits::default(), &AllowBindings).unwrap()
}

fn row(key: &str) -> PlacementNode {
    PlacementNode::Component {
        key: ComponentKey::from(key),
    }
}

fn group(key: &str, children: Vec<PlacementNode>) -> PlacementNode {
    PlacementNode::Group {
        key: GroupKey::from(key),
        children,
    }
}

fn drawer(key: &str, children: Vec<PlacementNode>) -> PlacementNode {
    PlacementNode::Drawer {
        key: DrawerKey::from(key),
        children,
    }
}

fn model(placement: Vec<PlacementNode>, descriptors: Vec<ComponentDescriptor>) -> Composition {
    Composition {
        components: descriptors,
        placement,
        unplaced: Vec::new(),
    }
}

#[derive(Clone, Copy)]
enum RowEvent {
    Clicked,
}
impl crate::Action for RowEvent {}

struct TestState {
    clicks: usize,
    transaction: CompositionTransaction,
    factory: RetainedComponentFactory<TestState, ()>,
    collapsed: BTreeSet<ContainerKey>,
    sink: LifecycleEventSink,
    factory_builds: Rc<Cell<usize>>,
}

fn factory(builds: Rc<Cell<usize>>) -> RetainedComponentFactory<TestState, ()> {
    Rc::new(move |descriptor| {
        builds.set(builds.get() + 1);
        let id = descriptor.key.0.clone();
        if descriptor.kind == ComponentKind::from("keyed") {
            let button = el::<_, TestState, ()>("button", id.clone()).attr("data-row", id);
            Box::new(on_key(
                on_click(button, |state: &mut TestState, _| state.clicks += 1),
                |_: &mut TestState, _: KeyEvent| {},
            ))
        } else {
            Box::new(
                component(
                    id,
                    |_| 0usize,
                    |_, _, _| {},
                    |id: &String, local: &usize| {
                        Box::new(on_click(
                            el::<_, usize, RowEvent>("button", format!("{id}:{local}"))
                                .attr("data-row", id.clone()),
                            |local: &mut usize, _| {
                                *local += 1;
                                RowEvent::Clicked
                            },
                        ))
                    },
                    |state: &mut TestState, RowEvent::Clicked| state.clicks += 1,
                )
                .memo(),
            )
        }
    })
}

fn state(model: Composition) -> TestState {
    let factory_builds = Rc::new(Cell::new(0));
    TestState {
        clicks: 0,
        transaction: transaction(model),
        factory: factory(factory_builds.clone()),
        collapsed: BTreeSet::new(),
        sink: LifecycleEventSink::default(),
        factory_builds,
    }
}

fn logic(state: &TestState) -> RetainedComposition<TestState, ()> {
    RetainedComposition::new(&state.transaction, state.factory.clone())
        .with_collapsed(state.collapsed.clone())
        .with_lifecycle_sink(state.sink.clone())
}

fn find_row(dom: &ScriptedDom, root: NodeId, id: &str) -> Option<NodeId> {
    fn walk(dom: &ScriptedDom, node: NodeId, id: &str) -> Option<NodeId> {
        if dom.kind(node) == NodeKind::Element
            && dom
                .element_name(node)
                .is_some_and(|name| AsRef::<str>::as_ref(&name.local) == "button")
            && dom.attribute(node, &Namespace::from(""), &LocalName::from("data-row")) == Some(id)
        {
            return Some(node);
        }
        dom.dom_children(node)
            .find_map(|child| walk(dom, child, id))
    }
    walk(dom, root, id)
}

fn text_child(dom: &ScriptedDom, node: NodeId) -> Option<String> {
    dom.dom_children(node)
        .find(|child| dom.kind(*child) == NodeKind::Text)
        .and_then(|child| dom.text(child).map(str::to_string))
}

fn two_groups(reverse: bool) -> Composition {
    let mut groups = vec![
        group("left", vec![row("a")]),
        group("right", vec![row("b")]),
    ];
    if reverse {
        groups.reverse();
    }
    model(groups, vec![descriptor("a", "row"), descriptor("b", "row")])
}

#[test]
fn move_rebuilds_wrapper_in_either_traversal_order_without_replacing_row() {
    for reverse in [false, true] {
        let dom = Rc::new(std::cell::RefCell::new(ScriptedDom::new()));
        let initial = two_groups(reverse);
        let mut runner = GenetAppRunner::new(dom.clone(), logic, state(initial.clone()));
        let root = runner.root();
        let original = find_row(&dom.borrow(), root, "a").unwrap();
        let before = runner.state().sink.route(&ComponentKey::from("a")).unwrap();
        runner.dispatch_click(original, PointerClick::at((1.0, 1.0)));
        assert_eq!(runner.state().clicks, 1);
        assert_eq!(text_child(&dom.borrow(), original).as_deref(), Some("a:1"));

        let mut moved = initial;
        moved.placement = if reverse {
            vec![
                group("right", vec![row("a"), row("b")]),
                group("left", vec![]),
            ]
        } else {
            vec![
                group("left", vec![]),
                group("right", vec![row("a"), row("b")]),
            ]
        };
        runner.update(|state| {
            state.transaction.apply(moved, &AllowBindings).unwrap();
        });

        let after_node = find_row(&dom.borrow(), root, "a").unwrap();
        let after = runner.state().sink.route(&ComponentKey::from("a")).unwrap();
        assert_eq!(
            after_node, original,
            "move must preserve actual DOM identity"
        );
        assert_eq!(after.route.slot, before.route.slot);
        assert_eq!(after.route.route_generation, before.route.route_generation);
        assert!(after.route.placement_generation > before.route.placement_generation);
        assert_eq!(
            text_child(&dom.borrow(), after_node).as_deref(),
            Some("a:1")
        );
        runner.dispatch_click(after_node, PointerClick::at((1.0, 1.0)));
        assert_eq!(runner.state().clicks, 2);
    }
}

#[test]
fn hidden_row_is_parked_through_collection_and_reveals_with_same_node() {
    let dom = Rc::new(std::cell::RefCell::new(ScriptedDom::new()));
    let source = model(
        vec![group("settings", vec![row("a")])],
        vec![descriptor("a", "row")],
    );
    let mut runner = GenetAppRunner::new(dom.clone(), logic, state(source));
    let root = runner.root();
    let node = find_row(&dom.borrow(), root, "a").unwrap();
    let key = ContainerKey {
        kind: ContainerKind::Group,
        key: "settings".into(),
    };
    runner.update(|state| {
        state.collapsed.insert(key.clone());
    });
    assert!(
        !runner
            .state()
            .sink
            .route(&ComponentKey::from("a"))
            .unwrap()
            .visible
    );
    assert_eq!(
        dom.borrow().parent(node),
        Some(
            dom.borrow()
                .dom_children(root)
                .find(|child| {
                    dom.borrow().attribute(
                        *child,
                        &Namespace::from(""),
                        &LocalName::from("data-cambium-parked"),
                    ) == Some("true")
                })
                .unwrap()
        )
    );
    dom.borrow_mut().collect([]);
    assert!(dom.borrow().is_live(node));
    runner.dispatch_click(node, PointerClick::at((1.0, 1.0)));
    assert_eq!(runner.state().clicks, 0, "parked route must reject input");

    runner.update(|state| {
        state.collapsed.clear();
    });
    assert_eq!(find_row(&dom.borrow(), root, "a"), Some(node));
    assert!(
        runner
            .state()
            .sink
            .route(&ComponentKey::from("a"))
            .unwrap()
            .visible
    );
    runner.dispatch_click(node, PointerClick::at((1.0, 1.0)));
    assert_eq!(runner.state().clicks, 1);
}

#[test]
fn group_to_drawer_replacement_preserves_child_and_disposes_removed_entries() {
    let dom = Rc::new(std::cell::RefCell::new(ScriptedDom::new()));
    let source = model(
        vec![group("settings", vec![row("a"), row("b")])],
        vec![descriptor("a", "row"), descriptor("b", "row")],
    );
    let mut runner = GenetAppRunner::new(dom.clone(), logic, state(source.clone()));
    let root = runner.root();
    let node_a = find_row(&dom.borrow(), root, "a").unwrap();
    let before = runner.state().sink.route(&ComponentKey::from("a")).unwrap();

    let next = model(
        vec![drawer("settings", vec![row("a")])],
        vec![descriptor("a", "row")],
    );
    let old_node_b = find_row(&dom.borrow(), root, "b").unwrap();
    runner.update(|state| {
        state.transaction.apply(next, &AllowBindings).unwrap();
    });
    assert_eq!(find_row(&dom.borrow(), root, "a"), Some(node_a));
    let after = runner.state().sink.route(&ComponentKey::from("a")).unwrap();
    assert_eq!(before.route.route_generation, after.route.route_generation);
    assert!(after.route.placement_generation > before.route.placement_generation);
    assert!(
        runner
            .state()
            .sink
            .route(&ComponentKey::from("b"))
            .is_none()
    );
    assert!(find_row(&dom.borrow(), root, "b").is_none());
    assert!(!dom.borrow().is_live(old_node_b));
}

#[test]
fn binding_or_kind_replacement_gets_new_route_incarnation() {
    let dom = Rc::new(std::cell::RefCell::new(ScriptedDom::new()));
    let source = model(vec![row("a")], vec![descriptor("a", "row")]);
    let mut runner = GenetAppRunner::new(dom.clone(), logic, state(source.clone()));
    let root = runner.root();
    let old_node = find_row(&dom.borrow(), root, "a").unwrap();
    let old_route = runner.state().sink.route(&ComponentKey::from("a")).unwrap();
    let next = model(vec![row("a")], vec![descriptor("a", "keyed")]);
    runner.update(|state| {
        state.transaction.apply(next, &AllowBindings).unwrap();
    });
    let new_node = find_row(&dom.borrow(), root, "a").unwrap();
    let new_route = runner.state().sink.route(&ComponentKey::from("a")).unwrap();
    assert_ne!(old_node, new_node);
    assert!(new_route.route.route_generation > old_route.route.route_generation);
}

#[test]
fn delayed_message_from_old_placement_route_is_rejected() {
    let first = model(
        vec![group("left", vec![row("a")]), group("right", vec![])],
        vec![descriptor("a", "row")],
    );
    let mut app = state(first);
    let sink = app.sink.clone();
    let previous = RetainedComposition::<TestState, ()>::new(&app.transaction, app.factory.clone())
        .with_lifecycle_sink(sink.clone());
    let dom: DomHandle = Rc::new(std::cell::RefCell::new(ScriptedDom::new()));
    let mut ctx = GenetCtx::new(dom.clone());
    let (mut element, mut retained) = previous.build(&mut ctx, &mut app);
    let old = sink.route(&ComponentKey::from("a")).unwrap().route;

    let next = model(
        vec![group("left", vec![]), group("right", vec![row("a")])],
        vec![descriptor("a", "row")],
    );
    app.transaction.apply(next, &AllowBindings).unwrap();
    let current = RetainedComposition::<TestState, ()>::new(&app.transaction, app.factory.clone())
        .with_lifecycle_sink(sink.clone());
    current.rebuild(
        &previous,
        &mut retained,
        &mut ctx,
        element.as_mut(),
        &mut app,
    );
    let mut late = MessageCtx::new(
        vec![
            ViewId::new(old.slot),
            ViewId::new(old.route_generation),
            ViewId::new(old.placement_generation),
        ],
        DynMessage::new(PointerClick::at((0.0, 0.0))),
    );
    assert!(matches!(
        current.message(&mut retained, &mut late, element.as_mut(), &mut app),
        MessageResult::Stale
    ));
    assert_eq!(app.clicks, 0);
    assert!(dom.borrow().is_live(element.node));
}

#[test]
fn invalid_model_and_foreign_document_leave_old_dom_and_routes_intact() {
    let source = model(vec![row("a")], vec![descriptor("a", "row")]);
    let mut tx = transaction(source.clone());
    let invalid = model(vec![row("missing")], vec![descriptor("a", "row")]);
    assert!(tx.apply(invalid, &AllowBindings).is_err());
    assert_eq!(tx.accepted(), &source);

    let dom_a: DomHandle = Rc::new(std::cell::RefCell::new(ScriptedDom::new()));
    let dom_b: DomHandle = Rc::new(std::cell::RefCell::new(ScriptedDom::new()));
    let mut state = state(source);
    let view = RetainedComposition::<TestState, ()>::new(&tx, factory(Rc::new(Cell::new(0))))
        .with_lifecycle_sink(state.sink.clone());
    let mut ctx_a = GenetCtx::new(dom_a.clone());
    let (mut element, mut view_state) = view.build(&mut ctx_a, &mut state);
    let node = find_row(&dom_a.borrow(), element.node, "a").unwrap();
    let old_route = state.sink.route(&ComponentKey::from("a")).unwrap();
    let mut ctx_b = GenetCtx::new(dom_b);
    view.rebuild(
        &view,
        &mut view_state,
        &mut ctx_b,
        element.as_mut(),
        &mut state,
    );
    assert!(dom_a.borrow().is_live(node));
    assert_eq!(
        state.sink.route(&ComponentKey::from("a")).unwrap().route,
        old_route.route
    );
    assert_eq!(
        state.sink.last_refusal(),
        Some(LifecycleRefusal::ForeignDocument)
    );
}

#[test]
fn steady_root_rebuild_is_dom_quiet_and_hidden_row_skips_factory() {
    let dom = Rc::new(std::cell::RefCell::new(ScriptedDom::new()));
    let source = model(
        vec![group("settings", vec![row("a")])],
        vec![descriptor("a", "row")],
    );
    let mut runner = GenetAppRunner::new(dom.clone(), logic, state(source));
    let root = runner.root();
    dom.borrow_mut().drain_mutations(&mut Vec::new());

    runner.update(|_| {});
    let mut steady_mutations = Vec::new();
    dom.borrow_mut().drain_mutations(&mut steady_mutations);
    assert!(
        steady_mutations.is_empty(),
        "steady rebuild should not mutate the DOM"
    );

    runner.update(|state| {
        state.collapsed.insert(ContainerKey {
            kind: ContainerKind::Group,
            key: "settings".into(),
        });
    });
    let after_hide = runner.state().factory_builds.get();
    runner.update(|_| {});
    assert_eq!(
        runner.state().factory_builds.get(),
        after_hide,
        "hidden rebuild skips the child factory"
    );
    let node = find_row(&dom.borrow(), root, "a").unwrap();
    assert!(dom.borrow().is_live(node));
}

#[test]
fn final_teardown_disposes_once_and_rejects_late_route_message() {
    let source = model(vec![row("a")], vec![descriptor("a", "row")]);
    let mut app = state(source.clone());
    let sink = app.sink.clone();
    let view = RetainedComposition::<TestState, ()>::new(&app.transaction, app.factory.clone())
        .with_lifecycle_sink(sink.clone());
    let dom: DomHandle = Rc::new(std::cell::RefCell::new(ScriptedDom::new()));
    let mut ctx = GenetCtx::new(dom.clone());
    let (mut element, mut retained) = view.build(&mut ctx, &mut app);
    let route = sink.route(&ComponentKey::from("a")).unwrap().route;
    let child_node = find_row(&dom.borrow(), element.node, "a").unwrap();
    assert!(ctx.has_click(child_node));
    sink.drain();

    view.teardown(&mut retained, &mut ctx, element.as_mut());
    assert!(
        !ctx.has_click(child_node),
        "teardown must unregister handlers"
    );
    assert!(
        !dom.borrow().is_live(child_node),
        "teardown removes the owned child node"
    );
    let disposed = sink
        .drain()
        .into_iter()
        .filter(|event| matches!(event, LifecycleEvent::Disposed { key, .. } if key == &ComponentKey::from("a")))
        .count();
    assert_eq!(disposed, 1);

    let mut late = MessageCtx::new(
        vec![
            ViewId::new(route.slot),
            ViewId::new(route.route_generation),
            ViewId::new(route.placement_generation),
        ],
        DynMessage::new(PointerClick::at((0.0, 0.0))),
    );
    assert!(matches!(
        view.message(&mut retained, &mut late, element.as_mut(), &mut app),
        MessageResult::Stale
    ));
    assert_eq!(app.clicks, 0);
}

#[cfg(test)]
mod admission;

#[test]
fn hiding_focused_row_refuses_until_host_sends_focus_loss() {
    let dom = Rc::new(std::cell::RefCell::new(ScriptedDom::new()));
    let source = model(
        vec![group("settings", vec![row("a")])],
        vec![descriptor("a", "keyed")],
    );
    let mut runner = GenetAppRunner::new(dom.clone(), logic, state(source));
    let root = runner.root();
    let node = find_row(&dom.borrow(), root, "a").unwrap();
    runner.set_focus(Some(node));
    let settings = ContainerKey {
        kind: ContainerKind::Group,
        key: "settings".into(),
    };
    runner.update(|state| {
        state.collapsed.insert(settings.clone());
    });
    assert_eq!(
        runner.state().sink.last_refusal(),
        Some(LifecycleRefusal::FocusedRetirement)
    );
    assert_eq!(
        dom.borrow().parent(node),
        Some(
            dom.borrow()
                .dom_children(root)
                .find(|child| {
                    dom.borrow().attribute(
                        *child,
                        &Namespace::from(""),
                        &LocalName::from("data-composition-key"),
                    ) == Some("settings")
                })
                .unwrap()
        )
    );

    runner.set_focus(None);
    assert!(
        !runner
            .state()
            .sink
            .route(&ComponentKey::from("a"))
            .unwrap()
            .visible
    );
    assert!(dom.borrow().is_live(node));
}
