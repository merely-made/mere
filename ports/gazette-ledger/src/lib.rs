// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The contact ledger projection.
//!
//! Gazette's directory domain, WebFinger discovery and the intake that hands
//! what it learns to `gaz`, lives in dramatis (`gazette`, 0.0.2) and links
//! nothing from mere. What stays here is the Cambium side: the ledger that
//! projects contacts into a scene over `chirograph`, `incipit` and `sceno`. It
//! moved out of `gazette` on 2026-11-03 so dramatis would not pin mere
//! (dramatis repo plan, D13); the reading-room article extractor went to
//! `mere-fleece`, behind its `article` feature.

pub mod ledger;
