// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeMap;

use super::*;
use sceno::{Arrangement, Scene, Spiral};

fn form_action(form: ActionFormV1) -> AdvertisedAction {
    AdvertisedAction {
        intent: IntentReference("fixture.save-pair".into()),
        label: "Save pair".into(),
        explanation: "Record the selected values together.".into(),
        payload_schema: "fixture.intent.save-pair/v1".into(),
        input_form: Some(form),
        effect: IntentEffect::DomainTruth,
    }
}

fn projection_capture_fixture() -> ProjectionCaptureV1 {
    let resource = ContentHash::of(b"captured presentation");
    let presentation = PresentationManifest {
        bindings: Vec::new(),
        offers: BTreeMap::from([(
            PresentationKey("fixture:scene".to_string()),
            vec![PresentationOffer {
                codec: PresentationCodec::PortableCardV1,
                resource,
                byte_size: 19,
                requires: PresentationCapability::PortableCard,
                semantics: PresentationSemantics {
                    label: "Captured fixture".to_string(),
                    role: SemanticRole::Graphic,
                    bounds: BoundsRelationship::FitWithinFootprint,
                    actions: Vec::new(),
                },
            }],
        )]),
    };
    ProjectionCaptureV1 {
        version: PROJECTION_CAPTURE_VERSION,
        scene: SceneSnapshot::from_dense(SceneEpoch(9), Revision(4), Scene::new())
            .expect("fixture scene is valid"),
        presentation,
    }
}

#[test]
fn projection_capture_replays_same_tables_bounds_and_presentation_addresses() {
    let capture = projection_capture_fixture();
    let bytes = capture.encode().expect("capture serializes");
    let restored = ProjectionCaptureV1::decode(&bytes).expect("projection capture reopens");
    assert_eq!(restored.scene.tables, capture.scene.tables);
    assert_eq!(restored.scene.tables.bounds, capture.scene.tables.bounds);
    assert_eq!(
        restored.presentation_resources(),
        capture.presentation_resources()
    );
    assert_eq!(
        restored.content_address().expect("address"),
        capture.content_address().expect("address"),
        "the same projection capture has the same content address"
    );
    assert!(
        restored
            .verify_resources(|resource| capture.presentation_resources().contains(&resource))
            .is_ok(),
        "all named presentation resources are available"
    );
    let missing = restored
        .verify_resources(|_| false)
        .expect_err("a missing resource must not be silently omitted");
    assert!(matches!(
        missing,
        ProjectionCaptureError::MissingPresentationResource(_)
    ));
}

#[test]
fn opaque_live_view_reference_round_trips_without_source_disclosure() {
    let reference = LiveViewReferenceV1::new("eidetic:6f2a4c");
    let restored =
        LiveViewReferenceV1::decode(&reference.encode().expect("encode")).expect("decode");
    assert_eq!(restored, reference);
    assert!(
        !serde_json::to_string(&restored)
            .expect("serialize")
            .contains("session:fixture"),
        "the carrier receives an opaque record reference, not source truth"
    );
    assert!(matches!(
        LiveViewReferenceV1::new(" ").validate(),
        Err(LiveViewReferenceError::MissingRecord)
    ));
    assert!(matches!(
        LiveViewReferenceV1::new("file:C:/private/graph.json").validate(),
        Err(LiveViewReferenceError::InvalidOpaqueReference)
    ));
}

#[test]
fn a_selection_round_trips_and_keeps_its_source() {
    let selection = Selection::one("sandbox", "node", "mere");
    let wire = serde_json::to_string(&selection).expect("serialize");
    let far_side: Selection = serde_json::from_str(&wire).expect("deserialize");
    assert_eq!(far_side, selection);
    assert_eq!(far_side.source, "sandbox");
    assert_eq!(far_side.targets[0].kind, "node");
}

#[test]
fn a_cleared_selection_is_not_an_absent_one() {
    let cleared = Selection::cleared("sandbox");
    assert!(cleared.is_cleared());
    // It survives the wire as a cleared selection rather than vanishing,
    // so a second view learns "they deselected" instead of "no news".
    let wire = serde_json::to_string(&cleared).expect("serialize");
    let far_side: Selection = serde_json::from_str(&wire).expect("deserialize");
    assert!(far_side.is_cleared());
    assert_eq!(far_side.source, "sandbox");
}

#[test]
fn foreignness_is_answerable_which_is_why_source_is_mandatory() {
    let mine = Selection::one("canvas", "node", "a");
    let theirs = Selection::one("swatch", "node", "b");
    let also_mine = Selection::one("canvas", "node", "c");
    assert!(mine.is_foreign_to(&theirs));
    assert!(!mine.is_foreign_to(&also_mine));
}

#[test]
fn crossfilter_applies_foreign_clauses_and_excludes_the_consuming_view() {
    let mut coordinated = CoordinatedSelection::new(SelectionResolution::Crossfilter);
    coordinated.set(
        SelectionRole::Focus,
        Selection::one("spatial", "node", "mere"),
    );
    coordinated.set(
        SelectionRole::Brush,
        Selection::one("matrix", "node", "turnstone"),
    );

    assert_eq!(
        coordinated.targets_for("spatial").unwrap(),
        BTreeSet::from([SelectionTarget {
            kind: "node".into(),
            id: "turnstone".into(),
        }])
    );
    assert_eq!(
        coordinated.targets_for("matrix").unwrap(),
        BTreeSet::from([SelectionTarget {
            kind: "node".into(),
            id: "mere".into(),
        }])
    );
}

#[test]
fn clause_removal_restores_the_unfiltered_reading() {
    let mut coordinated = CoordinatedSelection::new(SelectionResolution::Crossfilter);
    coordinated.set(
        SelectionRole::Brush,
        Selection::one("matrix", "node", "mere"),
    );
    assert!(coordinated.targets_for("spatial").is_some());
    assert!(coordinated.remove("matrix"));
    assert_eq!(coordinated.targets_for("spatial"), None);
}

#[test]
fn coordinated_selection_wire_is_deterministic() {
    let mut left = CoordinatedSelection::new(SelectionResolution::Crossfilter);
    left.set(
        SelectionRole::Brush,
        Selection::one("matrix", "node", "mere"),
    );
    left.set(
        SelectionRole::Focus,
        Selection::one("spatial", "node", "turnstone"),
    );

    let mut right = CoordinatedSelection::new(SelectionResolution::Crossfilter);
    right.set(
        SelectionRole::Focus,
        Selection::one("spatial", "node", "turnstone"),
    );
    right.set(
        SelectionRole::Brush,
        Selection::one("matrix", "node", "mere"),
    );

    let wire = serde_json::to_string(&left).expect("serialize coordinated selection");
    assert_eq!(wire, serde_json::to_string(&right).unwrap());
    assert_eq!(
        serde_json::from_str::<CoordinatedSelection>(&wire).unwrap(),
        left
    );
    assert!(wire.find("matrix").unwrap() < wire.find("spatial").unwrap());
}

#[test]
fn action_form_composes_exact_selected_values_under_its_schema() {
    let action = form_action(
        ActionFormV1::new("fixture.intent.save-pair/v1")
            .with_field(
                ActionFormFieldV1::choice(
                    "facts_digest",
                    "Astrology facts",
                    [
                        ActionFormChoiceV1::new("facts-a", "Morning facts"),
                        ActionFormChoiceV1::new("facts-b", "Evening facts"),
                    ],
                )
                .with_description("Choose one saved fact set."),
            )
            .with_field(ActionFormFieldV1::choice(
                "reading_session_id",
                "Reading session",
                [ActionFormChoiceV1::new("session-7", "Three-card cast")],
            )),
    );
    let values = BTreeMap::from([
        ("facts_digest".to_string(), "facts-b".to_string()),
        ("reading_session_id".to_string(), "session-7".to_string()),
    ]);

    let payload: serde_json::Value = serde_json::from_slice(
        &action
            .compose_payload(&values)
            .expect("valid selected values"),
    )
    .expect("form output is JSON");
    assert_eq!(
        payload,
        serde_json::json!({
            "schema": "fixture.intent.save-pair/v1",
            "facts_digest": "facts-b",
            "reading_session_id": "session-7",
        })
    );
}

#[test]
fn action_form_refuses_missing_unknown_and_unadvertised_values() {
    let action = form_action(ActionFormV1::new("fixture.intent.save-pair/v1").with_field(
        ActionFormFieldV1::choice(
            "facts_digest",
            "Astrology facts",
            [ActionFormChoiceV1::new("facts-a", "Morning facts")],
        ),
    ));
    assert_eq!(
        action.compose_payload(&BTreeMap::new()),
        Err(ActionFormError::MissingField("facts_digest".into()))
    );
    assert_eq!(
        action.compose_payload(&BTreeMap::from([(
            "facts_digest".to_string(),
            "facts-missing".to_string(),
        )])),
        Err(ActionFormError::InvalidChoice {
            field: "facts_digest".into(),
            value: "facts-missing".into(),
        })
    );
    assert_eq!(
        action.compose_payload(&BTreeMap::from([(
            "unexpected".to_string(),
            "value".to_string(),
        )])),
        Err(ActionFormError::UnknownField("unexpected".into()))
    );
}

#[test]
fn an_action_without_input_form_keeps_the_existing_empty_payload_path() {
    let action = AdvertisedAction {
        intent: IntentReference("fixture.open".into()),
        label: "Open".into(),
        explanation: "Open a fixture.".into(),
        payload_schema: "fixture.open/v1".into(),
        input_form: None,
        effect: IntentEffect::ExternalEffect,
    };
    assert_eq!(action.compose_payload(&BTreeMap::new()), Ok(Vec::new()));
    assert_eq!(
        action.compose_payload(&BTreeMap::from([("title".to_string(), "x".to_string())])),
        Err(ActionFormError::NoInputForm)
    );
    let legacy = r#"{
            "intent":"fixture.open",
            "label":"Open",
            "explanation":"Open a fixture.",
            "payload_schema":"fixture.open/v1",
            "effect":"ExternalEffect"
        }"#;
    assert!(
        serde_json::from_str::<AdvertisedAction>(legacy)
            .expect("old action wire remains readable")
            .input_form
            .is_none()
    );
}

#[test]
fn a_revision_notice_is_distinct_from_a_keyed_response() {
    let notice = CarrierNotice {
        session: ProjectionSession("knot:directory".into()),
        epoch: SceneEpoch(1),
        revision: Revision(7),
    };
    let wire = serde_json::to_string(&CarrierOutput::Notice(notice.clone())).unwrap();
    assert_eq!(
        serde_json::from_str::<CarrierOutput>(&wire).unwrap(),
        CarrierOutput::Notice(notice)
    );
    assert!(!wire.contains("\"id\""));
    assert!(!wire.contains("scene"));
}

#[test]
fn an_open_negotiates_capabilities_and_carries_no_principal() {
    // The absence is the assertion: admission belongs to the carrier's
    // handshake, so this message has nowhere to put a claimed identity.
    let open = SessionOpen {
        version: ProtocolVersion::V1,
        capabilities: CapabilityProfile::new([PresentationCapability::PortableCard]),
    };
    let request = CarrierRequest {
        id: 1,
        body: CarrierRequestBody::Open(Box::new(open.clone())),
    };
    let wire = serde_json::to_string(&request).unwrap();
    assert_eq!(
        serde_json::from_str::<CarrierRequest>(&wire).unwrap(),
        request
    );
    assert!(
        !wire.contains("subject") && !wire.contains("grant"),
        "no principal travels in the application protocol: {wire}"
    );
}

#[test]
fn an_accepted_session_asserts_no_endpoint_key() {
    // A key in this frame would be unverified by construction; the peer's
    // identity is whatever the carrier authenticated.
    let response = CarrierResponse {
        id: 1,
        body: Ok(CarrierResponseBody::Opened(Box::new(SessionOpened {
            version: ProtocolVersion::V1,
            descriptor: EndpointDescriptor {
                label: "turnstone".into(),
                projections: Vec::new(),
            },
            status: SessionStatus::Live,
            expires_at_ms: Some(1_000),
        }))),
    };
    let wire = serde_json::to_string(&response).unwrap();
    assert_eq!(
        serde_json::from_str::<CarrierResponse>(&wire).unwrap(),
        response
    );
    assert!(
        !wire.contains("endpoint_subject"),
        "the endpoint does not assert its own key: {wire}"
    );
}

#[test]
fn resynchronize_is_resume_not_snapshot() {
    // The client's own recovery path is `Resynchronize(ResumeRequest)`:
    // when an epoch or base revision disagrees it resumes, and the
    // endpoint may answer with diffs, an ack, or a snapshot. `Snapshot`
    // belongs to the projection plane and carries a score, so it selects a
    // projection rather than recovering a session.
    let session = ProjectionSession("s".into());
    let resync = CarrierRequestBody::Resume(ResumeRequest {
        session: session.clone(),
        epoch: SceneEpoch(1),
        revision: Revision(4),
    });
    match &resync {
        CarrierRequestBody::Resume(request) => {
            assert_eq!(request.epoch, SceneEpoch(1));
            assert_eq!(request.revision, Revision(4));
        },
        other => panic!("resynchronize recovers a session: {other:?}"),
    }
    // Every session verb still round-trips.
    for verb in [
        CarrierRequestBody::Open(Box::new(SessionOpen {
            version: ProtocolVersion::V1,
            capabilities: CapabilityProfile::default(),
        })),
        CarrierRequestBody::Close,
        CarrierRequestBody::Suspend,
        resync,
    ] {
        let request = CarrierRequest { id: 0, body: verb };
        let wire = serde_json::to_string(&request).unwrap();
        assert_eq!(
            serde_json::from_str::<CarrierRequest>(&wire).unwrap(),
            request
        );
    }
    let _ = session;
}

#[test]
fn request_serializes_a_product_free_score() {
    let request = ProjectionRequest {
        version: ProtocolVersion::V1,
        session: ProjectionSession("local:fixture".into()),
        score: Score::new(Arrangement::Spiral(Spiral::default())),
    };
    let json = serde_json::to_string(&request).unwrap();
    assert_eq!(
        serde_json::from_str::<ProjectionRequest>(&json).unwrap(),
        request
    );
    assert!(std::any::type_name::<Score>().starts_with("sceno::"));
}

#[test]
fn snapshot_keeps_presentation_beside_the_scene() {
    let snapshot = ProjectionSnapshot {
        version: ProtocolVersion::V1,
        session: ProjectionSession("local:fixture".into()),
        scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), Scene::new()).unwrap(),
        presentation: PresentationManifest::default(),
        cache_policy: CachePolicy::default(),
    };
    let json = serde_json::to_string(&snapshot).unwrap();
    assert!(json.contains("presentation"));
    assert_eq!(
        serde_json::from_str::<ProjectionSnapshot>(&json).unwrap(),
        snapshot
    );
}

#[test]
fn resource_address_detects_changed_bytes() {
    let mut response = ResourceResponse::new(
        ProjectionSession("local:fixture".into()),
        b"card bytes".to_vec(),
    );
    assert!(response.has_valid_address());
    response.bytes.push(b'!');
    assert!(!response.has_valid_address());
}

#[test]
fn diff_replay_round_trips_as_a_carrier_neutral_message() {
    let reply = ResumeReply::Diffs(vec![ProjectionDiff {
        version: ProtocolVersion::V1,
        session: ProjectionSession("local:fixture".into()),
        scene: SceneDiff {
            epoch: SceneEpoch(5),
            base: Revision(8),
            revision: Revision(9),
            operations: Vec::new(),
        },
        presentation: Vec::new(),
        status: Some(SessionStatus::Live),
    }]);
    let json = serde_json::to_string(&reply).unwrap();
    assert_eq!(serde_json::from_str::<ResumeReply>(&json).unwrap(), reply);
}

#[test]
fn discovery_and_requests_share_one_framed_vocabulary() {
    let request = CarrierRequest {
        id: 7,
        body: CarrierRequestBody::Discover,
    };
    let json = serde_json::to_string(&request).unwrap();
    assert_eq!(
        serde_json::from_str::<CarrierRequest>(&json).unwrap(),
        request
    );

    let descriptor = EndpointDescriptor {
        label: "Fixture".into(),
        projections: vec![ProjectionOffer {
            label: "Notes".into(),
            request: ProjectionRequest {
                version: ProtocolVersion::V1,
                session: ProjectionSession("fixture:notes".into()),
                score: Score::new(Arrangement::Spiral(Spiral::default())),
            },
        }],
    };
    let response = CarrierResponse {
        id: 7,
        body: Ok(CarrierResponseBody::Descriptor(descriptor)),
    };
    let json = serde_json::to_string(&response).unwrap();
    assert_eq!(
        serde_json::from_str::<CarrierResponse>(&json).unwrap(),
        response
    );
}

fn resource(bytes: Vec<u8>) -> ResourceResponse {
    ResourceResponse::new(ProjectionSession("fixture:blobs".into()), bytes)
}

/// Pull a whole resource the way a client does: request, accept, repeat
/// from wherever the assembly says it is.
fn pull(whole: &ResourceResponse, window: u32) -> Result<Vec<u8>, AssemblyError> {
    let mut assembly = ResourceAssembly::new();
    loop {
        let chunk = ResourceChunkResponse::slice(whole, assembly.received(), window);
        if let Some(bytes) = assembly.accept(&chunk)? {
            return Ok(bytes);
        }
    }
}

#[path = "tests/payloads.rs"]
mod payloads;
