// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A malformed facet record in a legacy journal is refused by name; an obsolete
//! record that has no live form is skipped, as it always was.
use super::legacy_resource_migration::{self, migrate_legacy_prefix};
use super::*;
use euclid::default::Point2D;
use uuid::Uuid;

const FACET: &str = "example.portable/v1";

fn baseline() -> Graph {
    let mut graph = Graph::new();
    for id in 1..=2u128 {
        graph.add_node_with_id(
            Uuid::from_u128(id),
            format!("https://n{id}.test/"),
            Point2D::zero(),
        );
    }
    graph
}

fn entry(delta: CapturedDelta) -> AttributedDelta {
    AttributedDelta {
        author: Author::person("legacy-author"),
        delta,
    }
}

fn facet(node: u128, value_json: &str) -> AttributedDelta {
    entry(CapturedDelta::ReplaySetNodeFacetById {
        node_id: Uuid::from_u128(node).to_string(),
        facet: FACET.into(),
        value_json: value_json.into(),
    })
}

#[test]
fn legacy_migration_refuses_a_malformed_facet_record_naming_the_node_and_facet() {
    let baseline = baseline();
    let valid = facet(1, r#"{"kept":true}"#);
    let migrated = migrate_legacy_prefix(&baseline, std::slice::from_ref(&valid)).unwrap();
    let id = chartulary::FacetId::new(FACET);
    assert!(
        migrated
            .graph
            .facets()
            .get(&Uuid::from_u128(1), &id)
            .is_some()
    );

    let entries = [valid, facet(2, "{not json")];
    let error = migrate_legacy_prefix(&baseline, &entries).err().unwrap();
    assert!(error.0.contains(&Uuid::from_u128(2).to_string()), "{error}");
    assert!(error.0.contains(FACET), "{error}");
    assert!(
        !error.0.contains(&Uuid::from_u128(1).to_string()),
        "{error}"
    );
}

#[test]
fn legacy_migration_still_skips_an_obsolete_thumbnail_record() {
    let baseline = baseline();
    let thumbnail = entry(CapturedDelta::ReplaySetNodeThumbnailById {
        node_id: Uuid::from_u128(1).to_string(),
        png_bytes: vec![0, 1, 2],
        width: 1,
        height: 1,
    });
    let skipped = migrate_legacy_prefix(&baseline, &[thumbnail]).unwrap();
    let plain = migrate_legacy_prefix(&baseline, &[]).unwrap();
    let nodes = |replay: &legacy_resource_migration::MigratedReplay| {
        serde_json::to_value(replay.graph.to_snapshot().nodes).unwrap()
    };
    assert_eq!(nodes(&skipped), nodes(&plain));
    assert_eq!(skipped.graph.facets(), plain.graph.facets());
}
