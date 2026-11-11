// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

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
            ChannelValues::Order(order) => assert!(covered(&order), "{id}"),
            // No focus on the fixture: nothing to ring around.
            ChannelValues::Rings(rings) => assert!(rings.is_empty(), "{id}"),
            ChannelValues::Coords(coords) => assert!(
                covered(&coords.iter().map(|(k, _)| *k).collect::<Vec<_>>()),
                "{id}"
            ),
            ChannelValues::Nodes(nodes) => {
                assert!(nodes.iter().all(|k| keys.contains(k)), "{id}")
            },
            ChannelValues::Edges(edges) => {
                assert_eq!(channel.family(), ChannelFamily::Edges, "{id}");
                // A spanning forest: some edges, fewer than the nodes.
                assert!(!edges.is_empty() && edges.len() < keys.len(), "{id}");
                assert!(
                    edges
                        .iter()
                        .all(|(a, b)| keys.contains(a) && keys.contains(b)),
                    "{id}"
                );
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
        // F85: the bridges are one channel, `groups.bridges`, the metric
        // not part of the id; and a kind cannot be read as the bridge set.
        "bridges.betweenness",
        "bridges.articulation",
        "kind.bridges",
        // F174: the skeleton is `edges.spanning`, nothing else.
        "edges.skeleton",
        "edges.dominator",
        "tree.spanning",
        "pairs.skeleton",
        "",
    ] {
        assert_eq!(Channel::parse(bad), None, "{bad:?} is no channel");
    }
    assert_eq!(Channel::parse("groups.bridges"), Some(Channel::Bridges));
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
        canvas.pick_groups(groups);
        canvas.pick_overlays(overlays).expect("not refused");
        canvas.pick_law(PhysicsLaw::Springs).expect("not refused");
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

#[test]
fn columns_by_cluster_and_kinds_by_cluster_read_one_computation() {
    use crate::signals::louvain_runs_on_this_thread;
    let (mut canvas, keys, _) = topic_canvas();
    let start = louvain_runs_on_this_thread();
    canvas
        .pick(&crate::canvas::PhysicsChoice {
            law: PhysicsLaw::Kinds,
            overlays: vec![PhysicsOverlay::DomainCluster],
            kind: PhysicsKindSource::Cluster,
            groups: PhysicsKindSource::Cluster,
            ..Default::default()
        })
        .expect("not refused");
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

    // Positive control: the graph-only path with a registry other than the
    // canvas's, as hosts took before the canvas became the binding, runs a
    // Louvain of its own.
    let before = louvain_runs_on_this_thread();
    let _ = crate::canvas::project_canvas_strategy(
        &mut crate::signals::ChannelRegistry::new(),
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
    canvas
        .pick(&crate::canvas::PhysicsChoice {
            law: PhysicsLaw::Kinds,
            overlays: vec![PhysicsOverlay::DomainCluster],
            kind: PhysicsKindSource::Meaning,
            groups: PhysicsKindSource::Meaning,
            ..Default::default()
        })
        .expect("not refused");
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
