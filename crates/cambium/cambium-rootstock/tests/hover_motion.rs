// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Hovering a spatial region uses the host's real layout and input path.
use cambium::{AnyView, GenetAppRunner, GenetCtx, GenetElement, HoverPhase, el, on_hover};
use cambium_rootstock::{Host, HostHooks, HostOptions, HostState, HostWake, ScriptedDom};
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
