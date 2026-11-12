// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

fn cases() -> Vec<PersistedSemanticStatement> {
    let plain = PersistedSemanticStatement {
        statement_id: "held\nassertion".into(),
        predicate: "urn:predicate:held".into(),
        recognized_sub_kind: None,
        label: None,
        graph_scope: GraphScope::Default,
        provenance_iri: Some("https://author.test/".into()),
        asserted_at_ms: None,
        first_asserted_ms: None,
        withdrawn: None,
        basis: Default::default(),
    };
    let mut legacy = plain.clone();
    legacy.provenance_iri = None;
    let mut marker = plain.clone();
    marker.provenance_iri = Some(crate::model::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI.into());
    let mut rich = plain.clone();
    rich.statement_id = String::new();
    rich.recognized_sub_kind = Some(PersistedSemanticSubKind::Cites);
    rich.label = Some(String::new());
    rich.graph_scope = GraphScope::Custom("urn:graph:held".into());
    rich.provenance_iri = Some(String::new());
    rich.asserted_at_ms = Some(0);
    rich.first_asserted_ms = Some(0);
    rich.basis.note = Some(String::new());
    vec![plain, legacy, marker, rich]
}

#[test]
fn statement_json_omits_absent_metadata_and_preserves_explicit_values() {
    let cases = cases();
    let plain = serde_json::to_value(&cases[0]).unwrap();
    assert_eq!(plain.as_object().unwrap().len(), 3);
    assert_eq!(plain["statement_id"], "held\nassertion");
    assert_eq!(plain["provenance_iri"], "https://author.test/");
    for absent in [
        "recognized_sub_kind",
        "label",
        "graph_scope",
        "asserted_at_ms",
    ] {
        assert!(plain.get(absent).is_none());
    }
    let legacy = serde_json::to_value(&cases[1]).unwrap();
    assert!(legacy.get("provenance_iri").is_none());
    assert_eq!(
        serde_json::from_value::<PersistedSemanticStatement>(legacy).unwrap(),
        cases[1]
    );
    let marker = serde_json::to_value(&cases[2]).unwrap();
    assert_eq!(
        marker["provenance_iri"],
        crate::model::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI
    );
    let rich = serde_json::to_value(&cases[3]).unwrap();
    assert_eq!(rich.as_object().unwrap().len(), 9);
    assert_eq!(rich["statement_id"], "");
    assert_eq!(rich["label"], "");
    assert_eq!(rich["provenance_iri"], "");
    assert_eq!(rich["asserted_at_ms"], 0);
    assert_eq!(rich["first_asserted_ms"], 0);
    for value in cases {
        let bytes = serde_json::to_vec(&value).unwrap();
        assert_eq!(
            serde_json::from_slice::<PersistedSemanticStatement>(&bytes).unwrap(),
            value
        );
    }
}

#[test]
fn statement_binary_current_shape_round_trips() {
    for value in cases() {
        let bytes = postcard::to_allocvec(&value).unwrap();
        assert_eq!(
            postcard::from_bytes::<PersistedSemanticStatement>(&bytes).unwrap(),
            value
        );
        let archive = rkyv::to_bytes::<rkyv::rancor::Error>(&value).unwrap();
        assert_eq!(
            rkyv::from_bytes::<PersistedSemanticStatement, rkyv::rancor::Error>(&archive).unwrap(),
            value
        );
    }
}

#[test]
fn legacy_postcard_sequence_is_not_a_supported_store_format() {
    #[derive(serde::Serialize)]
    struct Legacy {
        statement_id: String,
        predicate: String,
        recognized_sub_kind: Option<PersistedSemanticSubKind>,
        label: Option<String>,
        graph_scope: GraphScope,
        provenance_iri: Option<String>,
        asserted_at_ms: Option<u64>,
    }

    let legacy = Legacy {
        statement_id: "legacy-sequence".into(),
        predicate: "urn:p".into(),
        recognized_sub_kind: None,
        label: None,
        graph_scope: GraphScope::Default,
        provenance_iri: Some("urn:source".into()),
        asserted_at_ms: Some(42),
    };
    let bytes = postcard::to_allocvec(&legacy).unwrap();
    assert!(postcard::from_bytes::<PersistedSemanticStatement>(&bytes).is_err());
}

#[test]
fn legacy_time_falls_back_only_when_first_time_is_absent() {
    let legacy = serde_json::json!({
        "statement_id": "legacy",
        "predicate": "urn:p",
        "asserted_at_ms": 42
    });
    let decoded: PersistedSemanticStatement = serde_json::from_value(legacy).unwrap();
    assert_eq!(decoded.first_asserted_ms, Some(42));
    assert_eq!(decoded.asserted_at_ms, Some(42));

    let explicit_unknown = serde_json::json!({
        "statement_id": "unknown-first",
        "predicate": "urn:p",
        "first_asserted_ms": null,
        "asserted_at_ms": 42
    });
    let decoded: PersistedSemanticStatement = serde_json::from_value(explicit_unknown).unwrap();
    assert_eq!(decoded.first_asserted_ms, None);
    assert_eq!(decoded.asserted_at_ms, Some(42));
    let encoded = serde_json::to_value(decoded).unwrap();
    assert!(encoded.get("first_asserted_ms").is_some());
    assert!(encoded["first_asserted_ms"].is_null());
}
