// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mere's composed graph surface.
//!
//! Applications depend on this crate for graph truth and its portable
//! projections. It deliberately does not expose browser-window, input, render,
//! or session-host concerns; those belong to a consumer such as Turnstone.
//!
//! Capability features keep that facade honest:
//!
//! - `graph`: portable graph and domain vocabulary;
//! - `linked-data`: semantic import/export;
//! - `canvas`: Mere's graph-aware visual surface;
//! - `workbench`: app-host composition.

#[cfg(feature = "graph")]
pub use incipit::forme;
#[cfg(feature = "canvas")]
pub use pictograph::canvas;
#[cfg(feature = "graph")]
pub mod coupling_edit;
#[cfg(feature = "graph")]
pub use kernel;
// Turnstone still names `mere::navigation`; retire with its repoint.
#[cfg(feature = "graph")]
pub use incipit::navigation;
#[cfg(feature = "linked-data")]
pub use linked_data;
#[cfg(feature = "workbench")]
pub use platen;
#[cfg(feature = "workbench")]
pub use workbench;

#[cfg(feature = "clip")]
pub mod clip;
