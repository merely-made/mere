// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One Meaning run, resumable: embed the texts, search the pairs through
//! ESP's [`AffinityScan`], partition them. Whole on the actor and inline,
//! in slices on wasm (dynamics grammar plan, G2, F35).

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use esp::embed::{
    AffinityScan, EmbedError, SimilarityMetric, SparseIndex, SparseVector, VectorIndex,
};
use kernel::graph::{Graph, NodeKey};

use super::meaning::{
    Embedded, MeaningEngine, MeaningParams, MeaningSnapshot, MeaningState, meaning_text,
};
use crate::signals::{CommunitySnapshot, community_louvain_on_snapshot_at};

/// Texts a slice embeds at most.
const EMBED_CHUNK: usize = 128;

/// One run's inputs, `Send` so it can cross to the actor.
pub(crate) struct MeaningRequest {
    pub content_revision: u64,
    pub generation: u64,
    pub keys: Vec<NodeKey>,
    pub texts: Vec<String>,
    pub engine: Arc<dyn MeaningEngine>,
    pub runs: Arc<AtomicU64>,
}

impl MeaningRequest {
    pub(crate) fn from_graph(graph: &Graph, state: &MeaningState) -> Self {
        let mut nodes: Vec<_> = graph.nodes().collect();
        nodes.sort_by_key(|(key, _)| key.index());
        Self {
            content_revision: graph.content_revision(),
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

    pub(crate) fn tag(&self) -> (u64, u64) {
        (self.content_revision, self.generation)
    }
}

/// The pair search a run is in.
enum Scan {
    Dense(VectorIndex<NodeKey>, AffinityScan<NodeKey, Vec<f32>>),
    Sparse(SparseIndex<NodeKey>, AffinityScan<NodeKey, SparseVector>),
}

enum Stage {
    Embed(Option<Embedded>),
    Scan(Scan),
    Cluster(Vec<(NodeKey, NodeKey, f32)>),
}

/// One embedding run, resumable: embed, search, partition. A run counts once,
/// when it starts.
pub(crate) struct MeaningJob {
    request: MeaningRequest,
    run: u64,
    steps: u32,
    stage: Stage,
}

impl MeaningJob {
    pub(crate) fn start(request: MeaningRequest) -> Self {
        let run = request.runs.fetch_add(1, Ordering::SeqCst) + 1;
        Self {
            request,
            run,
            steps: 0,
            stage: Stage::Embed(None),
        }
    }

    pub(crate) fn tag(&self) -> (u64, u64) {
        self.request.tag()
    }

    /// Do one slice of at most `budget` pair scores (`None`: the rest of the
    /// run). The snapshot when the run is done.
    pub(crate) fn advance(
        &mut self,
        budget: Option<usize>,
    ) -> Option<Result<MeaningSnapshot, EmbedError>> {
        self.steps += 1;
        loop {
            match self.step(budget) {
                Ok(Some(snapshot)) => return Some(Ok(snapshot)),
                Ok(None) if budget.is_some() => return None,
                Ok(None) => {},
                Err(error) => return Some(Err(error)),
            }
        }
    }

    fn step(&mut self, budget: Option<usize>) -> Result<Option<MeaningSnapshot>, EmbedError> {
        let engine = self.request.engine.clone();
        let params = engine.params();
        let n = self.request.keys.len();
        match &mut self.stage {
            Stage::Embed(done) => {
                let have = done.as_ref().map_or(0, Embedded::len);
                let chunk = if budget.is_some() { EMBED_CHUNK } else { n };
                let end = (have + chunk).min(n);
                let texts: Vec<&str> = self.request.texts[have..end]
                    .iter()
                    .map(String::as_str)
                    .collect();
                if !texts.is_empty() {
                    let more = engine.embed(&texts)?;
                    match done {
                        Some(vectors) => vectors.extend(more)?,
                        None => *done = Some(more),
                    }
                }
                if end < n {
                    return Ok(None);
                }
                let vectors = done.take().unwrap_or(Embedded::Sparse(Vec::new()));
                self.stage = self.index(vectors, params)?;
                Ok(None)
            },
            Stage::Scan(scan) => {
                let rows = budget.map_or(usize::MAX, |scores| (scores / n.max(1)).max(1));
                let finished = match scan {
                    Scan::Dense(index, scan) => scan.advance(index, rows),
                    Scan::Sparse(index, scan) => scan.advance(index, rows),
                }
                .map_err(|error| EmbedError::Backend(format!("meaning search: {error:?}")))?;
                if finished {
                    let Stage::Scan(scan) = std::mem::replace(&mut self.stage, Stage::Embed(None))
                    else {
                        unreachable!("in the scan stage");
                    };
                    let pairs = match scan {
                        Scan::Dense(_, scan) => scan.into_pairs(),
                        Scan::Sparse(_, scan) => scan.into_pairs(),
                    };
                    self.stage = Stage::Cluster(pairs);
                }
                Ok(None)
            },
            Stage::Cluster(pairs) => {
                let pairs = std::mem::take(pairs);
                Ok(Some(self.snapshot(pairs)))
            },
        }
    }

    /// The vectors indexed, and the search to run over them.
    fn index(&self, vectors: Embedded, params: MeaningParams) -> Result<Stage, EmbedError> {
        let keys = &self.request.keys;
        let invalid = |error| EmbedError::Backend(format!("meaning index: {error:?}"));
        Ok(match vectors {
            Embedded::Dense(vectors) => {
                let dimensions = vectors.first().map_or(1, Vec::len);
                let mut index = VectorIndex::new(dimensions, SimilarityMetric::Cosine);
                for (key, vector) in keys.iter().zip(vectors) {
                    index.insert(*key, vector).map_err(invalid)?;
                }
                if let Some(pairs) = self.request.engine.index_pairs(&index, params) {
                    Stage::Cluster(pairs)
                } else {
                    let scan = AffinityScan::new(&index, params.top_k, params.min_similarity);
                    Stage::Scan(Scan::Dense(index, scan))
                }
            },
            Embedded::Sparse(vectors) => {
                let dimensions = vectors.first().map_or(1, SparseVector::dimensions);
                let mut index = SparseIndex::new(dimensions, SimilarityMetric::Cosine);
                for (key, vector) in keys.iter().zip(vectors) {
                    index.insert(*key, vector).map_err(invalid)?;
                }
                let scan = AffinityScan::new(&index, params.top_k, params.min_similarity);
                Stage::Scan(Scan::Sparse(index, scan))
            },
        })
    }

    fn snapshot(&self, mut pairs: Vec<(NodeKey, NodeKey, f32)>) -> MeaningSnapshot {
        for pair in &mut pairs {
            if pair.1 < pair.0 {
                *pair = (pair.1, pair.0, pair.2);
            }
        }
        pairs.sort_by_key(|(a, b, _)| (a.index(), b.index()));
        let clusters = community_louvain_on_snapshot_at(
            &CommunitySnapshot::from_weighted_pairs(self.request.keys.clone(), &pairs),
            f64::from(self.request.engine.params().resolution),
        );
        let mut groups: Vec<(NodeKey, u32)> = clusters
            .clusters
            .iter()
            .enumerate()
            .flat_map(|(i, cluster)| cluster.members.iter().map(move |&key| (key, i as u32)))
            .collect();
        groups.sort_by_key(|(key, _)| key.index());
        MeaningSnapshot {
            content_revision: self.request.content_revision,
            generation: self.request.generation,
            backend: self.request.engine.backend(),
            run: self.run,
            steps: self.steps,
            pairs,
            clusters,
            groups,
        }
    }
}

/// One whole embedding run.
pub(crate) fn compute_meaning(request: MeaningRequest) -> Result<MeaningSnapshot, EmbedError> {
    MeaningJob::start(request)
        .advance(None)
        .expect("a whole run finishes")
}
