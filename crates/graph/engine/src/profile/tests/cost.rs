// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The core pack's boot cost (declared-lens L2). Ignored by default; run in release:
//!
//! ```text
//! cargo test --release -p mere --lib profile::tests::cost -- --ignored --nocapture
//! ```

use std::time::{Duration, Instant};

use super::support::*;
use crate::profile::core::CORE_PACK;
use crate::profile::load::Limits;
use crate::profile::pack::decode;
use crate::profile::resolve::PackRef;

fn median(n: usize, mut f: impl FnMut()) -> Duration {
    let mut runs: Vec<Duration> = (0..n)
        .map(|_| {
            let start = Instant::now();
            f();
            start.elapsed()
        })
        .collect();
    runs.sort();
    runs[n / 2]
}

#[test]
#[ignore = "cost receipt; run in release"]
fn core_pack_boot_cost() {
    let n = 31;
    let limits = Limits::default();
    let decode_pin = median(n, || {
        std::hint::black_box(decode("core", CORE_PACK, &limits).unwrap());
    });
    let load = median(n, || {
        std::hint::black_box(load_with(&PackRef::Core, &Source::default()).unwrap());
    });
    let source = workspace_source();
    let workspace = median(n, || {
        std::hint::black_box(load_with(&workspace_lens(), &source).unwrap());
    });
    eprintln!(
        "core pack, {} canonical bytes, median of {n}: strict decode + BLAKE3 pin {decode_pin:?}, \
         load (decode, verdict, resolve, verify, intern) {load:?}; with the workspace stand-in {workspace:?}",
        CORE_PACK.len()
    );
}
