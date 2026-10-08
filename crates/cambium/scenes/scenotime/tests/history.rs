// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A host edits a scene trace through Cambium's one history (site canvas plan,
//! Ruling 133): the trace is the document, its length is the host's position,
//! moving the position is undo and redo, and committing after a move is
//! `History::record` clearing redo. Scenotime itself keeps no cursor.

use edit_history::History;
use sceno::{Footprint, InstanceId, ProjectedItem, Representation, Scene, SourceRef, Transform2};
use scenotime::{Revision, SceneDiff, SceneEpoch, SceneOp, SceneSnapshot, SceneTrace, TraceStep};

fn base() -> SceneSnapshot {
    let mut scene = Scene::new();
    let source = scene.intern_source(SourceRef::new("fixture", "only"));
    scene.items.push(ProjectedItem {
        source,
        space: Scene::WORLD,
        transform: Transform2::translation(0.0, 0.0),
        footprint: Footprint::Point,
        representation: Representation::Glyph,
        layer: 0,
        visible: true,
        hit: None,
        channels: Vec::new(),
    });
    SceneSnapshot::from_dense(SceneEpoch(1), Revision(0), scene).expect("snapshot")
}

fn raise(trace: &SceneTrace, layer: i16) -> TraceStep {
    let head = trace.revision_at(trace.len()).expect("end");
    TraceStep::diff(
        format!("Raise to {layer}"),
        SceneDiff {
            epoch: SceneEpoch(1),
            base: head,
            revision: Revision(head.0 + 1),
            operations: vec![SceneOp::SetItemLayer {
                index: InstanceId(0),
                layer,
            }],
        },
    )
}

fn layer(trace: &SceneTrace) -> i16 {
    trace.head().active_item(InstanceId(0)).expect("item").layer
}

/// The host's editor: the current trace and its history. The step bound is
/// the host's (Ruling 15); this one keeps every step.
struct Editor {
    current: SceneTrace,
    history: History<SceneTrace>,
}

impl Editor {
    fn new(trace: SceneTrace) -> Self {
        Self {
            current: trace,
            history: History::new().with_cap(0),
        }
    }

    fn commit(&mut self, step: TraceStep) {
        let next = self.current.appended(step).expect("the step chains");
        self.history.record(self.current.clone(), None, 0);
        self.current = next;
    }

    fn back(&mut self) -> bool {
        self.history
            .undo(self.current.clone())
            .map(|previous| self.current = previous)
            .is_some()
    }

    fn forward(&mut self) -> bool {
        self.history
            .redo(self.current.clone())
            .map(|next| self.current = next)
            .is_some()
    }
}

#[test]
fn moving_the_position_is_undo_and_redo() {
    let mut editor = Editor::new(SceneTrace::new(base()).expect("trace"));
    editor.commit(TraceStep::mark("Select"));
    for layer in 1..=3 {
        let step = raise(&editor.current, layer);
        editor.commit(step);
    }
    let full = editor.current.clone();
    assert_eq!((full.len(), layer(&full)), (4, 3));

    assert!(editor.back() && editor.back());
    assert_eq!(editor.current.len(), 2, "every step counts, marks included");
    assert_eq!(
        editor.current.head(),
        full.snapshot_at(2).expect("in range")
    );
    assert!(editor.back() && editor.back());
    assert_eq!(editor.current.head(), base());
    assert!(!editor.back(), "position 0 is the base");

    while editor.forward() {}
    assert_eq!(editor.current, full);
}

#[test]
fn committing_after_a_move_truncates_through_record() {
    let mut editor = Editor::new(SceneTrace::new(base()).expect("trace"));
    for layer in 1..=3 {
        let step = raise(&editor.current, layer);
        editor.commit(step);
    }
    assert!(editor.back() && editor.back());
    assert!(editor.history.can_redo());

    let step = raise(&editor.current, -5);
    editor.commit(step);
    assert!(
        !editor.history.can_redo(),
        "record clears redo: the old future is gone"
    );
    assert_eq!(editor.current.len(), 2);
    assert_eq!(layer(&editor.current), -5);
    assert_eq!(editor.current.head().revision, Revision(2));
}

#[test]
fn a_shared_trace_opens_at_a_host_supplied_position() {
    let mut trace = SceneTrace::new(base()).expect("trace");
    for layer in 1..=4 {
        trace = trace.appended(raise(&trace, layer)).expect("chains");
    }
    // The position comes from the host's link, not from the trace.
    let shared_position = 1;

    let mut editor = Editor::new(SceneTrace::new(base()).expect("trace"));
    for position in 1..=trace.len() {
        editor.commit(trace.steps()[position - 1].clone());
    }
    assert_eq!(editor.current, trace);
    for _ in shared_position..trace.len() {
        assert!(editor.back());
    }
    assert_eq!(
        editor.current.head(),
        trace.snapshot_at(shared_position).expect("in range")
    );
    while editor.forward() {}
    assert_eq!(editor.current, trace);
}
