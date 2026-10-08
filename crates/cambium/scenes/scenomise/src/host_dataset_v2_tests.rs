// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use serde_json::json;

fn source() -> Value {
    json!({ "authority": "host", "domain": "example.graph", "resource": "graph:1" })
}

fn occurrence(id: &str, label: &str) -> Value {
    json!({
        "occurrence_id": id,
        "source": { "adapter": "host.graph", "id": id },
        "values": {
            "occurrence_id": { "kind": "text", "value": id },
            "label": { "kind": "text", "value": label },
        },
    })
}

fn relationship(id: &str, from: &str, to: &str, revision: &str) -> Value {
    json!({
        "id": id,
        "from_occurrence": from,
        "to_occurrence": to,
        "kind": "cites",
        "label": format!("{from} cites {to}"),
        "explanation": "The host's source records this citation.",
        "provenance": {
            "source": source(),
            "source_revision": revision,
            "method": "host.citations",
            "method_version": 1,
            "provider": "host exporter",
            "evidence": [{ "adapter": "host.graph", "id": from }],
        },
    })
}

fn revision(
    sequence: u64,
    revision: &str,
    occurrences: Vec<Value>,
    relationships: Vec<Value>,
) -> Value {
    json!({
        "sequence": sequence,
        "revision": revision,
        "dataset": {
            "source": source(),
            "revision": revision,
            "fields": { "occurrence_id": "text", "label": "text" },
            "occurrences": occurrences,
        },
        "relationships": relationships,
    })
}

/// Two revisions: b is renamed, c is removed, d is added, and a's citation
/// of c becomes a citation of d.
fn history() -> Value {
    json!({
        "schema": HOST_DATASET_SCHEMA_V2,
        "revisions": [
            revision(
                1,
                "r1",
                vec![occurrence("a", "Alpha"), occurrence("b", "Beta"), occurrence("c", "Gamma")],
                vec![relationship("rel:a-c", "a", "c", "r1")],
            ),
            revision(
                2,
                "r2",
                vec![occurrence("a", "Alpha"), occurrence("b", "Beta two"), occurrence("d", "Delta")],
                vec![relationship("rel:a-d", "a", "d", "r2")],
            ),
        ],
        "compared_fields": ["label"],
    })
}

fn parse(value: &Value) -> Result<HostDatasetV2, HostDatasetError> {
    parse_host_history(&value.to_string())
}

#[test]
fn accepts_a_history_and_round_trips_it() {
    let parsed = parse(&history()).unwrap();
    assert_eq!(parsed.schema, HOST_DATASET_SCHEMA_V2);
    assert_eq!(parsed.revisions.len(), 2);
    assert_eq!(parsed.current().revision.as_str(), "r2");
    assert_eq!(parsed.compared_fields, Some(vec!["label".to_owned()]));
    let again = parse_host_history(&serde_json::to_string(&parsed).unwrap()).unwrap();
    assert_eq!(again, parsed);
    // Without a declaration, every field is compared.
    let mut value = history();
    value.as_object_mut().unwrap().remove("compared_fields");
    assert_eq!(parse(&value).unwrap().compared_fields, None);
}

#[test]
fn the_present_of_a_history_reads_as_v1() {
    let parsed = parse(&history()).unwrap();
    let present = parsed.current_v1();
    assert_eq!(present.dataset, parsed.current().dataset);
    assert_eq!(present.revisions.len(), 2);
    let checked = parse_host_dataset(&serde_json::to_string(&present).unwrap()).unwrap();
    assert_eq!(checked, present);
}

#[test]
fn a_v1_envelope_is_the_one_revision_history() {
    let current = &history()["revisions"][1];
    let v1 = json!({
        "schema": HOST_DATASET_SCHEMA_V1,
        "dataset": current["dataset"],
        "relationships": current["relationships"],
        "revisions": [
            { "sequence": 4, "revision": "r1" },
            { "sequence": 7, "revision": "r2" },
        ],
    });
    let lifted = parse(&v1).unwrap();
    assert_eq!(lifted.schema, HOST_DATASET_SCHEMA_V2);
    assert_eq!(lifted.revisions.len(), 1);
    assert_eq!(lifted.revisions[0].sequence, 7);
    assert_eq!(lifted.current().revision.as_str(), "r2");
    assert_eq!(
        lifted,
        HostDatasetV2::from(parse_host_dataset(&v1.to_string()).unwrap())
    );
    // A v1 envelope is still checked as v1.
    let mut stale = v1.clone();
    stale["revisions"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "sequence": 8, "revision": "r3" }));
    assert!(matches!(
        parse(&stale),
        Err(HostDatasetError::StaleRevision { .. })
    ));
    // And the v1 reader does not read a history.
    assert_eq!(
        parse_host_dataset(&history().to_string()).unwrap_err(),
        HostDatasetError::UnknownSchema(Some(HOST_DATASET_SCHEMA_V2.into()))
    );
}

#[test]
fn refuses_unknown_keys_at_any_depth() {
    for (pointer, key, path) in [
        ("", "facets", ""),
        ("/revisions/1", "date", "revisions[1]"),
        ("/revisions/0/dataset", "generation", "revisions[0].dataset"),
        (
            "/revisions/0/dataset/source",
            "owner",
            "revisions[0].dataset.source",
        ),
        (
            "/revisions/1/dataset/occurrences/2",
            "x",
            "revisions[1].dataset.occurrences[2]",
        ),
        (
            "/revisions/1/dataset/occurrences/0/source",
            "kind",
            "revisions[1].dataset.occurrences[0].source",
        ),
        (
            "/revisions/0/dataset/occurrences/1/values/label",
            "unit",
            "revisions[0].dataset.occurrences[1].values.label",
        ),
        (
            "/revisions/1/relationships/0",
            "weight",
            "revisions[1].relationships[0]",
        ),
        (
            "/revisions/1/relationships/0/provenance/source",
            "owner",
            "revisions[1].relationships[0].provenance.source",
        ),
        (
            "/revisions/0/relationships/0/provenance/evidence/0",
            "note",
            "revisions[0].relationships[0].provenance.evidence[0]",
        ),
    ] {
        let mut value = history();
        value
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert(key.into(), json!("extra"));
        assert_eq!(
            parse(&value).unwrap_err(),
            HostDatasetError::UnknownKey {
                path: path.into(),
                key: key.into()
            },
            "{pointer}"
        );
    }
}

#[test]
fn the_history_key_lists_match_the_serialized_types() {
    let value = serde_json::to_value(parse(&history()).unwrap()).unwrap();
    let keys = |value: &Value| {
        let mut keys: Vec<_> = value.as_object().unwrap().keys().cloned().collect();
        keys.sort();
        keys
    };
    let sorted = |list: &[&str]| {
        let mut list: Vec<_> = list.iter().map(|key| key.to_string()).collect();
        list.sort();
        list
    };
    assert_eq!(keys(&value), sorted(HISTORY_KEYS));
    assert_eq!(
        keys(value.pointer("/revisions/0").unwrap()),
        sorted(HISTORY_REVISION_KEYS)
    );
}

#[test]
fn refuses_another_schema_or_none() {
    let mut value = history();
    value["schema"] = json!("scenomise.host-dataset/v3");
    assert_eq!(
        parse(&value).unwrap_err(),
        HostDatasetError::UnknownHistorySchema(Some("scenomise.host-dataset/v3".into()))
    );
    value.as_object_mut().unwrap().remove("schema");
    assert_eq!(
        parse(&value).unwrap_err(),
        HostDatasetError::UnknownHistorySchema(None)
    );
}

#[test]
fn refuses_a_stale_dataset_filed_under_a_newer_revision() {
    let mut value = history();
    value["revisions"][1]["dataset"]["revision"] = json!("r1");
    assert_eq!(
        parse(&value).unwrap_err(),
        HostDatasetError::RevisionMismatch {
            index: 1,
            revision: "r2".into(),
            dataset: "r1".into()
        }
    );
}

#[test]
fn refuses_an_empty_disordered_repeated_or_blank_history() {
    let mut value = history();
    value["revisions"] = json!([]);
    assert_eq!(parse(&value).unwrap_err(), HostDatasetError::NoRevisions);

    let mut value = history();
    value["revisions"][1]["sequence"] = json!(1);
    assert_eq!(
        parse(&value).unwrap_err(),
        HostDatasetError::RevisionOutOfOrder { index: 1 }
    );

    let mut value = history();
    let first = value["revisions"][0].clone();
    value["revisions"] = json!([
        first,
        revision(2, "r1", vec![occurrence("a", "Alpha")], vec![])
    ]);
    assert_eq!(
        parse(&value).unwrap_err(),
        HostDatasetError::DuplicateRevision {
            revision: "r1".into()
        }
    );

    let mut value = history();
    value["revisions"][0]["revision"] = json!(" ");
    value["revisions"][0]["dataset"]["revision"] = json!(" ");
    assert_eq!(
        parse(&value).unwrap_err(),
        HostDatasetError::BlankRevision { index: 0 }
    );
}

#[test]
fn refuses_bad_relation_endpoints_in_any_revision() {
    let mut value = history();
    value["revisions"][0]["relationships"][0]["to_occurrence"] = json!("d");
    let HostDatasetError::Relationships(issues) = parse(&value).unwrap_err() else {
        panic!("expected a relationship refusal");
    };
    assert_eq!(issues.len(), 1);
    assert_eq!(
        issues[0].field,
        "revisions[0].relationships.rel:a-c.endpoints"
    );

    // Provenance at another revision of the same history is refused too.
    let mut value = history();
    value["revisions"][1]["relationships"][0]["provenance"]["source_revision"] = json!("r1");
    let HostDatasetError::Relationships(issues) = parse(&value).unwrap_err() else {
        panic!("expected a relationship refusal");
    };
    assert_eq!(
        issues[0].field,
        "revisions[1].relationships.rel:a-d.provenance.source_revision"
    );
}

#[test]
fn refuses_a_history_of_another_source() {
    let mut value = history();
    value["revisions"][1]["dataset"]["source"]["resource"] = json!("graph:2");
    for relationship in value["revisions"][1]["relationships"]
        .as_array_mut()
        .unwrap()
    {
        relationship["provenance"]["source"]["resource"] = json!("graph:2");
    }
    assert_eq!(
        parse(&value).unwrap_err(),
        HostDatasetError::SourceChanged { index: 1 }
    );
}

#[test]
fn refuses_empty_repeated_or_undeclared_compared_fields() {
    for (fields, problem) in [
        (json!([]), "are empty"),
        (json!(["label", "label"]), "name label twice"),
        (json!(["colour"]), "name colour, which no revision declares"),
    ] {
        let mut value = history();
        value["compared_fields"] = fields;
        assert_eq!(
            parse(&value).unwrap_err(),
            HostDatasetError::ComparedFields(problem.into())
        );
    }
}

#[test]
fn refuses_oversized_and_malformed_histories() {
    let large = " ".repeat(MAX_HOST_DATASET_BYTES + 1);
    assert!(matches!(
        parse_host_history(&large),
        Err(HostDatasetError::TooLarge { .. })
    ));
    assert!(matches!(
        parse_host_history("{"),
        Err(HostDatasetError::Malformed(_))
    ));
    let mut value = history();
    value["revisions"][0]["sequence"] = json!("first");
    assert!(matches!(parse(&value), Err(HostDatasetError::Malformed(_))));
}
