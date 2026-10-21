// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Independent std-only consumer of the published Apparatus extraction.
use apparatus_core::{
    InspectionLimits, ObservationMetadata, ObservationStore, RetentionLimits, inspect_batch,
};
use std::time::Duration;

fn main() -> Result<(), apparatus_core::StoreError> {
    let limits = RetentionLimits {
        max_records: 1,
        max_bytes: 1024,
        max_age: Duration::from_secs(5),
    };
    let mut store = ObservationStore::new("independent-run".into(), "sync-owner".into(), limits);
    let mut inspector = store.cursor();
    let mut receipt = store.cursor();
    for text in ["first", "second"] {
        store.record(
            text,
            text.len(),
            ObservationMetadata::default(),
            Duration::ZERO,
        )?;
    }
    let batch = store.read(&mut inspector, Duration::ZERO, 2)?;
    assert_eq!(store.read(&mut receipt, Duration::ZERO, 2)?, batch);
    assert_eq!(batch.stats.loss.evicted, 1);
    assert_eq!(batch.gaps[0].first_sequence, 1);
    assert_eq!(batch.records[0].payload, "second");
    let view = inspect_batch(
        &batch,
        limits,
        InspectionLimits {
            max_records: 1,
            max_gaps: 1,
            max_text_chars: 128,
        },
        |p| *p,
    );
    assert!(
        view.lines
            .iter()
            .any(|line| line.text.contains("Operation: unavailable"))
    );
    Ok(())
}
