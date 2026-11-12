// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

fn cases() -> Vec<NodeProperty> {
    let mut plain = NodeProperty::new("urn:predicate:value".into(), "held value".into());
    plain.statement_id = "held\nproperty".into();
    plain.provenance_iri = Some("https://author.test/".into());
    let mut legacy = plain.clone();
    legacy.provenance_iri = None;
    let mut marker = plain.clone();
    marker.provenance_iri = Some(crate::model::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI.into());
    let mut rich = plain.clone();
    rich.statement_id = String::new();
    rich.datatype = Some("http://www.w3.org/1999/02/22-rdf-syntax-ns#langString".into());
    rich.lang = Some("fr".into());
    rich.graph_scope = GraphScope::Custom("urn:graph:held".into());
    rich.provenance_iri = Some(String::new());
    rich.asserted_at_ms = Some(0);
    let mut explicit = plain.clone();
    explicit.datatype = Some("http://www.w3.org/2001/XMLSchema#string".into());
    explicit.lang = Some(String::new());
    vec![plain, legacy, marker, rich, explicit]
}

#[test]
fn property_json_omits_absent_metadata_and_preserves_explicit_values() {
    let cases = cases();
    let plain = serde_json::to_value(&cases[0]).unwrap();
    assert_eq!(plain.as_object().unwrap().len(), 4);
    assert_eq!(plain["statement_id"], "held\nproperty");
    assert_eq!(plain["provenance_iri"], "https://author.test/");
    for absent in ["datatype", "lang", "graph_scope", "asserted_at_ms"] {
        assert!(plain.get(absent).is_none());
    }
    let legacy = serde_json::to_value(&cases[1]).unwrap();
    assert!(legacy.get("provenance_iri").is_none());
    let decoded: NodeProperty = serde_json::from_value(legacy).unwrap();
    assert_eq!(decoded, cases[1], "missing metadata keeps legacy defaults");
    let marker = serde_json::to_value(&cases[2]).unwrap();
    assert_eq!(
        marker["provenance_iri"],
        crate::model::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI
    );
    let rich = serde_json::to_value(&cases[3]).unwrap();
    assert_eq!(rich.as_object().unwrap().len(), 8);
    assert_eq!(rich["statement_id"], "");
    assert_eq!(rich["provenance_iri"], "");
    assert_eq!(rich["asserted_at_ms"], 0);
    assert_eq!(rich["lang"], "fr");
    let explicit = serde_json::to_value(&cases[4]).unwrap();
    assert_eq!(
        explicit["datatype"],
        "http://www.w3.org/2001/XMLSchema#string"
    );
    assert_eq!(explicit["lang"], "");
    for value in cases {
        let bytes = serde_json::to_vec(&value).unwrap();
        assert_eq!(
            serde_json::from_slice::<NodeProperty>(&bytes).unwrap(),
            value
        );
    }
}

#[test]
fn property_binary_bytes_match_the_complete_legacy_shape() {
    #[derive(serde::Serialize)]
    struct Complete<'a> {
        statement_id: &'a String,
        predicate: &'a String,
        value: &'a String,
        datatype: &'a Option<String>,
        lang: &'a Option<String>,
        graph_scope: &'a GraphScope,
        provenance_iri: &'a Option<String>,
        asserted_at_ms: &'a Option<u64>,
    }
    for value in cases() {
        let old = Complete {
            statement_id: &value.statement_id,
            predicate: &value.predicate,
            value: &value.value,
            datatype: &value.datatype,
            lang: &value.lang,
            graph_scope: &value.graph_scope,
            provenance_iri: &value.provenance_iri,
            asserted_at_ms: &value.asserted_at_ms,
        };
        let bytes = postcard::to_allocvec(&value).unwrap();
        assert_eq!(bytes, postcard::to_allocvec(&old).unwrap());
        assert_eq!(postcard::from_bytes::<NodeProperty>(&bytes).unwrap(), value);
        let archive = rkyv::to_bytes::<rkyv::rancor::Error>(&value).unwrap();
        assert_eq!(
            rkyv::from_bytes::<NodeProperty, rkyv::rancor::Error>(&archive).unwrap(),
            value
        );
    }
}
