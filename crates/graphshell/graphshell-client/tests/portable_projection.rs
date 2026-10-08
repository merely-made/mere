// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The site's portable projection, decomposed (site canvas plan, S6 and
//! Rulings 131-132): its scene, score and authority become a
//! `ProjectionCaptureV2`, its default trace a sibling `SceneTrace` whose base
//! is the captured scene, and the frozen realization renders the capture from
//! its bytes without running the compiler.
//!
//! The fixture is scenotime's copy of the mer3ly site build's
//! `projection-scene.json`; its provenance is recorded in
//! `scenotime/tests/site_projection.rs`. The shelfmark third of the
//! decomposition is not exercised here: graphshell-client has no `incipit`
//! edge.

use std::collections::HashMap;

use chirograph::{
    CaptureAuthorityV1, PROJECTION_CAPTURE_V2, PresentationManifest, ProjectionCaptureV2,
    SceneSnapshot, Sha256NamedInformation,
};
use graphshell_client::frozen::FrozenScene;
use sceno::{InstanceId, Score};
use scenotime::{SceneDiff, SceneTrace, TraceStep};
use serde::Deserialize;
use serde_json::Value;

/// The parts of the site's artifact the decomposition reads.
#[derive(Deserialize)]
struct SiteArtifact {
    adapter: String,
    authority_schema: String,
    authority_sha256: String,
    score: Score,
    snapshot: SceneSnapshot,
    nodes: Vec<SiteNode>,
    default_trace: Vec<SiteStep>,
}

#[derive(Deserialize)]
struct SiteNode {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct SiteStep {
    label: String,
    selection: Option<Value>,
    diff: Option<SceneDiff>,
}

fn artifact() -> SiteArtifact {
    serde_json::from_str(include_str!(
        "../../../cambium/scenes/scenotime/tests/fixtures/mer3ly_projection_scene.json"
    ))
    .expect("the site artifact parses")
}

fn capture(artifact: &SiteArtifact) -> ProjectionCaptureV2 {
    ProjectionCaptureV2 {
        version: PROJECTION_CAPTURE_V2,
        scene: artifact.snapshot.clone(),
        presentation: PresentationManifest::default(),
        score: Some(artifact.score.clone()),
        authority: Some(CaptureAuthorityV1 {
            adapter: artifact.adapter.clone(),
            schema: artifact.authority_schema.clone(),
            sha256: Sha256NamedInformation::from_hex(&artifact.authority_sha256)
                .expect("the site names its authority by lowercase hex"),
            generation: artifact.score.generation,
        }),
    }
}

/// Names by instance, from the site's node metadata: host data a reader
/// supplies, not part of the capture.
fn names(scene: &SceneSnapshot, artifact: &SiteArtifact) -> HashMap<InstanceId, String> {
    let by_id = artifact
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node.name.clone()))
        .collect::<HashMap<_, _>>();
    scene
        .tables
        .items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let source = scene.tables.sources[item.as_ref()?.source.0 as usize].as_ref()?;
            Some((
                InstanceId(index as u32),
                by_id.get(source.id.as_str())?.clone(),
            ))
        })
        .collect()
}

#[test]
fn the_site_artifact_decomposes_into_a_capture_and_a_sibling_trace() {
    let artifact = artifact();
    let bytes = capture(&artifact).encode().expect("the capture encodes");

    // A static viewer: bytes in, no compiler.
    let capture = ProjectionCaptureV2::decode(&bytes).expect("the capture decodes");
    let authority = capture.authority.as_ref().expect("authority");
    assert_eq!(
        authority.sha256.to_string(),
        Sha256NamedInformation::from_hex(&artifact.authority_sha256)
            .expect("hex")
            .to_string()
    );
    // The site's own convention, checked by the host rather than chirograph.
    assert_eq!(capture.scene.epoch.0, authority.generation);

    let wire: Value = serde_json::from_slice(&bytes).expect("json");
    assert!(
        wire.get("trace").is_none() && wire.get("default_trace").is_none(),
        "the trace travels beside the capture, not inside it"
    );

    let trace = SceneTrace::from_steps(
        capture.scene.clone(),
        artifact
            .default_trace
            .iter()
            .map(|step| TraceStep {
                label: step.label.clone(),
                diff: step.diff.clone(),
                annotation: step.selection.clone(),
            })
            .collect(),
    )
    .expect("the default trace chains from the captured scene");
    assert_eq!(trace.base(), &capture.scene);

    let frozen = FrozenScene::freeze_snapshot(
        &capture.scene,
        "Repository graph",
        &names(&capture.scene, &artifact),
    );
    assert_eq!(frozen.instances.len(), artifact.nodes.len());
    assert_eq!(
        frozen.unnamed, 0,
        "every node is named by the host's metadata"
    );
    assert!(
        frozen
            .to_html("capture")
            .contains("data-projection-instance")
    );

    // The trace's end renders too: the relation it removed is gone.
    let head = trace.head();
    let after = FrozenScene::freeze_snapshot(&head, "Repository graph", &names(&head, &artifact));
    assert_eq!(after.relations.len() + 1, frozen.relations.len());
}
