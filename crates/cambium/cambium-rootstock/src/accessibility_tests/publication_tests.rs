// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{A11yDispatch as D, A11yRejection as R, ProducerChildLifetime, TextureProducer};
use std::cell::Cell;

fn published(host: &Host<u32, CounterLogic, CounterChild>, request: A11yRequest) -> A11yRequest {
    let runner = host.s.runner.as_ref().unwrap();
    let dom = runner.dom();
    request.bind_publication(
        &crate::A11yOwner::new(&host.s.a11y_owner),
        &WindowDom::new(&dom.borrow(), runner.mount()),
        &host.s.shared.leaves,
        &host.s.shared.producers,
    )
}

struct ChildBoard {
    life: Rc<RefCell<Option<ProducerChildLifetime>>>,
    calls: Rc<Cell<u32>>,
    replace_on_read: Rc<Cell<bool>>,
}
impl TextureProducer for ChildBoard {
    fn render(&mut self, _: &crate::ProducerContext<'_>) -> Option<crate::ProducedTexture> {
        None
    }
    fn child_lifetime(&self, key: u64) -> Option<crate::ProducerChildToken> {
        (key == 40)
            .then(|| {
                self.life
                    .borrow()
                    .as_ref()
                    .map(ProducerChildLifetime::token)
            })
            .flatten()
    }
    fn semantics(&self) -> Option<crate::ProducerSemantics> {
        if self.replace_on_read.replace(false) {
            *self.life.borrow_mut() = Some(ProducerChildLifetime::new());
        }
        self.life.borrow().as_ref()?;
        LifetimeBoard {
            calls: self.calls.clone(),
            revision: Rc::new(Cell::new(0)),
        }
        .semantics()
    }
    fn act(&mut self, key: u64, id: &str) -> bool {
        assert_eq!((key, id), (40, "pin"));
        self.calls.set(self.calls.get() + 1);
        true
    }
}

#[test]
fn producer_child_reuse_refuses_published_and_queued_actions_even_with_live_old_marker() {
    let mut host = counter_host();
    let life = Rc::new(RefCell::new(Some(ProducerChildLifetime::new())));
    let calls = Rc::new(Cell::new(0));
    let replace_on_read = Rc::new(Cell::new(false));
    host.s
        .shared
        .producers
        .register(
            7,
            ChildBoard {
                life: life.clone(),
                calls: calls.clone(),
                replace_on_read: replace_on_read.clone(),
            },
            &[],
        )
        .unwrap();
    let stale = published(&host, lifetime_produced());
    let queued = defer_lifetime_request(&mut host, lifetime_produced());
    let old = life.borrow_mut().take();
    *life.borrow_mut() = Some(ProducerChildLifetime::new());
    assert_eq!(
        host.apply_a11y_requests(&[stale.clone()]),
        [D::Rejected(R::StaleTarget)]
    );
    host.drain_pointer();
    assert_eq!(queued.result(), Some(D::Rejected(R::StaleTarget)));
    assert_eq!(calls.get(), 0);
    // Re-enqueueing an old publication must never recapture current authority.
    let queued = defer_lifetime_request(&mut host, stale);
    host.drain_pointer();
    assert_eq!(queued.result(), Some(D::Rejected(R::StaleTarget)));
    drop(old);

    let current = published(&host, lifetime_produced());
    host.relayout(320., 240.);
    assert!(current.same_publication_target(&published(&host, lifetime_produced())));
    assert_eq!(host.apply_a11y_requests(&[current]), [D::Dispatched]);
    assert_eq!(calls.get(), 1);

    let current = published(&host, lifetime_produced());
    replace_on_read.set(true);
    assert_eq!(
        host.apply_a11y_requests(&[current]),
        [D::Rejected(R::StaleTarget)]
    );
    assert_eq!(
        calls.get(),
        1,
        "an impure semantic callback cannot authorize a replacement"
    );
}

#[test]
fn publication_binding_cannot_follow_replaced_producers_documents_or_edited_targets() {
    let mut host = counter_host();
    let calls = Rc::new(Cell::new(0));
    let board = || LifetimeBoard {
        calls: calls.clone(),
        revision: Rc::new(Cell::new(0)),
    };
    host.s.shared.producers.register(7, board(), &[]).unwrap();
    let old = published(&host, lifetime_produced());
    host.s.shared.producers.remove(7);
    host.s.shared.producers.register(7, board(), &[]).unwrap();
    assert_eq!(
        host.apply_a11y_requests(&[old]),
        [D::Rejected(R::StaleTarget)]
    );
    let mut changed = published(&host, lifetime_produced());
    changed.target = lifetime_click(&host).target;
    assert_eq!(
        host.apply_a11y_requests(&[changed]),
        [D::Rejected(R::StaleTarget)]
    );

    let old = published(&host, lifetime_produced());
    let mut other = counter_host();
    std::mem::swap(&mut host.s.runner, &mut other.s.runner);
    assert_eq!(
        host.apply_a11y_requests(&[old]),
        [D::Rejected(R::StaleTarget)]
    );
    assert_eq!(calls.get(), 0);
}

#[test]
fn publication_binding_refuses_leaf_replacement_and_retains_mutable_updates() {
    let mut host = counter_host();
    let knob = || {
        Box::new(sprigging::Knob::new(sprigging::Size {
            width: 80.,
            height: 30.,
        }))
    };
    host.s.shared.leaves.insert(7, knob());
    let runner = host.s.runner.as_ref().unwrap();
    let dom = runner.dom();
    let slot = host
        .s
        .layout
        .as_ref()
        .unwrap()
        .custom_leaf_nodes(&WindowDom::new(&dom.borrow(), runner.mount()))[0]
        .1;
    let request = A11yRequest::node(crate::A11yAction::SetValue(4.), slot);
    let old = published(&host, request.clone());
    host.s.shared.leaves.insert(7, knob());
    assert_eq!(
        host.apply_a11y_requests(&[old.clone()]),
        [D::Rejected(R::StaleTarget)]
    );
    let current = published(&host, request.clone());
    assert!(!old.same_publication_target(&current));
    host.s
        .shared
        .leaves
        .get_mut_as::<sprigging::Knob>(&7)
        .unwrap()
        .set_value(2.);
    assert!(current.same_publication_target(&published(&host, request)));
    assert_eq!(host.apply_a11y_requests(&[current]), [D::Dispatched]);
}

#[test]
fn publication_owner_cannot_follow_identical_document_and_mount_to_another_host() {
    let mut first = counter_host();
    let node = first.s.runner.as_ref().unwrap().focusables()[0];
    let raw = A11yRequest::node(crate::A11yAction::Click, node);
    let old = published(&first, raw.clone());
    let mut second = counter_host();
    // Transfer the actual runner/document/mount and layout, so only host owner
    // changes. Contributor or document rejection cannot mask the owner check.
    second.s.runner = first.s.runner.take();
    second.s.layout = first.s.layout.take();
    assert!(first.s.a11y_owner.get(), "old owner still lives");
    assert_eq!(
        second.apply_a11y_requests(&[old.clone()]),
        [D::Rejected(R::StaleTarget)]
    );
    let queued = defer_lifetime_request(&mut second, old.clone());
    second.drain_pointer();
    assert_eq!(queued.result(), Some(D::Rejected(R::StaleTarget)));
    assert_eq!(*second.s.runner.as_ref().unwrap().state(), 0);
    let current = published(&second, raw);
    assert!(!old.same_publication_target(&current));
    assert_eq!(second.apply_a11y_requests(&[current]), [D::Dispatched]);
    assert_eq!(*second.s.runner.as_ref().unwrap().state(), 1);
}

#[test]
fn disposed_host_refuses_direct_bound_and_raw_actions_and_stops_publication() {
    let mut host = counter_host();
    let samples = Rc::new(RefCell::new(Vec::new()));
    host.s.a11y = Some(Box::new(Recording(samples.clone())));
    host.sync_a11y();
    let node = host.s.runner.as_ref().unwrap().focusables()[0];
    let raw = A11yRequest::node(crate::A11yAction::Click, node);
    let bound = published(&host, raw.clone());
    let guard = host.take_disposal();
    assert!(
        host.s.runner.is_some() && host.s.layout.is_some(),
        "retained CPU state is still present"
    );
    assert_eq!(
        host.apply_a11y_requests(&[raw, bound]),
        [D::Rejected(R::Cancelled), D::Rejected(R::Cancelled)]
    );
    assert_eq!(*host.s.runner.as_ref().unwrap().state(), 0);
    assert!(host.sync_a11y().is_empty());
    assert_eq!(samples.borrow().len(), 1, "disposed host cannot republish");
    drop(guard);
}

#[test]
fn publication_owner_is_weak_and_disposal_ends_it_even_with_a_retained_marker() {
    let mut host = counter_host();
    let marker = host.s.a11y_owner.clone();
    let owner = crate::A11yOwner::new(&marker);
    let strong = Rc::strong_count(&marker);
    let node = host.s.runner.as_ref().unwrap().focusables()[0];
    let request = published(&host, A11yRequest::node(crate::A11yAction::Focus, node));
    assert_eq!(Rc::strong_count(&marker), strong);
    assert!(owner.is_live());
    assert!(request.same_publication_target(&published(&host, request.clone())));
    let guard = host.take_disposal();
    assert!(!owner.is_live());
    drop(host);
    drop(guard);
    assert!(!owner.is_live());
    assert_eq!(Rc::strong_count(&marker), 1);
    drop(request);
    drop(marker);
    assert!(!owner.is_live());
}
