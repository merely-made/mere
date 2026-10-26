// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Reusable Forme workspace and editing component state.
mod model;
mod session;

pub use model::{FormeWorkspace, NoPreferences, RegionCell, WorkspacePreferences, WorkspaceRegion};
pub use session::{
    DropEffect, DropProposal, FormeLoadGate, FormeLoadTicket, FormeSession, FormeWriteFence,
    PreparedSave, SaveKind,
};
