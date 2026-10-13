// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Explicit mount cancellation and owned disposal. A request changes authority
//! synchronously; browser cleanup waits for a microtask, so it cannot drop the
//! JS closure currently executing or reenter that callback's host borrow.

use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MountPhase {
    New,
    Starting,
    Active,
    Disposing,
    Disposed,
}

/// Copied observations of owned Rust/browser resources. Disposed means startup
/// Rust future and registered cleanup actions have completed, not destruction
/// of a physical device or cancellation of an outstanding JS WebGPU promise.
/// Group-owned device recovery tasks keep their own finite deadline and may
/// outlive this mount. Application-owned GPU caches need on_dispose cleanup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MountStatus {
    pub phase: MountPhase,
    pub startup_pending: bool,
    pub cleanup_pending: bool,
    pub listeners: usize,
    pub raf_pending: bool,
    pub listener_removal_failures: usize,
}

struct Inner {
    status: Cell<MountStatus>,
    startup_waker: RefCell<Option<Waker>>,
    cleanup: RefCell<Vec<Box<dyn FnOnce()>>>,
}

/// Create before any startup await. Clones name the same one-use mount.
/// Dropping a handle preserves compatibility with document-lifetime mounts;
/// call dispose explicitly to cancel startup or release an activated mount.
#[derive(Clone)]
pub struct MountLifetime(Rc<Inner>);
impl Default for MountLifetime {
    fn default() -> Self {
        Self::new()
    }
}
impl MountLifetime {
    pub fn new() -> Self {
        Self(Rc::new(Inner {
            status: Cell::new(MountStatus {
                phase: MountPhase::New,
                startup_pending: false,
                cleanup_pending: false,
                listeners: 0,
                raf_pending: false,
                listener_removal_failures: 0,
            }),
            startup_waker: RefCell::new(None),
            cleanup: RefCell::new(Vec::new()),
        }))
    }
    pub fn status(&self) -> MountStatus {
        self.0.status.get()
    }
    fn update(&self, edit: impl FnOnce(&mut MountStatus)) {
        let mut status = self.status();
        edit(&mut status);
        self.0.status.set(status);
    }
    pub(crate) fn admitted(&self) -> bool {
        matches!(
            self.status().phase,
            MountPhase::Starting | MountPhase::Active
        )
    }
    pub(crate) fn check(&self) -> Result<(), String> {
        self.admitted()
            .then_some(())
            .ok_or_else(|| "mount disposed".into())
    }
    pub(crate) fn begin(&self) -> Result<(), String> {
        if self.status().phase != MountPhase::New {
            return Err("mount lifetime already used or disposed".into());
        }
        self.update(|s| {
            s.phase = MountPhase::Starting;
            s.startup_pending = true;
        });
        Ok(())
    }
    pub(crate) fn activate(&self) -> Result<(), String> {
        self.check()?;
        self.update(|s| s.phase = MountPhase::Active);
        Ok(())
    }
    fn finish_startup(&self) {
        self.update(|s| s.startup_pending = false);
        let retired = self.0.startup_waker.borrow_mut().take();
        drop(retired);
        self.finish_disposal();
    }
    fn finish_disposal(&self) {
        if self.status().phase == MountPhase::Disposing
            && !self.status().startup_pending
            && !self.status().cleanup_pending
        {
            self.update(|s| s.phase = MountPhase::Disposed);
        }
    }
    /// Add application-owned cleanup (for example TREE and gesture bindings).
    /// Runs once outside a mount callback on disposal. Registration after a
    /// disposal request is also deferred and prevents a premature Disposed.
    pub fn on_dispose(&self, action: impl FnOnce() + 'static) {
        self.0.cleanup.borrow_mut().push(Box::new(action));
        if matches!(
            self.status().phase,
            MountPhase::Disposing | MountPhase::Disposed
        ) {
            self.update(|s| s.phase = MountPhase::Disposing);
            self.schedule_cleanup();
        }
    }
    /// Request cancellation/disposal once. Cleanup completes asynchronously in
    /// the browser; observe status instead of equating this return with release.
    pub fn dispose(&self) -> bool {
        if matches!(
            self.status().phase,
            MountPhase::Disposing | MountPhase::Disposed
        ) {
            return false;
        }
        self.update(|s| s.phase = MountPhase::Disposing);
        let waker = self.0.startup_waker.borrow_mut().take();
        if let Some(waker) = waker {
            waker.wake();
        }
        self.schedule_cleanup();
        true
    }
    fn schedule_cleanup(&self) {
        if self.status().cleanup_pending {
            return;
        }
        self.update(|s| s.cleanup_pending = true);
        let lifetime = self.clone();
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move { lifetime.clean() });
        #[cfg(not(target_arch = "wasm32"))]
        lifetime.clean();
    }
    fn clean(&self) {
        // Release the RefCell before cleanup callbacks can register another
        // action. Reverse order removes listeners/RAF before host GPU teardown.
        loop {
            let actions = std::mem::take(&mut *self.0.cleanup.borrow_mut());
            if actions.is_empty() {
                break;
            }
            for action in actions.into_iter().rev() {
                action();
            }
        }
        self.update(|s| s.cleanup_pending = false);
        self.finish_disposal();
    }
    pub(crate) async fn startup<F: Future>(&self, future: F) -> Result<F::Output, String> {
        self.begin()?;
        Startup {
            future: Some(Box::pin(future)),
            lifetime: self.clone(),
        }
        .await
    }
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn raf_pending(&self, pending: bool) {
        self.update(|s| s.raf_pending = pending);
    }
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn listen(
        &self,
        target: &web_sys::EventTarget,
        name: &str,
        callback: &wasm_bindgen::JsValue,
        keep_closure: impl FnOnce() + 'static,
    ) -> Result<(), String> {
        use wasm_bindgen::JsCast;
        self.check()?;
        target
            .add_event_listener_with_callback(name, callback.unchecked_ref())
            .map_err(|_| format!("could not attach {name}"))?;
        let target = target.clone();
        let name = name.to_owned();
        let callback = callback.clone();
        let lifetime = self.clone();
        self.update(|s| s.listeners += 1);
        self.on_dispose(move || {
            let removed = target
                .remove_event_listener_with_callback(&name, callback.unchecked_ref())
                .is_ok();
            lifetime.update(|s| {
                s.listeners -= 1;
                if !removed {
                    s.listener_removal_failures += 1;
                }
            });
            // Dropping Rust closures after removal releases their host handles.
            keep_closure();
        });
        Ok(())
    }
}

struct Startup<F: Future> {
    future: Option<Pin<Box<F>>>,
    lifetime: MountLifetime,
}
impl<F: Future> Future for Startup<F> {
    type Output = Result<F::Output, String>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if !this.lifetime.admitted() {
            drop(this.future.take());
            this.lifetime.finish_startup();
            return Poll::Ready(Err("mount disposed during startup".into()));
        }
        let replacement = cx.waker().clone();
        let retired = this
            .lifetime
            .0
            .startup_waker
            .borrow_mut()
            .replace(replacement);
        drop(retired);
        let result = this
            .future
            .as_mut()
            .expect("startup already collected")
            .as_mut()
            .poll(cx);
        match result {
            Poll::Pending => Poll::Pending,
            Poll::Ready(output) => {
                drop(this.future.take());
                this.lifetime.finish_startup();
                if this.lifetime.admitted() {
                    Poll::Ready(Ok(output))
                } else {
                    drop(output);
                    Poll::Ready(Err("mount disposed during startup".into()))
                }
            },
        }
    }
}
impl<F: Future> Drop for Startup<F> {
    fn drop(&mut self) {
        drop(self.future.take());
        self.lifetime.finish_startup();
        if self.lifetime.status().phase == MountPhase::Starting {
            self.lifetime.dispose();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    struct WakeCount(AtomicUsize);
    impl std::task::Wake for WakeCount {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    struct Acquisition(Rc<Cell<usize>>);
    impl Future for Acquisition {
        type Output = ();
        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> {
            Poll::Pending
        }
    }
    impl Drop for Acquisition {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    #[test]
    fn cancellation_wakes_and_releases_actual_startup_future_before_disposed() {
        let lifetime = MountLifetime::new();
        let drops = Rc::new(Cell::new(0));
        let mut pending = Box::pin(lifetime.startup(Acquisition(drops.clone())));
        let wake = Arc::new(WakeCount(AtomicUsize::new(0)));
        let waker = Waker::from(wake.clone());
        let mut cx = Context::from_waker(&waker);
        assert!(pending.as_mut().poll(&mut cx).is_pending());
        assert!(lifetime.dispose());
        assert_eq!(wake.0.load(Ordering::SeqCst), 1);
        assert_eq!(lifetime.status().phase, MountPhase::Disposing);
        assert_eq!(drops.get(), 0);
        assert!(matches!(
            pending.as_mut().poll(&mut cx),
            Poll::Ready(Err(_))
        ));
        assert_eq!(drops.get(), 1);
        assert_eq!(lifetime.status().phase, MountPhase::Disposed);
        assert!(!lifetime.dispose());
    }
    #[test]
    fn cleanup_is_reverse_order_once_and_retained_handle_can_observe_completion() {
        let lifetime = MountLifetime::new();
        lifetime.begin().unwrap();
        lifetime.activate().unwrap();
        lifetime.finish_startup();
        let order = Rc::new(RefCell::new(Vec::new()));
        for id in 0..3 {
            let out = order.clone();
            lifetime.on_dispose(move || out.borrow_mut().push(id));
        }
        let observer = lifetime.clone();
        drop(lifetime);
        assert_eq!(observer.status().phase, MountPhase::Active);
        assert!(observer.dispose());
        assert_eq!(*order.borrow(), [2, 1, 0]);
        assert_eq!(observer.status().phase, MountPhase::Disposed);
        assert!(!observer.dispose());
        assert_eq!(*order.borrow(), [2, 1, 0]);
    }
}
