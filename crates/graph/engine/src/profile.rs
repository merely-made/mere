// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The profile stage, "load a semantic profile" (declared-lens plan §4): the
//! pack loader over `eidetic::pack`, the embedded core pack, and the standard
//! vocabulary alignment. The predicate registry is `model::predicate_registry`;
//! the core pack reproduces its placement table until L7 retires it.

/// The core pack, embedded, and its generated id constants.
pub mod core;
/// Term interning, axis rows and the core defaults.
pub mod intern;
/// The root lens's choices and the selection record.
pub mod lens;
/// The entry point: [`load::load`].
pub mod load;
/// Strict decoding and declare-and-verify.
pub mod pack;
/// Requirements to pins.
pub mod resolve;
/// Standard-vocabulary alignment for Mere's recognized relations: the 3-category
/// (exact / approximate / Mere-only) mapping projected as `owl:equivalentProperty`
/// / `rdfs:subPropertyOf` quads.
pub mod vocab;

#[cfg(test)]
pub(crate) mod tests;
