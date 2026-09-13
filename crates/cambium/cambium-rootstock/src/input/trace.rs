// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::Instant;
use std::{cell::Cell, sync::OnceLock};

thread_local! {
    static LAST_INPUT_END: Cell<Option<Instant>> = const { Cell::new(None) };
}

/// Input work happens before the redraw span. Keep it visible in the same
/// opt-in log, including the gap since the previous traced input completed.
pub(super) struct InputSpan {
    kind: &'static str,
    start: Instant,
    at_us: u128,
    gap_us: u128,
}

impl InputSpan {
    pub(super) fn start(kind: &'static str) -> Option<Self> {
        static ENABLED: OnceLock<bool> = OnceLock::new();
        if !*ENABLED.get_or_init(|| std::env::var_os("CAMBIUM_HOST_PERF_TRACE").is_some()) {
            return None;
        }
        static BASE: OnceLock<Instant> = OnceLock::new();
        let base = *BASE.get_or_init(Instant::now);
        let start = Instant::now();
        let gap_us = LAST_INPUT_END.with(|last| {
            last.get()
                .map_or(0, |last| start.duration_since(last).as_micros())
        });
        Some(Self {
            kind,
            start,
            at_us: start.duration_since(base).as_micros(),
            gap_us,
        })
    }
}

impl Drop for InputSpan {
    fn drop(&mut self) {
        let end = Instant::now();
        LAST_INPUT_END.with(|last| last.set(Some(end)));
        eprintln!(
            "[cambium-host] input kind={} at={}us gap={}us total={}us",
            self.kind,
            self.at_us,
            self.gap_us,
            end.duration_since(self.start).as_micros()
        );
    }
}
