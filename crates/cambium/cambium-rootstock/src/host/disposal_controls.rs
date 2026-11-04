// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::ScriptedDom;
use cambium::{AnyView, GenetCtx, GenetElement, el, text};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
struct App {
    draft: String,
}
type Child = Box<dyn AnyView<App, (), GenetCtx, GenetElement>>;
type Logic = fn(&App) -> Child;
fn root(app: &App) -> Child {
    Box::new(el("div", text(app.draft.clone())))
}
struct RetainedProducer {
    suspensions: Rc<Cell<usize>>,
}
impl crate::TextureProducer for RetainedProducer {
    fn render(&mut self, _: &crate::ProducerContext<'_>) -> Option<crate::ProducedTexture> {
        None
    }
    fn suspend(&mut self) {
        self.suspensions.set(self.suspensions.get() + 1);
    }
}
#[test]
fn disposed_actual_host_releases_retained_producer_and_refuses_captures_outside_borrow_once() {
    let mut state: HostState<App, Logic, Child> = HostState::new();
    state.runner = Some(Runner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        root as Logic,
        App {
            draft: "uncommitted application authority".into(),
        },
    ));
    // CPU-only ownership metadata: this asserts transfer rather than
    // pretending these IDs were registered in an actual renderer.
    state.shared.leaf_fragments.insert(7, (101, 9));
    state.shared.leaf_fragments.insert(8, (102, 4));
    let suspensions = Rc::new(Cell::new(0));
    let retained = Rc::new(RefCell::new(RetainedProducer {
        suspensions: suspensions.clone(),
    }));
    state
        .shared
        .producers
        .register(7, retained.clone(), &[])
        .unwrap();
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let host = Rc::new(RefCell::new(Host::new(
        HostOptions::default(),
        None,
        HostHooks::inert(),
        state,
        wake,
    )));
    let results = Rc::new(RefCell::new(Vec::new()));
    fn check(
        host: &Rc<RefCell<Host<App, Logic, Child>>>,
        results: &Rc<RefCell<Vec<u8>>>,
        id: u8,
        refusal: CaptureRefusal,
    ) {
        assert!(matches!(refusal, CaptureRefusal::HostDisposed));
        // Actual callback can inspect authority; no Host RefMut is live.
        assert_eq!(
            host.borrow().s.runner.as_ref().unwrap().state().draft,
            "uncommitted application authority"
        );
        results.borrow_mut().push(id);
    }
    let weak = Rc::downgrade(&host);
    let out = results.clone();
    host.borrow_mut().s.pending_capture = Some(Box::new(move |result| {
        check(&weak.upgrade().unwrap(), &out, 1, result.err().unwrap())
    }));
    let weak = Rc::downgrade(&host);
    let out = results.clone();
    host.borrow_mut().s.pending_stamped_capture = Some(Box::new(move |result| {
        check(&weak.upgrade().unwrap(), &out, 2, result.err().unwrap())
    }));
    let weak = Rc::downgrade(&host);
    let out = results.clone();
    host.borrow_mut().s.pending_paint_capture = Some(Box::new(move |result| {
        check(&weak.upgrade().unwrap(), &out, 3, result.err().unwrap())
    }));
    let disposal = { host.borrow_mut().take_disposal() };
    assert_eq!(suspensions.get(), 0);
    assert!(host.borrow().s.shared.leaf_fragments.is_empty());
    assert_eq!(disposal.fragments.len(), 2);
    assert_eq!(disposal.fragments[&7], (101, 9));
    assert_eq!(disposal.fragments[&8], (102, 4));
    assert!(!host.borrow().s.shared.producers.contains(7));
    drop(disposal);
    assert_eq!(
        suspensions.get(),
        1,
        "default producer retirement suspends the externally retained instance"
    );
    assert_eq!(*results.borrow(), [1, 2, 3]);
    let repeated = { host.borrow_mut().take_disposal() };
    assert!(repeated.fragments.is_empty());
    drop(repeated);
    assert_eq!(suspensions.get(), 1);
    assert_eq!(*results.borrow(), [1, 2, 3]);
    assert_eq!(Rc::strong_count(&retained), 1);
}
