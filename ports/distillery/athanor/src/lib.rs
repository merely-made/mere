// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Athanor's persistent evidence-brief workshop: the authority half of
//! Alembic's workshop. `workshop-worker` adds native Armillary execution. The
//! furnace's graph passes moved to `pandect::furnace` 2026-10-31
//! (distillery-extraction D12).

#[cfg(feature = "workshop")]
pub mod workshop;
#[cfg(feature = "workshop")]
pub mod workshop_seal;

#[cfg(all(feature = "workshop-worker", not(target_arch = "wasm32")))]
pub mod workshop_driver;

#[cfg(feature = "workshop-destination")]
pub mod workshop_destination;

#[cfg(all(feature = "workshop-destination", feature = "workshop-sealed"))]
pub mod workshop_sealed_destination;
