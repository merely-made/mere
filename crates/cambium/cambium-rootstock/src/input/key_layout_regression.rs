// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use crate::HostTree as _;
use cambium::{AnyView, GenetCtx, GenetElement, PointerClick, button, el};
use std::{cell::RefCell, rc::Rc, sync::Arc};

struct App {
    reveal: bool,
}
type Child = Box<dyn AnyView<App, (), GenetCtx, GenetElement>>;
type Logic = fn(&App) -> Child;

fn view(app: &App) -> Child {
    let first = button("first", |_: &mut App, _: PointerClick| {});
    if app.reveal {
        Box::new(el(
            "main",
            (first, button("inserted", |_: &mut App, _: PointerClick| {})),
        ))
    } else {
        Box::new(el("main", first))
    }
}

#[test]
fn key_dispatch_relayouts_inserted_focused_control_before_focus_restyle() {
    let mut state: crate::HostState<App, Logic, Child> = crate::HostState::new();
    state.shared.sheet = "main { display:block; } button { display:block; width:100px; height:30px; } button:focus { height:80px; }".into();
    state.runner = Some(crate::Runner::new(
        Rc::new(RefCell::new(crate::ScriptedDom::new())),
        view as Logic,
        App { reveal: false },
    ));
    let mut hooks: crate::HostHooks<App, Logic, Child> = crate::HostHooks::inert();
    hooks.after_dispatch = Box::new(|ctx| {
        ctx.runner.update(|app| app.reveal = true);
        let inserted = ctx.runner.focusables().last().copied();
        ctx.runner.set_focus(inserted);
    });
    let wake = crate::HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host = crate::Host::new(crate::HostOptions::default(), None, hooks, state, wake);
    host.relayout(320.0, 180.0);
    let first = host.s.runner.as_ref().unwrap().focusables()[0];
    host.s.runner.as_mut().unwrap().set_focus(Some(first));

    host.key(&crate::KeyPress::named(crate::NamedKey::Enter));

    let runner = host.s.runner.as_ref().unwrap();
    let inserted = *runner.focusables().last().unwrap();
    assert_eq!(runner.focus(), Some(inserted));
    let dom = runner.dom();
    let dom = dom.borrow();
    let window = crate::WindowDom::new(&dom, runner.mount());
    let rect = host
        .s
        .layout
        .as_ref()
        .unwrap()
        .painted_rect(&window, inserted)
        .unwrap();
    assert_eq!(rect.3, 80.0);
}
