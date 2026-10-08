/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A web module's static constructors, run exactly once.
//!
//! wasm-ld links a module that never calls `__wasm_call_ctors` itself
//! "command-style": it wraps every export so that each JS-to-wasm call runs
//! every static constructor first. Pre.4 CubeCL brings thousands of them
//! (Pliron's `inventory` registrations), so a page re-ran them on every call
//! and its frames took half a second. Calling `__wasm_call_ctors` from the
//! module's start makes the link "reactor-style" instead: no wrappers, and the
//! constructors run when this function says so.
//!
//! The guard matters as much as the call. A second run re-submits every
//! `inventory` node onto its registry, which makes the node point at itself,
//! so iterating that registry never ends. Burn migration plan §13.33 has the
//! measurements; rulings 532 and 536 put the helper here.

use core::sync::atomic::{AtomicBool, Ordering};

static RAN: AtomicBool = AtomicBool::new(false);

/// Run the module's static constructors, once per instance.
///
/// Call it first from the module's `#[wasm_bindgen(start)]`, before any other
/// Rust code. Later calls do nothing.
pub fn run_static_constructors_once() {
    if RAN.swap(true, Ordering::AcqRel) {
        return;
    }
    unsafe extern "C" {
        fn __wasm_call_ctors();
    }
    // SAFETY: the linker-synthesized constructor list. The guard above makes
    // this the only call, and the start function runs before every export.
    unsafe { __wasm_call_ctors() }
}
