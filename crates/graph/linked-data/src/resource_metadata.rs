// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! RDF-profile metadata for each shared Resource, independent of its surfaces.

use crate::{
    ClassificationScheme, Graph, GraphScope, Literal, NamedNode, Quad, RDF_TYPE, Term,
    property_literal, push_quad, push_statement_metadata_quads,
};

const SKOS_CONCEPT: &str = "http://www.w3.org/2004/02/skos/core#Concept";
const SKOS_PREF_LABEL: &str = "http://www.w3.org/2004/02/skos/core#prefLabel";

pub(crate) fn quads(graph: &Graph, resource: &kernel::graph::ResourceNode) -> Vec<Quad> {
    let mut quads = Vec::new();
    let Ok(subject) = NamedNode::new(resource.canonical_iri()) else {
        return quads;
    };
    if let Some(concept) = graph.resource_tag_concept(resource.id()) {
        push_quad(
            &mut quads,
            &subject,
            RDF_TYPE,
            NamedNode::new_unchecked(SKOS_CONCEPT).into(),
            &GraphScope::Default,
        );
        push_quad(
            &mut quads,
            &subject,
            SKOS_PREF_LABEL,
            Literal::new_simple_literal(concept.label).into(),
            &GraphScope::Default,
        );
        if let Ok(owner) = NamedNode::new(concept.owner_iri) {
            push_quad(
                &mut quads,
                &subject,
                crate::PROV_WAS_ATTRIBUTED_TO,
                owner.into(),
                &GraphScope::Default,
            );
        }
    }
    for variant in graph.resource_classification_variants(resource.id()) {
        let record = variant.classification;
        if record.status.is_affirmative()
            && matches!(&record.scheme, ClassificationScheme::Custom(scheme) if scheme == "rdf:type")
            && let Ok(value) = NamedNode::new(record.value)
        {
            push_quad(
                &mut quads,
                &subject,
                RDF_TYPE,
                value.into(),
                &GraphScope::Default,
            );
        }
    }
    for property in graph.resource_properties(resource.id()) {
        let literal: Term = property_literal(&property).into();
        push_quad(
            &mut quads,
            &subject,
            &property.predicate,
            literal.clone(),
            &property.graph_scope,
        );
        push_statement_metadata_quads(
            &mut quads,
            &subject,
            &property.statement_id,
            &property.predicate,
            literal,
            &property.graph_scope,
            None,
            property.provenance_iri.as_deref(),
            property.asserted_at_ms,
        );
    }
    let mut seen = std::collections::HashSet::new();
    quads.retain(|quad| seen.insert(quad.clone()));
    quads
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RDF_REIFIES, apply_contribution, apply_import, dataset_quads, from_jsonld,
        from_jsonld_envelope, from_nquads_envelope, from_trig_envelope, to_jsonld,
        to_jsonld_compact, to_nquads, to_trig,
    };
    use kernel::graph::apply::{GraphDelta, add_node, apply_graph_delta};
    use kernel::graph::{Author, ResourceNode};
    use kernel::types::{
        ClassificationProvenance, ClassificationStatus, NodeClassification, NodeProperty,
    };
    use oxrdf::GraphName;

    fn fixture() -> Graph {
        let mut graph = Graph::new();
        let a = add_node(
            &mut graph,
            None,
            "https://EXAMPLE.test/page#one".into(),
            Default::default(),
        );
        let b = add_node(
            &mut graph,
            None,
            "https://example.test/page#two".into(),
            Default::default(),
        );
        graph.write_as(Author::person("urn:author:alice"), |graph| {
            assert!(graph.insert_node_tags(a, vec!["Cat".into()]))
        });
        let id = graph.shown_resource_id(a).unwrap();
        let mut incoming = Vec::new();
        for (handle, source, time) in [
            ("literal\nA", "urn:author:alice", 0),
            ("literal B", "urn:author:bob", 7),
        ] {
            let mut property = NodeProperty::new("urn:predicate:greeting".into(), "bonjour".into());
            property.statement_id = handle.into();
            property.lang = Some("fr".into());
            property.provenance_iri = Some(source.into());
            property.asserted_at_ms = Some(time);
            incoming.push(property);
        }
        let mut date = NodeProperty::new("urn:predicate:date".into(), "2026-10-08".into())
            .with_graph_scope(GraphScope::User);
        date.statement_id = "date\0ID".into();
        date.datatype = Some("http://www.w3.org/2001/XMLSchema#date".into());
        date.provenance_iri = Some("urn:author:alice".into());
        date.asserted_at_ms = Some(9);
        incoming.push(date);
        assert!(graph.append_resource_properties(id, incoming).unwrap());
        for (label, status) in [
            ("Accepted", ClassificationStatus::Accepted),
            ("Verified", ClassificationStatus::Verified),
            ("Imported", ClassificationStatus::Imported),
            ("Suggested", ClassificationStatus::Suggested),
            ("Rejected", ClassificationStatus::Rejected),
        ] {
            assert!(graph.add_node_classifications(
                b,
                vec![NodeClassification {
                    scheme: ClassificationScheme::Custom("rdf:type".into()),
                    value: format!("urn:type:{label}"),
                    label: Some(label.into()),
                    confidence: 0.5,
                    provenance: ClassificationProvenance::Imported,
                    status,
                    primary: false
                }]
            ));
        }
        graph
    }

    #[test]
    fn resource_metadata_canonical_subjects_affirmative_types_and_reifiers() {
        let graph = fixture();
        let id = ResourceNode::new("https://example.test/page").id();
        assert_eq!(graph.resource_classification_variants(id).len(), 5);
        let quads = dataset_quads(&graph);
        for label in ["Accepted", "Verified", "Imported"] {
            assert!(quads.iter().any(|quad| quad.subject.to_string()
                == "<https://example.test/page>"
                && quad.predicate.as_str() == RDF_TYPE
                && quad.object
                    == Term::NamedNode(NamedNode::new(format!("urn:type:{label}")).unwrap())));
        }
        for label in ["Suggested", "Rejected"] {
            assert!(!quads.iter().any(|quad| quad.object
                == Term::NamedNode(NamedNode::new(format!("urn:type:{label}")).unwrap())));
        }
        let literals: Vec<_> = quads
            .iter()
            .filter(|quad| quad.predicate.as_str() == "urn:predicate:greeting")
            .collect();
        // RDF contains one base quad, while two independently attributable
        // literal assertions retain their separate reifiers.
        assert_eq!(literals.len(), 1);
        let greetings: Vec<_> = graph
            .resource_properties(id)
            .into_iter()
            .filter(|property| property.predicate == "urn:predicate:greeting")
            .collect();
        assert_eq!(greetings.len(), 2);
        assert_ne!(greetings[0].statement_id, greetings[1].statement_id);
        assert_ne!(greetings[0].provenance_iri, greetings[1].provenance_iri);
        assert!(
            literals
                .iter()
                .all(|quad| quad.subject.to_string() == "<https://example.test/page>")
        );
        assert_eq!(
            quads
                .iter()
                .filter(|quad| quad.predicate.as_str() == RDF_REIFIES)
                .count(),
            4
        );
        assert!(
            quads
                .iter()
                .any(|quad| quad.predicate.as_str() == SKOS_PREF_LABEL
                    && quad.object == Literal::new_simple_literal("Cat").into())
        );
        #[cfg(feature = "query")]
        {
            let rows = crate::query::sparql(
                &graph,
                "SELECT ?s WHERE { ?s <urn:predicate:greeting> ?v } ORDER BY ?s",
            )
            .unwrap();
            assert_eq!(rows.rows.len(), 1);
            assert_eq!(
                rows.rows[0][0].as_deref(),
                Some("https://example.test/page")
            );
            let rows=crate::query::sparql(&graph,"SELECT ?r WHERE { { ?r <http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies> ?t } UNION { GRAPH ?g { ?r <http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies> ?t } } }").unwrap();
            assert_eq!(rows.rows.len(), 4);
        }
    }

    #[test]
    fn resource_metadata_four_formats_keep_assertions_and_resource_only_tags() {
        let graph = fixture();
        let id = ResourceNode::new("https://example.test/page").id();
        let expected = graph.resource_properties(id);
        let mut snapshot = graph.to_snapshot();
        snapshot.nodes.clear();
        snapshot.edges.clear();
        snapshot.shown_resources.clear();
        let graph = Graph::try_from_snapshot(&snapshot).unwrap();
        assert_eq!(graph.nodes().count(), 0);
        let normalized = |graph: &Graph| {
            let mut lines: Vec<_> = dataset_quads(graph)
                .into_iter()
                .map(|quad| format!("{quad} ."))
                .collect();
            lines.sort();
            lines
        };
        let expected_dataset = normalized(&graph);
        let contributions = [
            from_jsonld_envelope(
                serde_json::to_string(&to_jsonld(&graph))
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap(),
            from_jsonld_envelope(
                serde_json::to_string(&to_jsonld_compact(&graph))
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap(),
            from_nquads_envelope(&to_nquads(&graph), "resource-profile").unwrap(),
            from_trig_envelope(&to_trig(&graph).unwrap(), "resource-profile").unwrap(),
        ];
        for (format, contribution) in ["expanded JSON-LD", "compact JSON-LD", "N-Quads", "TriG"]
            .into_iter()
            .zip(contributions)
        {
            let mut reingested = Graph::new();
            let outcome = apply_import(&mut reingested, &contribution);
            assert_eq!(outcome.edges_skipped, 0);
            let actual = reingested.resource_properties(id);
            for property in &expected {
                assert!(
                    actual.iter().any(|held| held == property),
                    "missing exact Resource literal {property:?}"
                );
            }
            assert!(reingested.resource_tag_labels(id).contains("Cat"));
            let original_tag: Vec<_> = graph
                .resource_edges()
                .filter(|(from, _, _)| from.id() == id)
                .flat_map(|(_, _, payload)| payload.semantic_statements())
                .collect();
            for statement in original_tag {
                assert_eq!(
                    reingested
                        .find_semantic_statement(&statement.statement_id)
                        .unwrap()
                        .1,
                    statement
                );
            }
            assert_eq!(
                normalized(&reingested),
                expected_dataset,
                "{format} must preserve the complete Resource RDF dataset, including exact reifiers"
            );
            let before = reingested.resource_properties(id);
            apply_import(&mut reingested, &contribution);
            assert_eq!(reingested.resource_properties(id), before);
        }
    }

    #[test]
    fn resource_metadata_partial_skos_stays_ordinary_with_complete_owner_control() {
        let text = r#"[
          {"@id":"urn:tag:foreign","@type":["http://www.w3.org/2004/02/skos/core#Concept"],"http://www.w3.org/2004/02/skos/core#prefLabel":[{"@value":"Foreign"}]},
          {"@id":"urn:tag:complete","@type":["http://www.w3.org/2004/02/skos/core#Concept"],"http://www.w3.org/2004/02/skos/core#prefLabel":[{"@value":"Complete"}],"http://www.w3.org/ns/prov#wasAttributedTo":[{"@id":"urn:owner:foreign"}]}
        ]"#;
        let contribution = from_jsonld(text.as_bytes()).unwrap();
        let mut graph = Graph::new();
        apply_contribution(&mut graph, &contribution);
        let foreign = ResourceNode::for_term("urn:tag:foreign").id();
        let complete = ResourceNode::for_term("urn:tag:complete").id();
        assert!(graph.resource_tag_concept(foreign).is_none());
        assert_eq!(graph.resource_properties(foreign)[0].value, "Foreign");
        assert_eq!(
            graph.resource_tag_concept(complete).unwrap().owner_iri,
            "urn:owner:foreign"
        );
        assert_eq!(graph.resource_properties(complete)[0].value, "Complete");
        let before = graph.to_snapshot();
        apply_graph_delta(
            &mut graph,
            GraphDelta::ReplaySetResourceRecordById {
                resource_id: foreign,
                record: None,
            },
        );
        assert_eq!(graph.to_snapshot().resources, before.resources); // still shown, deletion refused
    }

    #[test]
    fn envelope_keeps_paired_definition_and_literal_assertions_with_exact_handles() {
        let subject = NamedNode::new_unchecked("urn:concept:paired");
        let owner = NamedNode::new_unchecked("urn:owner:paired");
        let mut quads = Vec::new();
        for (predicate, object) in [
            (RDF_TYPE, NamedNode::new_unchecked(SKOS_CONCEPT).into()),
            (
                SKOS_PREF_LABEL,
                Literal::new_simple_literal("Paired").into(),
            ),
            (crate::PROV_WAS_ATTRIBUTED_TO, owner.clone().into()),
            (
                "urn:predicate:ordinary",
                Literal::new_simple_literal("value").into(),
            ),
        ] {
            push_quad(
                &mut quads,
                &subject,
                predicate,
                object,
                &GraphScope::Default,
            );
        }
        let pairs: [(&str, Term, [&str; 2]); 3] = [
            (
                SKOS_PREF_LABEL,
                Literal::new_simple_literal("Paired").into(),
                ["", "label\nopaque"],
            ),
            (
                crate::PROV_WAS_ATTRIBUTED_TO,
                owner.into(),
                ["owner\0opaque", "00000000-0000-0000-0000-000000000001"],
            ),
            (
                "urn:predicate:ordinary",
                Literal::new_simple_literal("value").into(),
                ["ordinary opaque", "00000000-0000-0000-0000-000000000002"],
            ),
        ];
        for (predicate, object, handles) in &pairs {
            for handle in handles {
                push_statement_metadata_quads(
                    &mut quads,
                    &subject,
                    handle,
                    predicate,
                    object.clone(),
                    &GraphScope::Default,
                    None,
                    None,
                    None,
                );
            }
        }
        let envelope = crate::from_quads_envelope(quads.clone(), "paired").unwrap();
        let mut graph = Graph::new();
        assert_eq!(apply_import(&mut graph, &envelope).edges_skipped, 0);
        let id = ResourceNode::for_term(subject.as_str()).id();
        let concept = graph.resource_tag_concept(id).unwrap();
        assert_eq!(concept.label, "Paired");
        assert_eq!(concept.owner_iri, "urn:owner:paired");
        let properties = graph.resource_properties(id);
        assert_eq!(properties.len(), 4);
        for (predicate, object, handles) in &pairs {
            for handle in handles {
                if let Term::Literal(literal) = object {
                    let property = properties
                        .iter()
                        .find(|p| p.statement_id == *handle)
                        .unwrap();
                    assert_eq!(property.predicate, *predicate);
                    assert_eq!(property.value, literal.value());
                    assert_eq!(property.graph_scope, GraphScope::Default);
                    assert_eq!(property.provenance_iri, None);
                    assert_eq!(property.asserted_at_ms, None);
                } else {
                    let held = graph.find_semantic_statement(handle).unwrap().1;
                    assert_eq!(held.predicate, *predicate);
                    assert_eq!(held.graph_scope, GraphScope::Default);
                    assert_eq!(held.provenance_iri, None);
                    assert_eq!(held.asserted_at_ms, None);
                }
            }
        }
        let normalized = |quads: Vec<Quad>| {
            quads
                .into_iter()
                .map(|q| q.to_string())
                .collect::<std::collections::BTreeSet<_>>()
        };
        assert_eq!(normalized(dataset_quads(&graph)), normalized(quads));
        let resources = graph.to_snapshot().resources;
        let edges = graph.to_snapshot().resource_edges;
        for imported in [
            from_jsonld_envelope(
                serde_json::to_string(&to_jsonld(&graph))
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap(),
            from_jsonld_envelope(
                serde_json::to_string(&to_jsonld_compact(&graph))
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap(),
            from_nquads_envelope(&to_nquads(&graph), "paired").unwrap(),
            from_trig_envelope(&to_trig(&graph).unwrap(), "paired").unwrap(),
        ] {
            let mut restored = Graph::new();
            apply_import(&mut restored, &imported);
            assert_eq!(restored.resource_properties(id), properties);
            assert_eq!(restored.to_snapshot().resources, resources);
            assert_eq!(restored.to_snapshot().resource_edges, edges);
            assert_eq!(
                normalized(dataset_quads(&restored)),
                normalized(dataset_quads(&graph))
            );
            let before = restored.to_snapshot();
            apply_import(&mut restored, &imported);
            let mut after = restored.to_snapshot();
            after.timestamp_secs = before.timestamp_secs;
            assert_eq!(
                serde_json::to_value(after).unwrap(),
                serde_json::to_value(before).unwrap()
            );
        }
    }

    #[test]
    fn envelope_foreign_definition_controls_keep_partial_typed_and_scoped_claims() {
        let cases = [
            ("complete", r#""Label""#, vec!["urn:owner:one"], true, false),
            ("partial", r#""Label""#, vec![], false, false),
            (
                "multiple-owner",
                r#""Label""#,
                vec!["urn:owner:one", "urn:owner:two"],
                false,
                false,
            ),
            (
                "multiple-label",
                r#""Label", "Other""#,
                vec!["urn:owner:one"],
                false,
                false,
            ),
            (
                "language",
                r#""Label"@en"#,
                vec!["urn:owner:one"],
                false,
                false,
            ),
            (
                "typed",
                r#""Label"^^<urn:datatype:label>"#,
                vec!["urn:owner:one"],
                false,
                false,
            ),
            ("scoped", r#""Label""#, vec!["urn:owner:one"], false, true),
        ];
        for (name, labels, owners, complete, scoped) in cases {
            let subject = format!("urn:concept:{name}");
            let mut text = format!("<{subject}> a <{SKOS_CONCEPT}> ; <{SKOS_PREF_LABEL}> {labels}");
            for (index, owner) in owners.iter().enumerate() {
                text.push_str(&format!(
                    " {} <{owner}>",
                    if index == 0 {
                        format!("; <{}>", crate::PROV_WAS_ATTRIBUTED_TO)
                    } else {
                        ",".into()
                    }
                ));
            }
            text.push_str(" .");
            if scoped {
                text = format!("GRAPH <urn:scope:foreign> {{ {text} }}");
            }
            let envelope = from_trig_envelope(&text, name).unwrap();
            let mut graph = Graph::new();
            apply_import(&mut graph, &envelope);
            let id = ResourceNode::for_term(&subject).id();
            assert_eq!(graph.resource_tag_concept(id).is_some(), complete, "{name}");
            if complete {
                assert!(graph.resource_properties(id).is_empty());
                assert!(graph.resource_edges().next().is_none());
            } else {
                let properties = graph.resource_properties(id);
                assert_eq!(
                    properties.len(),
                    if name == "multiple-label" { 2 } else { 1 },
                    "{name}"
                );
                let quads = dataset_quads(&graph);
                assert!(
                    quads.iter().any(|q| q.predicate.as_str() == RDF_TYPE
                        && q.object == Term::NamedNode(NamedNode::new_unchecked(SKOS_CONCEPT))
                        && q.graph_name
                            == if scoped {
                                GraphName::NamedNode(NamedNode::new_unchecked("urn:scope:foreign"))
                            } else {
                                GraphName::DefaultGraph
                            }),
                    "{name}"
                );
                if name == "language" {
                    assert_eq!(properties[0].lang.as_deref(), Some("en"));
                }
                if name == "typed" {
                    assert_eq!(
                        properties[0].datatype.as_deref(),
                        Some("urn:datatype:label")
                    );
                }
                let imported = from_nquads_envelope(&to_nquads(&graph), name).unwrap();
                let mut restored = Graph::new();
                apply_import(&mut restored, &imported);
                let normalize = |g: &Graph| {
                    dataset_quads(g)
                        .into_iter()
                        .map(|q| q.to_string())
                        .collect::<std::collections::BTreeSet<_>>()
                };
                assert_eq!(normalize(&restored), normalize(&graph), "{name}");
                assert!(restored.resource_tag_concept(id).is_none(), "{name}");
            }
        }
    }
}

#[cfg(test)]
mod attribution_tests {
    use crate::{apply_contribution, from_jsonld};
    use kernel::graph::resource::TAGGED_WITH_IRI;
    use kernel::graph::resource_tags::tag_concept_iri;
    use kernel::graph::{Author, Graph, ResourceNode};
    #[test]
    fn resource_metadata_foreign_keywords_use_explicit_agent_else_source() {
        let contribution=from_jsonld(br#"[
          {"@id":"https://source.test/page#A","https://schema.org/keywords":[{"@value":"Cat"}]},
          {"@id":"https://source.test/page#B","https://schema.org/keywords":[{"@value":"Cat"}],"http://www.w3.org/ns/prov#wasAttributedTo":[{"@id":"urn:author:explicit"}]}
        ]"#).unwrap();
        let mut graph = Graph::new();
        graph.write_as(Author::person("host"), |graph| {
            apply_contribution(graph, &contribution);
        });
        for (iri, source) in [
            ("https://source.test/page#A", "https://source.test/page#A"),
            ("https://source.test/page#B", "urn:author:explicit"),
        ] {
            let resource = ResourceNode::for_term(iri).id();
            let concept = ResourceNode::for_term(&tag_concept_iri(source, "Cat")).id();
            assert_eq!(
                graph.resource_tag_concept(concept).unwrap().owner_iri,
                source
            );
            let assertions: Vec<_> = graph
                .resource_edges()
                .filter(|(from, to, _)| from.id() == resource && to.id() == concept)
                .flat_map(|(_, _, payload)| payload.semantic_statements())
                .filter(|statement| statement.predicate == TAGGED_WITH_IRI)
                .collect();
            assert_eq!(assertions.len(), 1);
            assert_eq!(assertions[0].provenance_iri.as_deref(), Some(source));
            assert_ne!(
                assertions[0].provenance_iri.as_deref(),
                Some(Author::person("host").asserter_iri().as_str())
            );
        }
        let before = graph.to_snapshot().resource_edges;
        graph.write_as(Author::person("host"), |graph| {
            apply_contribution(graph, &contribution);
        });
        assert_eq!(graph.to_snapshot().resource_edges, before);
    }
}
