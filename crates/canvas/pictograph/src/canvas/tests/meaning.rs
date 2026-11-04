// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The channel registry and the Meaning channel (dynamics grammar plan, G2):
//! every channel resolves by id, Group pull by cluster reads the partition
//! Columns (by cluster) lays out, and one Meaning snapshot per content key
//! feeds Kinds, Group pull, the affinity force and the groups channel. The
//! lexical fallback's purity on the topic fixture is recorded here; the
//! sentence model's, on the host's device, in `tests/meaning_device.rs`.

use super::meaning_topics::{community, f_measure, inverse_purity, partition, purity, topic_graph};
use super::*;
use crate::canvas::channels::{Channel, ChannelFamily, ChannelValues};
use crate::canvas::meaning::{
    LexicalMeaning, MeaningBackend, MeaningEngine, MeaningParams, MeaningRequest, ProviderMeaning,
    compute_meaning,
};
use crate::canvas::physics_catalog::{PhysicsKindSource, PhysicsLaw, PhysicsOverlay};
use crate::canvas::tests::ThroughView;

fn topic_canvas() -> (Canvas, Vec<NodeKey>, HashMap<NodeKey, usize>) {
    let (graph, keys, topics) = topic_graph();
    (Canvas::with_graph(graph), keys, topics)
}

/// Steps the with/without runs take, so each is read at rest (F176). Under
/// rapier 0.36, Springs on the topic fixture rests from a scatter by about
/// 10 000 (energy 12.5 at 6 000, 0.009 at 10 000) and Group pull by cluster by
/// about 18 000 (326 at 6 000, 1.3 at 12 000, 0.002 at 18 000).
const SETTLED_STEPS: usize = 18_000;

/// Seed every node at a fixed pseudo-random spot in a 640-wide square, the
/// same for every run and blind to every partition.
fn scatter(canvas: &mut Canvas) {
    let mut state: u64 = 0x5CA7;
    let mut next = || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((state >> 33) as f32 / (1u64 << 31) as f32) * 640.0 - 320.0
    };
    let mut keys: Vec<NodeKey> = canvas.graph().nodes().map(|(k, _)| k).collect();
    keys.sort_by_key(|k| k.index());
    let seeds: Vec<(NodeKey, Point2D<f32>)> = keys
        .into_iter()
        .map(|k| (k, Point2D::new(next(), next())))
        .collect();
    for &(key, at) in &seeds {
        canvas.view.set_position(key, at);
    }
    canvas.physics.seed(seeds);
}

/// Step the inline physics `count` times, as frames do, without painting.
fn steps(canvas: &mut Canvas, count: usize) {
    for _ in 0..count {
        canvas.physics.advance_frame(&mut canvas.view);
    }
}

/// Between-group centroid distance over within-group spread, for `groups`
/// at the canvas's current positions.
fn separation(canvas: &Canvas, groups: &[(NodeKey, u32)]) -> f32 {
    let at: HashMap<NodeKey, Point2D<f32>> = canvas.view.positions().collect();
    let mut members: HashMap<u32, Vec<Point2D<f32>>> = HashMap::new();
    for (key, group) in groups {
        members.entry(*group).or_default().push(at[key]);
    }
    let centroids: Vec<(Point2D<f32>, f32)> = members
        .values()
        .map(|points| {
            let n = points.len() as f32;
            let c = points
                .iter()
                .fold(Point2D::<f32>::zero(), |acc, p| acc + p.to_vector() / n);
            let spread = (points.iter().map(|p| (*p - c).square_length()).sum::<f32>() / n).sqrt();
            (c, spread)
        })
        .collect();
    let mut gap = 0.0;
    let mut pairs = 0.0;
    for i in 0..centroids.len() {
        for j in (i + 1)..centroids.len() {
            gap += (centroids[i].0 - centroids[j].0).length();
            pairs += 1.0;
        }
    }
    let within = centroids.iter().map(|(_, s)| s).sum::<f32>() / centroids.len() as f32;
    (gap / pairs) / within.max(1e-3)
}

/// What one with/without run reads.
struct Reading {
    compact_cluster: f32,
    compact_site: f32,
    agree_cluster: f32,
    agree_site: f32,
    separation: f32,
    stats: crate::canvas::LayoutStats,
}

/// Mean within-group RMS spread over the whole layout's RMS spread: lower is
/// tighter groups.
fn compactness(canvas: &Canvas, groups: &[(NodeKey, u32)]) -> f32 {
    let at: HashMap<NodeKey, Point2D<f32>> = canvas.view.positions().collect();
    let rms = |points: &[Point2D<f32>]| {
        let n = points.len() as f32;
        let c = points
            .iter()
            .fold(Point2D::<f32>::zero(), |acc, p| acc + p.to_vector() / n);
        (points.iter().map(|p| (*p - c).square_length()).sum::<f32>() / n).sqrt()
    };
    let mut members: HashMap<u32, Vec<Point2D<f32>>> = HashMap::new();
    for (key, group) in groups {
        members.entry(*group).or_default().push(at[key]);
    }
    let within = members.values().map(|p| rms(p)).sum::<f32>() / members.len() as f32;
    let all: Vec<Point2D<f32>> = at.values().copied().collect();
    within / rms(&all).max(1e-3)
}

/// The mean share of each node's three nearest neighbours (in canvas space)
/// that share its group: one for a layout that gathers every group, about a
/// group's share of the nodes for one blind to the groups.
fn agreement(canvas: &Canvas, groups: &[(NodeKey, u32)]) -> f32 {
    let of: HashMap<NodeKey, u32> = groups.iter().copied().collect();
    let at: Vec<(NodeKey, Point2D<f32>)> = canvas.view.positions().collect();
    let mut total = 0.0;
    for (key, p) in &at {
        let mut near: Vec<(f32, NodeKey)> = at
            .iter()
            .filter(|(other, _)| other != key)
            .map(|(other, q)| ((*q - *p).length(), *other))
            .collect();
        near.sort_by(|a, b| a.0.total_cmp(&b.0));
        let same = near
            .iter()
            .take(3)
            .filter(|(_, other)| of[other] == of[key])
            .count();
        total += same as f32 / 3.0;
    }
    total / at.len() as f32
}

/// One row of the purity receipt: purity, inverse purity, their harmonic
/// mean, the group count, and the harmonic mean against shuffled topics.
pub(crate) fn purity_row(
    name: &str,
    groups: &[(NodeKey, u32)],
    topics: &HashMap<NodeKey, usize>,
    shuffled: &HashMap<NodeKey, usize>,
) -> (f64, f64) {
    let f = f_measure(groups, topics);
    let control = f_measure(groups, shuffled);
    println!(
        "{name}: purity {:.3}, inverse purity {:.3}, F {f:.3}, groups {}, F against shuffled topics {control:.3}",
        purity(groups, topics),
        inverse_purity(groups, topics),
        partition(groups).len(),
    );
    (f, control)
}

/// Print each large group's counts by category: where a partition puts each
/// category, for the receipts.
pub(crate) fn print_confusion(
    name: &str,
    groups: &[(NodeKey, u32)],
    topics: &HashMap<NodeKey, usize>,
) {
    use super::meaning_topics::{ARXIV_CATEGORIES, confusion};
    println!(
        "{name}: groups holding at least 2% of the titles, counts by category {ARXIV_CATEGORIES:?}"
    );
    for (size, counts) in confusion(groups, topics, ARXIV_CATEGORIES.len(), 0.02) {
        println!("  group of {size:>3}: {counts:?}");
    }
}

/// F34's lexical bar on the arXiv fixture (F42, F43): the lexical fallback
/// beats the shuffled-topic control, site and structure. On a flat title list
/// site and structure are the two degenerate partitions: every title is on
/// `arxiv.org` (one group) and no title links another (Louvain leaves every
/// title alone), so each stands for a channel that knows nothing of topics.
#[test]
fn the_lexical_fallback_records_its_purity_on_the_arxiv_fixture() {
    use super::meaning_topics::{arxiv_graph, shuffled};
    let (graph, keys, topics) = arxiv_graph();
    let mut canvas = Canvas::with_graph(graph);
    canvas.pick_kind(PhysicsKindSource::Meaning);
    canvas.pick_law(PhysicsLaw::Kinds).expect("not refused");
    let snapshot = canvas
        .meaning()
        .expect("Kinds by meaning embeds at build")
        .clone();
    assert_eq!(snapshot.backend, MeaningBackend::Lexical);
    canvas.refresh_community_cache("kanban.community");
    let shuffled = shuffled(&keys, &topics);
    let meaning = purity_row(
        "arxiv lexical meaning",
        &snapshot.groups,
        &topics,
        &shuffled,
    );
    let site_groups = canvas.channel_groups(PhysicsKindSource::Site);
    assert_eq!(
        partition(&site_groups).len(),
        1,
        "every title is on arxiv.org"
    );
    let site = purity_row("arxiv site (one group)", &site_groups, &topics, &shuffled);
    let cluster_groups = canvas.channel_groups(PhysicsKindSource::Cluster);
    assert_eq!(
        partition(&cluster_groups).len(),
        keys.len(),
        "no title links another"
    );
    let cluster = purity_row(
        "arxiv structure (every title alone)",
        &cluster_groups,
        &topics,
        &shuffled,
    );
    println!(
        "arxiv lexical meaning: {} pairs, mean weight {:.3}, params {:?}",
        snapshot.pairs.len(),
        snapshot.pairs.iter().map(|p| p.2).sum::<f32>() / snapshot.pairs.len().max(1) as f32,
        MeaningParams::LEXICAL,
    );
    print_confusion("arxiv lexical meaning", &snapshot.groups, &topics);
    // The tuning's neighbourhood, for the record.
    let texts: Vec<String> = keys
        .iter()
        .map(|k| canvas.graph().get_node(*k).unwrap().title.clone())
        .collect();
    for top_k in [2, 4, 8, 16] {
        for min_similarity in [0.05, 0.1, 0.15, 0.2, 0.3] {
            let params = MeaningParams {
                top_k,
                min_similarity,
                resolution: 1.0,
            };
            let request = MeaningRequest {
                content_revision: 0,
                generation: 0,
                keys: keys.clone(),
                texts: texts.clone(),
                engine: std::sync::Arc::new(LexicalMeaning::new().with_params(params)),
                runs: Default::default(),
            };
            let swept = compute_meaning(request).unwrap();
            purity_row(
                &format!("arxiv lexical sweep top_k {top_k} min {min_similarity}"),
                &swept.groups,
                &topics,
                &shuffled,
            );
        }
    }
    // F34: the lexical fallback beats the controls.
    assert!(meaning.0 > meaning.1, "beats shuffled topics: {meaning:?}");
    assert!(
        meaning.0 > site.0,
        "beats site: {meaning:?} against {site:?}"
    );
    assert!(
        meaning.0 > cluster.0,
        "beats structure: {meaning:?} against {cluster:?}"
    );
}

/// The request every engine-comparison receipt runs: the topic fixture's
/// titles, in key order.
fn topic_request(engine: std::sync::Arc<dyn MeaningEngine>) -> MeaningRequest {
    let (graph, keys, _) = topic_graph();
    MeaningRequest {
        content_revision: graph.content_revision(),
        generation: 0,
        keys: keys.clone(),
        texts: keys
            .iter()
            .map(|k| graph.get_node(*k).unwrap().title.clone())
            .collect(),
        engine,
        runs: Default::default(),
    }
}

/// F35: lexical pairs come from ESP's sparse index, and they are the dense
/// index's pairs exactly (the same hashed vectors, bit-identical scores), so
/// the recorded purity stands; the control is the dense run itself.
#[test]
fn the_sparse_lexical_search_matches_the_dense_one() {
    let sparse =
        compute_meaning(topic_request(std::sync::Arc::new(LexicalMeaning::new()))).unwrap();
    let dense_engine = ProviderMeaning::new(
        Box::new(
            esp::embed::LexicalEmbeddingProvider::new(crate::canvas::meaning::LEXICAL_DIMENSIONS)
                .unwrap(),
        ),
        MeaningBackend::Lexical,
    );
    let dense = compute_meaning(topic_request(std::sync::Arc::new(dense_engine))).unwrap();
    assert!(!sparse.pairs.is_empty());
    assert_eq!(
        sparse.pairs, dense.pairs,
        "the same pairs, the same weights"
    );
    assert_eq!(sparse.groups, dense.groups);
}

/// F35's off-frame run: sliced, a run spreads over frames, a bounded number
/// of pair scores each, and lands with the snapshot a whole run gives; it
/// counts once. While it is in flight Kinds by meaning reads site.
#[test]
fn a_sliced_run_lands_over_frames_with_the_whole_runs_snapshot() {
    let (mut whole, _, _) = topic_canvas();
    whole.pick_kind(PhysicsKindSource::Meaning);
    whole.pick_law(PhysicsLaw::Kinds).expect("not refused");
    let expected = whole.meaning().expect("inline: whole at build").clone();
    assert_eq!(expected.steps, 1);

    let (mut canvas, _, _) = topic_canvas();
    // 32 nodes: 64 scores is two rows a frame, so the scan alone is 16 slices.
    canvas.set_meaning_slice(Some(64));
    canvas.pick_kind(PhysicsKindSource::Meaning);
    canvas.pick_law(PhysicsLaw::Kinds).expect("not refused");
    assert!(
        canvas.meaning().is_none(),
        "the build did one slice, not the run"
    );
    assert!(canvas.meaning_pending());
    let mut frames = 0;
    while canvas.meaning().is_none() {
        canvas.frame(800, 600);
        frames += 1;
        assert!(frames < 100, "the sliced run landed");
    }
    let sliced = canvas.meaning().unwrap();
    println!("sliced run: {} steps, {frames} frames", sliced.steps);
    assert!(
        sliced.steps >= 17,
        "embed, sixteen scan slices, partition: {}",
        sliced.steps
    );
    assert_eq!(sliced.pairs, expected.pairs);
    assert_eq!(sliced.groups, expected.groups);
    assert_eq!(canvas.meaning_runs(), 1, "a sliced run counts once");
    assert!(!canvas.meaning_pending());
    assert_eq!(
        canvas.meaning.built_from,
        Some(1),
        "the landing rebuilt the law"
    );
}

/// F33's key is the graph's own counter, so a different graph can read the
/// same value: replacing the graph forgets the snapshot (the control: the
/// counters match, yet the run happens).
#[test]
fn a_replaced_graph_earns_its_own_run_whatever_its_revision_reads() {
    let (mut canvas, _, _) = topic_canvas();
    canvas.pick_kind(PhysicsKindSource::Meaning);
    canvas.pick_law(PhysicsLaw::Kinds).expect("not refused");
    assert_eq!(canvas.meaning_runs(), 1);
    let before = canvas.meaning().unwrap().generation;
    let (other, _, _) = topic_graph();
    assert_eq!(
        other.content_revision(),
        canvas.graph().content_revision(),
        "the counters agree"
    );
    canvas.set_graph(other);
    // The swap rebuilds the graph-bound law, which runs Meaning for the new
    // graph at once rather than reading the old graph's snapshot.
    for _ in 0..3 {
        canvas.frame(800, 600);
    }
    assert_eq!(canvas.meaning_runs(), 2, "the new graph's own run");
    let after = canvas.meaning().expect("the new graph's snapshot");
    assert_eq!(after.run, 2);
    assert!(
        after.generation > before,
        "computed under the new graph's generation"
    );
}

/// F35's cost receipt: a lexical run's time by graph size, whole through the
/// sparse search, beside the dense search it replaced, and the slowest slice
/// at the wasm default. Run in release:
/// `cargo test --release -p pictograph --features canvas --lib -- --ignored meaning_cost --nocapture`
#[test]
#[ignore = "a timing receipt: run in release"]
fn meaning_cost_by_graph_size() {
    use std::time::{Duration, Instant};

    use super::meaning_topics::TITLES;
    use crate::canvas::meaning::{DEFAULT_MEANING_SLICE, LEXICAL_DIMENSIONS, MeaningJob};
    println!(
        "profile: {}",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );
    for n in [32usize, 500, 2000] {
        let keys: Vec<NodeKey> = (0..n).map(NodeKey::new).collect();
        let texts: Vec<String> = (0..n)
            .map(|i| format!("{} {i}", TITLES[(i / 8) % 4][i % 8]))
            .collect();
        let request = |engine: std::sync::Arc<dyn MeaningEngine>| MeaningRequest {
            content_revision: 0,
            generation: 0,
            keys: keys.clone(),
            texts: texts.clone(),
            engine,
            runs: Default::default(),
        };
        let started = Instant::now();
        let sparse = compute_meaning(request(std::sync::Arc::new(LexicalMeaning::new()))).unwrap();
        let sparse_time = started.elapsed();
        let dense_engine = ProviderMeaning::new(
            Box::new(esp::embed::LexicalEmbeddingProvider::new(LEXICAL_DIMENSIONS).unwrap()),
            MeaningBackend::Lexical,
        );
        let started = Instant::now();
        let dense = compute_meaning(request(std::sync::Arc::new(dense_engine))).unwrap();
        let dense_time = started.elapsed();
        assert_eq!(sparse.pairs, dense.pairs, "{n} nodes: the same pairs");
        let mut job = MeaningJob::start(request(std::sync::Arc::new(LexicalMeaning::new())));
        let mut slices: Vec<Duration> = Vec::new();
        loop {
            let started = Instant::now();
            let done = job.advance(Some(DEFAULT_MEANING_SLICE));
            slices.push(started.elapsed());
            if let Some(result) = done {
                assert_eq!(result.unwrap().pairs, sparse.pairs);
                break;
            }
        }
        let partition = slices.last().copied().unwrap_or_default();
        let mut sorted = slices.clone();
        sorted.sort();
        println!(
            "lexical cost, {n} nodes: sparse whole {sparse_time:?}, dense whole {dense_time:?}; \
             sliced at {DEFAULT_MEANING_SLICE} scores: {} slices, p95 {:?}, slowest {:?}, the \
             partition slice {partition:?}; {} pairs, {} groups",
            slices.len(),
            sorted[(sorted.len() * 95 / 100).min(sorted.len() - 1)],
            sorted.last().unwrap(),
            sparse.pairs.len(),
            sparse.clusters.clusters.len(),
        );
    }
}

/// F50's knob: the partition's resolution. At 1 it is the classical
/// modularity Louvain always ran, partition for partition; away from 1 it
/// moves the partition (the control: the knob is wired to something).
#[test]
fn the_partitions_resolution_is_one_by_default_and_moves_the_partition() {
    let request = |resolution| {
        topic_request(std::sync::Arc::new(LexicalMeaning::new().with_params(
            MeaningParams {
                resolution,
                ..MeaningParams::LEXICAL
            },
        )))
    };
    let at_one = compute_meaning(request(1.0)).unwrap();
    let pairs: Vec<(NodeKey, NodeKey, f32)> = at_one.pairs.clone();
    let keys: Vec<NodeKey> = at_one.groups.iter().map(|(k, _)| *k).collect();
    let classical = crate::signals::community_louvain_on_snapshot(
        &crate::signals::CommunitySnapshot::from_weighted_pairs(keys, &pairs),
    );
    assert_eq!(
        partition(&at_one.groups),
        partition(
            &classical
                .clusters
                .iter()
                .enumerate()
                .flat_map(|(i, c)| c.members.iter().map(move |&k| (k, i as u32)))
                .collect::<Vec<_>>()
        ),
        "resolution 1 is classical modularity"
    );
    let coarse = compute_meaning(request(0.2)).unwrap();
    let fine = compute_meaning(request(3.0)).unwrap();
    let count = |s: &crate::canvas::MeaningSnapshot| partition(&s.groups).len();
    println!(
        "groups at resolution 0.2, 1 and 3: {}, {}, {}",
        count(&coarse),
        count(&at_one),
        count(&fine)
    );
    assert_eq!(
        coarse.pairs, at_one.pairs,
        "the resolution leaves the pairs alone"
    );
    assert!(count(&coarse) < count(&at_one), "below 1, fewer groups");
    assert!(count(&fine) > count(&at_one), "above 1, more groups");
}

#[path = "meaning/channels.rs"]
mod channels;
