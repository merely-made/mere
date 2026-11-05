// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! CPU admission policy with a controlled prior-picture proof; no OS adapter.

use super::*;
use crate::{A11yAction, A11yDispatch as D, A11yRejection as R, HostWindow};
use std::cell::Cell;

#[derive(Default)]
struct Counts {
    publications: Cell<usize>,
    bootstraps: Cell<usize>,
    redraws: Cell<usize>,
}
struct Adapter {
    counts: Rc<Counts>,
    pending: Rc<RefCell<Vec<A11yRequest>>>,
}
impl Accessibility for Adapter {
    fn prepare_native_window(&mut self) {
        self.counts.bootstraps.set(self.counts.bootstraps.get() + 1);
    }
    fn adapter_state(&self) -> crate::A11yAdapterState {
        crate::A11yAdapterState::Installed
    }
    fn drain_actions(&mut self, _: &crate::A11yOwner) -> Option<Vec<A11yRequest>> {
        Some(self.pending.take())
    }
    fn sync(
        &mut self,
        _: &crate::A11yOwner,
        dom: &WindowDom<'_>,
        layout: &OwnedLayout,
        leaves: &sprigging::LeafRegistry<u64>,
        producers: &ProducerRegistry,
        focus: Option<u64>,
        _: f64,
    ) -> crate::A11ySync {
        let projection = crate::project_semantics(dom, layout, leaves, producers, focus);
        self.counts
            .publications
            .set(self.counts.publications.get() + 1);
        crate::A11ySync {
            requests: self.pending.take(),
            publication: crate::A11yPublication {
                adapter: self.adapter_state(),
                outcome: crate::A11yPublicationOutcome::Submitted {
                    nodes: projection.tree.nodes.len(),
                    contribution_issues: 0,
                },
            },
        }
    }
}
struct Legacy(Adapter);
impl Accessibility for Legacy {
    fn adapter_state(&self) -> crate::A11yAdapterState {
        self.0.adapter_state()
    }
    fn sync(
        &mut self,
        owner: &crate::A11yOwner,
        dom: &WindowDom<'_>,
        layout: &OwnedLayout,
        leaves: &sprigging::LeafRegistry<u64>,
        producers: &ProducerRegistry,
        focus: Option<u64>,
        scale: f64,
    ) -> crate::A11ySync {
        self.0
            .sync(owner, dom, layout, leaves, producers, focus, scale)
    }
}
struct Window(Rc<Counts>);
impl HostWindow for Window {
    fn request_redraw(&self) {
        self.0.redraws.set(self.0.redraws.get() + 1);
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
type TestHost = Host<u32, CounterLogic, CounterChild>;
type Queue = Rc<RefCell<Vec<A11yRequest>>>;

fn fixture(legacy: bool) -> (TestHost, Rc<Counts>, Queue) {
    let mut host = counter_host();
    let counts = Rc::new(Counts::default());
    let pending = Rc::new(RefCell::new(Vec::new()));
    let adapter = Adapter {
        counts: counts.clone(),
        pending: pending.clone(),
    };
    host.s.a11y = Some(if legacy {
        Box::new(Legacy(adapter))
    } else {
        Box::new(adapter)
    });
    host.s.window = Some(Box::new(Window(counts.clone())));
    host.s.native_receiving = true;
    let input = {
        let runner = host.s.runner.as_ref().unwrap();
        let dom = runner.dom();
        let dom = dom.borrow();
        Rc::new(
            host.s
                .layout
                .as_ref()
                .unwrap()
                .presented_input(&WindowDom::new(&dom, runner.mount())),
        )
    };
    host.s.displayed_input = Some(input);
    host.s.displayed_presentation = Some(crate::PresentedFrame {
        host: host.s.presentation_host,
        sequence: 1,
        width: 320,
        height: 240,
        layout_scale: 1.,
        context_generation: crate::ContextGeneration(0),
    });
    host.s.last_redraw_presentation = host.s.displayed_presentation;
    host.sync_a11y();
    assert_eq!(counts.publications.get(), 1);
    host.s.last_redraw_presentation = None;
    host.s.native_dirty = false;
    (host, counts, pending)
}
fn click(host: &TestHost) -> A11yRequest {
    let runner = host.s.runner.as_ref().unwrap();
    let dom = runner.dom();
    A11yRequest::node(A11yAction::Click, runner.focusables()[0]).bind_publication(
        &crate::A11yOwner::new(&host.s.a11y_owner),
        &WindowDom::new(&dom.borrow(), runner.mount()),
        &host.s.shared.leaves,
        &host.s.shared.producers,
    )
}

#[test]
fn repeated_native_submission_tails_drain_without_duplicate_projection() {
    let (mut host, counts, _) = fixture(false);
    host.s.last_frame_profile = Some(crate::FrameProfile::default());
    for _ in 0..3 {
        assert!(host.sync_a11y().is_empty());
    }
    assert_eq!(counts.publications.get(), 1);
    assert_eq!(host.s.last_frame_profile.unwrap().a11y_publish_us, 0);
    assert_eq!(counts.redraws.get(), 0);
    assert_eq!(
        host.a11y_publication().publication.outcome,
        crate::A11yPublicationOutcome::NotAttempted {
            reason: crate::A11yPublicationSkip::PendingPresentation
        }
    );
}

#[test]
fn an_accepted_tree_action_requests_model_demand_without_republication() {
    let (mut host, counts, pending) = fixture(false);
    pending.borrow_mut().push(click(&host));
    assert_eq!(host.drain_a11y_actions(), [D::Dispatched]);
    assert_eq!(*host.s.runner.as_ref().unwrap().state(), 1);
    assert!(host.s.native_dirty);
    assert_eq!(counts.redraws.get(), 1);
    assert_eq!(counts.publications.get(), 1);
    assert!(host.drain_a11y_actions().is_empty());
    assert_eq!(counts.redraws.get(), 1, "empty drains create no new demand");
}

#[test]
fn draining_revalidates_each_action_after_the_previous_action_disables_it() {
    let (mut host, counts, pending) = fixture(false);
    let request = click(&host);
    pending.borrow_mut().extend([request.clone(), request]);
    assert_eq!(
        host.drain_a11y_actions(),
        [D::Dispatched, D::Rejected(R::Disabled)]
    );
    assert_eq!(*host.s.runner.as_ref().unwrap().state(), 1);
    assert_eq!(counts.publications.get(), 1);
    assert!(pending.borrow().is_empty());
}

#[test]
fn old_actions_do_not_follow_a_replacement_document_during_pending_publication() {
    let (mut host, counts, pending) = fixture(false);
    pending.borrow_mut().push(click(&host));
    host.s.runner = counter_host().s.runner.take();
    assert_eq!(host.drain_a11y_actions(), [D::Rejected(R::StaleTarget)]);
    assert_eq!(*host.s.runner.as_ref().unwrap().state(), 0);
    assert_eq!(counts.publications.get(), 1);
    assert_eq!(counts.redraws.get(), 0);
}

#[test]
fn hidden_closing_and_unavailable_contexts_consume_gestures_without_later_replay() {
    for unavailable in 0..4 {
        let (mut host, counts, pending) = fixture(false);
        pending.borrow_mut().push(click(&host));
        match unavailable {
            0 => host.s.hidden = true,
            1 => host.s.close_requested = true,
            2 => host.s.displayed_input = None,
            _ => host.s.needs_recovery_surface = true,
        }
        assert_eq!(host.drain_a11y_actions(), [D::Rejected(R::Unavailable)]);
        assert!(pending.borrow().is_empty());
        host.s.hidden = false;
        host.s.close_requested = false;
        host.s.needs_recovery_surface = false;
        assert!(host.drain_a11y_actions().is_empty());
        assert_eq!(*host.s.runner.as_ref().unwrap().state(), 0);
        assert_eq!(counts.publications.get(), 1);
        assert_eq!(counts.redraws.get(), 0);
    }
}

#[test]
fn revoked_owner_consumes_the_queue_and_never_dispatches() {
    let (mut host, counts, pending) = fixture(false);
    pending.borrow_mut().push(click(&host));
    host.s.a11y_owner.set(false);
    assert_eq!(host.drain_a11y_actions(), [D::Rejected(R::Cancelled)]);
    assert!(pending.borrow().is_empty());
    assert_eq!(counts.publications.get(), 1);
    assert_eq!(counts.redraws.get(), 0);
}

#[test]
fn adapters_without_the_independent_drain_keep_their_combined_action_service() {
    let (mut host, counts, pending) = fixture(true);
    pending.borrow_mut().push(click(&host));
    assert_eq!(host.drain_a11y_actions(), [D::Dispatched]);
    assert_eq!(counts.publications.get(), 2, "legacy fallback is explicit");
    assert_eq!(*host.s.runner.as_ref().unwrap().state(), 1);
}

#[test]
fn native_adapter_wakes_the_event_source_without_fabricating_application_work() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let (mut host, _, _) = fixture(false);
    let notifications = Arc::new(AtomicUsize::new(0));
    let signalled = notifications.clone();
    host.wake = HostWake::new(
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        Arc::new(move || {
            signalled.fetch_add(1, Ordering::SeqCst);
        }),
    );
    let notify: Box<dyn Fn() + Send + Sync> = Box::new(host.a11y_waker());
    notify();
    notify();
    assert_eq!(notifications.load(Ordering::SeqCst), 1);
    assert!(!host.process_application_wake());
    assert!(matches!(
        host.idle_policy(crate::Instant::now()),
        crate::IdlePolicy::A11yWake
    ));
    notify();
    assert_eq!(
        notifications.load(Ordering::SeqCst),
        2,
        "draining rearms notification"
    );
    assert!(!host.process_application_wake());
}
