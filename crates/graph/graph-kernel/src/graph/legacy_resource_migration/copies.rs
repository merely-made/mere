// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One handle means one row: legacy data may hold identical parallel copies of
//! a claim, and the migration keeps the first row and records the rest.
use super::{LegacyMigrationError, LegacyMigrationNote, Pair, Replay};
use crate::persistence::PersistedEdge;

pub(super) const DEDUPED_COPY: &str = "identical_parallel_copy_deduped_to_first_row";

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
        self.add_note(
            resource.0,
            LegacyMigrationNote {
                from_surface_id: pair.0.to_string(),
                to_surface_id: pair.1.to_string(),
                from_resource_id: resource.0.to_string(),
                to_resource_id: resource.1.to_string(),
                from_url,
                to_url,
                statement: copy.retained_assertions()[0].clone(),
                uncertainty: DEDUPED_COPY.into(),
            },
        )
    }
}
