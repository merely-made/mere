// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! UI-turn wakes and finite shared-device replacement, independent of RAF.
//! Hidden documents may stop receiving animation frames. They must still
//! detach their lost surface before the shared owner can acquire a new device.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use cambium_rootstock::{Host, HostWindow, meristem_bounds::RootView};
use genet_render_host::RecoveryStatus;
use wasm_bindgen::{JsCast, closure::Closure};

use crate::MountLifetime;
use crate::surface::{WebSurface, WebWindow};

thread_local! {
    // JS/Rc values stay on this UI thread. HostWake carries only the ID.
    static WAKES: RefCell<BTreeMap<u64, Rc<dyn Fn()>>> = RefCell::new(BTreeMap::new());
}
static NEXT_WAKE: AtomicU64 = AtomicU64::new(1);

pub(crate) struct WakeBridge(u64);

impl WakeBridge {
    pub(crate) fn new() -> Rc<Self> {
        Rc::new(Self(
            NEXT_WAKE
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
                .expect("browser wake identity exhausted"),
        ))
    }

    pub(crate) fn signal(&self) -> Arc<dyn Fn() + Send + Sync> {
        let id = self.0;
        Arc::new(move || {
            // Never borrow a Host synchronously: device-loss callbacks can
            // arrive while a host is holding its RefMut during rasterization.
            wasm_bindgen_futures::spawn_local(async move {
                let callback = WAKES.with(|wakes| wakes.borrow().get(&id).cloned());
                if let Some(callback) = callback {
                    callback();
                }
            });
        })
    }

    pub(crate) fn attach<State, Logic, V>(
        &self,
        host: &Rc<RefCell<Host<State, Logic, V>>>,
        window: &WebWindow,
        lifetime: &MountLifetime,
    ) where
        State: 'static,
        Logic: FnMut(&State) -> V + 'static,
        V: RootView<State>,
    {
        let host = Rc::downgrade(host);
        let window = window.clone();
        let lifetime = lifetime.clone();
        WAKES.with(|wakes| {
            wakes.borrow_mut().insert(
                self.0,
                Rc::new(move || {
                    // Disposal revokes callback authority synchronously, even
                    // before its cleanup microtask removes this bridge entry.
                    // Use phase admission only: passive root liveness is an
                    // acquisition checkpoint, not automatic wake disposal.
                    if !lifetime.admitted() {
                        return;
                    }
                    if let Some(host) = host.upgrade() {
                        drain(&host, &window);
                        if !lifetime.admitted() {
                            return;
                        }
                        // App wake hooks must never receive a detached/lost core.
                        // Process after the same detach authority as native wake.
                        host.borrow_mut().process_wake();
                        if !lifetime.admitted() {
                            return;
                        }
                        drain(&host, &window);
                    }
                }),
            );
        });
    }
}

impl Drop for WakeBridge {
    fn drop(&mut self) {
        WAKES.with(|wakes| wakes.borrow_mut().remove(&self.0));
    }
}

/// Detach/reattach under one host borrow, then claim/acquire outside it.
pub(crate) fn drain<State, Logic, V>(host: &Rc<RefCell<Host<State, Logic, V>>>, window: &WebWindow)
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    // Hidden documents may not receive RAF. Loss/visibility wakes retire
    // provisional GPU handles outside RefMut before detached acknowledgement.
    {
        let retired = {
            let mut host = host.borrow_mut();
            host.cancel_browser_frame_on_loss();
            host.take_browser_retirements()
        };
        drop(retired);
    }
    let owner = {
        let retired = {
            let mut current = host.borrow_mut();
            let _ = current.poll_device_recovery();
            current.take_browser_retirements()
        };
        drop(retired);
        let mut host = host.borrow_mut();
        match host.poll_device_recovery() {
            RecoveryStatus::Ready { generation } if host.s.needs_recovery_surface => {
                let core = host.s.shared.render_core.clone().expect("ready core");
                let (width, height) = window.physical_size();
                match WebSurface::from_core_for_window(window.canvas().clone(),core,width,height,window) {
                    Ok(surface) => {
                        host.s.surface = Some(Box::new(surface));
                        host.s.needs_recovery_surface = false;
                        if !host.s.hidden {
                            window.request_redraw();
                        }
                    },
                    Err(error) => host.refuse_recovery_surface(generation, error),
                }
                None
            },
            RecoveryStatus::Waiting {
                remaining_hosts: 0, ..
            } => host.s.shared.render_recovery.clone(),
            _ => None,
        }
    };
    let Some(owner) = owner else { return };
    // Exactly one mount can claim this owner/generation/attempt. Other mounts
    // wait for its completion wake without creating another acquisition task.
    let Ok(pending) = owner.begin_async_replacement() else {
        return;
    };
    let Some(browser) = web_sys::window() else {
        return;
    };
    let Some(clock) = browser.performance() else {
        return;
    };
    let deadline = clock.now() + 5_000.0;
    let pending = pending.with_deadline(move || clock.now() >= deadline);
    let expiry = pending.expiry();
    let timeout = Closure::<dyn FnMut()>::new(move || {
        let _ = expiry.expire();
    });
    let Ok(timer) = browser.set_timeout_with_callback_and_timeout_and_arguments_0(
        timeout.as_ref().unchecked_ref(),
        5_000,
    ) else {
        return;
    };
    wasm_bindgen_futures::spawn_local(async move {
        // The pending claim holds only a weak owner. Keep the authority alive
        // through this finite task, with no host/surface/old-device handle.
        let _outcome = pending.await;
        browser.clear_timeout_with_handle(timer);
        drop(timeout);
        drop(owner);
        // Genet wakes every subscriber after the completion transition.
    });
}
