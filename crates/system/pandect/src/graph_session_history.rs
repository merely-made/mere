// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! History reads and whole-change imports (engine-split D161): `graph_at`
//! starts from the stored checkpoint's replay point, and an imported change
//! with a refused edit commits nothing.
use super::*;
use kernel::graph::apply::GraphDeltaResult;

/// The replay point at the stored checkpoint, with the v2 digest of the
/// journal prefix it was replayed from.
#[derive(Clone)]
pub(super) struct HeldPoint {
    pub(super) point: ReplayPoint,
    pub(super) digest: Option<String>,
}

/// Apply `deltas` in order as one change, refusing it at the first edit
/// admission refuses: the error names that edit and its reason, the handle
/// for a collision, and the change commits nothing.
pub(super) fn apply_all(graph: &mut Graph, deltas: Vec<GraphDelta>) -> Result<(), SessionError> {
    for (position, delta) in deltas.into_iter().enumerate() {
        if let GraphDeltaResult::Refused(reason) = apply_graph_delta(graph, delta) {
            return Err(SessionError::Validation(format!(
                "edit {position} refused: {reason}"
            )));
        }
    }
    Ok(())
}

impl<B: Backend> GraphSession<B> {
    /// Whether `held`'s digest names the journal's prefix at its cursor.
    fn verifies(&self, held: &HeldPoint) -> bool {
        held.digest.is_some()
            && journal_digest(&self.digests, &self.journal, held.point.cursor())
                .is_ok_and(|digest| digest == held.digest)
    }

    /// The replay point `graph_at(cursor)` starts from: the held checkpoint's,
    /// when it lies at or before `cursor` and verifies.
    pub(super) fn replay_point_for(&self, cursor: Seq) -> Option<&ReplayPoint> {
        let held = self.replay_point.as_deref()?;
        (held.point.cursor() <= cursor && self.verifies(held)).then_some(&held.point)
    }

    /// Move the held replay point to the checkpoint just stored at `cursor`,
    /// replaying only the entries since the last one, or from the baseline when
    /// none verifies. A failed replay holds none, so `graph_at` reads the
    /// baseline.
    pub(super) fn hold_replay_point(&mut self, cursor: Seq) {
        if self.translation.is_some() {
            return;
        }
        let point = match self.replay_point.take() {
            Some(held) if held.point.cursor() <= cursor && self.verifies(&held) => {
                let mut point = Arc::unwrap_or_clone(held).point;
                matches!(
                    self.journal.advance_replay_point(&mut point, cursor),
                    Ok(true)
                )
                .then_some(point)
            },
            _ => self
                .journal
                .replay_point_from(&self.baseline, cursor)
                .ok()
                .flatten(),
        };
        self.replay_point = point.and_then(|point| {
            let digest = journal_digest(&self.digests, &self.journal, cursor).ok()?;
            Some(Arc::new(HeldPoint { point, digest }))
        });
    }
}

#[cfg(test)]
#[path = "graph_session_history_tests.rs"]
mod tests;
