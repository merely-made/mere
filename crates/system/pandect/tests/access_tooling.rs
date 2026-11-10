// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The access lane stamps what its caller names, not pandect (engine split D89).

use eidetic::manifest::load_manifest;
use eidetic::{PrivacyClass, TypedPayload};
use muniment::MemoryBackend;
use pandect::access::{
    AccessAction, AccessRecord, AccessTransition, bootstrap_access_record_schema,
    save_access_record,
};
use uuid::Uuid;

async fn tooling_of(store: &mut MemoryBackend, id: eidetic::ManifestId) -> Option<String> {
    load_manifest(store, id)
        .await
        .unwrap()
        .expect("the manifest was written")
        .provenance
        .tooling
}

#[test]
fn the_schema_and_each_record_carry_the_label_their_caller_passed() {
    pollster::block_on(async {
        let mut store = MemoryBackend::new();
        bootstrap_access_record_schema(&mut store, "probe-app/9.8.7")
            .await
            .unwrap();
        let record = AccessRecord {
            record_id: Uuid::from_bytes([1; 16]),
            container_id: Uuid::from_bytes([2; 16]),
            address: "https://example.invalid/tooling".to_string(),
            action: AccessAction::Open,
            persona: "persona:probe".to_string(),
            device: "device:probe".to_string(),
            application: "probe".to_string(),
            at_ms: 1_700_000_000_000,
            handler: "system.default".to_string(),
            dwell_ms: None,
            referring_container_id: None,
            referring_address: None,
            transition: AccessTransition::UrlTyped,
            capture_source: "probe".to_string(),
            source_event_id: None,
            privacy: PrivacyClass::PublicPortable,
        };
        let id = save_access_record(&mut store, &record, "probe-app-access/9.8.7")
            .await
            .unwrap();

        assert_eq!(
            tooling_of(&mut store, AccessRecord::schema_ref().0).await.as_deref(),
            Some("probe-app/9.8.7")
        );
        assert_eq!(
            tooling_of(&mut store, id).await.as_deref(),
            Some("probe-app-access/9.8.7")
        );
    });
}
