// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! First-party application door shared with the Djinn resident.
//!
//! The `door` feature exposes portable admission labels, route policy, wire
//! envelopes and synchronous framing. `door-native` adds the shared local
//! connection; `door-serve` adds the native listener and serving machinery.
//! `custody`, `custody-client`, `projection-door` and `profile` select their
//! respective types and clients on native Unix and Windows. `projection-offer`
//! adds portable provider messages; `projection-provider` adds their native runner.
//! The Djinn broker
//! owns the serving policy and session loops.
//!
//! A local socket or named pipe establishes a same-user boundary. Application
//! labels are self-declared inputs to route policy, not proof of executable
//! identity.

pub mod app_admission;
#[cfg(all(
    feature = "door-native",
    any(unix, windows),
    not(target_arch = "wasm32")
))]
pub mod app_client;
#[cfg(all(feature = "custody", any(unix, windows), not(target_arch = "wasm32")))]
pub mod custody;
#[cfg(all(
    feature = "custody-client",
    any(unix, windows),
    not(target_arch = "wasm32")
))]
pub mod custody_client;
#[cfg(all(
    feature = "custody-client",
    any(unix, windows),
    not(target_arch = "wasm32")
))]
pub mod custody_identity;
#[cfg(all(feature = "custody", any(unix, windows), not(target_arch = "wasm32")))]
pub mod custody_payload;
pub mod frame;
#[cfg(all(feature = "custody", any(unix, windows), not(target_arch = "wasm32")))]
pub mod identity;
#[cfg(all(
    feature = "door-native",
    any(unix, windows),
    not(target_arch = "wasm32")
))]
pub mod local_endpoint;
#[cfg(all(feature = "profile", any(unix, windows), not(target_arch = "wasm32")))]
pub mod profile;
#[cfg(all(
    feature = "projection-door",
    any(unix, windows),
    not(target_arch = "wasm32")
))]
pub mod projection;
#[cfg(all(feature = "custody", any(unix, windows), not(target_arch = "wasm32")))]
pub mod quarantine;
#[cfg(all(
    feature = "projection-door",
    any(unix, windows),
    not(target_arch = "wasm32")
))]
pub mod resident_mere_proxy;
pub mod route;
pub mod wire;

#[cfg(feature = "projection-offer")]
pub mod projection_offer;
#[cfg(all(feature = "projection-provider", any(unix, windows), not(target_arch = "wasm32")))]
pub mod projection_provider;
