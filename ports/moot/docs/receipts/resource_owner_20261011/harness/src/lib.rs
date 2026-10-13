// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

// Exercise the actual port and checked-in integration tests. No copied owner,
// UI implementation or synthetic authorization provider lives in this harness.
#[cfg(test)]
#[path = "../../../../../tests/resource_owner.rs"]
mod owner_tests;
