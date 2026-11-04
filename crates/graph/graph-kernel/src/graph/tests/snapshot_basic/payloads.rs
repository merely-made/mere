// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn nested_graph_survives_snapshot_roundtrip() {
    // The one-node containment ruling: a node bearing a graph (Node.nested)
    // keeps its world across save/load, and an unbearing node stays None.
    let mut graph = Graph::new();
    let bearer = graph.add_node("mere://denizen/ab12".to_string(), Point2D::new(0.0, 0.0));
    let plain = graph.add_node("https://a.test/".to_string(), Point2D::new(0.0, 0.0));
    let _ = apply::apply_graph_delta(
        &mut graph,
        apply::GraphDelta::SetNodeNested {
            key: bearer,
            nested: Some(muniment::LogId::new("denizens/trail-keeper")),
        },
    );

    let restored = Graph::from_snapshot(&graph.to_snapshot());

    let (rb, rnode) = restored.get_node_by_url("mere://denizen/ab12").unwrap();
    let _ = rb;
    assert_eq!(
        rnode.nested.as_ref().map(|log| log.as_str()),
        Some("denizens/trail-keeper")
    );
    let (_, rplain) = restored.get_node_by_url("https://a.test/").unwrap();
    assert_eq!(rplain.nested, None);
    // The plain node stayed unbearing in the ORIGINAL too (the setter targeted
    // one key, not the graph).
    assert_eq!(graph.get_node(plain).unwrap().nested, None);
}

#[test]
fn statement_bucket_survives_snapshot_roundtrip() {
    let mut graph = Graph::new();
    let a = graph.add_node("https://a.test/".to_string(), Point2D::new(0.0, 0.0));
    let b = graph.add_node("https://b.test/".to_string(), Point2D::new(0.0, 0.0));
    graph.assert_relation(a, b, cites()).unwrap();
    graph.assert_semantic_predicate_in_scope(
        a,
        b,
        "https://schema.org/citation".to_string(),
        crate::types::GraphScope::Source,
    );
    let edge_key = graph.projected_relations_between(a, b).next().unwrap().0;
    let payload = graph.get_relation_mut(edge_key).expect("payload");
    let statements = &mut payload.assertions.statements;
    statements[0].statement_id = "stmt-edge-1".to_string();
    statements[0].provenance_iri = Some("https://people.test/alice".to_string());
    statements[0].asserted_at_ms = Some(1_720_000_000_123);
    statements[1].statement_id = "stmt-edge-2".to_string();
    statements[1].provenance_iri = Some("https://people.test/bob".to_string());
    statements[1].asserted_at_ms = Some(1_720_000_100_456);

    let restored = Graph::from_snapshot(&graph.to_snapshot());

    let (ra, _) = restored.get_node_by_url("https://a.test/").unwrap();
    let (rb, _) = restored.get_node_by_url("https://b.test/").unwrap();
    let payload = resource_payload(&restored, ra, rb);
    assert_eq!(payload.semantic_statements().len(), 2);
    assert!(payload.semantic_statements().iter().any(|statement| {
        statement.recognized_sub_kind == Some(SemanticSubKind::Cites)
            && statement.predicate == "https://mere.computer/ns/rel#cites"
            && statement.graph_scope == crate::types::GraphScope::Default
            && statement.statement_id == "stmt-edge-1"
            && statement.provenance_iri.as_deref() == Some("https://people.test/alice")
            && statement.asserted_at_ms == Some(1_720_000_000_123)
    }));
    assert!(payload.semantic_statements().iter().any(|statement| {
        statement.recognized_sub_kind.is_none()
            && statement.predicate == "https://schema.org/citation"
            && statement.graph_scope == crate::types::GraphScope::Source
            && statement.statement_id == "stmt-edge-2"
            && statement.provenance_iri.as_deref() == Some("https://people.test/bob")
            && statement.asserted_at_ms == Some(1_720_000_100_456)
    }));
}

#[test]
fn node_properties_survive_snapshot_roundtrip() {
    let mut graph = Graph::new();
    let a = graph.add_node("https://a.test/".to_string(), Point2D::new(0.0, 0.0));
    let mut property = crate::types::NodeProperty::new(
        "https://schema.org/datePublished".to_string(),
        "2026-06-02".to_string(),
    )
    .with_graph_scope(crate::types::GraphScope::Source)
    .with_metadata(
        Some("https://people.test/alice".to_string()),
        Some(1_720_000_000_123),
    );
    property.statement_id = "stmt-property-1".to_string();
    property.datatype = Some("http://www.w3.org/2001/XMLSchema#date".to_string());
    graph.append_node_property(a, property);

    let facets = graph.facets().clone();
    let mut restored = Graph::from_snapshot(&graph.to_snapshot());
    restored.overlay_facets(facets).unwrap();

    let (key, _) = restored.get_node_by_url("https://a.test/").unwrap();
    let properties = restored.node_properties(key).unwrap();
    assert_eq!(properties.len(), 1);
    assert_eq!(properties[0].predicate, "https://schema.org/datePublished");
    assert_eq!(properties[0].value, "2026-06-02");
    assert_eq!(
        properties[0].datatype.as_deref(),
        Some("http://www.w3.org/2001/XMLSchema#date")
    );
    assert_eq!(properties[0].lang.as_deref(), None);
    assert_eq!(properties[0].graph_scope, crate::types::GraphScope::Source);
    assert_eq!(properties[0].statement_id, "stmt-property-1");
    assert_eq!(
        properties[0].provenance_iri.as_deref(),
        Some("https://people.test/alice")
    );
    assert_eq!(properties[0].asserted_at_ms, Some(1_720_000_000_123));
}

#[test]
fn legacy_node_metadata_columns_migrate_once_into_facets() {
    use std::time::UNIX_EPOCH;

    use crate::types::{
        ClassificationProvenance, ClassificationScheme, ClassificationStatus, FrameLayoutHint,
        NodeClassification, NodeDerivation, NodeImportProvenance, NodeProperty, SplitOrientation,
    };

    let mut graph = Graph::new();
    let key = graph.add_node("https://legacy.test".to_string(), Point2D::new(0.0, 0.0));
    let node_id = graph.get_node(key).unwrap().id;
    let mut snapshot = graph.to_snapshot();
    let legacy = snapshot.nodes.first_mut().unwrap();
    legacy.tag_presentation.ordered_tags = vec!["research".into()];
    legacy.import_provenance = vec![NodeImportProvenance {
        source_id: "bookmarks".into(),
        source_label: "Bookmarks".into(),
    }];
    legacy.is_pinned = true;
    legacy.classifications = vec![NodeClassification {
        scheme: ClassificationScheme::ContentKind,
        value: "article".into(),
        label: Some("Article".into()),
        confidence: 1.0,
        provenance: ClassificationProvenance::Imported,
        status: ClassificationStatus::Imported,
        primary: true,
    }];
    legacy.frame_layout_hints = vec![FrameLayoutHint::SplitHalf {
        first: "left".into(),
        second: "right".into(),
        orientation: SplitOrientation::Vertical,
    }];
    legacy.frame_split_offer_suppressed = true;
    legacy.properties = vec![NodeProperty::new(
        "https://schema.org/datePublished".into(),
        "2026-07-27".into(),
    )];
    legacy.derivations = vec![NodeDerivation {
        sub_kind: ProvenanceSubKind::CopiedFrom,
        source_node: "source-node".into(),
        source_graph: Some("source-graph".into()),
    }];
    legacy.session_state.as_mut().unwrap().last_visited_ms = Some(42);
    legacy.last_session_visited = 7;
    let expected_import_provenance = legacy.import_provenance.clone();
    let expected_classifications = legacy.classifications.clone();
    let expected_frame_layout_hints = legacy.frame_layout_hints.clone();
    let expected_properties = legacy.properties.clone();
    let expected_derivations = legacy.derivations.clone();

    let restored = Graph::from_snapshot(&snapshot);
    let restored_key = restored.get_node_key_by_id(node_id).unwrap();

    assert_eq!(restored.node_is_pinned(restored_key), Some(true));
    assert_eq!(
        restored
            .node_tag_presentation(restored_key)
            .unwrap()
            .ordered_tags,
        vec!["research"]
    );
    assert_eq!(
        restored.node_import_provenance(restored_key).unwrap(),
        expected_import_provenance
    );
    assert_eq!(
        restored.node_classifications(restored_key).unwrap(),
        expected_classifications
    );
    assert_eq!(
        restored.frame_layout_hints(restored_key).unwrap(),
        expected_frame_layout_hints
    );
    assert_eq!(
        restored.frame_split_offer_suppressed(restored_key),
        Some(true)
    );
    assert_eq!(
        restored.node_properties(restored_key).unwrap(),
        expected_properties
    );
    assert_eq!(
        restored.node_derivations(restored_key).unwrap(),
        expected_derivations
    );
    assert_eq!(
        restored
            .node_last_visited(restored_key)
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        42
    );
    assert_eq!(restored.node_last_session_visited(restored_key), Some(7));

    let canonical = restored.to_snapshot();
    let migrated = canonical.nodes.first().unwrap();
    assert!(!migrated.is_pinned);
    assert_eq!(migrated.tag_presentation, Default::default());
    assert!(migrated.import_provenance.is_empty());
    assert!(migrated.classifications.is_empty());
    assert!(migrated.frame_layout_hints.is_empty());
    assert!(!migrated.frame_split_offer_suppressed);
    assert!(migrated.properties.is_empty());
    assert!(migrated.derivations.is_empty());
    assert_eq!(migrated.last_session_visited, 0);
    assert_eq!(
        migrated
            .session_state
            .as_ref()
            .and_then(|state| state.last_visited_ms),
        None
    );
}

#[test]
fn test_snapshot_roundtrips_fields_and_couplings() {
    let mut graph = Graph::new();
    graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));

    let fid = FieldId::from_uuid(Uuid::from_u128(0xF1));
    graph.add_field(
        Field::new(
            fid,
            FieldDefinition::Scalar(ScalarField::gaussian_at(1.0, 2.0, 10.0)),
        )
        .with_name("focus")
        .with_extent(FieldExtent::Region {
            min_x: -1.0,
            min_y: -2.0,
            max_x: 3.0,
            max_y: 4.0,
        }),
    );
    // A retired field must keep its definition through the round-trip.
    let retired = FieldId::from_uuid(Uuid::from_u128(0xF2));
    graph.add_field(Field::new(
        retired,
        FieldDefinition::Scalar(ScalarField::Const(1.0)),
    ));
    assert!(graph.retire_field(retired));

    let cid = CouplingId::from_uuid(Uuid::from_u128(0xC1));
    graph.add_coupling(Coupling::new(
        cid,
        fid,
        NodeSelector::Kind("paper".into()),
        CouplingResponse::DampenInside { factor: 0.4 },
        2.5,
    ));

    let restored = Graph::from_snapshot(&graph.to_snapshot());

    let f = restored.field(fid).expect("field restored");
    assert_eq!(f.name.as_deref(), Some("focus"));
    assert_eq!(
        f.extent,
        FieldExtent::Region {
            min_x: -1.0,
            min_y: -2.0,
            max_x: 3.0,
            max_y: 4.0
        }
    );
    assert!(matches!(
        f.definition,
        FieldDefinition::Scalar(ScalarField::Gaussian { .. })
    ));
    assert!(f.is_active());

    let rf = restored.field(retired).expect("retired field restored");
    assert!(!rf.is_active(), "retired lifecycle survives");
    assert!(matches!(
        rf.definition,
        FieldDefinition::Scalar(ScalarField::Const(_))
    ));

    let c = restored
        .couplings_for_field(fid)
        .next()
        .expect("coupling restored");
    assert_eq!(c.id, cid);
    assert_eq!(c.selector, NodeSelector::Kind("paper".into()));
    assert_eq!(c.response, CouplingResponse::DampenInside { factor: 0.4 });
    assert_eq!(c.strength, 2.5);
}

#[test]
fn test_snapshot_roundtrips_open_coupling_response() {
    // The open response tail (a non-force family carried by IRI) persists faithfully.
    let mut graph = Graph::new();
    graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
    let fid = FieldId::from_uuid(Uuid::from_u128(0xF3));
    graph.add_field(Field::new(
        fid,
        FieldDefinition::Scalar(ScalarField::Const(1.0)),
    ));
    let iri = format!("{COUPLING_VOCAB}visual/highlight");
    graph.add_coupling(Coupling::new(
        CouplingId::from_uuid(Uuid::from_u128(0xC2)),
        fid,
        NodeSelector::All,
        CouplingResponse::open(iri.clone()),
        1.0,
    ));

    let restored = Graph::from_snapshot(&graph.to_snapshot());
    let c = restored
        .couplings_for_field(fid)
        .next()
        .expect("open coupling restored");
    assert_eq!(
        c.response,
        CouplingResponse::Open {
            predicate: iri.clone()
        }
    );
    assert_eq!(c.response.predicate(), Some(iri.as_str()));
}
