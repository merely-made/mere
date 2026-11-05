// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The same frame epilogue follows preparation and direct native collection.
use super::*;

#[cfg(test)]
mod tests;

impl<State, Logic, V, T> WinitHost<State, Logic, V, T>
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
    T: HostTree<State>,
{
    pub(crate) fn application_wake_turn(&mut self) -> bool {
        let application = self.core.process_application_wake();
        // Apply app-requested visibility, close and scale changes before a
        // previously prepared image can pass its publication checks.
        self.sync_app_frame_extents();
        self.run_window_commands();
        application
    }

    pub(crate) fn presentation_wake_turn(&mut self) -> bool {
        // Every receiving window applies its own queued commands before its
        // publication check, including a window other than the app drainer.
        self.sync_app_frame_extents();
        self.run_window_commands();
        if self.core.take_a11y_wake() {
            self.drain_a11y_action_turn();
        }
        self.refresh_geometry();
        #[cfg(not(target_arch = "wasm32"))]
        if self.core.collect_presentation_wake() {
            self.finish_redraw_attempt();
            return true;
        }
        false
    }

    pub(crate) fn finish_redraw_attempt(&mut self) {
        let a11y_started = std::time::Instant::now();
        self.sync_a11y();
        if let Some(profile) = self.s.last_frame_profile.as_mut() {
            profile.a11y_us = a11y_started.elapsed().as_micros().min(u64::MAX as u128) as u64;
            profile.total_us = profile.total_us.saturating_add(profile.a11y_us);
            if perf_trace() {
                eprintln!("[cambium-host] frame {}", profile.summary());
            }
        }
        if frame_trace()
            && self.s.last_redraw_presentation.is_none()
            && let Some(window) = self.native_window.as_ref()
        {
            #[cfg(target_os = "macos")]
            trace_owned_macos_window(window, "unpresented redraw");
            if let Some(failure) = &self.s.last_presentation_failure {
                eprintln!("[cambium-host] unpresented refusal: {failure}");
            }
            eprintln!(
                "[cambium-host] unpresented redraw visible={:?} focused={} minimized={:?}",
                window.is_visible(),
                window.has_focus(),
                window.is_minimized()
            );
        }
        self.with_ctx(Hook::AfterFrame);
        self.sync_app_frame_extents();
        self.run_window_commands();
    }

    pub(crate) fn drain_a11y_action_turn(&mut self) {
        // An AT gesture against the accepted tree remains serviceable during
        // native reception. No projection, frame hook or frame epilogue runs.
        // This is not an AfterFrame sample; do not amend the previous sample.
        let profile = self.s.last_frame_profile.take();
        self.core.drain_a11y_actions();
        self.s.last_frame_profile = profile;
        self.sync_app_frame_extents();
        self.run_window_commands();
    }
}
