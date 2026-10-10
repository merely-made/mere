// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::ingest::{IngestError, from_quads};
use crate::reifier::statement_reifier_id;
use kernel::graph::fixtures::GraphFixtures;
use kernel::graph::resource::ResourceNode;
use kernel::graph::{Graph, SemanticStatement};
use kernel::persistence::{PersistedEdge, PersistedEdgeFamily, PersistedResourceRecord};
use kernel::persistence::{PersistedSemanticEdgeData, PersistedSemanticStatement};
use kernel::types::{GraphScope, NodeProperty};

fn quad(subject: &str, predicate: &str, object: Term, graph: GraphName) -> Quad {
    Quad::new(
        NamedNode::new(subject).unwrap(),
        NamedNode::new(predicate).unwrap(),
        object,
        graph,
    )
}

fn classic_record(
    handle: &str,
    base_subject: &str,
    predicate: &str,
    object: Term,
    graph: GraphName,
) -> Vec<Quad> {
    let wrapper = statement_reifier_id(handle);
    [
        (RDF_TYPE, NamedNode::new_unchecked(RDF_STATEMENT).into()),
        (RDF_SUBJECT, NamedNode::new(base_subject).unwrap().into()),
        (RDF_PREDICATE, NamedNode::new(predicate).unwrap().into()),
        (RDF_OBJECT, object),
    ]
    .into_iter()
    .map(|(predicate, object)| quad(&wrapper, predicate, object, graph.clone()))
    .collect()
}

fn normalized(graph: &Graph) -> Vec<String> {
    let mut quads: Vec<_> = crate::dataset_quads(graph)
        .into_iter()
        .map(|quad| quad.to_string())
        .collect();
    quads.sort();
    quads.dedup();
    quads
}

fn profile_graph() -> Graph {
    let mut graph = Graph::new();
    let source = graph.add_node("https://source.test/".into(), Default::default());
    let target = graph.add_node("https://target.test/".into(), Default::default());
    for (index, scope) in [
        GraphScope::Default,
        GraphScope::Source,
        GraphScope::User,
        GraphScope::User,
        GraphScope::Agent,
        GraphScope::Moot,
        GraphScope::Custom("https://scope.test/exact#Case".into()),
    ]
    .into_iter()
    .enumerate()
    {
        assert!(
            graph
                .assert_persisted_semantic_statement(
                    source,
                    target,
                    SemanticStatement {
                        statement_id: [
                            "",
                            "plain",
                            "space id",
                            "line\nID",
                            "nul\0ID",
                            "日本語",
                            "custom"
                        ][index]
                            .into(),
                        predicate: "https://example.test/claims".into(),
                        recognized_sub_kind: None,
                        label: Some(format!("label {index}")),
                        graph_scope: scope,
                        provenance_iri: Some(format!("https://author.test/{index}")),
                        asserted_at_ms: Some(index as u64),
                    },
                )
                .is_some()
        );
    }
    let mut properties = Vec::new();
    for (handle, author, time) in [
        ("literal\nA", "https://author.test/A", 0),
        ("literal B", "https://author.test/B", 9),
    ] {
        let mut property =
            NodeProperty::new("https://example.test/greeting".into(), "bonjour".into());
        property.statement_id = handle.into();
        property.lang = Some("fr".into());
        property.provenance_iri = Some(author.into());
        property.asserted_at_ms = Some(time);
        properties.push(property);
    }
    let mut date = NodeProperty::new("https://example.test/date".into(), "2026-10-07".into())
        .with_graph_scope(GraphScope::User);
    date.statement_id = "typed\0literal".into();
    date.datatype = Some("http://www.w3.org/2001/XMLSchema#date".into());
    date.provenance_iri = Some("https://author.test/A".into());
    date.asserted_at_ms = Some(42);
    properties.push(date);

    // Explicit resources need no browsing surfaces to participate in export.
    let a = ResourceNode::for_term("https://terms.test/vocab#A");
    let b = ResourceNode::for_term("https://terms.test/vocab#B");
    let mut snapshot = graph.to_snapshot();
    snapshot
        .resources
        .extend([&a, &b].map(|resource| PersistedResourceRecord {
            canonical_iri: resource.canonical_iri().into(),
            facets: Vec::new(),
        }));
    snapshot.resource_edges.push(PersistedEdge {
        from_node_id: a.id().to_string(),
        to_node_id: b.id().to_string(),
        families: vec![PersistedEdgeFamily::Semantic],
        semantic: Some(PersistedSemanticEdgeData {
            statements: ["resource one", "resource\n二"]
                .into_iter()
                .enumerate()
                .map(|(index, id)| PersistedSemanticStatement {
                    statement_id: id.into(),
                    predicate: kernel::graph::resource::TAGGED_WITH_IRI.into(),
                    recognized_sub_kind: None,
                    label: Some(format!("resource {index}")),
                    graph_scope: GraphScope::User,
                    provenance_iri: Some(format!("https://tagger.test/{index}")),
                    asserted_at_ms: Some(100 + index as u64),
                })
                .collect(),
            ..Default::default()
        }),
        traversal: None,
        containment: None,
        arrangement: None,
        imported: None,
        provenance: None,
    });
    let mut graph = Graph::try_from_snapshot(&snapshot).unwrap_or_else(|error| panic!("{error}"));
    let source = graph
        .nodes()
        .find(|(_, node)| node.url() == "https://source.test/")
        .map(|(key, _)| key)
        .unwrap();
    // Surface facets live in their own sidecar, rather than GraphSnapshot.
    assert!(graph.append_node_properties(source, properties));
    assert_eq!(graph.node_properties(source).unwrap().len(), 3);
    graph
}

mod descriptions;
mod profile;

#[cfg(feature = "query")]
mod directional_descriptions {
    use super::*;
    use crate::ingest::GraphContribution;
    use oxjsonld::JsonLdParser;
    use oxrdf::BaseDirection;

    fn input(description_objects: Value) -> Vec<u8> {
        serde_json::to_vec(&json!([
            {
                "@id": "urn:subject",
                "urn:predicate": {
                    "@value": "same", "@language": "en", "@direction": "ltr"
                }
            },
            {
                "@id": "urn:foreign:claim",
                "@type": RDF_STATEMENT,
                (RDF_SUBJECT): {"@id": "urn:subject"},
                (RDF_PREDICATE): {"@id": "urn:predicate"},
                (RDF_OBJECT): description_objects,
                (crate::PROV_WAS_ATTRIBUTED_TO): {"@id": "urn:author"}
            }
        ]))
        .unwrap()
    }

    fn asserted_property(contribution: &GraphContribution) -> &NodeProperty {
        let subject = contribution
            .nodes
            .iter()
            .find(|node| node.id == "urn:subject")
            .expect("asserted subject remains present");
        assert_eq!(subject.properties.len(), 1);
        let property = &subject.properties[0];
        assert_eq!(property.predicate, "urn:predicate");
        assert_eq!(property.value, "same");
        assert_eq!(property.lang.as_deref(), Some("en"));
        property
    }

    fn assert_ordinary_description(contribution: &GraphContribution, bytes: &[u8]) {
        let wrapper = contribution
            .nodes
            .iter()
            .find(|node| node.id == "urn:foreign:claim")
            .expect("unpromoted description remains an ordinary RDF subject");
        assert_eq!(wrapper.types, vec![RDF_STATEMENT.to_string()]);
        assert!(wrapper.properties.iter().any(|property| {
            property.predicate == RDF_OBJECT
                && property.value == "same"
                && property.lang.as_deref() == Some("en")
                && property.provenance_iri.is_none()
        }));
        for (predicate, object) in [
            (RDF_SUBJECT, "urn:subject"),
            (RDF_PREDICATE, "urn:predicate"),
            (crate::PROV_WAS_ATTRIBUTED_TO, "urn:author"),
        ] {
            assert!(contribution.edges.iter().any(|edge| {
                edge.subject == "urn:foreign:claim"
                    && edge.predicate == predicate
                    && edge.object == object
                    && edge.statement_id.is_none()
                    && edge.provenance_iri.is_none()
            }));
        }
        // NodeProperty does not store direction; the RDF bridge must still
        // retain the exact original terms rather than promoting this record.
        let quads = JsonLdParser::new()
            .for_slice(bytes)
            .collect::<Result<Vec<_>, _>>()
            .expect("expanded directional JSON-LD parses");
        assert_eq!(bridge_classic_reification(quads.clone()), quads);
    }

    #[test]
    fn feature_compat_jsonld_mismatched_direction_keeps_description_rdf() {
        let bytes = input(json!({
            "@value": "same", "@language": "en", "@direction": "rtl"
        }));
        let contribution = crate::from_jsonld(&bytes).unwrap();
        assert_eq!(asserted_property(&contribution).provenance_iri, None);
        assert_ordinary_description(&contribution, &bytes);
    }

    #[test]
    fn feature_compat_jsonld_matching_direction_promotes_metadata() {
        let bytes = input(json!({
            "@value": "same", "@language": "en", "@direction": "ltr"
        }));
        let contribution = crate::from_jsonld(&bytes).unwrap();
        assert_eq!(
            asserted_property(&contribution).provenance_iri.as_deref(),
            Some("urn:author")
        );
        assert!(
            !contribution
                .nodes
                .iter()
                .any(|node| node.id == "urn:foreign:claim")
        );
        assert!(
            !contribution
                .edges
                .iter()
                .any(|edge| edge.subject == "urn:foreign:claim")
        );
        let quads = JsonLdParser::new()
            .for_slice(&bytes)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let objects = quads
            .iter()
            .filter(|quad| {
                quad.predicate.as_str() == "urn:predicate" || quad.predicate.as_str() == RDF_OBJECT
            })
            .collect::<Vec<_>>();
        assert_eq!(objects.len(), 2);
        for quad in objects {
            let Term::Literal(literal) = &quad.object else {
                panic!("matching-direction controls are literal terms");
            };
            assert_eq!(literal.direction(), Some(BaseDirection::Ltr));
        }
    }

    #[test]
    fn feature_compat_jsonld_opposite_direction_objects_refuse_promotion() {
        let bytes = input(json!([
            {"@value": "same", "@language": "en", "@direction": "ltr"},
            {"@value": "same", "@language": "en", "@direction": "rtl"}
        ]));
        let contribution = crate::from_jsonld(&bytes).unwrap();
        assert_eq!(asserted_property(&contribution).provenance_iri, None);
        assert_ordinary_description(&contribution, &bytes);
        let objects = JsonLdParser::new()
            .for_slice(&bytes)
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .into_iter()
            .filter(|quad| quad.predicate.as_str() == RDF_OBJECT)
            .map(|quad| quad.object)
            .collect::<HashSet<_>>();
        assert_eq!(objects.len(), 2, "directions distinguish RDF object values");
    }
}
