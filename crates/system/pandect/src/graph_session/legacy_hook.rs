// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The legacy translation, as a session over any graph reaches it (engine
//! split D252). A [`LegacyTranslationReceipt`] holds kernel deltas and frozen
//! kernel graphs, so only the kernel-specialised open and qualification install
//! [`KERNEL`]; a session without the hook refuses a stored receipt.

use super::*;

/// The receipt's reads, over the session's graph type.
pub(super) struct LegacyHook<G: SessionGraph> {
    pub(super) validate: fn(
        &LegacyTranslationReceipt,
        &SessionPlacement,
        &str,
        &[Attributed<G::Delta>],
    ) -> Result<(), String>,
    pub(super) graph_at:
        fn(&LegacyTranslationReceipt, &[Attributed<G::Delta>], Seq) -> Result<Option<G>, String>,
    pub(super) journal:
        fn(&LegacyTranslationReceipt, &[Attributed<G::Delta>], Seq) -> Option<SessionJournal<G>>,
    pub(super) baseline: fn(&LegacyTranslationReceipt) -> Result<G, String>,
    pub(super) same_graph: fn(&G, &G) -> bool,
    pub(super) entry_effects: fn(&LegacyTranslationReceipt, usize) -> Option<Vec<G::Delta>>,
}

/// The kernel graph's hook: the receipt's own methods.
pub(super) static KERNEL: LegacyHook<Graph> = LegacyHook {
    validate: |receipt, placement, digest, entries| receipt.validate(placement, digest, entries),
    graph_at: |receipt, entries, cursor| receipt.graph_at(entries, cursor),
    journal: |receipt, entries, cursor| receipt.journal(entries, cursor),
    baseline: |receipt| receipt.baseline.materialize(),
    same_graph: |a, b| FrozenGraph::of(a) == FrozenGraph::of(b),
    entry_effects: |receipt, index| receipt.entry_effects.get(index).cloned(),
};
