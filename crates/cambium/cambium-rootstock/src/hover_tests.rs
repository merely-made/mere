// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Hover arriving between a DOM change and the frame that drains it.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use cambium::{AnyView, GenetCtx, GenetElement, button, el};
use layout_dom_api::LayoutDom as _;

use crate::{
    Host, HostHooks, HostOptions, HostState, HostTree as _, HostWake, NodeId, Runner, ScriptedDom,
    WindowDom,
};

type Child = Box<dyn AnyView<bool, (), GenetCtx, GenetElement>>;
type Logic = fn(&bool) -> Child;

/// Two controls share a section; the panel opens outside it. Moving the
/// pointer from one control to the other restyles that section only.
fn view(open: &bool) -> Child {
    let panel = open.then(|| {
        let tools = button("Graph tools", |_: &mut bool, _| {});
        Box::new(el("nav", tools).attr("aria-label", "Graph controls")) as Child
    });
    let map = el("header", button("Map", |_: &mut bool, _| {}));
    let panes = el("aside", button("Panes", |open: &mut bool, _| *open = true));
    Box::new(el("main", (el("section", (map, panes)), panel)))
}

fn rect(host: &Host<bool, Logic, Child>, name: &str) -> Option<(f32, f32, f32, f32)> {
    let runner = host.s.runner.as_ref()?;
    let dom = runner.dom();
    let dom = dom.borrow();
    let view = WindowDom::new(&dom, runner.mount());
    let mut pending = vec![view.document()];
    let mut found: Option<NodeId> = None;
    while let Some(node) = pending.pop() {
        if view
            .element_name(node)
            .is_some_and(|qual| &*qual.local == name)
        {
            found = Some(node);
            break;
        }
        pending.extend(view.flat_children(node));
    }
    host.s.layout.as_ref()?.painted_rect(&view, found?)
}

fn point_at(host: &mut Host<bool, Logic, Child>, name: &str) {
    // The containers span the viewport; their 100px buttons sit at the left.
    let (x, y, _, height) = rect(host, name).expect("the control is laid out");
    host.s.cursor = (x + 10.0, y + height / 2.0);
    host.hover();
}

fn height(host: &Host<bool, Logic, Child>, name: &str) -> f32 {
    rect(host, name).expect("the control is laid out").3
}

/// Opening a panel changes the DOM at once, but the retained style sees the
/// new nodes only when the next relayout drains the mutations. A pointer move
/// in between used to cascade without them and lay the panel out on initial
/// values; in Graphshell's Graph controls that text then failed Genet's
/// used-colour assertion (`color: CanvasText` reached text shaping).
#[test]
fn hover_over_undrained_mutations_waits_for_the_next_relayout() {
    let mut state: HostState<bool, Logic, Child> = HostState::new();
    state.shared.sheet = "main, section, header, aside { display: block; } \
        button { display: block; width: 100px; height: 30px; } \
        button:hover { height: 50px; } nav { display: block; border: 1px solid; }"
        .into();
    state.runner = Some(Runner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        view as Logic,
        false,
    ));
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host = Host::new(
        HostOptions::default(),
        None,
        HostHooks::inert(),
        state,
        wake,
    );
    host.relayout(320.0, 200.0);
    point_at(&mut host, "header");
    assert_eq!(
        height(&host, "header"),
        50.0,
        "a settled DOM restyles at once"
    );

    host.s.runner.as_mut().unwrap().update(|open| *open = true);
    point_at(&mut host, "aside");
    assert_eq!(
        (height(&host, "header"), height(&host, "aside")),
        (50.0, 30.0),
        "the hover restyle waits for the relayout"
    );
    assert_eq!(
        rect(&host, "nav"),
        None,
        "the panel is not laid out unstyled"
    );

    host.relayout(320.0, 200.0);
    assert_eq!(
        (height(&host, "header"), height(&host, "aside")),
        (30.0, 50.0),
        "the relayout applies the hover"
    );
    assert!(
        rect(&host, "nav").is_some(),
        "and lays out the panel it drained"
    );
}
