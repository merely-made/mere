// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The site's projection proof, replayed as a `SceneTrace` (site canvas plan,
//! S4's done-condition).
//!
//! `fixtures/mer3ly_projection_scene.json` is the `projection-scene.json` the
//! mer3ly site build wrote at merelyllc.com `00b3997` (the projection code is
//! unchanged from `acb6bd6`), built with
//! `cargo run --locked --bin site -- --output <dir>` against mere `ea9e6da7`.
//! Its schema is `mer3ly.portable-projection/v1`. Only its `snapshot` and
//! `default_trace` are read here: each step's `selection` becomes the host
//! annotation, and its `diff` the step's diff.

use scenotime::{SceneDiff, SceneSnapshot, SceneTrace, TraceStep};
use serde::Deserialize;
use serde_json::Value;

/// The two fields of the site's artifact this test reads; the rest is the
/// site's.
#[derive(Deserialize)]
struct SiteArtifact {
    snapshot: SceneSnapshot,
    default_trace: Vec<SiteStep>,
}

/// The site's `ProjectionStep`, verbatim.
#[derive(Deserialize)]
struct SiteStep {
    label: String,
    selection: Option<Value>,
    diff: Option<SceneDiff>,
}

fn artifact() -> SiteArtifact {
    serde_json::from_str(include_str!("fixtures/mer3ly_projection_scene.json"))
        .expect("the site artifact parses")
}

fn as_trace(artifact: &SiteArtifact) -> SceneTrace {
    let steps = artifact
        .default_trace
        .iter()
        .map(|step| TraceStep {
            label: step.label.clone(),
            diff: step.diff.clone(),
            annotation: step.selection.clone(),
        })
        .collect();
    SceneTrace::from_steps(artifact.snapshot.clone(), steps).expect("the site's chain holds")
}

/// The site's native consumer (`consume_portable_projection`), position by
/// position: clone the snapshot and apply each step's diff in turn.
fn native_replay(artifact: &SiteArtifact) -> Vec<SceneSnapshot> {
    let mut snapshot = artifact.snapshot.clone();
    let mut positions = vec![snapshot.clone()];
    for step in &artifact.default_trace {
        if let Some(diff) = &step.diff {
            snapshot
                .apply_diff(diff)
                .expect("the site's consumer applies it");
        }
        positions.push(snapshot.clone());
    }
    positions
}

#[test]
fn every_position_matches_the_site_consumer() {
    let artifact = artifact();
    let trace = as_trace(&artifact);
    let native = native_replay(&artifact);

    assert_eq!(trace.len(), 7, "the default trace's seven steps all count");
    assert_eq!(native.len(), trace.len() + 1);
    for (position, expected) in native.iter().enumerate() {
        assert_eq!(
            &trace.snapshot_at(position).expect("in range"),
            expected,
            "position {position} ({})",
            position
                .checked_sub(1)
                .map_or("base", |index| trace.steps()[index].label.as_str())
        );
    }
    let revisions = (0..=trace.len())
        .map(|position| trace.revision_at(position).expect("in range").0)
        .collect::<Vec<_>>();
    assert_eq!(revisions, [1, 1, 2, 2, 3, 3, 4, 5]);
}

#[test]
fn the_selection_rides_as_the_host_annotation() {
    let artifact = artifact();
    let trace = as_trace(&artifact);
    for (step, site) in trace.steps().iter().zip(&artifact.default_trace) {
        assert_eq!(step.label, site.label);
        assert_eq!(step.annotation, site.selection);
    }
    assert_eq!(
        trace.steps()[0].annotation,
        Some(serde_json::json!({"kind": "node", "id": "turnstone"}))
    );

    let wire = serde_json::to_string(&trace).expect("serializes");
    let read: SceneTrace = serde_json::from_str(&wire).expect("reads back");
    assert_eq!(read, trace);
}
