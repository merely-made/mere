// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Disjoint native wall intervals. Existing UI stage spans remain subsets.

use crate::{Duration, FrameProfile, Instant};
use genet_render_host::NativeFrameTiming;

/// Round cumulative boundaries so adjacent integer-microsecond intervals
/// retain an exact sum, including sub-microsecond observations.
fn rounded_partition<const N: usize>(spans: [Duration; N]) -> [u64; N] {
    let mut elapsed = Duration::ZERO;
    let mut previous = 0;
    spans.map(|span| {
        elapsed += span;
        let boundary = super::micros(elapsed);
        let interval = boundary - previous;
        previous = boundary;
        interval
    })
}

pub(super) fn finish(
    profile: &mut FrameProfile,
    timing: NativeFrameTiming,
    prepared: Instant,
    finished: Instant,
) {
    let spans = [
        timing.sealed_at().duration_since(prepared),
        timing.queue(),
        timing.worker(),
        timing.ready_to_collect(),
        finished.duration_since(timing.collected_at()),
    ];
    let [preseal, queue, worker, ready, postcollect] = rounded_partition(spans);
    profile.native_timing_available = true;
    profile.native_preseal_us = preseal;
    profile.native_queue_us = queue;
    profile.native_worker_us = worker;
    profile.native_ready_to_collect_us = ready;
    profile.native_postcollect_us = postcollect;
    profile.native_timeline_us = preseal + queue + worker + ready + postcollect;
    profile.native_latency_us = super::micros(finished.duration_since(prepared));
    debug_assert_eq!(profile.native_timeline_us, profile.native_latency_us);
}

#[cfg(test)]
mod tests;
