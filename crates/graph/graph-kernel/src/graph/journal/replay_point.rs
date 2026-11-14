// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Resuming a baseline replay part-way (engine-split D161): the graph a replay
//! has reached at a cursor, with the attribution it carried through the prefix.

use super::*;

/// Where a replay from a baseline stands at journal cursor [`cursor`]: the
/// graph it built and the statement sources it carried (a source outlives its
/// handle's removal, so a later restoration keeps it). Replaying on from a
/// point gives exactly what replaying from the baseline gives.
///
/// [`cursor`]: Self::cursor
#[derive(Clone)]
pub struct ReplayPoint {
    cursor: Seq,
    graph: Graph,
    attribution: ReplayAttribution,
}

impl std::fmt::Debug for ReplayPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReplayPoint")
            .field("cursor", &self.cursor)
            .finish_non_exhaustive()
    }
}

impl ReplayPoint {
    /// The point before the first entry: `baseline` itself.
    pub fn at_baseline(baseline: &Graph) -> Self {
        Self {
            cursor: Seq(0),
            graph: baseline.clone(),
            attribution: ReplayAttribution::from_baseline(baseline),
        }
    }

    pub fn cursor(&self) -> Seq {
        self.cursor
    }

    /// The graph a baseline replay holds at [`cursor`](Self::cursor).
    pub fn graph(&self) -> &Graph {
        &self.graph
    }
}

/// `refused` re-addressed from a replay that began at `from`, so its position
/// counts journal entries.
fn from_cursor(mut refused: ReplayRefused, from: Seq) -> ReplayRefused {
    refused.position += from.index();
    refused
}

impl GraphJournal {
    /// The point a replay of `baseline` reaches at `cursor`; `None` past the
    /// journal. Costs what [`snapshot_at_from`](Self::snapshot_at_from) costs.
    pub fn replay_point_from(
        &self,
        baseline: &Graph,
        cursor: Seq,
    ) -> Result<Option<ReplayPoint>, ReplayRefused> {
        if cursor.index() > self.log.len() {
            return Ok(None);
        }
        let mut point = ReplayPoint::at_baseline(baseline);
        self.advance_replay_point(&mut point, cursor)?;
        Ok(Some(point))
    }

    /// Move `point` on to `cursor`, replaying only the entries between.
    /// `false`, with `point` untouched, for a cursor before it or past the
    /// journal. A refused entry leaves `point` part-way, so drop it.
    pub fn advance_replay_point(
        &self,
        point: &mut ReplayPoint,
        cursor: Seq,
    ) -> Result<bool, ReplayRefused> {
        if cursor < point.cursor || cursor.index() > self.log.len() {
            return Ok(false);
        }
        let from = point.cursor;
        replay_attributed_deltas_onto(
            &mut point.graph,
            &mut point.attribution,
            &self.log.entries()[from.index()..cursor.index()],
            OnRefusal::Stop,
        )
        .map_err(|refused| from_cursor(refused, from))?;
        point.cursor = cursor;
        Ok(true)
    }

    /// The graph at `cursor`, replayed on from `point`: equal to
    /// [`snapshot_at_from`](Self::snapshot_at_from) from the baseline `point`
    /// began at. `None` for a cursor before `point` or past the journal.
    pub fn snapshot_at_from_point(
        &self,
        point: &ReplayPoint,
        cursor: Seq,
    ) -> Result<Option<Graph>, ReplayRefused> {
        if cursor < point.cursor || cursor.index() > self.log.len() {
            return Ok(None);
        }
        let mut graph = point.graph.clone();
        if cursor > point.cursor {
            let mut attribution = point.attribution.clone();
            replay_attributed_deltas_onto(
                &mut graph,
                &mut attribution,
                &self.log.entries()[point.cursor.index()..cursor.index()],
                OnRefusal::Stop,
            )
            .map_err(|refused| from_cursor(refused, point.cursor))?;
        }
        Ok(Some(graph))
    }

    /// Advance a checkpoint `graph` by the entries from `point` on, with the
    /// attribution `point` carried: [`replay_from_with_baseline`] once its
    /// prefix replay is `point`.
    ///
    /// [`replay_from_with_baseline`]: Self::replay_from_with_baseline
    pub fn replay_from_point(
        &self,
        point: &ReplayPoint,
        graph: &mut Graph,
    ) -> Result<(), ReplayRefused> {
        let mut attribution = point.attribution.clone();
        attribution.repair_checkpoint(graph);
        replay_attributed_deltas_onto(
            graph,
            &mut attribution,
            self.log.from(point.cursor),
            OnRefusal::Stop,
        )
    }
}
