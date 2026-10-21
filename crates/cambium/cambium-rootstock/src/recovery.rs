// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shared-core loss handling in the actual host, before producer/GPU work.
use crate::meristem_bounds::RootView;
use crate::{CaptureRefusal, Host, HostTree};
use genet_render_host::{RecoveryStatus, SharedRenderCore};
use std::sync::Arc;

impl<State, Logic, V, T> Host<State, Logic, V, T>
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
    T: HostTree<State>,
{
    fn refuse_pending_captures(&mut self, refusal: CaptureRefusal) {
        if let Some(capture) = self.s.pending_capture.take() {
            capture(Err(refusal.clone()));
        }
        if let Some(capture) = self.s.pending_stamped_capture.take() {
            capture(Err(refusal.clone()));
        }
        if let Some(capture) = self.s.pending_paint_capture.take() {
            capture(Err(refusal));
        }
        self.s.pending_presentation_observer = None;
    }
    /// Refuse the current frame explicitly. Only an actual installed loss
    /// notice enters recovery; healthy provider refusal preserves this core.
    pub fn refuse_rasterization(
        &mut self,
        failure: genet_render_host::RasterizeFailure,
        source_key: Option<u64>,
    ) {
        let failure = if matches!(
            failure,
            genet_render_host::RasterizeFailure::ProviderError(_)
        ) {
            self.s
                .shared
                .render_core
                .as_ref()
                .and_then(|core| core.device_failure())
                .map(genet_render_host::RasterizeFailure::DeviceLost)
                .unwrap_or(failure)
        } else {
            failure
        };
        self.s.last_redraw_presentation = None;
        self.s.last_raster_failure = Some(failure.clone());
        match failure {
            genet_render_host::RasterizeFailure::DeviceLost(loss) => {
                self.refuse_pending_captures(CaptureRefusal::DeviceLost(loss));
                self.poll_device_recovery();
            },
            genet_render_host::RasterizeFailure::ProviderError(error) => {
                let generation = self
                    .s
                    .shared
                    .render_core
                    .as_ref()
                    .map_or(0, |core| core.generation());
                self.refuse_pending_captures(CaptureRefusal::ProviderError {
                    generation,
                    source_key,
                    error,
                });
            },
        }
    }

    pub fn attach_shared_recovery(&mut self, owner: Arc<SharedRenderCore>) {
        if self
            .s
            .shared
            .render_recovery
            .as_ref()
            .is_some_and(|old| Arc::ptr_eq(old, &owner))
            && self.s.recovery_subscription.is_some()
        {
            return;
        }
        self.s.recovery_subscription = Some(owner.subscribe(self.wake.callback()));
        self.s.shared.render_recovery = Some(owner);
    }

    /// Each live host drops its own GPU resources before acknowledging. The
    /// shared owner boots once only after every live host has acknowledged.
    /// Application runners, layout, CPU leaves and portable sources survive.
    pub fn poll_device_recovery(&mut self) -> RecoveryStatus {
        if self.s.shared.render_recovery.is_none()
            && let Some(core) = self.s.shared.render_core.clone()
        {
            self.attach_shared_recovery(core.shared_recovery());
        }
        let Some(owner) = self.s.shared.render_recovery.clone() else {
            return RecoveryStatus::Ready {
                generation: self
                    .s
                    .shared
                    .render_core
                    .as_ref()
                    .map_or(0, |core| core.generation()),
            };
        };
        if self.s.recovery_subscription.is_none() {
            self.attach_shared_recovery(owner.clone());
        }
        let status = owner.status();
        if let Some(loss) = status.loss() {
            #[cfg(target_arch = "wasm32")]
            {
                self.cancel_browser_frame();
                if self.has_browser_retirements() {
                    // Event source releases these outside Host RefMut before
                    // invoking us again; never acknowledge live frame resources.
                    return status;
                }
            }
            if self.s.detached_generation != Some(loss.generation) {
                self.s.last_redraw_presentation = None;
                // Clear every producer in this shared core, including leaves
                // held by another window. Per-window suspension is insufficient.
                #[cfg(not(target_arch = "wasm32"))]
                {
                    self.s.shared.producers.suspend_all(None);
                    self.s.shared.leaf_fragments.clear();
                    self.s.surface = None;
                    self.s.shared.render_core = None;
                    self.s.needs_recovery_surface = true;
                    self.s.detached_generation = Some(loss.generation);
                }
                #[cfg(target_arch = "wasm32")]
                {
                    self.s.shared.producers.suspend_browser_except(&|_,_| false,None);
                    self.s.shared.producers.forget_device();
                    self.s.browser_retirements.push(Box::new(crate::frame::LostBrowserSession {
                        core:self.s.shared.render_core.take(), surface:self.s.surface.take(),
                        fragments:std::mem::take(&mut self.s.shared.leaf_fragments),
                    }));
                    self.s.needs_recovery_surface = true;
                    self.s.detached_generation = Some(loss.generation);
                    // The next call, after outside-borrow drop, acknowledges.
                    return status;
                }
            }
            self.refuse_pending_captures(CaptureRefusal::DeviceLost(loss.clone()));
            owner.acknowledge_detached(
                self.s
                    .recovery_subscription
                    .as_ref()
                    .expect("subscribed host"),
                loss.generation,
            );
        }
        let status = owner.recover();
        if let RecoveryStatus::Ready { generation } = status {
            if let Some(RecoveryStatus::SurfaceUnavailable {
                generation: failed, ..
            }) = &self.s.recovery_status
                && *failed == generation
            {
                let status = self.s.recovery_status.clone().expect("surface refusal");
                if let RecoveryStatus::SurfaceUnavailable { generation, error } = &status {
                    self.refuse_pending_captures(CaptureRefusal::SurfaceUnavailable {
                        generation: *generation,
                        error: error.clone(),
                    });
                }
                return status;
            }
            if let Ok(core) = owner.current() {
                self.s.shared.render_core = Some(core);
            }
        }
        self.s.recovery_status = Some(status.clone());
        status
    }

    /// A new device is ready but this window's surface could not be configured.
    /// Keep the failure typed and explicit; never reattach the lost surface.
    pub fn refuse_recovery_surface(&mut self, generation: u64, error: String) {
        self.s.surface = None;
        self.s.last_redraw_presentation = None;
        self.s.recovery_status = Some(RecoveryStatus::SurfaceUnavailable {
            generation,
            error: error.clone(),
        });
        let refusal = CaptureRefusal::SurfaceUnavailable { generation, error };
        self.refuse_pending_captures(refusal);
    }
}

#[cfg(test)]
mod raster_refusal_tests {
    use super::*;
    use crate::{HostHooks, HostOptions, HostState, HostWake, Runner, ScriptedDom};
    use cambium::{AnyView, GenetCtx, GenetElement, el, text};
    use genet_render_host::{DeviceLossNotice, RasterizeFailure};
    use std::{
        cell::RefCell,
        rc::Rc,
        sync::atomic::{AtomicUsize, Ordering},
    };

    struct App {
        draft: String,
        source: Rc<RefCell<String>>,
    }
    type Child = Box<dyn AnyView<App, (), GenetCtx, GenetElement>>;
    type Logic = fn(&App) -> Child;
    fn root(app: &App) -> Child {
        Box::new(el("div", text(app.draft.clone())))
    }

    #[test]
    fn healthy_provider_failure_refuses_all_captures_once_without_recovery() {
        let source = Rc::new(RefCell::new("bound application source".to_owned()));
        let mut state: HostState<App, Logic, Child> = HostState::new();
        state.runner = Some(Runner::new(
            Rc::new(RefCell::new(ScriptedDom::new())),
            root as Logic,
            App {
                draft: "authored pending draft".into(),
                source: source.clone(),
            },
        ));
        let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
        let hooks: HostHooks<App, Logic, Child> = HostHooks::inert();
        let mut host: Host<App, Logic, Child> =
            Host::new(HostOptions::default(), None, hooks, state, wake);
        let calls = Arc::new(AtomicUsize::new(0));
        let counted = calls.clone();
        let owner = SharedRenderCore::from_detached_loss(
            DeviceLossNotice {
                generation: 811,
                reason: wgpu::DeviceLostReason::Unknown,
                message: "CPU-only owner state; no GPU loss claimed".into(),
            },
            Arc::new(move || {
                counted.fetch_add(1, Ordering::SeqCst);
                Err("must not run".into())
            }),
        );
        host.attach_shared_recovery(owner.clone());
        let before = owner.status();
        let refusals = Rc::new(RefCell::new(Vec::new()));
        fn check(refusal: CaptureRefusal, out: &Rc<RefCell<Vec<u64>>>) {
            match refusal {
                CaptureRefusal::ProviderError {
                    source_key: Some(7),
                    error,
                    ..
                } => {
                    assert!(matches!(
                        &*error,
                        netrender::VelloError::CurveQualityReadbackFailed
                    ));
                    out.borrow_mut().push(7);
                },
                other => panic!("healthy provider error was misclassified: {other:?}"),
            }
        }
        let image = refusals.clone();
        host.s.pending_capture = Some(Box::new(move |result| check(result.err().unwrap(), &image)));
        let stamped = refusals.clone();
        host.s.pending_stamped_capture = Some(Box::new(move |result| {
            check(result.err().unwrap(), &stamped)
        }));
        let paint = refusals.clone();
        host.s.pending_paint_capture =
            Some(Box::new(move |result| check(result.err().unwrap(), &paint)));
        for _ in 0..2 {
            host.refuse_rasterization(
                RasterizeFailure::ProviderError(Arc::new(
                    netrender::VelloError::CurveQualityReadbackFailed,
                )),
                Some(7),
            );
        }
        assert_eq!(&*refusals.borrow(), &[7, 7, 7]);
        assert_eq!(owner.status(), before);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(
            host.s
                .shared
                .render_recovery
                .as_ref()
                .is_some_and(|actual| Arc::ptr_eq(actual, &owner))
        );
        assert!(!host.s.needs_recovery_surface);
        let app = host.s.runner.as_ref().unwrap().state();
        assert_eq!(app.draft, "authored pending draft");
        assert!(Rc::ptr_eq(&app.source, &source));
        assert_eq!(&*source.borrow(), "bound application source");
        assert!(matches!(
            host.s.last_raster_failure,
            Some(RasterizeFailure::ProviderError(_))
        ));
    }
}
