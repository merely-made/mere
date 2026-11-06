// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn cumulative_rounding_preserves_every_native_wall_interval() {
    let spans = [1_399, 2_799, 3_899, 0, 2_777].map(Duration::from_nanos);
    let rounded = rounded_partition(spans);
    assert_eq!(rounded, [1, 3, 4, 0, 2]);
    assert_eq!(
        rounded.iter().sum::<u64>(),
        spans.iter().sum::<Duration>().as_micros() as u64
    );
    for (span, rounded) in spans.into_iter().zip(rounded) {
        assert!(rounded.abs_diff(span.as_micros() as u64) <= 1);
    }
}

#[test]
fn missing_partition_is_distinct_from_measured_zero_intervals() {
    assert!(!FrameProfile::default().native_timing_available);
    assert_eq!(rounded_partition([Duration::ZERO; 5]), [0; 5]);
}

#[test]
fn long_ready_to_collect_does_not_become_worker_or_ui_work() {
    let spans = [10, 2, 3, 80_000, 7].map(Duration::from_micros);
    assert_eq!(rounded_partition(spans), [10, 2, 3, 80_000, 7]);
}
