// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mere's diagnostics channels and apparatus observations, one feature each.
//!
//! These were nine `register-*` crates until 2026-09-23, then one crate of
//! registries. The theme registry moved to tabard on 2026-09-24 and the viewer
//! registry retired into inker on 2026-10-27. On 2026-11-08 (engine split D14)
//! the five never-enabled registries were deleted and the mod loader moved to
//! document-host; the name stays because Turnstone and the resident use it.

#[cfg(feature = "diagnostics")]
pub mod diagnostics;
#[cfg(feature = "observations")]
pub mod observations;
