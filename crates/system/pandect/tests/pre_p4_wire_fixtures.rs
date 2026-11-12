// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The saved scene stays a typed part of pandect's sync grammar and codicil
//! (engine split D134, D69). Payloads written by the code before the sidecar
//! moves decode strictly and re-encode to the same bytes.

#[cfg(feature = "personal-sync")]
#[test]
fn a_sync_record_written_before_the_change_re_encodes_unchanged() {
    use pandect::personal_sync::{PersonalGraphEvent, PersonalGraphRecord};

    let bytes: &[u8] = include_bytes!("fixtures/pre_p4_sync_record_with_scene.cbor");
    let record: PersonalGraphRecord =
        p2panda_core::cbor::decode_cbor_strict(bytes).expect("the old record decodes");
    let scene = record
        .events
        .iter()
        .find_map(|event| match event {
            PersonalGraphEvent::SaveScene { scene, .. } => Some(scene),
            _ => None,
        })
        .expect("the record carries its scene");
    assert_eq!(scene.name, "Shared scene");
    let dynamics = scene.dynamics.as_ref().expect("the dynamics ride along");
    assert_eq!(dynamics.value()["future"]["k"][2], "x");
    assert_eq!(
        p2panda_core::cbor::encode_cbor(&record).expect("re-encodes"),
        bytes,
        "the wire bytes do not move"
    );
}

#[test]
fn a_product_codicil_written_before_the_change_re_encodes_unchanged() {
    let bytes: &[u8] = include_bytes!("fixtures/pre_p4_product_codicil_with_scene.json");
    let codicil = pandect::product_codicil::decode_profiled_codicil(bytes)
        .expect("the old codicil decodes and materializes");
    let scene = codicil.scene.as_ref().expect("the codicil carries its scene");
    assert_eq!(scene.name, "Codicil scene");
    assert_eq!(
        serde_json::to_vec_pretty(&codicil).expect("re-encodes"),
        bytes,
        "the codicil bytes do not move"
    );
}
