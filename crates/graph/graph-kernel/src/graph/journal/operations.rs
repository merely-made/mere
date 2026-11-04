// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl GraphJournal {
    /// A fresh, empty journal, recording as [`Author::user`].
    pub fn new() -> Self {
        Self::default()
    }

    /// A journal with a stable identity, so it can later be forked with
    /// provenance pointing back at it.
    pub fn with_id(id: LogId) -> Self {
        Self {
            log: Journal::with_id(id),
            author: Author::user(),
        }
    }

    /// An empty journal that starts where `provenance` says another left off,
    /// without copying its entries: the keeper holds the graph they built as a
    /// snapshot. A session forked at a cursor starts this way.
    pub fn starting_from(id: LogId, provenance: Provenance) -> Self {
        Self::from_log(Journal::starting_from(id, provenance))
    }

    /// Adopt an existing journal of attributed deltas (e.g. one just loaded
    /// from a store or received from a peer).
    pub fn from_log(log: Journal<AttributedDelta>) -> Self {
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

    /// Append one captured edit under the current author, returning the [`Seq`]
    /// it was stamped with. This is the write path: a live mutation's capture
    /// lands here (usually via [`journal_capture_hook`]).
    pub fn record(&mut self, delta: CapturedDelta) -> Seq {
        let author = self.author.clone();
        self.record_as(author, delta)
    }

    /// Append one captured edit under an explicit author (the gate path).
    pub fn record_as(&mut self, author: Author, delta: CapturedDelta) -> Seq {
        self.log.append(AttributedDelta { author, delta })
    }

    /// The underlying journal, for cursors, replication, and persistence.
    pub fn log(&self) -> &Journal<AttributedDelta> {
        &self.log
    }

    /// Every attributed edit, oldest first.
    pub fn entries(&self) -> &[AttributedDelta] {
        self.log.entries()
    }

    /// The number of edits recorded.
    pub fn len(&self) -> usize {
        self.log.len()
    }

    /// Whether the journal holds no edits.
    pub fn is_empty(&self) -> bool {
        self.log.is_empty()
    }

    /// The [`Seq`] the next recorded edit will receive — a durable cursor a
    /// checkpoint can store to resume replay from.
    pub fn next_seq(&self) -> Seq {
        self.log.next_seq()
    }

    /// The earliest source-time cursor: the empty graph before the first
    /// captured delta. Journal source time is a **prefix cursor**, so this is
    /// intentionally `Seq(0)` even though no entry is addressed there yet.
    pub const fn earliest_cursor(&self) -> Seq {
        Seq(0)
    }

    /// The current live source-time cursor: the prefix immediately after the
    /// newest captured delta. Appending later deltas never changes the graph a
    /// previously returned cursor replays to.
    pub fn live_cursor(&self) -> Seq {
        self.next_seq()
    }

    /// Materialize graph truth at a journal prefix cursor without changing the
    /// journal or its current live graph. `Seq(n)` replays exactly the first
    /// `n` entries; `Seq(0)` is empty and [`live_cursor`](Self::live_cursor)
    /// replays every current entry. A cursor beyond the known prefix is stale
    /// and is refused rather than silently clamped to live.
    pub fn snapshot_at(&self, cursor: Seq) -> Result<Option<Graph>, ReplayRefused> {
        self.snapshot_at_from(&Graph::new(), cursor)
    }

    /// Replay a prefix over its retained baseline, preserving baseline attribution.
    pub fn snapshot_at_from(
        &self,
        baseline: &Graph,
        cursor: Seq,
    ) -> Result<Option<Graph>, ReplayRefused> {
        let end = cursor.index();
        if end > self.log.len() {
            return Ok(None);
        }
        let mut graph = baseline.clone();
        replay_entries_onto(&mut graph, &self.log.entries()[..end], OnRefusal::Stop)?;
        Ok(Some(graph))
    }

    /// Migration of a caller-qualified legacy baseline and prefix.
    /// Ordinary replay does not infer a legacy profile from absent resource columns.
    pub fn migrated_prefix_at_from(
        &self,
        baseline: &Graph,
        cursor: Seq,
    ) -> Result<
        Option<super::super::legacy_resource_migration::MigratedReplay>,
        super::super::legacy_resource_migration::LegacyMigrationError,
    > {
        let end = cursor.index();
        if end > self.log.len() {
            return Ok(None);
        }
        super::super::legacy_resource_migration::migrate_legacy_prefix(
            baseline,
            &self.log.entries()[..end],
        )
        .map(Some)
    }

    /// Validate choices against this complete qualified legacy journal, then read a prefix.
    /// Future carried-handle choices do not change or invalidate earlier prefixes.
    /// A host with a modern suffix must supply only its qualified legacy prefix to the adapter.
    pub fn migrated_prefix_at_from_with_resolutions(
        &self,
        baseline: &Graph,
        cursor: Seq,
        resolutions: &[super::super::legacy_resource_migration::LegacyResourceOriginResolution],
    ) -> Result<
        Option<super::super::legacy_resource_migration::MigratedReplay>,
        super::super::legacy_resource_migration::LegacyMigrationError,
    > {
        use super::super::legacy_resource_migration::{
            migrate_legacy_prefix_with_resolutions, probe_legacy_mint_links,
            validate_legacy_origin_resolutions,
        };
        let end = cursor.index();
        if end > self.log.len() {
            return Ok(None);
        }
        validate_legacy_origin_resolutions(baseline, self.log.entries(), resolutions)?;
        let prefix = &self.log.entries()[..end];
        let ids: std::collections::BTreeSet<_> = probe_legacy_mint_links(baseline, prefix)?
            .into_iter()
            .map(|diagnostic| diagnostic.statement_id)
            .collect();
        let relevant: Vec<_> = resolutions
            .iter()
            .filter(|choice| ids.contains(&choice.statement_id))
            .cloned()
            .collect();
        migrate_legacy_prefix_with_resolutions(baseline, prefix, &relevant).map(Some)
    }

    pub fn migrated_snapshot_at_from_with_resolutions(
        &self,
        baseline: &Graph,
        cursor: Seq,
        resolutions: &[super::super::legacy_resource_migration::LegacyResourceOriginResolution],
    ) -> Result<Option<Graph>, super::super::legacy_resource_migration::LegacyMigrationError> {
        self.migrated_prefix_at_from_with_resolutions(baseline, cursor, resolutions)
            .map(|replay| replay.map(|replay| replay.graph))
    }

    /// The graph-only convenience form of [`migrated_prefix_at_from`](Self::migrated_prefix_at_from).
    pub fn migrated_snapshot_at_from(
        &self,
        baseline: &Graph,
        cursor: Seq,
    ) -> Result<Option<Graph>, super::super::legacy_resource_migration::LegacyMigrationError> {
        self.migrated_prefix_at_from(baseline, cursor)
            .map(|replay| replay.map(|replay| replay.graph))
    }

    /// Reconstruct retained history before replacing a legacy checkpoint atomically.
    /// The full prefix supplies historical URLs and withdrawn handles; `since` is
    /// validated but cannot substitute the checkpoint's current URL for that evidence.
    pub fn migrated_replay_from_with_baseline(
        &self,
        since: Seq,
        graph: &mut Graph,
        baseline: &Graph,
    ) -> Result<(), super::super::legacy_resource_migration::LegacyMigrationError> {
        self.migrated_replay_from_with_baseline_and_resolutions(since, graph, baseline, &[])
    }

    pub fn migrated_replay_from_with_baseline_and_resolutions(
        &self,
        since: Seq,
        graph: &mut Graph,
        baseline: &Graph,
        resolutions: &[super::super::legacy_resource_migration::LegacyResourceOriginResolution],
    ) -> Result<(), super::super::legacy_resource_migration::LegacyMigrationError> {
        if since.index() > self.log.len() {
            return Err(
                super::super::legacy_resource_migration::LegacyMigrationError(
                    "legacy checkpoint cursor is beyond retained history".into(),
                ),
            );
        }
        let mut replay = self
            .migrated_prefix_at_from_with_resolutions(baseline, self.live_cursor(), resolutions)?
            .expect("live cursor exists");
        replay.graph.recorder.0 = graph.recorder.0.take();
        replay.graph.write_author = graph.write_author.clone();
        replay.graph.current_session = graph.current_session;
        replay.graph.revision = replay.graph.revision.max(graph.revision).saturating_add(1);
        replay.graph.url_grouping_revision = replay
            .graph
            .url_grouping_revision
            .max(graph.url_grouping_revision)
            .saturating_add(1);
        *graph = replay.graph;
        Ok(())
    }

    /// Stable, evenly distributed prefix cursors for a compact scrubber. This
    /// uses sequence order alone: journal history has no required wall-clock
    /// timestamp. `max_points == 1` yields only the current live cursor;
    /// larger requests always include both earliest and live when distinct.
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

    /// This journal's log identity, if any.
    pub fn id(&self) -> Option<&LogId> {
        self.log.id()
    }

    /// Where this journal forked from, if it is a fork.
    pub fn provenance(&self) -> Option<&Provenance> {
        self.log.provenance()
    }

    /// Rebuild the whole graph, using its author for legacy unattributed assertions.
    pub fn replay(&self) -> Graph {
        let mut graph = Graph::new();
        // A whole-log rebuild drops a refused entry and keeps going; checked
        // readers use `snapshot_at`.
        let _ = replay_entries_onto(&mut graph, self.log.entries(), OnRefusal::Drop);
        graph
    }

    /// Advance an already-materialized `graph` by the edits from `since` onward.
    /// The checkpoint-plus-tail path: restore a `GraphSnapshot`, then apply only
    /// the journal entries recorded after the snapshot's sequence. The incremental
    /// twin of [`replay`](Self::replay).
    /// Without a retained baseline, exact records missing attribution stay unknown.
    /// Use [`replay_from_with_baseline`](Self::replay_from_with_baseline) to recover
    /// the original minting author across earlier removal and later restoration.
    pub fn replay_from(&self, since: Seq, graph: &mut Graph) -> Result<(), ReplayRefused> {
        replay_outcomes_onto(
            graph,
            self.log
                .from(since)
                .iter()
                .map(|entry| entry.delta.replay_as(&entry.author)),
            OnRefusal::Stop,
        )
    }

    /// Replay a checkpoint tail with the baseline and original minting entries.
    /// `baseline` is the graph before entry zero, not the checkpoint at `since`.
    pub fn replay_from_with_baseline(
        &self,
        since: Seq,
        graph: &mut Graph,
        baseline: &Graph,
    ) -> Result<(), ReplayRefused> {
        let mut attribution = ReplayAttribution::from_baseline(baseline);
        // Reconstruct prefix evidence on one shadow graph. Pair/kind legacy
        // attribution needs ordered membership transitions, while cloning the
        // graph once per journal entry would make checkpoint replay quadratic.
        let mut shadow = baseline.clone();
        replay_attributed_deltas_onto(
            &mut shadow,
            &mut attribution,
            self.log.entries().iter().take(since.index()),
            OnRefusal::Stop,
        )?;
        attribution.repair_checkpoint(graph);
        replay_attributed_deltas_onto(
            graph,
            &mut attribution,
            self.log.from(since),
            OnRefusal::Stop,
        )
    }

    /// Fork this journal under a new identity: copies the whole edit history and
    /// records where it branched from, then diverges independently. The log-level
    /// mirror of a graph fork; the fork replays to an identical graph and then
    /// edits without touching the source.
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

    /// Load a journal from the muniment slot at `key`, or an empty journal if the
    /// slot is absent.
    pub async fn load<B: Backend, C: Codec>(
        slots: &SlotStore<B, C>,
        key: &str,
    ) -> Result<Self, StoreError> {
        Ok(Self::from_log(Journal::load(slots, key).await?))
    }
}
