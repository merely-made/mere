// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The application side of Djinn's first-party door, as Graphshell uses it.
//!
//! This is the half of the old `graphshell-endpoint` that needs Mere's custody
//! and identity stack (Ostiary, Castellan, Dramatis, Personae). The projection
//! protocol itself, the endpoint traits and the carriers, lives in the
//! Graphshell repository (`graphshell-endpoint`), which this crate depends on
//! and which depends on nothing here (cycle breaks plan, D9).
//!
//! Each module is behind one feature, native only. The default feature set is
//! empty: `projection-door` (the projection client and carrier over the
//! application connection, and the endpoint that forwards over it),
//! `projection-offer` (the portable messages for the provider route),
//! `projection-provider` (the native runner for a projection offer) and
//! `blob-door` (the client and wire of the resident's blob custody route),
//! `profile` (the Graphshell application's identity, as Djinn releases it).
//! Projection and identity adapters use the independent Djinn door; native
//! adapters are unavailable on Wasm.

#[cfg(feature = "declaration")]
pub mod declaration;
#[cfg(all(feature = "profile", any(unix, windows), not(target_arch = "wasm32")))]
pub mod profile;
#[cfg(all(
    feature = "projection-door",
    any(unix, windows),
    not(target_arch = "wasm32")
))]
pub mod projection;
#[cfg(all(
    feature = "blob-door",
    any(unix, windows),
    not(target_arch = "wasm32")
))]
pub mod blob_door;
#[cfg(all(
    feature = "blob-serve",
    any(unix, windows),
    not(target_arch = "wasm32")
))]
pub mod blob_door_source;
#[cfg(feature = "projection-offer")]
pub mod projection_offer;
#[cfg(all(
    feature = "projection-provider",
    any(unix, windows),
    not(target_arch = "wasm32")
))]
pub mod projection_provider;
#[cfg(all(
    feature = "projection-door",
    any(unix, windows),
    not(target_arch = "wasm32")
))]
pub mod resident_mere_proxy;
