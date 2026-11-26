// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! The one rule for assertion handles over persisted edge rows: a handle
//! names one assertion on one row, across both strata. A repeat is refused
//! whether or not the copy is identical. Checked load and both freezes ask
//! here; none of them decides legality on its own.

pub use mere::model::assertion_handles::{
    AssertionHandleRepeat, AssertionHandles, HandleRow, validate_assertion_handles,
};
