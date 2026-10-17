// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mere's late-binding registries, one module and one feature each.
//!
//! These were nine `register-*` crates until 2026-09-23. They are components
//! of one registry system, so they are modules of one crate: a consumer turns
//! off the default features and enables the registries it uses, and pays for
//! nothing else. `mod-loader` pulls in `diagnostics` and `viewer`, and
//! `viewer` pulls in `layout`. The theme registry moved to tabard on
//! 2026-09-24.

#[cfg(feature = "diagnostics")]
pub mod diagnostics;
#[cfg(feature = "input")]
pub mod input;
#[cfg(feature = "knowledge")]
pub mod knowledge;
#[cfg(feature = "layout")]
pub mod layout;
#[cfg(feature = "lens")]
pub mod lens;
#[cfg(feature = "mod-loader")]
pub mod mod_loader;
#[cfg(feature = "protocol")]
pub mod protocol;
#[cfg(feature = "viewer")]
pub mod viewer;
