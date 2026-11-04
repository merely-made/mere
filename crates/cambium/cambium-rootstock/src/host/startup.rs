// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl<State, Logic, V, T> Host<State, Logic, V, T>
where
    T: crate::HostTree<State>,
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    /// Detach an independent host's rendering lifetime, including an installed
    /// presentation driver and its owned captures. The application runner
    /// and CPU authority remain inspectable. Drop the returned guard only after
    /// releasing the host borrow. This is disposal, not a recovery transition.
    /// The registry and fragment map must belong to this independent host;
    /// disposing a window while lending a shared forest is not qualified.
    pub fn take_disposal(&mut self) -> HostDisposal {
        self.s.hidden = true;
        self.s.a11y_owner.set(false);
        let a11y_notice = if !matches!(
            self.s.a11y_publication.outcome,
            crate::A11yPublicationOutcome::NotAttempted {
                reason: crate::A11yPublicationSkip::OwnerEnded
            }
        ) {
            let publication = crate::A11yPublication::skipped(
                self.s
                    .a11y
                    .as_ref()
                    .map_or(crate::A11yAdapterState::NotConfigured, |a11y| {
                        a11y.adapter_state()
                    }),
                crate::A11yPublicationSkip::OwnerEnded,
            );
            Some(self.stamp_a11y_publication(publication))
        } else {
            None
        };
        self.s.pending_a11y.clear();
        self.s.needs_recovery_surface = false;
        self.s.last_redraw_presentation = None;
        self.s.pending_presentation_observer = None;
        // Owned in-flight captures are refused now; this needs no host loan
        // and keeps later frame preparation from presenting.
        self.s.capture_owner.scope().dispose();
        HostDisposal {
            producers: Some(std::mem::take(&mut self.s.shared.producers)),
            fragments: std::mem::take(&mut self.s.shared.leaf_fragments),
            driver: self.s.presentation_driver.take(),
            surface: self.s.surface.take(),
            core: self.s.shared.render_core.take(),
            subscription: self.s.recovery_subscription.take(),
            owner: self.s.shared.render_recovery.take(),
            capture: self.s.pending_capture.take(),
            stamped: self.s.pending_stamped_capture.take(),
            paint: self.s.pending_paint_capture.take(),
            a11y_observer: self.s.a11y_publication_observer.take(),
            a11y_notice,
        }
    }

    /// Assemble a host and seed the runtime zoom from its options.
    ///
    /// The seeding is why this exists rather than a struct literal:
    /// [`HostOptions::ui_zoom`] is an input and `HostState::user_zoom` is
    /// where it lives afterwards, and a second event source that built the
    /// struct by hand would silently launch at 1.0.
    pub fn new(
        options: HostOptions,
        init: Option<InitFn<State, Logic>>,
        hooks: HostHooks<State, Logic, V, T>,
        s: HostState<State, Logic, V, T>,
        wake: HostWake,
    ) -> Self {
        let mut host = Self {
            options,
            init,
            hooks,
            s,
            wake,
        };
        host.s.user_zoom = host.options.ui_zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        host.s.zoom_seen = host.ui_zoom();
        host
    }

    /// The application's handle on the window-verb queue. Core data: pushing a
    /// verb is portable, honouring one is the event source's problem.
    pub fn commands(&self) -> WindowCommands {
        self.s.commands.clone()
    }

    /// The **device** scale: physical pixels per device-logical pixel, and
    /// nothing else. This is the window's truth — frame insets, the monitor
    /// clamp, geometry persistence, the platform's own logical coordinates —
    /// and it never carries zoom. Everything the *document* is measured in
    /// goes through [`layout_scale`](Self::layout_scale) instead.
    pub fn scale_factor(&self) -> f64 {
        self.s.window.as_ref().map_or(1.0, |w| w.scale_factor())
    }

    /// The effective interface zoom: the fit factor, if a design was named,
    /// times the user's own zoom.
    ///
    /// One rule covers both knobs. Without `fit_design` the fit factor is
    /// `1.0`, so this is the user's zoom exactly and the keyboard ladder walks
    /// real rungs. With one, the user's zoom is an offset multiplied onto the
    /// fit, so a step still means "a rung more than whatever fits".
    pub fn ui_zoom(&self) -> f32 {
        (self.s.fit_factor * self.s.user_zoom).clamp(MIN_ZOOM, MAX_ZOOM)
    }

    /// The user's own zoom, before the fit factor: what `Ctrl+0` resets and
    /// what [`set_ui_zoom`](Self::set_ui_zoom) writes.
    pub fn user_zoom(&self) -> f32 {
        self.s.user_zoom
    }

    /// The one scale the frame is composed at: device scale times zoom.
    ///
    /// Layout runs at `window_physical / layout_scale`, rasterization runs at
    /// `layout_scale`, and every logical coordinate the host exchanges with
    /// the application is in that post-zoom space.
    pub fn layout_scale(&self) -> f64 {
        self.scale_factor() * f64::from(self.ui_zoom())
    }

    /// The surface in **device-logical** pixels — the pre-zoom size, and what
    /// [`HostOptions::fit_design`] is measured against.
    ///
    /// Windowless, this is whatever the event source last stated (the
    /// `cambium_genet_winit_host::Harness` states it as it lays out), falling
    /// back to the size the retained layout was built at.
    pub fn available_size(&self) -> (f32, f32) {
        match self.s.window.as_ref() {
            Some(window) => {
                let size = window.inner_size();
                let scale = window.scale_factor() as f32;
                (size.0.max(1) as f32 / scale, size.1.max(1) as f32 / scale)
            },
            None => self.s.surface_size.unwrap_or(self.s.layout_size),
        }
    }

    /// CPU timings from the most recently presented frame. Windowless
    /// harnesses use [`Self::relayout_profile`] instead.
    pub fn frame_profile(&self) -> Option<FrameProfile> {
        self.s.last_frame_profile
    }

    /// Timings from the most recent retained relayout, including windowless
    /// harness relayouts that never enter the presentation pipeline.
    pub fn relayout_profile(&self) -> RelayoutProfile {
        RelayoutProfile {
            layout_update_us: self.s.last_layout_update_us,
            layout_tick_us: self.s.last_layout_tick_us,
            layout_apply_us: self.s.last_layout_apply_us,
            layout_rebuild_us: self.s.last_layout_rebuild_us,
            style_resolve_us: self.s.last_style_resolve_us,
            layout_with_text_us: self.s.last_layout_with_text_us,
            content_extent_us: self.s.last_content_extent_us,
            layout_mutations: self.s.last_layout_mutations,
            layout_rebuilt: self.s.last_layout_rebuilt,
            leaf_boxes_us: self.s.last_leaf_boxes_us,
            leaf_render_us: self.s.last_leaf_render_us,
            leaf_repaints: self.s.last_leaf_repaints,
        }
    }

    /// The logical size being laid out: the window's over the layout scale,
    /// or — windowless, under `cambium_genet_winit_host::Harness` — the size the
    /// retained layout was last built at.
    pub fn logical_size(&self) -> (f32, f32) {
        match self.s.window.as_ref() {
            Some(window) => {
                let size = window.inner_size();
                let scale = self.layout_scale() as f32;
                (size.0.max(1) as f32 / scale, size.1.max(1) as f32 / scale)
            },
            None => self.s.layout_size,
        }
    }

    /// A point the platform reported in **physical** window pixels, in the
    /// layout's own coordinates.
    ///
    /// The one conversion every pointer path crosses, so an event source does
    /// not divide by hand — and so the harness can drive the same arithmetic
    /// the winit `CursorMoved` arm runs.
    pub fn layout_point(&self, x: f64, y: f64) -> (f32, f32) {
        let scale = self.layout_scale();
        ((x / scale) as f32, (y / scale) as f32)
    }

    /// State the surface size a windowless host stands in for, in
    /// device-logical pixels.
    ///
    /// Ignored while there is a window, which knows better. This is how the
    /// `cambium_genet_winit_host::Harness` gives `fit_design` something to measure
    /// against without inventing a fake window.
    pub fn set_surface_size(&mut self, width: f32, height: f32) {
        self.s.surface_size = Some((width, height));
    }

    /// Set the user's zoom and relayout under it.
    ///
    /// Returns whether the effective zoom actually moved. With `fit_design`
    /// set this is the offset on the fit factor, not the whole zoom; see
    /// [`ui_zoom`](Self::ui_zoom).
    pub fn set_ui_zoom(&mut self, zoom: f32) -> bool {
        let zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        if zoom == self.s.user_zoom {
            return false;
        }
        self.s.user_zoom = zoom;
        self.note_zoom_change();
        true
    }

    /// Step the user's zoom one rung of [`ZOOM_LADDER`], up or down.
    pub fn step_ui_zoom(&mut self, up: bool) -> bool {
        self.set_ui_zoom(ladder_step(self.s.user_zoom, up))
    }

    /// Ctrl+0: clear the user's zoom. With `fit_design` set that leaves the
    /// fit factor in place rather than forcing the interface to 1.0, because
    /// "reset" there means "back to what fits", not "back to unscaled".
    pub fn reset_ui_zoom(&mut self) -> bool {
        self.set_ui_zoom(1.0)
    }

    /// Recompute the fit factor against the surface as it is now. Returns
    /// whether the effective zoom moved. A no-op without `fit_design`.
    pub fn refresh_fit_zoom(&mut self) -> bool {
        let Some(design) = self.options.fit_design else {
            return false;
        };
        let factor = fit_zoom(design, self.available_size());
        if factor == self.s.fit_factor {
            return false;
        }
        self.s.fit_factor = factor;
        self.note_zoom_change();
        true
    }

    /// A zoom change: the next frame lays out at the new logical size and
    /// rasterizes at the new layout scale.
    ///
    /// Zeroing `layout_size` rather than dropping the layout is deliberate.
    /// The rebuild branch carries both scroll planes across from the previous
    /// layout, and dropping it would snap a scrolled interface back to the top
    /// every time somebody pressed Ctrl+plus.
    pub(super) fn note_zoom_change(&mut self) {
        self.s.layout_size = (0.0, 0.0);
        if let Some(window) = self.s.window.as_ref() {
            window.request_redraw();
        }
    }

    /// Run one application hook with the standard context, then apply any
    /// host-owned requests it made (sheet swap → relayout, queued pointer
    /// events → the real input path).
    /// The zoom to report, and whether it moved since the application last
    /// saw one. Consuming: the edge belongs to the first hook built after the
    /// change, not to every hook that follows it.
    pub(crate) fn take_zoom_edge(&mut self) -> (f32, bool) {
        let zoom = self.ui_zoom();
        let changed = zoom != self.s.zoom_seen;
        self.s.zoom_seen = zoom;
        (zoom, changed)
    }

    pub fn with_ctx(&mut self, which: Hook) {
        {
            let logical_size = self.logical_size();
            let (ui_zoom, zoom_changed) = self.take_zoom_edge();
            let geometry = self.s.geometry;
            let frame_profile = self.s.last_frame_profile;
            let commands = self.s.commands.clone();
            let window = self.s.window.as_deref();
            let layout = self.s.layout.as_ref();
            let Some(runner) = self.s.runner.as_mut() else {
                return;
            };
            let mut ctx = AppCtx {
                runner,
                layout,
                window,
                logical_size,
                ui_zoom,
                zoom_changed,
                leaves: &mut self.s.shared.leaves,
                files: &mut self.s.files,
                producers: &mut self.s.shared.producers,
                set_sheet: &mut self.s.pending_sheet,
                set_ui_zoom: &mut self.s.pending_ui_zoom,
                close: &mut self.s.close_requested,
                wake: &self.wake,
                capture: &mut self.s.pending_capture,
                presentation_observer: &mut self.s.pending_presentation_observer,
                capture_stamped: &mut self.s.pending_stamped_capture,
                capture_paint: &mut self.s.pending_paint_capture,
                pointer: &mut self.s.pending_pointer,
                a11y: &mut self.s.pending_a11y,
                a11y_owner: &self.s.a11y_owner,
                a11y_publication: crate::A11yPublicationSnapshot {
                    host: self.s.presentation_host,
                    sequence: self.s.a11y_publication_sequence,
                    publication: self.s.a11y_publication,
                },
                scroll: &mut self.s.pending_scroll,
                window_commands: &commands,
                render_core: self.s.shared.render_core.as_ref(),
                geometry,
                frame_profile,
                presentation: self.s.last_redraw_presentation,
            };
            match which {
                Hook::AfterDispatch => (self.hooks.after_dispatch)(&mut ctx),
                Hook::AfterFrame => (self.hooks.after_frame)(&mut ctx),
                Hook::AfterWake => (self.hooks.after_wake)(&mut ctx),
            }
        }
        self.apply_pending();
    }

    /// Seal a read-only projection of the source just submitted. Deliberately
    /// does not apply pending requests, consume zoom edges or dispatch input.
    pub fn observe_presentation(&mut self, frame: PresentedFrame) {
        let Some(observer) = self.s.pending_presentation_observer.take() else {
            return;
        };
        let logical_size = self.logical_size();
        let ui_zoom = self.ui_zoom();
        let commands = self.s.commands.clone();
        let Some(runner) = self.s.runner.as_mut() else {
            return;
        };
        let ctx = AppCtx {
            runner,
            layout: self.s.layout.as_ref(),
            window: self.s.window.as_deref(),
            logical_size,
            ui_zoom,
            zoom_changed: false,
            leaves: &mut self.s.shared.leaves,
            files: &mut self.s.files,
            producers: &mut self.s.shared.producers,
            set_sheet: &mut self.s.pending_sheet,
            set_ui_zoom: &mut self.s.pending_ui_zoom,
            close: &mut self.s.close_requested,
            wake: &self.wake,
            capture: &mut self.s.pending_capture,
            presentation_observer: &mut self.s.pending_presentation_observer,
            capture_stamped: &mut self.s.pending_stamped_capture,
            capture_paint: &mut self.s.pending_paint_capture,
            pointer: &mut self.s.pending_pointer,
            a11y: &mut self.s.pending_a11y,
            a11y_owner: &self.s.a11y_owner,
            a11y_publication: crate::A11yPublicationSnapshot {
                host: self.s.presentation_host,
                sequence: self.s.a11y_publication_sequence,
                publication: self.s.a11y_publication,
            },
            scroll: &mut self.s.pending_scroll,
            window_commands: &commands,
            render_core: self.s.shared.render_core.as_ref(),
            geometry: self.s.geometry,
            frame_profile: self.s.last_frame_profile,
            presentation: self.s.last_redraw_presentation,
        };
        observer(&ctx, frame);
    }

    /// Apply requests a hook made after its temporary borrows of the runner and
    /// view state ended. Shared by normal hooks and close negotiation.
    /// Swap the application's stylesheet: this window lays out afresh under
    /// it, and the generation tells every other window to.
    pub(crate) fn swap_sheet(&mut self, sheet: String) {
        self.s.shared.sheet = sheet;
        self.s.shared.sheet_generation += 1;
        self.s.layout = None;
        self.s.layout_size = (0.0, 0.0);
    }

    pub(super) fn apply_pending(&mut self) {
        if let Some(sheet) = self.s.pending_sheet.take() {
            self.swap_sheet(sheet);
        }
        if let Some(zoom) = self.s.pending_ui_zoom.take() {
            self.set_ui_zoom(zoom);
        }
        // Scroll requests resolve at the next layout; a hook that runs after
        // the frame (`after_frame`) would otherwise wait for an unrelated one.
        if !self.s.pending_scroll.is_empty()
            && let Some(window) = self.s.window.as_ref()
        {
            window.request_redraw();
        }
        self.drain_pointer();
    }

    /// Drain one application wake. The caller is the native `UserEvent` path
    /// or the windowless harness; either way, a worker owns its channel and
    /// this host only gives it a UI-thread drain turn and a redraw.
    pub fn process_wake(&mut self) -> bool {
        if !self.wake.take_pending() {
            return false;
        }
        self.with_ctx(Hook::AfterWake);
        if !self.s.hidden {
            if let Some(window) = self.s.window.as_ref() {
                window.request_redraw();
            }
        }
        true
    }

    /// Ask the application whether a close should keep the window visible,
    /// hide it while its work continues, or terminate the event loop.
    /// Run the application's close policy and record what the core can act on
    /// itself ([`CloseDisposition::Exit`] sets the exit flag). `None` when there
    /// is no runner yet, which also sets the flag: closing an app that never
    /// booted should succeed. Reacting to `Hide` and `KeepVisible` needs a real
    /// window, so that half lives with the event source.
    pub fn decide_close(&mut self, request: CloseRequest) -> Option<CloseDisposition> {
        let disposition = {
            let logical_size = self.logical_size();
            let (ui_zoom, zoom_changed) = self.take_zoom_edge();
            let geometry = self.s.geometry;
            let frame_profile = self.s.last_frame_profile;
            let commands = self.s.commands.clone();
            let window = self.s.window.as_deref();
            let layout = self.s.layout.as_ref();
            let Some(runner) = self.s.runner.as_mut() else {
                self.s.close_requested = true;
                return None;
            };
            let mut ctx = AppCtx {
                runner,
                layout,
                window,
                logical_size,
                ui_zoom,
                zoom_changed,
                leaves: &mut self.s.shared.leaves,
                files: &mut self.s.files,
                producers: &mut self.s.shared.producers,
                set_sheet: &mut self.s.pending_sheet,
                set_ui_zoom: &mut self.s.pending_ui_zoom,
                close: &mut self.s.close_requested,
                wake: &self.wake,
                capture: &mut self.s.pending_capture,
                presentation_observer: &mut self.s.pending_presentation_observer,
                capture_stamped: &mut self.s.pending_stamped_capture,
                capture_paint: &mut self.s.pending_paint_capture,
                pointer: &mut self.s.pending_pointer,
                a11y: &mut self.s.pending_a11y,
                a11y_owner: &self.s.a11y_owner,
                a11y_publication: crate::A11yPublicationSnapshot {
                    host: self.s.presentation_host,
                    sequence: self.s.a11y_publication_sequence,
                    publication: self.s.a11y_publication,
                },
                scroll: &mut self.s.pending_scroll,
                window_commands: &commands,
                render_core: self.s.shared.render_core.as_ref(),
                geometry,
                frame_profile,
                presentation: self.s.last_redraw_presentation,
            };
            (self.hooks.close_request)(&mut ctx, request)
        };
        self.apply_pending();
        if self.s.close_requested {
            return None;
        }
        if matches!(disposition, CloseDisposition::Exit) {
            self.s.close_requested = true;
        }
        Some(disposition)
    }

    /// Deliver the pointer events an application queued, in order, through the
    /// same routing a real mouse takes. Drained outside the hook's borrow of
    /// the runner, so the delivery can hit-test, capture, and dispatch exactly
    /// as `window_event` does.
    ///
    /// `after_dispatch` is *not* re-entered from here for the hook that queued
    /// the events — each delivery runs its own, and the queue is taken whole so
    /// an event queued by that dispatch lands on the next drain rather than
    /// looping.
    // Pointer gestures retain their existing order. Semantic requests follow
    // through the same admission/dispatch path used by platform accessibility.
    pub fn drain_pointer(&mut self) {
        for event in std::mem::take(&mut self.s.pending_pointer) {
            match event {
                HostPointer::Moved(x, y) => {
                    self.pointer_moved(x, y);
                },
                HostPointer::Press(x, y) => {
                    self.s.cursor = (x, y);
                    self.click();
                },
                HostPointer::Release(x, y) => {
                    self.s.cursor = (x, y);
                    self.release();
                },
                HostPointer::SecondaryPress(x, y) => {
                    self.s.cursor = (x, y);
                    self.secondary_press();
                },
                HostPointer::SecondaryRelease(x, y) => {
                    self.s.cursor = (x, y);
                    self.release_button(cambium::PointerButton::Secondary);
                },
                HostPointer::Cancel => self.cancel_pointer(),
                HostPointer::Wheel {
                    x,
                    y,
                    dx,
                    dy,
                    unit,
                    modifiers,
                } => {
                    self.s.cursor = (x, y);
                    let scale = match unit {
                        WheelUnit::Pixels => 1.0,
                        WheelUnit::Lines => HOST_WHEEL_LINE_PX,
                    };
                    let held = std::mem::replace(&mut self.s.modifiers, modifiers);
                    self.wheel(dx * scale, dy * scale);
                    self.s.modifiers = held;
                },
            }
        }
        // Post-dispatch hooks may enqueue another request. Reentrant hook
        // drains must not consume it recursively within this batch.
        if !self.s.draining_a11y {
            self.s.draining_a11y = true;
            for queued in std::mem::take(&mut self.s.pending_a11y) {
                let current = self.s.runner.as_ref().is_some_and(|runner| {
                    queued.is_current(
                        &self.s.a11y_owner,
                        &runner.dom(),
                        runner.mount(),
                        &self.s.shared.producers,
                        &self.s.shared.leaves,
                    )
                });
                let result = if !self.s.a11y_owner.get() {
                    crate::A11yDispatch::Rejected(crate::A11yRejection::Cancelled)
                } else if current {
                    self.apply_a11y_requests(std::slice::from_ref(&queued.request))[0]
                } else {
                    crate::A11yDispatch::Rejected(crate::A11yRejection::StaleTarget)
                };
                queued.finish(result);
            }
            self.s.draining_a11y = false;
        }
    }

    /// The tail of every input dispatch: the application hook, then the
    /// host-owned IME and repaint policy.
    pub fn after_dispatch(&mut self) {
        self.with_ctx(Hook::AfterDispatch);
        self.open_requested_file();
        let Some(window) = self.s.window.as_ref() else {
            return;
        };
        window.set_ime_allowed(
            self.s
                .runner
                .as_ref()
                .is_some_and(|runner| (self.hooks.focused_text)(runner).is_some()),
        );
        window.request_redraw();
    }
}
