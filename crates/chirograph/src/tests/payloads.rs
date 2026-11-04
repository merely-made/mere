// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn editable_text_and_save_payloads_are_versioned_and_strict() {
    assert_eq!(ProtocolVersion::V1_1.minor, 1);
    assert_eq!(ProtocolVersion::V1_2.minor, 2);
    assert_eq!(ProtocolVersion::V1.minor, 3);
    let editable = EditableTextV1 {
        address: "knot://field-note".into(),
        media_type: "text/vnd.knot".into(),
        encoding: TextEncoding::Utf8,
        source: "# Field note\n".into(),
        base_token: vec![1, 2, 3],
        public_revision: None,
        derived: Some(DerivedTextV1 {
            source: "# Field note\n\nFetched.\n".into(),
            summary: "resolved 1; denied 0; failed 0".into(),
            cache: Some(DerivedCacheInfoV1 {
                effect: "resolve".into(),
                sources: vec!["https://example.test/field".into()],
                provider_version: "fixture/v1".into(),
                policy_fingerprint: "policy-digest".into(),
                fetched_at_unix_ms: 42,
                source_revision: 7,
            }),
        }),
    };
    let bytes = serde_json::to_vec(&editable).unwrap();
    assert_eq!(
        serde_json::from_slice::<EditableTextV1>(&bytes).unwrap(),
        editable
    );
    let legacy = br#"{"address":"knot://legacy","media_type":"text/vnd.knot","encoding":"Utf8","source":"old","base_token":[1]}"#;
    assert!(
        serde_json::from_slice::<EditableTextV1>(legacy)
            .unwrap()
            .public_revision
            .is_none()
    );
    assert!(
        serde_json::from_slice::<EditableTextV1>(legacy)
            .unwrap()
            .derived
            .is_none()
    );
    let protocol_1_2_derived =
        br##"{"source":"# Field note\n\nFetched.\n","summary":"resolved 1"}"##;
    assert!(
        serde_json::from_slice::<DerivedTextV1>(protocol_1_2_derived)
            .unwrap()
            .cache
            .is_none()
    );

    let save = SaveTextV1 {
        base_token: vec![1, 2, 3],
        source: "# Revised\n".into(),
    };
    let wire = serde_json::to_string(&save).unwrap();
    assert_eq!(serde_json::from_str::<SaveTextV1>(&wire).unwrap(), save);
    assert!(
        serde_json::from_str::<SaveTextV1>(
            r#"{"base_token":[1,2,3],"source":"x","path":"outside.knot"}"#
        )
        .is_err(),
        "a save payload cannot smuggle endpoint-native authority"
    );

    let clip = InsertKnotClipV1 {
        base_token: vec![1, 2, 3],
        source_url: "https://example.test/field".into(),
        title: Some("Field report".into()),
        selector: Some("main > article".into()),
        knot_body: "# Field report\n\nA finding.\n".into(),
    };
    let wire = serde_json::to_string(&clip).unwrap();
    assert_eq!(
        serde_json::from_str::<InsertKnotClipV1>(&wire).unwrap(),
        clip
    );
    assert!(
        serde_json::from_str::<InsertKnotClipV1>(
            r#"{"base_token":[1],"source_url":"https://example.test","title":null,"selector":null,"knot_body":"x","path":"outside.knot"}"#
        )
        .is_err(),
        "a clip payload cannot smuggle endpoint-native authority"
    );

    let effect = KnotEffectV1 {
        base_token: vec![1, 2, 3],
        confirmed: true,
    };
    let wire = serde_json::to_string(&effect).unwrap();
    assert_eq!(serde_json::from_str::<KnotEffectV1>(&wire).unwrap(), effect);
    assert!(
        serde_json::from_str::<KnotEffectV1>(
            r#"{"base_token":[1,2,3],"confirmed":true,"command":"rm"}"#
        )
        .is_err(),
        "an effect payload cannot smuggle an evaluator or fetch command"
    );
}

#[test]
fn a_resource_larger_than_one_frame_arrives_whole_and_verified() {
    // Deliberately not a multiple of the window, so the last chunk is short.
    let bytes: Vec<u8> = (0..300_000u32).map(|index| (index % 251) as u8).collect();
    let whole = resource(bytes.clone());
    assert_eq!(pull(&whole, 64 * 1024).unwrap(), bytes);
    // A window of one proves the loop terminates on the awkward sizes too.
    assert_eq!(pull(&resource(vec![7; 5]), 1).unwrap(), vec![7; 5]);
    // Zero means "endpoint chooses" rather than an infinite stall.
    assert_eq!(pull(&whole, 0).unwrap(), bytes);
}

#[test]
fn an_empty_resource_completes_in_one_chunk() {
    let whole = resource(Vec::new());
    let chunk = ResourceChunkResponse::slice(&whole, 0, 1024);
    assert_eq!(chunk.total_len, 0);
    assert_eq!(
        ResourceAssembly::new().accept(&chunk).unwrap(),
        Some(Vec::new())
    );
}

#[test]
fn a_client_request_is_clamped_rather_than_trusted() {
    let whole = resource(vec![0; 4 * 1024 * 1024]);
    let chunk = ResourceChunkResponse::slice(&whole, 0, u32::MAX);
    assert_eq!(
        chunk.decode().unwrap().len(),
        MAX_RESOURCE_CHUNK_BYTES as usize,
        "a client asking for everything must not get a frame over the cap"
    );
}

#[test]
fn base64_keeps_a_full_chunk_inside_the_native_message_frame() {
    let whole = resource(vec![0xa5; MAX_RESOURCE_CHUNK_BYTES as usize]);
    let chunk = ResourceChunkResponse::slice(&whole, 0, MAX_RESOURCE_CHUNK_BYTES);
    let framed = serde_json::to_vec(&CarrierResponse {
        id: 1,
        body: Ok(CarrierResponseBody::ResourceChunk(Box::new(chunk))),
    })
    .unwrap();
    // The 1 MiB cap is browser_carrier's MAX_NATIVE_MESSAGE_BYTES. This is
    // the assertion that keeps the chunk size and the frame cap in step;
    // raising one without the other should fail here rather than in a
    // browser.
    assert!(
        framed.len() < 1024 * 1024,
        "a full chunk framed to {} bytes, over the native-messaging cap",
        framed.len()
    );
}

#[test]
fn a_damaged_or_misordered_chunk_is_refused_rather_than_assembled() {
    let whole = resource(vec![1, 2, 3, 4, 5, 6, 7, 8]);
    let mut assembly = ResourceAssembly::new();

    let mut corrupt = ResourceChunkResponse::slice(&whole, 0, 4);
    corrupt.bytes = base64_encode(b"xxxx");
    assert_eq!(
        assembly.accept(&corrupt),
        Err(AssemblyError::CorruptChunk),
        "a frame that fails its own address must not enter the assembly"
    );

    let skipped = ResourceChunkResponse::slice(&whole, 4, 4);
    assert_eq!(
        assembly.accept(&skipped),
        Err(AssemblyError::OutOfOrder {
            expected: 0,
            found: 4
        })
    );

    // Intact frames, but the whole they claim to form is another resource.
    let mut first = ResourceChunkResponse::slice(&whole, 0, 8);
    first.resource = ContentHash::of(b"a different resource entirely");
    assert_eq!(
        assembly.accept(&first),
        Err(AssemblyError::WrongResource),
        "per-frame checks passing must not be mistaken for the whole verifying"
    );
}

#[test]
fn a_chunk_reply_round_trips_through_the_carrier_envelope() {
    let whole = resource(b"round trip".to_vec());
    let response = CarrierResponse {
        id: 11,
        body: Ok(CarrierResponseBody::ResourceChunk(Box::new(
            ResourceChunkResponse::slice(&whole, 0, 4),
        ))),
    };
    let json = serde_json::to_string(&response).unwrap();
    assert_eq!(
        serde_json::from_str::<CarrierResponse>(&json).unwrap(),
        response
    );
    let request = CarrierRequest {
        id: 12,
        body: CarrierRequestBody::ResourceChunk(ResourceChunkRequest {
            session: ProjectionSession("fixture:blobs".into()),
            resource: whole.resource,
            offset: 4,
            length: 4,
        }),
    };
    let json = serde_json::to_string(&request).unwrap();
    assert_eq!(
        serde_json::from_str::<CarrierRequest>(&json).unwrap(),
        request
    );
}
