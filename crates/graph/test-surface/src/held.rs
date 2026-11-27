// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The links a surface holds pending: its session's held state, observed
//! but never journaled.

use serde::{Deserialize, Serialize};

use super::Surface;

/// How long a surface keeps the links it holds pending.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Retention {
    /// Only while the session is open: never stored.
    #[default]
    SessionOnly,
    /// Stored with the session until purged.
    UntilPurged,
}

/// Links a surface holds beside its journal, waiting for their targets: the
/// session's held state (`SessionGraph::Held`), observed but never journaled.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingLinks {
    pub retention: Retention,
    pub entries: Vec<String>,
}

impl PendingLinks {
    /// What a session stores of these: the policy, and the entries only when
    /// they outlive the session.
    pub(crate) fn retained(mut self) -> Self {
        if self.retention == Retention::SessionOnly {
            self.entries.clear();
        }
        self
    }
}

impl Surface {
    /// The links held pending, live.
    pub fn pending_links(&self) -> &[String] {
        &self.pending.entries
    }

    /// The pending links and their retention.
    pub fn pending_link_state(&self) -> &PendingLinks {
        &self.pending
    }

    /// Hold a link to `target` pending: an observation, not a journal edit.
    pub fn queue_pending_link(&mut self, target: &str) {
        self.pending.entries.push(target.to_owned());
        self.observed += 1;
    }

    /// Drop every pending link.
    pub fn purge_pending_links(&mut self) {
        self.pending.entries.clear();
        self.observed += 1;
    }

    pub fn set_pending_link_retention(&mut self, retention: Retention) {
        self.pending.retention = retention;
        self.observed += 1;
    }
}
