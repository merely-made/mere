// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Installed presentation drivers share the host's observation and input order.
use super::*;
use crate::{Host, HostTree, meristem_bounds::RootView};
fn micros(duration: crate::Duration) -> u64 {
    duration.as_micros().min(u64::MAX as u128) as u64
}
impl<State, Logic, V, T> Host<State, Logic, V, T>
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
    T: HostTree<State>,
{
    pub(crate) fn present_with_driver(&mut self, prepared: PreparedFrame) {
        let expected = prepared.presentation_stamp();
        let result = self
            .s
            .presentation_driver
            .as_mut()
            .expect("installed driver")
            .present(&prepared);
        let presented = match result {
            Ok(Some(stamp)) if stamp == expected => stamp,
            Ok(Some(_)) => {
                let driver = self
                    .s
                    .presentation_driver
                    .as_ref()
                    .expect("installed driver");
                self.refuse_presentation(PresentationFailure {
                    backend: driver.backend(),
                    generation: prepared.generation,
                    producer_key: None,
                    message: "driver returned a foreign presentation identity".into(),
                    native: None,
                });
                return;
            },
            Ok(None) => {
                let mut profile = prepared.profile;
                profile.total_us = micros(prepared.started.elapsed());
                self.s.last_frame_profile = Some(profile);
                return;
            },
            Err(failure) => {
                self.refuse_presentation(failure);
                return;
            },
        };
        self.s.presentation_sequence = presented.sequence;
        self.s.last_redraw_presentation = Some(presented);
        self.observe_presentation(presented);
        let phase = crate::Instant::now();
        if let Some(capture) = self.s.pending_paint_capture.take() {
            let envelope = match &prepared.paint {
                PreparedPaint::Native(envelope) => envelope.clone(),
                PreparedPaint::Portable(capture) => capture.envelope.clone(),
            };
            capture(Ok(envelope));
        }
        // Borrowed native capture hooks cannot represent another backend.
        // The owned capture migration replaces this explicit refusal.
        if self.s.pending_capture.is_some() || self.s.pending_stamped_capture.is_some() {
            let driver = self
                .s
                .presentation_driver
                .as_ref()
                .expect("installed driver");
            self.refuse_pending_captures(crate::CaptureRefusal::PresentationError {
                backend: driver.backend(),
                generation: presented.context_generation,
                source_key: None,
                error: "borrowed native capture is unavailable on this driver".into(),
            });
        }
        let mut profile = prepared.profile;
        profile.capture_us = micros(phase.elapsed());
        if prepared.animating
            && let Some(window) = self.s.window.as_ref()
        {
            window.request_redraw();
        }
        let phase = crate::Instant::now();
        self.drain_pointer();
        profile.pointer_us = micros(phase.elapsed());
        profile.total_us = micros(prepared.started.elapsed());
        self.s.last_frame_profile = Some(profile);
    }
}
