// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Actual event-source wake routing; CPU projection, no native window/GPU/AT.

use super::*;
use cambium::{AnyView, GenetCtx, GenetElement, button};
use cambium_rootstock::{
    A11yAction, A11yRequest, Accessibility, OwnedLayout, ProducerRegistry, WindowDom,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

type Child = Box<dyn AnyView<u32, (), GenetCtx, GenetElement>>;
type Logic = fn(&u32) -> Child;
type TestHost = WinitHost<u32, Logic, Child>;

fn root(_: &u32) -> Child {
    Box::new(
        button("Count", |value: &mut u32, _| *value += 1)
            .attr("style", "display:block;width:80px;height:30px;"),
    )
}
#[derive(Default)]
struct Counts {
    publish: Cell<usize>,
    frame: Cell<usize>,
    after_frame: Cell<usize>,
    application: Cell<usize>,
    redraw: Cell<usize>,
}
struct Window(Rc<Counts>);
impl HostWindow for Window {
    fn request_redraw(&self) {
        self.0.redraw.set(self.0.redraw.get() + 1);
    }
    fn inner_size(&self) -> (u32, u32) {
        (320, 240)
    }
    fn scale_factor(&self) -> f64 {
        1.
    }
    fn set_ime_allowed(&self, _: bool) {}
    fn set_ime_cursor_area(&self, _: f64, _: f64, _: f64, _: f64) {}
}
struct Adapter {
    counts: Rc<Counts>,
    queue: Rc<RefCell<Vec<A11yRequest>>>,
    published: Rc<RefCell<Option<A11yRequest>>>,
}
impl Accessibility for Adapter {
    fn adapter_state(&self) -> cambium_rootstock::A11yAdapterState {
        cambium_rootstock::A11yAdapterState::Installed
    }
    fn drain_actions(&mut self, _: &cambium_rootstock::A11yOwner) -> Option<Vec<A11yRequest>> {
        Some(self.queue.take())
    }
    fn sync(
        &mut self,
        owner: &cambium_rootstock::A11yOwner,
        dom: &WindowDom<'_>,
        layout: &OwnedLayout,
        leaves: &cambium_rootstock::LeafRegistry<u64>,
        producers: &ProducerRegistry,
        focus: Option<u64>,
        _: f64,
    ) -> cambium_rootstock::A11ySync {
        let projection =
            cambium_rootstock::project_semantics(dom, layout, leaves, producers, focus);
        let bindings = projection
            .publication_bindings(owner, dom, leaves, producers)
            .unwrap();
        let button = projection
            .tree
            .nodes
            .iter()
            .find(|(_, node)| node.role() == accesskit::Role::Button)
            .map(|(id, _)| id.0)
            .expect("projected button");
        let mut request = bindings
            .requests
            .get(&button)
            .cloned()
            .expect("button binding");
        request.action = A11yAction::Click;
        *self.published.borrow_mut() = Some(request);
        self.counts.publish.set(self.counts.publish.get() + 1);
        cambium_rootstock::A11ySync {
            requests: self.queue.take(),
            publication: cambium_rootstock::A11yPublication {
                adapter: self.adapter_state(),
                outcome: cambium_rootstock::A11yPublicationOutcome::Submitted {
                    nodes: projection.tree.nodes.len(),
                    contribution_issues: 0,
                },
            },
        }
    }
}
struct Fixture {
    host: TestHost,
    counts: Rc<Counts>,
    queue: Rc<RefCell<Vec<A11yRequest>>>,
    request: A11yRequest,
    notifications: Arc<AtomicUsize>,
}
fn fixture() -> Fixture {
    let counts = Rc::new(Counts::default());
    let queue = Rc::new(RefCell::new(Vec::new()));
    let published = Rc::new(RefCell::new(None));
    let notifications = Arc::new(AtomicUsize::new(0));
    let mut state = cambium_rootstock::HostState::<u32, Logic, Child>::new();
    state.window = Some(Box::new(Window(counts.clone())));
    state.runner = Some(Runner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        root as Logic,
        0,
    ));
    state.a11y = Some(Box::new(Adapter {
        counts: counts.clone(),
        queue: queue.clone(),
        published: published.clone(),
    }));
    let signalled = notifications.clone();
    let wake = cambium_rootstock::HostWake::new(
        state.wake_pending.clone(),
        Arc::new(move || {
            signalled.fetch_add(1, Ordering::SeqCst);
        }),
    );
    let mut hooks = cambium_rootstock::HostHooks::<u32, Logic, Child>::inert();
    let c = counts.clone();
    hooks.frame = Box::new(move |_| {
        c.frame.set(c.frame.get() + 1);
        false
    });
    let c = counts.clone();
    hooks.after_frame = Box::new(move |_| c.after_frame.set(c.after_frame.get() + 1));
    let c = counts.clone();
    hooks.after_wake = Box::new(move |_| c.application.set(c.application.get() + 1));
    let mut host = WinitHost::new(Host::new(HostOptions::default(), None, hooks, state, wake));
    host.relayout(320., 240.);
    host.sync_a11y();
    let request = published.borrow().clone().unwrap();
    Fixture {
        host,
        counts,
        queue,
        request,
        notifications,
    }
}
fn signal(f: &mut Fixture) {
    f.queue.borrow_mut().push(f.request.clone());
    let notify: Box<dyn Fn() + Send + Sync> = Box::new(f.host.a11y_waker());
    notify();
    notify();
}
fn assert_no_frame(f: &Fixture) {
    assert_eq!(f.counts.frame.get(), 0);
    assert_eq!(f.counts.after_frame.get(), 0);
    assert_eq!(f.counts.application.get(), 0);
    assert_eq!(f.counts.publish.get(), 1);
}

#[test]
fn at_user_event_dispatches_the_accepted_tree_action_before_any_frame_preparation() {
    let mut f = fixture();
    signal(&mut f);
    assert_eq!(f.notifications.load(Ordering::SeqCst), 1);
    f.host.wake_turn();
    assert_eq!(*f.host.s.runner.as_ref().unwrap().state(), 1);
    assert!(f.queue.borrow().is_empty());
    assert_eq!(
        f.counts.redraw.get(),
        1,
        "actual model change owes one picture"
    );
    assert_no_frame(&f);
    f.host.wake_turn();
    assert_eq!(
        f.counts.redraw.get(),
        1,
        "duplicate user event creates no demand"
    );
    assert_no_frame(&f);
}

#[test]
fn hidden_and_closing_user_events_consume_old_gestures_before_a_later_reveal() {
    for closing in [false, true] {
        let mut f = fixture();
        f.host.s.hidden = !closing;
        f.host.s.close_requested = closing;
        signal(&mut f);
        f.host.wake_turn();
        assert!(f.queue.borrow().is_empty());
        assert_eq!(*f.host.s.runner.as_ref().unwrap().state(), 0);
        f.host.s.hidden = false;
        f.host.s.close_requested = false;
        f.host.wake_turn();
        assert_eq!(*f.host.s.runner.as_ref().unwrap().state(), 0);
        assert_eq!(f.counts.redraw.get(), 0);
        assert_no_frame(&f);
    }
}
