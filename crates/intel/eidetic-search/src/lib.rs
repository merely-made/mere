// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! eidetic-search — lexical recall over your own trail (Phase 9, producer half).
//!
//! The [`TrailIndex`] is an in-tree BM25 index minted **from** `BrowsingTrace`
//! codicils: derived state, never the source of truth. The trace corpus in
//! the eidetic store is the authority; the index can always be re-minted
//! from it ([`TrailIndex::rebuild`]), which is exactly what happens when an
//! on-disk projection's format no longer matches this build
//! ([`SearchError::FormatMismatch`] — the format version travels with the
//! index and recipients reject-or-re-mint, the workspace-pins doctrine
//! applied to a wire format).
//!
//! The engine is a postings map and a weighted per-field BM25 ([`bm25`]) over
//! one tokenizer ([`tokenize`]): no mmap, no threads, no C, so the crate
//! follows muniment to wasm/OPFS. tantivy was retired for that reason and for
//! its fixed cost per re-mint, which is the only thing the consumer does.
//!
//! Three surfaces:
//!
//! - **Recall** — [`TrailIndex::search`]: BM25 over tokenized titles, page
//!   text, and URL components, weighted by [`index::FieldWeights`]; a single
//!   absolute URL takes the exact canonical-URL lane. "Where did I read
//!   about X?"
//! - **Reports** — [`TrailIndex::top_domains`] /
//!   [`TrailIndex::visits_histogram`] over the stored columns (domain / owner
//!   / time / transition, so reports need no re-index).
//! - **Fusion** — [`fuse`] / [`fuse_many`]: the engine-agnostic seam that
//!   merges this crate's lexical ranking with a vector ranking (`intel/esp`
//!   or any other) and any further lane — the behavioural one in
//!   `eidetic::browsing::frecency` — by reciprocal-rank fusion. This crate
//!   deliberately does not depend on an embedding engine; the caller brings
//!   the rankings.
//!
//! The `SearchIndexSpec` codicil ([`spec`]) is the hand-off contract: it
//! names the field set, tokenizer, and scoring engine. Locally it also rides
//! a sidecar file in the index directory so `open` can check compatibility
//! before reading the projection; as a codicil it is what a moot's consume
//! half (deferred) would verify before merging.

pub mod bm25;
pub mod fusion;
pub mod index;
pub mod spec;
pub mod tokenize;

pub use bm25::{Bm25Config, Bm25Index};
pub use fusion::{FusedHit, Ranking, fuse, fuse_many};
pub use index::{FieldWeights, Hit, IndexConfig, TrailIndex};
pub use spec::{SEARCH_INDEX_SCHEMA_REF, SPEC_SIDECAR, SearchIndexSpec, bootstrap_search_schema};
pub use tokenize::{Stemmer, Tokenizer};

/// Search-lane failures.
#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    /// The on-disk projection was written by a different engine or field set
    /// than this build. Re-mint from traces ([`TrailIndex::rebuild`]) — the
    /// corpus is the source of truth, the index is derived.
    #[error("index format mismatch: on disk {found}, this build {current} — re-mint from traces")]
    FormatMismatch { found: String, current: String },
    /// No index (or no spec sidecar) at the path. Build one from traces.
    #[error("no trail index at {0} — build one from traces first")]
    Missing(String),
    #[error("search engine: {0}")]
    Engine(String),
    #[error("search io: {0}")]
    Io(String),
}

impl From<std::io::Error> for SearchError {
    fn from(e: std::io::Error) -> Self {
        SearchError::Io(e.to_string())
    }
}

/// Crate result alias.
pub type Result<T> = std::result::Result<T, SearchError>;
