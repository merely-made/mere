// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! CPU notification policy; no pending picture or physical publication implied.

use super::*;
use std::sync::atomic::Ordering;

struct CountingWindow(Rc<Cell<usize>>);
impl HostWindow for CountingWindow {
    fn request_redraw(&self) {
        self.0.set(self.0.get() + 1);
    }
    fn inner_size(&self) -> (u32, u32) {
        (640, 480)
    }
    fn scale_factor(&self) -> f64 {
        2.
    }
    fn set_ime_allowed(&self, _: bool) {}
    fn set_ime_cursor_area(&self, _: f64, _: f64, _: f64, _: f64) {}
}
fn counted() -> (Fixture, Rc<Cell<usize>>, Rc<Cell<usize>>) {
    let mut f = fixture();
    let requests = Rc::new(Cell::new(0));
    let application = Rc::new(Cell::new(0));
    f.host.s.window = Some(Box::new(CountingWindow(requests.clone())));
    let called = application.clone();
    f.host.hooks.after_wake = Box::new(move |_| called.set(called.get() + 1));
    f.host.wake.take_pending();
    f.host
        .s
        .native_completion_pending
        .store(false, Ordering::Release);
    f.host.s.native_dirty = false;
    (f, requests, application)
}

#[test]
fn application_drain_leaves_native_notice_for_its_owner_and_runs_once() {
    let (mut f, requests, application) = counted();
    f.host
        .s
        .native_completion_pending
        .store(true, Ordering::Release);
    f.host.wake.wake();
    assert!(f.host.process_application_wake());
    assert_eq!(application.get(), 1);
    assert_eq!(requests.get(), 1);
    assert!(f.host.s.native_dirty);
    assert!(f.host.s.native_completion_pending.load(Ordering::Acquire));
    assert!(!f.host.collect_presentation_wake());
    assert!(!f.host.s.native_completion_pending.load(Ordering::Acquire));
    assert!(!f.host.process_application_wake());
    assert_eq!(application.get(), 1);
    assert_eq!(requests.get(), 1);
}

#[test]
fn orphan_completion_and_empty_coalescing_create_no_frame_demand_or_app_hook() {
    let (mut f, requests, application) = counted();
    f.host
        .s
        .native_completion_pending
        .store(true, Ordering::Release);
    assert!(!f.host.collect_presentation_wake());
    assert!(!f.host.coalesce_presentation_redraw());
    assert!(!f.host.s.native_dirty);
    assert_eq!(requests.get(), 0);
    assert_eq!(application.get(), 0);
    f.host
        .s
        .native_completion_pending
        .store(true, Ordering::Release);
    assert!(
        !f.host.process_wake(),
        "deferred fallback consumes an orphan without scheduling"
    );
    assert!(!f.host.s.native_completion_pending.load(Ordering::Acquire));
    assert_eq!(requests.get(), 0);
    f.host.wake.wake();
    assert!(
        !f.host.collect_presentation_wake(),
        "collector does not steal application work"
    );
    assert!(f.host.process_wake());
    assert_eq!(requests.get(), 1);
    assert_eq!(application.get(), 1);
    assert!(!f.host.process_wake());
}
