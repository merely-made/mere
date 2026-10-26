// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Athanor's proposal-producing furnace and persistent evidence-brief workshop.
//! The default furnace API is retained; a workshop host can select only
//! `workshop`, adding `workshop-worker` for native Armillary execution.

#[cfg(feature = "furnace")]
mod furnace;
#[cfg(feature = "furnace")]
pub use furnace::*;
#[cfg(feature = "workshop")]
pub mod workshop;
