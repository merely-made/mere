// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Legacy migration reads over the kernel journal (engine split D249): an
//! extension trait, since the journal type is the engine's.

use super::*;
use muniment::Seq;

/// Migrating a qualified legacy journal prefix: the kernel's own reads over
/// [`GraphJournal`], which only the kernel and its tests call.
pub trait LegacyJournalMigration {
    /// Migration of a caller-qualified legacy baseline and prefix.
    /// Ordinary replay does not infer a legacy profile from absent resource columns.
    fn migrated_prefix_at_from(
        &self,
        baseline: &Graph,
        cursor: Seq,
    ) -> Result<
        Option<super::super::legacy_resource_migration::MigratedReplay>,
        super::super::legacy_resource_migration::LegacyMigrationError,
    >;

    /// Validate choices against this complete qualified legacy journal, then read a prefix.
    /// Future carried-handle choices do not change or invalidate earlier prefixes.
    /// A host with a modern suffix must supply only its qualified legacy prefix to the adapter.
    fn migrated_prefix_at_from_with_resolutions(
        &self,
        baseline: &Graph,
        cursor: Seq,
        resolutions: &[super::super::legacy_resource_migration::LegacyResourceOriginResolution],
    ) -> Result<
        Option<super::super::legacy_resource_migration::MigratedReplay>,
        super::super::legacy_resource_migration::LegacyMigrationError,
    >;

    fn migrated_snapshot_at_from_with_resolutions(
        &self,
        baseline: &Graph,
        cursor: Seq,
        resolutions: &[super::super::legacy_resource_migration::LegacyResourceOriginResolution],
    ) -> Result<Option<Graph>, super::super::legacy_resource_migration::LegacyMigrationError>;

    /// The graph-only convenience form of [`migrated_prefix_at_from`](Self::migrated_prefix_at_from).
    fn migrated_snapshot_at_from(
        &self,
        baseline: &Graph,
        cursor: Seq,
    ) -> Result<Option<Graph>, super::super::legacy_resource_migration::LegacyMigrationError>;

    /// Reconstruct retained history before replacing a legacy checkpoint atomically.
    /// The full prefix supplies historical URLs and withdrawn handles; `since` is
    /// validated but cannot substitute the checkpoint's current URL for that evidence.
    fn migrated_replay_from_with_baseline(
        &self,
        since: Seq,
        graph: &mut Graph,
        baseline: &Graph,
    ) -> Result<(), super::super::legacy_resource_migration::LegacyMigrationError>;

    fn migrated_replay_from_with_baseline_and_resolutions(
        &self,
        since: Seq,
        graph: &mut Graph,
        baseline: &Graph,
        resolutions: &[super::super::legacy_resource_migration::LegacyResourceOriginResolution],
    ) -> Result<(), super::super::legacy_resource_migration::LegacyMigrationError>;
}

impl LegacyJournalMigration for GraphJournal {
    fn migrated_prefix_at_from(
        &self,
        baseline: &Graph,
        cursor: Seq,
    ) -> Result<
        Option<super::super::legacy_resource_migration::MigratedReplay>,
        super::super::legacy_resource_migration::LegacyMigrationError,
    > {
        let end = cursor.index();
        if end > self.len() {
            return Ok(None);
        }
        super::super::legacy_resource_migration::migrate_legacy_prefix(
            baseline,
            &self.entries()[..end],
        )
        .map(Some)
    }

    fn migrated_prefix_at_from_with_resolutions(
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
        if end > self.len() {
            return Ok(None);
        }
        validate_legacy_origin_resolutions(baseline, self.entries(), resolutions)?;
        let prefix = &self.entries()[..end];
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

    fn migrated_snapshot_at_from_with_resolutions(
        &self,
        baseline: &Graph,
        cursor: Seq,
        resolutions: &[super::super::legacy_resource_migration::LegacyResourceOriginResolution],
    ) -> Result<Option<Graph>, super::super::legacy_resource_migration::LegacyMigrationError> {
        self.migrated_prefix_at_from_with_resolutions(baseline, cursor, resolutions)
            .map(|replay| replay.map(|replay| replay.graph))
    }

    fn migrated_snapshot_at_from(
        &self,
        baseline: &Graph,
        cursor: Seq,
    ) -> Result<Option<Graph>, super::super::legacy_resource_migration::LegacyMigrationError> {
        self.migrated_prefix_at_from(baseline, cursor)
            .map(|replay| replay.map(|replay| replay.graph))
    }

    fn migrated_replay_from_with_baseline(
        &self,
        since: Seq,
        graph: &mut Graph,
        baseline: &Graph,
    ) -> Result<(), super::super::legacy_resource_migration::LegacyMigrationError> {
        self.migrated_replay_from_with_baseline_and_resolutions(since, graph, baseline, &[])
    }

    fn migrated_replay_from_with_baseline_and_resolutions(
        &self,
        since: Seq,
        graph: &mut Graph,
        baseline: &Graph,
        resolutions: &[super::super::legacy_resource_migration::LegacyResourceOriginResolution],
    ) -> Result<(), super::super::legacy_resource_migration::LegacyMigrationError> {
        if since.index() > self.len() {
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
}
