// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The profile stage: the semantic profile's vocabulary. The predicate registry
//! still lives in `mere-kernel` until the model seam (P5 L3).

/// Standard-vocabulary alignment for Mere's recognized relations: the 3-category
/// (exact / approximate / Mere-only) mapping projected as `owl:equivalentProperty`
/// / `rdfs:subPropertyOf` quads.
pub mod vocab;
