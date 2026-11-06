// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Isolated source consumer: no Mere renderer/browser or unrelated port devdeps.
#[path = "../../../ports/moot/src/coop.rs"]
pub mod coop;
#[path = "../../../ports/moot/src/ocapn_coop.rs"]
pub mod ocapn_coop;
#[cfg(test)]
mod tests;
