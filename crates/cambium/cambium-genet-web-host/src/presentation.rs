// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Browser selection precedes application initialization.
pub mod boot;
#[cfg(feature = "webgl")]
mod readback;
#[cfg(feature = "webgl")]
pub mod webgl;
