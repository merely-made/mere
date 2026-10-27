// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One handle means one row: legacy data may hold identical parallel copies of
//! a claim, and the migration keeps the first row and records the rest. A copy
//! is a whole persisted row equal to an earlier one on its pair. A handle
//! repeated on rows that differ is a conflict, and the final checked load
//! refuses it.
//!
//! Each dropped copy is a note on the owning Resource, one per statement it
//! held, tagged by where the copy sat:
//! - [`DEDUPED_COPY`]: a Resource row the migration built from a Surface claim.
//! - [`DEDUPED_SURFACE_COPY`]: a Surface-placed claim has no Resource row, so the
//!   note names the Surface pair and the current Resources of its endpoints.
//! - [`DEDUPED_RESOURCE_COPY`]: a Resource row the output already held. It never had
//!   a Surface origin, so the Surface ids are empty.
use std::collections::BTreeMap;

use super::{LegacyMigrationError, LegacyMigrationNote, Pair, Replay};
use crate::persistence::PersistedEdge;

pub(super) const DEDUPED_COPY: &str = "identical_parallel_copy_deduped_to_first_row";
pub(super) const DEDUPED_SURFACE_COPY: &str =
    "identical_parallel_surface_copy_deduped_to_first_row";
pub(super) const DEDUPED_RESOURCE_COPY: &str =
    "identical_parallel_resource_copy_deduped_to_first_row";

/// `rows` without the later copies of a row that holds a handle, and those copies.
pub(super) fn split_copies(rows: Vec<PersistedEdge>) -> (Vec<PersistedEdge>, Vec<PersistedEdge>) {
    let (mut kept, mut copies) = (Vec::<PersistedEdge>::new(), Vec::new());
    for row in rows {
        if !row.retained_assertions().is_empty() && kept.contains(&row) {
            copies.push(row);
        } else {
            kept.push(row);
        }
    }
    (kept, copies)
}

impl Replay {
    /// Record that `copy`, a repeat of a row already placed for `resource`, was dropped.
    pub(super) fn note_deduped_copy(
        &mut self,
        pair: Pair,
        resource: Pair,
        copy: &PersistedEdge,
    ) -> Result<(), LegacyMigrationError> {
        let (_, from_url) = self.ensure_resource(pair.0)?;
        let (_, to_url) = self.ensure_resource(pair.1)?;
        self.note_copy(Some(pair), resource, (from_url, to_url), copy, DEDUPED_COPY)
    }

    /// Record that `copy`, a repeat of a Surface row on `pair`, was dropped.
    pub(super) fn note_deduped_surface_copy(
        &mut self,
        pair: Pair,
        copy: &PersistedEdge,
    ) -> Result<(), LegacyMigrationError> {
        let (from, from_url) = self.ensure_resource(pair.0)?;
        let (to, to_url) = self.ensure_resource(pair.1)?;
        self.note_copy(
            Some(pair),
            (from, to),
            (from_url, to_url),
            copy,
            DEDUPED_SURFACE_COPY,
        )
    }

    /// Drop each later identical copy of a Resource row the output holds, and record it.
    pub(super) fn dedupe_resource_copies(&mut self) -> Result<(), LegacyMigrationError> {
        let mut held = BTreeMap::<Pair, usize>::new();
        for (from, to, payload) in self.output.resource_edges() {
            if !payload.assertions().is_empty() {
                *held.entry((from.id(), to.id())).or_default() += 1;
            }
        }
        for pair in held
            .into_iter()
            .filter_map(|(pair, rows)| (rows > 1).then_some(pair))
        {
            let rows = self.output.persisted_resource_edges_between(pair.0, pair.1);
            let (kept, copies) = split_copies(rows);
            if copies.is_empty() {
                continue;
            }
            let iri = |id| {
                self.output
                    .resource_record(id)
                    .map(|record| record.canonical_iri)
                    .ok_or_else(|| LegacyMigrationError("resource copy has no record".into()))
            };
            let urls = (iri(pair.0)?, iri(pair.1)?);
            for copy in &copies {
                self.note_copy(None, pair, urls.clone(), copy, DEDUPED_RESOURCE_COPY)?;
            }
            if !self
                .output
                .set_resource_edges_between(pair.0, pair.1, &kept)
            {
                return Err(LegacyMigrationError(format!(
                    "resource handle collision on {} -> {}",
                    pair.0, pair.1
                )));
            }
        }
        Ok(())
    }

    /// One note per statement the copy held, on the Resource that starts `resource`.
    /// No `surface` pair means the copy never sat on a Surface row.
    fn note_copy(
        &mut self,
        surface: Option<Pair>,
        resource: Pair,
        urls: (String, String),
        copy: &PersistedEdge,
        uncertainty: &str,
    ) -> Result<(), LegacyMigrationError> {
        let (from_surface_id, to_surface_id) = surface.map_or_else(Default::default, |pair| {
            (pair.0.to_string(), pair.1.to_string())
        });
        for statement in copy.retained_assertions() {
            self.add_note(
                resource.0,
                LegacyMigrationNote {
                    from_surface_id: from_surface_id.clone(),
                    to_surface_id: to_surface_id.clone(),
                    from_resource_id: resource.0.to_string(),
                    to_resource_id: resource.1.to_string(),
                    from_url: urls.0.clone(),
                    to_url: urls.1.clone(),
                    statement: statement.clone(),
                    uncertainty: uncertainty.into(),
                },
            )?;
        }
        Ok(())
    }
}
