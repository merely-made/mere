// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn four_formats_preserve_the_complete_assertion_profile() {
    let graph = profile_graph();
    let expected = normalized(&graph);
    assert_eq!(
        graph.node_count(),
        2,
        "resource-only subjects have no surfaces"
    );
    assert!(
        expected
            .iter()
            .any(|quad| quad.contains("vocab#A") && quad.contains("vocab#B"))
    );
    let expanded = crate::to_jsonld(&graph);
    let compact = crate::to_jsonld_compact(&graph);
    let expanded_text = serde_json::to_vec(&expanded).unwrap();
    let compact_text = serde_json::to_vec(&compact).unwrap();
    assert!(
        !String::from_utf8(expanded_text.clone())
            .unwrap()
            .contains(RDF_REIFIES)
    );
    assert!(
        String::from_utf8(expanded_text.clone())
            .unwrap()
            .contains(RDF_STATEMENT)
    );
    assert!(
        expanded
            .as_array()
            .unwrap()
            .iter()
            .any(|node| { node["@id"] == crate::GRAPH_SCOPE_USER && node["@graph"].is_array() })
    );

    let contributions = [
        crate::from_jsonld(&expanded_text).unwrap(),
        crate::from_jsonld(&compact_text).unwrap(),
        crate::from_nquads(&crate::to_nquads(&graph), "formats").unwrap(),
        crate::from_trig(&crate::to_trig(&graph).unwrap(), "formats").unwrap(),
    ];
    for (index, contribution) in contributions.iter().enumerate() {
        assert_eq!(
            contribution.edges.len(),
            9,
            "format {index}: independent edges"
        );
        assert_eq!(
            contribution
                .nodes
                .iter()
                .flat_map(|node| &node.properties)
                .count(),
            3
        );
        let mut restored = Graph::new();
        let outcome = crate::apply_contribution(&mut restored, contribution);
        assert_eq!(outcome.edges_skipped, 0);
        assert_eq!(outcome.edges_asserted, 9);
        assert_eq!(
            normalized(&restored),
            expected,
            "format {index}: exact profile"
        );
        let before = normalized(&restored);
        crate::apply_contribution(&mut restored, contribution);
        assert_eq!(
            normalized(&restored),
            before,
            "format {index}: same assertions are idempotent"
        );
        #[cfg(feature = "query")]
        {
            let query = format!(
                "SELECT ?id ?author ?time WHERE {{ GRAPH <{}> {{ ?id <{}> ?triple ; <{}> ?author ; <{}> ?time }} }}",
                crate::GRAPH_SCOPE_USER,
                RDF_REIFIES,
                crate::PROV_WAS_ATTRIBUTED_TO,
                crate::PROV_GENERATED_AT_TIME,
            );
            let mut expected_rows = crate::query::sparql(&graph, &query).unwrap().rows;
            let mut rows = crate::query::sparql(&restored, &query).unwrap().rows;
            assert_eq!(
                expected_rows.len(),
                5,
                "User graph exposes all edge/literal reifiers"
            );
            expected_rows.sort();
            rows.sort();
            assert_eq!(
                rows, expected_rows,
                "format {index}: borrowed query sees exact metadata"
            );
        }
    }
}

#[test]
fn two_classic_literal_reifiers_retain_ids_and_sources() {
    let object: Term = oxrdf::Literal::new_language_tagged_literal("bonjour", "fr")
        .unwrap()
        .into();
    let base = quad(
        "https://s.test/",
        "https://p.test/",
        object.clone(),
        GraphName::DefaultGraph,
    );
    let mut input = vec![base];
    for (id, author) in [
        ("literal one", "https://a.test/"),
        ("literal\ntwo", "https://b.test/"),
    ] {
        input.extend(classic_record(
            id,
            "https://s.test/",
            "https://p.test/",
            object.clone(),
            GraphName::DefaultGraph,
        ));
        input.push(quad(
            &statement_reifier_id(id),
            crate::PROV_WAS_ATTRIBUTED_TO,
            NamedNode::new(author).unwrap().into(),
            GraphName::DefaultGraph,
        ));
    }
    let contribution = from_quads(input, "literal").unwrap();
    let properties = &contribution
        .nodes
        .iter()
        .find(|node| node.id == "https://s.test/")
        .unwrap()
        .properties;
    assert_eq!(properties.len(), 2);
    let exact: HashSet<_> = properties
        .iter()
        .map(|property| {
            (
                property.statement_id.as_str(),
                property.provenance_iri.as_deref(),
                property.lang.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        exact,
        HashSet::from([
            ("literal one", Some("https://a.test/"), Some("fr")),
            ("literal\ntwo", Some("https://b.test/"), Some("fr")),
        ])
    );
}

#[test]
fn complete_blank_node_records_keep_document_scoped_identity() {
    let subject = oxrdf::BlankNode::new("s").unwrap();
    let object = oxrdf::BlankNode::new("o").unwrap();
    let wrapper = oxrdf::BlankNode::new("record").unwrap();
    let graph = GraphName::from(oxrdf::BlankNode::new("scope").unwrap());
    let predicate = NamedNode::new("https://p.test/").unwrap();
    let base = Quad::new(
        subject.clone(),
        predicate.clone(),
        object.clone(),
        graph.clone(),
    );
    let mut input = vec![base];
    for (predicate, object) in [
        (
            RDF_TYPE,
            Term::from(NamedNode::new_unchecked(RDF_STATEMENT)),
        ),
        (RDF_SUBJECT, Term::from(subject)),
        (RDF_PREDICATE, Term::from(predicate)),
        (RDF_OBJECT, Term::from(object)),
    ] {
        input.push(Quad::new(
            wrapper.clone(),
            NamedNode::new_unchecked(predicate),
            object,
            graph.clone(),
        ));
    }
    let contribution = from_quads(input.clone(), "doc-A").unwrap();
    assert_eq!(contribution.edges.len(), 1);
    let edge = &contribution.edges[0];
    assert_eq!(edge.subject, "urn:mere:bnode:doc-A:s");
    assert_eq!(edge.object, "urn:mere:bnode:doc-A:o");
    assert_eq!(
        edge.graph_scope,
        GraphScope::Custom("urn:mere:bnode:doc-A:scope".into())
    );
    assert!(
        edge.statement_id.is_none(),
        "foreign blank reifier has no carried handle"
    );
    assert!(
        !contribution
            .nodes
            .iter()
            .any(|node| node.id.ends_with(":record"))
    );
    let other = from_quads(input, "doc-B").unwrap();
    assert_ne!(other.edges[0].subject, edge.subject);
    assert_eq!(other.edges[0].predicate, edge.predicate);
}

#[test]
fn nonclassic_reifiers_keep_existing_known_field_attachment() {
    let object: Term = NamedNode::new("https://o.test/").unwrap().into();
    let wrapper = statement_reifier_id("legacy metadata");
    let graph = GraphName::from(NamedNode::new("https://annotation.test/graph").unwrap());
    let input = vec![
        quad(
            "https://s.test/",
            "https://p.test/",
            object.clone(),
            GraphName::DefaultGraph,
        ),
        quad(
            &wrapper,
            RDF_REIFIES,
            Triple::new(
                NamedNode::new("https://s.test/").unwrap(),
                NamedNode::new("https://p.test/").unwrap(),
                object,
            )
            .into(),
            GraphName::DefaultGraph,
        ),
        quad(
            &wrapper,
            crate::RDFS_LABEL,
            oxrdf::Literal::new_language_tagged_literal("bonjour", "fr")
                .unwrap()
                .into(),
            graph.clone(),
        ),
        quad(
            &wrapper,
            crate::PROV_GENERATED_AT_TIME,
            oxrdf::Literal::new_simple_literal("1970-01-01T00:00:00Z").into(),
            graph.clone(),
        ),
        quad(
            &wrapper,
            crate::PROV_WAS_ATTRIBUTED_TO,
            NamedNode::new("https://author.test/").unwrap().into(),
            graph,
        ),
        quad(
            &wrapper,
            "https://annotation.test/note",
            oxrdf::Literal::new_simple_literal("retained").into(),
            GraphName::DefaultGraph,
        ),
    ];
    let contribution = from_quads(input, "legacy").unwrap();
    assert_eq!(contribution.edges.len(), 1);
    let claim = &contribution.edges[0];
    assert_eq!(claim.statement_id.as_deref(), Some("legacy metadata"));
    assert_eq!(claim.label.as_deref(), Some("bonjour"));
    assert_eq!(claim.asserted_at_ms, Some(0));
    assert_eq!(claim.graph_scope, GraphScope::Default);
    assert_eq!(
        claim.provenance_iri.as_deref(),
        Some("https://author.test/")
    );
    assert!(
        contribution
            .nodes
            .iter()
            .find(|node| node.id == wrapper)
            .unwrap()
            .properties
            .iter()
            .any(
                |property| property.predicate == "https://annotation.test/note"
                    && property.value == "retained"
            )
    );
}
