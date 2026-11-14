// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Pinned views (engine-split D163): a session's graph at a journal cursor
//! with its query index, owned by the holder and released on drop. The live
//! view is the host's index refreshed per edit; a pin is rebuilt from
//! `graph_at`, which starts from the stored checkpoint's replay point.
use super::*;
use mere::dataset::IndexedDataset;

/// What a pin holds: the graph at its cursor and the index built over it.
pub(super) struct Pinned {
    graph: Graph,
    dataset: IndexedDataset,
}

/// A session's graph as it stood at a journal cursor, with the indexed
/// dataset built over it. The holder owns it and decides how many to keep;
/// the session keeps no reference, so dropping it frees both.
pub struct PinnedView {
    session: SessionId,
    cursor: Seq,
    digest: String,
    pinned: Arc<Pinned>,
}

impl PinnedView {
    pub fn session(&self) -> SessionId {
        self.session
    }

    pub fn cursor(&self) -> Seq {
        self.cursor
    }

    /// The v2 digest of the journal's first `cursor` entries, as a
    /// checkpoint at `cursor` stores it.
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn graph(&self) -> &Graph {
        &self.pinned.graph
    }

    pub fn dataset(&self) -> &IndexedDataset {
        &self.pinned.dataset
    }

    #[cfg(test)]
    pub(super) fn downgrade(&self) -> std::sync::Weak<Pinned> {
        Arc::downgrade(&self.pinned)
    }
}

impl std::fmt::Debug for PinnedView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PinnedView")
            .field("session", &self.session.as_uuid())
            .field("cursor", &self.cursor.0)
            .field("digest", &self.digest)
            .field("rows", &self.pinned.dataset.len())
            .finish()
    }
}

impl<B: Backend> GraphSession<B> {
    /// Pin the graph at journal cursor `cursor` with its indexed dataset:
    /// [`graph_at`](Self::graph_at), then `IndexedDataset::for_graph`, both
    /// built now. [`SessionError::CursorOutOfRange`] past the live cursor or
    /// where `graph_at` has no graph.
    pub fn pin(&self, cursor: Seq) -> Result<PinnedView, SessionError> {
        let out_of_range = || SessionError::CursorOutOfRange {
            cursor,
            live: self.journal.live_cursor(),
        };
        if cursor > self.journal.live_cursor() {
            return Err(out_of_range());
        }
        let digest =
            journal_digest(&self.digests, &self.journal, cursor)?.ok_or_else(out_of_range)?;
        let graph = self.graph_at(cursor).ok_or_else(out_of_range)?;
        let dataset = IndexedDataset::for_graph(&graph);
        Ok(PinnedView {
            session: self.id(),
            cursor,
            digest,
            pinned: Arc::new(Pinned { graph, dataset }),
        })
    }
}

#[cfg(test)]
#[path = "graph_session_pin_tests.rs"]
mod tests;
