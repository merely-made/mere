// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! **djinn-testkit**: one tested way to run djinn residents under test
//! (`design_docs/mere_docs/implementation_strategy/2026-10-05_djinn_test_harness_plan.md`).
//!
//! A [`Run`] takes the machine-wide receipt lock, samples load, and captures
//! the walls around the installed resident. Each [`Resident`] it hands out
//! lives under the run's directory with its own roots and endpoints, is
//! spawned through the [`guard`] into a kill-on-close job, and is observed
//! through its status route and event file rather than its log. Finishing
//! the run re-checks the walls and writes a `mere.djinn.receipt/v1` record
//! that [`receipt::verify`] recomputes.
//!
//! Unpublished, and free of djinn, castellan and personae, so any of them
//! can take it as a dev-dependency; tests hand it djinn's binary path.

pub mod guard;
pub mod load;
pub mod receipt;
pub mod resident;
pub mod run;
pub mod status;
pub mod sys;
pub mod walls;

pub use receipt::{Bound, Receipt, verify};
pub use resident::{Resident, Unlock};
pub use run::{Controls, HarnessError, Run, RunConfig};
pub use status::{ResidentEvent, ResidentStatus};

/// Wall-clock milliseconds since the Unix epoch.
pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_millis() as u64)
        .unwrap_or(0)
}
