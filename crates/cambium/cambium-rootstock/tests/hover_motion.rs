// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Hovering a spatial region uses the host's real layout and input path.
use cambium::{AnyView, GenetAppRunner, GenetCtx, GenetElement, HoverPhase, el, on_hover};
use cambium_rootstock::{Host, HostHooks, HostOptions, HostState, HostWake, ScriptedDom};
use layout_dom_api::LayoutDom as _;
use std::{cell::RefCell, rc::Rc, sync::Arc};

#[derive(Default)]
struct State(Vec<(HoverPhase, (f32, f32), (f32, f32))>);
type Child = Box<dyn AnyView<State, (), GenetCtx, GenetElement>>;
type Logic = fn(&State) -> Child;
fn view(_: &State) -> Child {
    Box::new(on_hover(
        el("div", el("div", ()).attr("class", "child")).attr("class", "zone"),
        |state: &mut State, event: cambium::HoverEvent| {
            state.0.push((event.phase, event.local, event.size));
            event.defer_rebuild();
        },
    ))
}
#[test]
fn hover_motion_resolves_ancestor_coordinates_and_leaves_without_clicking() {
    let mut state = HostState::<State, Logic, Child>::new();
    state.shared.sheet = ".zone { position:absolute;left:40px;top:30px;width:200px;height:100px; } .child { width:80px;height:50px;margin-left:20px; }".into();
    state.runner = Some(GenetAppRunner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        view as Logic,
        State::default(),
    ));
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host = Host::new(
        HostOptions::default(),
        None,
        HostHooks::inert(),
        state,
        wake,
    );
    host.set_surface_size(400., 300.);
    host.relayout(400., 300.);
    host.pointer_moved(65., 40.); // descendant of the registered zone
    host.pointer_moved(70., 45.); // ordinary movement with no button held
    host.pointer_moved(350., 250.);
    let events = &host.s.runner.as_ref().unwrap().state().0;
    assert_eq!(events.len(), 3);
    assert_eq!(events[0], (HoverPhase::Enter, (25., 10.), (200., 100.)));
    assert_eq!(events[1], (HoverPhase::Move, (30., 15.), (200., 100.)));
    assert_eq!(events[2].0, HoverPhase::Leave);
}

#[derive(Default)]
struct ReplacementState {
    replaced: bool,
    replace_on_move: bool,
    events: Vec<(&'static str, HoverPhase)>,
}
type ReplacementChild = Box<dyn AnyView<ReplacementState, (), GenetCtx, GenetElement>>;
type ReplacementLogic = fn(&ReplacementState) -> ReplacementChild;

fn replacement_view(state: &ReplacementState) -> ReplacementChild {
    if state.replaced {
        Box::new(el(
            "section",
            on_hover(
                el("div", ()).attr("class", "zone"),
                |state: &mut ReplacementState, event: cambium::HoverEvent| {
                    state.events.push(("new", event.phase));
                    event.defer_rebuild();
                },
            ),
        ))
    } else {
        Box::new(on_hover(
            el("div", ()).attr("class", "zone"),
            |state: &mut ReplacementState, event: cambium::HoverEvent| {
                state.events.push(("old", event.phase));
                if state.replace_on_move && event.phase == HoverPhase::Move {
                    state.replaced = true;
                } else {
                    event.defer_rebuild();
                }
            },
        ))
    }
}

#[test]
fn replacing_a_hovered_subtree_does_not_deliver_leave_to_its_retired_node() {
    let mut state = HostState::<ReplacementState, ReplacementLogic, ReplacementChild>::new();
    state.shared.sheet =
        ".zone { position:absolute;left:40px;top:30px;width:200px;height:100px; }".into();
    state.runner = Some(GenetAppRunner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        replacement_view as ReplacementLogic,
        ReplacementState::default(),
    ));
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host = Host::new(
        HostOptions::default(),
        None,
        HostHooks::inert(),
        state,
        wake,
    );
    host.set_surface_size(400., 300.);
    host.relayout(400., 300.);
    host.pointer_moved(65., 40.);
    let retired = host
        .s
        .last_hover_hit
        .expect("the old subtree received Enter");
    host.s
        .runner
        .as_mut()
        .unwrap()
        .update(|state| state.replaced = true);
    assert!(
        !host
            .s
            .runner
            .as_ref()
            .unwrap()
            .dom()
            .borrow()
            .is_live(retired)
    );
    host.relayout(400., 300.);
    host.pointer_moved(65., 40.);
    host.pointer_moved(70., 45.);
    host.pointer_moved(350., 250.);
    assert_eq!(host.s.last_hover_hit, None);
    assert_eq!(
        host.s.runner.as_ref().unwrap().state().events,
        [
            ("old", HoverPhase::Enter),
            ("new", HoverPhase::Enter),
            ("new", HoverPhase::Move),
            ("new", HoverPhase::Leave),
        ]
    );
}

#[derive(Default)]
struct LeaveState {
    hide_next: bool,
    events: Vec<(&'static str, HoverPhase)>,
}
type LeaveChild = Box<dyn AnyView<LeaveState, (), GenetCtx, GenetElement>>;
type LeaveLogic = fn(&LeaveState) -> LeaveChild;
fn leave_view(state: &LeaveState) -> LeaveChild {
    Box::new(el(
        "main",
        (
            on_hover(
                el("div", ()).attr("class", "old-zone"),
                |state: &mut LeaveState, event: cambium::HoverEvent| {
                    state.events.push(("old", event.phase));
                    if event.phase == HoverPhase::Leave {
                        state.hide_next = true;
                    } else {
                        event.defer_rebuild();
                    }
                },
            ),
            (!state.hide_next).then(|| {
                on_hover(
                    el("div", ()).attr("class", "next-zone"),
                    |state: &mut LeaveState, event: cambium::HoverEvent| {
                        state.events.push(("next", event.phase));
                        event.defer_rebuild();
                    },
                )
            }),
        ),
    ))
}

#[test]
fn a_leave_handler_can_retire_the_next_hover_target_without_an_enter() {
    let mut state = HostState::<LeaveState, LeaveLogic, LeaveChild>::new();
    state.shared.sheet = ".old-zone { position:absolute;left:40px;top:30px;width:100px;height:100px; } .next-zone { position:absolute;left:200px;top:30px;width:100px;height:100px; }".into();
    state.runner = Some(GenetAppRunner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        leave_view as LeaveLogic,
        LeaveState::default(),
    ));
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host = Host::new(
        HostOptions::default(),
        None,
        HostHooks::inert(),
        state,
        wake,
    );
    host.set_surface_size(400., 300.);
    host.relayout(400., 300.);
    host.pointer_moved(65., 40.);
    host.pointer_moved(225., 40.);
    assert_eq!(
        host.s.last_hover_hit, None,
        "the retired prospective target is cleared"
    );
    assert_eq!(
        host.s.runner.as_ref().unwrap().state().events,
        [("old", HoverPhase::Enter), ("old", HoverPhase::Leave),]
    );
}

#[test]
fn a_move_handler_can_replace_its_own_hovered_subtree_before_the_next_event() {
    let mut state = HostState::<ReplacementState, ReplacementLogic, ReplacementChild>::new();
    state.shared.sheet =
        ".zone { position:absolute;left:40px;top:30px;width:200px;height:100px; }".into();
    state.runner = Some(GenetAppRunner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        replacement_view as ReplacementLogic,
        ReplacementState {
            replace_on_move: true,
            ..Default::default()
        },
    ));
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host = Host::new(
        HostOptions::default(),
        None,
        HostHooks::inert(),
        state,
        wake,
    );
    host.set_surface_size(400., 300.);
    host.relayout(400., 300.);
    host.pointer_moved(65., 40.);
    let retired = host.s.last_hover_hit.unwrap();
    host.pointer_moved(70., 45.);
    assert!(
        !host
            .s
            .runner
            .as_ref()
            .unwrap()
            .dom()
            .borrow()
            .is_live(retired)
    );
    host.relayout(400., 300.);
    host.pointer_moved(75., 50.);
    assert_eq!(
        host.s.runner.as_ref().unwrap().state().events,
        [
            ("old", HoverPhase::Enter),
            ("old", HoverPhase::Move),
            ("new", HoverPhase::Enter),
        ]
    );
    assert_ne!(host.s.last_hover_hit, Some(retired));
}
