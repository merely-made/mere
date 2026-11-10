// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Retention: the store-side memory program, bound for Eidetic (engine split
//! D45). It names no kernel and no pandect type, only Eidetic's store, so the
//! whole directory can move as it is:
//!
//! - [`levels`]: short-term and long-term memory over any node record, and the
//!   eviction policy;
//! - [`forgetting`]: applying a forget proposal (dropping cached content);
//! - [`retirement`]: proposing and applying the purge of aged tombstones.
//!
//! The graph-reading halves stay in pandect as thin adapters:
//! `memory_levels` reads the graph's nodes into [`MemoryRecord`]s, and the
//! furnace's `propose_forgetting` maps evictable node ids to urls.

pub mod forgetting;
pub mod levels;
pub mod retirement;

pub use forgetting::{ForgetProposal, apply_forgetting};
pub use levels::{
    EvictionPolicy, MemoryCensus, MemoryLevel, MemoryRecord, census, evictable_short_term,
    level_of,
};
pub use retirement::{RetirementProposal, apply_retirement, propose_retirement};
