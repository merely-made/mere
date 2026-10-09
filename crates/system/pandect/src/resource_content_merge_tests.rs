// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use crate::snapshot_merge::try_merge_snapshots;
use kernel::graph::Graph;
use kernel::graph::apply::add_node;
use kernel::types::{
    ClassificationProvenance, ClassificationScheme, ClassificationStatus, NodeClassification,
    NodeProperty,
};

fn graph(status: ClassificationStatus, author: &str, handle: &str) -> Graph {
    let mut graph = Graph::new();
    let key = add_node(
        &mut graph,
        None,
        "https://example.test/page".into(),
        Default::default(),
    );
    assert!(graph.add_node_classifications(
        key,
        vec![NodeClassification {
            scheme: ClassificationScheme::Custom("rdf:type".into()),
            value: "https://types.test/Article".into(),
            label: Some("Article".into()),
            confidence: 0.8,
            provenance: ClassificationProvenance::Imported,
            status,
            primary: false,
        }]
    ));
    let resource = graph.shown_resource_id(key).unwrap();
    let mut property = NodeProperty::new("urn:score".into(), "42".into())
        .with_metadata(Some(author.into()), Some(10));
    property.statement_id = handle.into();
    assert!(
        graph
            .append_resource_properties(resource, vec![property])
            .unwrap()
    );
    graph
}

#[test]
fn composition_preserves_classification_variants_origins_and_literal_handles() {
    let a = graph(
        ClassificationStatus::Accepted,
        "urn:people:alice",
        "alice-score",
    );
    let b = graph(
        ClassificationStatus::Suggested,
        "urn:people:bob",
        "bob-score",
    );
    let left = a.to_snapshot();
    let right = b.to_snapshot();
    let (merged, _) = try_merge_snapshots(&left, &right).unwrap();
    assert_eq!(merged.nodes.len(), 2);
    let resource = chartulary::resource_id("https://example.test/page");
    let materialized = Graph::try_from_recorded_snapshot(&merged).unwrap();
    let variants = materialized.resource_classification_variants(resource);
    assert_eq!(variants.len(), 2);
    assert!(materialized.resource_classifications_conflicted(resource));
    assert!(
        variants
            .iter()
            .any(|variant| variant.classification.status == ClassificationStatus::Accepted)
    );
    assert!(
        variants
            .iter()
            .any(|variant| variant.classification.status == ClassificationStatus::Suggested)
    );
    let surfaces: std::collections::HashSet<_> = variants
        .iter()
        .flat_map(|variant| variant.origins.iter().map(|origin| origin.surface_id))
        .collect();
    assert_eq!(surfaces.len(), 2);
    let properties = materialized.resource_properties(resource);
    assert_eq!(properties.len(), 2);
    assert!(
        properties
            .iter()
            .any(|property| property.statement_id == "alice-score"
                && property.provenance_iri.as_deref() == Some("urn:people:alice"))
    );
    assert!(
        properties
            .iter()
            .any(|property| property.statement_id == "bob-score"
                && property.provenance_iri.as_deref() == Some("urn:people:bob"))
    );
    let (repeat, _) = try_merge_snapshots(&merged, &right).unwrap();
    assert_eq!(repeat.resources, merged.resources);
    assert_eq!(a.to_snapshot().resources, left.resources);
    assert_eq!(b.to_snapshot().resources, right.resources);
}

#[test]
fn conflicting_literal_handle_refuses_composition_without_selecting_a_survivor() {
    let a = graph(
        ClassificationStatus::Accepted,
        "urn:people:alice",
        "held-score",
    );
    let b = graph(
        ClassificationStatus::Suggested,
        "urn:people:bob",
        "held-score",
    );
    let left = a.to_snapshot();
    let right = b.to_snapshot();
    assert!(try_merge_snapshots(&left, &right).is_err());
    assert_eq!(a.to_snapshot().resources, left.resources);
    assert_eq!(b.to_snapshot().resources, right.resources);
    let distinct = graph(
        ClassificationStatus::Suggested,
        "urn:people:bob",
        "other-score",
    );
    assert!(try_merge_snapshots(&left, &distinct.to_snapshot()).is_ok());
}
