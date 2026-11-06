// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Browser selection precedes application initialization.
pub mod boot;
#[cfg(feature = "webgl")]
mod readback;
#[cfg(feature = "webgl")]
pub mod webgl;
