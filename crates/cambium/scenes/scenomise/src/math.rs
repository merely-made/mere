// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The solvers' transcendentals, through `libm`.
//!
//! `f32::sin` and its kin call the platform's maths library, and those differ
//! in the last bits: macOS, glibc and the Windows runtime round some
//! arguments differently, most visibly the large angles of a 500-item spiral.
//! An arrangement is closed-form (a score in, the same positions out), so its
//! output should not depend on which machine solved it. `libm` is plain Rust,
//! and gives the same bits on every target, wasm included (the stack seams
//! plan's ruling S4, "cross-platform", and the workspace's `libm` entry).

pub(crate) fn sin(x: f32) -> f32 {
    libm::sinf(x)
}

pub(crate) fn cos(x: f32) -> f32 {
    libm::cosf(x)
}

/// `(sin, cos)`, as `f32::sin_cos` returns them.
pub(crate) fn sin_cos(x: f32) -> (f32, f32) {
    libm::sincosf(x)
}

pub(crate) fn ln(x: f32) -> f32 {
    libm::logf(x)
}
