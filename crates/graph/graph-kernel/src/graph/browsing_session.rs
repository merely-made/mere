// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Idle-gap browsing sessions (engine-split D200, D202). A browsing session
//! ends when no traversal is recorded for the idle gap; the next traversal
//! opens a new one under a fresh UUID, which each event carries and the
//! journal replays. This is apart from the launch number
//! ([`Graph::current_session`]) stamped on visited nodes.

use uuid::Uuid;

use super::Graph;

/// The idle gap a graph uses until its host installs one: 30 minutes.
pub const DEFAULT_BROWSING_IDLE_GAP_MS: u64 = 30 * 60 * 1000;

/// The open browsing session and its last traversal time. Runtime state: a
/// reopened graph seeds it from its latest recorded event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BrowsingSessionClock {
    open: Option<(Uuid, u64)>,
    idle_gap_ms: u64,
}

impl Default for BrowsingSessionClock {
    fn default() -> Self {
        Self {
            open: None,
            idle_gap_ms: DEFAULT_BROWSING_IDLE_GAP_MS,
        }
    }
}

impl BrowsingSessionClock {
    /// The session a traversal recorded at `at_ms` joins, opening one when
    /// none is open or the gap since the last traversal reached the idle gap.
    pub(crate) fn session_for(&mut self, at_ms: u64) -> Uuid {
        match self.open {
            Some((session, last)) if at_ms.abs_diff(last) < self.idle_gap_ms => {
                self.open = Some((session, last.max(at_ms)));
                session
            },
            _ => {
                let session = Uuid::new_v4();
                self.open = Some((session, at_ms));
                session
            },
        }
    }

    /// Note a recorded event (replay, reopen) so later traversals within the
    /// gap continue its session. The latest event wins.
    pub(crate) fn observe(&mut self, session: Uuid, at_ms: u64) {
        if !matches!(self.open, Some((_, last)) if last > at_ms) {
            self.open = Some((session, at_ms));
        }
    }
}

impl Graph {
    /// Install the host's idle gap (a persona setting in Graphshell).
    pub fn set_browsing_idle_gap_ms(&mut self, idle_gap_ms: u64) {
        self.browsing_session.idle_gap_ms = idle_gap_ms;
    }

    pub fn browsing_idle_gap_ms(&self) -> u64 {
        self.browsing_session.idle_gap_ms
    }

    /// The browsing session the next traversal joins if it comes within the
    /// gap, or `None` when the next one opens a new session.
    pub fn open_browsing_session(&self) -> Option<Uuid> {
        self.browsing_session.open.map(|(session, _)| session)
    }

    /// Close the open browsing session by hand: the next traversal opens a new
    /// one. Not journaled; the next event's recorded session carries the split.
    pub fn split_browsing_session(&mut self) {
        self.browsing_session.open = None;
    }

    /// Seed the clock from the latest event a loaded graph holds.
    pub(crate) fn seed_browsing_session(&mut self) {
        let latest = self
            .surface_relation_buckets()
            .filter_map(|(_, _, _, payload)| payload.traversal_data())
            .flat_map(|data| data.traversals.iter())
            .max_by_key(|event| event.timestamp_ms)
            .map(|event| (event.browsing_session, event.timestamp_ms));
        if let Some((session, at_ms)) = latest {
            self.browsing_session.observe(session, at_ms);
        }
    }
}
