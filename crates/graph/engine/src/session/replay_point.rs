// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Resuming a baseline replay part-way (engine split D161, D251): the graph a
//! replay has reached at a cursor, with what it carried through the prefix.

use muniment::Seq;

use super::{OnRefusal, ReplayRefused, SessionGraph, SessionJournal};

/// Where a replay from a baseline stands at journal cursor [`cursor`]: the
/// graph it built and what it carried (a statement's source outlives its
/// handle's removal, so a later restoration keeps it). Replaying on from a
/// point gives exactly what replaying from the baseline gives.
///
/// [`cursor`]: Self::cursor
pub struct ReplayPoint<G: SessionGraph> {
    cursor: Seq,
    graph: G,
    carry: G::Carry,
}

impl<G: SessionGraph> Clone for ReplayPoint<G> {
    fn clone(&self) -> Self {
        Self {
            cursor: self.cursor,
            graph: self.graph.clone(),
            carry: self.carry.clone(),
        }
    }
}

impl<G: SessionGraph> std::fmt::Debug for ReplayPoint<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReplayPoint")
            .field("cursor", &self.cursor)
            .finish_non_exhaustive()
    }
}

impl<G: SessionGraph> ReplayPoint<G> {
    /// The point before the first entry: `baseline` itself.
    pub fn at_baseline(baseline: &G) -> Self {
        Self {
            cursor: Seq(0),
            graph: baseline.clone(),
            carry: G::carry_from(baseline),
        }
    }

    pub fn cursor(&self) -> Seq {
        self.cursor
    }

    /// The graph a baseline replay holds at [`cursor`](Self::cursor).
    pub fn graph(&self) -> &G {
        &self.graph
    }
}

/// `refused` re-addressed from a replay that began at `from`, so its position
/// counts journal entries.
fn from_cursor(mut refused: ReplayRefused, from: Seq) -> ReplayRefused {
    refused.position += from.index();
    refused
}

impl<G: SessionGraph> SessionJournal<G> {
    /// The point a replay of `baseline` reaches at `cursor`; `None` past the
    /// journal. Costs what [`snapshot_at_from`](Self::snapshot_at_from) costs.
    pub fn replay_point_from(
        &self,
        baseline: &G,
        cursor: Seq,
    ) -> Result<Option<ReplayPoint<G>>, ReplayRefused> {
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
        point: &mut ReplayPoint<G>,
        cursor: Seq,
    ) -> Result<bool, ReplayRefused> {
        if cursor < point.cursor || cursor.index() > self.log.len() {
            return Ok(false);
        }
        let from = point.cursor;
        point
            .graph
            .replay_carried(
                &mut point.carry,
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
        point: &ReplayPoint<G>,
        cursor: Seq,
    ) -> Result<Option<G>, ReplayRefused> {
        if cursor < point.cursor || cursor.index() > self.log.len() {
            return Ok(None);
        }
        let mut graph = point.graph.clone();
        if cursor > point.cursor {
            let mut carry = point.carry.clone();
            graph
                .replay_carried(
                    &mut carry,
                    &self.log.entries()[point.cursor.index()..cursor.index()],
                    OnRefusal::Stop,
                )
                .map_err(|refused| from_cursor(refused, point.cursor))?;
        }
        Ok(Some(graph))
    }

    /// Advance a checkpoint `graph` by the entries from `point` on, with what
    /// `point` carried: [`replay_from_with_baseline`] once its prefix replay
    /// is `point`.
    ///
    /// [`replay_from_with_baseline`]: Self::replay_from_with_baseline
    pub fn replay_from_point(
        &self,
        point: &ReplayPoint<G>,
        graph: &mut G,
    ) -> Result<(), ReplayRefused> {
        let mut carry = point.carry.clone();
        G::repair_checkpoint(&carry, graph);
        graph.replay_carried(&mut carry, self.log.from(point.cursor), OnRefusal::Stop)
    }
}
