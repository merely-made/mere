// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use crate::graph::resource::TAGGED_WITH_IRI;
use crate::graph::resource_tags::tag_concept_iri;
use crate::graph::{Author, Graph, ResourceNode};
use crate::rdf::apply_contribution;
use mere::from_jsonld;
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
