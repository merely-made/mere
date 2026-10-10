// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Accessibility focus follows the runner, independently of hover restyling.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use cambium::{AnyView, GenetCtx, GenetElement, button, el};
use layout_dom_api::LayoutDom as _;

use crate::{
    A11yRequest, Accessibility, Host, HostHooks, HostOptions, HostState, HostTree as _, HostWake,
    OwnedLayout, ProducerRegistry, Runner, ScriptedDom, WindowDom,
};

type Child = Box<dyn AnyView<(), (), GenetCtx, GenetElement>>;
type Logic = fn(&()) -> Child;

fn view(_: &()) -> Child {
    Box::new(el(
        "main",
        (
            button("Before", |_: &mut (), _| {}),
            button("After", |_: &mut (), _| {}),
        ),
    ))
}

#[derive(Debug, PartialEq)]
struct FocusSample {
    reported: Option<u64>,
    projected: Vec<u64>,
}

struct Recording(Rc<RefCell<Vec<FocusSample>>>);

impl Accessibility for Recording {
    fn sync(
        &mut self,
        dom: &WindowDom<'_>,
        layout: &OwnedLayout,
        _: &mut sprigging::LeafRegistry<u64>,
        _: &mut ProducerRegistry,
        focus: Option<u64>,
        _: f64,
    ) -> Vec<A11yRequest> {
        let projection = crate::document_projection(dom, layout, focus);
        self.0.borrow_mut().push(FocusSample {
            reported: focus,
            projected: projection
                .nodes()
                .iter()
                .filter(|node| node.state.focused)
                .map(|node| node.id.get())
                .collect(),
        });
        Vec::new()
    }
}

#[test]
fn accessibility_sync_tracks_focus_changes_and_clear_without_hover() {
    let samples = Rc::new(RefCell::new(Vec::new()));
    let mut state: HostState<(), Logic, Child> = HostState::new();
    state.shared.sheet =
        "main { display: block; } button { display: block; width: 100px; height: 30px; }".into();
    state.runner = Some(Runner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        view as Logic,
        (),
    ));
    state.a11y = Some(Box::new(Recording(samples.clone())));
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host = Host::new(
        HostOptions::default(),
        None,
        HostHooks::inert(),
        state,
        wake,
    );
    host.relayout(320.0, 200.0);
    let buttons = host.s.runner.as_ref().unwrap().focusables();
    assert_eq!(buttons.len(), 2);
    let ids = {
        let runner = host.s.runner.as_ref().unwrap();
        let dom = runner.dom();
        let dom = dom.borrow();
        let window = WindowDom::new(&dom, runner.mount());
        buttons
            .iter()
            .map(|&node| window.opaque_id(node))
            .collect::<Vec<_>>()
    };

    // Prime the restyle cache, then change and clear focus as a scenario or
    // programmatic request can, with no hover event to refresh that cache.
    host.s.runner.as_mut().unwrap().set_focus(Some(buttons[0]));
    host.hover();
    assert_eq!(host.s.last_focus, Some(ids[0]));
    host.sync_a11y();
    host.s.runner.as_mut().unwrap().set_focus(Some(buttons[1]));
    host.sync_a11y();
    assert_eq!(
        host.s.last_focus,
        Some(ids[0]),
        "sync leaves restyling pending"
    );
    host.s.runner.as_mut().unwrap().set_focus(None);
    host.sync_a11y();
    assert_eq!(
        host.s.last_focus,
        Some(ids[0]),
        "clear also leaves the cache alone"
    );

    assert_eq!(
        *samples.borrow(),
        vec![
            FocusSample {
                reported: Some(ids[0]),
                projected: vec![ids[0]]
            },
            FocusSample {
                reported: Some(ids[1]),
                projected: vec![ids[1]]
            },
            FocusSample {
                reported: None,
                projected: Vec::new()
            },
        ]
    );
}
