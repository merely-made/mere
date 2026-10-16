// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use kernel::persistence::{PersistedEdge, PersistedSemanticEdgeData};
use kernel::types::{AssertionBasis, AssertionMethod, GraphScope};
use serde::Serialize;
use serde::ser::{SerializeStruct, Serializer};
use uuid::Uuid;

/// The pre-P6 receipt serializer's graph fields, copied from Mere
/// `0f8c1bf59`: GraphSnapshot ended at `shown_resources`, and PersistedEdge
/// ended at `provenance`. Keeping this independent serializer in the test
/// checks the exact legacy tuple bytes consumed by LegacyTranslationReceipt's
/// original digest algorithm without using its current constructor as oracle.
struct HistoricalEdge<'a>(&'a PersistedEdge);

impl Serialize for HistoricalEdge<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let edge = self.0;
        let mut state = serializer.serialize_struct("PersistedEdge", 9)?;
        state.serialize_field("from_node_id", &edge.from_node_id)?;
        state.serialize_field("to_node_id", &edge.to_node_id)?;
        state.serialize_field("families", &edge.families)?;
        state.serialize_field("semantic", &edge.semantic)?;
        state.serialize_field("traversal", &edge.traversal)?;
        state.serialize_field("containment", &edge.containment)?;
        state.serialize_field("arrangement", &edge.arrangement)?;
        state.serialize_field("imported", &edge.imported)?;
        state.serialize_field("provenance", &edge.provenance)?;
        state.end()
    }
}

struct HistoricalSnapshot<'a>(&'a kernel::persistence::GraphSnapshot);

impl Serialize for HistoricalSnapshot<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let snapshot = self.0;
        let mut state = serializer.serialize_struct("GraphSnapshot", 10)?;
        let edges: Vec<_> = snapshot.edges.iter().map(HistoricalEdge).collect();
        let resource_edges: Vec<_> = snapshot.resource_edges.iter().map(HistoricalEdge).collect();
        state.serialize_field("nodes", &snapshot.nodes)?;
        state.serialize_field("edges", &edges)?;
        state.serialize_field("import_records", &snapshot.import_records)?;
        state.serialize_field("timestamp_secs", &snapshot.timestamp_secs)?;
        state.serialize_field("fields", &snapshot.fields)?;
        state.serialize_field("couplings", &snapshot.couplings)?;
        state.serialize_field("navigation", &snapshot.navigation)?;
        state.serialize_field("resources", &snapshot.resources)?;
        state.serialize_field("resource_edges", &resource_edges)?;
        state.serialize_field("shown_resources", &snapshot.shown_resources)?;
        state.end()
    }
}

struct HistoricalFrozenGraph<'a>(&'a FrozenGraph);

impl Serialize for HistoricalFrozenGraph<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("FrozenGraph", 2)?;
        state.serialize_field("graph", &HistoricalSnapshot(&self.0.graph))?;
        state.serialize_field("facets", &self.0.facets)?;
        state.end()
    }
}

fn historical_v1_digest(receipt: &LegacyTranslationReceipt) -> String {
    let payload = (
        receipt.version,
        &receipt.source_digest,
        &receipt.placement,
        HistoricalFrozenGraph(&receipt.baseline),
        &receipt.entry_effects,
        HistoricalFrozenGraph(&receipt.legacy_final),
    );
    blake3::hash(&serde_json::to_vec(&payload).unwrap())
        .to_hex()
        .to_string()
}

fn legacy_receipt_graph(graph: &Graph) -> FrozenGraph {
    let mut frozen = FrozenGraph::of(graph);
    frozen.graph.assertion_schema = 0;
    for edge in &mut frozen.graph.edges {
        let statements = edge
            .assertions
            .take()
            .expect("canonical fixture has assertion records");
        edge.semantic = Some(PersistedSemanticEdgeData {
            statements,
            ..Default::default()
        });
    }
    frozen
}

#[test]
fn old_json_translation_receipt_keeps_checksum_and_reopens_semantic_records() {
    let graph = kernel::graph::replay_captured_deltas([
        CapturedDelta::ReplayAddNodeWithIdIfMissing {
            id: Uuid::from_u128(0xA1).to_string(),
            url: "https://legacy-receipt-a.test/".into(),
            position: [1.0, 2.0],
        },
        CapturedDelta::ReplayAddNodeWithIdIfMissing {
            id: Uuid::from_u128(0xA2).to_string(),
            url: "https://legacy-receipt-b.test/".into(),
            position: [3.0, 4.0],
        },
        CapturedDelta::ReplayAssertRelationByIds {
            from_id: Uuid::from_u128(0xA1).to_string(),
            to_id: Uuid::from_u128(0xA2).to_string(),
            assertion: kernel::graph::EdgeAssertion::Semantic {
                sub_kind: kernel::graph::SemanticSubKind::UserGrouped,
                label: Some("legacy direction".into()),
                decay_progress: None,
            },
        },
    ]);

    let mut canonical = FrozenGraph::of(&graph);
    let first = &mut canonical.graph.edges[0].assertions.as_mut().unwrap()[0];
    first.statement_id = "legacy-receipt-id-1".into();
    first.predicate = "urn:mere:legacy:claim".into();
    first.recognized_sub_kind = None;
    first.label = Some("first reason".into());
    first.basis = AssertionBasis {
        note: Some("first reason".into()),
        method: Some(AssertionMethod {
            iri: "urn:method:legacy-receipt".into(),
            version: Some("v1".into()),
        }),
        evidence: vec!["urn:evidence:legacy-receipt".into()],
    };
    first.graph_scope = GraphScope::Custom("urn:scope:legacy-receipt".into());
    first.provenance_iri = Some("urn:author:legacy-receipt".into());
    first.first_asserted_ms = Some(17);
    first.asserted_at_ms = Some(29);
    let mut second = first.clone();
    second.statement_id = "legacy-receipt-id-2".into();
    second.label = Some("second reason".into());
    second.basis.note = second.label.clone();
    second.withdrawn = Some(kernel::types::AssertionWithdrawal {
        at_ms: Some(31),
        by: Some("urn:author:withdrawer".into()),
    });
    canonical.graph.edges[0]
        .assertions
        .as_mut()
        .unwrap()
        .push(second);

    let old_frozen = legacy_receipt_graph(&canonical.materialize().unwrap());
    let profile = SessionPlacement::legacy(Seq(0), vec![]);
    let receipt = LegacyTranslationReceipt::new(
        "historical-source-digest".into(),
        profile.clone(),
        old_frozen.clone(),
        vec![],
        old_frozen,
    )
    .unwrap();
    let bytes = serde_json::to_vec(&receipt).unwrap();
    let encoded: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(
        encoded["baseline"]["graph"]
            .get("assertion_schema")
            .is_none()
    );
    assert!(
        encoded["legacy_final"]["graph"]
            .get("assertion_schema")
            .is_none()
    );

    let reopened: LegacyTranslationReceipt = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(reopened.translation_digest, receipt.translation_digest);
    assert_eq!(
        reopened.translation_digest,
        historical_v1_digest(&reopened),
        "the checksum matches an independent serializer with the pre-P6 field schema"
    );
    reopened
        .validate(&profile, "historical-source-digest", &[])
        .expect("old checksum and translated graph remain valid after reopen");
    let reopened_graph = reopened.baseline.materialize().unwrap();
    let snapshot = reopened_graph.to_snapshot();
    assert_eq!(snapshot.edges.len(), 1);
    assert_eq!(
        snapshot.edges[0].from_node_id,
        Uuid::from_u128(0xA1).to_string()
    );
    assert_eq!(
        snapshot.edges[0].to_node_id,
        Uuid::from_u128(0xA2).to_string()
    );
    assert_eq!(
        snapshot.edges[0]
            .retained_assertions()
            .iter()
            .map(|record| record.statement_id.as_str())
            .collect::<Vec<_>>(),
        ["legacy-receipt-id-1", "legacy-receipt-id-2"]
    );
    let first = &snapshot.edges[0].retained_assertions()[0];
    assert_eq!(first.first_asserted_ms, Some(17));
    assert_eq!(first.asserted_at_ms, Some(29));
    assert_eq!(
        first.graph_scope,
        GraphScope::Custom("urn:scope:legacy-receipt".into())
    );
    assert_eq!(
        first.provenance_iri.as_deref(),
        Some("urn:author:legacy-receipt")
    );
    assert_eq!(first.basis.evidence, ["urn:evidence:legacy-receipt"]);
    assert_eq!(
        snapshot.edges[0].retained_assertions()[1]
            .withdrawn
            .as_ref()
            .and_then(|withdrawal| withdrawal.at_ms),
        Some(31)
    );

    let canonical_snapshot = Graph::new().to_snapshot();
    let canonical_value = serde_json::to_value(canonical_snapshot).unwrap();
    assert_eq!(canonical_value["assertion_schema"], 1);
}
