/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Cambium's click, key and focus listeners on the shared event layer (genet
//! `docs/2026-10-17_native_event_dispatch_plan.md`, Phase 2).
//!
//! Three groups: the scenario table of the retired 2026-06-01 convergence plan,
//! kept as a permanent test of the one dispatcher (genet's `dom_node_events_work`
//! is its script column); listeners of two owners sharing one registry; and the
//! queue for a native call that arrives while the runner is mid-walk.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use genet_events::{
    EventInit, Invocation, InvokeResult, Invoker, ListenerSpec, Owner, Registry, TargetKey,
    build_path, dispatch,
};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LayoutDomMut, NodeKind};
use meristem::{View, ViewId};

use crate::{
    CAMBIUM_OWNER, DomHandle, FocusEvent, GenetAppRunner, GenetCtx, GenetElement, Key, KeyEvent,
    NativeOwner, PointerClick, el, html_qual, on_click, on_focus, on_key,
};

type Log = Rc<RefCell<Vec<String>>>;

fn log() -> Log {
    Log::default()
}

fn seen(log: &Log) -> Vec<String> {
    log.borrow().clone()
}

/// A listener that records `name`, for any event.
fn note<E: 'static>(log: &Log, name: &'static str) -> impl Fn(&mut (), E) + 'static + use<E> {
    let log = log.clone();
    move |_, _| log.borrow_mut().push(name.to_owned())
}

/// A click listener that records `name`, then calls `then` on the event.
fn note_click_then(
    log: &Log,
    name: &'static str,
    then: fn(&PointerClick),
) -> impl Fn(&mut (), PointerClick) + 'static + use<> {
    let log = log.clone();
    move |_, event| {
        log.borrow_mut().push(name.to_owned());
        then(&event);
    }
}

fn note_key_then(
    log: &Log,
    name: &'static str,
    then: fn(&KeyEvent),
) -> impl Fn(&mut (), KeyEvent) + 'static + use<> {
    let log = log.clone();
    move |_, event| {
        log.borrow_mut().push(name.to_owned());
        then(&event);
    }
}

/// A runner over a fresh document and a registry the test also holds.
fn start<S, L, V>(
    logic: L,
    state: S,
) -> (
    DomHandle,
    Rc<RefCell<Registry>>,
    GenetAppRunner<S, L, V, ()>,
)
where
    S: 'static,
    L: FnMut(&S) -> V,
    V: View<S, (), GenetCtx, Element = GenetElement>,
{
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let events = Rc::new(RefCell::new(Registry::new()));
    let mount = dom.borrow().document();
    let runner = GenetAppRunner::with_events(dom.clone(), mount, events.clone(), logic, state);
    (dom, events, runner)
}

fn first_named(dom: &DomHandle, root: NodeId, name: &str) -> NodeId {
    fn find(dom: &ScriptedDom, node: NodeId, name: &str) -> Option<NodeId> {
        if dom.kind(node) == NodeKind::Element
            && dom
                .element_name(node)
                .is_some_and(|q| AsRef::<str>::as_ref(&q.local) == name)
        {
            return Some(node);
        }
        dom.dom_children(node).find_map(|c| find(dom, c, name))
    }
    find(&dom.borrow(), root, name).unwrap_or_else(|| panic!("no <{name}>"))
}

fn click() -> PointerClick {
    PointerClick::at((0.0, 0.0))
}

fn key() -> KeyEvent {
    KeyEvent::new(Key::Character("x".to_owned()))
}

// --- MARK: the scenario table -------------------------------------------------
//
// One row per scenario of the 2026-06-01 event-model convergence plan, now
// asserted against the single dispatcher. The script column is genet's
// `dom_node_events_work`, and `genet-events` runs the same rows on its own.
//
// | scenario                                  | expected firing order            | prevented |
// | plain bubble (child + parent listen)      | child, parent                    | no        |
// | non-bubbling event                        | the target only                  | no        |
// | stopPropagation at child                  | the child's node, not the parent | no        |
// | stopImmediatePropagation, node of two     | first listener only              | no        |
// | preventDefault                            | handler fires                    | yes       |

/// `<div on_click(parent)><button on_click(child)/></div>`.
fn nested_clicks<C, P>(
    child: C,
    parent: P,
) -> impl View<(), (), GenetCtx, Element = GenetElement> + use<C, P>
where
    C: Fn(&mut (), PointerClick) + 'static,
    P: Fn(&mut (), PointerClick) + 'static,
{
    on_click(
        el::<_, (), ()>("div", on_click(el::<_, (), ()>("button", "+"), child)),
        parent,
    )
}

/// The same with two listeners stacked on the button.
fn stacked_clicks<F, S, P>(
    first: F,
    second: S,
    parent: P,
) -> impl View<(), (), GenetCtx, Element = GenetElement> + use<F, S, P>
where
    F: Fn(&mut (), PointerClick) + 'static,
    S: Fn(&mut (), PointerClick) + 'static,
    P: Fn(&mut (), PointerClick) + 'static,
{
    on_click(
        el::<_, (), ()>(
            "div",
            on_click(on_click(el::<_, (), ()>("button", "+"), first), second),
        ),
        parent,
    )
}

#[test]
fn table_plain_bubble_fires_child_then_parent() {
    let log = log();
    let (dom, _, mut runner) = start(
        {
            let log = log.clone();
            move |_: &()| nested_clicks(note(&log, "child"), note(&log, "parent"))
        },
        (),
    );
    let button = first_named(&dom, runner.root(), "button");

    runner.dispatch_click(button, click());

    assert_eq!(seen(&log), ["child", "parent"]);
    assert!(!runner.default_prevented());
}

#[test]
fn table_non_bubbling_event_reaches_only_its_target() {
    let log = log();
    let (dom, _, mut runner) = start(
        {
            let log = log.clone();
            move |_: &()| {
                on_focus(
                    el::<_, (), ()>(
                        "div",
                        on_focus(el::<_, (), ()>("button", "+"), {
                            let log = log.clone();
                            move |_: &mut (), ev: FocusEvent| {
                                log.borrow_mut().push(format!("child {:?}", ev.phase));
                            }
                        }),
                    ),
                    {
                        let log = log.clone();
                        move |_: &mut (), ev: FocusEvent| {
                            log.borrow_mut().push(format!("parent {:?}", ev.phase));
                        }
                    },
                )
            }
        },
        (),
    );
    let button = first_named(&dom, runner.root(), "button");

    runner.set_focus(Some(button));
    assert_eq!(
        seen(&log),
        ["child Gained"],
        "focus does not bubble to the parent's observer"
    );

    runner.set_focus(None);
    assert_eq!(seen(&log), ["child Gained", "child Lost"]);
}

#[test]
fn table_stop_propagation_ends_the_walk_after_the_current_node() {
    let log = log();
    let (dom, _, mut runner) = start(
        {
            let log = log.clone();
            move |_: &()| {
                stacked_clicks(
                    note_click_then(&log, "first", PointerClick::stop_propagation),
                    note(&log, "second"),
                    note(&log, "parent"),
                )
            }
        },
        (),
    );
    let button = first_named(&dom, runner.root(), "button");

    runner.dispatch_click(button, click());

    assert_eq!(
        seen(&log),
        ["first", "second"],
        "the node's other listeners still run; the parent is skipped"
    );
}

#[test]
fn table_stop_immediate_propagation_skips_the_nodes_other_listeners() {
    let log = log();
    let (dom, _, mut runner) = start(
        {
            let log = log.clone();
            move |_: &()| {
                stacked_clicks(
                    note_click_then(&log, "first", PointerClick::stop_immediate_propagation),
                    note(&log, "second"),
                    note(&log, "parent"),
                )
            }
        },
        (),
    );
    let button = first_named(&dom, runner.root(), "button");

    runner.dispatch_click(button, click());

    assert_eq!(seen(&log), ["first"]);
}

#[test]
fn table_prevent_default_marks_the_event_and_the_runner() {
    let log = log();
    let (dom, _, mut runner) = start(
        {
            let log = log.clone();
            move |_: &()| {
                nested_clicks(
                    note_click_then(&log, "handler", PointerClick::prevent_default),
                    note(&log, "parent"),
                )
            }
        },
        (),
    );
    let button = first_named(&dom, runner.root(), "button");
    let event = click();
    let prop = event.prop.clone();

    runner.dispatch_click(button, event);

    assert_eq!(seen(&log), ["handler", "parent"], "prevent does not stop");
    assert!(prop.default_prevented());
    assert!(runner.default_prevented());
}

/// The same rows through the key walk, which shares the dispatcher.
fn key_tree<F, S, P>(
    first: F,
    second: S,
    parent: P,
) -> impl View<(), (), GenetCtx, Element = GenetElement> + use<F, S, P>
where
    F: Fn(&mut (), KeyEvent) + 'static,
    S: Fn(&mut (), KeyEvent) + 'static,
    P: Fn(&mut (), KeyEvent) + 'static,
{
    on_key(
        el::<_, (), ()>(
            "div",
            on_key(on_key(el::<_, (), ()>("button", "+"), first), second),
        ),
        parent,
    )
}

#[test]
fn table_key_walk_bubbles_stops_and_prevents_like_the_click_walk() {
    // Bubble to the parent, stop at the node, stop-immediate and prevent.
    let rows: [(&str, fn(&KeyEvent), &[&str], bool); 4] = [
        ("bubble", |_| {}, &["first", "second", "parent"], false),
        (
            "stop",
            KeyEvent::stop_propagation,
            &["first", "second"],
            false,
        ),
        (
            "stop-immediate",
            KeyEvent::stop_immediate_propagation,
            &["first"],
            false,
        ),
        (
            "prevent",
            KeyEvent::prevent_default,
            &["first", "second", "parent"],
            true,
        ),
    ];
    for (row, then, expected, prevented) in rows {
        let log = log();
        let (dom, _, mut runner) = start(
            {
                let log = log.clone();
                move |_: &()| {
                    key_tree(
                        note_key_then(&log, "first", then),
                        note(&log, "second"),
                        note(&log, "parent"),
                    )
                }
            },
            (),
        );
        let button = first_named(&dom, runner.root(), "button");
        runner.set_focus(Some(button));

        runner.dispatch_key(key());

        assert_eq!(seen(&log), expected, "row {row}");
        assert_eq!(runner.default_prevented(), prevented, "row {row}");
    }
}

// --- MARK: listeners of two owners on one registry ----------------------------

/// Records which owner and token each listener belongs to, in the order the
/// walk reaches them.
struct Recorder(Rc<RefCell<Vec<(Owner, u64)>>>);

impl Invoker for Recorder {
    fn invoke(&mut self, call: Invocation<'_>) -> InvokeResult {
        self.0
            .borrow_mut()
            .push((call.listener.owner, call.listener.token));
        InvokeResult::default()
    }
}

fn script_listener(event_type: &str, token: u64, capture: bool) -> ListenerSpec {
    ListenerSpec {
        owner: Owner::SCRIPT,
        token,
        event_type: event_type.to_owned(),
        capture,
        once: false,
        passive: false,
    }
}

#[test]
fn script_and_cambium_listeners_on_one_node_interleave_by_registration_order() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let events = Rc::new(RefCell::new(Registry::new()));
    let mut ctx = GenetCtx::with_events(dom.clone(), events.clone());
    let node = {
        let mut d = dom.borrow_mut();
        let node = d.create_element(html_qual("button"));
        let doc = d.document();
        d.insert_before(doc, node, None);
        node
    };
    let at = TargetKey::Node(node);
    let path = |n: u64| vec![ViewId::new(n)];

    // Cambium, script, Cambium, script: one list, in this order.
    ctx.register_click(node, path(1), false);
    events
        .borrow_mut()
        .add(at, script_listener("click", 10, false));
    ctx.register_click(node, path(2), false);
    events
        .borrow_mut()
        .add(at, script_listener("click", 11, false));

    let ran = Rc::new(RefCell::new(Vec::new()));
    let route = build_path(&dom.borrow(), at, None, "click", true);
    let init = EventInit {
        event_type: "click".to_owned(),
        bubbles: true,
        stop_propagation: false,
        stop_immediate: false,
        canceled: false,
        activation_event: false,
    };
    dispatch(&events, route, &init, &mut Recorder(ran.clone()));

    let owners: Vec<Owner> = ran.borrow().iter().map(|(owner, _)| *owner).collect();
    assert_eq!(
        owners,
        [CAMBIUM_OWNER, Owner::SCRIPT, CAMBIUM_OWNER, Owner::SCRIPT]
    );
    // Each Cambium token resolves to the view path it was registered with.
    let paths: Vec<Option<Vec<ViewId>>> = ran
        .borrow()
        .iter()
        .filter(|(owner, _)| *owner == CAMBIUM_OWNER)
        .map(|(_, token)| ctx.listeners().path(*token).map(<[ViewId]>::to_vec))
        .collect();
    assert_eq!(paths, [Some(path(1)), Some(path(2))]);
}

/// A script's listener, as the runner's walk reaches it: records itself.
struct Script(Log);

impl Invoker for Script {
    fn invoke(&mut self, call: Invocation<'_>) -> InvokeResult {
        self.0
            .borrow_mut()
            .push(format!("script {}", call.listener.token));
        InvokeResult::default()
    }
}

#[test]
fn a_runner_walk_runs_a_shared_registrys_script_listeners_in_place() {
    let log = log();
    let (_, events, mut runner) = start(
        {
            let log = log.clone();
            move |capture: &bool| {
                on_key(el::<_, bool, ()>("button", "+"), {
                    let log = log.clone();
                    move |_: &mut bool, _: KeyEvent| log.borrow_mut().push("cambium".into())
                })
                .capture(*capture)
            }
        },
        false,
    );
    runner.set_foreign_invoker(Some(Box::new(Script(log.clone()))));
    let button = runner.root();
    runner.set_focus(Some(button));
    events.borrow_mut().add(
        TargetKey::Node(button),
        script_listener("keydown", 1, false),
    );

    runner.dispatch_key(key());
    assert_eq!(
        seen(&log),
        ["cambium", "script 1"],
        "registered in this order"
    );

    // Re-registering Cambium's listener (a rebuild that moves it to the capture
    // pass and back) puts it after the script's: the shared list is ordered by
    // registration, not by owner.
    log.borrow_mut().clear();
    runner.update(|capture| *capture = true);
    runner.update(|capture| *capture = false);
    runner.dispatch_key(key());
    assert_eq!(seen(&log), ["script 1", "cambium"]);
}

#[test]
fn script_listeners_in_the_walk_wrap_cambiums_by_capture_and_bubble() {
    let log = log();
    let (dom, events, mut runner) = start(
        {
            let log = log.clone();
            move |_: &()| {
                on_click(
                    el::<_, (), ()>(
                        "div",
                        on_click(el::<_, (), ()>("button", "+"), note(&log, "cambium")),
                    ),
                    note(&log, "cambium parent"),
                )
            }
        },
        (),
    );
    runner.set_foreign_invoker(Some(Box::new(Script(log.clone()))));
    let button = first_named(&dom, runner.root(), "button");
    let div = first_named(&dom, runner.root(), "div");
    events
        .borrow_mut()
        .add(TargetKey::Node(div), script_listener("click", 1, true));
    events
        .borrow_mut()
        .add(TargetKey::Node(div), script_listener("click", 2, false));

    runner.dispatch_click(button, click());

    assert_eq!(
        seen(&log),
        ["script 1", "cambium", "cambium parent", "script 2"],
        "capture on the div, the button, then the div's bubble in registration order \
         (Cambium's parent listener registered at build, before the script's)"
    );
}

#[test]
fn a_registry_shared_with_no_foreign_invoker_skips_other_owners() {
    let log = log();
    let (_, events, mut runner) = start(
        {
            let log = log.clone();
            move |_: &()| on_click(el::<_, (), ()>("button", "+"), note(&log, "cambium"))
        },
        (),
    );
    events.borrow_mut().add(
        TargetKey::Node(runner.root()),
        script_listener("click", 1, false),
    );

    runner.dispatch_click(runner.root(), click());

    assert_eq!(seen(&log), ["cambium"]);
}

// --- MARK: the token table ----------------------------------------------------

#[test]
fn cambium_listeners_register_and_unregister_in_the_events_registry() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut ctx = GenetCtx::new(dom.clone());
    let events = ctx.events();
    assert!(
        !Rc::ptr_eq(&events, &GenetCtx::new(dom.clone()).events()),
        "each context makes its own registry unless one is handed in"
    );
    let node = dom.borrow_mut().create_element(html_qual("button"));
    let at = TargetKey::Node(node);
    let path = vec![ViewId::new(7)];

    ctx.register_click(node, path.clone(), false);
    ctx.register_key(node, path.clone(), true, true);
    ctx.register_focus(node, path.clone());
    assert!(events.borrow().has_type("click"));
    assert!(events.borrow().has_type("keydown"));
    assert!(events.borrow().has_type("focus") && events.borrow().has_type("blur"));
    assert!(ctx.has_click(node) && ctx.has_key(node) && ctx.is_focusable(node));
    assert_eq!(events.borrow().snapshot(at, "keydown").len(), 1);

    // Registering the same path again replaces it: no duplicate in the list.
    ctx.register_click(node, path.clone(), true);
    let clicks = events.borrow().snapshot(at, "click");
    assert_eq!(clicks.len(), 1);
    assert!(clicks[0].capture, "the phase updated in place");

    ctx.unregister_click(node, &path);
    ctx.unregister_key(node, &path);
    ctx.unregister_focus(node);
    assert!(!ctx.has_click(node) && !ctx.has_key(node) && !ctx.is_focusable(node));
    assert!(
        !events.borrow().has_listeners(at),
        "nothing left in the registry"
    );
    assert!(!events.borrow().has_type("click"));
}

#[test]
fn a_non_focusable_key_listener_does_not_make_its_node_focusable() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut ctx = GenetCtx::new(dom.clone());
    let node = dom.borrow_mut().create_element(html_qual("div"));
    ctx.register_key(node, vec![ViewId::new(1)], false, false);
    assert!(ctx.has_key(node) && !ctx.is_focusable(node));
    ctx.register_focusable(node);
    assert!(ctx.is_focusable(node));
}

// --- MARK: a native call that arrives mid-walk --------------------------------

/// A script listener whose handler dispatches a click again, nested, over the
/// same registry, handing Cambium's owner the runner's queue.
struct Reentrant {
    log: Log,
    events: Rc<RefCell<Registry>>,
    dom: DomHandle,
    target: NodeId,
    native: NativeOwner,
    entered: Cell<bool>,
}

/// The nested dispatch's invoker: script listeners run, Cambium's go to the
/// runner's queue.
struct Nested<'a> {
    log: &'a Log,
    native: &'a mut NativeOwner,
}

impl Invoker for Nested<'_> {
    fn invoke(&mut self, call: Invocation<'_>) -> InvokeResult {
        if call.listener.owner == CAMBIUM_OWNER {
            self.log.borrow_mut().push("queue cambium".into());
            return self.native.invoke(call);
        }
        self.log.borrow_mut().push("script nested".into());
        InvokeResult::default()
    }
}

impl Invoker for Reentrant {
    fn invoke(&mut self, _call: Invocation<'_>) -> InvokeResult {
        self.log.borrow_mut().push("script".into());
        if !self.entered.replace(true) {
            let route = build_path(
                &self.dom.borrow(),
                TargetKey::Node(self.target),
                None,
                "click",
                true,
            );
            let init = EventInit {
                event_type: "click".to_owned(),
                bubbles: true,
                stop_propagation: false,
                stop_immediate: false,
                canceled: false,
                activation_event: false,
            };
            let mut nested = Nested {
                log: &self.log,
                native: &mut self.native,
            };
            dispatch(&self.events, route, &init, &mut nested);
        }
        InvokeResult::default()
    }
}

#[test]
fn a_nested_native_call_is_queued_and_runs_when_the_outer_walk_unwinds() {
    let log = log();
    let (dom, events, mut runner) = start(
        {
            let log = log.clone();
            move |_: &()| {
                on_click(
                    el::<_, (), ()>(
                        "div",
                        on_click(el::<_, (), ()>("button", "+"), note(&log, "cambium button")),
                    ),
                    note(&log, "cambium div"),
                )
            }
        },
        (),
    );
    let button = first_named(&dom, runner.root(), "button");
    // A script listener after Cambium's on the button; its handler re-enters.
    events
        .borrow_mut()
        .add(TargetKey::Node(button), script_listener("click", 1, false));
    runner.set_foreign_invoker(Some(Box::new(Reentrant {
        log: log.clone(),
        events: events.clone(),
        dom: dom.clone(),
        target: button,
        native: runner.native_owner(),
        entered: Cell::new(false),
    })));

    runner.dispatch_click(button, click());

    // The outer walk runs to its end first: button, the script listener (whose
    // nested walk only queues Cambium's two listeners), then the div. The
    // queued pair runs after, in the order they were reached. A synchronous
    // nested walk would have put "cambium button" before "script nested".
    assert_eq!(
        seen(&log),
        [
            "cambium button",
            "script",
            "queue cambium",
            "script nested",
            "queue cambium",
            "cambium div",
            "cambium button",
            "cambium div",
        ]
    );
}
