// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! # chrome
//!
//! Host-neutral browser chrome state: what the location bar's text means
//! ([`nav`]), the suggestions offered for it ([`suggest`]), the omnibar's
//! search session ([`omnibar`]), the toolbar's draft and load state
//! ([`toolbar`]), the focus and graph-search mutation bundles
//! ([`authorities`]) and where focus returns when an overlay closes
//! ([`routing`]). Rendering and effects stay with the host.
//!
//! The frame model, its effects and the host-intent shape retired with the
//! pre-pivot shell they served (engine-split D30). Every module is
//! WASM-clean (no egui, no servo, no tokio, no platform I/O) and testable
//! without booting a host.

#![doc(html_root_url = "https://docs.rs/chrome/0.0.1")]

pub mod authorities;
pub mod nav;
pub mod omnibar;
pub mod routing;
pub mod suggest;
pub mod toolbar;

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Lifecycle stage marker.
pub const STAGE: &str = "pre-alpha";
