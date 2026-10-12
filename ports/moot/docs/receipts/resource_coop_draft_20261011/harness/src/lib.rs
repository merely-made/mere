// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Compile the actual portable port sources with no governance, network, renderer
//! or backend dependencies. This is a source-contract receipt, not host acceptance.
#[path = "../../../../../src/coop.rs"]
pub mod coop;
#[path = "../../../../../src/resource_coop.rs"]
pub mod resource_coop;
