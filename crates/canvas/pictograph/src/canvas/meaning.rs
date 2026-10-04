// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The Meaning channel: what the nodes say, as one snapshot of pairs, groups
//! and kinds. (Dynamics grammar plan, G2: "'Meaning' as a source".)
//!
//! Each node's text (its title, or its URL when untitled) is embedded once
//! per **content revision**, the graph kernel's counter of node-text changes
//! (`Graph::content_revision`, F33). A structural change re-reads the same
//! snapshot; a title edit earns exactly one new run. The snapshot holds the
//! top-k similarity pairs, a Louvain partition over them, and every node's
//! cluster. Kinds, Group pull and the partition G3's grouped laws will read
//! take the clusters; the affinity force takes the pairs, through
//! [`Canvas::set_content_affinity`]. Meaning enters the physics through the
//! ordinary rebuild path ("Snapshots only for now"): no per-step semantic
//! field.
//!
//! The embedder is a [`MeaningEngine`]. The default, [`LexicalMeaning`], is
//! feature hashing on the CPU searched through ESP's sparse index (F35): the
//! fallback everywhere and the wasm default. A native host with a renderer
//! installs a sentence model on its own device (feature `meaning-gpu`). Where
//! a run happens:
//!
//! - on a native, offloaded canvas, on the Meaning actor ("Burn on the host
//!   device, off-path");
//! - on wasm, in slices across frames, a bounded amount of work each frame
//!   ([`Canvas::set_meaning_slice`], F35's "off-frame");
//! - otherwise (tests, an unoffloaded native canvas) inline, whole.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use esp::embed::{
    EmbedError, EmbeddingProvider, LexicalEmbeddingProvider, SparseVector, VectorIndex,
};
use kernel::graph::NodeKey;

use super::Canvas;
pub(crate) use super::meaning_job::{MeaningJob, MeaningRequest, compute_meaning};
use super::meaning_lane::MeaningActor;
use super::physics_catalog::PhysicsKindSource;
use crate::signals::ClusterSet;

/// The lexical fallback's vector size: the top of ESP's range for short texts.
pub const LEXICAL_DIMENSIONS: usize = 512;

/// The wasm default for [`Canvas::set_meaning_slice`]: pair scores a frame
/// may spend. Lexical vectors are sparse, so a score is a merge of a few
/// dozen entries.
pub const DEFAULT_MEANING_SLICE: usize = 40_000;

/// What computed a snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MeaningBackend {
    /// Feature hashing of the words, on the CPU: the fallback, and the wasm
    /// default.
    Lexical,
    /// A sentence model on the CPU.
    ModelCpu,
    /// A sentence model on the host's GPU device.
    ModelGpu,
}

impl MeaningBackend {
    pub fn id(self) -> &'static str {
        match self {
            MeaningBackend::Lexical => "lexical",
            MeaningBackend::ModelCpu => "model-cpu",
            MeaningBackend::ModelGpu => "model-gpu",
        }
    }
}

/// How a snapshot turns vectors into pairs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeaningParams {
    /// Each node's nearest neighbours considered.
    pub top_k: usize,
    /// The least cosine similarity a pair needs.
    pub min_similarity: f32,
    /// The partition's modularity resolution γ: 1 is classical modularity,
    /// above it smaller clusters, below it larger (F50).
    pub resolution: f32,
}

impl MeaningParams {
    /// The lexical fallback's tuning, the best F of a sweep on the topic
    /// fixture (F34; dynamics grammar plan, Findings, G2): at a floor of 0.2
    /// most titles find no pair and the clusters score no better than chance.
    pub const LEXICAL: MeaningParams = MeaningParams {
        top_k: 4,
        min_similarity: 0.1,
        resolution: 1.0,
    };
    /// A sentence model's tuning, the best F of a sweep of MiniLM on the
    /// topic fixture (F34): at a floor of 0.3 the clusters stay pure but each
    /// topic splits in four.
    pub const MODEL: MeaningParams = MeaningParams {
        top_k: 4,
        min_similarity: 0.15,
        resolution: 1.0,
    };
}

/// One embedding run's vectors.
#[derive(Clone, Debug)]
pub enum Embedded {
    Dense(Vec<Vec<f32>>),
    Sparse(Vec<SparseVector>),
}

impl Embedded {
    pub(crate) fn len(&self) -> usize {
        match self {
            Embedded::Dense(vectors) => vectors.len(),
            Embedded::Sparse(vectors) => vectors.len(),
        }
    }

    pub(crate) fn extend(&mut self, more: Embedded) -> Result<(), EmbedError> {
        match (self, more) {
            (Embedded::Dense(a), Embedded::Dense(b)) => a.extend(b),
            (Embedded::Sparse(a), Embedded::Sparse(b)) => a.extend(b),
            _ => {
                return Err(EmbedError::Backend(
                    "an engine returned dense and sparse vectors in one run".to_string(),
                ));
            },
        }
        Ok(())
    }
}

/// What embeds the nodes' texts and finds their pairs.
pub trait MeaningEngine: Send + Sync {
    fn backend(&self) -> MeaningBackend;

    fn params(&self) -> MeaningParams;

    /// One vector per text, in order.
    fn embed(&self, texts: &[&str]) -> Result<Embedded, EmbedError>;

    /// The pairs of a whole dense index at once, when this engine has a
    /// faster path than the resumable row scan (a device kernel); `None`
    /// takes the scan.
    fn index_pairs(
        &self,
        _index: &VectorIndex<NodeKey>,
        _params: MeaningParams,
    ) -> Option<Vec<(NodeKey, NodeKey, f32)>> {
        None
    }
}

/// The lexical fallback: hashed words, searched through the sparse index.
pub struct LexicalMeaning {
    provider: LexicalEmbeddingProvider,
    params: MeaningParams,
}

impl LexicalMeaning {
    pub fn new() -> Self {
        Self {
            provider: LexicalEmbeddingProvider::new(LEXICAL_DIMENSIONS)
                .expect("a positive lexical dimension"),
            params: MeaningParams::LEXICAL,
        }
    }

    pub fn with_params(mut self, params: MeaningParams) -> Self {
        self.params = params;
        self
    }
}

impl Default for LexicalMeaning {
    fn default() -> Self {
        Self::new()
    }
}

impl MeaningEngine for LexicalMeaning {
    fn backend(&self) -> MeaningBackend {
        MeaningBackend::Lexical
    }

    fn params(&self) -> MeaningParams {
        self.params
    }

    fn embed(&self, texts: &[&str]) -> Result<Embedded, EmbedError> {
        Ok(Embedded::Sparse(
            texts
                .iter()
                .map(|text| self.provider.embed_sparse_one(text))
                .collect(),
        ))
    }
}

/// Any embedding provider as an engine, with the CPU row scan.
pub struct ProviderMeaning {
    provider: Box<dyn EmbeddingProvider>,
    backend: MeaningBackend,
    params: MeaningParams,
}

impl ProviderMeaning {
    pub fn new(provider: Box<dyn EmbeddingProvider>, backend: MeaningBackend) -> Self {
        let params = match backend {
            MeaningBackend::Lexical => MeaningParams::LEXICAL,
            MeaningBackend::ModelCpu | MeaningBackend::ModelGpu => MeaningParams::MODEL,
        };
        Self {
            provider,
            backend,
            params,
        }
    }

    pub fn with_params(mut self, params: MeaningParams) -> Self {
        self.params = params;
        self
    }
}

impl MeaningEngine for ProviderMeaning {
    fn backend(&self) -> MeaningBackend {
        self.backend
    }

    fn params(&self) -> MeaningParams {
        self.params
    }

    fn embed(&self, texts: &[&str]) -> Result<Embedded, EmbedError> {
        self.provider.embed(texts).map(Embedded::Dense)
    }
}

/// One Meaning snapshot: what every consumer of the channel reads.
#[derive(Clone, Debug)]
pub struct MeaningSnapshot {
    /// The graph's content revision it was computed for.
    pub content_revision: u64,
    /// The engine and graph generation it was computed under.
    pub generation: u64,
    pub backend: MeaningBackend,
    /// Which embedding run produced it, counting from one.
    pub run: u64,
    /// How many slices the run took (one when it ran whole).
    pub steps: u32,
    /// Similarity pairs, sorted, each unordered pair once.
    pub pairs: Vec<(NodeKey, NodeKey, f32)>,
    /// The Louvain partition of the pairs.
    pub clusters: ClusterSet,
    /// Every node's cluster index, in key order.
    pub groups: Vec<(NodeKey, u32)>,
}

/// The text a node contributes: its title, or its URL when untitled.
pub(crate) fn meaning_text(node: &kernel::graph::Node) -> &str {
    if node.title.is_empty() {
        node.url()
    } else {
        node.title.as_str()
    }
}

/// The canvas's Meaning state: the engine, the accepted snapshot, the run
/// count, and where a run happens.
pub(crate) struct MeaningState {
    pub(crate) engine: Arc<dyn MeaningEngine>,
    /// Bumped when the engine or the whole graph changes, so a snapshot of
    /// the old one is stale whatever its content revision reads.
    pub(crate) generation: u64,
    snapshot: Option<MeaningSnapshot>,
    pub(crate) runs: Arc<AtomicU64>,
    actor: Option<MeaningActor>,
    /// A sliced run in progress (wasm, or a host that asked for slices).
    pending: Option<MeaningJob>,
    /// Pair scores a frame may spend on a run; `None` runs it whole.
    slice: Option<usize>,
    /// Whether the snapshot's pairs feed the affinity force's content signal.
    feeds_affinity: bool,
    /// The `(content revision, generation)` whose run failed, not retried
    /// until either moves, and the error.
    failed: Option<(u64, u64, String)>,
    /// The run the last law build read Meaning from. Test introspection.
    #[cfg(test)]
    pub(crate) built_from: Option<u64>,
}

impl Default for MeaningState {
    fn default() -> Self {
        Self {
            engine: Arc::new(LexicalMeaning::new()),
            generation: 0,
            snapshot: None,
            runs: Arc::new(AtomicU64::new(0)),
            actor: None,
            pending: None,
            slice: cfg!(target_arch = "wasm32").then_some(DEFAULT_MEANING_SLICE),
            feeds_affinity: false,
            failed: None,
            #[cfg(test)]
            built_from: None,
        }
    }
}

impl MeaningState {
    pub(crate) fn snapshot(&self) -> Option<&MeaningSnapshot> {
        self.snapshot.as_ref()
    }

    fn is_fresh(&self, tag: (u64, u64)) -> bool {
        self.snapshot
            .as_ref()
            .is_some_and(|s| (s.content_revision, s.generation) == tag)
            || self
                .failed
                .as_ref()
                .is_some_and(|(revision, generation, _)| (*revision, *generation) == tag)
    }

    /// The whole graph was replaced: its content revision counts from its own
    /// origin, so nothing computed for the old graph stands.
    pub(crate) fn forget_graph(&mut self) {
        self.generation += 1;
        self.snapshot = None;
        self.pending = None;
        self.failed = None;
    }
}

impl Canvas {
    /// Install the engine Meaning runs on. The snapshot goes stale and the
    /// next run, if any consumer reads the channel, uses this engine.
    pub fn set_meaning_engine(&mut self, engine: Arc<dyn MeaningEngine>) {
        self.meaning.engine = engine;
        self.meaning.generation += 1;
        self.meaning.pending = None;
        self.meaning.failed = None;
    }

    /// What the next run computes on.
    pub fn meaning_backend(&self) -> MeaningBackend {
        self.meaning.engine.backend()
    }

    /// The accepted snapshot, if any.
    pub fn meaning(&self) -> Option<&MeaningSnapshot> {
        self.meaning.snapshot.as_ref()
    }

    /// How many embedding runs this canvas has started.
    pub fn meaning_runs(&self) -> u64 {
        self.meaning.runs.load(Ordering::SeqCst)
    }

    /// The last run's error, if it failed.
    pub fn meaning_error(&self) -> Option<&str> {
        self.meaning
            .failed
            .as_ref()
            .map(|(_, _, error)| error.as_str())
    }

    /// Spread an inline run over frames, at most `scores` pair scores a frame
    /// (`None`: run it whole when it starts). Wasm defaults to
    /// [`DEFAULT_MEANING_SLICE`], so a large graph's run leaves the frame
    /// (F35); an offloaded native canvas runs on its actor either way.
    pub fn set_meaning_slice(&mut self, scores: Option<usize>) {
        self.meaning.slice = scores.map(|scores| scores.max(1));
    }

    pub fn meaning_slice(&self) -> Option<usize> {
        self.meaning.slice
    }

    /// Whether a sliced run is in progress.
    pub fn meaning_pending(&self) -> bool {
        self.meaning.pending.is_some()
    }

    /// Feed the affinity force's content signal from the Meaning snapshot's
    /// pairs (under the [`cluster_by_affinity`](Self::set_cluster_by_affinity)
    /// toggle and its [`AffinityBlend`](crate::canvas::AffinityBlend)). Off
    /// by default (F36); turning it off clears the content signal.
    pub fn set_meaning_affinity(&mut self, on: bool) {
        if self.meaning.feeds_affinity == on {
            return;
        }
        self.meaning.feeds_affinity = on;
        if on {
            if let Some(pairs) = self.meaning.snapshot.as_ref().map(|s| s.pairs.clone()) {
                self.set_content_affinity(Some(pairs));
            }
        } else {
            self.set_content_affinity(None);
        }
    }

    pub fn meaning_affinity(&self) -> bool {
        self.meaning.feeds_affinity
    }

    /// Whether any consumer reads the Meaning channel now.
    pub(crate) fn wants_meaning(&self) -> bool {
        self.physics_reads(PhysicsKindSource::Meaning)
            || (self.meaning.feeds_affinity && self.cluster_by_affinity())
    }

    /// Bring the snapshot up to the graph's content revision: a request to
    /// the actor on a native offloaded canvas, one slice of a sliced run, or
    /// the whole run inline. Returns whether a new snapshot was accepted now.
    pub(crate) fn refresh_meaning(&mut self) -> bool {
        let tag = (self.graph.content_revision(), self.meaning.generation);
        if self.meaning.is_fresh(tag) {
            self.meaning.pending = None;
            return false;
        }
        if let Some(wake) = self.offthread_wake.clone() {
            if self
                .meaning
                .actor
                .as_ref()
                .is_some_and(|actor| actor.inflight() == Some(tag))
            {
                return false;
            }
            let request = MeaningRequest::from_graph(&self.graph, &self.meaning);
            self.meaning
                .actor
                .get_or_insert_with(|| MeaningActor::spawn(wake))
                .request(request);
            return false;
        }
        let Some(budget) = self.meaning.slice else {
            let request = MeaningRequest::from_graph(&self.graph, &self.meaning);
            return self.accept_meaning(tag, compute_meaning(request));
        };
        if self
            .meaning
            .pending
            .as_ref()
            .is_none_or(|job| job.tag() != tag)
        {
            let request = MeaningRequest::from_graph(&self.graph, &self.meaning);
            self.meaning.pending = Some(MeaningJob::start(request));
        }
        let finished = self
            .meaning
            .pending
            .as_mut()
            .and_then(|job| job.advance(Some(budget)));
        match finished {
            Some(result) => {
                self.meaning.pending = None;
                self.accept_meaning(tag, result)
            },
            None => false,
        }
    }

    /// Keep the channel in step, once a frame: take a finished off-thread
    /// run, start or continue one when the content moved, and hand a new
    /// snapshot to its consumers.
    pub(crate) fn sync_meaning(&mut self) {
        let mut landed = false;
        if let Some(update) = self.meaning.actor.as_mut().and_then(MeaningActor::drain) {
            let tag = (self.graph.content_revision(), self.meaning.generation);
            if (update.content_revision, update.generation) == tag {
                landed = self.accept_meaning(tag, update.result);
            }
        }
        if self.wants_meaning() {
            landed |= self.refresh_meaning();
        }
        if landed {
            self.apply_meaning();
        }
    }

    fn accept_meaning(
        &mut self,
        (revision, generation): (u64, u64),
        result: Result<MeaningSnapshot, EmbedError>,
    ) -> bool {
        match result {
            Ok(snapshot) => {
                let pairs = self.meaning.feeds_affinity.then(|| snapshot.pairs.clone());
                self.meaning.snapshot = Some(snapshot);
                self.meaning.failed = None;
                if let Some(pairs) = pairs {
                    self.set_content_affinity(Some(pairs));
                }
                true
            },
            Err(error) => {
                self.meaning.failed = Some((revision, generation, error.to_string()));
                false
            },
        }
    }

    /// A snapshot that landed outside a law build reaches the law and
    /// overlays that read the channel. (The affinity signal took its pairs on
    /// acceptance.)
    fn apply_meaning(&mut self) {
        if self.physics_reads(PhysicsKindSource::Meaning) {
            self.rebuild_law_forces();
            self.settle_for_law();
        }
    }
}
