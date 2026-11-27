// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The wall clock for assertion records and session stamps, on native and in
//! browsers, where `SystemTime::now()` panics.

/// Milliseconds since the Unix epoch, or `0` before it.
pub fn unix_epoch_millis() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

/// The wall clock as a `std::time::SystemTime`, for crates that persist one
/// and may run in a browser, as mere-kernel's `time` had it (engine split
/// lane 4).
pub fn wall_clock_now() -> std::time::SystemTime {
    let since = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default();
    std::time::UNIX_EPOCH + since
}
