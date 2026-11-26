// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The engine journal (D249): a muniment log of a graph's attributed deltas.
//!
//! The materialized graph is the replay of this log. What an edit reads from
//! the clock or mints is journaled beside it, so replay reproduces it too. A
//! checkpoint is a stored snapshot plus [`replay_from`](SessionJournal::replay_from)
//! over the tail past its cursor.

use muniment::{Backend, Codec, Journal, LogId, Provenance, Seq, SlotStore, StoreError};

use super::{OnRefusal, ReplayRefused, SessionGraph};
use crate::model::{Attributed, Author};

/// An append-only journal of a graph's attributed edits: one log, one cursor.
pub struct SessionJournal<G: SessionGraph> {
    pub(super) log: Journal<Attributed<G::Delta>>,
    /// The author the next [`record`](Self::record) attributes:
    /// [`Author::user`] by default; a host scopes a participant run with
    /// [`set_author`](Self::set_author) and restores it afterwards.
    author: Author,
}

impl<G: SessionGraph> Clone for SessionJournal<G> {
    fn clone(&self) -> Self {
        Self {
            log: self.log.clone(),
            author: self.author.clone(),
        }
    }
}

impl<G: SessionGraph> std::fmt::Debug for SessionJournal<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionJournal")
            .field("log", &self.log)
            .field("author", &self.author)
            .finish()
    }
}

impl<G: SessionGraph> Default for SessionJournal<G> {
    fn default() -> Self {
        Self::from_log(Journal::default())
    }
}

impl<G: SessionGraph> SessionJournal<G> {
    /// A fresh, empty journal, recording as [`Author::user`].
    pub fn new() -> Self {
        Self::default()
    }

    /// A journal with a stable identity, so it can later be forked with
    /// provenance pointing back at it.
    pub fn with_id(id: LogId) -> Self {
        Self::from_log(Journal::with_id(id))
    }

    /// An empty journal that starts where `provenance` says another left off,
    /// without copying its entries: the keeper holds the graph they built as a
    /// snapshot. A session forked at a cursor starts this way.
    pub fn starting_from(id: LogId, provenance: Provenance) -> Self {
        Self::from_log(Journal::starting_from(id, provenance))
    }

    /// Adopt an existing journal of attributed deltas (e.g. one just loaded
    /// from a store or received from a peer).
    pub fn from_log(log: Journal<Attributed<G::Delta>>) -> Self {
        Self {
            log,
            author: Author::user(),
        }
    }

    /// The author subsequent [`record`](Self::record)s attribute.
    pub fn author(&self) -> &Author {
        &self.author
    }

    /// Scope the recording author (a participant run); the host restores the
    /// previous one when the run ends.
    pub fn set_author(&mut self, author: Author) {
        self.author = author;
    }

    /// Append one edit under the current author, returning the [`Seq`] it was
    /// stamped with.
    pub fn record(&mut self, delta: G::Delta) -> Seq {
        let author = self.author.clone();
        self.record_as(author, delta)
    }

    /// Append one edit under an explicit author (the gate path).
    pub fn record_as(&mut self, author: Author, delta: G::Delta) -> Seq {
        self.log.append(Attributed { author, delta })
    }

    /// The underlying journal, for cursors, replication, and persistence.
    pub fn log(&self) -> &Journal<Attributed<G::Delta>> {
        &self.log
    }

    /// Every attributed edit, oldest first.
    pub fn entries(&self) -> &[Attributed<G::Delta>] {
        self.log.entries()
    }

    pub fn len(&self) -> usize {
        self.log.len()
    }

    pub fn is_empty(&self) -> bool {
        self.log.is_empty()
    }

    /// The [`Seq`] the next recorded edit will receive: a durable cursor a
    /// checkpoint can store to resume replay from.
    pub fn next_seq(&self) -> Seq {
        self.log.next_seq()
    }

    /// The earliest source-time cursor: the empty graph before the first
    /// delta. Journal source time is a prefix cursor, so this is `Seq(0)`
    /// even though no entry is addressed there yet.
    pub const fn earliest_cursor(&self) -> Seq {
        Seq(0)
    }

    /// The current live cursor: the prefix just after the newest delta.
    /// Appending later deltas never changes the graph an earlier cursor
    /// replays to.
    pub fn live_cursor(&self) -> Seq {
        self.next_seq()
    }

    /// Graph truth at a prefix cursor, leaving the journal as it is. `Seq(n)`
    /// replays exactly the first `n` entries. A cursor past the known prefix
    /// is stale and refused (`None`), never clamped to live.
    pub fn snapshot_at(&self, cursor: Seq) -> Result<Option<G>, ReplayRefused> {
        self.snapshot_at_from(&G::default(), cursor)
    }

    /// Replay a prefix over its retained baseline, keeping what the baseline
    /// carries.
    pub fn snapshot_at_from(&self, baseline: &G, cursor: Seq) -> Result<Option<G>, ReplayRefused> {
        let end = cursor.index();
        if end > self.log.len() {
            return Ok(None);
        }
        let mut graph = baseline.clone();
        let mut carry = G::carry_from(&graph);
        graph.replay_carried(&mut carry, &self.log.entries()[..end], OnRefusal::Stop)?;
        Ok(Some(graph))
    }

    /// Stable, evenly distributed prefix cursors for a compact scrubber, by
    /// sequence order alone. `max_points == 1` yields only the live cursor;
    /// larger requests include both earliest and live when distinct.
    pub fn sequence_ticks(&self, max_points: usize) -> Vec<Seq> {
        let live = self.live_cursor();
        if max_points == 0 {
            return Vec::new();
        }
        if max_points == 1 || live.0 == 0 {
            return vec![live];
        }
        let points = max_points.min(live.0 as usize + 1);
        let denominator = (points - 1) as u64;
        (0..points)
            .map(|index| Seq((index as u64 * live.0) / denominator))
            .collect()
    }

    pub fn id(&self) -> Option<&LogId> {
        self.log.id()
    }

    /// Where this journal forked from, if it is a fork.
    pub fn provenance(&self) -> Option<&Provenance> {
        self.log.provenance()
    }

    /// Rebuild the whole graph. A refused entry is dropped and the rebuild
    /// goes on; checked readers use [`snapshot_at`](Self::snapshot_at).
    pub fn replay(&self) -> G {
        let mut graph = G::default();
        let mut carry = G::carry_from(&graph);
        let _ = graph.replay_carried(&mut carry, self.log.entries(), OnRefusal::Drop);
        graph
    }

    /// Advance a materialized `graph` by the edits from `since` onward: the
    /// checkpoint-plus-tail twin of [`replay`](Self::replay). Without a
    /// baseline nothing is carried;
    /// [`replay_from_with_baseline`](Self::replay_from_with_baseline) carries
    /// what the prefix held.
    pub fn replay_from(&self, since: Seq, graph: &mut G) -> Result<(), ReplayRefused> {
        graph.replay_tail(self.log.from(since))
    }

    /// Replay a checkpoint tail with the baseline and the entries before it.
    /// `baseline` is the graph before entry zero, not the checkpoint at `since`.
    pub fn replay_from_with_baseline(
        &self,
        since: Seq,
        graph: &mut G,
        baseline: &G,
    ) -> Result<(), ReplayRefused> {
        let mut carry = G::carry_from(baseline);
        // The prefix replays on one shadow graph, once: a clone per entry
        // would make checkpoint replay quadratic.
        let mut shadow = baseline.clone();
        let prefix = &self.log.entries()[..since.index().min(self.log.len())];
        shadow.replay_carried(&mut carry, prefix, OnRefusal::Stop)?;
        G::repair_checkpoint(&carry, graph);
        graph.replay_carried(&mut carry, self.log.from(since), OnRefusal::Stop)
    }

    /// Fork under a new identity: the whole history, with where it branched.
    pub fn fork(&self, new_id: LogId) -> Self {
        Self {
            log: self.log.fork(new_id),
            author: self.author.clone(),
        }
    }

    /// Write the whole journal to the muniment slot at `key`.
    pub async fn save<B: Backend, C: Codec>(
        &self,
        slots: &SlotStore<B, C>,
        key: &str,
    ) -> Result<(), StoreError> {
        self.log.save(slots, key).await
    }

    /// Load a journal from the slot at `key`, or an empty one if it is absent.
    pub async fn load<B: Backend, C: Codec>(
        slots: &SlotStore<B, C>,
        key: &str,
    ) -> Result<Self, StoreError> {
        Ok(Self::from_log(Journal::load(slots, key).await?))
    }
}
