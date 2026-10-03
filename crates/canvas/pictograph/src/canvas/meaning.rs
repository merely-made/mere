// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The Meaning channel: what the nodes say, as one snapshot of pairs, groups
//! and kinds. (Dynamics grammar plan, G2: "'Meaning' as a source".)
//!
//! Each node's text (its title, or its URL when untitled) is embedded once
//! per **content key**, a digest of every node's identity and text. A
//! structural change re-reads the same snapshot; a title edit earns exactly
//! one new run. The snapshot holds the top-k similarity pairs, a Louvain
//! partition over them, and every node's cluster. Kinds, Group pull and the
//! partition G3's grouped laws will read take the clusters; the affinity
//! force takes the pairs, through [`Canvas::set_content_affinity`]. Meaning
//! enters the physics through the ordinary rebuild path ("Snapshots only for
//! now"): no per-step semantic field.
//!
//! The embedder is a [`MeaningEngine`]. The default is lexical feature
//! hashing on the CPU, which is the fallback everywhere and the wasm default.
//! A native host with a renderer installs a sentence model on its own device
//! (feature `meaning-gpu`). On a native, offloaded canvas each run happens on
//! an actor thread ("Burn on the host device, off-path"); otherwise inline, as
//! the community lane does.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use esp::embed::{
    EmbedError, EmbeddingProvider, LexicalEmbeddingProvider, SimilarityMetric, VectorIndex,
};
use kernel::graph::{Graph, NodeKey};

use super::Canvas;
use super::meaning_lane::MeaningActor;
use super::physics_catalog::PhysicsKindSource;
use crate::signals::{ClusterSet, CommunitySnapshot, community_louvain_on_snapshot};

/// The lexical fallback's vector size: the top of ESP's range for short texts.
pub const LEXICAL_DIMENSIONS: usize = 512;

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
}

impl MeaningParams {
    /// The lexical fallback's tuning, the best of a sweep on the topic
    /// fixture (dynamics grammar plan, Findings, G2): at a floor of 0.2 most
    /// titles find no pair and the clusters score no better than chance.
    pub const LEXICAL: MeaningParams = MeaningParams {
        top_k: 4,
        min_similarity: 0.1,
    };
    /// A sentence model's tuning, the best of a sweep of MiniLM on the topic
    /// fixture (dynamics grammar plan, Findings, G2): at a floor of 0.3 the
    /// clusters stay pure but each topic splits in four.
    pub const MODEL: MeaningParams = MeaningParams {
        top_k: 4,
        min_similarity: 0.15,
    };
}

/// What embeds the nodes' texts and finds their pairs.
pub trait MeaningEngine: Send + Sync {
    fn backend(&self) -> MeaningBackend;

    fn params(&self) -> MeaningParams;

    /// One vector per text, in order.
    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbedError>;

    /// Each entry's top-k neighbours above the floor, every unordered pair
    /// once. The CPU scan by default; a device engine may override it.
    fn pairs(
        &self,
        index: &VectorIndex<NodeKey>,
        params: MeaningParams,
    ) -> Vec<(NodeKey, NodeKey, f32)> {
        esp::embed::affinity_pairs(index, params.top_k, params.min_similarity).unwrap_or_default()
    }
}

/// Any embedding provider as an engine, with the CPU pair scan.
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

    /// The lexical fallback.
    pub fn lexical() -> Self {
        let provider = LexicalEmbeddingProvider::new(LEXICAL_DIMENSIONS)
            .expect("a positive lexical dimension");
        Self::new(Box::new(provider), MeaningBackend::Lexical)
    }
}

impl MeaningEngine for ProviderMeaning {
    fn backend(&self) -> MeaningBackend {
        self.backend
    }

    fn params(&self) -> MeaningParams {
        self.params
    }

    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbedError> {
        self.provider.embed(texts)
    }
}

/// One Meaning snapshot: what every consumer of the channel reads.
#[derive(Clone, Debug)]
pub struct MeaningSnapshot {
    /// The content key it was computed for.
    pub content_key: u64,
    /// The engine generation it was computed under.
    pub generation: u64,
    pub backend: MeaningBackend,
    /// Which embedding run produced it, counting from one.
    pub run: u64,
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

/// The digest a snapshot is keyed to: every node's identity and text, in key
/// order. Structure, positions and anything else a node carries are out.
pub fn content_key(graph: &Graph) -> u64 {
    let mut hasher = DefaultHasher::new();
    let mut nodes: Vec<_> = graph.nodes().collect();
    nodes.sort_by_key(|(key, _)| key.index());
    for (_, node) in nodes {
        node.id.hash(&mut hasher);
        meaning_text(node).hash(&mut hasher);
    }
    hasher.finish()
}

/// One run's inputs, `Send` so it can cross to the actor.
pub(crate) struct MeaningRequest {
    pub content_key: u64,
    pub generation: u64,
    pub keys: Vec<NodeKey>,
    pub texts: Vec<String>,
    pub engine: Arc<dyn MeaningEngine>,
    pub runs: Arc<AtomicU64>,
}

impl MeaningRequest {
    fn from_graph(graph: &Graph, state: &MeaningState, content_key: u64) -> Self {
        let mut nodes: Vec<_> = graph.nodes().collect();
        nodes.sort_by_key(|(key, _)| key.index());
        Self {
            content_key,
            generation: state.generation,
            keys: nodes.iter().map(|(key, _)| *key).collect(),
            texts: nodes
                .iter()
                .map(|(_, node)| meaning_text(node).to_string())
                .collect(),
            engine: state.engine.clone(),
            runs: state.runs.clone(),
        }
    }
}

/// Embed, pair and partition: one embedding run.
pub(crate) fn compute_meaning(request: &MeaningRequest) -> Result<MeaningSnapshot, EmbedError> {
    let run = request.runs.fetch_add(1, Ordering::SeqCst) + 1;
    let engine = &request.engine;
    let texts: Vec<&str> = request.texts.iter().map(String::as_str).collect();
    let vectors = if texts.is_empty() {
        Vec::new()
    } else {
        engine.embed(&texts)?
    };
    let dimensions = vectors.first().map_or(1, Vec::len);
    let mut index = VectorIndex::new(dimensions, SimilarityMetric::Cosine);
    for (key, vector) in request.keys.iter().zip(vectors) {
        index
            .insert(*key, vector)
            .map_err(|error| EmbedError::Backend(format!("meaning index: {error:?}")))?;
    }
    let mut pairs = engine.pairs(&index, engine.params());
    for pair in &mut pairs {
        if pair.1 < pair.0 {
            *pair = (pair.1, pair.0, pair.2);
        }
    }
    pairs.sort_by_key(|(a, b, _)| (a.index(), b.index()));
    let clusters = community_louvain_on_snapshot(&CommunitySnapshot::from_weighted_pairs(
        request.keys.clone(),
        &pairs,
    ));
    let mut groups: Vec<(NodeKey, u32)> = clusters
        .clusters
        .iter()
        .enumerate()
        .flat_map(|(i, cluster)| cluster.members.iter().map(move |&key| (key, i as u32)))
        .collect();
    groups.sort_by_key(|(key, _)| key.index());
    Ok(MeaningSnapshot {
        content_key: request.content_key,
        generation: request.generation,
        backend: engine.backend(),
        run,
        pairs,
        clusters,
        groups,
    })
}

/// The canvas's Meaning state: the engine, the accepted snapshot, the run
/// count, and the actor when runs go off-thread.
pub(crate) struct MeaningState {
    engine: Arc<dyn MeaningEngine>,
    /// Bumped when the engine changes, so an older engine's snapshot is stale.
    generation: u64,
    snapshot: Option<MeaningSnapshot>,
    runs: Arc<AtomicU64>,
    actor: Option<MeaningActor>,
    /// Whether the snapshot's pairs feed the affinity force's content signal.
    feeds_affinity: bool,
    /// The `(content key, generation)` whose run failed, not retried until
    /// either moves, and the error.
    failed: Option<(u64, u64, String)>,
    /// The run the last law build read Meaning from. Test introspection.
    #[cfg(test)]
    pub(crate) built_from: Option<u64>,
}

impl Default for MeaningState {
    fn default() -> Self {
        Self {
            engine: Arc::new(ProviderMeaning::lexical()),
            generation: 0,
            snapshot: None,
            runs: Arc::new(AtomicU64::new(0)),
            actor: None,
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

    fn is_fresh(&self, content_key: u64) -> bool {
        self.snapshot
            .as_ref()
            .is_some_and(|s| s.content_key == content_key && s.generation == self.generation)
            || self.failed.as_ref().is_some_and(|(key, generation, _)| {
                *key == content_key && *generation == self.generation
            })
    }
}

impl Canvas {
    /// Install the engine Meaning runs on. The snapshot goes stale and the
    /// next run, if any consumer reads the channel, uses this engine.
    pub fn set_meaning_engine(&mut self, engine: Arc<dyn MeaningEngine>) {
        self.meaning.engine = engine;
        self.meaning.generation += 1;
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

    /// How many embedding runs this canvas has made.
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

    /// Feed the affinity force's content signal from the Meaning snapshot's
    /// pairs (under the [`cluster_by_affinity`](Self::set_cluster_by_affinity)
    /// toggle and its [`AffinityBlend`](crate::canvas::AffinityBlend)). Off
    /// by default; turning it off clears the content signal.
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

    /// Bring the snapshot up to the graph's content: inline, or by a request
    /// to the actor on a native offloaded canvas. Returns whether a new
    /// snapshot was accepted now (inline only).
    pub(crate) fn refresh_meaning(&mut self) -> bool {
        let key = content_key(&self.graph);
        if self.meaning.is_fresh(key) {
            return false;
        }
        let generation = self.meaning.generation;
        match self.offthread_wake.clone() {
            Some(wake) => {
                if self
                    .meaning
                    .actor
                    .as_ref()
                    .is_some_and(|actor| actor.inflight() == Some((key, generation)))
                {
                    return false;
                }
                let request = MeaningRequest::from_graph(&self.graph, &self.meaning, key);
                self.meaning
                    .actor
                    .get_or_insert_with(|| MeaningActor::spawn(wake))
                    .request(request);
                false
            },
            None => {
                let request = MeaningRequest::from_graph(&self.graph, &self.meaning, key);
                let result = compute_meaning(&request);
                self.accept_meaning(key, generation, result)
            },
        }
    }

    /// Keep the channel in step, once a frame: take a finished off-thread
    /// run, start one when the content moved, and hand a new snapshot to its
    /// consumers.
    pub(crate) fn sync_meaning(&mut self) {
        let mut landed = false;
        if let Some(update) = self.meaning.actor.as_mut().and_then(MeaningActor::drain) {
            let key = content_key(&self.graph);
            if update.content_key == key && update.generation == self.meaning.generation {
                landed = self.accept_meaning(key, update.generation, update.result);
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
        key: u64,
        generation: u64,
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
                self.meaning.failed = Some((key, generation, error.to_string()));
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
