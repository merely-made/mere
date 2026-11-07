// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A shelfmark cites a portable capture by its content address and the
//! authority's generation (site canvas plan, S6 and Rulings 131-132); the check
//! is written here as a host would write it, not shipped.
//!
//! This was the second test of `graphshell-client`'s `portable_projection`
//! tests, which read the site artifact from scenotime's fixtures. It needs
//! `incipit`, which is Mere's, so it moved to Mere when `graphshell-client`
//! moved to the Graphshell repository. The capture is built here from a small
//! scene, since the shelfmark cites any capture by its bytes and the site
//! fixture stays with scenotime.

use chirograph::{
    CaptureAuthorityV1, PROJECTION_CAPTURE_V2, PresentationManifest, ProjectionCaptureV2,
    Sha256NamedInformation,
};
use incipit::{ShelfmarkAuthorityV1, ShelfmarkInputV1, ShelfmarkV1};
use sceno::{Footprint, ProjectedItem, Representation, Scene, SourceRef, Transform2};
use scenotime::{Revision, SceneEpoch, SceneSnapshot};

const GENERATION: u64 = 7;

fn capture() -> ProjectionCaptureV2 {
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
    ProjectionCaptureV2 {
        version: PROJECTION_CAPTURE_V2,
        scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(0), scene).expect("snapshot"),
        presentation: PresentationManifest::default(),
        score: None,
        authority: Some(CaptureAuthorityV1 {
            adapter: "fixture".to_owned(),
            schema: "fixture.authority/v1".to_owned(),
            sha256: Sha256NamedInformation::from_hex(&"ab".repeat(32))
                .expect("lowercase hex names the authority"),
            generation: GENERATION,
        }),
    }
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
    let bytes = capture().encode().expect("the capture encodes");
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
        .expects_generation = (GENERATION - 1).to_string();
    let refused = check(&stale, &capture).expect_err("another generation");
    assert!(refused.starts_with("generation"), "{refused}");
}
