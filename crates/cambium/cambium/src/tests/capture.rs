/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, NodeKind};

use crate::{
    DomHandle, GenetAppRunner, GenetCtx, GenetElement, Key, KeyEvent, PointerClick, View, el,
    on_click, on_key,
};

/// The app state: an ordered firing log every handler appends to.
struct Log {
    events: Vec<String>,
}

/// The first element in `node`'s subtree (pre-order) named `name`.
fn find_element_by_name(dom: &ScriptedDom, node: NodeId, name: &str) -> Option<NodeId> {
    if dom.kind(node) == NodeKind::Element
        && dom
            .element_name(node)
            .is_some_and(|q| AsRef::<str>::as_ref(&q.local) == name)
    {
        return Some(node);
    }
    dom.dom_children(node)
        .find_map(|c| find_element_by_name(dom, c, name))
}

fn ch(s: &str) -> KeyEvent {
    KeyEvent::new(Key::Character(s.to_string()))
}

// --- click ordering: capture parent before bubble child -------------------

/// `<div on_click(capture) ><button on_click /></div>`: a capture-phase
/// handler on the parent div, a default (bubble) handler on the child button.
/// Each logs its label. Clicking the button must fire the capture parent
/// *before* the bubble child.
fn click_phase_view(_s: &Log) -> impl View<Log, (), GenetCtx, Element = GenetElement> + use<> {
    on_click(
        el::<_, Log, ()>(
            "div",
            on_click(el::<_, Log, ()>("button", "+"), |s: &mut Log, _ev| {
                s.events.push("bubble-child".to_string());
            }),
        ),
        |s: &mut Log, _ev| s.events.push("capture-parent".to_string()),
    )
    .capture(true)
}

#[test]
fn click_capture_parent_fires_before_bubble_child() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        click_phase_view,
        Log { events: Vec::new() },
    );
    let root = runner.root();

    let button = {
        let d = dom.borrow();
        find_element_by_name(&d, root, "button").expect("a <button>")
    };

    runner.dispatch_click(button, PointerClick::at((0.0, 0.0)));

    assert_eq!(
        runner.state().events,
        vec!["capture-parent".to_string(), "bubble-child".to_string()],
        "capture ancestor fires before the bubble target (capture → target → bubble)"
    );
}

// --- stacked listeners: a node carries several of one kind ----------------

/// `on_click(on_click(el("button"), inner), outer)`: two click listeners stack
/// on one element. Both must fire — a second `on_click` no longer silently
/// clobbers the first (the Vec-per-node registry, G2.3). Within the shared
/// bubble phase they fire in registration order, and `build` registers the
/// inner before the outer.
fn stacked_click_view(_s: &Log) -> impl View<Log, (), GenetCtx, Element = GenetElement> + use<> {
    on_click(
        on_click(el::<_, Log, ()>("button", "+"), |s: &mut Log, _ev| {
            s.events.push("inner".to_string());
        }),
        |s: &mut Log, _ev| s.events.push("outer".to_string()),
    )
}

#[test]
fn stacked_click_listeners_all_fire() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        stacked_click_view,
        Log { events: Vec::new() },
    );
    let button = {
        let d = dom.borrow();
        find_element_by_name(&d, runner.root(), "button").expect("a <button>")
    };

    runner.dispatch_click(button, PointerClick::at((0.0, 0.0)));

    assert_eq!(
        runner.state().events,
        vec!["inner".to_string(), "outer".to_string()],
        "both stacked click listeners fire in registration order; neither clobbers the other",
    );
}

// --- key ordering: same, via dispatch_key ---------------------------------

/// `<div on_key(capture)><span on_key /></div>`: a capture-phase key handler
/// on the div, a default (bubble) key handler on the span. Focus the span and
/// dispatch a key — the capture div must fire before the bubble span.
fn key_phase_view(_s: &Log) -> impl View<Log, (), GenetCtx, Element = GenetElement> + use<> {
    on_key(
        el::<_, Log, ()>(
            "div",
            on_key(el::<_, Log, ()>("span", "x"), |s: &mut Log, _ev| {
                s.events.push("bubble-child".to_string());
            }),
        ),
        |s: &mut Log, _ev| s.events.push("capture-parent".to_string()),
    )
    .capture(true)
}

#[test]
fn key_capture_parent_fires_before_bubble_child() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner =
        GenetAppRunner::<_, _, _, ()>::new(dom.clone(), key_phase_view, Log { events: Vec::new() });
    let root = runner.root();

    let span = {
        let d = dom.borrow();
        find_element_by_name(&d, root, "span").expect("a <span>")
    };

    // The span carries a (bubble) key handler, so it is focusable.
    runner.set_focus(Some(span));
    runner.dispatch_key(ch("a"));

    assert_eq!(
        runner.state().events,
        vec!["capture-parent".to_string(), "bubble-child".to_string()],
        "capture key ancestor fires before the bubble focused node"
    );
}

// --- default is bubble ----------------------------------------------------

/// A default `on_click` (no `.capture()`) on the parent, a `.capture(true)`
/// handler on... the same parent? No — to show the default sits in the bubble
/// pass *after* a capture ancestor, the capture handler is on the (outer)
/// grandparent and the default on the (inner) child. Click the child: the
/// capture grandparent fires first, the default child last, proving the
/// no-`.capture()` listener only runs in the bubble pass.
fn default_is_bubble_view(
    _s: &Log,
) -> impl View<Log, (), GenetCtx, Element = GenetElement> + use<> {
    on_click(
        el::<_, Log, ()>(
            "section",
            on_click(el::<_, Log, ()>("button", "+"), |s: &mut Log, _ev| {
                // No `.capture()` → default bubble. Must fire AFTER the
                // capture grandparent, never before it.
                s.events.push("default-child".to_string());
            }),
        ),
        |s: &mut Log, _ev| s.events.push("capture-grandparent".to_string()),
    )
    .capture(true)
}

#[test]
fn default_on_click_only_fires_in_bubble_pass() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        default_is_bubble_view,
        Log { events: Vec::new() },
    );
    let root = runner.root();

    let button = {
        let d = dom.borrow();
        find_element_by_name(&d, root, "button").expect("a <button>")
    };

    runner.dispatch_click(button, PointerClick::at((0.0, 0.0)));

    assert_eq!(
        runner.state().events,
        vec![
            "capture-grandparent".to_string(),
            "default-child".to_string()
        ],
        "a default (no `.capture()`) listener fires only in the bubble pass, \
             after the capture ancestor"
    );
}

// --- capture-only ancestor fires on a descendant click --------------------

/// `<div on_click(capture)><button /></div>`: only the div carries a handler,
/// and it is capture-phase. The button has no handler. Clicking the button (a
/// descendant) must still fire the capture ancestor.
fn capture_only_ancestor_view(
    _s: &Log,
) -> impl View<Log, (), GenetCtx, Element = GenetElement> + use<> {
    on_click(
        el::<_, Log, ()>("div", el::<_, Log, ()>("button", "+")),
        |s: &mut Log, _ev| s.events.push("capture-ancestor".to_string()),
    )
    .capture(true)
}

#[test]
fn capture_only_ancestor_fires_on_descendant_click() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        capture_only_ancestor_view,
        Log { events: Vec::new() },
    );
    let root = runner.root();

    let button = {
        let d = dom.borrow();
        find_element_by_name(&d, root, "button").expect("a <button>")
    };
    assert_ne!(button, root, "button is a descendant of the capture div");

    runner.dispatch_click(button, PointerClick::at((0.0, 0.0)));

    assert_eq!(
        runner.state().events,
        vec!["capture-ancestor".to_string()],
        "a capture-only ancestor fires when a handler-less descendant is clicked"
    );
}

// --- a node's listener fires in exactly one phase -------------------------

/// A single capture handler on a node that is itself the click target fires
/// exactly once (in the capture pass), never twice — the target-node listener
/// is in whichever single phase it registered.
fn single_capture_target_view(
    _s: &Log,
) -> impl View<Log, (), GenetCtx, Element = GenetElement> + use<> {
    on_click(el::<_, Log, ()>("button", "+"), |s: &mut Log, _ev| {
        s.events.push("fired".to_string());
    })
    .capture(true)
}

#[test]
fn target_listener_fires_in_exactly_one_phase() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        single_capture_target_view,
        Log { events: Vec::new() },
    );
    let button = runner.root();

    runner.dispatch_click(button, PointerClick::at((0.0, 0.0)));

    assert_eq!(
        runner.state().events,
        vec!["fired".to_string()],
        "a capture listener on the target itself fires once, not in both passes"
    );
}

// --- MARK: native cancellation (stopPropagation / preventDefault) ---------
//
// The native column of the scenario table both owners satisfy through the
// one `genet_events::dispatch` walk (genet
// `docs/2026-10-17_native_event_dispatch_plan.md`). The JS-side equivalents
// live in `script-runtime-api`'s `dom_node_events_work` (run on Boa + Nova);
// the full table, including stacked listeners, is in `event_layer_tests`.

/// `<div on_click><button on_click(stops) /></div>`: the child (target,
/// bubble) calls `stop_propagation`, so the parent's bubble handler must NOT
/// fire — the native counterpart of the JS `stop` scenario.
fn click_stop_view(_s: &Log) -> impl View<Log, (), GenetCtx, Element = GenetElement> + use<> {
    on_click(
        el::<_, Log, ()>(
            "div",
            on_click(
                el::<_, Log, ()>("button", "+"),
                |s: &mut Log, ev: PointerClick| {
                    s.events.push("child".to_string());
                    ev.stop_propagation();
                },
            ),
        ),
        |s: &mut Log, _ev| s.events.push("parent-SHOULD-NOT".to_string()),
    )
}

#[test]
fn stop_propagation_halts_the_bubble_walk() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        click_stop_view,
        Log { events: Vec::new() },
    );
    let root = runner.root();
    let button = {
        let d = dom.borrow();
        find_element_by_name(&d, root, "button").expect("a <button>")
    };

    runner.dispatch_click(button, PointerClick::at((0.0, 0.0)));

    assert_eq!(
        runner.state().events,
        vec!["child".to_string()],
        "stop_propagation on the target halts the bubble to the parent"
    );
}

/// A handler that calls `prevent_default`; the caller reads the shared
/// Propagation cell *after* dispatch (cloning the handle before the event is
/// moved in) and sees the cancellation — the seam the host uses to gate a
/// default action (form activation, drag start).
fn click_prevent_view(_s: &Log) -> impl View<Log, (), GenetCtx, Element = GenetElement> + use<> {
    on_click(
        el::<_, Log, ()>("button", "+"),
        |s: &mut Log, ev: PointerClick| {
            s.events.push("fired".to_string());
            ev.prevent_default();
        },
    )
}

#[test]
fn prevent_default_is_visible_to_the_caller() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        click_prevent_view,
        Log { events: Vec::new() },
    );
    let button = runner.root();

    // Build the event and keep a clone of its propagation handle; the event
    // is moved into dispatch but the handle shares the same cell, so the
    // host reads the handler's cancellation back here.
    let ev = PointerClick::at((0.0, 0.0));
    let prop = ev.prop.clone();
    assert!(!prop.default_prevented(), "not canceled before dispatch");

    runner.dispatch_click(button, ev);

    assert_eq!(runner.state().events, vec!["fired".to_string()]);
    assert!(
        prop.default_prevented(),
        "the handler's prevent_default is visible through the shared cell after dispatch"
    );
}

/// A click handler that fires but does NOT prevent the default.
fn click_plain_view(_s: &Log) -> impl View<Log, (), GenetCtx, Element = GenetElement> + use<> {
    on_click(
        el::<_, Log, ()>("button", "+"),
        |s: &mut Log, _ev: PointerClick| {
            s.events.push("fired".to_string());
        },
    )
}

/// The host-friendly half of the contract: after dispatch, the *runner*
/// reports `default_prevented()` directly, so a host gates its default action
/// without having to clone the Propagation handle before dispatch.
#[test]
fn runner_reports_default_prevented_after_dispatch() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        click_prevent_view,
        Log { events: Vec::new() },
    );
    let button = runner.root();
    assert!(!runner.default_prevented(), "false before any dispatch");

    runner.dispatch_click(button, PointerClick::at((0.0, 0.0)));
    assert_eq!(runner.state().events, vec!["fired".to_string()]);
    assert!(
        runner.default_prevented(),
        "the handler's prevent_default is visible via the runner accessor"
    );
}

/// A handler that does not prevent leaves `default_prevented()` false, so the
/// host proceeds with its default action; a fresh dispatch resets the flag.
#[test]
fn runner_default_prevented_is_false_without_prevent() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        click_plain_view,
        Log { events: Vec::new() },
    );
    let button = runner.root();
    runner.dispatch_click(button, PointerClick::at((0.0, 0.0)));
    assert_eq!(
        runner.state().events,
        vec!["fired".to_string()],
        "handler ran"
    );
    assert!(
        !runner.default_prevented(),
        "no prevent_default → the host's default action proceeds"
    );
}
