// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What the session's replay check compares (engine split D156): the kernel's
//! replay state, read through the kernel graph's `SessionGraph` impl (D241).

#[cfg(test)]
pub(crate) use kernel::graph::replay_state::{Decided, replay_bytes, replay_equal, state_bytes};

#[cfg(test)]
#[path = "replay_state_tests.rs"]
mod tests;
