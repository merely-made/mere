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

fn topic_canvas() -> (Canvas, Vec<NodeKey>, HashMap<NodeKey, usize>) {
    let (graph, keys, topics) = topic_graph();
    (Canvas::with_graph(graph), keys, topics)
}

/// Steps the with/without runs take: Springs on the topic fixture needs about
/// 4 000 to come to rest from a scatter (energy 870 at 1 500, 3.2 at 6 000).
const SETTLED_STEPS: usize = 6000;

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

#[test]
fn every_channel_id_round_trips_and_resolves_over_every_node() {
    let (mut canvas, keys, _) = topic_canvas();
    canvas.refresh_community_cache("kanban.community");
    let all = Channel::all();
    let ids: std::collections::HashSet<String> = all.iter().map(|c| c.id()).collect();
    assert_eq!(ids.len(), all.len(), "channel ids are unique");
    for family in ChannelFamily::ALL {
        assert!(
            all.iter().any(|c| c.family() == family),
            "{} has a channel",
            family.id()
        );
    }
    for channel in &all {
        let id = channel.id();
        assert_eq!(Channel::parse(&id), Some(*channel), "{id} round-trips");
        assert!(id.starts_with(channel.family().id()));
        let covered = |values: &[NodeKey]| {
            let mut v = values.to_vec();
            v.sort_by_key(|k| k.index());
            v == keys
        };
        match canvas.channel_values(*channel) {
            ChannelValues::Groups(values) => {
                assert!(
                    covered(&values.iter().map(|(k, _)| *k).collect::<Vec<_>>()),
                    "{id}"
                )
            },
            ChannelValues::Weights(values) => {
                assert!(
                    covered(&values.iter().map(|(k, _)| *k).collect::<Vec<_>>()),
                    "{id}"
                )
            },
            ChannelValues::Depths(values) => {
                assert!(
                    covered(&values.iter().map(|(k, _)| *k).collect::<Vec<_>>()),
                    "{id}"
                )
            },
            ChannelValues::Pairs(values) => {
                assert!(
                    matches!(
                        channel.family(),
                        ChannelFamily::Pairs | ChannelFamily::Distances
                    ),
                    "{id}"
                );
                // Content pairs exist only once Meaning or a host feeds them.
                if *channel != Channel::Pairs(crate::canvas::AffinityBlend::ContentOnly) {
                    assert!(!values.is_empty(), "{id} has pairs on the fixture");
                }
            },
        }
    }
    for bad in [
        "kind",
        "kind.",
        "kind.planet",
        "colour.site",
        "distances.miles",
        "",
    ] {
        assert_eq!(Channel::parse(bad), None, "{bad:?} is no channel");
    }
    // A kind folds to at most eight; the groups channel keeps every group.
    let ChannelValues::Groups(kinds) =
        canvas.channel_values(Channel::Kind(PhysicsKindSource::Coloring))
    else {
        panic!("kinds are groups");
    };
    assert!(kinds.iter().all(|(_, k)| *k < 8));
}

#[test]
fn group_pull_by_cluster_is_the_columns_by_cluster_twin() {
    let (mut canvas, keys, _) = topic_canvas();
    // The partition Columns (by cluster) lays out is the canvas's Louvain
    // cache; Group pull by cluster reads the same one.
    canvas.refresh_community_cache("kanban.community");
    let columns: Vec<(NodeKey, u32)> = canvas
        .community()
        .expect("the cluster arrangement's partition")
        .clusters
        .iter()
        .enumerate()
        .flat_map(|(i, c)| c.members.iter().map(move |&k| (k, i as u32)))
        .collect();
    let by_cluster = canvas.channel_groups(PhysicsKindSource::Cluster);
    assert_eq!(partition(&by_cluster), partition(&columns), "one partition");
    let expected: Vec<(NodeKey, u32)> = keys
        .iter()
        .enumerate()
        .map(|(i, &k)| (k, community(i) as u32))
        .collect();
    assert_eq!(
        partition(&columns),
        partition(&expected),
        "Louvain finds the fixture's four communities"
    );
    assert_ne!(
        partition(&canvas.channel_groups(PhysicsKindSource::Site)),
        partition(&columns),
        "site is another partition"
    );

    // With and without, from one scattered start: Springs alone, then with
    // Group pull by cluster, then by site. Each run is read for how far each
    // partition's members gather (neighbour agreement) and how far apart the
    // groups sit (separation).
    let site = canvas.channel_groups(PhysicsKindSource::Site);
    let run = |overlays: Vec<PhysicsOverlay>, groups: PhysicsKindSource| {
        let (mut canvas, _, _) = topic_canvas();
        canvas.set_physics_group_source(groups);
        canvas.set_physics_overlays(overlays);
        canvas.set_physics_law(PhysicsLaw::Springs);
        scatter(&mut canvas);
        canvas.physics.settle(SETTLED_STEPS as u32);
        steps(&mut canvas, SETTLED_STEPS);
        Reading {
            compact_cluster: compactness(&canvas, &columns),
            compact_site: compactness(&canvas, &site),
            agree_cluster: agreement(&canvas, &columns),
            agree_site: agreement(&canvas, &site),
            separation: separation(&canvas, &columns),
            stats: canvas.layout_stats(),
        }
    };
    let without = run(Vec::new(), PhysicsKindSource::Cluster);
    let with = run(
        vec![PhysicsOverlay::DomainCluster],
        PhysicsKindSource::Cluster,
    );
    let by_site = run(vec![PhysicsOverlay::DomainCluster], PhysicsKindSource::Site);
    for (name, r) in [
        ("Springs alone", &without),
        ("with Group pull by cluster", &with),
        ("with Group pull by site", &by_site),
    ] {
        println!(
            "{name}: compactness (within-group spread / layout spread) by cluster {:.3}, \
             by site {:.3}; neighbour agreement by cluster {:.3}, by site {:.3}; cluster \
             separation {:.3}; {:?}",
            r.compact_cluster, r.compact_site, r.agree_cluster, r.agree_site, r.separation, r.stats
        );
    }
    // Every run is read at rest.
    for r in [&without, &with, &by_site] {
        assert!(r.stats.energy < 10.0, "at rest: {:?}", r.stats);
    }
    // With: the overlay gathers the cluster partition beyond what the law's
    // own edges do. The partition is drawn from those edges, so the margin is
    // small; the overlay's own channel shows in what it does to the others.
    assert!(
        with.agree_cluster > without.agree_cluster + 0.05,
        "{:.3} against {:.3}",
        with.agree_cluster,
        without.agree_cluster
    );
    // It reads exactly its channel: by cluster gathers clusters more than by
    // site does, and by site tightens sites more than by cluster does.
    assert!(with.agree_cluster > by_site.agree_cluster + 0.1);
    assert!(by_site.compact_site < with.compact_site * 0.8);
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

#[test]
fn columns_by_cluster_and_kinds_by_cluster_read_one_computation() {
    use crate::signals::louvain_runs_on_this_thread;
    let (mut canvas, keys, _) = topic_canvas();
    let start = louvain_runs_on_this_thread();
    canvas.set_physics_choice(&crate::canvas::PhysicsChoice {
        law: PhysicsLaw::Kinds,
        overlays: vec![PhysicsOverlay::DomainCluster],
        kind: PhysicsKindSource::Cluster,
        groups: PhysicsKindSource::Cluster,
        ..Default::default()
    });
    let columns =
        canvas.project_arrangement_for_view("kanban.community", 1024, 600, None, true, 1.0, None);
    for _ in 0..5 {
        canvas.frame(1024, 600);
    }
    assert_eq!(columns.positions.len(), keys.len());
    assert_eq!(
        louvain_runs_on_this_thread() - start,
        1,
        "one partition for Columns (by cluster), Kinds and Group pull"
    );
    assert_eq!(canvas.community_runs(), 1);
    // The columns lay out the registry's groups: members of one cluster share
    // a column, members of two never do.
    let x_of: HashMap<NodeKey, i32> = columns
        .positions
        .iter()
        .map(|(k, p)| (*k, p.x.round() as i32))
        .collect();
    let groups = canvas.channel_groups(PhysicsKindSource::Cluster);
    let by_column: Vec<(NodeKey, u32)> = groups.iter().map(|(k, _)| (*k, x_of[k] as u32)).collect();
    println!(
        "columns {:?} against groups {:?}",
        partition(&by_column).len(),
        partition(&groups).len()
    );
    assert_eq!(
        partition(&by_column),
        partition(&groups),
        "a column per cluster"
    );
    let ChannelValues::Groups(kinds) =
        canvas.channel_values(Channel::Kind(PhysicsKindSource::Cluster))
    else {
        panic!("kinds are groups");
    };
    assert_eq!(
        partition(&kinds),
        partition(&groups),
        "Kinds read the same partition"
    );

    // Positive control: the graph-only path, which hosts took before the
    // canvas became the binding, runs a Louvain of its own.
    let before = louvain_runs_on_this_thread();
    let _ = crate::canvas::project_canvas_strategy(
        "kanban.community",
        canvas.graph(),
        None,
        1024,
        600,
        None,
        None,
        true,
    );
    assert_eq!(
        louvain_runs_on_this_thread() - before,
        1,
        "the instrument sees a second run"
    );

    // A structural change earns exactly one more, shared again.
    let before = louvain_runs_on_this_thread();
    canvas.ingest_graph(|g| {
        g.assert_semantic_predicate(keys[0], keys[31], "links".to_string());
        true
    });
    let _ =
        canvas.project_arrangement_for_view("kanban.community", 1024, 600, None, true, 1.0, None);
    for _ in 0..5 {
        canvas.frame(1024, 600);
    }
    assert_eq!(
        louvain_runs_on_this_thread() - before,
        1,
        "one run for the new revision"
    );
    assert_eq!(canvas.community_runs(), 2);
}

#[test]
fn one_meaning_snapshot_feeds_kinds_group_pull_affinity_and_groups() {
    let (mut canvas, keys, _) = topic_canvas();
    assert_eq!(canvas.meaning_backend(), MeaningBackend::Lexical);
    assert_eq!(canvas.meaning_runs(), 0);
    // No consumer, no run: frames alone embed nothing.
    for _ in 0..3 {
        canvas.frame(800, 600);
    }
    assert_eq!(canvas.meaning_runs(), 0, "nothing reads the channel yet");

    // All four consumers at once.
    canvas.set_affinity_blend(crate::canvas::AffinityBlend::ContentOnly);
    canvas.set_cluster_by_affinity(true);
    canvas.set_meaning_affinity(true);
    canvas.set_physics_choice(&crate::canvas::PhysicsChoice {
        law: PhysicsLaw::Kinds,
        overlays: vec![PhysicsOverlay::DomainCluster],
        kind: PhysicsKindSource::Meaning,
        groups: PhysicsKindSource::Meaning,
        ..Default::default()
    });
    for _ in 0..5 {
        canvas.frame(800, 600);
    }
    assert_eq!(canvas.meaning_runs(), 1, "one run for one content");
    let snapshot = canvas.meaning().expect("a snapshot").clone();
    assert_eq!(snapshot.run, 1);
    assert_eq!(snapshot.content_revision, canvas.graph().content_revision());
    assert_eq!(
        canvas.meaning.built_from,
        Some(1),
        "the law build read run 1"
    );
    // Group pull and G3's groups read the snapshot's clusters.
    assert_eq!(
        canvas.channel_groups(PhysicsKindSource::Meaning),
        snapshot.groups
    );
    // Kinds read them folded to at most eight.
    let ChannelValues::Groups(kinds) =
        canvas.channel_values(Channel::Kind(PhysicsKindSource::Meaning))
    else {
        panic!("kinds are groups");
    };
    let count = snapshot.clusters.clusters.len().clamp(1, 8) as u32;
    assert!(
        kinds
            .iter()
            .zip(&snapshot.groups)
            .all(|((ka, k), (ga, g))| ka == ga && *k == g % count)
    );
    // The affinity force holds exactly the snapshot's pairs.
    assert!(!snapshot.pairs.is_empty(), "the fixture's titles pair up");
    assert_eq!(canvas.affinity_pair_count(), snapshot.pairs.len());

    // Structure moves, content does not: no run, and the rebuild (Kinds is
    // graph-bound) reads the same snapshot.
    canvas.ingest_graph(|g| {
        g.assert_semantic_predicate(keys[0], keys[31], "links".to_string());
        true
    });
    for _ in 0..3 {
        canvas.frame(800, 600);
    }
    assert_eq!(canvas.meaning_runs(), 1, "an edge is not content");
    assert_eq!(canvas.meaning.built_from, Some(1));

    // A title rewritten to itself is not content either.
    let same = canvas.graph().get_node(keys[3]).unwrap().title.clone();
    canvas.ingest_graph(|g| {
        g.set_node_title(keys[3], same);
        true
    });
    for _ in 0..3 {
        canvas.frame(800, 600);
    }
    assert_eq!(
        canvas.meaning_runs(),
        1,
        "the same text is the same content"
    );

    // A real title edit is content: exactly one more run, read by everyone.
    canvas.ingest_graph(|g| g.set_node_title(keys[3], "Baking rye bread at home".to_string()));
    for _ in 0..5 {
        canvas.frame(800, 600);
    }
    assert_eq!(canvas.meaning_runs(), 2, "one run for the new content");
    let second = canvas.meaning().unwrap();
    assert_eq!(second.run, 2);
    assert_eq!(canvas.meaning.built_from, Some(2));
    assert_eq!(canvas.affinity_pair_count(), second.pairs.len());

    // Off: the content signal clears, and nothing reads Meaning but the law.
    canvas.set_meaning_affinity(false);
    canvas.frame(800, 600);
    assert!(!canvas.has_content_affinity());
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
    canvas.set_physics_kind_source(PhysicsKindSource::Meaning);
    canvas.set_physics_law(PhysicsLaw::Kinds);
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
    whole.set_physics_kind_source(PhysicsKindSource::Meaning);
    whole.set_physics_law(PhysicsLaw::Kinds);
    let expected = whole.meaning().expect("inline: whole at build").clone();
    assert_eq!(expected.steps, 1);

    let (mut canvas, _, _) = topic_canvas();
    // 32 nodes: 64 scores is two rows a frame, so the scan alone is 16 slices.
    canvas.set_meaning_slice(Some(64));
    canvas.set_physics_kind_source(PhysicsKindSource::Meaning);
    canvas.set_physics_law(PhysicsLaw::Kinds);
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
    canvas.set_physics_kind_source(PhysicsKindSource::Meaning);
    canvas.set_physics_law(PhysicsLaw::Kinds);
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
