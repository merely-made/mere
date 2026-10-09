// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn incomplete_multivalued_and_unasserted_descriptions_remain_rdf() {
    let object: Term = NamedNode::new("https://o.test/").unwrap().into();
    let base = quad(
        "https://s.test/",
        "https://p.test/",
        object.clone(),
        GraphName::DefaultGraph,
    );
    for mode in 0..4 {
        let mut input = classic_record(
            "description",
            "https://s.test/",
            "https://p.test/",
            object.clone(),
            GraphName::DefaultGraph,
        );
        if mode != 2 {
            input.push(base.clone());
        }
        match mode {
            0 => input.retain(|quad| quad.predicate.as_str() != RDF_OBJECT),
            1 => input.push(quad(
                &statement_reifier_id("description"),
                RDF_OBJECT,
                NamedNode::new("https://other.test/").unwrap().into(),
                GraphName::DefaultGraph,
            )),
            3 => input.push(quad(
                &statement_reifier_id("description"),
                crate::PROV_WAS_ATTRIBUTED_TO,
                NamedNode::new("https://first.test/").unwrap().into(),
                GraphName::DefaultGraph,
            )),
            _ => {},
        }
        if mode == 3 {
            input.push(quad(
                &statement_reifier_id("description"),
                crate::PROV_WAS_ATTRIBUTED_TO,
                NamedNode::new("https://second.test/").unwrap().into(),
                GraphName::DefaultGraph,
            ));
        }
        assert_eq!(
            bridge_classic_reification(input.clone()),
            input,
            "description {mode} keeps every quad"
        );
        let contribution = from_quads(input, "descriptions").unwrap();
        assert!(
            contribution
                .nodes
                .iter()
                .any(|node| node.id == statement_reifier_id("description"))
        );
        assert!(
            contribution
                .edges
                .iter()
                .all(|edge| edge.statement_id.is_none())
        );
        if mode == 2 {
            assert!(!contribution.edges.iter().any(
                |edge| edge.subject == "https://s.test/" && edge.predicate == "https://p.test/"
            ));
        } else {
            assert!(contribution.edges.iter().any(
                |edge| edge.subject == "https://s.test/" && edge.predicate == "https://p.test/"
            ));
        }
    }
    // A complete asserted record does lift, without becoming another entity.
    let mut positive = classic_record(
        "description",
        "https://s.test/",
        "https://p.test/",
        object,
        GraphName::DefaultGraph,
    );
    positive.push(base);
    let contribution = from_quads(positive, "descriptions").unwrap();
    assert_eq!(contribution.edges.len(), 1);
    assert_eq!(
        contribution.edges[0].statement_id.as_deref(),
        Some("description")
    );
    assert!(
        !contribution
            .nodes
            .iter()
            .any(|node| node.id == statement_reifier_id("description"))
    );
}

#[test]
fn classic_reserved_ids_reject_atomically_with_foreign_controls() {
    let object: Term = NamedNode::new("https://o.test/").unwrap().into();
    let base = quad(
        "https://s.test/",
        "https://p.test/",
        object.clone(),
        GraphName::DefaultGraph,
    );
    for wrapper in [
        "urn:mere:statement-id:v1:1",
        "urn:mere:statement-id:v1:zz",
        "urn:mere:statement-id:v1:ff",
    ] {
        let mut input = classic_record(
            "valid",
            "https://s.test/",
            "https://p.test/",
            object.clone(),
            GraphName::DefaultGraph,
        );
        input.push(base.clone());
        let mut invalid = classic_record(
            "bad",
            "https://s.test/",
            "https://p.test/",
            object.clone(),
            GraphName::DefaultGraph,
        );
        for quad in &mut invalid {
            quad.subject = NamedNode::new(wrapper).unwrap().into();
        }
        input.extend(invalid);
        assert!(
            matches!(from_quads(input, "ids"), Err(IngestError::Parse(error)) if error.contains("invalid assertion ID reifier"))
        );
    }
    for (wrapper, wanted) in [
        ("urn:mere:statement:legacy", Some("legacy")),
        ("urn:mere:statement-id:v1:6c696e650a4944", Some("line\nID")),
        ("urn:mere:statement-id:v2:zz", None),
        ("https://foreign.test/statement", None),
    ] {
        let mut input = classic_record(
            "placeholder",
            "https://s.test/",
            "https://p.test/",
            object.clone(),
            GraphName::DefaultGraph,
        );
        for quad in &mut input {
            quad.subject = NamedNode::new(wrapper).unwrap().into();
        }
        input.push(base.clone());
        let contribution = from_quads(input, "ids").unwrap();
        assert_eq!(contribution.edges.len(), 1);
        assert_eq!(contribution.edges[0].statement_id.as_deref(), wanted);
    }
}

#[test]
fn conflicting_description_and_cross_graph_base_do_not_select_a_claim() {
    let object: Term = NamedNode::new("https://o.test/").unwrap().into();
    let base = quad(
        "https://s.test/",
        "https://p.test/",
        object.clone(),
        GraphName::DefaultGraph,
    );
    let named = GraphName::from(NamedNode::new("https://graph.test/").unwrap());
    let mut wrong_graph = classic_record(
        "wrong graph",
        "https://s.test/",
        "https://p.test/",
        object.clone(),
        named.clone(),
    );
    wrong_graph.push(base.clone());
    assert_eq!(bridge_classic_reification(wrong_graph.clone()), wrong_graph);
    let contribution = from_quads(wrong_graph, "conflicts").unwrap();
    assert!(
        contribution
            .edges
            .iter()
            .all(|edge| edge.statement_id.is_none())
    );
    assert!(
        contribution
            .edges
            .iter()
            .any(|edge| edge.predicate == RDF_SUBJECT
                && edge.graph_scope == GraphScope::Custom("https://graph.test/".into()))
    );

    let mut conflicting = classic_record(
        "conflict",
        "https://s.test/",
        "https://p.test/",
        object.clone(),
        GraphName::DefaultGraph,
    );
    conflicting.push(base.clone());
    conflicting.push(quad(
        &statement_reifier_id("conflict"),
        RDF_REIFIES,
        Triple::new(
            NamedNode::new("https://different.test/").unwrap(),
            NamedNode::new("https://p.test/").unwrap(),
            object.clone(),
        )
        .into(),
        GraphName::DefaultGraph,
    ));
    assert_eq!(bridge_classic_reification(conflicting.clone()), conflicting);
    let contribution = from_quads(conflicting, "conflicts").unwrap();
    assert!(
        contribution
            .edges
            .iter()
            .all(|edge| edge.statement_id.is_none())
    );
    assert!(
        contribution
            .edges
            .iter()
            .any(|edge| edge.predicate == RDF_SUBJECT)
    );
    assert!(
        !contribution
            .edges
            .iter()
            .any(|edge| edge.subject == "https://different.test/")
    );

    // Matching representations of one wrapper are the same assertion.
    let mut matching = classic_record(
        "match",
        "https://s.test/",
        "https://p.test/",
        object.clone(),
        GraphName::DefaultGraph,
    );
    matching.push(base.clone());
    matching.push(quad(
        &statement_reifier_id("match"),
        RDF_REIFIES,
        Triple::new(
            NamedNode::new("https://s.test/").unwrap(),
            NamedNode::new("https://p.test/").unwrap(),
            object.clone(),
        )
        .into(),
        GraphName::DefaultGraph,
    ));
    let contribution = from_quads(matching, "conflicts").unwrap();
    assert_eq!(contribution.edges.len(), 1);
    assert_eq!(contribution.edges[0].statement_id.as_deref(), Some("match"));

    // A complete record plus another incomplete scope stays descriptive.
    let mut multi_scope = classic_record(
        "scopes",
        "https://s.test/",
        "https://p.test/",
        object,
        GraphName::DefaultGraph,
    );
    multi_scope.push(base);
    multi_scope.push(quad(
        &statement_reifier_id("scopes"),
        RDF_TYPE,
        NamedNode::new_unchecked(RDF_STATEMENT).into(),
        named,
    ));
    assert_eq!(bridge_classic_reification(multi_scope.clone()), multi_scope);
    let contribution = from_quads(multi_scope, "conflicts").unwrap();
    assert!(
        contribution
            .edges
            .iter()
            .all(|edge| edge.statement_id.is_none())
    );
}

#[test]
fn reserved_identifier_validation_also_covers_standalone_jsonld_records() {
    for (id, valid) in [
        ("urn:mere:statement-id:v1:zz", false),
        ("urn:mere:statement-id:v1:ff", false),
        ("urn:mere:statement-id:v1:1", false),
        ("urn:mere:statement-id:v1:", true),
        ("urn:mere:statement-id:v2:zz", true),
        ("https://foreign.test/description", true),
    ] {
        let input = serde_json::to_vec(&json!([
            { "@id": "https://positive.test/", "https://schema.org/name": [{ "@value": "positive" }] },
            { "@id": id, "@type": [RDF_STATEMENT], RDF_SUBJECT: [{ "@id": "https://s.test/" }] }
        ])).unwrap();
        let result = crate::from_jsonld(&input);
        assert_eq!(result.is_ok(), valid, "{id}");
        if let Ok(contribution) = result {
            assert!(
                contribution
                    .nodes
                    .iter()
                    .any(|node| node.title.as_deref() == Some("positive"))
            );
            assert!(contribution.nodes.iter().any(|node| node.id == id));
            assert!(
                contribution
                    .edges
                    .iter()
                    .all(|edge| edge.statement_id.is_none())
            );
        }
    }
}

#[test]
fn unsupported_metadata_stays_descriptive_and_extra_annotations_are_retained() {
    let object: Term = NamedNode::new("https://o.test/").unwrap().into();
    let base = quad(
        "https://s.test/",
        "https://p.test/",
        object.clone(),
        GraphName::DefaultGraph,
    );
    let wrapper = statement_reifier_id("metadata");
    for (predicate, value) in [
        (
            crate::PROV_GENERATED_AT_TIME,
            oxrdf::Literal::new_typed_literal(
                "not a date",
                NamedNode::new_unchecked(crate::XSD_DATETIME),
            )
            .into(),
        ),
        (
            crate::PROV_GENERATED_AT_TIME,
            oxrdf::Literal::new_typed_literal(
                "1960-01-01T00:00:00Z",
                NamedNode::new_unchecked(crate::XSD_DATETIME),
            )
            .into(),
        ),
        (
            crate::PROV_WAS_ATTRIBUTED_TO,
            oxrdf::Literal::new_simple_literal("not an agent IRI").into(),
        ),
        (
            crate::RDFS_LABEL,
            NamedNode::new("https://label.test/").unwrap().into(),
        ),
        (
            crate::RDFS_LABEL,
            oxrdf::Literal::new_language_tagged_literal("bonjour", "fr")
                .unwrap()
                .into(),
        ),
    ] {
        let mut input = classic_record(
            "metadata",
            "https://s.test/",
            "https://p.test/",
            object.clone(),
            GraphName::DefaultGraph,
        );
        input.push(base.clone());
        input.push(quad(&wrapper, predicate, value, GraphName::DefaultGraph));
        assert_eq!(bridge_classic_reification(input.clone()), input);
        let contribution = from_quads(input, "metadata").unwrap();
        assert!(contribution.nodes.iter().any(|node| node.id == wrapper));
        assert!(
            contribution
                .edges
                .iter()
                .all(|edge| edge.statement_id.is_none())
        );
        assert!(
            contribution
                .edges
                .iter()
                .any(|edge| edge.predicate == RDF_SUBJECT)
        );
    }
    let mut valid = classic_record(
        "metadata",
        "https://s.test/",
        "https://p.test/",
        object,
        GraphName::DefaultGraph,
    );
    valid.push(base);
    valid.push(quad(
        &wrapper,
        crate::PROV_GENERATED_AT_TIME,
        oxrdf::Literal::new_typed_literal(
            "1970-01-01T00:00:00Z",
            NamedNode::new_unchecked(crate::XSD_DATETIME),
        )
        .into(),
        GraphName::DefaultGraph,
    ));
    valid.push(quad(
        &wrapper,
        crate::RDFS_LABEL,
        oxrdf::Literal::new_simple_literal("label").into(),
        GraphName::DefaultGraph,
    ));
    valid.push(quad(
        &wrapper,
        crate::PROV_WAS_ATTRIBUTED_TO,
        NamedNode::new("https://author.test/").unwrap().into(),
        GraphName::DefaultGraph,
    ));
    valid.push(quad(
        &wrapper,
        "https://annotation.test/evidence",
        oxrdf::Literal::new_language_tagged_literal("preuve", "fr")
            .unwrap()
            .into(),
        GraphName::DefaultGraph,
    ));
    valid.push(quad(
        &wrapper,
        "https://annotation.test/ref",
        NamedNode::new("https://evidence.test/").unwrap().into(),
        GraphName::DefaultGraph,
    ));
    let contribution = from_quads(valid, "metadata").unwrap();
    let claim = contribution
        .edges
        .iter()
        .find(|edge| edge.subject == "https://s.test/")
        .unwrap();
    assert_eq!(claim.statement_id.as_deref(), Some("metadata"));
    assert_eq!(claim.label.as_deref(), Some("label"));
    assert_eq!(
        claim.provenance_iri.as_deref(),
        Some("https://author.test/")
    );
    assert_eq!(claim.asserted_at_ms, Some(0));
    let record = contribution
        .nodes
        .iter()
        .find(|node| node.id == wrapper)
        .unwrap();
    assert!(record.properties.iter().any(|property| property.predicate
        == "https://annotation.test/evidence"
        && property.value == "preuve"
        && property.lang.as_deref() == Some("fr")));
    assert!(contribution.edges.iter().any(|edge| edge.subject == wrapper
        && edge.predicate == "https://annotation.test/ref"
        && edge.object == "https://evidence.test/"));
}
