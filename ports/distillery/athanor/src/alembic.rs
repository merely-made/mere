// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Alembic, the workshop's host-neutral half, folded into Athanor
//! (engine split D56; the `mere-alembic` crate is freed). `workshop` holds the
//! shared records and typed controls, `workshop-surface` the Cambium surface
//! over them and `workshop-native` its desktop host.

pub mod workshop;

#[cfg(feature = "workshop-surface")]
pub mod workshop_surface;

#[cfg(all(feature = "workshop-native", not(target_arch = "wasm32")))]
pub mod workshop_native;
