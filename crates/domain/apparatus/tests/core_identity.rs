// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Receiving proofs for the extraction's public type and wire identity.
use apparatus::*;
use std::time::Duration;

#[test]
fn facade_and_core_share_the_same_store_batch_and_inspection_types() {
    let limits = RetentionLimits {
        max_records: 2,
        max_bytes: 4096,
        max_age: Duration::from_secs(5),
    };
    let mut store: apparatus_core::ObservationStore<String> =
        ObservationStore::new("run".into(), "source".into(), limits);
    let mut cursor: Cursor = store.cursor();
    store
        .record(
            "redacted".to_string(),
            8,
            ObservationMetadata::default(),
            Duration::ZERO,
        )
        .unwrap();
    let batch: Batch<String> = store.read(&mut cursor, Duration::ZERO, 4).unwrap();
    let _: &apparatus_core::Batch<String> = &batch;
    let inspection: apparatus_core::Inspection = inspect_batch(
        &batch,
        limits,
        InspectionLimits {
            max_records: 2,
            max_gaps: 2,
            max_text_chars: 128,
        },
        String::as_str,
    );
    assert!(
        inspection
            .lines
            .iter()
            .any(|line| line.text.contains("redacted"))
    );
}

#[cfg(feature = "serde")]
#[test]
fn facade_serde_matches_the_actual_donor_wire_values() {
    let limits = RetentionLimits {
        max_records: 2,
        max_bytes: 4096,
        max_age: Duration::from_secs(5),
    };
    let mut store = ObservationStore::new("wire-run".into(), "worker".into(), limits);
    let mut cursor = store.cursor();
    for value in ["first", "é琴𝄞", "last"] {
        let metadata = ObservationMetadata {
            source_boot: Some("boot".into()),
            source_time: Some(Duration::from_millis(7)),
            operation: Some("save:1".into()),
            cause: Some(RecordRef {
                run: "parent".into(),
                source: "owner".into(),
                sequence: 9,
            }),
            subject: Some(SubjectRef {
                namespace: "doc".into(),
                id: "example".into(),
            }),
            semantic_revision: Some(3),
            presented_frame: Some(4),
        };
        store
            .record(
                value.to_string(),
                value.len(),
                metadata,
                Duration::from_secs(1),
            )
            .unwrap();
    }
    let batch = store.read(&mut cursor, Duration::from_secs(2), 4).unwrap();
    let encoded = serde_json::json!({"batch": batch, "cursor": cursor});
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("donor-wire-v1.json")).unwrap();
    assert_eq!(encoded, expected);
    let decoded: apparatus_core::Batch<String> =
        serde_json::from_value(encoded["batch"].clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), expected["batch"]);
}
