// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The goal surface's retained pages: the provider over what Graphshell holds,
//! the graph facts it reads, the door's wire form and the shared fixture.
//! The viewer stays in the Graphshell app (engine split D91).

mod facts;
pub mod door;
pub mod fixture;
mod provider;
#[cfg(test)]
mod door_tests;

pub use facts::{ConceptFact, GraphFacts};
pub use provider::{LoadBounds, Origin, PagesProvider, RetainedPage, RetainedPages, record_text, resource_of, source_id};
