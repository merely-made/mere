// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Wasm-bindgen wrapper for the same occurrence replay implementation.

#[path = "support/occurrence_replay.rs"]
mod occurrence_replay;

use pictograph::canvas::dynamics_recipe::bind_slot;
use pictograph::canvas::projection_dynamics::ProjectionDynamics;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn run_occurrence_replay(input: &str) -> Result<String, JsError> {
        super::occurrence_replay::run(input).map_err(|error| JsError::new(&error))
    }

    #[wasm_bindgen]
    pub fn canonical_occurrence_case() -> Result<String, JsError> {
        super::occurrence_replay::canonical_case().map_err(|error| JsError::new(&error))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("occurrence_replay_wasm is a wasm32 wrapper; use occurrence_replay for native runs");
}
