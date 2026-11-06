// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Wasm-bindgen wrapper for direct Canvas schedule replay.

#[path = "support/schedule_replay.rs"]
mod schedule_replay;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn run_schedule_replay(input: &str) -> Result<String, JsError> {
        super::schedule_replay::run(input).map_err(|error| JsError::new(&error))
    }

    #[wasm_bindgen]
    pub fn canonical_schedule_case() -> Result<String, JsError> {
        super::schedule_replay::canonical_case().map_err(|error| JsError::new(&error))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("schedule_replay_wasm is a wasm32 wrapper; use schedule_replay for native runs");
}
