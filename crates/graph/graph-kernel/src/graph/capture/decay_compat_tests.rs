// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::{CapturedDelta, EdgeAssertion};
use crate::graph::{
    ArrangementSubKind, ContainmentSubKind, Graph, ImportedSubKind, ProvenanceSubKind,
    SemanticSubKind, replay_captured_deltas_onto,
};
use uuid::Uuid;

const LEGACY_JSON: &[u8] = include_bytes!("fixtures/captured-with-decay.json");
const LEGACY_POSTCARD: &[u8] = include_bytes!("fixtures/captured-with-decay.postcard");
const LEGACY_RKYV: &[u8] = include_bytes!("fixtures/captured-with-decay.rkyv");

fn assert_cleaned_fixture(deltas: &[CapturedDelta]) {
    assert_eq!(deltas.len(), 9);
    for delta in &deltas[..4] {
        assert!(matches!(
            delta,
            CapturedDelta::ReplayAssertRelationByIds {
                assertion: EdgeAssertion::Semantic {
                    sub_kind: SemanticSubKind::Cites,
                    label: Some(label),
                },
                ..
            } if label == "historical reason"
        ));
    }
    assert!(matches!(
        &deltas[4],
        CapturedDelta::ReplayAssertRelationByIds {
            assertion: EdgeAssertion::Containment {
                sub_kind: ContainmentSubKind::UrlPath,
            },
            ..
        }
    ));
    assert!(matches!(
        &deltas[5],
        CapturedDelta::ReplayAssertRelationByIds {
            assertion: EdgeAssertion::Arrangement {
                sub_kind: ArrangementSubKind::FrameMember,
            },
            ..
        }
    ));
    assert!(matches!(
        &deltas[6],
        CapturedDelta::ReplayAssertRelationByIds {
            assertion: EdgeAssertion::Imported {
                sub_kind: ImportedSubKind::BookmarkFolder,
            },
            ..
        }
    ));
    assert!(matches!(
        &deltas[7],
        CapturedDelta::ReplayAssertRelationByIds {
            assertion: EdgeAssertion::Provenance {
                sub_kind: ProvenanceSubKind::ClippedFrom,
            },
            ..
        }
    ));
    assert!(matches!(
        &deltas[8],
        CapturedDelta::ReplaySetNodeTitleById { title, .. }
            if title == "sentinel after old assertion fields"
    ));
}

#[test]
fn baseline_json_postcard_and_rkyv_captured_deltas_decode_without_decay() {
    let json: Vec<CapturedDelta> = serde_json::from_slice(LEGACY_JSON).expect("legacy JSON");
    let postcard: Vec<CapturedDelta> =
        postcard::from_bytes(LEGACY_POSTCARD).expect("legacy postcard");
    let mut aligned = rkyv::util::AlignedVec::<16>::new();
    aligned.extend_from_slice(LEGACY_RKYV);
    let archived: Vec<CapturedDelta> =
        rkyv::from_bytes::<Vec<CapturedDelta>, rkyv::rancor::Error>(&aligned)
            .expect("legacy rkyv archive");

    assert_cleaned_fixture(&json);
    assert_cleaned_fixture(&postcard);
    assert_cleaned_fixture(&archived);
    assert_eq!(postcard, json);
    assert_eq!(archived, json);

    let rewritten = postcard::to_allocvec(&json).expect("rewrite positional journal");
    assert_eq!(
        postcard::from_bytes::<Vec<CapturedDelta>>(&rewritten).unwrap(),
        json
    );
    let rewritten = rkyv::to_bytes::<rkyv::rancor::Error>(&json).expect("rewrite archive");
    assert_eq!(
        rkyv::from_bytes::<Vec<CapturedDelta>, rkyv::rancor::Error>(&rewritten).unwrap(),
        json
    );
}

#[test]
fn legacy_json_assertion_replays_and_new_captured_json_omits_decay() {
    let legacy: Vec<CapturedDelta> = serde_json::from_slice(LEGACY_JSON).expect("legacy JSON");
    let from_id = Uuid::from_u128(1);
    let to_id = Uuid::from_u128(2);
    let mut graph = Graph::new();
    let from = graph.add_node_with_id(from_id, "https://from.test/".into(), Default::default());
    graph.add_node_with_id(to_id, "https://to.test/".into(), Default::default());

    let relation_rows = legacy.iter().take(4).cloned();
    let with_sentinel = relation_rows.chain(std::iter::once(legacy[8].clone()));
    replay_captured_deltas_onto(&mut graph, with_sentinel);
    let key = graph
        .find_edge_key(from, graph.get_node_key_by_id(to_id).unwrap())
        .unwrap();
    let statements = graph.get_edge(key).unwrap().semantic_statements();
    assert_eq!(
        statements.len(),
        1,
        "legacy decay values do not split claims"
    );
    assert_eq!(statements[0].label.as_deref(), Some("historical reason"));
    assert_eq!(
        graph.get_node(from).unwrap().title,
        "sentinel after old assertion fields",
        "replay continues past the legacy assertion shape"
    );

    let value = serde_json::to_value(&legacy[0]).expect("serialize current captured JSON");
    let assertion = &value["ReplayAssertRelationByIds"]["assertion"]["Semantic"];
    assert_eq!(assertion["sub_kind"], "Cites");
    assert_eq!(assertion["label"], "historical reason");
    assert!(assertion.get("decay_progress").is_none());
}

#[test]
fn direct_edge_assertion_json_rejects_the_retired_field() {
    let legacy_assertion = serde_json::json!({
        "Semantic": {
            "sub_kind": "Cites",
            "label": "historical reason",
            "decay_progress": 0.25
        }
    });
    assert!(serde_json::from_value::<EdgeAssertion>(legacy_assertion).is_err());
}
