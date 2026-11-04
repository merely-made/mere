/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

/// A focused control may replace itself from its click handler. The old
/// NodeId is retired during the dispatch-tail rebuild, so the runner must
/// clear focus before the next key or host hit can read through it.
#[test]
fn self_replacing_focused_control_retires_interaction_handles() {
    #[derive(Default)]
    struct ReplaceState {
        replaced: bool,
    }

    fn view(state: &ReplaceState) -> Box<dyn AnyView<ReplaceState, (), GenetCtx, GenetElement>> {
        if state.replaced {
            Box::new(el::<_, ReplaceState, ()>("span", ("replacement", "!")))
        } else {
            Box::new(button(
                "replace",
                |state: &mut ReplaceState, _: PointerClick| state.replaced = true,
            ))
        }
    }

    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(dom.clone(), view, ReplaceState::default());
    let retired = runner.root();
    runner.set_focus(Some(retired));

    runner.dispatch_click(retired, PointerClick::at((2.0, 2.0)));

    assert!(runner.state().replaced);
    assert!(!dom.borrow().is_live(retired), "the old button was dropped");
    assert_eq!(runner.focus(), None, "dead focus is cleared after rebuild");
    assert_eq!(runner.pointer_target(retired), None);
    assert_eq!(runner.hover_target(retired), None);
    assert_eq!(runner.wheel_target(retired), None);
    assert!(
        runner
            .dispatch_click(retired, PointerClick::at((2.0, 2.0)))
            .is_empty(),
        "a host-delivered stale hit is safely ignored",
    );
}

/// The Down handler itself can remove the node that just captured the
/// pointer. Capture must be cleared by that same dispatch's rebuild rather
/// than surviving until a Move reads the retired node.
#[test]
fn self_replacing_drag_target_clears_capture() {
    #[derive(Default)]
    struct ReplaceState {
        replaced: bool,
    }

    fn view(state: &ReplaceState) -> Box<dyn AnyView<ReplaceState, (), GenetCtx, GenetElement>> {
        if state.replaced {
            Box::new(el::<_, ReplaceState, ()>("span", ("replacement", "!")))
        } else {
            Box::new(on_pointer(
                el::<_, ReplaceState, ()>("div", "drag target"),
                |state: &mut ReplaceState, event: PointerEvent| {
                    if event.phase == PointerPhase::Down {
                        state.replaced = true;
                    }
                },
            ))
        }
    }

    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(dom.clone(), view, ReplaceState::default());
    let retired = runner.root();
    runner.dispatch_pointer_down(
        retired,
        PointerEvent::new(PointerPhase::Down, (5.0, 5.0), (40.0, 20.0)),
    );

    assert!(!dom.borrow().is_live(retired));
    assert_eq!(runner.pointer_capture(), None);
    assert!(
        runner
            .dispatch_pointer_move(PointerEvent::new(
                PointerPhase::Move,
                (10.0, 5.0),
                (40.0, 20.0),
            ))
            .is_empty(),
    );
}

/// Pointer drag: `down` captures the element and routes a `Down`, `move`
/// routes to the captured element (cursor wherever), `up` routes `Up` and
/// clears capture — the foundation under sliders / scrollbar-drag.
#[test]
fn secondary_capture_is_opt_in_and_keeps_its_qualified_release() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom,
        |_: &Vec<PointerPhase>| {
            on_pointer(
                el::<_, Vec<PointerPhase>, ()>("div", "canvas"),
                |state: &mut Vec<PointerPhase>, event: PointerEvent| {
                    if event.phase == PointerPhase::Down {
                        event.capture_pointer();
                    }
                    state.push(event.phase);
                },
            )
        },
        Vec::<PointerPhase>::new(),
    );
    let node = runner.root();
    let event = |phase| {
        PointerEvent::new(phase, (10.0, 10.0), (20.0, 20.0))
            .with_button(crate::PointerButton::Secondary)
            .with_pointer_id(9)
    };
    runner.dispatch_pointer_down(node, event(PointerPhase::Down));
    assert_eq!(runner.pointer_capture(), Some(node));
    runner.dispatch_pointer_up(event(PointerPhase::Up).with_pointer_id(10));
    assert_eq!(runner.pointer_capture(), Some(node));
    runner.dispatch_pointer_move(event(PointerPhase::Move));
    runner.dispatch_pointer_up(event(PointerPhase::Up));
    assert_eq!(
        runner.state().as_slice(),
        &[PointerPhase::Down, PointerPhase::Move, PointerPhase::Up]
    );
    assert_eq!(runner.pointer_capture(), None);
}

#[test]
fn unrelated_button_release_does_not_end_pointer_capture() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom,
        |_: &Vec<PointerPhase>| {
            on_pointer(
                el::<_, Vec<PointerPhase>, ()>("div", "track"),
                |s: &mut Vec<PointerPhase>, e: PointerEvent| s.push(e.phase),
            )
        },
        Vec::<PointerPhase>::new(),
    );
    let node = runner.root();
    runner.dispatch_pointer_down(
        node,
        PointerEvent::new(PointerPhase::Down, (0.0, 0.0), (20.0, 20.0)),
    );
    runner.dispatch_pointer_up(
        PointerEvent::new(PointerPhase::Up, (0.0, 0.0), (20.0, 20.0))
            .with_button(crate::PointerButton::Secondary),
    );
    assert_eq!(runner.pointer_capture(), Some(node));
    assert_eq!(runner.state().as_slice(), &[PointerPhase::Down]);
}

#[test]
fn pointer_drag_captures_and_routes() {
    #[derive(Default)]
    struct Drag {
        phases: Vec<PointerPhase>,
        last_x: f32,
    }
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |_: &Drag| {
            on_pointer(
                el::<_, Drag, ()>("div", "track"),
                |s: &mut Drag, e: PointerEvent| {
                    s.phases.push(e.phase);
                    s.last_x = e.local.0;
                },
            )
        },
        Drag::default(),
    );
    let node = runner.root();

    runner.dispatch_pointer_down(
        node,
        PointerEvent::new(PointerPhase::Down, (5.0, 0.0), (100.0, 10.0)),
    );
    assert_eq!(
        runner.pointer_capture(),
        Some(node),
        "down captures the element"
    );

    runner.dispatch_pointer_move(PointerEvent::new(
        PointerPhase::Move,
        (40.0, 0.0),
        (100.0, 10.0),
    ));
    runner.dispatch_pointer_up(PointerEvent::new(
        PointerPhase::Up,
        (40.0, 0.0),
        (100.0, 10.0),
    ));
    assert_eq!(runner.pointer_capture(), None, "up clears capture");
    assert_eq!(
        runner.state().phases,
        vec![PointerPhase::Down, PointerPhase::Move, PointerPhase::Up]
    );
    assert_eq!(
        runner.state().last_x,
        40.0,
        "the captured handler saw the move's local x"
    );
}

/// Pointer cancellation (G1.3): a drag handler that calls
/// `e.prop.prevent_default()` is recorded, and — the regression this closes —
/// each pointer pass records its *own* value, so a later un-preventing event
/// resets `default_prevented` rather than inheriting the press's stale `true`
/// (or a prior click/key's).
#[test]
fn each_pointer_event_records_its_own_default_prevented() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |_: &()| {
            on_pointer(
                el::<_, (), ()>("div", "track"),
                |_s: &mut (), e: PointerEvent| {
                    // Prevent the default on the press only; the move does not.
                    if e.phase == PointerPhase::Down {
                        e.prop.prevent_default();
                    }
                },
            )
        },
        (),
    );
    let node = runner.root();

    runner.dispatch_pointer_down(
        node,
        PointerEvent::new(PointerPhase::Down, (5.0, 0.0), (100.0, 10.0)),
    );
    assert!(
        runner.default_prevented(),
        "the press handler's prevent_default is recorded"
    );

    runner.dispatch_pointer_move(PointerEvent::new(
        PointerPhase::Move,
        (40.0, 0.0),
        (100.0, 10.0),
    ));
    assert!(
        !runner.default_prevented(),
        "the move records its own un-prevented value, not the press's stale true",
    );
}

/// Wheel: a notch routes to the nearest ancestor (including the hit node)
/// carrying an `on_wheel` handler, with no capture — each notch resolves its
/// own target and accumulates. A notch whose ancestors carry no handler is a
/// no-op. The scroll foundation parallel to `on_pointer`.
#[test]
fn wheel_routes_to_nearest_handler_no_capture() {
    #[derive(Default)]
    struct Scroll {
        total_y: f32,
        last_local: (f32, f32),
        notches: u32,
    }
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |_: &Scroll| {
            on_wheel(
                el::<_, Scroll, ()>("div", "scroller"),
                |s: &mut Scroll, e: WheelEvent| {
                    s.total_y += e.delta.1;
                    s.last_local = e.local;
                    s.notches += 1;
                },
            )
        },
        Scroll::default(),
    );
    let scroller = runner.root();
    assert_eq!(
        runner.wheel_target(scroller),
        Some(scroller),
        "the scroller is its own target"
    );

    // A notch on the handler node routes to it, carrying delta + cursor-local.
    runner.dispatch_wheel(
        scroller,
        WheelEvent::new((0.0, -30.0), (5.0, 8.0), (100.0, 50.0)),
    );
    assert_eq!(runner.state().notches, 1, "the wheel reached the handler");
    assert_eq!(runner.state().total_y, -30.0);
    assert_eq!(
        runner.state().last_local,
        (5.0, 8.0),
        "the handler saw the cursor-local point"
    );

    // A second notch accumulates — no leftover capture state between notches.
    runner.dispatch_wheel(
        scroller,
        WheelEvent::new((0.0, -12.0), (5.0, 8.0), (100.0, 50.0)),
    );
    assert_eq!(runner.state().notches, 2);
    assert_eq!(
        runner.state().total_y,
        -42.0,
        "successive notches accumulate"
    );

    // A notch whose ancestor chain has no handler (the document, above the
    // scroller) resolves nothing and mutates nothing.
    let doc = dom.borrow().document();
    assert_eq!(
        runner.wheel_target(doc),
        None,
        "no handler above the scroller"
    );
    runner.dispatch_wheel(doc, WheelEvent::new((0.0, -99.0), (0.0, 0.0), (0.0, 0.0)));
    assert_eq!(runner.state().notches, 2, "a handler-less notch is a no-op");
    assert_eq!(runner.state().total_y, -42.0);
}

/// Wheel default-blocking guard: a native wheel handler can cancel the host
/// scroll default for one notch, and the next unprevented notch resets the
/// runner flag. This mirrors the JS passive-wheel guard in script-runtime.
#[test]
fn wheel_prevent_default_blocks_one_scroll_notch() {
    #[derive(Default)]
    struct Scroll {
        prevent_next: bool,
        notches: u32,
    }
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |_: &Scroll| {
            on_wheel(
                el::<_, Scroll, ()>("div", "scroller"),
                |s: &mut Scroll, e: WheelEvent| {
                    s.notches += 1;
                    if s.prevent_next {
                        e.prevent_default();
                        s.prevent_next = false;
                    }
                },
            )
        },
        Scroll {
            prevent_next: true,
            notches: 0,
        },
    );
    let scroller = runner.root();

    runner.dispatch_wheel(
        scroller,
        WheelEvent::new((0.0, -30.0), (5.0, 8.0), (100.0, 50.0)),
    );
    assert!(
        runner.default_prevented(),
        "prevented notch blocks host scroll default"
    );

    runner.dispatch_wheel(
        scroller,
        WheelEvent::new((0.0, -12.0), (5.0, 8.0), (100.0, 50.0)),
    );
    assert!(
        !runner.default_prevented(),
        "the next notch records its own unprevented default state"
    );
    assert_eq!(runner.state().notches, 2);
}

/// `slider`: a press sets the value to the pointer fraction, and a drag
/// (move while captured) tracks it.
#[test]
fn slider_drag_sets_value() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner =
        GenetAppRunner::<_, _, _, ()>::new(dom.clone(), |s: &Slider| slider(s), Slider::default());
    let track = runner.root();

    // Press at 50/100 → value 0.5.
    runner.dispatch_pointer_down(
        track,
        PointerEvent::new(PointerPhase::Down, (50.0, 0.0), (100.0, 12.0)),
    );
    assert!((runner.state().value - 0.5).abs() < 0.001, "press sets 0.5");

    // Drag to 80/100 → value 0.8.
    runner.dispatch_pointer_move(PointerEvent::new(
        PointerPhase::Move,
        (80.0, 0.0),
        (100.0, 12.0),
    ));
    assert!(
        (runner.state().value - 0.8).abs() < 0.001,
        "drag tracks to 0.8"
    );

    // Past the right edge clamps to 1.0.
    runner.dispatch_pointer_move(PointerEvent::new(
        PointerPhase::Move,
        (130.0, 0.0),
        (100.0, 12.0),
    ));
    assert!((runner.state().value - 1.0).abs() < 0.001, "clamps to 1.0");
    runner.dispatch_pointer_up(PointerEvent::new(
        PointerPhase::Up,
        (130.0, 0.0),
        (100.0, 12.0),
    ));
    assert_eq!(runner.pointer_capture(), None, "release ends the drag");

    runner.set_focus(Some(track));
    runner.dispatch_key(named(NamedKey::ArrowLeft));
    assert!((runner.state().value - 0.99).abs() < 0.001);
    runner.dispatch_key(named(NamedKey::Home));
    assert_eq!(runner.state().value, 0.0);
}
