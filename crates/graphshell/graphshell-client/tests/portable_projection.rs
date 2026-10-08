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
//! `scenotime/tests/site_projection.rs`. The third sibling, a shelfmark,
//! cites the capture by its content address and the authority's generation;
//! the check is written here as a host would write it, not shipped.

use std::collections::HashMap;

use chirograph::{
    CaptureAuthorityV1, PROJECTION_CAPTURE_V2, PresentationManifest, ProjectionCaptureV2,
    SceneSnapshot, Sha256NamedInformation,
};
use graphshell_client::frozen::FrozenScene;
use incipit::{ShelfmarkAuthorityV1, ShelfmarkInputV1, ShelfmarkV1};
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

/// The shelfmark a host would write beside a capture: the projection is the
/// capture's content address, and the authority input names the authority's
/// digest and the generation it expects.
fn cite(capture: &ProjectionCaptureV2) -> ShelfmarkV1 {
    let authority = capture.authority.as_ref().expect("authority");
    let mut shelfmark = ShelfmarkV1::new(
        capture
            .content_address()
            .expect("the capture has an address")
            .to_string(),
    );
    shelfmark.inputs.insert(
        "authority".to_owned(),
        ShelfmarkInputV1 {
            authority: ShelfmarkAuthorityV1 {
                adapter: authority.adapter.clone(),
                record: authority.sha256.to_string(),
            },
            reading: authority.schema.clone(),
            reading_parameters: None,
            arrangement: None,
            expects_generation: authority.generation.to_string(),
        },
    );
    shelfmark
}

/// A host's check of a shelfmark against the capture it is handed.
fn check(shelfmark: &ShelfmarkV1, capture: &ProjectionCaptureV2) -> Result<(), String> {
    shelfmark.validate().map_err(|error| format!("{error:?}"))?;
    let address = capture
        .content_address()
        .map_err(|error| error.to_string())?;
    if shelfmark.projection != address.to_string() {
        return Err(format!(
            "address: cited {}, capture is {address}",
            shelfmark.projection
        ));
    }
    let authority = capture
        .authority
        .as_ref()
        .ok_or("capture has no authority")?;
    let input = shelfmark
        .inputs
        .get("authority")
        .ok_or("shelfmark cites no authority")?;
    if input.authority.adapter != authority.adapter
        || input.authority.record != authority.sha256.to_string()
    {
        return Err("authority: cited a different authority".to_owned());
    }
    if input.expects_generation != authority.generation.to_string() {
        return Err(format!(
            "generation: cited {}, capture is {}",
            input.expects_generation, authority.generation
        ));
    }
    Ok(())
}

#[test]
fn a_shelfmark_cites_the_capture_by_address_and_generation() {
    let artifact = artifact();
    let bytes = capture(&artifact).encode().expect("the capture encodes");
    let capture = ProjectionCaptureV2::decode(&bytes).expect("the capture decodes");
    let shelfmark = cite(&capture);

    // The shelfmark travels as its own artifact and still matches.
    let wire = serde_json::to_string(&shelfmark).expect("serializes");
    let shelfmark: ShelfmarkV1 = serde_json::from_str(&wire).expect("reads back");
    assert_eq!(check(&shelfmark, &capture), Ok(()));
    assert_eq!(
        shelfmark.projection,
        chirograph::ContentHash::of(&bytes).to_string(),
        "the cited address is the capture's bytes"
    );

    // A different capture: one later revision of the same scene.
    let mut moved = capture.clone();
    moved.scene.revision.0 += 1;
    let refused = check(&shelfmark, &moved).expect_err("another capture");
    assert!(refused.starts_with("address"), "{refused}");

    // The same capture, cited at another generation.
    let mut stale = shelfmark.clone();
    stale
        .inputs
        .get_mut("authority")
        .expect("authority input")
        .expects_generation = (artifact.score.generation - 1).to_string();
    let refused = check(&stale, &capture).expect_err("another generation");
    assert!(refused.starts_with("generation"), "{refused}");
}
