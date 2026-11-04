// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn lifetime_tests_leaf_replacement_refuses_queued_values_but_updates_keep_the_life() {
    use crate::{A11yAction, A11yDispatch as D, A11yRejection as R};
    for replacement in 0..4 {
        let mut host = counter_host();
        let knob = || {
            Box::new(sprigging::Knob::new(sprigging::Size {
                width: 80.0,
                height: 30.0,
            }))
        };
        if replacement != 3 {
            host.s.shared.leaves.insert(7, knob());
        }
        let runner = host.s.runner.as_ref().unwrap();
        let dom = runner.dom();
        let slot = host
            .s
            .layout
            .as_ref()
            .unwrap()
            .custom_leaf_nodes(&WindowDom::new(&dom.borrow(), runner.mount()))[0]
            .1;
        let request = A11yRequest::node(A11yAction::SetValue(4.0), slot);
        let stale = defer_lifetime_request(&mut host, request.clone());
        let detached = match replacement {
            1 => {
                host.s.shared.leaves.remove(&7);
                None
            },
            2 => Some(std::mem::take(&mut host.s.shared.leaves)),
            _ => None,
        };
        host.s.shared.leaves.insert(7, knob());
        host.drain_pointer();
        assert_eq!(stale.result(), Some(D::Rejected(R::StaleTarget)));
        assert_eq!(*host.s.runner.as_ref().unwrap().state(), 0);
        drop(detached);

        let current = defer_lifetime_request(&mut host, request);
        host.s
            .shared
            .leaves
            .get_mut_as::<sprigging::Knob>(&7)
            .unwrap()
            .set_value(2.0);
        host.relayout(320.0, 240.0);
        host.drain_pointer();
        assert_eq!(current.result(), Some(D::Dispatched));
        assert_eq!(*host.s.runner.as_ref().unwrap().state(), 4);
    }
}

#[test]
fn lifetime_tests_queued_document_and_host_replacement_refuse_foreign_owners() {
    use crate::{A11yDispatch as D, A11yRejection as R};
    for replace_document in [false, true] {
        let mut origin = counter_host();
        let mut replacement = counter_host();
        // Supplier DOM IDs already carry distinct arena namespaces. Generated
        // actions, however, use local slot/child keys that can recur in either
        // document. Keep the producer registration live while replacing DOM.
        assert_ne!(lifetime_click(&origin), lifetime_click(&replacement));
        let calls = Rc::new(std::cell::Cell::new(0));
        let request = if replace_document {
            origin
                .s
                .shared
                .producers
                .register(
                    7,
                    LifetimeBoard {
                        calls: calls.clone(),
                        revision: Rc::new(std::cell::Cell::new(0)),
                    },
                    &[],
                )
                .unwrap();
            lifetime_produced()
        } else {
            lifetime_click(&origin)
        };
        let receipt = defer_lifetime_request(&mut origin, request);
        assert_eq!(receipt.result(), None);
        if replace_document {
            // Keep the old document and registration live. A liveness-only
            // weak check would accept the same local slot/child in the new DOM.
            std::mem::swap(&mut origin.s.runner, &mut replacement.s.runner);
            origin.drain_pointer();
            assert_eq!(*origin.s.runner.as_ref().unwrap().state(), 0);
        } else {
            // Even moving the exact runner/document into another host cannot
            // move the originating host authority with the queued request.
            replacement.s.runner = origin.s.runner.take();
            replacement
                .s
                .pending_a11y
                .append(&mut origin.s.pending_a11y);
            replacement.drain_pointer();
            assert_eq!(*replacement.s.runner.as_ref().unwrap().state(), 0);
        }
        assert_eq!(receipt.result(), Some(D::Rejected(R::StaleTarget)));
        assert_eq!(calls.get(), 0);
    }
}

#[test]
fn lifetime_tests_reused_producer_registration_refuses_old_queue_and_accepts_current_updates() {
    use crate::{A11yDispatch as D, A11yRejection as R};
    for detach_registry in [false, true] {
        let mut host = counter_host();
        let calls = Rc::new(std::cell::Cell::new(0));
        let revision = Rc::new(std::cell::Cell::new(0));
        let board = || LifetimeBoard {
            calls: calls.clone(),
            revision: revision.clone(),
        };
        host.s.shared.producers.register(7, board(), &[]).unwrap();
        let stale = defer_lifetime_request(&mut host, lifetime_produced());
        // Both removal/re-registration and whole-registry replacement must reject,
        // including while the detached old registry remains alive.
        let old_registry = if detach_registry {
            Some(std::mem::take(&mut host.s.shared.producers))
        } else {
            assert!(host.s.shared.producers.remove(7));
            None
        };
        host.s.shared.producers.register(7, board(), &[]).unwrap();
        host.drain_pointer();
        assert_eq!(stale.result(), Some(D::Rejected(R::StaleTarget)));
        assert_eq!(calls.get(), 0, "the replacement was not activated");
        drop(old_registry);

        let current = defer_lifetime_request(&mut host, lifetime_produced());
        revision.set(1);
        // Ordinary re-publication/layout cannot manufacture or revoke a life.
        host.relayout(320.0, 240.0);
        host.drain_pointer();
        assert_eq!(current.result(), Some(D::Dispatched));
        assert_eq!(calls.get(), 1);
        drop(host);
        assert_eq!(
            current.result(),
            Some(D::Dispatched),
            "disposal preserves the result"
        );
    }
}

#[test]
fn lifetime_tests_registration_added_after_queue_cannot_adopt_an_old_request() {
    let mut host = counter_host();
    let receipt = defer_lifetime_request(&mut host, lifetime_produced());
    let calls = Rc::new(std::cell::Cell::new(0));
    host.s
        .shared
        .producers
        .register(
            7,
            LifetimeBoard {
                calls: calls.clone(),
                revision: Rc::new(std::cell::Cell::new(0)),
            },
            &[],
        )
        .unwrap();
    host.drain_pointer();
    assert_eq!(
        receipt.result(),
        Some(crate::A11yDispatch::Rejected(
            crate::A11yRejection::StaleTarget
        ))
    );
    assert_eq!(calls.get(), 0);
}

#[test]
fn lifetime_tests_disposal_and_drop_finish_pending_acknowledgements() {
    use crate::{A11yDispatch as D, A11yRejection as R};
    for explicit_disposal in [false, true] {
        let mut host = counter_host();
        let request = lifetime_click(&host);
        let dom = Rc::downgrade(&host.s.runner.as_ref().unwrap().dom());
        let receipt = defer_lifetime_request(&mut host, request.clone());
        if explicit_disposal {
            drop(host.take_disposal());
            assert_eq!(receipt.result(), Some(D::Rejected(R::Cancelled)));
            let later = defer_lifetime_request(&mut host, request);
            assert_eq!(later.result(), Some(D::Rejected(R::Cancelled)));
            assert_eq!(*host.s.runner.as_ref().unwrap().state(), 0);
        }
        drop(host);
        assert!(
            dom.upgrade().is_none(),
            "the acknowledgement retains no document"
        );
        assert_eq!(receipt.result(), Some(D::Rejected(R::Cancelled)));
    }
}

#[test]
fn lifetime_tests_queue_binding_distinguishes_mounts_in_one_live_document() {
    let mut host = counter_host();
    let request = lifetime_click(&host);
    let receipt = defer_lifetime_request(&mut host, request);
    let runner = host.s.runner.as_ref().unwrap();
    let dom = runner.dom();
    let mount = runner.mount();
    let another_mount = runner.focusables()[0];
    assert_ne!(mount, another_mount);
    let queued = &host.s.pending_a11y[0];
    assert!(queued.is_current(
        &host.s.a11y_owner,
        &dom,
        mount,
        &host.s.shared.producers,
        &host.s.shared.leaves
    ));
    assert!(!queued.is_current(
        &host.s.a11y_owner,
        &dom,
        another_mount,
        &host.s.shared.producers,
        &host.s.shared.leaves
    ));
    host.drain_pointer();
    assert_eq!(receipt.result(), Some(crate::A11yDispatch::Dispatched));
}
