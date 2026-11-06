// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
use super::*;

fn stamp(sequence: u64) -> PresentedFrame {
    PresentedFrame {
        host: 2,
        sequence,
        width: 80,
        height: 60,
        layout_scale: 1.5,
        context_generation: cambium_rootstock::ContextGeneration(1),
    }
}

#[test]
fn prepared_product_facts_are_owned_before_later_model_changes() {
    let mut model = serde_json::json!({"selected": 8, "camera": "A", "revision": 3});
    let proposed = stamp(1);
    let frozen = CaptureProjection {
        fields: BTreeMap::new(),
        viewport: None,
        product: model.clone(),
    };
    model["selected"] = 99.into();
    model["camera"] = "B".into();
    model["revision"] = 4.into();
    let sealed = seal_projection("run-a", 3, proposed, proposed, Ok(frozen), 4096).unwrap();
    assert_eq!(
        sealed.projection.product,
        serde_json::json!({"selected":8,"camera":"A","revision":3})
    );
    assert_ne!(sealed.projection.product, model);
    assert_eq!(sealed.presentation, proposed);
    assert!(sealed.encoded_bytes > 0);
}

#[test]
fn a_different_acceptance_stamp_cannot_relabel_frozen_facts() {
    let projection = CaptureProjection {
        fields: BTreeMap::new(),
        viewport: None,
        product: serde_json::json!({"revision":3}),
    };
    let why = seal_projection("run-a", 1, stamp(1), stamp(2), Ok(projection), 4096)
        .err()
        .unwrap();
    assert!(why.contains("changed after preparation"));
    assert!(
        seal_projection(
            "run-a",
            1,
            stamp(1),
            stamp(1),
            Err("product refusal".into()),
            4096
        )
        .is_err()
    );
}

#[test]
fn prepared_projection_still_obeys_the_encoded_metadata_limit() {
    let projection = CaptureProjection {
        fields: BTreeMap::new(),
        viewport: None,
        product: serde_json::json!({"value":"payload"}),
    };
    assert!(seal_projection("run-a", 1, stamp(1), stamp(1), Ok(projection), 1).is_err());
}
