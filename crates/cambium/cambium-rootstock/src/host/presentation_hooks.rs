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
    /// Physical input follows the displayed picture while a new UI scale is
    /// being prepared. Layout preparation still uses `layout_scale`.
    pub fn input_scale(&self) -> f64 {
        if self.s.native_receiving {
            self.s
                .displayed_presentation
                .map_or_else(|| self.layout_scale(), |p| f64::from(p.layout_scale))
        } else {
            self.layout_scale()
        }
    }

    /// Freeze source observations while this picture is being prepared.
    pub(crate) fn prepare_presentation_observers(
        &mut self,
        frame: PresentedFrame,
    ) -> Vec<crate::PreparedPresentationObserver> {
        let observer = self.s.pending_presentation_observer.take();
        let logical_size = self.logical_size();
        let ui_zoom = self.ui_zoom();
        let commands = self.s.commands.clone();
        let Some(runner) = self.s.runner.as_mut() else {
            return Vec::new();
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
        let mut prepared = Vec::new();
        if let Some(observation) = (self.hooks.prepare_presentation)(&ctx, frame) {
            prepared.push(observation);
        }
        if let Some(observer) = observer {
            prepared.push(observer(&ctx, frame));
        }
        prepared
    }

    /// Synchronous routes prepare and publish within the same host turn.
    pub fn observe_presentation(&mut self, frame: PresentedFrame) {
        for observer in self.prepare_presentation_observers(frame) {
            observer.publish(frame);
        }
    }

    /// Drain only application-owned work. Native image completion never runs
    /// the application hook or creates model demand through this method.
    pub fn process_application_wake(&mut self) -> bool {
        if !self.wake.take_pending() {
            return false;
        }
        self.s.native_dirty = true;
        self.with_ctx(Hook::AfterWake);
        if !self.s.hidden {
            if let Some(window) = self.s.window.as_ref() {
                window.request_redraw();
            }
        }
        true
    }

    /// Compatibility for event sources that collect on a later redraw. Native
    /// Winit receivers collect completion directly through its separate seam.
    /// An orphan notification is consumed without scheduling a fresh picture.
    pub fn process_wake(&mut self) -> bool {
        let application = self.process_application_wake();
        let notified = self
            .s
            .native_completion_pending
            .swap(false, Ordering::AcqRel);
        #[cfg(not(target_arch = "wasm32"))]
        let completion = notified && self.s.pending_native.is_some();
        #[cfg(target_arch = "wasm32")]
        let completion = {
            let _ = notified;
            false
        };
        #[cfg(not(target_arch = "wasm32"))]
        let availability = notified && !completion && self.wake_native_waiter();
        #[cfg(target_arch = "wasm32")]
        let availability = false;
        if completion && !self.s.hidden {
            if let Some(window) = self.s.window.as_ref() {
                window.request_redraw();
            }
        }
        application || completion || availability
    }
}
