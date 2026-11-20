// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Pinned views (engine-split D163, D166): a session's graph at a journal
//! cursor with its query index, owned by the holder and released on drop. The
//! live view is the host's index refreshed per edit; a pin is rebuilt from
//! `graph_at`, which starts from the stored checkpoint's replay point, and
//! indexes on its first read.
use super::*;
use mere::dataset::IndexedDataset;
use std::sync::OnceLock;

/// What a pin holds: the graph at its cursor and, once read, its index.
pub(super) struct Pinned {
    graph: Graph,
    dataset: OnceLock<IndexedDataset>,
}

/// Identity of a journal revision without its graph or index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphRevision {
    pub session: SessionId,
    pub cursor: Seq,
    /// The v2 digest of the journal prefix at this cursor.
    pub digest: String,
}

/// A session's graph as it stood at a journal cursor, with the indexed
/// dataset over it built on the first [`dataset`](Self::dataset) call. The
/// holder owns it and decides how many to keep; the session keeps no
/// reference, so dropping it frees both.
///
/// Dropping is the holder's cost, paid on the dropping thread: freeing an
/// indexed pin took about 0.4 s at 100k nodes and about 36 ms at 10k on the
/// reference laptop (engine-split D166). A host that cannot wait drops it on a
/// worker.
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

    pub fn revision_identity(&self) -> GraphRevision {
        GraphRevision {
            session: self.session,
            cursor: self.cursor,
            digest: self.digest.clone(),
        }
    }

    pub fn graph(&self) -> &Graph {
        &self.pinned.graph
    }

    /// The indexed dataset over [`graph`](Self::graph), built on the first
    /// call (1.7 s at 100k nodes) and held from then on.
    pub fn dataset(&self) -> &IndexedDataset {
        self.pinned
            .dataset
            .get_or_init(|| IndexedDataset::for_graph(&self.pinned.graph))
    }

    #[cfg(test)]
    pub(super) fn downgrade(&self) -> std::sync::Weak<Pinned> {
        Arc::downgrade(&self.pinned)
    }

    #[cfg(test)]
    pub(super) fn is_indexed(&self) -> bool {
        self.pinned.dataset.get().is_some()
    }
}

impl std::fmt::Debug for PinnedView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PinnedView")
            .field("session", &self.session.as_uuid())
            .field("cursor", &self.cursor.0)
            .field("digest", &self.digest)
            .field("rows", &self.pinned.dataset.get().map(IndexedDataset::len))
            .finish()
    }
}

impl<B: Backend> GraphSession<B> {
    /// Identify the live journal revision without copying the graph or index.
    pub fn live_revision(&self) -> Result<GraphRevision, SessionError> {
        self.revision_at(self.journal.live_cursor())
    }

    /// Pin the graph at journal cursor `cursor`: [`graph_at`](Self::graph_at)
    /// now, its `IndexedDataset` on the view's first
    /// [`dataset`](PinnedView::dataset) read. [`SessionError::CursorOutOfRange`]
    /// past the live cursor or where `graph_at` has no graph.
    pub fn pin(&self, cursor: Seq) -> Result<PinnedView, SessionError> {
        let revision = self.revision_at(cursor)?;
        let graph = self
            .graph_at(cursor)
            .ok_or_else(|| SessionError::CursorOutOfRange {
                cursor,
                live: self.journal.live_cursor(),
            })?;
        Ok(PinnedView {
            session: revision.session,
            cursor: revision.cursor,
            digest: revision.digest,
            pinned: Arc::new(Pinned {
                graph,
                dataset: OnceLock::new(),
            }),
        })
    }

    fn revision_at(&self, cursor: Seq) -> Result<GraphRevision, SessionError> {
        let out_of_range = || SessionError::CursorOutOfRange {
            cursor,
            live: self.journal.live_cursor(),
        };
        if cursor > self.journal.live_cursor() {
            return Err(out_of_range());
        }
        let digest =
            journal_digest(&self.digests, &self.journal, cursor)?.ok_or_else(out_of_range)?;
        Ok(GraphRevision {
            session: self.id(),
            cursor,
            digest,
        })
    }
}

#[cfg(test)]
#[path = "graph_session_pin_tests.rs"]
mod tests;
