// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Pinned views (engine-split D163, D166): a session's graph at a journal
//! cursor with its query index, owned by the holder and released on drop. The
//! live view is the host's index refreshed per edit; a pin is rebuilt from
//! `graph_at`, which starts from the stored checkpoint's replay point, and
//! indexes on its first read. Its Resource digest (D190 to D192) is likewise
//! computed on the first ask and kept.
use super::*;
use mere::dataset::IndexedDataset;
use mere::formats::canon::{SortedNQuadsError, sorted_nquads};
use mere::projection::RdfSource;
use std::sync::OnceLock;

/// The Resource digest's algorithm: BLAKE3 in key-derivation mode over the
/// sorted N-Quads of the Resource stratum (engine-split D190).
pub const RESOURCE_DIGEST_ALGORITHM: &str = "mere-resource-sorted-nquads-blake3";
/// The Resource digest's version; a change to its definition bumps it.
pub const RESOURCE_DIGEST_VERSION: u32 = 1;
/// The key-derivation context: domain separation, naming the version.
const RESOURCE_DIGEST_DOMAIN: &str = "mere resource digest v1";

/// What a pin holds: the graph at its cursor and, once read, its index and
/// Resource digest.
pub(super) struct Pinned<G> {
    graph: G,
    dataset: OnceLock<IndexedDataset>,
    resource_digest: OnceLock<Result<ResourceDigest, SortedNQuadsError>>,
    #[cfg(test)]
    resource_digests_computed: std::sync::atomic::AtomicUsize,
}

impl<G> Pinned<G> {
    fn new(graph: G) -> Self {
        Self {
            graph,
            dataset: OnceLock::new(),
            resource_digest: OnceLock::new(),
            #[cfg(test)]
            resource_digests_computed: Default::default(),
        }
    }
}

/// A digest of a pin's Resource-stratum content, with the algorithm and
/// version that define it (engine-split D190, D192).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceDigest {
    /// [`RESOURCE_DIGEST_ALGORITHM`].
    pub algorithm: &'static str,
    /// [`RESOURCE_DIGEST_VERSION`].
    pub version: u32,
    /// Lowercase hex of the 32-byte BLAKE3 output.
    pub digest: String,
}

/// Every Resource-stratum quad of `graph`, held and withdrawn reifiers, as
/// Graphshell's P8 probe reads them; may repeat a quad.
fn resource_quads<G: RdfSource>(graph: &G) -> impl Iterator<Item = oxrdf::Quad> + '_ {
    let resources = graph.resource_graph();
    resources
        .held_quads()
        .chain(resources.withdrawn_reifier_quads())
}

/// BLAKE3 key derivation under `domain` over `nquads`, as hex.
fn digest_nquads(domain: &str, nquads: &str) -> String {
    let mut hasher = blake3::Hasher::new_derive_key(domain);
    hasher.update(nquads.as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// The Resource digest of `graph`: D190's sorted N-Quads under
/// [`RESOURCE_DIGEST_DOMAIN`].
fn resource_digest_of<G: RdfSource>(graph: &G) -> Result<ResourceDigest, SortedNQuadsError> {
    let nquads = sorted_nquads(resource_quads(graph))?;
    Ok(ResourceDigest {
        algorithm: RESOURCE_DIGEST_ALGORITHM,
        version: RESOURCE_DIGEST_VERSION,
        digest: digest_nquads(RESOURCE_DIGEST_DOMAIN, &nquads),
    })
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
pub struct PinnedView<G: SessionGraph = Graph> {
    session: SessionId,
    cursor: Seq,
    digest: String,
    pinned: Arc<Pinned<G>>,
}

impl<G: SessionGraph> PinnedView<G> {
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

    pub fn graph(&self) -> &G {
        &self.pinned.graph
    }

    /// The indexed dataset over [`graph`](Self::graph), built on the first
    /// call (1.7 s at 100k nodes) and held from then on.
    pub fn dataset(&self) -> &IndexedDataset {
        self.pinned
            .dataset
            .get_or_init(|| IndexedDataset::for_graph(&self.pinned.graph))
    }

    /// The digest of every Resource-stratum quad at this cursor, reifier rows
    /// included: BLAKE3 over their deduplicated, sorted RDF 1.2 N-Quads,
    /// domain-separated and versioned (engine-split D190). Computed on the
    /// first call and held from then on (D191); the asker pays, about 3 s at
    /// 100k nodes, and no edit computes it.
    ///
    /// # Errors
    ///
    /// [`SortedNQuadsError::BlankNode`] if a Resource quad holds a blank node,
    /// which sorting cannot label canonically; the refusal is kept too.
    pub fn resource_digest(&self) -> Result<&ResourceDigest, SortedNQuadsError> {
        self.pinned
            .resource_digest
            .get_or_init(|| {
                #[cfg(test)]
                self.pinned
                    .resource_digests_computed
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                resource_digest_of(&self.pinned.graph)
            })
            .as_ref()
            .map_err(Clone::clone)
    }

    #[cfg(test)]
    pub(super) fn downgrade(&self) -> std::sync::Weak<Pinned<G>> {
        Arc::downgrade(&self.pinned)
    }

    #[cfg(test)]
    pub(super) fn is_indexed(&self) -> bool {
        self.pinned.dataset.get().is_some()
    }

    #[cfg(test)]
    pub(super) fn resource_digests_computed(&self) -> usize {
        self.pinned
            .resource_digests_computed
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl<G: SessionGraph> std::fmt::Debug for PinnedView<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PinnedView")
            .field("session", &self.session.as_uuid())
            .field("cursor", &self.cursor.0)
            .field("digest", &self.digest)
            .field("rows", &self.pinned.dataset.get().map(IndexedDataset::len))
            .field("resource_digest", &self.pinned.resource_digest.get())
            .finish()
    }
}

impl<B: Backend, G: SessionGraph> GraphSession<B, G> {
    /// Identify the live journal revision without copying the graph or index.
    pub fn live_revision(&self) -> Result<GraphRevision, SessionError> {
        self.revision_at(self.journal.live_cursor())
    }

    /// Pin the graph at journal cursor `cursor`: [`graph_at`](Self::graph_at)
    /// now, its `IndexedDataset` on the view's first
    /// [`dataset`](PinnedView::dataset) read. [`SessionError::CursorOutOfRange`]
    /// past the live cursor or where `graph_at` has no graph.
    pub fn pin(&self, cursor: Seq) -> Result<PinnedView<G>, SessionError> {
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
            pinned: Arc::new(Pinned::new(graph)),
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
