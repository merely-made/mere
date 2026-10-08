// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Tabard's appearance workshop: one retained product model and Cambium
//! surface for standalone and embedding hosts.

mod state;
mod surface;

pub use state::{SeedRole, WorkshopState};
pub use surface::{WORKSHOP_CSS, WorkshopView, workshop_view};
