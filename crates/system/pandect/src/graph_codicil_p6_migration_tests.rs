// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Focused codicil input compatibility for the assertion-record snapshot schema.

use super::*;
use eidetic::{
    BlobSource, PrivacyClass, SchemaRef, Timestamp, TrustEnvelope, TypedPayload, save_typed_sealed,
};
use euclid::default::Point2D;
use kernel::graph::fixtures::GraphFixtures;
use kernel::graph::{ContainmentSubKind, EdgeAssertion, RelationKind, apply::assert_relation};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Stores a deliberately pre-v4 JSON value under its historical schema tag.
/// The typed layer is JSON for graph codicils, so this tests the admitted bytes
/// instead of manufacturing an unrelated positional binary archive.
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
struct HistoricalJson(Value);

impl TypedPayload for HistoricalJson {
    fn schema_ref() -> SchemaRef {
        v3_profiled_graph_snapshot_schema_ref()
    }
}

fn historical_v3_profiled_json(mut value: Value, from: &str, to: &str) -> Value {
    let snapshot = value
        .get_mut("snapshot")
        .expect("profiled codicil has flattened snapshot field");
    snapshot.as_object_mut().unwrap().remove("assertion_schema");
    for column in ["edges", "resource_edges"] {
        let Some(edges) = snapshot.get_mut(column).and_then(Value::as_array_mut) else {
            continue;
        };
        for edge in edges {
            edge.as_object_mut()
                .expect("persisted edge is an object")
                .remove("assertions");
            if edge["from_node_id"] == from && edge["to_node_id"] == to {
                edge["containment"] = serde_json::json!({ "sub_kinds": ["UserFolder"] });
            }
        }
    }
    value
}

fn relation_graph() -> (Graph, String, String) {
    let mut graph = Graph::new();
    let from = graph.add_node("https://codicil-a.example/".into(), Point2D::new(1.0, 2.0));
    let to = graph.add_node("https://codicil-b.example/".into(), Point2D::new(3.0, 4.0));
    assert_relation(
        &mut graph,
        from,
        to,
        EdgeAssertion::Containment {
            sub_kind: ContainmentSubKind::UserFolder,
        },
    )
    .expect("legacy sidecar edge");
    let from_id = graph.get_node(from).unwrap().id.to_string();
    let to_id = graph.get_node(to).unwrap().id.to_string();
    (graph, from_id, to_id)
}

#[test]
fn v3_json_codicil_upgrades_legacy_sidecar_once_and_v4_reopen_keeps_id_and_unknown_time() {
    pollster::block_on(async {
        let (graph, from, to) = relation_graph();
        let mut store = muniment::MemoryBackend::default();
        let profiled = ProfiledGraphCodicil {
            codicil: GraphCodicil {
                snapshot: graph.to_snapshot(),
                facets: graph.facets().clone(),
                sessions: Vec::new(),
            },
            placement: Some(PlacementProfile::RecordedStrataV1),
        };
        let old_json =
            historical_v3_profiled_json(serde_json::to_value(profiled).unwrap(), &from, &to);
        let old_id = save_typed_sealed(
            &mut store,
            None,
            &HistoricalJson(old_json),
            Vec::<BlobSource>::new(),
            PrivacyClass::LocalOnly,
            graph_codicil_provenance(Timestamp(1)),
            TrustEnvelope::self_asserted(),
            Timestamp(1),
        )
        .await
        .unwrap();

        let old_manifest = eidetic::manifest::load_manifest(&mut store, old_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(old_manifest.schema, v3_profiled_graph_snapshot_schema_ref());

        let reopened = open_codicil_as_session(&mut store, old_id)
            .await
            .unwrap()
            .expect("v3 JSON codicil opens");
        let canonical = reopened.to_snapshot();
        let persisted = canonical
            .edges
            .iter()
            .find(|edge| edge.from_node_id == from && edge.to_node_id == to)
            .expect("canonical directed edge");
        let assertions = persisted
            .assertions
            .as_ref()
            .expect("v4 snapshot writes canonical assertions");
        assert_eq!(assertions.len(), 1);
        let statement = &assertions[0];
        assert_eq!(
            statement.predicate,
            RelationKind::Containment(ContainmentSubKind::UserFolder)
                .predicate_iri()
                .unwrap()
        );
        assert_eq!(statement.first_asserted_ms, None);
        assert_eq!(statement.asserted_at_ms, None);
        assert_eq!(statement.withdrawn, None);
        assert!(persisted.containment.is_none());
        let statement_id = statement.statement_id.clone();

        let canonical_id = save_graph_codicil(
            &mut store,
            &reopened,
            RedactionPolicy::include_all(),
            Timestamp(2),
        )
        .await
        .unwrap();
        let manifest = eidetic::manifest::load_manifest(&mut store, canonical_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(manifest.schema, graph_snapshot_schema_ref());
        let listed = list_graph_codicils(&mut store).await.unwrap();
        assert!(listed.iter().any(|manifest| manifest.id == old_id));
        assert!(listed.iter().any(|manifest| manifest.id == canonical_id));

        let reopened_again = open_codicil_as_session(&mut store, canonical_id)
            .await
            .unwrap()
            .expect("v4 codicil reopens");
        let canonical_again = reopened_again.to_snapshot();
        let statement_again = canonical_again
            .edges
            .iter()
            .find(|edge| edge.from_node_id == from && edge.to_node_id == to)
            .and_then(|edge| edge.assertions.as_ref())
            .and_then(|assertions| assertions.first())
            .expect("one canonical assertion survives reopen");
        assert_eq!(statement_again.statement_id, statement_id);
        assert_eq!(statement_again.first_asserted_ms, None);
        assert_eq!(statement_again.asserted_at_ms, None);
    });
}

#[test]
fn schema_ids_keep_v1_v2_v3_inputs_distinct_from_v4_outputs() {
    assert_ne!(graph_snapshot_schema_ref(), v2_graph_snapshot_schema_ref());
    assert_ne!(
        graph_snapshot_schema_ref(),
        v3_profiled_graph_snapshot_schema_ref()
    );
    assert_ne!(
        profiled_graph_snapshot_schema_ref(),
        v3_profiled_graph_snapshot_schema_ref()
    );
    assert_eq!(GRAPH_SNAPSHOT_SCHEMA_ID, b"mere.graph-snapshot/v4");
}
