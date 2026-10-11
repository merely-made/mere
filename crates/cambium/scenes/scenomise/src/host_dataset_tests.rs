// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::projection::ItemSizes;
use sceno::Size2;
use scenograph::{
    Appearance, Arrangement, Channel, Encoding, Interaction, Provenance, Reading, RevisionEvidence,
    SelectionMode,
};
use serde_json::json;

/// A host-shaped envelope: three occurrences, one disclosed relationship, and
/// two published revisions with the dataset at the second.
fn envelope() -> Value {
    let source = json!({ "authority": "host", "domain": "example.graph", "resource": "graph:1" });
    let occurrence = |id: &str, label: &str, x: f64, y: f64| {
        json!({
            "occurrence_id": id,
            "source": { "adapter": "host.graph", "id": id },
            "values": {
                "occurrence_id": { "kind": "text", "value": id },
                "label": { "kind": "text", "value": label },
                "x": { "kind": "number", "value": x },
                "y": { "kind": "number", "value": y },
            },
        })
    };
    json!({
        "schema": HOST_DATASET_SCHEMA_V1,
        "dataset": {
            "source": source,
            "revision": "r2",
            "fields": { "occurrence_id": "text", "label": "text", "x": "number", "y": "number" },
            "occurrences": [
                occurrence("a", "Alpha", 0.0, 0.0),
                occurrence("b", "Beta", 3.0, 1.0),
                occurrence("c", "Gamma", 1.0, 2.0),
            ],
        },
        "relationships": [{
            "id": "rel:a-b",
            "from_occurrence": "a",
            "to_occurrence": "b",
            "kind": "cites",
            "label": "Alpha cites Beta",
            "explanation": "The host's source records this citation.",
            "provenance": {
                "source": source,
                "source_revision": "r2",
                "method": "host.citations",
                "method_version": 1,
                "provider": "host exporter",
                "evidence": [{ "adapter": "host.graph", "id": "a" }],
            },
        }],
        "revisions": [
            { "sequence": 1, "revision": "r1" },
            { "sequence": 2, "revision": "r2" },
        ],
    })
}

fn parse(value: &Value) -> Result<HostDatasetV1, HostDatasetError> {
    parse_host_dataset(&value.to_string())
}

#[test]
fn accepts_a_valid_envelope() {
    let parsed = parse(&envelope()).unwrap();
    assert_eq!(parsed.schema, HOST_DATASET_SCHEMA_V1);
    assert_eq!(parsed.dataset.occurrences.len(), 3);
    assert_eq!(parsed.relationships[0].from_occurrence, "a");
    assert_eq!(parsed.revisions.len(), 2);
    assert_eq!(parsed.dataset.revision.as_str(), "r2");
    // The envelope round-trips through its own serde.
    let again = parse_host_dataset(&serde_json::to_string(&parsed).unwrap()).unwrap();
    assert_eq!(again, parsed);
}

#[test]
fn accepts_an_envelope_without_relationships() {
    let mut value = envelope();
    value["relationships"] = json!([]);
    assert!(parse(&value).unwrap().relationships.is_empty());
}

#[test]
fn refuses_an_unknown_top_level_key() {
    let mut value = envelope();
    value["facets"] = json!([]);
    assert_eq!(
        parse(&value).unwrap_err(),
        HostDatasetError::UnknownKey {
            path: String::new(),
            key: "facets".into()
        }
    );
}

#[test]
fn refuses_unknown_keys_inside_the_dataset() {
    for (pointer, key, path) in [
        ("/dataset", "generation", "dataset"),
        ("/dataset/source", "owner", "dataset.source"),
        ("/dataset/occurrences/1", "x", "dataset.occurrences[1]"),
        (
            "/dataset/occurrences/0/source",
            "kind",
            "dataset.occurrences[0].source",
        ),
        (
            "/dataset/occurrences/2/values/label",
            "unit",
            "dataset.occurrences[2].values.label",
        ),
        (
            "/relationships/0/provenance/source",
            "owner",
            "relationships[0].provenance.source",
        ),
        (
            "/relationships/0/provenance/evidence/0",
            "note",
            "relationships[0].provenance.evidence[0]",
        ),
        ("/revisions/0", "date", "revisions[0]"),
    ] {
        let mut value = envelope();
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
fn the_known_key_lists_match_the_serialized_types() {
    let parsed = parse(&envelope()).unwrap();
    let value = serde_json::to_value(&parsed).unwrap();
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
    for (pointer, list) in [
        ("", ENVELOPE_KEYS),
        ("/dataset", DATASET_KEYS),
        ("/dataset/source", BINDING_KEYS),
        ("/dataset/occurrences/0", OCCURRENCE_KEYS),
        ("/dataset/occurrences/0/source", SOURCE_REF_KEYS),
        ("/dataset/occurrences/0/values/label", VALUE_KEYS),
        ("/relationships/0", RELATIONSHIP_KEYS),
        ("/relationships/0/provenance", PROVENANCE_KEYS),
        ("/relationships/0/provenance/source", BINDING_KEYS),
        ("/relationships/0/provenance/evidence/0", SOURCE_REF_KEYS),
        ("/revisions/0", REVISION_KEYS),
    ] {
        assert_eq!(
            keys(value.pointer(pointer).unwrap()),
            sorted(list),
            "{pointer}"
        );
    }
}

#[test]
fn refuses_another_schema_or_none() {
    let mut value = envelope();
    value["schema"] = json!("scenomise.host-dataset/v2");
    assert_eq!(
        parse(&value).unwrap_err(),
        HostDatasetError::UnknownSchema(Some("scenomise.host-dataset/v2".into()))
    );
    value.as_object_mut().unwrap().remove("schema");
    assert_eq!(
        parse(&value).unwrap_err(),
        HostDatasetError::UnknownSchema(None)
    );
}

#[test]
fn refuses_a_dataset_behind_its_latest_revision() {
    let mut value = envelope();
    value["revisions"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "sequence": 3, "revision": "r3" }));
    assert_eq!(
        parse(&value).unwrap_err(),
        HostDatasetError::StaleRevision {
            dataset: "r2".into(),
            latest: "r3".into()
        }
    );
}

#[test]
fn refuses_relationship_provenance_at_another_revision() {
    let mut value = envelope();
    value["relationships"][0]["provenance"]["source_revision"] = json!("r1");
    let HostDatasetError::Relationships(issues) = parse(&value).unwrap_err() else {
        panic!("expected a relationship refusal");
    };
    assert_eq!(issues.len(), 1);
    assert_eq!(
        issues[0].field,
        "relationships.rel:a-b.provenance.source_revision"
    );
}

#[test]
fn refuses_an_empty_disordered_or_repeated_revision_sequence() {
    let mut value = envelope();
    value["revisions"] = json!([]);
    assert_eq!(parse(&value).unwrap_err(), HostDatasetError::NoRevisions);

    value["revisions"] = json!([
        { "sequence": 2, "revision": "r1" },
        { "sequence": 2, "revision": "r2" },
    ]);
    assert_eq!(
        parse(&value).unwrap_err(),
        HostDatasetError::RevisionOutOfOrder { index: 1 }
    );

    value["revisions"] = json!([
        { "sequence": 1, "revision": "r2" },
        { "sequence": 2, "revision": "r2" },
    ]);
    assert_eq!(
        parse(&value).unwrap_err(),
        HostDatasetError::DuplicateRevision {
            revision: "r2".into()
        }
    );

    value["revisions"] = json!([
        { "sequence": 1, "revision": " " },
        { "sequence": 2, "revision": "r2" },
    ]);
    assert_eq!(
        parse(&value).unwrap_err(),
        HostDatasetError::BlankRevision { index: 0 }
    );
}

#[test]
fn refuses_a_relationship_with_a_bad_endpoint() {
    for (end, id) in [("to_occurrence", "missing"), ("from_occurrence", "b")] {
        let mut value = envelope();
        value["relationships"][0][end] = json!(id);
        let HostDatasetError::Relationships(issues) = parse(&value).unwrap_err() else {
            panic!("expected a relationship refusal");
        };
        assert_eq!(issues.len(), 1, "{end}");
        assert_eq!(issues[0].field, "relationships.rel:a-b.endpoints");
    }
}

#[test]
fn refuses_oversized_and_malformed_input() {
    let large = " ".repeat(MAX_HOST_DATASET_BYTES + 1);
    assert!(matches!(
        parse_host_dataset(&large),
        Err(HostDatasetError::TooLarge { .. })
    ));
    assert!(matches!(
        parse_host_dataset("{"),
        Err(HostDatasetError::Malformed(_))
    ));
    let mut value = envelope();
    value["dataset"]["fields"]["label"] = json!("colour");
    assert!(matches!(parse(&value), Err(HostDatasetError::Malformed(_))));
}

fn compiler() -> ProjectionCompiler {
    ProjectionCompiler::new(ItemSizes {
        card: Size2::new(164.0, 68.0),
    })
}

/// A host's own definition over the envelope's dataset.
fn definition(data: &HostDatasetV1, kind: &str, x: &str) -> ProjectionDefinition {
    ProjectionDefinition {
        dynamics: None,
        version: 1,
        id: "host-view".into(),
        label: "Host view".into(),
        source: data.dataset.source.clone(),
        reading: Reading {
            kind: "nodes".into(),
            key: "occurrence_id".into(),
            value: None,
        },
        encoding: Encoding {
            x: Channel::Field(x.into()),
            y: Channel::Field("y".into()),
            color: None,
            label: Some(Channel::Field("label".into())),
        },
        arrangement: Arrangement {
            kind: kind.into(),
            direction: "coordinates".into(),
            spacing: 16,
            options: Default::default(),
        },
        interaction: Interaction {
            selection: SelectionMode::Single,
            pan: false,
            zoom: false,
        },
        appearance: Appearance {
            realization: "canvas".into(),
            title: "Host view".into(),
            theme: "slate".into(),
        },
        provenance: Provenance {
            author: "host".into(),
            source_revision: Some(data.dataset.revision.clone()),
            revision_evidence: RevisionEvidence::PublicGeneration,
            note: String::new(),
        },
    }
}

/// Every routed relation runs between the placed positions of its endpoints.
fn assert_routed(compiled: &HostProjection) {
    let scene = &compiled.projection.scene;
    assert_eq!(scene.relations.len(), compiled.relationships.len());
    for (relation, relationship) in scene.relations.iter().zip(&compiled.relationships) {
        let instance = |occurrence: &str| compiled.projection.instance_by_occurrence[occurrence];
        assert_eq!(
            relation.from,
            instance(&relationship.disclosure.from_occurrence)
        );
        assert_eq!(
            relation.to,
            instance(&relationship.disclosure.to_occurrence)
        );
        assert_eq!(
            relation.points,
            vec![
                scene.items[relation.from.0 as usize].transform.translate,
                scene.items[relation.to.0 as usize].transform.translate,
            ]
        );
        assert_eq!(
            relation.kind.as_deref(),
            Some(relationship.disclosure.kind.as_str())
        );
    }
}

#[test]
fn relationships_become_scene_relations() {
    let data = parse(&envelope()).unwrap();
    let compiled = data
        .compile(&compiler(), &definition(&data, "spiral", "occurrence_id"))
        .unwrap();
    assert_eq!(compiled.projection.scene.relations.len(), 1);
    assert_eq!(compiled.relationships[0].disclosure, data.relationships[0]);
    assert_routed(&compiled);
}

#[test]
fn relations_follow_the_chosen_arrangement() {
    let data = parse(&envelope()).unwrap();
    let mut points = Vec::new();
    for (kind, x) in [
        ("spiral", "occurrence_id"),
        ("grid", "x"),
        ("geographic", "x"),
    ] {
        let compiled = data
            .compile(&compiler(), &definition(&data, kind, x))
            .unwrap();
        assert_routed(&compiled);
        points.push(compiled.projection.scene.relations[0].points.clone());
    }
    assert_ne!(points[0], points[1]);
    assert_ne!(points[1], points[2]);
}

#[test]
fn a_dataset_without_relationships_compiles_as_it_would_bare() {
    let mut value = envelope();
    value["relationships"] = json!([]);
    let data = parse(&value).unwrap();
    let recipe = definition(&data, "grid", "x");
    let compiled = data.compile(&compiler(), &recipe).unwrap();
    assert!(compiled.relationships.is_empty());
    assert!(compiled.projection.scene.relations.is_empty());
    assert_eq!(
        compiled.projection.scene,
        compiler().compile(&recipe, &data.dataset).unwrap().scene
    );
}

#[test]
fn a_reused_placement_routes_its_relations_once() {
    let data = parse(&envelope()).unwrap();
    let recipe = definition(&data, "grid", "x");
    let first = data.compile(&compiler(), &recipe).unwrap();
    let again = data
        .refresh(&compiler(), &first.projection, &recipe)
        .unwrap();
    assert!(again.projection.placement_reused);
    assert_eq!(
        again.projection.scene.relations,
        first.projection.scene.relations
    );
    let moved = data
        .refresh(
            &compiler(),
            &first.projection,
            &definition(&data, "spiral", "occurrence_id"),
        )
        .unwrap();
    assert!(!moved.projection.placement_reused);
    assert_eq!(moved.projection.scene.relations.len(), 1);
    assert_routed(&moved);
}

#[test]
fn refuses_to_route_onto_a_projection_of_another_revision() {
    let data = parse(&envelope()).unwrap();
    let mut other = data.dataset.clone();
    other.revision = "r3".into();
    let mut recipe = definition(&data, "grid", "x");
    recipe.provenance.source_revision = Some(other.revision.clone());
    let mut projection = compiler().compile(&recipe, &other).unwrap();
    let issues =
        route_relationships(&mut projection, &data.dataset, &data.relationships).unwrap_err();
    assert_eq!(issues[0].field, "projection");
}
