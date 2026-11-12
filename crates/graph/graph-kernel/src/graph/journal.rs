// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The edit spine: mere's captured-delta stream over the substrate log.
//!
//! The graph's durable mutations already exist as a stream of [`CapturedDelta`]s
//! (stable-id, serializable), emitted through the capture hook as each live
//! [`GraphDelta`](super::apply::GraphDelta) applies (see `capture.rs`). This module
//! gives that stream its principled home: a [`muniment::Journal`] of captured
//! deltas, the append-only log primitive shared across the Merely apps. The
//! materialized graph is the *replay* of the journal, and because live editing and
//! replay both funnel through `apply_graph_delta`, the two cannot diverge. What an
//! edit reads from the clock or mints (a new node's visit stamp, a statement id)
//! is journaled beside it, so replay reproduces those too.
//!
//! This is the edit spine over the substrate. mere keeps its own rich edit
//! vocabulary — `CapturedDelta` carries content edits (title, tags, body,
//! navigation) that chartulary's topology-only `GraphEdit` deliberately cannot
//! express, and muniment supplies the ordered, forkable, persistable log beneath
//! it. Checkpointing reuses mere's existing `GraphSnapshot`: load a snapshot, then
//! [`replay_from`](GraphJournal::replay_from) the journal tail past its sequence,
//! mirroring `chartulary::GraphLog::load_checkpointed`.
//!
//! WASM-clean: muniment is the substrate's portable persistence primitive (the
//! same ones the browser persists through), so this module compiles to
//! `wasm32-unknown-unknown` like the rest of `graph/`.

use std::sync::{Arc, Mutex};

use muniment::{Backend, Codec, SlotStore, StoreError};
use muniment::{Journal, LogId, Provenance, Seq};

use rkyv::{Archive, Deserialize, Serialize};

use super::Graph;
use super::capture::{
    CapturedDelta, DeltaRecorder, OnRefusal, ReplayAttribution, ReplayRefused,
    replay_attributed_deltas_onto, replay_outcomes_onto,
};
use super::source_time::{SourceExtent, SourceTime};
pub use mere::model::author::{Author, AuthorKind, USER_AUTHOR};

/// One journal entry: a captured delta in the attribution envelope — the
/// participant-gate plan's B1 adoption of chartulary's `Batch { author, edits }`
/// shape over mere's edit spine. WHO made a change rides the journal, so a
/// participant's edits read back attributed and compensable.
#[derive(
    Debug, Clone, PartialEq, Archive, Serialize, Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AttributedDelta {
    pub author: Author,
    pub delta: CapturedDelta,
}

/// An append-only journal of a graph's captured edits: the persisted edit
/// spine. The materialized [`Graph`] is the replay of this log; forking it forks
/// the graph's whole history with provenance, and it persists whole through one
/// muniment slot.
#[derive(Clone, Debug)]
pub struct GraphJournal {
    log: Journal<AttributedDelta>,
    /// The author the next [`record`](Self::record) attributes: [`Author::user`]
    /// by default; a host scopes a participant run with
    /// [`set_author`](Self::set_author) and restores afterwards.
    author: Author,
}

impl Default for GraphJournal {
    fn default() -> Self {
        Self {
            log: Journal::default(),
            author: Author::user(),
        }
    }
}

fn replay_entries_onto<'a, I>(
    graph: &mut Graph,
    entries: I,
    on_refusal: OnRefusal,
) -> Result<(), ReplayRefused>
where
    I: IntoIterator<Item = &'a AttributedDelta>,
{
    let mut attribution = ReplayAttribution::from_baseline(graph);
    replay_attributed_deltas_onto(graph, &mut attribution, entries, on_refusal)
}

impl SourceTime for GraphJournal {
    type Cursor = Seq;
    type Snapshot = Graph;

    fn source_extent(&self) -> SourceExtent<Self::Cursor> {
        SourceExtent {
            earliest: self.earliest_cursor(),
            current: self.live_cursor(),
        }
    }

    fn source_ticks(&self, max_points: usize) -> Vec<Self::Cursor> {
        self.sequence_ticks(max_points)
    }

    fn source_snapshot(&self, cursor: &Self::Cursor) -> Option<Self::Snapshot> {
        self.snapshot_at(*cursor).ok().flatten()
    }
}

/// A shared journal plus a capture hook that records every emitted
/// [`CapturedDelta`] into it. Install the hook with
/// [`set_captured_delta_hook`](super::set_captured_delta_hook) and keep the handle
/// to replay or persist the journal:
///
/// ```ignore
/// let (journal, hook) = journal_capture_hook();
/// kernel::graph::set_captured_delta_hook(Some(hook));
/// // ... drive live mutations through apply_graph_delta ...
/// let restored = journal.lock().unwrap().replay();
/// ```
///
/// The capture hook is one per-thread slot, so this replaces any previously
/// installed hook. It is offered as the host's journal-backed persistence path.
pub fn journal_capture_hook() -> (Arc<Mutex<GraphJournal>>, DeltaRecorder) {
    let journal = Arc::new(Mutex::new(GraphJournal::new()));
    let sink = Arc::clone(&journal);
    let hook: DeltaRecorder = Arc::new(move |delta| {
        if let Ok(mut journal) = sink.lock() {
            journal.record(delta.clone());
        }
    });
    (journal, hook)
}

#[cfg(test)]
#[path = "journal/tests.rs"]
mod tests;

#[path = "journal/operations.rs"]
mod operations;
