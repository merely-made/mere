// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The site's dependency fold, expressed as the fold fact (site canvas plan,
//! S5; Rulings 149 to 152).
//!
//! `scenotime/tests/fixtures/mer3ly_projection_trace.json` is the
//! `projection-trace.json` the mer3ly site build wrote at merelyllc.com
//! `6a0d85d` (mere pinned at `7a5bedd1`), built with
//! `cargo run --locked --bin site -- --output <dir>`. Its fold step is the
//! site's `visibility_diff` (`crates/repo-graph/src/lib.rs`): Mere gains a
//! `["fold", 1]` channel and each target of a relation leaving Mere is set
//! `visible: false`; the expand step reverses both, and the projection
//! proof's JavaScript computes the "+N" from the same relations.
//!
//! `mer3ly_projection_trace_folded.json` is that trace with the two steps
//! rewritten as the fact: `AddFold` with Mere as its member stand-in and the
//! rule `Descendants { root: Mere, family: depends_on, direction: Outgoing }`,
//! then `TombstoneFold`. Every other step is the site's, unchanged. This
//! test derives it afresh and holds the committed copy to it; set
//! `MERE_WRITE_FIXTURES=1` to rewrite it.
//!
//! What is asserted: at every position the fact-based trace shows exactly the
//! items the site's trace shows, carries the same "+N" on the same stand-in,
//! and leaves every item's placement as the site's; and freezing the folded
//! position lists the group instead of dropping its members.

use std::collections::{BTreeSet, HashMap, VecDeque};

use graphshell_client::frozen::FrozenScene;
use sceno::{Fold, FoldDirection, FoldRule, InstanceId, StandIn};
use scenotime::{FoldId, SceneOp, SceneSnapshot, SceneTrace, TraceStep};

const FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../cambium/scenes/scenotime/tests/fixtures"
);
const ROOT: &str = "mere";
const FAMILY: &str = "depends_on";

fn site_trace() -> SceneTrace {
    serde_json::from_str(include_str!(
        "../../../cambium/scenes/scenotime/tests/fixtures/mer3ly_projection_trace.json"
    ))
    .expect("the site's trace parses and its chain holds")
}

fn instance_of(snapshot: &SceneSnapshot, id: &str) -> InstanceId {
    let index = snapshot
        .tables
        .items
        .iter()
        .position(|item| {
            item.as_ref().is_some_and(|item| {
                snapshot.tables.sources[item.source.0 as usize]
                    .as_ref()
                    .is_some_and(|source| source.id == id)
            })
        })
        .expect("the source is placed");
    InstanceId(index as u32)
}

/// The host's half of Ruling 151: it chooses the members, here by following
/// `depends_on` relations out of Mere, and records how.
fn mere_dependencies(snapshot: &SceneSnapshot) -> Fold {
    let root = instance_of(snapshot, ROOT);
    let mut members = vec![root];
    let mut seen = BTreeSet::from([root.0]);
    let mut queue = VecDeque::from([root]);
    while let Some(at) = queue.pop_front() {
        for relation in snapshot.tables.relations.iter().flatten() {
            if relation.from == at
                && relation.kind.as_deref() == Some(FAMILY)
                && seen.insert(relation.to.0)
            {
                members.push(relation.to);
                queue.push_back(relation.to);
            }
        }
    }
    Fold {
        members,
        stand_in: StandIn::Member(root),
        rule: Some(FoldRule::Descendants {
            root,
            family: FAMILY.into(),
            direction: FoldDirection::Outgoing,
        }),
        boundary: None,
        label: None,
    }
}

/// The site's trace with its fold and expand steps rewritten as the fact.
fn folded_trace(site: &SceneTrace) -> SceneTrace {
    let (base, steps) = site.clone().into_parts();
    let mut fold = None;
    let steps = steps
        .into_iter()
        .enumerate()
        .map(|(index, mut step)| {
            let Some(diff) = step.diff.as_mut() else {
                return step;
            };
            if step.label.starts_with("Fold ") {
                let before = site.snapshot_at(index).unwrap();
                let value = mere_dependencies(&before);
                fold = Some(value.clone());
                diff.operations = vec![SceneOp::AddFold {
                    index: FoldId(0),
                    value,
                }];
            } else if step.label.starts_with("Expand ") {
                assert!(fold.is_some(), "expand follows a fold");
                diff.operations = vec![SceneOp::TombstoneFold { index: FoldId(0) }];
            }
            step
        })
        .collect::<Vec<TraceStep>>();
    assert!(fold.is_some(), "the site's trace folds");
    SceneTrace::from_steps(base, steps).expect("the fact-based chain holds")
}

/// What the site shows at one position: each active item's own flag.
fn site_shown(snapshot: &SceneSnapshot) -> Vec<bool> {
    snapshot
        .tables
        .items
        .iter()
        .map(|item| item.as_ref().is_some_and(|item| item.visible))
        .collect()
}

/// What a fact reader shows: the flag, and no fold hiding it.
fn fact_shown(snapshot: &SceneSnapshot) -> Vec<bool> {
    (0..snapshot.tables.items.len())
        .map(|index| snapshot.is_shown(InstanceId(index as u32)))
        .collect()
}

/// The site's "+N" per item: an item carrying the fold channel counts the
/// distinct targets of relations leaving it, as `dependencyIds` does over
/// the proof's relations (the base table, before any edit).
fn site_badges(snapshot: &SceneSnapshot, base: &SceneSnapshot) -> Vec<usize> {
    snapshot
        .tables
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let folded = item.as_ref().is_some_and(|item| {
                item.channels
                    .iter()
                    .any(|(name, value)| name == "fold" && *value > 0.0)
            });
            if !folded {
                return 0;
            }
            base.tables
                .relations
                .iter()
                .flatten()
                .filter(|relation| relation.from.0 == index as u32)
                .map(|relation| relation.to.0)
                .collect::<BTreeSet<_>>()
                .len()
        })
        .collect()
}

fn fact_badges(snapshot: &SceneSnapshot) -> Vec<usize> {
    let mut badges = vec![0; snapshot.tables.items.len()];
    for (_, fold) in snapshot.active_folds() {
        if let Some(stand_in) = fold.stand_in_member() {
            badges[stand_in.0 as usize] = fold.hidden_count();
        }
    }
    badges
}

/// The site wrote its trace with a scenotime that predates folds. Read and
/// written back by this one, it is the same bytes: a fold-free scene carries
/// no fold key, so older readers meet nothing new.
#[test]
fn a_fold_free_trace_writes_back_byte_identical() {
    let bytes = include_str!(
        "../../../cambium/scenes/scenotime/tests/fixtures/mer3ly_projection_trace.json"
    );
    assert_eq!(serde_json::to_string(&site_trace()).unwrap(), bytes);
}

#[test]
fn the_committed_fact_trace_is_the_derived_one() {
    let derived = folded_trace(&site_trace());
    let path = format!("{FIXTURES}/mer3ly_projection_trace_folded.json");
    if std::env::var_os("MERE_WRITE_FIXTURES").is_some() {
        std::fs::write(&path, serde_json::to_string(&derived).unwrap()).unwrap();
    }
    let committed: SceneTrace =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("fixture committed"))
            .expect("the committed fact trace parses and replays");
    assert_eq!(committed, derived);
}

#[test]
fn every_position_shows_what_the_site_shows() {
    let site = site_trace();
    let fact = folded_trace(&site);
    assert_eq!(site.len(), 7, "the default trace's seven steps");
    assert_eq!(fact.len(), site.len());
    let base = site.base().clone();

    let mut folded_positions = 0;
    for position in 0..=site.len() {
        let theirs = site.snapshot_at(position).unwrap();
        let ours = fact.snapshot_at(position).unwrap();
        assert_eq!(
            fact_shown(&ours),
            site_shown(&theirs),
            "visibility at position {position}"
        );
        assert_eq!(
            fact_badges(&ours),
            site_badges(&theirs, &base),
            "+N at position {position}"
        );
        assert_eq!(ours.revision, theirs.revision, "revision at {position}");
        assert_eq!(
            ours.tables.relations, theirs.tables.relations,
            "relations at {position}"
        );
        for (index, (mine, site_item)) in ours
            .tables
            .items
            .iter()
            .zip(&theirs.tables.items)
            .enumerate()
        {
            let (mine, site_item) = (mine.as_ref().unwrap(), site_item.as_ref().unwrap());
            assert_eq!(
                mine.transform, site_item.transform,
                "item {index} placement at {position}"
            );
            assert!(mine.visible, "the fact never touches an item's own flag");
            assert!(
                mine.channels.iter().all(|(name, _)| name != "fold"),
                "the fact needs no fold channel"
            );
        }
        if !ours.active_folds().is_empty() {
            folded_positions += 1;
        }
    }
    assert_eq!(folded_positions, 1, "folded at exactly one position");
}

#[test]
fn the_rule_reaches_what_the_site_hides() {
    let site = site_trace();
    let base = site.base();
    let fold = mere_dependencies(base);
    let root = instance_of(base, ROOT);
    // The site hides the direct targets of every relation leaving Mere,
    // whatever its kind; the rule follows depends_on transitively. On this
    // data they agree: renders_with only repeats genet, and the one
    // depends_on path onward (knot-editor) leads back to Mere.
    let direct = base
        .tables
        .relations
        .iter()
        .flatten()
        .filter(|relation| relation.from == root)
        .map(|relation| relation.to.0)
        .collect::<BTreeSet<_>>();
    let hidden = fold
        .hidden()
        .map(|member| member.0)
        .collect::<BTreeSet<_>>();
    assert_eq!(hidden, direct);
    assert_eq!(fold.hidden_count(), 6, "the site's +6");
}

#[test]
fn freezing_the_fold_lists_the_group_instead_of_dropping_it() {
    let site = site_trace();
    let fact = folded_trace(&site);
    let position = (0..=fact.len())
        .find(|position| {
            !fact
                .snapshot_at(*position)
                .unwrap()
                .active_folds()
                .is_empty()
        })
        .unwrap();
    let names = HashMap::from([(instance_of(site.base(), ROOT), "Mere".to_owned())]);

    let theirs =
        FrozenScene::freeze_snapshot(&site.snapshot_at(position).unwrap(), "Repositories", &names);
    let ours =
        FrozenScene::freeze_snapshot(&fact.snapshot_at(position).unwrap(), "Repositories", &names);

    assert_eq!(theirs.instances.len(), 5, "the site's fold drops six");
    assert!(theirs.folds.is_empty(), "and says nothing about them");

    assert_eq!(
        ours.instances
            .iter()
            .map(|i| &i.instance)
            .collect::<Vec<_>>(),
        theirs
            .instances
            .iter()
            .map(|i| &i.instance)
            .collect::<Vec<_>>(),
        "the same items are met directly"
    );
    assert_eq!(ours.folds.len(), 1);
    let group = &ours.folds[0];
    assert_eq!(group.name, "Mere");
    assert_eq!(group.badge, "+6");
    assert_eq!(
        group
            .members
            .iter()
            .map(|m| m.source.id.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "emblem",
            "genet",
            "knot-editor",
            "mora",
            "netrender",
            "retinue"
        ]),
    );
    assert_eq!(
        group.rule.as_deref(),
        Some("Mere and everything it reaches by depends on")
    );
    assert_eq!(
        ours.instances.len() + group.members.len(),
        site.base().active_item_count(),
        "every item is listed somewhere"
    );
    let html = ours.to_html("proof");
    assert!(html.contains(
        "<details><summary>Mere, +6 folded: Mere and everything it reaches by depends on</summary>"
    ));
}
