/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The one-run fixture for [`run_static_constructors_once`]: a module with one
//! counting static constructor, a start function that calls the helper, and an
//! export that calls it again. `run.mjs` beside this loads it in Node and
//! requires exactly one constructor run.
//!
//! ```text
//! cargo build -p cambium-genet-web-host --example ctor_once --target wasm32-unknown-unknown
//! wasm-bindgen --target web --out-dir <dir> \
//!     target/wasm32-unknown-unknown/debug/examples/ctor_once.wasm
//! node crates/cambium/cambium-genet-web-host/examples/ctor_once/run.mjs <dir>
//! ```
//!
//! [`run_static_constructors_once`]: cambium_genet_web_host::run_static_constructors_once
#![cfg(target_arch = "wasm32")]

use core::sync::atomic::{AtomicU32, Ordering};

use cambium_genet_web_host::run_static_constructors_once;
use wasm_bindgen::prelude::*;

static RUNS: AtomicU32 = AtomicU32::new(0);

extern "C" fn count_run() {
    RUNS.fetch_add(1, Ordering::Relaxed);
}

/// One static constructor that counts its own runs.
#[used]
#[unsafe(link_section = ".init_array")]
static COUNT_RUNS: extern "C" fn() = count_run;

#[wasm_bindgen(start)]
pub fn start() {
    run_static_constructors_once();
}

/// How many times the module's constructors have run.
#[wasm_bindgen]
pub fn ctor_runs() -> u32 {
    RUNS.load(Ordering::Relaxed)
}

/// Call the helper a second time, as a careless caller might.
#[wasm_bindgen]
pub fn run_helper_again() {
    run_static_constructors_once();
}

/// An ordinary export with a string argument, as page glue calls them.
#[wasm_bindgen]
pub fn echo(text: String) -> String {
    text
}
