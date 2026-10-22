// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Wasm-bindgen wrapper for the direct Canvas grouped replay.

#[path = "support/grouped_replay.rs"]
mod grouped_replay;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn run_grouped_replay(input: &str) -> Result<String, JsError> {
        super::grouped_replay::run(input).map_err(|error| JsError::new(&error))
    }

    #[wasm_bindgen]
    pub fn canonical_grouped_case() -> Result<String, JsError> {
        super::grouped_replay::canonical_case().map_err(|error| JsError::new(&error))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("grouped_replay_wasm is a wasm32 wrapper; use grouped_replay for native runs");
}
