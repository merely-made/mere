// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn rdf_subject_identity_is_exact_by_default_and_pages_are_explicit() {
    use crate::graph::ResourceNode;
    use oxrdf::{GraphName, Literal, NamedNode};
    let terms = ["https://vocab.test/terms#A", "https://vocab.test/terms#B"];
    let pages = [
        "https://PAGE.test:443/item#first",
        "https://page.test/item#second",
    ];
    let quads = terms.into_iter().chain(pages).map(|iri| {
        Quad::new(
            NamedNode::new(iri).unwrap(),
            NamedNode::new(SCHEMA_NAME).unwrap(),
            Literal::new_simple_literal(iri),
            GraphName::DefaultGraph,
        )
    });
    let contribution = from_quads(quads, "identity-input").unwrap();
    assert_eq!(contribution.nodes.len(), 4);
    assert!(contribution.edges.is_empty());
    assert!(
        contribution.nodes.iter().all(|node| node.types.is_empty()),
        "foreign subjects carry no entity/page hint"
    );
    let mut graph = Graph::new();
    let outcome = apply_contribution(&mut graph, &contribution);
    assert_eq!(outcome.nodes_created, 4);
    assert_eq!(outcome.edges_skipped, 0);
    let surface = |iri| graph.get_node_by_url(iri).unwrap().0;
    assert_ne!(
        graph.get_node(surface(terms[0])).unwrap().id,
        graph.get_node(surface(terms[1])).unwrap().id
    );
    let shown = |iri| graph.shown_resource_id(surface(iri)).unwrap();
    assert_ne!(
        shown(terms[0]),
        shown(terms[1]),
        "generic RDF preserves vocabulary fragments"
    );
    assert_ne!(
        shown(pages[0]),
        shown(pages[1]),
        "a URL shape alone does not infer a page subject"
    );
    assert_ne!(shown(terms[0]), shown(pages[0]));
    assert_eq!(graph.resource_nodes().count(), 4);
    let exact = terms.map(ResourceNode::for_term);
    assert_ne!(exact[0].id(), exact[1].id());
    assert!(graph.resource(ResourceNode::new(terms[0]).id()).is_none());
    for iri in terms.into_iter().chain(pages) {
        let resource = graph.resource(shown(iri)).unwrap();
        assert_eq!(resource.id(), ResourceNode::for_term(iri).id());
        assert_eq!(resource.canonical_iri(), iri);
        assert_eq!(graph.get_node(surface(iri)).unwrap().url(), iri);
    }
    let restored = Graph::try_from_snapshot(&graph.to_snapshot()).unwrap();
    assert_eq!(restored.resource_nodes().count(), 4);
    for resource in exact {
        assert_eq!(
            restored.resource(resource.id()).unwrap().canonical_iri(),
            resource.canonical_iri(),
            "exact term storage retains both identities"
        );
    }

    let mut mixed = Graph::new();
    let outcome = apply_contribution_with_identity(&mut mixed, &contribution, |node| {
        if pages.contains(&node.id.as_str()) {
            SubjectIdentity::Page
        } else {
            SubjectIdentity::ExactIri
        }
    });
    assert_eq!(outcome.nodes_created, 4);
    assert_eq!(mixed.node_count(), 4);
    assert_eq!(mixed.resource_nodes().count(), 3);
    let shown = |iri| {
        mixed
            .shown_resource_id(mixed.get_node_by_url(iri).unwrap().0)
            .unwrap()
    };
    assert_ne!(shown(terms[0]), shown(terms[1]));
    assert_eq!(shown(pages[0]), shown(pages[1]));
    assert_eq!(shown(pages[0]), ResourceNode::new(pages[0]).id());
    assert_eq!(
        mixed.resource(shown(pages[0])).unwrap().canonical_iri(),
        "https://page.test/item"
    );
    assert!(mixed.resource(ResourceNode::new(terms[0]).id()).is_none());
    for iri in terms {
        assert_eq!(mixed.resource(shown(iri)).unwrap().canonical_iri(), iri);
    }
    let records = mixed.to_snapshot().resources;
    let bindings = mixed.to_snapshot().shown_resources;
    let repeat =
        apply_contribution_with_identity(&mut mixed, &contribution, |_| SubjectIdentity::Page);
    assert_eq!(repeat.nodes_created, 0);
    assert_eq!(mixed.to_snapshot().resources, records);
    assert_eq!(
        mixed.to_snapshot().shown_resources,
        bindings,
        "existing prepared bindings retain their intent"
    );
}

#[test]
fn exact_subject_binding_replays_without_an_implicit_page_resource() {
    use crate::graph::ResourceNode;
    use crate::graph::capture::{CapturedDelta, replay_captured_deltas};
    use std::sync::{Arc, Mutex};

    let iri = "https://vocab.test/terms#Case";
    let contribution = GraphContribution {
        nodes: vec![NodeContribution::new(iri)],
        edges: vec![],
    };
    let mut graph = Graph::new();
    let seen = Arc::new(Mutex::new(Vec::<CapturedDelta>::new()));
    let sink = seen.clone();
    graph.set_recorder(Some(Arc::new(move |delta| {
        sink.lock().unwrap().push(delta.clone())
    })));
    assert_eq!(
        apply_contribution(&mut graph, &contribution).nodes_created,
        1
    );
    let captures = seen.lock().unwrap().clone();
    let surface_id = Graph::node_namespace_id(iri);
    let exact = ResourceNode::for_term(iri);
    assert!(matches!(&captures[..], [
        CapturedDelta::ReplayAddNodeWithIdIfMissing { id, url, .. },
        CapturedDelta::ReplayTouchNodeLastVisitedById { node_id, .. },
        CapturedDelta::ReplaySetResourceRecordById { resource_id, record: Some(record) },
        CapturedDelta::ReplaySetShownResourceById { surface_id: shown_surface, resource_id: Some(shown) },
    ] if id == &surface_id.to_string()
        && url == iri
        && node_id == id
        && resource_id == &exact.id().to_string()
        && record.canonical_iri == iri
        && shown_surface == id
        && shown == resource_id));
    let replayed = replay_captured_deltas(captures);
    assert_eq!(replayed.node_count(), 1);
    assert_eq!(replayed.resource_nodes().count(), 1);
    assert_eq!(replayed.resource(exact.id()).unwrap().canonical_iri(), iri);
    assert!(replayed.resource(ResourceNode::new(iri).id()).is_none());
    assert_eq!(
        replayed.to_snapshot().resources,
        graph.to_snapshot().resources
    );
    assert_eq!(
        replayed.to_snapshot().shown_resources,
        graph.to_snapshot().shown_resources
    );
    assert_eq!(
        replayed.node_last_visited(replayed.get_node_key_by_id(surface_id).unwrap()),
        graph.node_last_visited(graph.get_node_key_by_id(surface_id).unwrap())
    );

    seen.lock().unwrap().clear();
    assert_eq!(
        apply_contribution(&mut graph, &contribution).nodes_created,
        0
    );
    assert!(
        seen.lock().unwrap().is_empty(),
        "same prepared subject is a true no-op"
    );
}

#[test]
fn subject_identity_preserves_existing_resource_metadata_and_shown_binding() {
    use crate::graph::ResourceNode;
    use crate::graph::apply::{GraphDelta, apply_graph_delta};
    use crate::persistence::{PersistedResourceFacet, PersistedResourceRecord};

    let iri = "https://vocab.test/terms#A";
    let prepared = ResourceNode::for_term(iri);
    let record = PersistedResourceRecord {
        canonical_iri: iri.into(),
        facets: vec![PersistedResourceFacet {
            facet: "source.record".into(),
            value_json: r#"{"author":"retained","revision":7}"#.into(),
        }],
    };
    let mut graph = Graph::new();
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplaySetResourceRecordById {
            resource_id: prepared.id(),
            record: Some(record.clone()),
        },
    );
    let surface_id = Graph::node_namespace_id(iri);
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplayAddNodeWithIdIfMissing {
            id: surface_id,
            url: iri.into(),
            position: Default::default(),
        },
    );
    let key = graph.get_node_key_by_id(surface_id).unwrap();
    assert!(graph.shown_resource_id(key).is_none());
    let contribution = GraphContribution {
        nodes: vec![NodeContribution::new(iri)],
        edges: vec![],
    };
    assert_eq!(
        apply_contribution(&mut graph, &contribution).nodes_created,
        0
    );
    assert_eq!(graph.shown_resource_id(key), Some(prepared.id()));
    assert_eq!(
        graph
            .to_snapshot()
            .resources
            .into_iter()
            .find(|resource| resource.canonical_iri == iri),
        Some(record.clone())
    );
    assert_eq!(graph.resource_nodes().count(), 1);
    assert!(graph.resource(ResourceNode::new(iri).id()).is_none());

    let alternate = ResourceNode::for_term("https://prepared.test/#elsewhere");
    let alternate_record = PersistedResourceRecord {
        canonical_iri: alternate.canonical_iri().into(),
        facets: vec![],
    };
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplaySetResourceRecordById {
            resource_id: alternate.id(),
            record: Some(alternate_record.clone()),
        },
    );
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplaySetShownResourceById {
            surface_id,
            resource_id: Some(alternate.id()),
        },
    );
    let before = graph.revision();
    let outcome =
        apply_contribution_with_identity(&mut graph, &contribution, |_| SubjectIdentity::Page);
    assert_eq!(outcome.nodes_created, 0);
    assert_eq!(graph.revision(), before);
    assert_eq!(graph.shown_resource_id(key), Some(alternate.id()));
    let records = graph.to_snapshot().resources;
    assert_eq!(
        records
            .iter()
            .find(|resource| resource.canonical_iri == alternate.canonical_iri()),
        Some(&alternate_record)
    );
    assert_eq!(
        records
            .iter()
            .find(|resource| resource.canonical_iri == iri),
        Some(&record)
    );
    assert_eq!(graph.resource_nodes().count(), 2);
    assert!(graph.resource(ResourceNode::new(iri).id()).is_none());
}
