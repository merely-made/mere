// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The grouped layout through the canvas (dynamics grammar plan, G3):
//! Charge's repulsion between groups at weight 16, Springs within (F71), on
//! a graph whose topics no edge structure discloses, read against Springs
//! alone and shuffled groups.
//!
//! The fixture is G2's quick topic fixture (`meaning_topics::topic_graph`,
//! F70): node `i` has topic `i / 8`, site `i % 4`, and sits in structural
//! community `(i / 2) % 4` (a ring of eight with four diameters per
//! community, four bridges between), with a title per topic. The topics are
//! handed in as the partition, the receipt's ground truth (F70); the rows on
//! `groups.meaning` read the Meaning channel through the law inputs, lexical
//! on the CPU here and the pinned model on the host's GPU in the ignored row.
//! Every run starts from one seeded scatter, placed as a seeded arrangement,
//! and is read when the bodies rest (F46) or after 6 000 frames. The
//! within-group stress is reported and not asserted: its bar waits on a
//! fixture whose topics have more structure inside them (F72).

use crate::canvas::tests::ThroughView;
use std::sync::Arc;

use super::meaning_topics::topic_graph;
use super::*;
use crate::canvas::composition::{GroupSource, PhysicsComposition, PhysicsGrouping};
use crate::canvas::meaning::MeaningEngine;
use crate::canvas::physics_catalog::PhysicsKindSource;
use seiche::observe::{Separation, group_stress, group_stress_each, separation};

const N: usize = 32;

fn topic(i: usize) -> u32 {
    (i / 8) as u32
}

/// A seeded scatter in a disc of 300, one point per node index.
fn scatter() -> Vec<PortablePoint> {
    let mut state: u64 = 0x6e0u64;
    let mut unit = || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 40) as f32 / (1u64 << 24) as f32
    };
    (0..N)
        .map(|_| {
            let (r, a) = (300.0 * unit().sqrt(), std::f32::consts::TAU * unit());
            PortablePoint::new(r * a.cos(), r * a.sin())
        })
        .collect()
}

/// The topic fixture over `members` (node indices): the whole graph with the
/// other nodes removed, so the edges among the members stay.
fn graph_of(members: &[usize]) -> (Graph, Vec<NodeKey>) {
    let (mut graph, keys, _) = topic_graph();
    for (i, &key) in keys.iter().enumerate() {
        if !members.contains(&i) {
            graph.remove_node(key);
        }
    }
    (graph, members.iter().map(|&i| keys[i]).collect())
}

/// `labels` dealt out again by a fixed shuffle, each as often as before.
fn shuffled(keys: &[NodeKey], labels: &HashMap<NodeKey, u32>) -> HashMap<NodeKey, u32> {
    let mut values: Vec<u32> = keys.iter().map(|k| labels[k]).collect();
    let mut state: u64 = 0x5EED;
    for i in (1..values.len()).rev() {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        values.swap(i, (state >> 33) as usize % (i + 1));
    }
    keys.iter().copied().zip(values).collect()
}

/// A group per node.
type Groups = HashMap<NodeKey, u32>;

/// What a run leaves: positions, how many frames it took to rest, the
/// Meaning channel's groups as the canvas held them, and its run count.
struct Rest {
    positions: Vec<(NodeKey, (f64, f64))>,
    frames: usize,
    meaning: Groups,
    meaning_runs: u64,
}

/// How the law slot is filled for a run.
enum Slot {
    /// The picked law (Springs).
    Law,
    /// A composition through the catalog.
    Composition(PhysicsComposition),
}

/// Run `slot` over the graph on `members` from the scatter until rest, with
/// `engine` as the canvas's Meaning engine (its lexical default if none).
fn rest_on(members: &[usize], slot: Slot, engine: Option<Arc<dyn MeaningEngine>>) -> Rest {
    let (graph, keys) = graph_of(members);
    let mut canvas = Canvas::with_graph(graph);
    if let Some(engine) = engine {
        canvas.set_meaning_engine(engine);
    }
    canvas.resize(1400, 900);
    canvas.set_physics_paused(true);
    canvas.set_layout_strategy(Some("test.scatter".to_string()));
    let scatter = scatter();
    let seed: Vec<_> = members.iter().map(|&i| scatter[i]).collect();
    canvas.apply_strategy_positions(&keys.iter().copied().zip(seed).collect::<Vec<_>>());
    if let Slot::Composition(c) = slot {
        canvas.pick_composition(Some(c)).unwrap();
    }
    canvas.set_physics_paused(false);
    let mut frames = 0;
    while canvas.settle_count() == 0 && frames < 6_000 {
        canvas.step_layout();
        frames += 1;
    }
    let meaning = if canvas.meaning().is_some() {
        canvas
            .channel_groups(PhysicsKindSource::Meaning)
            .into_iter()
            .collect()
    } else {
        HashMap::new()
    };
    Rest {
        positions: canvas
            .view
            .positions()
            .map(|(k, p)| (k, (f64::from(p.x), f64::from(p.y))))
            .collect(),
        frames,
        meaning,
        meaning_runs: canvas.meaning_runs(),
    }
}

fn rest(members: &[usize], slot: Slot) -> Rest {
    rest_on(members, slot, None)
}

fn all() -> Vec<usize> {
    (0..N).collect()
}

fn sorted(groups: &HashMap<NodeKey, u32>) -> Vec<(NodeKey, u32)> {
    let mut list: Vec<(NodeKey, u32)> = groups.iter().map(|(k, g)| (*k, *g)).collect();
    list.sort_by_key(|(k, _)| k.index());
    list
}

/// Charge's repulsion between `groups` at `weight`, Springs within, through
/// the catalog: F71's reading, ruled at weight 16.
fn charge_between(groups: GroupSource, weight: f32) -> Slot {
    Slot::Composition(PhysicsComposition::Grouped(PhysicsGrouping {
        outer_weight: weight,
        ..PhysicsGrouping::charge_between(groups)
    }))
}

/// Charge between a partition the host hands in.
fn charge_between_given(groups: &HashMap<NodeKey, u32>, weight: f32) -> Slot {
    charge_between(GroupSource::Given(sorted(groups)), weight)
}

fn show(v: Option<f64>) -> String {
    v.map_or("none".into(), |v| format!("{v:.4}"))
}

fn line(name: &str, run: &Rest, against: &HashMap<NodeKey, u32>) -> Separation {
    let edges: Vec<(NodeKey, NodeKey)> = {
        let (graph, _) = graph_of(&all());
        crate::canvas::seiche_bridge::visible_relation_edges(
            &graph,
            &std::collections::HashSet::new(),
        )
    };
    let s = separation(&run.positions, against);
    println!(
        "{name}: ratio {:.3} (gap {:.0}, spread {:.0}, nearest gap {:.0}); within-group stress {} \
         at one scale, {} at a scale per group; rest after {} frames",
        s.ratio,
        s.gap,
        s.spread,
        s.nearest_gap,
        show(group_stress(&run.positions, &edges, against)),
        show(group_stress_each(&run.positions, &edges, against)),
        run.frames
    );
    s
}

/// The partitions by node key of the full fixture: the topics, the sites,
/// Louvain's clusters, and the topics shuffled.
fn partitions() -> (Groups, Groups, Groups, Groups) {
    let (graph, keys) = graph_of(&all());
    let topics = keys
        .iter()
        .enumerate()
        .map(|(i, &k)| (k, topic(i)))
        .collect();
    let sites = keys
        .iter()
        .enumerate()
        .map(|(i, &k)| (k, (i % 4) as u32))
        .collect();
    let clusters = crate::signals::community_louvain(&graph)
        .clusters
        .iter()
        .enumerate()
        .flat_map(|(c, cluster)| cluster.members.iter().map(move |&m| (m, c as u32)))
        .collect();
    let shuffled_topics = shuffled(&keys, &topics);
    (topics, sites, clusters, shuffled_topics)
}

/// Springs run on each topic alone: the other baseline the stress bar may
/// take (F72), a scale per topic.
fn springs_on_each_topic() -> f64 {
    let mut each = Vec::new();
    for t in 0..4u32 {
        let members: Vec<usize> = (0..N).filter(|&i| topic(i) == t).collect();
        let run = rest(&members, Slot::Law);
        let one: HashMap<NodeKey, u32> = run.positions.iter().map(|(k, _)| (*k, 0)).collect();
        let (graph, _) = graph_of(&members);
        let edges = crate::canvas::seiche_bridge::visible_relation_edges(
            &graph,
            &std::collections::HashSet::new(),
        );
        each.push(group_stress_each(&run.positions, &edges, &one).unwrap_or(0.0));
    }
    each.iter().sum::<f64>() / each.len() as f64
}

/// G3's done-condition on the grouped layout, with the topics handed in
/// (F70) and Charge's repulsion between them at weight 16 (F71): the gap
/// between topics exceeds the spread within them; Springs alone does not
/// separate them (the negative control), though it does separate Louvain's
/// clusters, the structure it is drawn from; and the same composition on
/// shuffled topics does not separate the real ones (the shuffled control).
/// The within-topic stress is printed beside Springs alone's and Springs on
/// each topic alone's, its bar open (F72).
#[test]
fn charge_between_topics_and_springs_within_separates_them() {
    let (topics, _, clusters, shuffled_topics) = partitions();
    let springs = rest(&all(), Slot::Law);
    let base = line("springs alone, against topics", &springs, &topics);
    let structural = line("springs alone, against clusters", &springs, &clusters);
    let by_topics = rest(&all(), charge_between_given(&topics, 16.0));
    let grouped = line(
        "charge between topics at 16, against topics",
        &by_topics,
        &topics,
    );
    let by_shuffled = rest(&all(), charge_between_given(&shuffled_topics, 16.0));
    let control = line(
        "charge between shuffled topics at 16, against topics",
        &by_shuffled,
        &topics,
    );
    println!(
        "springs on each topic alone: within-topic stress {:.4} at a scale per topic",
        springs_on_each_topic()
    );
    assert!(
        grouped.ratio > 1.0,
        "grouping by topics separates them: {grouped:?}"
    );
    assert!(
        base.ratio < 1.0,
        "Springs alone does not separate topics: {base:?}"
    );
    assert!(
        structural.ratio > 1.0,
        "Springs alone separates the structure's own clusters: {structural:?}"
    );
    assert!(
        control.ratio < 1.0,
        "shuffled topics do not separate the real ones: {control:?}"
    );
}

/// F70's rows on `groups.meaning`: the grouping reads the Meaning channel
/// through the law inputs (`GroupSource::Channel`), so the outer law acts
/// between the groups one Meaning run gave. The bars are the topic row's:
/// the grouped layout separates its own groups, Springs alone does not, and
/// the same composition over those groups shuffled does not. Read against
/// the topics as well, for the record.
fn meaning_rows(engine: Option<Arc<dyn MeaningEngine>>) {
    let (topics, _, _, _) = partitions();
    let by_meaning = rest_on(
        &all(),
        charge_between(GroupSource::Channel(PhysicsKindSource::Meaning), 16.0),
        engine,
    );
    let meaning = by_meaning.meaning.clone();
    assert_eq!(
        meaning.len(),
        N,
        "the channel's partition covers the fixture"
    );
    assert_eq!(
        by_meaning.meaning_runs, 1,
        "one Meaning run feeds the grouping"
    );
    let groups = sorted(&meaning)
        .iter()
        .map(|(_, g)| *g)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let labels: HashMap<NodeKey, usize> = topics.iter().map(|(k, t)| (*k, *t as usize)).collect();
    println!(
        "groups.meaning: {groups} groups, F {:.3} against the topics",
        super::meaning_topics::f_measure(&sorted(&meaning), &labels)
    );
    let grouped = line(
        "charge between groups.meaning at 16, against groups.meaning",
        &by_meaning,
        &meaning,
    );
    line(
        "charge between groups.meaning at 16, against topics",
        &by_meaning,
        &topics,
    );
    let springs = rest(&all(), Slot::Law);
    let base = line("springs alone, against groups.meaning", &springs, &meaning);
    let keys: Vec<NodeKey> = sorted(&meaning).iter().map(|(k, _)| *k).collect();
    let by_shuffled = rest(
        &all(),
        charge_between_given(&shuffled(&keys, &meaning), 16.0),
    );
    let control = line(
        "charge between shuffled groups.meaning at 16, against groups.meaning",
        &by_shuffled,
        &meaning,
    );
    assert!(
        grouped.ratio > 1.0,
        "grouping by groups.meaning separates its groups: {grouped:?}"
    );
    assert!(
        base.ratio < 1.0,
        "Springs alone does not separate the meaning groups: {base:?}"
    );
    assert!(
        control.ratio < 1.0,
        "shuffled meaning groups do not separate the real ones: {control:?}"
    );
}

/// The `groups.meaning` rows on the canvas's default engine, the lexical
/// fallback on the CPU.
#[test]
fn charge_between_meaning_groups_separates_them_on_the_lexical_fallback() {
    meaning_rows(None);
}

/// The `groups.meaning` rows on the pinned model (e5-base-v2, F56) on the
/// host's GPU, booted greedy (F31).
///
/// `ESP_MODELS_DIR=<repo>/models cargo test --release -p pictograph --features
/// meaning-gpu --lib grouped -- --ignored --nocapture --test-threads=1`
#[cfg(feature = "meaning-gpu")]
#[test]
#[ignore = "requires the local models directory (ESP_MODELS_DIR) and a wgpu adapter"]
fn charge_between_meaning_groups_separates_them_on_the_pinned_model() {
    use crate::canvas::meaning_device::DeviceMeaning;
    use crate::canvas::physics_device_for;
    let models = std::env::var_os("ESP_MODELS_DIR")
        .map(std::path::PathBuf::from)
        .expect("ESP_MODELS_DIR names the local models directory");
    let adapter = netrender::boot().expect("a wgpu adapter");
    let backend = adapter.adapter.get_info().backend;
    drop(adapter);
    let needs = netrender::TenantNeeds {
        greedy: true,
        label: Some("grouped meaning receipt host"),
        ..Default::default()
    };
    let handles = netrender::boot_shared(backend.into(), None, &needs)
        .expect("a device for a JIT compute tenant");
    let device = physics_device_for(&handles);
    let engine = DeviceMeaning::load_pinned(&models, &device).expect("the pinned model loads");
    println!("engine: {:?}", engine.backend());
    meaning_rows(Some(Arc::new(engine)));
}

/// The ruled reading at other weights and on the other partitions, for
/// the record. Printed only.
#[test]
#[ignore = "a probe beside G3's grouped receipt; prints its table"]
fn probe_grouped_readings() {
    let (topics, sites, clusters, shuffled_topics) = partitions();
    for weight in [1.0, 4.0, 16.0, 64.0] {
        for (name, groups) in [("topics", &topics), ("shuffled topics", &shuffled_topics)] {
            let run = rest(&all(), charge_between_given(groups, weight));
            line(
                &format!("charge between {name} at {weight}, against topics"),
                &run,
                &topics,
            );
        }
    }
    for (name, groups) in [("clusters", &clusters), ("sites", &sites)] {
        let run = rest(&all(), charge_between_given(groups, 16.0));
        line(
            &format!("charge between {name} at 16, against topics"),
            &run,
            &topics,
        );
        line(
            &format!("charge between {name} at 16, against {name}"),
            &run,
            groups,
        );
    }
}

/// F49's key, alongside (F70): a group override reads the chosen `groups.*`
/// channel, a group labelled by its smallest member's id (F134). Under
/// `groups.cluster` a role set on one Louvain cluster's label
/// holds that cluster's members and no other node; the same label under the
/// default `groups.site` matches no node (the control); back on the site,
/// a site's override reads as before.
#[test]
fn a_group_role_reads_the_chosen_groups_channel() {
    use seiche::Role;
    let (graph, keys, _) = topic_graph();
    let clusters: HashMap<NodeKey, u32> = crate::signals::community_louvain(&graph)
        .clusters
        .iter()
        .enumerate()
        .flat_map(|(c, cluster)| cluster.members.iter().map(move |&m| (m, c as u32)))
        .collect();
    let mut canvas = Canvas::with_graph(graph);
    canvas.resize(1400, 900);
    canvas.set_physics_paused(true);
    canvas.set_layout_strategy(Some("test.scatter".to_string()));
    let scatter = scatter();
    canvas.apply_strategy_positions(&keys.iter().copied().zip(scatter).collect::<Vec<_>>());
    assert_eq!(canvas.role_group_source(), PhysicsKindSource::Site);

    canvas.set_role_group_source(PhysicsKindSource::Cluster);
    let label = canvas.role_group_of(keys[0]).expect("a cluster label");
    // F134: the label is the cluster's smallest member's stable id, not its
    // index, so a renumbered partition keeps it.
    let smallest = keys
        .iter()
        .filter(|key| clusters[*key] == clusters[&keys[0]])
        .map(|key| canvas.graph().get_node(*key).unwrap().id)
        .min()
        .unwrap();
    assert_eq!(label, format!("groups.cluster#{smallest}"));
    canvas.set_group_role(&label, Some(Role::Pinned));
    for key in &keys {
        let same = clusters[key] == clusters[&keys[0]];
        assert_eq!(
            canvas.role_group_of(*key).as_deref() == Some(label.as_str()),
            same,
            "the label is the partition's"
        );
        assert_eq!(
            canvas.arrangement_role_of(*key),
            if same { Role::Pinned } else { Role::Seeded },
            "node {}",
            key.index()
        );
    }

    // The control: the same override under the site matches no node.
    canvas.set_role_group_source(PhysicsKindSource::Site);
    assert!(
        keys.iter()
            .all(|key| canvas.arrangement_role_of(*key) == Role::Seeded)
    );
    let site = canvas.role_group_of(keys[0]).expect("a site");
    canvas.set_group_role(&site, Some(Role::Anchored));
    assert_eq!(
        canvas.arrangement_role_of(keys[4]),
        Role::Anchored,
        "same site"
    );
    assert_eq!(
        canvas.arrangement_role_of(keys[1]),
        Role::Seeded,
        "another site"
    );
}
