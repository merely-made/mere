// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The fold fact through time: folding, unfolding and refolding as scene
//! diffs, refused transactionally when invalid, and replayed by a
//! `SceneTrace` (site canvas plan, S5; Rulings 149 to 151).

use sceno::{
    Fold, FoldDirection, FoldRule, Footprint, InstanceId, ProjectedItem, Representation, Scene,
    SourceRef, StandIn, Transform2,
};
use scenotime::{
    DiffError, FoldId, Revision, SceneDiff, SceneEpoch, SceneOp, SceneSnapshot, SceneTrace,
    TraceStep,
};

const EPOCH: SceneEpoch = SceneEpoch(3);

/// Four items, `root` at 0 with three dependents.
fn base() -> SceneSnapshot {
    let mut scene = Scene::new();
    for (index, id) in ["root", "a", "b", "c"].into_iter().enumerate() {
        let source = scene.intern_source(SourceRef::new("fixture", id));
        scene.items.push(ProjectedItem {
            source,
            space: Scene::WORLD,
            transform: Transform2::translation(index as f32 * 40.0, 0.0),
            footprint: Footprint::Circle { radius: 10.0 },
            representation: Representation::Glyph,
            layer: 0,
            visible: true,
            hit: None,
            channels: Vec::new(),
        });
    }
    SceneSnapshot::from_dense(EPOCH, Revision(1), scene).expect("base")
}

fn dependents() -> Fold {
    Fold {
        members: [0, 1, 2, 3].into_iter().map(InstanceId).collect(),
        stand_in: StandIn::Member(InstanceId(0)),
        rule: Some(FoldRule::Descendants {
            root: InstanceId(0),
            family: "depends_on".into(),
            direction: FoldDirection::Outgoing,
        }),
        boundary: None,
        label: None,
    }
}

fn diff(base: u64, operations: Vec<SceneOp>) -> SceneDiff {
    SceneDiff {
        epoch: EPOCH,
        base: Revision(base),
        revision: Revision(base + 1),
        operations,
    }
}

fn shown(snapshot: &SceneSnapshot) -> Vec<u32> {
    (0..snapshot.tables.items.len() as u32)
        .filter(|index| snapshot.is_shown(InstanceId(*index)))
        .collect()
}

#[test]
fn fold_unfold_and_refold() {
    let mut state = base();
    assert_eq!(shown(&state), [0, 1, 2, 3]);

    state
        .apply_diff(&diff(
            1,
            vec![SceneOp::AddFold {
                index: FoldId(0),
                value: dependents(),
            }],
        ))
        .expect("fold");
    assert_eq!(shown(&state), [0], "the fact alone hides the members");
    assert_eq!(state.active_fold(FoldId(0)).unwrap().hidden_count(), 3);
    assert!(
        state.tables.items.iter().flatten().all(|item| item.visible),
        "no item's own flag changed"
    );

    state
        .apply_diff(&diff(2, vec![SceneOp::TombstoneFold { index: FoldId(0) }]))
        .expect("unfold");
    assert_eq!(shown(&state), [0, 1, 2, 3]);
    assert_eq!(state.tables.folds.len(), 1, "the slot stays allocated");

    let reused = state.clone();
    let mut refused = reused.clone();
    assert!(matches!(
        refused.apply_diff(&diff(
            3,
            vec![SceneOp::AddFold {
                index: FoldId(0),
                value: dependents(),
            }],
        )),
        Err(DiffError::InvalidOperation(_))
    ));
    assert_eq!(refused, reused, "a tombstoned fold slot is not reused");

    state
        .apply_diff(&diff(
            3,
            vec![SceneOp::AddFold {
                index: FoldId(1),
                value: dependents(),
            }],
        ))
        .expect("refold");
    assert_eq!(shown(&state), [0]);
}

#[test]
fn the_stand_in_changes_without_refolding() {
    let mut state = base();
    state
        .apply_diff(&diff(
            1,
            vec![SceneOp::AddFold {
                index: FoldId(0),
                value: dependents(),
            }],
        ))
        .unwrap();
    state
        .apply_diff(&diff(
            2,
            vec![SceneOp::SetFoldStandIn {
                index: FoldId(0),
                stand_in: StandIn::Summary {
                    label: Some("root and 3 dependents".into()),
                },
            }],
        ))
        .unwrap();
    assert_eq!(
        shown(&state),
        Vec::<u32>::new(),
        "a summary hides every member"
    );
    assert_eq!(state.active_fold(FoldId(0)).unwrap().hidden_count(), 4);

    let before = state.clone();
    assert!(matches!(
        state.apply_diff(&diff(
            3,
            vec![SceneOp::SetFoldStandIn {
                index: FoldId(0),
                stand_in: StandIn::Member(InstanceId(9)),
            }],
        )),
        Err(DiffError::InvalidSnapshot(_))
    ));
    assert_eq!(state, before);
}

#[test]
fn invalid_folds_are_refused_whole() {
    let state = base();
    let refusals = [
        Fold {
            members: vec![InstanceId(0)],
            ..dependents()
        },
        Fold {
            members: vec![InstanceId(0), InstanceId(1), InstanceId(1)],
            ..dependents()
        },
        Fold {
            members: vec![InstanceId(0), InstanceId(8)],
            ..dependents()
        },
        Fold {
            stand_in: StandIn::Member(InstanceId(7)),
            ..dependents()
        },
        Fold {
            label: Some(" ".into()),
            ..dependents()
        },
    ];
    for fold in refusals {
        let mut attempt = state.clone();
        let result = attempt.apply_diff(&diff(
            1,
            vec![
                SceneOp::SetGeneration { generation: 99 },
                SceneOp::AddFold {
                    index: FoldId(0),
                    value: fold.clone(),
                },
            ],
        ));
        assert!(
            matches!(result, Err(DiffError::InvalidSnapshot(_))),
            "{fold:?} was accepted"
        );
        assert_eq!(attempt, state, "nothing in the diff applied");
    }

    // Two folds may not share a member.
    let mut overlapping = state.clone();
    let result = overlapping.apply_diff(&diff(
        1,
        vec![
            SceneOp::AddFold {
                index: FoldId(0),
                value: Fold {
                    members: vec![InstanceId(0), InstanceId(1)],
                    ..dependents()
                },
            },
            SceneOp::AddFold {
                index: FoldId(1),
                value: Fold {
                    members: vec![InstanceId(1), InstanceId(2)],
                    stand_in: StandIn::Member(InstanceId(2)),
                    rule: Some(FoldRule::Selection),
                    boundary: None,
                    label: None,
                },
            },
        ],
    ));
    assert!(matches!(result, Err(DiffError::InvalidSnapshot(_))));
    assert_eq!(overlapping, state);

    // Tombstoning a member out from under its fold is refused too.
    let mut folded = state.clone();
    folded
        .apply_diff(&diff(
            1,
            vec![SceneOp::AddFold {
                index: FoldId(0),
                value: dependents(),
            }],
        ))
        .unwrap();
    let before = folded.clone();
    assert!(matches!(
        folded.apply_diff(&diff(
            2,
            vec![SceneOp::TombstoneItem {
                index: InstanceId(2)
            }]
        )),
        Err(DiffError::InvalidSnapshot(_))
    ));
    assert_eq!(folded, before);
}

#[test]
fn a_trace_replays_fold_and_unfold() {
    let mut trace = SceneTrace::new(base()).unwrap();
    trace = trace
        .appended(TraceStep::mark("Select root"))
        .unwrap()
        .appended(TraceStep::diff(
            "Fold root dependents",
            diff(
                1,
                vec![SceneOp::AddFold {
                    index: FoldId(0),
                    value: dependents(),
                }],
            ),
        ))
        .unwrap()
        .appended(TraceStep::diff(
            "Expand root dependents",
            diff(2, vec![SceneOp::TombstoneFold { index: FoldId(0) }]),
        ))
        .unwrap();

    let wire = serde_json::to_string(&trace).unwrap();
    let far_side: SceneTrace = serde_json::from_str(&wire).expect("the trace crosses the wire");
    assert_eq!(far_side, trace);

    let positions = (0..=far_side.len())
        .map(|position| shown(&far_side.snapshot_at(position).unwrap()))
        .collect::<Vec<_>>();
    assert_eq!(
        positions,
        [
            vec![0, 1, 2, 3],
            vec![0, 1, 2, 3],
            vec![0],
            vec![0, 1, 2, 3]
        ]
    );
}

#[test]
fn a_fold_free_snapshot_writes_what_it_wrote_before() {
    let snapshot = base();
    let wire = serde_json::to_string(&snapshot).unwrap();
    assert!(!wire.contains("folds"), "no fold key on a fold-free wire");
    let back: SceneSnapshot = serde_json::from_str(&wire).unwrap();
    assert!(back.tables.folds.is_empty());
    assert_eq!(
        serde_json::to_string(&back).unwrap(),
        wire,
        "byte-identical"
    );
}

#[test]
fn a_folded_member_never_picks() {
    let mut state = base();
    let under_b = sceno::Vec2::new(80.0, 0.0);
    assert_eq!(state.tables.pick(under_b), Some(InstanceId(2)));
    state
        .apply_diff(&diff(
            1,
            vec![SceneOp::AddFold {
                index: FoldId(0),
                value: dependents(),
            }],
        ))
        .unwrap();
    assert_eq!(state.tables.pick(under_b), None);
    assert_eq!(
        state.tables.pick(sceno::Vec2::ZERO),
        Some(InstanceId(0)),
        "the stand-in still picks"
    );
}
